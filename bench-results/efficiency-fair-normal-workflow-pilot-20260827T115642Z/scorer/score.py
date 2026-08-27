#!/usr/bin/env python3
"""Score the two saved pilot implementations without launching a provider."""

from __future__ import annotations

import argparse
import hashlib
import json
import os
import shutil
import subprocess
import tempfile
import time
from pathlib import Path
from typing import Any, Sequence


SCRIPT_DIR = Path(__file__).resolve().parent
STUDY_DIR = SCRIPT_DIR.parent
REPO_ROOT = SCRIPT_DIR.parents[2]
FIXTURES = {
    "visual": ("quality_visual.rs", "quality_visual_behavior"),
    "status": ("quality_status.rs", "quality_status_behavior"),
    "completion": ("quality_completion.rs", "quality_completion_behavior"),
}
USAGE_FIELDS = (
    "input_tokens",
    "cached_input_tokens",
    "uncached_input_tokens",
    "output_tokens",
    "reasoning_output_tokens",
    "raw_input_plus_output",
    "uncached_input_plus_output",
)


def sha256_file(path: Path) -> str:
    digest = hashlib.sha256()
    with path.open("rb") as stream:
        for chunk in iter(lambda: stream.read(1024 * 1024), b""):
            digest.update(chunk)
    return digest.hexdigest()


def read_json(path: Path) -> dict[str, Any]:
    return json.loads(path.read_text(encoding="utf-8"))


def write_json(path: Path, value: Any) -> None:
    temporary = path.with_suffix(path.suffix + ".tmp")
    temporary.write_text(
        json.dumps(value, indent=2, sort_keys=True) + "\n", encoding="utf-8"
    )
    temporary.replace(path)


def run_command(
    argv: Sequence[str],
    cwd: Path,
    *,
    timeout: int,
    environment: dict[str, str] | None = None,
) -> subprocess.CompletedProcess[str]:
    return subprocess.run(
        list(argv),
        cwd=cwd,
        text=True,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
        timeout=timeout,
        check=False,
        env=environment,
    )


def resolve_repo_path(value: str) -> Path:
    path = Path(value)
    return path if path.is_absolute() else REPO_ROOT / path


def snapshot_directory(artifact: Path, workflow_result: str) -> Path | None:
    preferred = artifact / "patches" / workflow_result
    if preferred.is_dir():
        return preferred
    candidates = sorted(path for path in (artifact / "patches").glob("*") if path.is_dir())
    return candidates[0] if len(candidates) == 1 else None


def apply_saved_diff(checkout: Path, path: Path, timeout: int) -> None:
    if not path.is_file() or path.stat().st_size == 0:
        return
    completed = run_command(
        ["git", "apply", "--whitespace=nowarn", str(path)],
        checkout,
        timeout=timeout,
    )
    if completed.returncode != 0:
        raise RuntimeError(f"cannot apply {path.name}: {completed.stderr.strip()}")


def prepare_checkout(
    entry: dict[str, Any], source_repo: Path, base_commit: str, work_root: Path, timeout: int
) -> tuple[Path, str, list[str]]:
    checkout = work_root / "checkouts" / entry["id"]
    checkout.parent.mkdir(parents=True, exist_ok=True)
    clone = run_command(
        ["git", "clone", "--quiet", "--no-checkout", "--shared", str(source_repo), str(checkout)],
        work_root,
        timeout=timeout,
    )
    if clone.returncode != 0:
        raise RuntimeError(f"git clone failed: {clone.stderr.strip()}")

    artifact = resolve_repo_path(entry["artifact"])
    report = read_json(resolve_repo_path(entry["report"]))
    snapshot = snapshot_directory(artifact, str(report.get("workflow_result", report.get("result", "fail"))))
    notes: list[str] = []
    final_commit = base_commit
    bundle = None if snapshot is None else snapshot / "commits.bundle"
    if bundle is not None and bundle.is_file():
        fetched = run_command(
            ["git", "fetch", "--quiet", str(bundle), "HEAD"], checkout, timeout=timeout
        )
        if fetched.returncode != 0:
            raise RuntimeError(f"git fetch bundle failed: {fetched.stderr.strip()}")
        resolved = run_command(["git", "rev-parse", "FETCH_HEAD"], checkout, timeout=timeout)
        if resolved.returncode != 0:
            raise RuntimeError(f"cannot resolve bundle HEAD: {resolved.stderr.strip()}")
        final_commit = resolved.stdout.strip()
    else:
        notes.append("no saved commit bundle; scoring starts from the fixed base")

    checked_out = run_command(
        ["git", "checkout", "--quiet", "--detach", final_commit], checkout, timeout=timeout
    )
    if checked_out.returncode != 0:
        raise RuntimeError(f"git checkout failed: {checked_out.stderr.strip()}")
    ancestor = run_command(
        ["git", "merge-base", "--is-ancestor", base_commit, final_commit],
        checkout,
        timeout=timeout,
    )
    if ancestor.returncode != 0:
        raise RuntimeError("fixed benchmark base is not an ancestor of the saved result")

    if snapshot is not None:
        apply_saved_diff(checkout, snapshot / "index.diff", timeout)
        apply_saved_diff(checkout, snapshot / "worktree.diff", timeout)

    tests_dir = checkout / "tests"
    tests_dir.mkdir(exist_ok=True)
    for fixture, _ in FIXTURES.values():
        shutil.copyfile(SCRIPT_DIR / "fixtures" / fixture, tests_dir / fixture)
    return checkout, final_commit, notes


