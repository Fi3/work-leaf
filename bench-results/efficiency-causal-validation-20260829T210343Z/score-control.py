#!/usr/bin/env python3

import argparse
import hashlib
import importlib.util
import json
import tempfile
from pathlib import Path
from types import ModuleType


STUDY = Path(__file__).resolve().parent
ROOT = STUDY.parents[1]
FROZEN_STUDY = ROOT / "bench-results/efficiency-exact-normal-work-leaf-20260829T181318Z"
FROZEN_SCORER = FROZEN_STUDY / "scorer/score.py"


def sha256(path: Path) -> str:
    digest = hashlib.sha256()
    with path.open("rb") as handle:
        for chunk in iter(lambda: handle.read(1024 * 1024), b""):
            digest.update(chunk)
    return digest.hexdigest()


def load_frozen_scorer() -> ModuleType:
    specification = importlib.util.spec_from_file_location(
        "causal_validation_frozen_scorer", FROZEN_SCORER
    )
    if specification is None or specification.loader is None:
        raise RuntimeError(f"cannot load frozen scorer: {FROZEN_SCORER}")
    module = importlib.util.module_from_spec(specification)
    specification.loader.exec_module(module)
    module.STUDY_DIR = STUDY
    return module


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser()
    parser.add_argument("--manifest", type=Path, default=STUDY / "control-score-manifest.json")
    parser.add_argument("--output", type=Path, default=STUDY / "control-quality.json")
    parser.add_argument("--work-dir", type=Path)
    parser.add_argument("--timeout-seconds", type=int, default=900)
    return parser.parse_args()


def main() -> int:
    args = parse_args()
    manifest = json.loads(args.manifest.resolve().read_text(encoding="utf-8"))
    scorer = load_frozen_scorer()
    temporary: tempfile.TemporaryDirectory[str] | None = None
    if args.work_dir is None:
        temporary = tempfile.TemporaryDirectory(prefix="work-leaf-direct-read-score-")
        work_root = Path(temporary.name)
    else:
        work_root = args.work_dir.resolve()
        work_root.mkdir(parents=True, exist_ok=True)
    try:
        runs = [
            scorer.score_entry(entry, manifest, work_root, args.timeout_seconds)
            for entry in manifest["runs"]
        ]
        result = {
            "schema_version": 1,
            "complete": True,
            "study": manifest["study"],
            "base_commit": manifest["base_commit"],
            "task_list_sha256": manifest["task_list_sha256"],
            "model": manifest["model"],
            "reasoning_effort": manifest["reasoning_effort"],
            "frozen_scorer": str(FROZEN_SCORER.relative_to(ROOT)),
            "frozen_scorer_sha256": sha256(FROZEN_SCORER),
            "fixtures": {
                feature: {
                    "path": str((FROZEN_STUDY / "scorer/fixtures" / fixture).relative_to(ROOT)),
                    "sha256": sha256(FROZEN_STUDY / "scorer/fixtures" / fixture),
                }
                for feature, (fixture, _) in scorer.FIXTURES.items()
            },
            "runs": runs,
        }
        scorer.write_json(args.output.resolve(), result)
    finally:
        if temporary is not None:
            temporary.cleanup()
    print(args.output.resolve())
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
