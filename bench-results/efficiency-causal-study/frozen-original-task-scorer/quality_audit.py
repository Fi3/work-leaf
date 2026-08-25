#!/usr/bin/env python3
"""Score saved three-feature implementations without launching an agent."""

from __future__ import annotations

import argparse
import hashlib
import itertools
import json
import os
import random
import re
import shutil
import subprocess
import tempfile
import time
from pathlib import Path
from typing import Any, Sequence


SCRIPT_DIR = Path(__file__).resolve().parent
REPO_ROOT = SCRIPT_DIR.parents[2]
DEFAULT_MANIFEST = SCRIPT_DIR / "manifest.json"
DEFAULT_OUTPUT = SCRIPT_DIR / "result.json"
FIXTURES = {
    "visual": ("quality_visual.rs", "quality_visual_behavior"),
    "slash_status": ("quality_commands.rs", "quality_status_behavior"),
    "slash_fork_continuation": (
        "quality_commands.rs",
        "quality_fork_continuation_behavior",
    ),
    "completion": ("quality_completion.rs", "quality_completion_behavior"),
}
SCORED_FEATURES = {
    "visual": "visual",
    "commands": "slash_status",
    "completion": "completion",
}
SUPPLEMENTAL_CHECKS = ("slash_fork_continuation",)


def load_manifest(path: Path) -> dict[str, Any]:
    return json.loads(path.read_text(encoding="utf-8"))


def write_json(path: Path, value: Any) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    temporary = path.with_suffix(path.suffix + ".tmp")
    temporary.write_text(
        json.dumps(value, indent=2, sort_keys=True) + "\n", encoding="utf-8"
    )
    temporary.replace(path)


def sha256_file(path: Path) -> str:
    digest = hashlib.sha256()
    with path.open("rb") as stream:
        for chunk in iter(lambda: stream.read(1024 * 1024), b""):
            digest.update(chunk)
    return digest.hexdigest()


def feature_scores(checks: dict[str, str]) -> dict[str, bool]:
    return {
        "visual": checks.get("visual") == "pass",
        "commands": checks.get("slash_status") == "pass",
        "completion": checks.get("completion") == "pass",
    }


def scored_run(
    run_id: str,
    workflow: str,
    workflow_result: str,
    checks: dict[str, str],
    **extra: Any,
) -> dict[str, Any]:
    features = feature_scores(checks)
    return {
        "id": run_id,
        "workflow": workflow,
        "workflow_result": workflow_result,
        "checks": checks,
        "features": features,
        "completed_features": sum(features.values()),
        **extra,
    }


def summarize(runs: Sequence[dict[str, Any]]) -> dict[str, Any]:
    result: dict[str, Any] = {}
    for workflow in sorted({run["workflow"] for run in runs}):
        selected = [run for run in runs if run["workflow"] == workflow]
        feature_counts = {
            feature: sum(bool(run["features"][feature]) for run in selected)
            for feature in ("visual", "commands", "completion")
        }
        subcheck_counts = {
            check: sum(run["checks"].get(check) == "pass" for run in selected)
            for check in FIXTURES
        }
        distribution = {
            str(count): sum(run["completed_features"] == count for run in selected)
            for count in range(4)
        }
        run_count = len(selected)
        result[workflow] = {
            "runs": run_count,
            "workflow_passes": sum(
                run.get("workflow_result") == "pass" for run in selected
            ),
            "feature_pass_counts": feature_counts,
            "feature_pass_rates": {
                feature: round(count / run_count, 6)
                for feature, count in feature_counts.items()
            },
            "subcheck_pass_counts": subcheck_counts,
            "completed_feature_distribution": distribution,
            "all_three_features": distribution["3"],
            "mean_completed_features": round(
                sum(run["completed_features"] for run in selected) / run_count, 6
            ),
        }
    return result


