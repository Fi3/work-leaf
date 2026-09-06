#!/usr/bin/env python3
"""Provider-free, raw-primary accounting and frozen quality scoring for one pair.

Missing turns are not response counts. A finite bound requires the saved normal
response-tail predicates; unfamiliar/ambiguous tails retain a null upper bound.
No interval-width threshold selects or drops either workflow's outcome.
"""

import argparse
from bisect import bisect_left
from collections import Counter, defaultdict
import hashlib
import importlib.util
import json
import math
import os
from pathlib import Path
import sys
import tempfile

ROOT = Path(__file__).resolve().parents[2]
GATE = Path(__file__).resolve().parent
METRICS = ("raw_input_plus_output", "uncached_input_plus_output")
FIELDS = ("input_tokens", "cached_input_tokens", "output_tokens", "reasoning_output_tokens")
CAMEL = ("inputTokens", "cachedInputTokens", "outputTokens", "reasoningOutputTokens")
EXACT = {"forwarded-after-exact-usage", "forwarded-after-resumed-output-usage"}
UNRESOLVED = {"forwarded-after-output-resumed", "forwarded-after-timeout"}
PRE_METHODS = {"turn/started", "item/started", "item/completed", "item/agentMessage/delta",
               "mcpServer/startupStatus/updated"}
POST_METHODS = {"item/started", "item/agentMessage/delta", "thread/tokenUsage/updated",
                "thread/status/changed", "turn/completed"}


def sha256(path):
    digest = hashlib.sha256()
    with Path(path).open("rb") as stream:
        for block in iter(lambda: stream.read(1024 * 1024), b""):
            digest.update(block)
    return digest.hexdigest()


def resolve(path):
    path = Path(path)
    return path if path.is_absolute() else ROOT / path


def read_json(path):
    return json.loads(Path(path).read_text(encoding="utf-8"))


def records(path):
    with Path(path).open(encoding="utf-8") as stream:
        for line in stream:
            if line.strip():
                value = json.loads(line)
                if not isinstance(value, dict):
                    raise ValueError("stream record must be an object")
                yield value


def integer(value):
    if type(value) is not int or value < 0:
        raise ValueError("token counters must be nonnegative integers")
    return value


def usage(value, *, camel=False):
    names = CAMEL if camel else FIELDS
    values = [integer(value.get(name)) for name in names]
    i, c, o, r = values
    if c > i or r > o:
        raise ValueError("cached/reasoning counters exceed their parent counters")
    result = dict(zip(FIELDS, values))
    result.update(uncached_input_tokens=i-c, raw_input_plus_output=i+o,
                  uncached_input_plus_output=i-c+o)
    if camel and integer(value.get("totalTokens")) != i+o:
        raise ValueError("provider total differs from input plus output")
    if not camel and any(name in value and value[name] != count for name, count in result.items()):
        raise ValueError("derived usage arithmetic is inconsistent")
    return result


def saving_interval(direct, work_leaf):
    def check(value, positive=False, allow_unknown=False):
        low, high = value["lower"], value["upper"]
        for count in (low,) if high is None and allow_unknown else (low, high):
            if type(count) not in (int, float) or not math.isfinite(count):
                raise ValueError("bounds must be finite numeric values")
        if low < 0 or (positive and low == 0) or (high is not None and high < low):
            raise ValueError("invalid bounds or nonpositive denominator")
        return low, high
    dl, du = check(direct, positive=True)
    wl, wu = check(work_leaf, allow_unknown=True)
    lower = None if wu is None else 100.0 * (dl-wu)/dl
    upper = 100.0 * (du-wl)/du
    return {"status": "unbounded" if lower is None else "bounded",
            "saving_percent": {"lower": lower, "upper": upper},
            "width_percentage_points": None if lower is None else upper-lower,
            "direct": direct, "work_leaf": work_leaf}


def compare_bounds(direct, work_leaf):
    return {"primary_metric": METRICS[0], "secondary_metric": METRICS[1],
            **{name: saving_interval(direct[name], work_leaf[name]) for name in METRICS},
            "sampling_confidence_interval": False,
            "interpretation": "One pair is descriptive; accounting bounds are not sampling confidence."}