def run_fixture(
    checkout: Path,
    run_id: str,
    feature: str,
    work_root: Path,
    log_root: Path,
    timeout: int,
) -> tuple[str, dict[str, Any]]:
    fixture, test_name = FIXTURES[feature]
    target = Path(fixture).stem
    argv = ["cargo", "test", "--test", target, test_name, "--", "--exact"]
    environment = dict(os.environ)
    environment.update(
        {
            "CARGO_TARGET_DIR": str(work_root / "cargo-target"),
            "CARGO_TERM_COLOR": "never",
            "WORK_LEAF_CONTEXT_BUNDLE_DIR": str(work_root / "runtime" / run_id / "bundles"),
            "WORK_LEAF_COMMAND_TMPDIR": str(work_root / "runtime" / run_id / "commands"),
        }
    )
    started = time.monotonic()
    try:
        completed = run_command(argv, checkout, timeout=timeout, environment=environment)
        status = "pass" if completed.returncode == 0 else "fail"
        exit_code: int | None = completed.returncode
        stdout = completed.stdout
        stderr = completed.stderr
    except subprocess.TimeoutExpired as error:
        status = "timeout"
        exit_code = None
        stdout = error.stdout or ""
        stderr = error.stderr or ""
        if isinstance(stdout, bytes):
            stdout = stdout.decode(errors="replace")
        if isinstance(stderr, bytes):
            stderr = stderr.decode(errors="replace")
    duration = time.monotonic() - started
    log_dir = log_root / run_id
    log_dir.mkdir(parents=True, exist_ok=True)
    log_path = log_dir / f"{feature}.log"
    log_path.write_text(
        "\n".join(
            [
                f"command: {' '.join(argv)}",
                f"exit_code: {exit_code if exit_code is not None else 'timeout'}",
                f"duration_seconds: {duration:.3f}",
                "",
                "[stdout]",
                stdout,
                "[stderr]",
                stderr,
            ]
        ),
        encoding="utf-8",
    )
    return status, {
        "status": status,
        "exit_code": exit_code,
        "duration_seconds": round(duration, 3),
        "log": str(log_path.relative_to(STUDY_DIR)),
        "log_sha256": sha256_file(log_path),
    }


def measurement(
    report: dict[str, Any], artifact: Path, expected_model: str, expected_effort: str
) -> dict[str, Any]:
    reasons = []
    analysis_path = artifact / "observation" / "analysis.json"
    analysis: dict[str, Any] = {}
    try:
        analysis = read_json(analysis_path)
    except (OSError, json.JSONDecodeError) as error:
        reasons.append(f"observer analysis is unreadable: {error}")

    if report.get("measurement_status") != "complete":
        reasons.append(f"measurement status is {report.get('measurement_status', 'missing')}")
    if report.get("agent_model") != expected_model:
        reasons.append(f"model is {report.get('agent_model', 'missing')}")
    if report.get("agent_reasoning_effort") != expected_effort:
        reasons.append(f"reasoning effort is {report.get('agent_reasoning_effort', 'missing')}")

    if analysis and analysis.get("capture_complete") is not True:
        reasons.append("observer analysis marks the capture incomplete")
    strata = analysis.get("model_strata", []) if analysis else []
    if not strata:
        reasons.append("observer analysis has no provider model stratum")
    for stratum in strata:
        if stratum.get("model") != expected_model or stratum.get("effort") != expected_effort:
            reasons.append(
                "observer model stratum is "
                f"{stratum.get('model', 'missing')}/{stratum.get('effort', 'missing')}"
            )

    analysis_usage = (
        analysis.get("usage_scopes", {}).get("total_workflow", {}) if analysis else {}
    )
    if not isinstance(analysis_usage, dict):
        analysis_usage = {}
    usage = {field: int(analysis_usage.get(field, 0)) for field in USAGE_FIELDS}
    report_usage = report.get("total_workflow_usage")
    if not isinstance(report_usage, dict):
        reasons.append("report total-workflow usage is missing")
    else:
        normalized_report_usage = {
            field: int(report_usage.get(field, 0)) for field in USAGE_FIELDS
        }
        if normalized_report_usage != usage:
            reasons.append("report total-workflow usage does not match observer analysis")
    if int(usage.get("raw_input_plus_output", 0)) <= 0:
        reasons.append("raw total-workflow tokens are zero")
    if int(usage.get("uncached_input_plus_output", 0)) <= 0:
        reasons.append("uncached total-workflow tokens are zero")
    return {
        "usable": not reasons,
        "reasons": reasons,
        "usage": usage,
        "analysis": str(analysis_path),
        "analysis_sha256": sha256_file(analysis_path) if analysis_path.is_file() else None,
    }