def comparison(runs: Sequence[dict[str, Any]]) -> dict[str, Any] | None:
    sequential = [
        run["completed_features"]
        for run in runs
        if run["workflow"] == "sequential"
    ]
    work_leaf = [
        run["completed_features"]
        for run in runs
        if run["workflow"] == "work-leaf"
    ]
    if not sequential or not work_leaf:
        return None

    observed = sum(work_leaf) / len(work_leaf) - sum(sequential) / len(sequential)
    combined = sequential + work_leaf
    null_differences = []
    for work_leaf_indices in itertools.combinations(range(len(combined)), len(work_leaf)):
        selected = set(work_leaf_indices)
        permuted_work_leaf = [
            value for index, value in enumerate(combined) if index in selected
        ]
        permuted_sequential = [
            value for index, value in enumerate(combined) if index not in selected
        ]
        null_differences.append(
            sum(permuted_work_leaf) / len(permuted_work_leaf)
            - sum(permuted_sequential) / len(permuted_sequential)
        )
    extreme = sum(abs(value) >= abs(observed) - 1e-12 for value in null_differences)

    generator = random.Random(228)
    bootstrap = []
    for _ in range(20_000):
        sampled_sequential = [generator.choice(sequential) for _ in sequential]
        sampled_work_leaf = [generator.choice(work_leaf) for _ in work_leaf]
        bootstrap.append(
            sum(sampled_work_leaf) / len(sampled_work_leaf)
            - sum(sampled_sequential) / len(sampled_sequential)
        )
    bootstrap.sort()
    low = bootstrap[int(len(bootstrap) * 0.025)]
    high = bootstrap[int(len(bootstrap) * 0.975)]

    return {
        "estimand": "work-leaf mean completed features minus sequential mean",
        "observed_difference": round(observed, 6),
        "bootstrap_95_percent_interval": [round(low, 6), round(high, 6)],
        "exact_label_permutation_two_sided_p": round(
            extreme / len(null_differences), 6
        ),
        "interpretation_limit": (
            "Descriptive small-sample comparison; it does not prove equality or superiority."
        ),
    }


def same_block_comparison(runs: Sequence[dict[str, Any]]) -> dict[str, Any]:
    grouped: dict[tuple[str, str], dict[str, dict[str, Any]]] = {}
    unmatched_ids = []
    for run in runs:
        block = run.get("block")
        attempt = run.get("attempt")
        workflow = run.get("workflow")
        if block is None or attempt is None or workflow not in {
            "sequential",
            "work-leaf",
        }:
            unmatched_ids.append(run["id"])
            continue
        key = (str(block), str(attempt))
        bucket = grouped.setdefault(key, {})
        if workflow in bucket:
            raise ValueError(
                f"duplicate {workflow} run for block {block} attempt {attempt}"
            )
        bucket[workflow] = run

    pair_rows = []
    for (block, attempt), bucket in sorted(grouped.items()):
        if set(bucket) != {"sequential", "work-leaf"}:
            unmatched_ids.extend(run["id"] for run in bucket.values())
            continue
        sequential = bucket["sequential"]
        work_leaf = bucket["work-leaf"]
        pair_rows.append(
            {
                "block": block,
                "attempt": int(attempt) if attempt.isdigit() else attempt,
                "sequential_id": sequential["id"],
                "work_leaf_id": work_leaf["id"],
                "sequential_completed_features": sequential["completed_features"],
                "work_leaf_completed_features": work_leaf["completed_features"],
                "difference_work_leaf_minus_sequential": (
                    work_leaf["completed_features"]
                    - sequential["completed_features"]
                ),
            }
        )

    if not pair_rows:
        return {
            "pairs": 0,
            "pair_results": [],
            "unmatched_run_ids": sorted(unmatched_ids),
            "interpretation_limit": "No same-block attempt pairs are available.",
        }

    sequential_values = [
        row["sequential_completed_features"] for row in pair_rows
    ]
    work_leaf_values = [row["work_leaf_completed_features"] for row in pair_rows]
    differences = [
        row["difference_work_leaf_minus_sequential"] for row in pair_rows
    ]
    observed = sum(differences) / len(differences)

    generator = random.Random(228)
    bootstrap = []
    for _ in range(20_000):
        sampled = [generator.choice(differences) for _ in differences]
        bootstrap.append(sum(sampled) / len(sampled))
    bootstrap.sort()

    sign_flipped = []
    for signs in itertools.product((-1, 1), repeat=len(differences)):
        sign_flipped.append(
            sum(sign * value for sign, value in zip(signs, differences, strict=True))
            / len(differences)
        )
    extreme = sum(abs(value) >= abs(observed) - 1e-12 for value in sign_flipped)

    return {
        "pairs": len(pair_rows),
        "mean_completed_features": {
            "sequential": round(sum(sequential_values) / len(pair_rows), 6),
            "work-leaf": round(sum(work_leaf_values) / len(pair_rows), 6),
        },
        "observed_mean_paired_difference": round(observed, 6),
        "bootstrap_95_percent_interval": [
            round(bootstrap[int(len(bootstrap) * 0.025)], 6),
            round(bootstrap[int(len(bootstrap) * 0.975)], 6),
        ],
        "exact_sign_flip_two_sided_p": round(extreme / len(sign_flipped), 6),
        "pair_results": pair_rows,
        "unmatched_run_ids": sorted(unmatched_ids),
        "interpretation_limit": (
            "Descriptive paired comparison; four pairs cannot establish equivalence."
        ),
    }


