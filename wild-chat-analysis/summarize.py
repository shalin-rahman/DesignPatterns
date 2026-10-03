"""Print summary statistics for an output file: python summarize.py output/prompt_smells.json"""

from __future__ import annotations

import argparse
import json
from pathlib import Path

from app.summary import summarize


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("output_file", type=Path)
    parser.add_argument("--save", type=Path, help="also write the summary to this JSON file")
    args = parser.parse_args()

    summary = summarize(json.loads(args.output_file.read_text(encoding="utf-8")))
    text = json.dumps(summary, indent=2, ensure_ascii=False)
    print(text)
    if args.save:
        args.save.write_text(text, encoding="utf-8")


if __name__ == "__main__":
    main()
