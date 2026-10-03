# WildChat Prompt Smell Analysis

This tool reads user prompts from the WildChat dataset. It asks an LLM whether each prompt has "prompt smells", which are habits that make a prompt hard to answer well. The result is a JSON file with one record per smell found.

Dataset file used by default:
`https://huggingface.co/datasets/allenai/WildChat/blob/main/data/train-00003-of-00006.parquet`

## What it does

1. Loads the parquet file. It downloads the file once into `data/` and reuses that copy on later runs.
2. Reads `conversation_id`, `model`, `timestamp` and `conversation` from each row.
3. Keeps user messages only. It drops assistant and system messages.
4. Keeps English messages only.
5. Sends each prompt to the LLM on its own. No assistant replies or other turns are sent with it.
6. Checks the LLM answer against a Pydantic schema. If the answer is broken, it asks again.
7. Writes one output record for each smell. A prompt with no smells still gets one record, with `smell_type` and `smell_reason` set to `null`.

## Project layout

```
wild-chat-analysis/
  main.py                   command line entry point
  config.yaml               default settings
  .env.example              template for API settings
  requirements.txt
  app/
    config.py               loads settings from CLI, env, YAML
    dataset_loader.py       downloads and reads the parquet file
    conversation_parser.py  pulls English user prompts out of a row
    llm_client.py           calls the API with retries and rate limits
    prompt_analyzer.py      builds the LLM prompt and checks its answer
    output_writer.py        checkpoint file and final JSON
    pipeline.py             ties the steps together
    models.py               data models
  tests/                    unit tests (no API key needed)
```

## Setup

You need Python 3.10 or newer.

```powershell
cd wild-chat-analysis
python -m venv .venv
.venv\Scripts\activate
python -m pip install -r requirements.txt
```

On macOS or Linux, activate with `source .venv/bin/activate`.

The dataset file is about 300 MB. If you already have `train-00003-of-00006.parquet`, put it in `data/` and the tool will skip the download.

## Configure the API (Groq)

1. Copy the template: `copy .env.example .env` (or `cp .env.example .env`).
2. Open `.env` and set your key:

```
LLM_API_KEY=your-groq-key
LLM_BASE_URL=https://api.groq.com/openai/v1
LLM_MODEL=openai/gpt-oss-120b
```

`.env` is in `.gitignore`. Never commit it. The key is never written to logs.

Any provider with an OpenAI-style `/chat/completions` endpoint works. Change `LLM_BASE_URL` and `LLM_MODEL`. For example, OpenAI uses `https://api.openai.com/v1`, and a local Ollama server uses `http://localhost:11434/v1`. If the provider does not support JSON mode, set `LLM_JSON_MODE=false`.

### Environment variables

| Variable | Meaning | Default |
|---|---|---|
| `LLM_API_KEY` | API key (required for real runs) | none |
| `LLM_BASE_URL` | API base URL | Groq |
| `LLM_MODEL` | Model name | `openai/gpt-oss-120b` |
| `LLM_TEMPERATURE` | Sampling temperature | `0` |
| `LLM_MAX_TOKENS` | Max tokens in the answer | `1000` |
| `LLM_REQUEST_TIMEOUT` | Seconds per request | `60` |
| `LLM_RETRY_ATTEMPTS` | Retries after the first try | `3` |
| `LLM_REQUESTS_PER_MINUTE` | Client-side rate cap, `0` means no cap | `0` |
| `LLM_JSON_MODE` | Ask the API for a JSON-only answer | `true` |
| `LLM_ORGANIZATION`, `LLM_PROJECT` | Optional OpenAI headers | none |
| `HF_TOKEN` | Hugging Face token, only if the download is refused | none |
| `DATASET_URL`, `OUTPUT_FILE` | Override the dataset or output path | from `config.yaml` |

Settings are applied in this order, and later ones win: built-in defaults, then `config.yaml`, then `.env` and environment variables, then command line flags.

## Run it

Check the data first. A dry run reads the dataset and counts prompts but makes no API calls:

```powershell
python main.py --dry-run --max-records 2000
```

Small test run with the API:

```powershell
python main.py --max-prompts 20
```

Full run:

```powershell
python main.py --concurrency 5 --output output/prompt_smells.json
```

### Command line options