def historical_sanity(references: Sequence[dict[str, Any]]) -> dict[str, Any]:
    mean = round(
        sum(run["completed_features"] for run in references) / len(references), 6
    )
    return {
        "reference_runs": len(references),
        "mean_completed_features": mean,
    }


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


def command_text(argv: Sequence[str]) -> str:
    return " ".join(subprocess.list2cmdline([part]) for part in argv)


def locate_bundle(artifact: Path) -> Path:
    candidates = sorted(artifact.glob("**/commits.bundle"))
    if len(candidates) != 1:
        raise RuntimeError(
            f"{artifact} must contain exactly one commits.bundle; found {len(candidates)}"
        )
    return candidates[0]


def report_metadata(repo_root: Path, entry: dict[str, Any]) -> dict[str, Any]:
    if "report" not in entry:
        return {"workflow_result": entry.get("workflow_result", "unknown")}
    report_path = repo_root / entry["report"]
    report = json.loads(report_path.read_text(encoding="utf-8"))
    metadata: dict[str, Any] = {
        "workflow_result": report.get("result", "unknown"),
        "report": entry["report"],
        "report_sha256": sha256_file(report_path),
        "base_commit": report.get("base_commit"),
        "model": report.get("agent_model"),
        "reasoning_effort": report.get("agent_reasoning_effort"),
        "agent_cli_version": report.get("agent_cli_version"),
    }
    observation = report.get("observation")
    if isinstance(observation, str):
        observation_path = Path(observation)
        if not observation_path.is_absolute():
            observation_path = repo_root / observation_path
        analysis = observation_path / "analysis.json"
        if analysis.is_file():
            payload = json.loads(analysis.read_text(encoding="utf-8"))
            usage = payload.get("usage_scopes", {}).get("total_workflow")
            if isinstance(usage, dict):
                metadata["exact_token_usage"] = usage
                metadata["analysis_sha256"] = sha256_file(analysis)
    return metadata


