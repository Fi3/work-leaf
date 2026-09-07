#!/usr/bin/env python3
"""Secondary descriptive token envelopes using the unchanged frozen accounting.

No v1 prompt conversion, token hypothesis test, native-compaction retotal, or
captured-response sum replaces the observer's recorded whole-workflow total.
"""
import argparse
import hashlib
import json
from pathlib import Path
import sys
import types

HERE = Path(__file__).resolve().parent
ROOT = HERE.parents[1]
VARIANT = "buildable-work-unit-incremental"
PINS = {"analyze.py": "2bb28891a2e158d51cf577bcf7c1fc2781065e38dc5ec4c6c30f7d4c146f2f78",
        "analyze_work_units.py": "e59e6a22bf1ab09e0aabec86791cefb54cd3f3d38167a4c9822cdce1d09b50d3"}


def load(name):
    path = HERE/name; data = path.read_bytes()
    if hashlib.sha256(data).hexdigest() != PINS[name]:
        raise ValueError("frozen source differs: "+name)
    module = types.ModuleType("work_unit_secondary_"+path.stem); module.__file__ = str(path)
    exec(compile(data, str(path), "exec"), module.__dict__)
    return module


def server_records(path):
    values = []; refs = {}
    with Path(path).open(encoding="utf-8") as stream:
        for line, text in enumerate(stream, 1):
            if not text.strip(): continue
            value = json.loads(text)
            if not isinstance(value, dict): raise ValueError("stream record must be an object")
            values.append(value)
            if value.get("method") == "rawResponse/completed":
                response = value.get("params", {}).get("responseId")
                if isinstance(response, str): refs.setdefault(response, []).append(line)
    return values, refs


def measure(observed, report, captures, base):
    """Merge exact capture identities, then call the unchanged accounting rule."""
    combined = {"gaps": [], "errors": [], "raw_responses": {}, "captures": []}; evidence = {}
    for capture in captures:
        inv = capture["inventory"]; location = capture["path"]
        combined["gaps"].extend(inv["gaps"]); combined["errors"].extend(inv["errors"])
        combined["errors"].extend(capture["provenance"]["errors"])
        combined["captures"].append({"path": location, "provenance": capture["provenance"],
                                     **{k: v for k, v in inv.items() if k not in {"gaps", "raw_responses"}}})
        for rid, value in inv["raw_responses"].items():
            if not all(isinstance(v, str) and v for v in (rid, value.get("thread_id"), value.get("turn_id"))):
                combined["errors"].append("invalid exact response/thread/turn identity"); continue
            if rid in combined["raw_responses"] and combined["raw_responses"][rid] != value:
                combined["errors"].append("conflicting response identity across captures")
                continue
            combined["raw_responses"][rid] = value
            record = evidence.setdefault(rid, {**value, "sources": []})
            record["sources"].extend({"capture": location, "server_line": line}
                                     for line in capture["response_sources"].get(rid, []))
    if not captures: combined["errors"].append("no primary app-server capture")
    if report.get("agent_model") != base.POLICY["model"] or report.get("agent_reasoning_effort") != base.POLICY["reasoning_effort"]:
        combined["errors"].append("report model or effort differs from frozen ceiling policy")
    if base.usage(report.get("total_workflow_usage", {})) != base.usage(observed["usage_scopes"]["total_workflow"]):
        combined["errors"].append("report and strict observer usage disagree")
    return {"measurement": base.STRICT.accounting(observed, combined, base.POLICY),
            "exact_completed_response_id_count": len(evidence), "exact_response_evidence": evidence,
            "exhaustive_model_call_count": False}


def summarize(rows, gate, base):
    errors = list(gate["integrity_errors"]); ids = set(); owners = {}
    if gate.get("primary", {}).get("status") != "available" or any(r.get("errors") for r in gate["observations"]):
        errors.append("independent v2 source/configuration/delivery gate unavailable")
    for row in rows:
        rid = row.get("id")
        if not isinstance(rid, str) or not rid or rid in ids:
            errors.append("invalid or duplicate workflow identity")
        ids.add(rid)
        if row.get("errors"): errors.append("workflow evidence has errors: "+str(rid))
        for response in row.get("exact_response_evidence", {}):
            if response in owners and owners[response] != rid:
                errors.append("response identity belongs to multiple workflows: "+response)
            owners[response] = rid
    if errors:
        return {"status": "unavailable", "errors": errors, "row_ids": [r.get("id") for r in rows]}
    return base.contrast(rows, VARIANT)


