#!/usr/bin/env python3
"""Evaluate accounting precision or replay saved evidence without provider calls.

The evaluate input contains metrics keyed by raw_input_plus_output and
uncached_input_plus_output. Each metric contains direct and work_leaf objects with
lower and upper token bounds. An optional quality object is retained separately.
Exit status is 0 for a precision pass, 1 for a wide interval, and 2 for invalid
inputs. Neither a positive saving nor quality equivalence is required or inferred.
"""

import argparse
from bisect import bisect_right
from collections import Counter, defaultdict
import hashlib
import json
import math
from pathlib import Path
import statistics
import sys


METRICS = ("raw_input_plus_output", "uncached_input_plus_output")
USAGE_FIELDS = (
    "inputTokens", "cachedInputTokens", "outputTokens", "reasoningOutputTokens", "totalTokens",
)
ROOT = Path(__file__).resolve().parents[2]


def finite_number(value, label):
    if type(value) not in (int, float) or not math.isfinite(value):
        raise ValueError(f"{label} must be a finite number")
    return value


def token_bounds(value, label, *, positive=False):
    if not isinstance(value, dict) or not {"lower", "upper"} <= value.keys():
        raise ValueError(f"{label} requires lower and upper token bounds")
    lower = finite_number(value["lower"], f"{label}.lower")
    upper = finite_number(value["upper"], f"{label}.upper")
    if lower > upper or lower < 0 or (positive and lower == 0):
        raise ValueError(f"{label} has invalid or nonpositive denominator bounds")
    return {"lower": lower, "upper": upper}


def saving_interval(direct, work_leaf, max_width_pp=5.0):
    direct = token_bounds(direct, "direct", positive=True)
    work_leaf = token_bounds(work_leaf, "work_leaf")
    target = finite_number(max_width_pp, "max_width_pp")
    if target <= 0:
        raise ValueError("max_width_pp must be positive")
    # For nonnegative workload tokens and a positive baseline, saving increases
    # with the baseline and decreases with workload usage.
    lower = 100.0 * (direct["lower"] - work_leaf["upper"]) / direct["lower"]
    upper = 100.0 * (direct["upper"] - work_leaf["lower"]) / direct["upper"]
    width = upper - lower
    if not all(math.isfinite(value) for value in (lower, upper, width)):
        raise ValueError("saving interval arithmetic is not finite")
    return {
        "direct": direct,
        "work_leaf": work_leaf,
        "saving_percent": {"lower": lower, "upper": upper},
        "width_percentage_points": width,
        "maximum_width_percentage_points": target,
        "precision_pass": width <= target,
    }


def evaluate_precision(metrics, max_width_pp=5.0, quality=None):
    if not isinstance(metrics, dict) or not set(METRICS) <= metrics.keys():
        raise ValueError("both raw and uncached input-plus-output metrics are required")
    results = {}
    for metric in METRICS:
        value = metrics[metric]
        if not isinstance(value, dict) or not {"direct", "work_leaf"} <= value.keys():
            raise ValueError(f"{metric} requires direct and work_leaf token intervals")
        results[metric] = saving_interval(value["direct"], value["work_leaf"], max_width_pp)
    return {
        "schema_version": 1,
        "metrics": results,
        "precision_pass": all(value["precision_pass"] for value in results.values()),
        "scope": "accounting_uncertainty_only",
        "sampling_confidence_interval": False,
        "quality": quality,
        "quality_equivalence": "not_assessed_by_precision_gate",
    }


def checked_usage(value):
    if not isinstance(value, dict):
        raise ValueError("usage must be an object")
    if any(type(value.get(key)) is not int or value[key] < 0 for key in USAGE_FIELDS):
        raise ValueError("usage requires nonnegative integer counts")
    if (value["cachedInputTokens"] > value["inputTokens"]
            or value["reasoningOutputTokens"] > value["outputTokens"]
            or value["totalTokens"] != value["inputTokens"] + value["outputTokens"]):
        raise ValueError("usage has inconsistent subsets or total")
    return {key: value[key] for key in USAGE_FIELDS}