def key(value):
    p = value.get("params") or {}
    thread = p.get("threadId")
    turn = p.get("turnId") or (p.get("turn") or {}).get("id")
    return (thread, turn) if isinstance(thread, str) and isinstance(turn, str) else None


def rpc_id(value):
    identity = value.get("id")
    if type(identity) not in (str, int):
        raise ValueError("RPC ID must be a string or integer")
    return type(identity).__name__, identity


def directive(value):
    item = (value.get("params") or {}).get("item") or {}
    return (value.get("method") == "item/completed" and item.get("type") == "agentMessage"
            and isinstance(item.get("text"), str)
            and any(line.startswith("@work-leaf ") for line in item["text"].splitlines()))


def tail_proof(turn_key, events, thread_events, usage_events, interrupt_count, grace):
    """Reuse the frozen study's isolated-tail proof, with indexed stream slices."""
    result = {"thread_id": turn_key[0], "turn_id": turn_key[1], "response_count_upper": None,
              "proof": None, "reason": "isolated response tail not established"}
    starts = [n for n, v in events if v.get("method") == "turn/started"]
    directives = [n for n, v in events if directive(v)]
    completions = [(n, v) for n, v in events if v.get("method") == "turn/completed"]
    if interrupt_count != 1 or len(starts) != 1 or len(directives) != 1 or len(completions) != 1:
        return result
    dseq = directives[0]; endseq, completed = completions[0]
    if not starts[0] <= dseq < endseq or completed["params"]["turn"].get("status") != "interrupted":
        return result
    seqs, totals = usage_events
    prior_index = bisect_left(seqs, dseq)-1
    prior_seq, prior = (seqs[prior_index], totals[prior_index]) if prior_index >= 0 else (-1, (0,)*4)
    startseq = max(starts[0], prior_seq+1)
    thread_seqs, values = thread_events
    begin = bisect_left(thread_seqs, startseq)
    stop = bisect_left(thread_seqs, endseq+1)
    window = ((thread_seqs[index], values[index]) for index in range(begin, stop))
    pre, post = [], []
    for seq, value in window:
        scoped = key(value)
        if scoped is not None and scoped != turn_key:
            return result
        (pre if seq <= dseq else post).append((seq, value))
    if any(v.get("method") not in PRE_METHODS for _, v in pre):
        return result
    if any(v.get("method") not in POST_METHODS for _, v in post):
        return result
    started, ended = Counter(), Counter(); agent_messages = 0
    for _, value in pre:
        if value.get("method") not in {"item/started", "item/completed"}:
            continue
        item = value["params"].get("item") or {}
        if item.get("type") not in {"userMessage", "reasoning", "agentMessage"} or not isinstance(item.get("id"), str):
            return result
        identity = item["type"], item["id"]
        (started if value["method"] == "item/started" else ended)[identity] += 1
        agent_messages += value["method"] == "item/completed" and item["type"] == "agentMessage"
    if started != ended or any(count != 1 for count in started.values()) or agent_messages != 1:
        return result
    post_started = 0
    for _, value in post:
        item = (value.get("params") or {}).get("item")
        if item is not None and (item.get("type") not in {"reasoning", "agentMessage"}
                                 or not isinstance(item.get("id"), str)):
            return result
        post_started += value.get("method") == "item/started"
        if value.get("method") == "thread/tokenUsage/updated":
            total = usage(value["params"]["tokenUsage"]["total"], camel=True)
            if tuple(total[name] for name in FIELDS) != prior:
                return result
    if post_started > 1 or (grace["outcome"] == "forwarded-after-output-resumed" and post_started != 1):
        return result
    result.update(response_count_upper=1, proof="isolated_normal_response_tail",
                  reason=None, previous_usage_sequence=prior_seq, response_start_sequence=startseq,
                  directive_sequence=dseq, completion_sequence=endseq,
                  paired_items=len(started), post_directive_unfinished_items=post_started,
                  no_tool_boundary=True)
    return result


