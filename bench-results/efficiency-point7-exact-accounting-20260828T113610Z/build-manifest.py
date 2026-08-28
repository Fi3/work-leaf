#!/usr/bin/env python3
"""Build the immutable scorer manifest from all three preserved observations."""

import json
import os
from pathlib import Path


STUDY = Path(__file__).resolve().parent
SOURCE = Path("/tmp/work-leaf-point7-exact-source-db5fe21")
CONDITIONS = (
    ("direct", "direct-sequential", "point7-exact-direct-three-feature-sequential-bench-artifacts"),
    ("wl-000", "work-leaf-concurrent", "point7-exact-wl-000-three-feature-bench-artifacts"),
    ("wl-111", "work-leaf-concurrent", "point7-exact-wl-111-three-feature-bench-artifacts"),
)


def main():
    runs = []
    for condition, workflow, artifact_name in CONDITIONS:
        artifact = STUDY / "runs" / condition / artifact_name
        report = artifact / "report.json"
        exit_path = STUDY / "logs" / f"{condition}.exit"
        if not report.is_file() or not exit_path.is_file():
            raise RuntimeError(f"{condition} has no complete preserved report and exit record")
        runs.append(
            {
                "id": f"point7-exact-{condition}",
                "condition": condition,
                "workflow": workflow,
                "launcher_exit_code": int(exit_path.read_text().strip()),
                "artifact": str(artifact),
                "report": str(report),
            }
        )
    payload = {
        "schema_version": 1,
        "study": STUDY.name,
        "base_commit": "c92a0b7060a36eac6db2d869b85e589a7a9480f9",
        "task_list_sha256": "45bee25a4b929182d36612fc5a159597e7770f25dba9c95760713a401d45598a",
        "model": "gpt-5.5",
        "reasoning_effort": "xhigh",
        "source_repo": str(SOURCE),
        "runs": runs,
    }
    output = STUDY / "manifest.json"
    if output.exists():
        raise RuntimeError("refusing to replace the existing manifest")
    temporary = output.with_suffix(".json.tmp")
    temporary.write_text(json.dumps(payload, indent=2, sort_keys=True) + "\n")
    os.replace(temporary, output)


if __name__ == "__main__":
    main()