class SavedImplementationScorer:
    def __init__(
        self,
        repo_root: Path,
        source_repo: Path,
        base_commit: str,
        output: Path,
        work_root: Path,
        timeout: int,
    ) -> None:
        self.repo_root = repo_root
        self.source_repo = source_repo
        self.base_commit = base_commit
        self.output = output
        self.work_root = work_root
        self.timeout = timeout
        self.fixture_dir = SCRIPT_DIR / "fixtures"
        self.log_root = output.parent / "logs"
        self.target_dir = work_root / "cargo-target"

    def prepare_checkout(self, entry: dict[str, Any], bundle: Path) -> tuple[Path, str]:
        checkout = self.work_root / "checkouts" / entry["id"]
        checkout.parent.mkdir(parents=True, exist_ok=True)
        clone = run_command(
            [
                "git",
                "clone",
                "--quiet",
                "--no-checkout",
                "--shared",
                str(self.source_repo),
                str(checkout),
            ],
            self.work_root,
            timeout=self.timeout,
        )
        if clone.returncode != 0:
            raise RuntimeError(f"git clone failed: {clone.stderr.strip()}")
        fetched = run_command(
            ["git", "fetch", "--quiet", str(bundle), "HEAD"],
            checkout,
            timeout=self.timeout,
        )
        if fetched.returncode != 0:
            raise RuntimeError(f"git fetch bundle failed: {fetched.stderr.strip()}")
        head = run_command(
            ["git", "rev-parse", "FETCH_HEAD"],
            checkout,
            timeout=self.timeout,
        )
        if head.returncode != 0:
            raise RuntimeError(f"git rev-parse failed: {head.stderr.strip()}")
        final_commit = head.stdout.strip()
        checkout_result = run_command(
            ["git", "checkout", "--quiet", "--detach", final_commit],
            checkout,
            timeout=self.timeout,
        )
        if checkout_result.returncode != 0:
            raise RuntimeError(
                f"git checkout failed: {checkout_result.stderr.strip()}"
            )
        ancestor = run_command(
            [
                "git",
                "merge-base",
                "--is-ancestor",
                self.base_commit,
                final_commit,
            ],
            checkout,
            timeout=self.timeout,
        )
        if ancestor.returncode != 0:
            raise RuntimeError("fixed benchmark base is not an ancestor of bundle HEAD")
        tests_dir = checkout / "tests"
        tests_dir.mkdir(exist_ok=True)
        for fixture_name, _ in set(FIXTURES.values()):
            shutil.copyfile(self.fixture_dir / fixture_name, tests_dir / fixture_name)
        return checkout, final_commit

    def run_fixture(
        self, checkout: Path, run_id: str, check: str
    ) -> tuple[str, dict[str, Any]]:
        fixture_name, test_name = FIXTURES[check]
        target = Path(fixture_name).stem
        argv = [
            "cargo",
            "test",
            "--test",
            target,
            test_name,
            "--",
            "--exact",
        ]
        environment = dict(os.environ)
        environment.update(
            {
                "CARGO_TARGET_DIR": str(self.target_dir),
                "CARGO_TERM_COLOR": "never",
                "WORK_LEAF_CONTEXT_BUNDLE_DIR": str(
                    self.work_root / "runtime" / run_id / "context-bundles"
                ),
                "WORK_LEAF_COMMAND_TMPDIR": str(
                    self.work_root / "runtime" / run_id / "commands"
                ),
            }
        )
        started = time.monotonic()
        try:
            completed = run_command(
                argv,
                checkout,
                timeout=self.timeout,
                environment=environment,
            )
            duration = time.monotonic() - started
            status = "pass" if completed.returncode == 0 else "fail"
            stdout = completed.stdout
            stderr = completed.stderr
            exit_code: int | None = completed.returncode
        except subprocess.TimeoutExpired as error:
            duration = time.monotonic() - started
            status = "timeout"
            stdout = error.stdout or ""
            stderr = error.stderr or ""
            if isinstance(stdout, bytes):
                stdout = stdout.decode(errors="replace")
            if isinstance(stderr, bytes):
                stderr = stderr.decode(errors="replace")
            exit_code = None

        log_dir = self.log_root / run_id
        log_dir.mkdir(parents=True, exist_ok=True)
        log_path = log_dir / f"{check}.log"
        log_path.write_text(
            "\n".join(
                [
                    f"command: {command_text(argv)}",
                    f"cwd: {checkout}",
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
            "log": str(log_path.relative_to(self.output.parent)),
            "log_sha256": sha256_file(log_path),
        }

    def score(self, entry: dict[str, Any]) -> dict[str, Any]:
        artifact = self.repo_root / entry["artifact"]
        bundle = locate_bundle(artifact)
        metadata = report_metadata(self.repo_root, entry)
        workflow_result = metadata["workflow_result"]
        base = {
            "cohort": entry["cohort"],
            "artifact": entry["artifact"],
            "bundle": str(bundle.relative_to(self.repo_root)),
            "bundle_sha256": sha256_file(bundle),
            **{
                key: entry[key]
                for key in ("block", "attempt")
                if key in entry
            },
            **{
                key: value
                for key, value in metadata.items()
                if key != "workflow_result"
            },
        }
        try:
            checkout, final_commit = self.prepare_checkout(entry, bundle)
            checks: dict[str, str] = {}
            details = {}
            for check in FIXTURES:
                status, detail = self.run_fixture(checkout, entry["id"], check)
                checks[check] = status
                details[check] = detail
            return scored_run(
                entry["id"],
                entry["workflow"],
                workflow_result,
                checks,
                final_commit=final_commit,
                check_details=details,
                **base,
            )
        except (OSError, RuntimeError) as error:
            checks = {check: "not-run" for check in FIXTURES}
            return scored_run(
                entry["id"],
                entry["workflow"],
                workflow_result,
                checks,
                audit_error=str(error),
                **base,
            )


def historical_evidence(repo_root: Path, manifest: dict[str, Any]) -> list[dict[str, Any]]:
    evidence = []
    for item in manifest["historical_status_evidence"]:
        path = repo_root / item["audit"]
        text = path.read_text(encoding="utf-8")
        evidence.append(
            {
                **item,
                "sha256": sha256_file(path),
                "records_status": "/status" in text
                or "thread/read" in text
                or "slash_status" in text,
                "records_fork": "/fork" in text or "thread/fork" in text,
            }
        )
    return evidence


def build_result(
    manifest_path: Path,
    manifest: dict[str, Any],
    runs: list[dict[str, Any]],
    complete: bool,
) -> dict[str, Any]:
    current = [run for run in runs if run["cohort"] == "current"]
    calibration_references = [
        run for run in runs if run["cohort"] == "historical-reference"
    ]
    historical_work_leaf = [
        run
        for run in runs
        if run["cohort"] == "historical-work-leaf-sanity"
    ]
    reference_status_passes = sum(
        run["checks"].get("slash_status") == "pass"
        for run in calibration_references
    )
    reference_command_passes = sum(
        run["features"].get("commands") is True
        for run in calibration_references
    )
    current_summary = summarize(current) if current else {}
    return {
        "schema_version": 2,
        "complete": complete,
        "method": (
            "Offline final-HEAD behavior tests; no benchmark, agent, or provider service is launched."
        ),
        "manifest": str(manifest_path.relative_to(REPO_ROOT)),
        "manifest_sha256": sha256_file(manifest_path),
        "scorer_sha256": sha256_file(Path(__file__).resolve()),
        "fixtures": {
            fixture_name: {
                "sha256": sha256_file(SCRIPT_DIR / "fixtures" / fixture_name),
                "tests": sorted(
                    test_name
                    for candidate_name, test_name in FIXTURES.values()
                    if candidate_name == fixture_name
                ),
            }
            for fixture_name in sorted({name for name, _ in FIXTURES.values()})
        },
        "base_commit": manifest["base_commit"],
        "scoring_contract": {
            "original_task": manifest["original_task"],
            "scored_features": SCORED_FEATURES,
            "supplemental_checks": list(SUPPLEMENTAL_CHECKS),
        },
        "historical_status_evidence": historical_evidence(REPO_ROOT, manifest),
        "historical_sequential_calibration_regression": {
            "runs": len(calibration_references),
            "status_passes": reference_status_passes,
            "original_command_feature_passes": reference_command_passes,
            "all_status_references_pass": bool(calibration_references)
            and reference_status_passes == len(calibration_references),
            "all_original_command_features_pass": bool(calibration_references)
            and reference_command_passes == len(calibration_references),
        },
        "historical_sequential_calibration_summary": (
            summarize(calibration_references) if calibration_references else {}
        ),
        "historical_work_leaf_sanity_summary": (
            summarize(historical_work_leaf) if historical_work_leaf else {}
        ),
        "historical_work_leaf_sanity_comparison": (
            {
                **historical_sanity(historical_work_leaf),
                "interpretation_limit": (
                    "Sequentially scheduled historical Work Leaf is a behavior sanity check, "
                    "not the normal concurrent comparison cohort or an equivalence gate."
                ),
            }
            if historical_work_leaf
            else None
        ),
        "runs": runs,
        "current_summary": current_summary,
        "current_comparison": comparison(current),
        "same_block_comparison": same_block_comparison(current),
    }


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(
        description="Score saved sequential and Work Leaf implementations feature by feature."
    )
    parser.add_argument("--manifest", type=Path, default=DEFAULT_MANIFEST)
    parser.add_argument("--output", type=Path, default=DEFAULT_OUTPUT)
    parser.add_argument("--source-repo", type=Path)
    parser.add_argument("--work-dir", type=Path)
    parser.add_argument("--timeout-seconds", type=int, default=900)
    parser.add_argument("--only", action="append", default=[])
    return parser.parse_args()


def main() -> int:
    args = parse_args()
    manifest_path = args.manifest.resolve()
    output = args.output.resolve()
    manifest = load_manifest(manifest_path)
    source_repo = (args.source_repo or Path(manifest["source_repo"])).resolve()
    selected = [
        entry
        for entry in manifest["runs"]
        if not args.only or entry["id"] in set(args.only)
    ]
    if args.only and len(selected) != len(set(args.only)):
        known = {entry["id"] for entry in manifest["runs"]}
        missing = sorted(set(args.only) - known)
        raise SystemExit(f"unknown run ids: {', '.join(missing)}")

    temporary: tempfile.TemporaryDirectory[str] | None = None
    if args.work_dir:
        work_root = args.work_dir.resolve()
        work_root.mkdir(parents=True, exist_ok=True)
    else:
        temporary = tempfile.TemporaryDirectory(prefix="work-leaf-quality-audit-")
        work_root = Path(temporary.name)

    scorer = SavedImplementationScorer(
        REPO_ROOT,
        source_repo,
        manifest["base_commit"],
        output,
        work_root,
        args.timeout_seconds,
    )
    runs: list[dict[str, Any]] = []
    try:
        for entry in selected:
            print(f"scoring {entry['id']}", flush=True)
            runs.append(scorer.score(entry))
            write_json(
                output,
                build_result(manifest_path, manifest, runs, complete=False),
            )
        result = build_result(manifest_path, manifest, runs, complete=True)
        write_json(output, result)
    finally:
        if temporary is not None:
            temporary.cleanup()

    print(output)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
