"""Command-line entry point: python main.py --help"""

from __future__ import annotations

import argparse
import logging
import sys
from pathlib import Path
from typing import Any

from app.config import ConfigError, Settings, load_settings
from app.dataset_loader import DatasetError
from app.llm_client import FatalLLMError
from app.pipeline import run_pipeline

logger = logging.getLogger("prompt_smells")


def build_parser() -> argparse.ArgumentParser:
    parser = argparse.ArgumentParser(
        description="Detect prompt smells in English user prompts from the WildChat dataset.",
    )
    parser.add_argument("--config", type=Path, default=Path("config.yaml"), help="YAML config file (default: config.yaml)")
    parser.add_argument("--env-file", type=Path, default=Path(".env"), help="dotenv file (default: .env)")
    parser.add_argument("--dataset", dest="dataset_url", help="Parquet URL or local path")
    parser.add_argument("--output", dest="output_file", type=Path, help="Output JSON file")
    parser.add_argument("--max-records", type=int, help="Read at most this many dataset rows")
    parser.add_argument("--max-prompts", type=int, help="Send at most this many prompts in this run")
    parser.add_argument("--sample-fraction", type=float, help="Keep this share of conversations (0-1]")
    parser.add_argument("--concurrency", type=int, help="Parallel API requests")
    parser.add_argument("--request-timeout", type=float, help="API request timeout in seconds")
    parser.add_argument("--retry-attempts", type=int, help="Retries per API request")
    parser.add_argument("--requests-per-minute", type=float, help="API request rate limit (0 = none)")
    parser.add_argument("--model", help="LLM model name")
    parser.add_argument(
        "--resume",
        action=argparse.BooleanOptionalAction,
        default=None,
        help="Skip prompts already in the checkpoint (default from config: on). "
        "--no-resume deletes the checkpoint and starts over.",
    )
    parser.add_argument("--dry-run", action="store_true", help="Load and filter the dataset only; call no API")
    parser.add_argument("--log-level", help="DEBUG, INFO, WARNING or ERROR")
    parser.add_argument("--log-file", type=Path, help="Also write logs to this file")
    return parser


def setup_logging(level: str, log_file: Path | None) -> None:
    handlers: list[logging.Handler] = [logging.StreamHandler()]
    if log_file is not None:
        log_file.parent.mkdir(parents=True, exist_ok=True)
        handlers.append(logging.FileHandler(log_file, encoding="utf-8"))
    logging.basicConfig(
        level=level.upper(),
        format="%(asctime)s %(levelname)s %(name)s - %(message)s",
        handlers=handlers,
        force=True,
    )
    # urllib3 debug logs can include request details; keep them quiet.
    logging.getLogger("urllib3").setLevel(logging.WARNING)


def main(argv: list[str] | None = None) -> int:
    args = build_parser().parse_args(argv)
    overrides: dict[str, Any] = {
        key: value
        for key, value in vars(args).items()
        if key not in {"config", "env_file", "dry_run"}
    }
    config_path = args.config if args.config.exists() or args.config != Path("config.yaml") else None
    try:
        settings: Settings = load_settings(config_path, overrides, args.env_file)
    except ConfigError as exc:
        print(f"Configuration error: {exc}", file=sys.stderr)
        return 1

    setup_logging(settings.log_level, settings.log_file)
    try:
        stats = run_pipeline(settings, dry_run=args.dry_run)
    except (ConfigError, DatasetError) as exc:
        logger.error("%s", exc)
        return 1
    except FatalLLMError as exc:
        logger.error("API refused the request (%s). Check LLM_API_KEY, LLM_BASE_URL and LLM_MODEL.", exc)
        return 1
    except KeyboardInterrupt:
        logger.warning("Interrupted. Run the same command again to resume.")
        return 130
    logger.info("Summary: %s", stats)
    return 0


if __name__ == "__main__":
    sys.exit(main())