def analyze_manifest(path):
    base = load("analyze.py"); mediator = load("analyze_work_units.py")
    gate = mediator.analyze_manifest(path)
    sources = dict(gate["source_sha256"]); errors = []; rows = []

    def remember(value):
        p = Path(value); p = p if p.is_absolute() else ROOT/p
        actual = base.sha256(p)
        if str(p) in sources and sources[str(p)] != actual:
            raise ValueError("source changed since independent v2 replay: "+str(p))
        sources[str(p)] = actual
        return p

    def read(value):
        return json.loads(remember(value).read_text())

    manifest = read(path); frozen = read(manifest["phase_manifest"])
    mediator.require_frozen_helper(frozen["files"], __file__); remember(__file__)
    # Compile pinned direct sources, and retain the legacy gate's existing hash.
    for name in PINS: remember(HERE/name)
    remember(base.GATE/"batch_analysis.py")
    entries = {}
    for entry in manifest["runs"]:
        rid = entry.get("run_id", entry.get("id"))
        entries.setdefault(rid, []).append(entry)
    for checked in gate["observations"]:
        rid = checked["id"]
        row = {k: checked.get(k) for k in ("id", "condition", "block_id", "wave", "launch_status", "launcher_exit_code", "workflow_result")}
        row.update(errors=list(checked.get("errors", [])), measurement={"status": "ineligible", "bounds": None})
        rows.append(row)
        try:
            candidates = entries.get(rid, [])
            if len(candidates) != 1: raise ValueError("missing or duplicate result workflow retained")
            entry = candidates[0]
            row.update({k: entry.get(k) for k in ("started_at", "finished_at")})
            if not base.launched(entry): raise ValueError("planned workflow was not launched; retained")
            artifact = Path(entry["artifact"]); artifact = artifact if artifact.is_absolute() else ROOT/artifact
            report = read(entry["report"]); observed = read(artifact/"observation/analysis.json")
            row["workflow_result"] = report.get("workflow_result", report.get("result", "unknown"))
            captures = []
            for app in sorted((artifact/"observation/app-server").iterdir()):
                if not app.is_dir(): continue
                provenance = base.capture_provenance(app)
                for source, expected in provenance["source_sha256"].items():
                    if sources[str(remember(source))] != expected:
                        raise ValueError("capture source changed during provenance replay")
                clients = list(base.STRICT.records(remember(app/"client-to-server.raw")))
                servers, refs = server_records(remember(app/"server-to-client.raw"))
                grace = app/"provider-usage-grace.jsonl"
                inv = base.inventory(clients, servers, list(base.STRICT.records(remember(grace))) if grace.is_file() else None)
                captures.append({"path": str(app), "inventory": inv, "provenance": provenance, "response_sources": refs})
            row.update(measure(observed, report, captures, base))
            row["observer_errors"] = observed.get("errors")
        except (ValueError, TypeError, KeyError, OSError) as error:
            row["errors"].append(str(error))
    for source, expected in sources.items():
        try:
            if base.sha256(source) != expected: errors.append("source changed during analysis: "+source)
        except OSError as error: errors.append(str(error))
    gate["integrity_errors"].extend(errors)
    return {"schema": "work-leaf-work-unit-secondary-tokens-v1", "phase": manifest.get("phase"),
            "observations": rows, "comparison": summarize(rows, gate, base), "source_sha256": sources,
            "independent_v2_gate": {k: gate.get(k) for k in ("integrity_errors", "trust_replay",
                "legacy_behavioral_config_drift_detected", "unexplained_config_drift_detected", "pending_config_attestation_at_finish")},
            "policy": base.POLICY, "metrics": list(base.METRICS), "sampling_confidence_interval": False,
            "token_hypothesis_test": None, "native_compaction_retotal": None, "causal_share": None,
            "scope": "Secondary accounting envelopes over every frozen workflow. Recorded observer totals are not replaced by sums of raw response records. Native-compaction and exact-input audits remain mandatory separate scope checks; exhaustive hidden-provider usage is not proven."}


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--manifest", type=Path, required=True)
    parser.add_argument("--output", type=Path, required=True)
    args = parser.parse_args()
    if args.output.exists(): raise ValueError("output exists; reports are create-new")
    result = analyze_manifest(args.manifest)
    with args.output.open("x", encoding="utf-8") as stream:
        json.dump(result, stream, indent=2, sort_keys=True); stream.write("\n")
    print(args.output)


if __name__ == "__main__":
    try: main()
    except (ValueError, KeyError, TypeError, OSError) as error:
        print(f"secondary token report error: {error}", file=sys.stderr); sys.exit(2)
