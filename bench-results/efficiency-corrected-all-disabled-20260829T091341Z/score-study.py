#!/usr/bin/env python3
"""Apply the frozen three-feature scorer to this independent control cohort."""

from __future__ import annotations

import argparse
import importlib.util
import json
import tempfile
from pathlib import Path
from types import ModuleType
from typing import Any


STUDY_DIR = Path(__file__).resolve().parent
SCORER_PATH = STUDY_DIR / "scorer" / "score.py"


def load_frozen_scorer() -> ModuleType:
    specification = importlib.util.spec_from_file_location("frozen_feature_scorer", SCORER_PATH)
    if specification is None or specification.loader is None:
        raise RuntimeError(f"cannot load frozen scorer: {SCORER_PATH}")
    module = importlib.util.module_from_spec(specification)
    specification.loader.exec_module(module)
    return module


def build_result(
    scorer: ModuleType, manifest: dict[str, Any], runs: list[dict[str, Any]]
) -> dict[str, Any]:
    return {
        "schema_version": 1,
        "complete": True,
        "study": manifest["study"],
        "base_commit": manifest["base_commit"],
        "task_list_sha256": manifest["task_list_sha256"],
        "model": manifest["model"],
        "reasoning_effort": manifest["reasoning_effort"],
        "frozen_scorer_sha256": scorer.sha256_file(SCORER_PATH),
        "fixtures": {
            name: scorer.sha256_file(
                STUDY_DIR / "scorer" / "fixtures" / fixture
            )
            for name, (fixture, _) in scorer.FIXTURES.items()
        },
        "runs": runs,
    }


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser()
    parser.add_argument("--manifest", type=Path, default=STUDY_DIR / "score-manifest.json")
    parser.add_argument("--output", type=Path, default=STUDY_DIR / "quality.json")
    parser.add_argument("--work-dir", type=Path)
    parser.add_argument("--timeout-seconds", type=int, default=900)
    return parser.parse_args()


def main() -> int:
    args = parse_args()
    manifest = json.loads(args.manifest.resolve().read_text(encoding="utf-8"))
    scorer = load_frozen_scorer()
    temporary: tempfile.TemporaryDirectory[str] | None = None
    if args.work_dir:
        work_root = args.work_dir.resolve()
        work_root.mkdir(parents=True, exist_ok=True)
    else:
        temporary = tempfile.TemporaryDirectory(prefix="work-leaf-corrected-control-score-")
        work_root = Path(temporary.name)
    try:
        runs = [
            scorer.score_entry(entry, manifest, work_root, args.timeout_seconds)
            for entry in manifest["runs"]
        ]
        scorer.write_json(args.output.resolve(), build_result(scorer, manifest, runs))
    finally:
        if temporary is not None:
            temporary.cleanup()
    print(args.output.resolve())
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