def score_entry(
    entry: dict[str, Any], manifest: dict[str, Any], work_root: Path, timeout: int
) -> dict[str, Any]:
    report_path = resolve_repo_path(entry["report"])
    report = read_json(report_path)
    artifact = resolve_repo_path(entry["artifact"])
    workflow_result = str(report.get("workflow_result", report.get("result", "unknown")))
    base: dict[str, Any] = {
        "id": entry["id"],
        "workflow": entry["workflow"],
        "workflow_result": workflow_result,
        "report": entry["report"],
        "report_sha256": sha256_file(report_path),
        "measurement": measurement(
            report, artifact, manifest["model"], manifest["reasoning_effort"]
        ),
    }
    try:
        checkout, final_commit, notes = prepare_checkout(
            entry,
            resolve_repo_path(manifest["source_repo"]),
            manifest["base_commit"],
            work_root,
            timeout,
        )
        checks: dict[str, str] = {}
        details = {}
        for feature in FIXTURES:
            status, detail = run_fixture(
                checkout, entry["id"], feature, work_root, STUDY_DIR / "scorer" / "logs", timeout
            )
            checks[feature] = status
            details[feature] = detail
        completed = sum(status == "pass" for status in checks.values())
        return {
            **base,
            "final_commit": final_commit,
            "materialization_notes": notes,
            "checks": checks,
            "check_details": details,
            "completed_features": completed,
        }
    except (OSError, RuntimeError) as error:
        return {
            **base,
            "checks": {feature: "not-run" for feature in FIXTURES},
            "completed_features": 0,
            "scoring_error": str(error),
        }


def reduction(direct: int, work_leaf: int) -> float | None:
    if direct <= 0:
        return None
    return round((direct - work_leaf) / direct * 100.0, 3)


def build_comparison(runs: Sequence[dict[str, Any]]) -> dict[str, Any]:
    by_workflow = {run["workflow"]: run for run in runs}
    direct = by_workflow.get("direct-sequential")
    work_leaf = by_workflow.get("work-leaf-concurrent")
    if direct is None or work_leaf is None:
        return {"usable": False, "reasons": ["both pilot workflows are required"]}
    reasons = []
    measurements_usable = bool(
        direct["measurement"]["usable"] and work_leaf["measurement"]["usable"]
    )
    if not direct["measurement"]["usable"]:
        reasons.append("direct token measurement is not usable")
    if not work_leaf["measurement"]["usable"]:
        reasons.append("Work Leaf token measurement is not usable")
    completed_count_match = direct["completed_features"] == work_leaf["completed_features"]
    quality_match = direct["checks"] == work_leaf["checks"]
    if not completed_count_match:
        reasons.append("the two saved implementations completed different numbers of requested features")
    elif not quality_match:
        reasons.append("the two saved implementations completed different requested features")
    direct_usage = direct["measurement"]["usage"]
    work_leaf_usage = work_leaf["measurement"]["usage"]
    raw_direct = int(direct_usage.get("raw_input_plus_output", 0))
    raw_work_leaf = int(work_leaf_usage.get("raw_input_plus_output", 0))
    uncached_direct = int(direct_usage.get("uncached_input_plus_output", 0))
    uncached_work_leaf = int(work_leaf_usage.get("uncached_input_plus_output", 0))
    return {
        "usable": not reasons,
        "reasons": reasons,
        "token_measurements_usable": measurements_usable,
        "quality_match_in_this_pair": quality_match,
        "completed_features": {
            "direct_sequential": direct["completed_features"],
            "work_leaf_concurrent": work_leaf["completed_features"],
        },
        "raw_tokens": {
            "direct_sequential": raw_direct,
            "work_leaf_concurrent": raw_work_leaf,
            "work_leaf_reduction_percent": reduction(raw_direct, raw_work_leaf),
        },
        "uncached_tokens": {
            "direct_sequential": uncached_direct,
            "work_leaf_concurrent": uncached_work_leaf,
            "work_leaf_reduction_percent": reduction(uncached_direct, uncached_work_leaf),
        },
        "interpretation_limit": "One pilot pair is descriptive and cannot establish an average effect or statistical confidence.",
    }


