# SpecDetect4LLM_ICSE: Windows Setup Guide

This guide explains how to install, configure, test, and run
`SpecDetect4LLM_ICSE` on Windows. It also describes the repository layout and
the directory to use for each task.

> **Recommended Python version:** Python 3.11
>
> Python 3.12 may also work. Python 3.13 and newer may have dependency
> compatibility issues with this research project. Use the version pinned by
> `requirements.txt` when it differs from this recommendation.

## What this tool does

In simple terms, SpecDetect4LLM:

1. Reads Python files from a project folder.
2. Runs static-analysis rules that look for LLM integration code smells.
3. Writes the findings to a JSON file.

The command-line detector is the simplest way to run it. The web application
does the same analysis after you upload a ZIP or TAR archive. The PDF files in
`sdp_lab_tools/materials/` explain the research and smell taxonomy; they are
not required for installation.

## Why this project exists

Large language models are now commonly called from application code. An LLM
call can work correctly and still create maintainability or reliability
problems when important choices are implicit, unbounded, or difficult to
change. These recurring problems are called **LLM integration code smells**.

The research behind SpecDetect4LLM:

- defines a taxonomy of problematic LLM-integration practices;
- describes the design and quality risks associated with those practices;
- implements static-analysis rules to identify some of them; and
- evaluates detection performance and smell prevalence in open-source systems.

This is a **static-analysis tool**, not an LLM-powered reviewer. It reads source
code and applies deterministic rules; it does not execute the target project,
send its source code to an LLM, or prove that a finding is a runtime bug.

### What the current checkout detects

The current command-line checkout exposes rules `R25` through `R29`. Their
high-level topics are:

| Rule | Topic |
| --- | --- |
| `R25` | LLM temperature is not explicitly set |
| `R26` | LLM model/version is not explicitly pinned |
| `R27` | LLM call has no system message |
| `R28` | LLM call has no bounded metrics |
| `R29` | LLM pipeline has no structured output |

Rule names and availability can change between repository revisions. Always
run `python specDetect4LLM.py --list-rules` before selecting individual rules.
The research taxonomy can be broader than the rules present in a particular
checkout.

### How to use the research materials

Read the materials in this order if you want background before running the
tool:

1. **Specification and Detection paper** - why LLM code smells were defined
   and how detection was studied.
2. **LLM Code Smells: A Taxonomy and Detection Approach** - the expanded
   taxonomy, examples, and evaluation context.
3. **LLM Code Smells Design Patterns lecture** - a simpler explanation of the
   smells and possible design responses.
4. **Code Smells Lecture** and **Design Patterns lecture** - general
   software-design background.

These documents help explain *why* a rule may report a finding. The setup
steps below explain *how* to install and run the detector.

## Quick start: detector only

Use these actions for the normal command-line workflow:

| Action | Command | In simple words |
| --- | --- | --- |
| Clone | `git clone https://github.com/Brahim-Mahmoudi/SpecDetect4LLM_ICSE.git` | Download the project |
| Enter detector | `cd SpecDetect4LLM_ICSE\Detection` | Move to the detector folder |
| Create environment | `py -3.11 -m venv .venv` | Make an isolated Python workspace |
| Activate | `.venv\Scripts\Activate.ps1` | Use that workspace |
| Install | `python -m pip install -r requirements.txt` | Install detector packages |
| Check options | `python specDetect4LLM.py --help` | See available commands |
| List rules | `python specDetect4LLM.py --list-rules` | See which smell rules are available |
| Analyze | `python specDetect4LLM.py --input-dir "C:\path\to\project" --all` | Scan the project |
| Save results | Add `--output-file results.json` | Choose the results filename |

Run the full sections below when you need tests, Docker, prevalence analysis,
or troubleshooting.

## 1. Required software

Install the following before starting:

- [Git for Windows](https://git-scm.com/download/win)
- Python 3.9 or newer (Python 3.11 recommended)
- `pip` (included with the standard Python installer)
- PowerShell
- Docker (optional; required only for the web application)

## 2. Install and verify Git

Open PowerShell and run:

```powershell
git --version
```

If Git is not installed, download it from
[git-scm.com/download/win](https://git-scm.com/download/win). Close and
reopen PowerShell after installation, then run `git --version` again.

An output similar to the following confirms that Git is available:

```text
git version 2.x.x
```

## 3. Clone the repository

Choose a directory in which to store the project. For example:

```powershell
cd "C:\Users\YourName\workstation\shaleen\iit\DesignPatterns\sdp_lab_tools"
git clone https://github.com/Brahim-Mahmoudi/SpecDetect4LLM_ICSE.git
cd SpecDetect4LLM_ICSE
```

## 4. Repository layout

After cloning, the repository should approximately look like this. The exact
contents can vary by revision.

```text
SpecDetect4LLM_ICSE/
├── Catalog_Construction/
├── Detection/
│   ├── Rules/
│   ├── grammar/
│   ├── parser/
│   ├── test_rules/
│   ├── docs/
│   ├── static/
│   ├── specDetect4LLM.py
│   ├── requirements.txt
│   └── ...
├── Prevalence/
│   ├── Dataset/
│   ├── Extraction_LLM_Files/
│   ├── Extracted_Metrics/
│   └── Precision_Calculation/
├── web-app/
├── static/
├── Dockerfile
├── .dockerignore
├── requirements.txt
└── README.md
```

Use `git status` and `Get-ChildItem` to inspect the checkout if its layout
differs.

### Directory descriptions

- **`SpecDetect4LLM_ICSE/`** - repository root. Use it for Docker commands,
  repository-wide work, and research-artifact access.
- **`Catalog_Construction/`** - formal definitions and examples for the LLM
  code-smell catalog.
- **`Detection/`** - main detector directory. The detector, its requirements,
  parser, rules, and tests are here.
- **`Detection/Rules/`** - detection-rule implementations.
- **`Detection/grammar/`** - grammar files used by the parser.
- **`Detection/parser/`** - parser implementation.
- **`Detection/test_rules/`** - tests for detection rules.
- **`Detection/test_rules/run_all_tests.sh`** - shell-based rule-generation and
  test runner. Run it from `Detection/test_rules/`.
- **`Detection/docs/`** - detector and application documentation.
- **`Detection/static/`** - static resources for the application or web
  interface.
- **`Prevalence/`** - research prevalence-analysis material, not the normal
  detector workflow.
- **`Prevalence/Dataset/`** - research datasets.
- **`Prevalence/Extraction_LLM_Files/`** - LLM-file extraction scripts and
  supporting material.
- **`Prevalence/Extracted_Metrics/`** - extracted research metrics.
- **`Prevalence/Precision_Calculation/`** - precision-calculation scripts.
- **`web-app/`** - browser-based application.
- **`static/`** - root-level static assets used by the repository documentation.
- **`requirements.txt`** - root-level dependencies used by the Docker web app.
- **`Dockerfile`** - Docker image definition. Run Docker commands from the
  repository root.

### Working-directory reference

| Task | Directory |
| --- | --- |
| Clone the repository | Parent directory of the repository |
| Create the detector virtual environment | `Detection/` |
| Install detector requirements | `Detection/` |
| Run `specDetect4LLM.py` | `Detection/` |
| Run detector tests | `Detection/test_rules/` |
| Build the Docker image | Repository root |
| Run the Docker web app | Repository root |
| Run prevalence analysis | `Prevalence/` |
| Calculate precision | `Prevalence/Precision_Calculation/` |

## 5. Navigate to `Detection`

From the repository root, run:

```powershell
cd Detection
Get-Location
```

The prompt should resemble:

```text
PS C:\Users\YourName\...\SpecDetect4LLM_ICSE\Detection>
```

## 6. Check the installed Python versions

List Python installations:

```powershell
py -0p
```

For example:

```text
-V:3.14    C:\Python314\python.exe
-V:3.11    C:\Users\YourName\AppData\Local\Programs\Python\Python311\python.exe
```

Use Python 3.11 for this project where possible and verify it with:

```powershell
py -3.11 --version
```

Expected output:

```text
Python 3.11.x
```

If Python is not installed, download it from
[python.org/downloads/windows](https://www.python.org/downloads/windows/).
During installation, enable **Add python.exe to PATH** and ensure that `pip`
is selected. Reopen PowerShell after installation, then repeat `py -0p` and
`py -3.11 --version`.

## 7. Create the virtual environment

Make sure the current directory is `SpecDetect4LLM_ICSE\Detection`, then run:

```powershell
py -3.11 -m venv .venv
```

The `Detection/` directory will contain a virtual environment similar to:

```text
Detection/
├── .venv/
│   ├── Scripts/
│   ├── Lib/
│   └── ...
├── specDetect4LLM.py
├── requirements.txt
└── ...
```

## 8. Activate the virtual environment

In PowerShell, run:

```powershell
.venv\Scripts\Activate.ps1
```

The prompt should include `(.venv)`, for example:

```text
(.venv) PS C:\...\SpecDetect4LLM_ICSE\Detection>
```

If PowerShell reports that script execution is disabled, allow locally
created scripts for the current user and activate again:

```powershell
Set-ExecutionPolicy -ExecutionPolicy RemoteSigned -Scope CurrentUser
.venv\Scripts\Activate.ps1
```

## 9. Verify Python and `pip`

After activation, run:

```powershell
python --version
where.exe python
python -m pip --version
```

The first `where.exe python` result should point inside:

```text
...\SpecDetect4LLM_ICSE\Detection\.venv\Scripts\python.exe
```

Upgrade `pip` before installing project dependencies:

```powershell
python -m pip install --upgrade pip
```

## 10. Install detector dependencies

Confirm that the current directory is `SpecDetect4LLM_ICSE\Detection`, then
run:

```powershell
python -m pip install -r requirements.txt
```

The detector requirements file currently pins Lark to `1.2.2` and contains the
packages needed by the command-line detector and rule tests. Use this file for
local detector work; do not install the root requirements file into the
detector environment unless you also intend to run the web app.

To inspect the installed Lark version when Lark is a project dependency, run:

```powershell
python -c "import lark; print(lark.__version__)"
```

The repository also has a separate root-level `requirements.txt` for the
Docker web app. Docker installs that file automatically during the image build.

## 11. Verify the detector

First inspect the command-line interface so that the options match the
checkout you cloned:

```powershell
python specDetect4LLM.py --help
```

If the checkout exposes `--list-rules`, list the available detection rules:

```powershell
python specDetect4LLM.py --list-rules
```

If the rules are displayed, the basic installation is working.

## 12. Run the project tests

The repository-provided test runner is:

```text
Detection/test_rules/run_all_tests.sh
```

Run it from `Detection/test_rules/` in Git Bash or WSL:

```bash
cd Detection/test_rules
./run_all_tests.sh
```

The script regenerates rule modules and runs tests for the rule directories it
lists. Inspect the script before relying on it for newly added rules.

## 13. Run the detector on a project

Assume the target project is:

```text
C:\Users\YourName\Projects\MyLLMProject
```

From `SpecDetect4LLM_ICSE\Detection`, run all rules:

```powershell
python specDetect4LLM.py --input-dir "C:\Users\YourName\Projects\MyLLMProject" --all
```

To save results, add an output path supported by the checkout:

```powershell
python specDetect4LLM.py --input-dir "C:\Users\YourName\Projects\MyLLMProject" --all --output-file results.json
```

Relative output paths are resolved from the current directory. With the
command above, the expected location is `Detection/results.json`.

If supported by the checkout, generate a summary with:

```powershell
python specDetect4LLM.py --input-dir "C:\Users\YourName\Projects\MyLLMProject" --all --summary
```

To run selected rules, use the rule identifiers reported by `--list-rules`.
For example:

```powershell
python specDetect4LLM.py --input-dir "C:\Users\YourName\Projects\MyLLMProject" --rules R25 R26 R29
```

To run all rules except selected rules:

```powershell
python specDetect4LLM.py --input-dir "C:\Users\YourName\Projects\MyLLMProject" --all --exclude R26 R29
```

The detector recursively scans only `.py` files under `--input-dir`. It does
not analyze JavaScript, notebooks, archives, or non-Python source files.

## 14. Run the Docker web application

Docker commands must be run from the repository root, not from `Detection/`.
If you are currently in `Detection/`, run:

```powershell
cd ..
Get-Location
```

The location should resemble:

```text
...\SpecDetect4LLM_ICSE
```

Build and start the application:

```powershell
docker build -t specdetect4llm-web .
docker run -d -p 8080:5000 --name specdetect4llm_app specdetect4llm-web
```

Open [http://localhost:8080](http://localhost:8080) in a browser.

To stop and remove the container later:

```powershell
docker stop specdetect4llm_app
docker rm specdetect4llm_app
```

If the page does not load, inspect the container logs:

```powershell
docker logs specdetect4llm_app
```

The web application accepts ZIP/TAR project archives and has a 500 MB upload
limit. The Docker image uses the root-level `requirements.txt`, not
`Detection/requirements.txt`.

## 15. Further documentation

The repository includes:

- [CLI usage](SpecDetect4LLM_ICSE/Detection/docs/usage.md)
- [Docker web-app guide](SpecDetect4LLM_ICSE/Detection/docs/docker.md)
- [Repository overview](SpecDetect4LLM_ICSE/README.md)

The PDF files under the separate `sdp_lab_tools/materials/` directory are
research papers and lecture material. They explain the smell taxonomy and
motivation, but they are not required to install or run the tool.

## 16. Run prevalence analysis

The detector and the research-analysis pipeline are separate workflows. The
prevalence material is under:

```text
SpecDetect4LLM_ICSE/
└── Prevalence/
```

## 17. Calculate precision

Navigate to the precision-calculation directory:

```powershell
cd "C:\Users\YourName\...\SpecDetect4LLM_ICSE\Prevalence\Precision_Calculation"
python compute_precision.py
```

## 18. Recommended end-to-end setup

For a normal detector run, use this sequence:

```powershell
# Check Git and clone the repository
git --version
git clone https://github.com/Brahim-Mahmoudi/SpecDetect4LLM_ICSE.git
cd SpecDetect4LLM_ICSE\Detection

# Check Python and create the environment
py -3.11 --version
py -3.11 -m venv .venv
.venv\Scripts\Activate.ps1

# Install dependencies
python --version
python -m pip install --upgrade pip
python -m pip install -r requirements.txt

# Verify and run the detector
python specDetect4LLM.py --help
python specDetect4LLM.py --list-rules
python specDetect4LLM.py --input-dir "C:\path\to\your\project" --all
```

## 19. Final directory layout

After setup, the relevant layout will look approximately like this:

```text
SpecDetect4LLM_ICSE/
├── Detection/
│   ├── .venv/                 # Python virtual environment
│   ├── Rules/                 # Detection rules
│   ├── grammar/               # Grammar files
│   ├── parser/                # Parser implementation
│   ├── test_rules/            # Rule tests
│   ├── docs/                  # Documentation
│   ├── static/                # Static web resources
│   ├── specDetect4LLM.py      # Main detector
│   ├── requirements.txt       # Python dependencies
│   └── results.json           # Optional generated output
├── Catalog_Construction/      # Smell specifications and examples
├── Prevalence/                # Research analysis
│   ├── Dataset/
│   ├── Extraction_LLM_Files/
│   ├── Extracted_Metrics/
│   └── Precision_Calculation/
├── web-app/                   # Web application
├── static/                    # Root-level documentation assets
├── Dockerfile                 # Docker configuration
├── .dockerignore
├── requirements.txt           # Docker/web-app dependencies
└── README.md
```

## 20. Windows command reference

| Purpose | PowerShell command |
| --- | --- |
| List Python versions | `py -0p` |
| Select Python 3.11 | `py -3.11` |
| Create a virtual environment | `py -3.11 -m venv .venv` |
| Activate the environment | `.venv\Scripts\Activate.ps1` |
| Check Python | `python --version` |
| Check Python path | `where.exe python` |
| Install packages | `python -m pip install -r requirements.txt` |

Do not use the Linux/macOS activation command in PowerShell:

```bash
source .venv/bin/activate
```

Use this Windows PowerShell command instead:

```powershell
.venv\Scripts\Activate.ps1
```

## 21. Python-version guidance

| Version | Recommendation |
| --- | --- |
| Python 3.11 | Recommended |
| Python 3.12 | Acceptable alternative |
| Python 3.13 | Try only if required |
| Python 3.14 | Not recommended |

For reproducibility, use Python 3.11 where possible and record the exact
Python and dependency versions used in the experiment.

## 22. Troubleshooting actions

### `py -3.11` is not found

Install Python 3.11, reopen PowerShell, and run:

```powershell
py -0p
py -3.11 --version
```

If you intentionally use another supported Python version, use the same
version in both the virtual-environment command and your version record.

### PowerShell cannot activate `.venv`

Run this once for the current Windows user, then activate again:

```powershell
Set-ExecutionPolicy -ExecutionPolicy RemoteSigned -Scope CurrentUser
.venv\Scripts\Activate.ps1
```

### `ModuleNotFoundError` appears

Confirm that the prompt starts with `(.venv)`, then reinstall the detector
dependencies from `Detection/`:

```powershell
python -m pip install -r requirements.txt
```

Using `python -m pip` makes sure packages are installed into the Python
interpreter that will run the detector.

### No rules are listed

Confirm that you are in `SpecDetect4LLM_ICSE\Detection` and that
`Detection\test_rules` exists:

```powershell
Get-Location
Get-ChildItem .\test_rules
python specDetect4LLM.py --list-rules
```

### Docker reports that the container name is already in use

Stop and remove the previous container, then run it again:

```powershell
docker stop specdetect4llm_app
docker rm specdetect4llm_app
docker run -d -p 8080:5000 --name specdetect4llm_app specdetect4llm-web
```

### The web page does not open

Check whether the container is running and read its logs:

```powershell
docker ps
docker logs specdetect4llm_app
```

Also confirm that port `8080` is not already being used by another program.