def inventory(clients, servers, grace_records=None):
    errors = []; requests = {}; starts = set(); interrupts = Counter()
    for value in clients:
        method = value.get("method")
        if method == "turn/start":
            ident = rpc_id(value)
            if ident in requests: errors.append("duplicate client turn/start RPC ID")
            requests[ident] = (value.get("params") or {}).get("threadId")
        elif method == "turn/interrupt":
            if key(value) is None: errors.append("interrupt lacks thread/turn identity")
            else: interrupts[key(value)] += 1
    for value in servers:
        if "method" in value or "result" not in value or "id" not in value:
            continue
        thread = requests.get(rpc_id(value))
        turn = (value.get("result") or {}).get("turn") or {}
        if thread is not None and isinstance(turn.get("id"), str):
            k = thread, turn["id"]
            if k in starts: errors.append("duplicate captured turn/start reply")
            starts.add(k)
    by_turn = defaultdict(list); by_thread = defaultdict(lambda: ([], []))
    usage_by_thread = defaultdict(lambda: ([], [])); previous = {}; fresh = defaultdict(list)
    interrupted = set(); raw = {}; active = {}; method_counts = Counter(); item_counts = Counter()
    for sequence, value in enumerate(servers):
        method = value.get("method"); method_counts[method or "RPC_RESPONSE"] += 1
        p = value.get("params") or {}; thread = p.get("threadId"); k = key(value)
        if isinstance(thread, str):
            by_thread[thread][0].append(sequence); by_thread[thread][1].append(value)
        if k is not None: by_turn[k].append((sequence, value))
        if method == "turn/started" and k is not None:
            if k not in starts: errors.append("turn start lacks a captured client request/reply")
            if thread in active: errors.append("overlapping turns in one provider thread")
            active[thread] = k
        if method == "turn/completed" and k is not None:
            if active.get(thread) != k: errors.append("turn completion does not match the active same-thread turn")
            active.pop(thread, None)
            if (p.get("turn") or {}).get("status") in {"interrupted", "cancelled", "canceled"}:
                interrupted.add(k)
        if method in {"item/started", "item/completed"}:
            item_counts[str((p.get("item") or {}).get("type"))] += 1
        if method == "thread/tokenUsage/updated":
            total = usage(p["tokenUsage"]["total"], camel=True)
            last = usage(p["tokenUsage"]["last"], camel=True)
            current = tuple(total[name] for name in FIELDS); prior = previous.get(thread, (0,)*4)
            if any(a < b for a, b in zip(current, prior)):
                errors.append("cumulative provider usage regressed")
            advance = tuple(a-b for a, b in zip(current, prior))
            last_tuple = tuple(last[name] for name in FIELDS)
            if any(last_tuple) and all(a >= b for a, b in zip(advance, last_tuple)) and k is not None:
                fresh[k].append(sequence)
            usage_by_thread[thread][0].append(sequence); usage_by_thread[thread][1].append(current)
            previous[thread] = current
        if method == "rawResponse/completed":
            identity = p.get("responseId")
            if k not in starts or not isinstance(identity, str) or not identity:
                errors.append("raw response lacks matching captured turn or response identity")
                continue
            measured = None if p.get("usage") is None else usage(p["usage"], camel=True)
            record = {"thread_id": k[0], "turn_id": k[1], "usage": measured}
            if identity in raw and raw[identity] != record:
                errors.append("conflicting raw response identity or usage")
            raw[identity] = record
    for k in set(interrupts) | interrupted:
        if k not in starts: errors.append("interrupt does not belong to a captured local turn")
    candidates = set(interrupts) | interrupted
    grace_by_turn = {}
    if grace_records is not None:
        candidates = set()
        for record in grace_records:
            k = record.get("thread_id"), record.get("turn_id")
            if k in grace_by_turn: errors.append("duplicate grace decision for one turn")
            grace_by_turn[k] = record
            outcome = record.get("outcome")
            if k not in starts or interrupts[k] != 1:
                errors.append("grace decision lacks exactly one matching local interrupt")
            if outcome in EXACT:
                ds = [n for n, v in by_turn[k] if directive(v)]
                if len(ds) != 1 or not any(n > ds[0] for n in fresh[k]):
                    errors.append("exact grace decision lacks fresh same-turn post-directive usage")
            elif outcome in UNRESOLVED:
                candidates.add(k)
            else:
                errors.append("unrecognized nonexact grace outcome")
        if set(interrupts) != set(grace_by_turn):
            errors.append("grace decisions do not inventory every client interrupt")
    gaps = []
    # Each non-overlapping turn is inspected once. Thread windows use binary search,
    # not a full-stream scan per missing response; ambiguous overlap is fail-closed.
    for k in sorted(candidates):
        if grace_records is not None and not errors:
            gaps.append(tail_proof(k, by_turn[k], by_thread[k[0]], usage_by_thread[k[0]],
                                  interrupts[k], grace_by_turn[k]))
        else:
            gaps.append({"thread_id": k[0], "turn_id": k[1], "response_count_upper": None,
                         "proof": None, "reason": "no validated isolated-tail proof"})
    return {"gaps": gaps, "errors": errors, "raw_response_count": len(raw),
            "raw_responses": raw, "captured_turn_count": len(starts),
            "method_counts": dict(method_counts), "item_event_counts": dict(item_counts),
            "model_call_inventory_exhaustive": False}