def markdown_report(result: dict[str, Any]) -> str:
    runs = result["runs"]
    comparison = result["comparison"]
    lines = [
        "# Provisional Pilot Result",
        "",
        "This report covers exactly one normal concurrent Work Leaf run and one fair direct sequential Codex run. It is a protocol check and descriptive first result, not a statistically reliable estimate.",
        "",
        "## Saved Implementations",
        "",
        "| Workflow | Workflow result | Visual | `/status` | Completion | Features | Token capture |",
        "| --- | --- | --- | --- | --- | ---: | --- |",
    ]
    labels = {"work-leaf-concurrent": "Concurrent Work Leaf", "direct-sequential": "Direct sequential Codex"}
    for run in runs:
        checks = run["checks"]
        lines.append(
            f"| {labels.get(run['workflow'], run['workflow'])} | {run['workflow_result']} | {checks['visual']} | {checks['status']} | {checks['completion']} | {run['completed_features']}/3 | {'usable' if run['measurement']['usable'] else 'not usable'} |"
        )
    lines.extend(["", "## Tokens", ""])
    if comparison["token_measurements_usable"]:
        raw = comparison["raw_tokens"]
        uncached = comparison["uncached_tokens"]
        lines.extend(
            [
                f"- Raw input plus output: Work Leaf {raw['work_leaf_concurrent']:,}; direct {raw['direct_sequential']:,}; arithmetic difference {raw['work_leaf_reduction_percent']:.3f}% lower for Work Leaf.",
                f"- Uncached input plus output: Work Leaf {uncached['work_leaf_concurrent']:,}; direct {uncached['direct_sequential']:,}; arithmetic difference {uncached['work_leaf_reduction_percent']:.3f}% lower for Work Leaf.",
            ]
        )
        if comparison["usable"]:
            lines.append(
                "The exact three feature outcomes match in this pair, so these values form a descriptive comparable-output efficiency result."
            )
        else:
            lines.append(
                "The quality outcomes differ, so this arithmetic is useful observed data but not a comparable-output efficiency claim."
            )
            lines.extend(f"- {reason}." for reason in comparison["reasons"])
    else:
        lines.append("The pilot has no usable two-workflow token comparison:")
        lines.extend(f"- {reason}." for reason in comparison["reasons"])
    lines.extend(
        [
            "",
            "## Limits",
            "",
            "One run per workflow cannot estimate normal variability, an average quality difference, or statistical confidence. No mechanism attribution is part of this pilot. All outcomes are retained; a failed or partial implementation is data rather than a reason to discard the run.",
            "",
            "The study stops here pending user review.",
            "",
        ]
    )
    return "\n".join(lines)


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser()
    parser.add_argument("--manifest", type=Path, default=STUDY_DIR / "manifest.json")
    parser.add_argument("--output", type=Path, default=STUDY_DIR / "result.json")
    parser.add_argument("--markdown", type=Path, default=STUDY_DIR / "PROVISIONAL-RESULT.md")
    parser.add_argument("--work-dir", type=Path)
    parser.add_argument("--timeout-seconds", type=int, default=900)
    return parser.parse_args()


def main() -> int:
    args = parse_args()
    manifest = read_json(args.manifest.resolve())
    temporary: tempfile.TemporaryDirectory[str] | None = None
    if args.work_dir:
        work_root = args.work_dir.resolve()
        work_root.mkdir(parents=True, exist_ok=True)
    else:
        temporary = tempfile.TemporaryDirectory(prefix="work-leaf-fair-pilot-score-")
        work_root = Path(temporary.name)
    try:
        runs = [
            score_entry(entry, manifest, work_root, args.timeout_seconds)
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
            "fixtures": {
                name: sha256_file(SCRIPT_DIR / "fixtures" / fixture)
                for name, (fixture, _) in FIXTURES.items()
            },
            "runs": runs,
            "comparison": build_comparison(runs),
        }
        write_json(args.output.resolve(), result)
        args.markdown.resolve().write_text(markdown_report(result), encoding="utf-8")
    finally:
        if temporary is not None:
            temporary.cleanup()
    print(args.markdown.resolve())
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
