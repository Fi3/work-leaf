#!/usr/bin/env python3
"""Offline assertion replay of admitted, completed read-inline smoke captures only."""
import argparse
import hashlib
from pathlib import Path
import sys
import types

sys.path.insert(0, str(Path(__file__).resolve().parent.parent))

def compile_source(path):
    data = path.read_bytes()
    module = types.ModuleType("diagnostic_census"); module.__file__ = str(path)
    exec(compile(data, str(path), "exec"), module.__dict__)
    return module, hashlib.sha256(data).hexdigest()


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--admission", type=Path, required=True)
    parser.add_argument("--review", type=Path, required=True)
    parser.add_argument("--sessions-root", type=Path, required=True)
    parser.add_argument("--output", type=Path, required=True)
    args = parser.parse_args()
    source_path = Path(__file__).resolve().parent.parent/"audit_read_mechanism.py"
    census, executed_sha256 = compile_source(source_path)
    index = census.SourceIndex()
    index.read(Path(__file__).resolve()); index.read(source_path, executed_sha256)
    review = census.decode(index.read(args.review))
    assert review["diagnostic_only"] is True and review["study_observations"] == 0
    assert review["phase_primary_invoked"] is False
    recorded = review["source_sha256"]
    for path, expected in recorded.items(): index.read(path, expected)
    _, _, read_json, _ = census.recorded_access(index, recorded)
    admission = read_json(args.admission)
    assert admission["excluded_from_benchmark_observations"] is True
    assert admission["case_count"] == len(admission["cases"]) == len(review["runs"]) == 2
    expected_runs = {row["run_id"]: row for row in review["runs"]}
    assert len(expected_runs) == 2
    accounting_path = census.HERE/"accounting_untracked_reads.py"
    # A diagnostic-only interface projection, never an admitted phase manifest.
    frozen = {"model": admission["model"], "reasoning_effort": admission["reasoning_effort"],
              "files": [{"path": str(accounting_path), "sha256": recorded[str(accounting_path)], "role": "frozen-evidence"}]}
    output = {"schema": "work-leaf-read-mechanism-diagnostic-replay-v1", "diagnostic_only": True,
              "study_observations": 0, "provider_calls": 0, "phase_primary_invoked": False,
              "phase_wrapper_exercised": False, "shared_launched_source_branch_exercised": True,
              "closure": "Retained diagnostic admission, successful bounded test log, and actual invocation start/end receipts; not a twelve-workflow phase.",
              "runs": []}
    for case in admission["cases"]:
        rid = case["run_id"]; expected = expected_runs.pop(rid)
        artifact = Path(case["evidence_dir"])
        assert expected["condition"] == case["condition"]
        log = index.read(artifact/"smoke-output.log", recorded[str(artifact/"smoke-output.log")]).decode()
        assert f"WORK_LEAF_READ_INLINE_SMOKE_OK condition={case['condition']} turns=3 sessions=1 first_read=verified repeated_read=unchanged" in log
        assert "test result: ok. 1 passed; 0 failed; 0 ignored" in log
        experiment = read_json(case["experiment_manifest"])
        assert experiment["run_id"] == rid and experiment["condition"] == case["condition"]
        invocations = list((artifact/"observation/invocations").iterdir())
        assert len(invocations) == 1
        start = read_json(invocations[0]/"start.json"); end = read_json(invocations[0]/"end.json")
        assert start["invocation_id"] == end["invocation_id"] == invocations[0].name
        assert type(end["exit_code"]) is int and end["exit_code"] == 0
        assert type(start["start_unix_ns"]) is int and type(end["end_unix_ns"]) is int
        assert start["start_unix_ns"] < end["end_unix_ns"]
        entry = {"run_id": rid, "condition": case["condition"], "artifact": str(artifact),
                 "prompt_trace": str(artifact/"prompt-events.jsonl"), "report": str(artifact/"DIAGNOSTIC-REPORT-PROJECTION.json"),
                 "started_at": str(start["start_unix_ns"])}
        derived, accounting = census.replay_launched_sources(entry, frozen, args.sessions_root, index, recorded)
        assert accounting["status"] == "validated" and accounting["errors"] == []
        assert accounting["exact_response_evidence"] == expected["accounting"]["exact_response_evidence"]
        assert derived["errors"] == [] and derived["delivery_status"] == "available"
        assert len(derived["reads"]) == 2 and len(derived["responses"]) == 3
        assert not derived["tool_calls"] and not derived["tool_outputs"]
        for row, prior in zip(derived["reads"], expected["selected_native_inputs"]):
            user = row["native_user"]
            assert user["status"] == "linked" and user["join_method"] == "explicit_thread_turn_text"
            assert user["item_id"] == prior["item_id"] and user["line"] == prior["native_line"]
            assert row["trace_line"] == prior["trace_line"] and row["turn_id"] == prior["accepted_turn_id"]
            names = {"input_tokens": "input_tokens", "cached_tokens": "cached_input_tokens",
                     "output_tokens": "output_tokens", "cache_write_tokens": "cache_write_input_tokens"}
            charges = [{"response_id": derived["responses"][ref["response_index"]]["response_id"],
                        "counts": {raw: derived["responses"][ref["response_index"]]["input_items"][ref["input_item_index"]][normalized]
                                   for raw, normalized in names.items()}}
                       for ref in row["charge_refs"]]
            assert charges == [{"response_id": r["response_id"], "counts": r["counts"]} for r in prior["charges"]]
        assert all(all(value == 0 for value in response["residual"].values()) for response in derived["responses"])
        output["runs"].append({**derived, "terminal_sources": [str(invocations[0]/"start.json"), str(invocations[0]/"end.json"), str(artifact/"smoke-output.log")]})
    assert not expected_runs
    assert args.output.is_absolute() and args.output.parent == Path(__file__).resolve().parent
    census.publish(args.output, output, index)
    print(f"PASS: 2 diagnostic source replays, 4 exact input identities, 6 reconciled response records; {args.output}")


if __name__ == "__main__":
    main()