def accounting(analysis, gap_inventory, policy):
    reasons = list(gap_inventory.get("errors", [])); measured = None; bounds = None
    try:
        measured = usage(analysis["usage_scopes"]["total_workflow"])
        missing = integer(analysis.get("interrupted_provider_turns"))
        expected = f"interrupted provider turn has no complete usage: count={missing}"
        error_list = analysis.get("errors")
        if not isinstance(error_list, list): raise ValueError("capture error inventory missing")
        reasons.extend(e for e in error_list if not (missing > 0 and e == expected))
        if len(gap_inventory["gaps"]) != missing: reasons.append("gap count disagrees with strict observer")
        if analysis.get("capture_complete") is not (missing == 0): reasons.append("capture status is inconsistent")
        if missing and error_list.count(expected) != 1: reasons.append("missing expected accounting error")
        if analysis.get("invocation_count") != analysis.get("complete_invocation_count"):
            reasons.append("unfinished invocation inventory")
        strata = analysis.get("model_strata") or []
        if not strata or any(s.get("model") != policy["model"] or s.get("effort") != policy["reasoning_effort"] for s in strata):
            reasons.append("provider model/effort differs from bound policy")
        cap = policy["per_response"]
        maximum = integer(cap["input_upper"])+integer(cap["output_upper"])
        if maximum == 0: raise ValueError("response ceiling must be positive")
        for row in gap_inventory.get("raw_responses", {}).values():
            u = row["usage"]
            if u is not None and (u["input_tokens"] > cap["input_upper"] or u["output_tokens"] > cap["output_upper"]):
                reasons.append("observed response exceeds the declared model bound")
        if not reasons:
            known = all(g.get("response_count_upper") == 1 and g.get("proof") == "isolated_normal_response_tail"
                        for g in gap_inventory["gaps"])
            bounds = {name: {"lower": measured[name],
                            "upper": measured[name]+missing*maximum if known else None} for name in METRICS}
            status = "exact" if missing == 0 else ("bounded" if known else "unbounded_accounting_gap")
        else: status = "ineligible"
    except (KeyError, TypeError, ValueError) as error:
        reasons.append(str(error)); status = "ineligible"
    return {"status": status, "reasons": reasons, "recorded_usage": measured, "bounds": bounds,
            "gap_inventory": gap_inventory, "bound_policy": policy,
            "bound_interpretation": "Conditional on documented model limits and complete visibility of normal response/tool boundaries; not an exhaustive hidden-provider-call guarantee.",
            "measurement_status_does_not_change_workflow_result": True}