def inspect_stream(records, gaps):
    """Index advancing thread totals once; audit gaps without claiming recovery."""
    totals = {}
    usage_by_sequence = {}
    advances = defaultdict(list)
    advance_sequences = defaultdict(list)
    counts = Counter()
    for sequence, record in enumerate(records):
        method = record.get("method")
        if method in {"rawResponse/completed", "rawResponseItem/completed"}:
            counts[method] += 1
        if method != "thread/tokenUsage/updated":
            continue
        params = record.get("params") or {}
        thread_id = params.get("threadId")
        if not isinstance(thread_id, str) or not thread_id:
            raise ValueError("usage event has no thread identity")
        token_usage = params.get("tokenUsage") or {}
        total = checked_usage(token_usage.get("total"))
        last = checked_usage(token_usage.get("last"))
        previous = totals.get(thread_id, dict.fromkeys(USAGE_FIELDS, 0))
        if any(total[key] < previous[key] for key in USAGE_FIELDS):
            raise ValueError("cumulative usage regressed within a thread")
        usage_by_sequence[sequence] = (thread_id, total)
        if total != previous:
            advances[thread_id].append((sequence, total, last, params.get("turnId")))
            advance_sequences[thread_id].append(sequence)
        totals[thread_id] = total
        counts[method] += 1

    details = []
    for gap in gaps:
        thread_id = gap["thread_id"]
        previous_sequence = gap["previous_usage_sequence"]
        directive_sequence = gap["directive_sequence"]
        if previous_sequence == -1:
            baseline = dict.fromkeys(USAGE_FIELDS, 0)
        else:
            previous = usage_by_sequence.get(previous_sequence)
            if (previous is None or previous[0] != thread_id
                    or previous_sequence >= directive_sequence):
                raise ValueError("gap baseline does not identify prior same-thread usage")
            baseline = previous[1]
        index = bisect_right(advance_sequences[thread_id], directive_sequence)
        detail = {
            "thread_id": thread_id,
            "turn_id": gap["turn_id"],
            "previous_usage_sequence": previous_sequence,
            "directive_sequence": directive_sequence,
            "status": "no_later_advance",
            "later_usage_sequence": None,
            "later_turn_id": None,
            "residual": None,
        }
        if index < len(advances[thread_id]):
            sequence, total, last, turn_id = advances[thread_id][index]
            residual = {key: total[key] - baseline[key] - last[key] for key in USAGE_FIELDS}
            detail.update(
                later_usage_sequence=sequence,
                later_turn_id=turn_id,
                residual=residual,
                status=("zero_unreported_residual" if not any(residual.values())
                        else "unattributed_residual"),
            )
        details.append(detail)
    return {
        "raw_response_completed_events": counts["rawResponse/completed"],
        "raw_response_item_completed_events": counts["rawResponseItem/completed"],
        "cumulative_usage_events": counts["thread/tokenUsage/updated"],
        "unresolved_responses": len(gaps),
        "gap_status_counts": dict(sorted(Counter(row["status"] for row in details).items())),
        "gaps": details,
    }


def read_records(path):
    with path.open(encoding="utf-8") as handle:
        for line in handle:
            if not line.strip():
                continue
            value = json.loads(line)
            if not isinstance(value, dict):
                raise ValueError("provider stream contains a non-object record")
            yield value


def sha256(path):
    digest = hashlib.sha256()
    with path.open("rb") as handle:
        for chunk in iter(lambda: handle.read(1024 * 1024), b""):
            digest.update(chunk)
    return digest.hexdigest()


