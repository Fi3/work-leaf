"""Append-only approved R13–R20 scope; the prior validator remains authoritative."""
import hashlib
import json

import progress_counter as core

SCOPE_SHA256 = "771e0dfbff126cfd8c60b1ce1db8daf666b0fb632374822b9de26ffc7ecf6e8b"


def load_contract():
    raw = (core.STUDY / "PROGRESS-TASK-EXTENSION-20260914.json").read_bytes()
    core.require(hashlib.sha256(raw).hexdigest() == SCOPE_SHA256,
                 "the approved eight-task extension is frozen")
    scope = json.loads(raw)
    records = {}
    for name in ("parent_ledger", "parent_publications", "proposal", "plan"):
        item = scope[name]
        data = (core.ROOT / item["path"]).read_bytes()
        core.require(hashlib.sha256(data).hexdigest() == item["sha256"],
                     "preserve the approved proposal and the original zero checkpoint")
        if name != "plan":
            records[name] = json.loads(data)
    parent = records["parent_ledger"]
    pubs = records["parent_publications"]
    core.require(core.validate_published_progress(parent, pubs) == (66, 0),
                 "the extension starts from the retained completed scope")
    core.require((core.ROOT / scope["authority"]).is_file(), "explicit authority is required")
    return scope, parent, pubs


def checkpoint(scope, parent):
    return {"event": "approved_scope_extension", "approval_id": scope["id"],
            "at": scope["approved_on"], "done": 66, "todo": 8, "total": 74,
            "completed": list(parent["completed"])}


def validate_ledger(ledger):
    scope, parent, _ = load_contract()
    core.keys(ledger, "schema scope_id fixed_total completed pending tasks history research scope_issues")
    core.require(type(ledger["schema"]) is int and ledger["schema"] == 4
                 and type(ledger["fixed_total"]) is int and ledger["fixed_total"] == 74
                 and ledger["scope_id"] == scope["id"], "only the approved fixed-74 scope is active")
    definitions = {item["id"]: item for item in scope["tasks"]}
    completed, pending = ledger["completed"], ledger["pending"]
    core.require(core.strings(completed) and core.strings(pending, allow_empty=True),
                 "stable task identities are required")
    core.require(completed[:66] == parent["completed"], "preserve all 66 completed records")
    core.require(len(completed) + len(pending) == 74
                 and len(set(completed + pending)) == 74
                 and set(completed + pending) == set(parent["completed"]) | set(definitions),
                 "no added, removed, duplicate or reopened task")
    core.require(pending == [item for item in definitions if item not in completed],
                 "pending is exactly the unfinished approved task set")
    core.require(isinstance(ledger["tasks"], dict)
                 and set(ledger["tasks"]) == set(parent["tasks"]) | set(definitions),
                 "all approved definitions must be retained")
    for identity, record in parent["tasks"].items():
        core.require(ledger["tasks"][identity] == record, "old completed records are immutable")
    for identity, definition in definitions.items():
        item = ledger["tasks"][identity]
        core.keys(item, "definition status started_at result")
        core.require(item["definition"] == definition, "the task scope and result path are frozen")
        if item["started_at"] is not None:
            core.timestamp(item["started_at"])
        if identity in pending:
            core.require(item["status"] in {"pending", "checking"} and item["result"] is None,
                         "pending state cannot hide a completion")
            if item["status"] == "checking":
                core.timestamp(item["started_at"])
            continue
        core.require(item["status"] == "done", "a completion requires a terminal result")
        result = item["result"]
        core.keys(result, "completed_at outcome checked evidence finding limits")
        core.require(result["outcome"] in core.OUTCOMES and core.strings(result["evidence"])
                     and all(core.nonempty(result[key]) for key in ("checked", "finding", "limits")),
                     "completion requires evidence, an actual check and honest limits")
        core.require(core.timestamp(result["completed_at"]) >= core.timestamp(item["started_at"]),
                     "completion must follow the recorded start")
    history = ledger["history"]
    boundary = len(parent["history"])
    core.require(isinstance(history, list) and len(history) >= boundary + 1
                 and history[:boundary] == parent["history"]
                 and history[boundary] == checkpoint(scope, parent),
                 "preserve every checkpoint and append the exact approved extension")
    previous = list(parent["completed"])
    for point in history[boundary + 1:]:
        core.keys(point, "event scope_id at done todo total completed completion")
        core.keys(point["completion"], "id record")
        identity = point["completion"]["id"]
        core.require(identity in definitions and identity not in previous,
                     "advance exactly one new approved task")
        if identity == "R20":
            core.require(set(definitions) - {"R20"} <= set(previous),
                         "the final review requires every input task first")
        expected = previous + [identity]
        core.require(point["event"] == "task_completed" and point["scope_id"] == scope["id"]
                     and all(type(point[key]) is int for key in ("done", "todo", "total"))
                     and point["total"] == 74 and point["done"] == len(expected)
                     and point["todo"] == 74 - len(expected) and point["completed"] == expected,
                     "each completion is exactly DONE +1 / TODO -1")
        item = ledger["tasks"][identity]
        core.require(item["status"] == "done" and point["completion"]["record"] == item
                     and point["at"] == item["result"]["completed_at"],
                     "published completed results cannot be rewritten")
        previous = expected
    core.require(previous == completed, "every completion requires its retained checkpoint")
    all_definitions = {key: item["definition"] for key, item in ledger["tasks"].items()}
    core.validate_zero_decision(ledger, all_definitions, final_id="R20")
    return len(completed), len(pending)


def validate_publications(ledger, publications):
    counts = validate_ledger(ledger)
    scope, parent, parent_pubs = load_contract()
    core.keys(publications, "schema scope_id fixed_total checkpoints")
    core.require(type(publications["schema"]) is int and publications["schema"] == 3
                 and publications["scope_id"] == scope["id"]
                 and type(publications["fixed_total"]) is int and publications["fixed_total"] == 74,
                 "the approved extension requires its separate publication history")
    expected = parent_pubs["checkpoints"] + ledger["history"][len(parent["history"]):]
    core.require(publications["checkpoints"] == expected,
                 "preserve the complete publication history, including the zero checkpoint")
    return counts