def checked_module(entry, name):
    path = resolve(entry["path"])
    if sha256(path) != entry["sha256"]: raise ValueError(f"frozen source digest mismatch: {path}")
    spec = importlib.util.spec_from_file_location(name, path)
    if spec is None or spec.loader is None: raise ValueError("cannot load frozen module")
    module = importlib.util.module_from_spec(spec); spec.loader.exec_module(module)
    return module


def load_scorer(config, study_dir):
    module = checked_module(config["scorer"], "raw_pilot_frozen_scorer")
    expected = {str((module.SCRIPT_DIR / "fixtures" / f).resolve()) for f, _ in module.FIXTURES.values()}
    actual = {str(resolve(entry["path"]).resolve()) for entry in config["fixtures"]}
    if actual != expected: raise ValueError("frozen fixture inventory differs from canonical scorer")
    for entry in config["fixtures"]:
        if sha256(resolve(entry["path"])) != entry["sha256"]: raise ValueError("frozen fixture digest mismatch")
    module.STUDY_DIR = Path(study_dir)
    return module


def score_all(manifest, config, study_dir, work_root, timeout):
    scorer = load_scorer(config, study_dir); rows = []
    for entry in manifest["runs"]:
        try:
            rows.append(scorer.score_entry(entry, manifest, work_root, timeout))
        except (OSError, ValueError, KeyError, RuntimeError) as error:
            rows.append({"id": entry["id"], "condition": entry.get("condition"),
                         "checks": {name: "not-run" for name in scorer.FIXTURES},
                         "completed_features": 0, "scoring_error": str(error)})
    return rows


def launched_counts(rows):
    counts = {"direct": 0, "work-leaf": 0}
    for row in rows:
        if row.get("started_at") and row.get("launch_status") in {"running", "completed", "supervisor_stopped"}:
            counts[row["condition"]] += 1
    return counts


def stage_activity(helper, analysis_path, analysis, condition):
    try:
        activity = helper.rollout_activity(analysis_path, analysis,
                                         "compact-direct" if condition == "direct" else "normal-work-leaf")
        return {"status": "available", "reported_usage_advances": activity["usage_changes"],
                "reported_usage_advances_by_stage": activity["usage_changes_by_stage"],
                "exact_model_call_count": False,
                "interpretation": "Cumulative usage changes are a reported-generation proxy, not an exhaustive count of model calls. Tool actions and stage mapping reuse the frozen historical helper.",
                "evidence": activity}
    except (OSError, ValueError, KeyError) as error:
        return {"status": "unavailable", "reason": str(error), "reported_usage_advances": None,
                "exact_model_call_count": False}