| Flag | Meaning |
|---|---|
| `--max-prompts N` | Stop after N prompts are sent this run |
| `--max-records N` | Read only the first N dataset rows |
| `--sample-fraction F` | Keep a fixed random share of conversations, for example `0.1` |
| `--output PATH` | Output JSON file |
| `--concurrency N` | Number of requests at the same time |
| `--resume` / `--no-resume` | Skip prompts done in earlier runs (default), or start over |
| `--requests-per-minute N` | Client-side rate cap |
| `--request-timeout S`, `--retry-attempts N` | Request limits |
| `--model NAME`, `--dataset PATH_OR_URL` | Change the model or the input file |
| `--config PATH`, `--env-file PATH` | Use other settings files |
| `--dry-run` | Count prompts, make no API calls |
| `--log-level LEVEL`, `--log-file PATH` | Logging |
| `--help` | Show all options |

## Rate limits and free tiers

Groq's free tier has low limits per minute and per day. If you see many "API rate limit encountered" messages, lower `--concurrency` to 1 or 2 and set `--requests-per-minute` to match your plan, for example `--requests-per-minute 30`. When the API returns HTTP 429, the tool waits as long as the `Retry-After` header says, or backs off with growing waits, and then tries again.

The shard holds tens of thousands of English prompts. A full run takes many hours and may cost money on a paid plan. Start with `--max-prompts` or `--sample-fraction`.

## Resume after a stop

Every finished prompt is saved at once to `<output>.checkpoint.jsonl`. If the run stops because of a crash, Ctrl+C or a daily limit, run the same command again. Prompts already done are skipped. The final JSON file is rebuilt from the checkpoint at the end of every run, including runs that stop early.

Prompts that failed after all retries are listed in `<output>.failures.jsonl`, with the error but without the prompt text. They are not saved as done, so the next run tries them again.

Use `--no-resume` to delete the checkpoint and start from zero.

Each prompt has a stable ID: a SHA-256 hash of the conversation ID and the message text. The same user message repeated in one conversation is analysed once.

## Smell categories

| Smell | Meaning |
|---|---|
| Vague / Missing Context | Too little detail to give a useful answer |
| Ambiguous References | Words like "this" or "it" with nothing to point to |
| Format Ambiguity | The wanted output form is unclear when it matters |
| Overloaded Prompt | Many unrelated tasks in one prompt |
| Prompt Bloat / Convoluted Prompt | Long, padded or hard to follow |
| Unnecessary Repetition | The same instruction said again without need |
| Conflicting Constraints | Instructions that cannot all be met |
| Irrelevant / Excessive Persona | A role or persona that adds nothing |
| Bias / Loaded Framing | The question pushes toward a set answer |
| Formality / Audience Mismatch | Tone or level does not fit the task |

The LLM may also return a new category if none of these fit. Short greetings and clear simple questions should get no smells.

Follow-up messages, the second user message onward in a conversation, are marked as follow-ups in the request. This stops the model from flagging "it" or "this" when the earlier turns would explain them.

## Output format

`output/prompt_smells.json` is a JSON array, written in UTF-8:

```json
[
  {
    "conversation_id": "abc123",
    "model": "gpt-4",
    "timestamp": "2023-05-01T10:00:00+00:00",
    "content": "Help me fix this.",
    "smell_type": "Vague / Missing Context",
    "smell_reason": "The prompt does not say what needs to be fixed."
  },
  {
    "conversation_id": "abc123",
    "model": "gpt-4",
    "timestamp": "2023-05-01T10:00:00+00:00",
    "content": "Help me fix this.",
    "smell_type": "Ambiguous References",
    "smell_reason": "'this' does not point to anything in the prompt."
  },
  {
    "conversation_id": "def456",
    "model": "gpt-3.5-turbo",
    "timestamp": "2023-05-02T11:30:00+00:00",
    "content": "What is the capital of France?",
    "smell_type": null,
    "smell_reason": null
  }
]
```

`content` is the user message exactly as it is in the dataset.

## Logging and privacy

Logs show progress, counts, retries and errors. They never show the API key, and they do not show prompt text. Use `--log-file run.log` to keep a copy. WildChat holds real user messages, so treat the output file as sensitive data.

## Run the tests

The tests use fake API responses and need no key or network:

```powershell
python -m pytest -q
```

## Limits

- The smell labels come from an LLM, so they are judgments, not ground truth. Check a sample by hand before you draw conclusions.
- Results can change between models, and sometimes between runs.
- The language filter uses the language labels already in WildChat. Some are wrong.
- Prompts longer than 12,000 characters are cut before sending (`max_prompt_chars` in `config.yaml`). The model is told the text was cut.