def replay_historical(source, repo_root=ROOT, max_width_pp=5.0):
    repo_root = Path(repo_root).resolve()
    source = Path(source).resolve()
    evidence = json.loads(source.read_text(encoding="utf-8"))
    source_records = {}

    def verify_source(relative_path, expected=None):
        path = (repo_root / relative_path).resolve()
        if not path.is_relative_to(repo_root):
            raise ValueError("historical source path leaves the repository")
        digest = sha256(path)
        if expected is not None and digest != expected:
            raise ValueError(f"saved source digest mismatch: {relative_path}")
        source_records[relative_path] = {
            "sha256": digest, "matched_saved_digest": expected is not None,
        }
        return path

    verify_source(str(source.relative_to(repo_root)))
    bound_path = evidence["accounting"]["response_bound_source"]
    verify_source(bound_path, evidence["accounting"]["response_bound_source_sha256"])
    groups = {
        group: [row for row in evidence["observations"] if row["group"] == group]
        for group in ("direct", "normal_work_leaf")
    }
    if any(not rows for rows in groups.values()):
        raise ValueError("historical evidence requires both nonempty workflow cohorts")
    metrics = {}
    for metric, row_key, group_key in (
        ("raw_input_plus_output", "raw_tokens", "raw_token_mean_interval"),
        ("uncached_input_plus_output", "uncached_tokens", "uncached_token_mean_interval"),
    ):
        intervals = {}
        for group, rows in groups.items():
            values = [token_bounds(row[row_key], row_key) for row in rows]
            interval = {
                side: statistics.fmean(row[side] for row in values)
                for side in ("lower", "upper")
            }
            saved = evidence["groups"][group][group_key]
            if any(not math.isclose(interval[side], saved[side], rel_tol=0, abs_tol=1e-7)
                   for side in ("lower", "upper")):
                raise ValueError("historical cohort interval disagrees with its observations")
            intervals["work_leaf" if group == "normal_work_leaf" else "direct"] = interval
        metrics[metric] = intervals

    quality = {
        group: {
            "observations": len(rows),
            "completed_features": sum(row["completed_features"] for row in rows),
            "possible_features": sum(len(row["feature_checks"]) for row in rows),
            "observations_retained": [
                {key: row.get(key) for key in (
                    "run_id", "feature_checks", "workflow_result", "outcome_note",
                )} for row in rows
            ],
        } for group, rows in groups.items()
    }
    result = evaluate_precision(metrics, max_width_pp, quality)
    audit_by_id = {row["run_id"]: row for row in evidence["accounting"]["capture_bound_audits"]}
    run_audits = []
    for row in groups["normal_work_leaf"]:
        analysis_path = row["analysis"]
        verify_source(analysis_path, evidence["source_sha256"][analysis_path])
        verify_source(row["source"])
        audit = audit_by_id[row["run_id"]]
        server = verify_source(audit["server_stream"], audit["server_stream_sha256"])
        verify_source(audit["client_stream"], audit["client_stream_sha256"])
        tails = audit["unresolved_response_tail_audit"]
        verify_source(tails["grace_stream"], tails["grace_stream_sha256"])
        if len(tails["details"]) != row["missing_responses"]:
            raise ValueError("saved missing-response count disagrees with tail inventory")
        stream = inspect_stream(read_records(server), tails["details"])
        run_audits.append({"run_id": row["run_id"], "server_stream": audit["server_stream"], **stream})

    status_counts = Counter()
    for run in run_audits:
        status_counts.update(run["gap_status_counts"])
    result.update({
        "source": str(source.relative_to(repo_root)),
        "helper_sha256": sha256(Path(__file__)),
        "sources": source_records,
        "inherited_response_cap": {
            "raw_tokens": evidence["accounting"]["maximum_raw_tokens_per_unresolved_response"],
            "source": bound_path,
            "independently_reestablished_by_this_replay": False,
        },
        "historical_response_audit": {
            "runs": run_audits,
            "unresolved_responses": sum(run["unresolved_responses"] for run in run_audits),
            "raw_response_completed_events": sum(run["raw_response_completed_events"] for run in run_audits),
            "raw_response_item_completed_events": sum(run["raw_response_item_completed_events"] for run in run_audits),
            "gap_status_counts": dict(sorted(status_counts.items())),
            "bounds_narrowed": False,
            "rule": "first later advancing same-thread total minus prior total minus later response last usage",
            "limitation": "a residual alone does not uniquely attribute an interrupted response; original bounds are retained",
        },
    })
    return result


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("mode", choices=("evaluate", "replay"))
    parser.add_argument("input", type=Path)
    parser.add_argument("--repo-root", type=Path, default=ROOT)
    parser.add_argument("--max-width-pp", type=float, default=5.0)
    parser.add_argument("--output", type=Path, help="new output path; existing files are not replaced")
    arguments = parser.parse_args()
    try:
        if arguments.mode == "replay":
            result = replay_historical(arguments.input, arguments.repo_root, arguments.max_width_pp)
        else:
            source = json.loads(arguments.input.read_text(encoding="utf-8"))
            result = evaluate_precision(source["metrics"], arguments.max_width_pp, source.get("quality"))
            result["source"] = str(arguments.input.resolve())
            result["source_sha256"] = sha256(arguments.input)
        encoded = json.dumps(result, indent=2, sort_keys=True, allow_nan=False) + "\n"
        if arguments.output:
            with arguments.output.open("x", encoding="utf-8") as handle:
                handle.write(encoded)
            print(json.dumps({"output": str(arguments.output), "precision_pass": result["precision_pass"]}))
        else:
            print(encoded, end="")
        return 0 if result["precision_pass"] else 1
    except (OSError, ValueError, KeyError, TypeError) as error:
        print(f"accounting precision gate: {error}", file=sys.stderr)
        return 2


if __name__ == "__main__":
    raise SystemExit(main())