def analyze_manifest(manifest, policy, config, quality):
    stage_helper = checked_module(config["stage_helper"], "raw_pilot_frozen_stage_helper")
    stage_helper.SESSIONS = Path(os.environ.get("CODEX_HOME", str(Path.home()/".codex"))) / "sessions"
    rows = []; sources = {}
    def remember(path):
        sources[str(path)] = sha256(path)
        return read_json(path)
    for entry in manifest["runs"]:
        row = {"id": entry["id"], "condition": entry["condition"],
               "launcher_exit_code": entry.get("launcher_exit_code"), "workflow_result": "unknown",
               "launch_status": entry.get("launch_status"), "started_at": entry.get("started_at")}
        try:
            if not launched_counts([entry])[entry["condition"]]:
                raise ValueError("workflow was not launched; planned row is not an observation")
            artifact = resolve(entry["artifact"]); report = remember(resolve(entry["report"]))
            analysis = remember(artifact / "observation/analysis.json")
            row["workflow_result"] = report.get("workflow_result", report.get("result", "unknown"))
            combined = {"gaps": [], "errors": [], "raw_responses": {}, "captures": []}
            for app in sorted((artifact / "observation/app-server").glob("*")):
                paths = [app / "client-to-server.raw", app / "server-to-client.raw"]
                values = [list(records(p)) for p in paths]
                grace_path = app / "provider-usage-grace.jsonl"
                grace = list(records(grace_path)) if grace_path.is_file() else None
                inv = inventory(*values, grace)
                for p in paths + ([grace_path] if grace is not None else []): sources[str(p)] = sha256(p)
                combined["gaps"].extend(inv["gaps"]); combined["errors"].extend(inv["errors"])
                for identity, response in inv["raw_responses"].items():
                    if identity in combined["raw_responses"] and combined["raw_responses"][identity] != response:
                        combined["errors"].append("conflicting response across captures")
                    combined["raw_responses"][identity] = response
                combined["captures"].append({"path": str(app), **{k:v for k,v in inv.items() if k not in {"gaps", "raw_responses"}}})
            if report.get("agent_model") != policy["model"] or report.get("agent_reasoning_effort") != policy["reasoning_effort"]:
                combined["errors"].append("report profile differs from frozen policy")
            if usage(report.get("total_workflow_usage", {})) != usage(analysis["usage_scopes"]["total_workflow"]):
                combined["errors"].append("report and observer usage differ")
            row["measurement"] = accounting(analysis, combined, policy)
            row["stage_recorded_usage"] = stage_helper.stage_usage(analysis, "compact-direct" if entry["condition"] == "direct" else "normal-work-leaf")
            row["stage_usage_scope"] = "recorded lower bounds; missing tails not allocated"
            row["generation_activity"] = stage_activity(stage_helper, artifact / "observation/analysis.json",
                                                       analysis, entry["condition"])
            row["mechanism_observations"] = analysis.get("mechanisms", {})
        except (OSError, ValueError, TypeError, KeyError) as error:
            row["measurement"] = {"status": "ineligible", "reasons": [str(error)], "bounds": None}
        rows.append(row)
    by_condition = {row["condition"]: row for row in rows}; comparison = None
    if len(rows) != 2 or set(by_condition) != {"direct", "work-leaf"}:
        raise ValueError("the pilot requires exactly one direct and one Work Leaf observation")
    d, w = (by_condition[name]["measurement"] for name in ("direct", "work-leaf"))
    if d.get("bounds") and w.get("bounds") and d["bounds"][METRICS[0]]["upper"] is not None:
        comparison = compare_bounds(d["bounds"], w["bounds"])
    return {"schema_version": 1, "study": manifest["study"], "observations": rows,
            "comparison": comparison, "primary_metric": METRICS[0], "secondary_metric": METRICS[1],
            "quality": quality, "quality_equivalence": "not established by a single pair",
            "source_sha256": sources, "benchmark_observations": launched_counts(manifest["runs"]),
            "planned_workflows": {"direct": 1, "work-leaf": 1},
            "sampling_confidence_interval": False, "next_action": "pause"}


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("mode", choices=("analyze", "score"))
    parser.add_argument("--manifest", type=Path, required=True)
    parser.add_argument("--scorer-config", type=Path, default=GATE / "BATCH-SCORER.json")
    parser.add_argument("--bound-policy", type=Path, default=GATE / "BATCH-BOUND-POLICY.json")
    parser.add_argument("--quality", type=Path)
    parser.add_argument("--output", type=Path, required=True)
    parser.add_argument("--timeout-seconds", type=int, default=900)
    args = parser.parse_args()
    if args.output.exists(): raise ValueError("output exists; refusing to overwrite prior evidence")
    manifest = read_json(args.manifest); config = read_json(args.scorer_config)
    if args.mode == "score":
        with tempfile.TemporaryDirectory(prefix="work-leaf-raw-pilot-score-") as directory:
            result = {"runs": score_all(manifest, config, args.output.parent, Path(directory), args.timeout_seconds),
                      "frozen_scorer": config, "scope": "feature quality; separate from token measurement"}
    else:
        result = analyze_manifest(manifest, read_json(args.bound_policy), config,
                                  read_json(args.quality) if args.quality else None)
    with args.output.open("x", encoding="utf-8") as stream:
        json.dump(result, stream, indent=2, sort_keys=True); stream.write("\n")
    print(args.output)
    return 0


if __name__ == "__main__":
    try:
        sys.exit(main())
    except (OSError, ValueError, KeyError) as error:
        print(f"analysis error: {error}", file=sys.stderr)
        sys.exit(2)
