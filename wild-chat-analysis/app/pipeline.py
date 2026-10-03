"""Runs the steps in order: load dataset, extract prompts, analyze, write output."""

from __future__ import annotations

import logging
from collections.abc import Iterator
from concurrent.futures import FIRST_COMPLETED, Future, ThreadPoolExecutor, wait
from dataclasses import dataclass
from pathlib import Path

from tqdm import tqdm
from tqdm.contrib.logging import logging_redirect_tqdm

from app.config import Settings
from app.conversation_parser import extract_user_prompts, in_sample
from app.dataset_loader import count_rows, iter_records, resolve_dataset
from app.llm_client import FatalLLMError, LLMClient
from app.models import AnalysisResult, UserPrompt
from app.output_writer import CheckpointStore, FailureLog, write_final_json
from app.prompt_analyzer import PromptAnalyzer, build_output_records

logger = logging.getLogger(__name__)


@dataclass
class RunStats:
    """Counts for one run, logged at the end and returned to the caller."""
    records_read: int = 0
    malformed_records: int = 0
    prompts_extracted: int = 0
    already_processed: int = 0
    to_process: int = 0
    succeeded: int = 0
    failed: int = 0
    output_records: int = 0


class Analyzer:
    """Anything with an `analyze(UserPrompt) -> AnalysisResult` method."""

    def analyze(self, prompt: UserPrompt) -> AnalysisResult:  # pragma: no cover
        """Return the smells found in one prompt."""
        raise NotImplementedError


def collect_prompts(settings: Settings, dataset_path: Path, stats: RunStats) -> list[UserPrompt]:
    """Read the dataset and return the user prompts in the sample, without duplicates.

    Keeps only conversations whose id falls in the sample fraction, and only turns in
    the chosen language. A bad row is counted in `stats` and skipped.
    """
    prompts: dict[str, UserPrompt] = {}
    records: Iterator[dict] = iter_records(
        dataset_path, batch_size=settings.read_batch_size, max_records=settings.max_records
    )
    for record in records:
        stats.records_read += 1
        try:
            conversation_id = str(record.get("conversation_id") or "")
            if not in_sample(conversation_id, settings.sample_fraction, settings.sample_seed):
                continue
            for prompt in extract_user_prompts(record, settings.language):
                prompts.setdefault(prompt.prompt_id, prompt)
        except Exception as exc:  # one bad row must not stop the run
            stats.malformed_records += 1
            logger.warning("Skipping malformed record %d: %s", stats.records_read, type(exc).__name__)
    return list(prompts.values())


def process_prompts(
    pending: list[UserPrompt],
    analyzer: Analyzer,
    store: CheckpointStore,
    failures: FailureLog,
    settings: Settings,
    stats: RunStats,
) -> None:
    """Send the pending prompts to the analyzer on a thread pool and save each result.

    Only a few prompts are queued at a time. A success goes to the checkpoint and a
    failure to the failure log. A FatalLLMError, Ctrl+C or disk error cancels the rest.
    """
    total = len(pending)
    queue = iter(pending)
    in_flight: dict[Future[AnalysisResult], UserPrompt] = {}
    pool = ThreadPoolExecutor(max_workers=settings.concurrency, thread_name_prefix="llm")

    def submit_next() -> bool:
        """Queue the next prompt. Returns False when none are left."""
        prompt = next(queue, None)
        if prompt is None:
            return False
        in_flight[pool.submit(analyzer.analyze, prompt)] = prompt
        return True

    done_count = 0
    try:
        with logging_redirect_tqdm(), tqdm(total=total, unit="prompt", ascii=True) as bar:
            # Keep only a small window of work queued so memory stays flat.
            for _ in range(settings.concurrency * 2):
                if not submit_next():
                    break
            while in_flight:
                finished, _ = wait(in_flight, return_when=FIRST_COMPLETED)
                for future in finished:
                    prompt = in_flight.pop(future)
                    try:
                        result = future.result()
                    except FatalLLMError:
                        raise
                    except Exception as exc:
                        stats.failed += 1
                        error = f"{type(exc).__name__}: {exc}"
                        logger.error("Failed to analyze prompt: %s (%s)", prompt.prompt_id[:16], error)
                        failures.append(prompt.prompt_id, prompt.conversation_id, error)
                    else:
                        store.append(prompt.prompt_id, build_output_records(prompt, result))
                        stats.succeeded += 1
                    done_count += 1
                    bar.update(1)
                    if done_count % settings.progress_log_every == 0:
                        logger.info("Progress: %d/%d", done_count, total)
                    submit_next()
    except BaseException:
        # Ctrl+C, a fatal API error or a disk error: drop queued work so no more calls go out.
        pool.shutdown(wait=False, cancel_futures=True)
        raise
    pool.shutdown(wait=True)


def run_pipeline(
    settings: Settings,
    analyzer: Analyzer | None = None,
    dry_run: bool = False,
) -> RunStats:
    """Run the whole job: load the dataset, pick prompts, skip ones already done, label the
    rest and write the final JSON.

    With `dry_run`, stops after counting what would be sent. The final JSON is written
    even when the run stops early, so finished work is never lost.
    """
    stats = RunStats()
    store = CheckpointStore(settings.checkpoint_file)
    failures = FailureLog(settings.failures_file)

    if not settings.resume and not dry_run:
        logger.info("Resume disabled: clearing checkpoint %s", store.path)
        store.reset()
    processed_ids = store.load_processed_ids()

    logger.info("Loading dataset")
    dataset_path = resolve_dataset(
        settings.dataset_url,
        settings.cache_dir,
        settings.hf_token.get_secret_value() if settings.hf_token else None,
        timeout=settings.llm.request_timeout,
    )
    logger.info("Dataset loaded: %d records", count_rows(dataset_path))

    prompts = collect_prompts(settings, dataset_path, stats)
    stats.prompts_extracted = len(prompts)
    logger.info("Records read: %d (malformed: %d)", stats.records_read, stats.malformed_records)
    logger.info("Extracted %s user prompts: %d", settings.language, stats.prompts_extracted)

    pending = [p for p in prompts if p.prompt_id not in processed_ids]
    stats.already_processed = len(prompts) - len(pending)
    if stats.already_processed:
        logger.info("Already in checkpoint, skipping: %d", stats.already_processed)
    if settings.max_prompts is not None:
        pending = pending[: settings.max_prompts]
    stats.to_process = len(pending)

    if dry_run:
        logger.info("Dry run: %d prompts would be sent to the API. Nothing sent.", stats.to_process)
        return stats

    client: LLMClient | None = None
    if analyzer is None:
        client = LLMClient(settings.llm)
        analyzer = PromptAnalyzer(client, settings.max_prompt_chars, settings.parse_retries)

    failures.reset()
    logger.info("Processing prompts... (%d to send, concurrency %d)", stats.to_process, settings.concurrency)
    try:
        with store:
            process_prompts(pending, analyzer, store, failures, settings, stats)
        logger.info("Processing completed: %d ok, %d failed", stats.succeeded, stats.failed)
    finally:
        if client is not None:
            client.close()
        stats.output_records = write_final_json(store, settings.output_file)
        logger.info("Results written to %s (%d records)", settings.output_file, stats.output_records)
        if stats.failed:
            logger.info("Failed prompts listed in %s; run again to retry them", settings.failures_file)
    return stats
