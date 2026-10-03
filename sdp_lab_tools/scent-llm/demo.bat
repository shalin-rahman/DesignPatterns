@echo off
setlocal enabledelayedexpansion

rem Uses "python -m scent_llm.cli" instead of the "scent-llm" console script
rem so the demo works even when the package is installed but Scripts\ isn't
rem on PATH. Run this from the scent-llm project directory.

echo ============================================
echo  scent-llm full-cycle demo
echo ============================================

rem ---------------------------------------------------------------
rem Step 1: choose a generate prompt
rem ---------------------------------------------------------------
set "PROMPT1=write a function that summarizes a list of strings"
set "PROMPT2=write a function that calls an LLM to summarize text, using client.chat.completions.create with no other arguments"
set "PROMPT3=write a function that calls an LLM to summarize text, pinning an exact model version, and passing an explicit temperature, max_tokens, and system message"

echo.
echo Step 1 -- generate --analyze : choose a prompt
echo ----------------------------------------------
echo   1. Plain algorithm, no LLM call        (expect 0 smells - nothing to analyze)
echo   2. LLM-calling code, minimal args       (expect real smells: NSO/UMM/TNES/NMVP/NSM)
echo   3. LLM-calling code, fully configured   (expect 0 smells - a clean LLM call)
echo   4. Custom prompt (type your own)
choice /c 1234 /n /t 15 /d 1 /m "Select 1-4 (defaults to 1 after 15s): "
if errorlevel 4 (
    set /p GEN_PROMPT="Enter your own prompt: "
) else if errorlevel 3 (
    set "GEN_PROMPT=%PROMPT3%"
) else if errorlevel 2 (
    set "GEN_PROMPT=%PROMPT2%"
) else (
    set "GEN_PROMPT=%PROMPT1%"
)

echo.
echo   - Loads config (.env -^> env var -^> scent_llm.toml -^> default), then
echo     POSTs one chat-completions request to the configured provider.
echo   - --analyze then runs the same AST detectors as Step 2 on the
echo     generated code below (see "Analyzing generated code..." output).
echo     They only fire on LLM calls *inside* that code, so a plain
echo     algorithm (option 1) is expected to show 0 smells.
echo.
echo Prompt sent to the LLM:
echo   "!GEN_PROMPT!"
echo.
echo Generated code and smell analysis:
python -m scent_llm.cli generate "!GEN_PROMPT!" --analyze
echo.

rem ---------------------------------------------------------------
rem Step 2: choose an analyze target
rem ---------------------------------------------------------------
echo Step 2 -- analyze : choose a target
echo ----------------------------------------------
echo   1. examples\smelly_example.py   (known-smelly demo file, trips 4 detectors)
echo   2. scent_llm                     (this tool's own source)
echo   3. Custom path
choice /c 123 /n /t 15 /d 1 /m "Select 1-3 (defaults to 1 after 15s): "
if errorlevel 3 (
    set /p ANALYZE_PATH="Enter a path to analyze: "
) else if errorlevel 2 (
    set "ANALYZE_PATH=scent_llm"
) else (
    set "ANALYZE_PATH=examples"
)

echo.
echo   - Static only, no LLM call. Walks the Python AST looking for LLM SDK
echo     call shapes (e.g. .chat.completions.create), then runs all 5
echo     detectors (NSO, UMM, TNES, NMVP, NSM) on each match.
echo   - Target: "!ANALYZE_PATH!"
python -m scent_llm.cli analyze "!ANALYZE_PATH!" --json
echo.

echo Step 3 -- sandbox
echo ----------------------------------------------
echo   - Runs the file in Docker if the CLI is on PATH, else a plain
echo     subprocess. Docker installed but daemon not running -^> local
echo     machine issue, not a bug; use --no-docker to skip straight to the
echo     subprocess fallback.
python -m scent_llm.cli sandbox examples\smelly_example.py --language python
echo.

echo Step 4 -- diagram sequence
echo ----------------------------------------------
echo   - Static, no I/O: prints this tool's own pipeline as ASCII.
python -m scent_llm.cli diagram sequence
echo.

echo ============================================
echo  Demo complete
echo ============================================

endlocal
