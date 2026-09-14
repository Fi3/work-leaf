"""Frozen research-task ledger validation, independent of WL and provider runtimes.

This is an accidental-drift guard, not protection against an operator deliberately
rewriting the contract, its pinned digest, both histories and this validator.
"""

import hashlib
import json
import pathlib
import re
from datetime import datetime


ROOT = pathlib.Path(__file__).resolve().parents[2]
STUDY = pathlib.Path(__file__).resolve().parent
SCOPE_SHA256 = "3c6815915dc69a67a8fea8d8346adb3eaaccea4a2181e9e6d95ed4a6358ec944"
FIXED_TOTAL = 66
OUTCOMES = {"supported", "unsupported", "inconclusive", "failed_setup",
            "justified_not_needed"}


def require(condition, message):
    if not condition:
        raise ValueError(message)


def keys(value, names):
    require(isinstance(value, dict) and set(value) == set(names.split()),
            "unexpected or missing fields: no hidden work or omitted requirements")


def nonempty(value):
    return isinstance(value, str) and bool(value.strip())


def strings(value, allow_empty=False):
    return (isinstance(value, list) and (allow_empty or bool(value))
            and all(nonempty(item) for item in value))


def timestamp(value):
    require(nonempty(value), "a dated result/start is required")
    try:
        parsed = datetime.fromisoformat(value.replace("Z", "+00:00"))
    except (TypeError, ValueError) as error:
        raise ValueError("a valid ISO timestamp is required") from error
    require(parsed.tzinfo is not None, "timestamps require a timezone")
    return parsed


def load_contract():
    raw = (STUDY / "PROGRESS-TASK-SCOPE-20260914.json").read_bytes()
    require(hashlib.sha256(raw).hexdigest() == SCOPE_SHA256,
            "the user-approved questions, limits and scope cannot change")
    scope = json.loads(raw)
    archives = []
    for name in ("legacy_ledger", "legacy_publications"):
        item = scope[name]
        data = (ROOT / item["path"]).read_bytes()
        require(hashlib.sha256(data).hexdigest() == item["sha256"],
                "legacy records/publications must remain byte-for-byte")
        archives.append(json.loads(data))
    if "plan_sha256" in scope:
        require(hashlib.sha256((ROOT / scope["plan"]).read_bytes()).hexdigest()
                == scope["plan_sha256"], "the approved plan and its budgets are frozen")
    return scope, *archives


def migration_checkpoint(scope, legacy):
    return {
        "event": "approved_scope_correction", "approval_id": scope["id"],
        "at": scope["approved_on"], "done": 54, "todo": 12, "total": 66,
        "completed": list(legacy["completed"]),
    }


def validate_proposals(proposals):
    require(isinstance(proposals, list) and bool(proposals),
            "identify the exact additional tasks before asking for an increase")
    identities = []
    for proposal in proposals:
        keys(proposal, "id question scope")
        require(all(nonempty(value) for value in proposal.values()),
                "proposed tasks require concrete identities, questions and scope")
        identities.append(proposal["id"])
    require(len(set(identities)) == len(identities), "duplicate proposed task")


def validate_zero_decision(ledger, definitions):
    research = ledger["research"]
    keys(research, "status zero_decision")
    issues = ledger["scope_issues"]
    require(isinstance(issues, list), "scope issues must be an explicit list")
    for issue in issues:
        keys(issue, "case status reason proposed_tasks")
        require(type(issue["case"]) is int and issue["case"] in {1, 2}
                and issue["status"] == "unapproved" and nonempty(issue["reason"]),
                "new work is an unapproved scope issue, never uncounted execution")
        validate_proposals(issue["proposed_tasks"])
    decision = research["zero_decision"]
    if ledger["pending"]:
        require(research["status"] in {"paused", "active"} and decision is None,
                "pending tasks cannot be reported as a completed analysis")
        return
    require(isinstance(decision, dict), "zero TODO requires an explicit stop decision")
    require(strings(decision.get("evidence"))
            and definitions["R12"]["result"] in decision["evidence"],
            "the zero decision requires the final coverage/result evidence")
    if decision.get("kind") == "supported":
        keys(decision, "kind evidence conclusion")
        require(research["status"] == "complete" and not issues
                and nonempty(decision["conclusion"]),
                "a supported answer requires evidence and no unapproved necessary work")
        return
    keys(decision, "kind cases reason evidence additional_tasks requested_todo_increase permission_question")
    require(decision["kind"] == "permission_required"
            and research["status"] == "awaiting_permission",
            "zero with missing work must stop for user permission")
    cases = decision["cases"]
    require(isinstance(cases, list) and bool(cases)
            and all(type(case) is int and case in {1, 2} for case in cases)
            and len(set(cases)) == len(cases)
            and {issue["case"] for issue in issues} <= set(cases),
            "report case (1) additional work and/or case (2) incorrect initial checklist")
    require(nonempty(decision["reason"]) and nonempty(decision["permission_question"]),
            "state the concrete reason and ask permission")
    validate_proposals(decision["additional_tasks"])
    require(type(decision["requested_todo_increase"]) is int
            and decision["requested_todo_increase"] == len(decision["additional_tasks"]),
            "the requested increase must match the exact additional tasks")
    require(not set(definitions) & {task["id"] for task in decision["additional_tasks"]},
            "completed tasks cannot be reopened as an extension")


def validate_ledger(ledger):
    """Validate the declared finite work and retained evidence; not causal truth."""
    scope, legacy, _ = load_contract()
    keys(ledger, "schema scope_id fixed_total completed pending tasks history research scope_issues")
    require(type(ledger["schema"]) is int and ledger["schema"] == 3
            and type(ledger["fixed_total"]) is int and ledger["fixed_total"] == FIXED_TOTAL
            and ledger["scope_id"] == scope["id"],
            "only the explicitly approved fixed-66 scope is active")
    definitions = {task["id"]: task for task in scope["tasks"]}
    completed, pending = ledger["completed"], ledger["pending"]
    require(strings(completed) and strings(pending, allow_empty=True),
            "completed and pending must contain stable record IDs")
    require(completed[:54] == legacy["completed"],
            "all 54 completed historical records must be preserved")
    require(len(completed) + len(pending) == FIXED_TOTAL
            and len(set(completed + pending)) == FIXED_TOTAL
            and set(completed + pending) == set(legacy["completed"]) | set(definitions),
            "no added, removed, duplicate, swapped or reopened task")
    require(pending == [identity for identity in definitions if identity not in completed],
            "pending is exactly the unfinished approved task set")
    require(isinstance(ledger["tasks"], dict) and set(ledger["tasks"]) == set(definitions),
            "every remaining task has an explicit frozen definition")
    for identity, definition in definitions.items():
        item = ledger["tasks"][identity]
        keys(item, "definition status started_at result")
        require(item["definition"] == definition, "task question, scope and result path are frozen")
        if item["started_at"] is not None:
            timestamp(item["started_at"])
        if identity in pending:
            require(item["status"] in {"pending", "checking"} and item["result"] is None,
                    "pending state cannot hide a completion")
            if item["status"] == "checking":
                timestamp(item["started_at"])
            continue
        require(item["status"] == "done", "completed IDs need a terminal result")
        result = item["result"]
        keys(result, "completed_at outcome checked evidence finding limits")
        require(result["outcome"] in OUTCOMES and strings(result["evidence"])
                and all(nonempty(result[field]) for field in ("checked", "finding", "limits")),
                "completion needs the actual bounded check, evidence, outcome and limits")
        finished = timestamp(result["completed_at"])
        require(item["started_at"] is not None and finished >= timestamp(item["started_at"]),
                "completion must follow the recorded start")

    history = ledger["history"]
    boundary = len(legacy["history"])
    require(isinstance(history, list) and len(history) >= boundary + 1
            and history[:boundary] == legacy["history"]
            and history[boundary] == migration_checkpoint(scope, legacy),
            "retain legacy checkpoints and the exact once-approved correction")
    previous = list(legacy["completed"])
    for checkpoint in history[boundary + 1:]:
        keys(checkpoint, "event scope_id at done todo total completed completion")
        keys(checkpoint["completion"], "id record")
        identity = checkpoint["completion"]["id"]
        require(identity in definitions and identity not in previous,
                "a completion checkpoint advances one new approved task")
        if identity == "R12":
            require(set(definitions) - {"R12"} <= set(previous),
                    "final coverage needs all eleven input-task results first")
        expected = previous + [identity]
        require(checkpoint["event"] == "task_completed" and checkpoint["scope_id"] == scope["id"]
                and all(type(checkpoint[field]) is int for field in ("done", "todo", "total"))
                and checkpoint["total"] == FIXED_TOTAL
                and checkpoint["done"] == len(expected)
                and checkpoint["todo"] == FIXED_TOTAL - len(expected)
                and checkpoint["completed"] == expected,
                "each advance is exactly DONE +1 / TODO -1; no rollback or replenishment")
        item = ledger["tasks"][identity]
        require(item["status"] == "done" and checkpoint["completion"]["record"] == item
                and checkpoint["at"] == item["result"]["completed_at"],
                "published completed records cannot be rewritten or reopened")
        previous = expected
    require(previous == completed, "every completion requires its retained checkpoint")
    validate_zero_decision(ledger, definitions)
    return len(completed), len(pending)


def read_counts(first_line):
    match = re.fullmatch(r"# DONE: (\d+) \| TODO: (\d+) \| TOTAL: (\d+)", first_line)
    require(match is not None, "DONE / TODO / TOTAL must occupy the first line")
    done, todo, total = map(int, match.groups())
    require(total == FIXED_TOTAL and done + todo == FIXED_TOTAL and done >= 54,
            "DONE + TODO = TOTAL = 66; all 54 historical completions remain")
    return done, todo


def validate_published_progress(ledger, publications):
    counts = validate_ledger(ledger)
    scope, legacy, old_publications = load_contract()
    keys(publications, "schema scope_id fixed_total checkpoints")
    require(type(publications["schema"]) is int and publications["schema"] == 2
            and publications["scope_id"] == scope["id"]
            and type(publications["fixed_total"]) is int
            and publications["fixed_total"] == FIXED_TOTAL,
            "a separate fixed-66 publication archive is required")
    expected = (old_publications["checkpoints"]
                + ledger["history"][len(legacy["history"]):])
    require(publications["checkpoints"] == expected,
            "publication checkpoints differ: preserve history and append before reporting")
    return counts


def validate_visible_progress(ledger, hypothesis, note):
    counts = validate_ledger(ledger)
    lines = hypothesis.splitlines()
    require(bool(lines) and read_counts(lines[0]) == counts, "header and ledger disagree")
    note_lines = note.splitlines()
    require(len(note_lines) > 2 and lines[0].removeprefix("# ") in note_lines[2],
            "the live note and hypothesis header must agree")
    rows = re.findall(r"^\| (R\d\d) \| (TODO|CHECKING|DONE) \| ([^|]+) \|$",
                      "\n".join(lines[:35]), re.MULTILINE)
    expected = [(identity, {"pending": "TODO", "checking": "CHECKING", "done": "DONE"}[
        item["status"]], item["definition"]["question"]) for identity, item in ledger["tasks"].items()]
    require(rows == expected, "all twelve visible task IDs, statuses and questions must match")


def validate_evidence_files(ledger):
    """Check retained source existence and dated result records before publication."""
    validate_ledger(ledger)
    _, legacy, _ = load_contract()
    sources = [group["source"] for group in legacy["completed_record_sources"].values()]
    sources += [source for item in legacy["deliverables"].values() for source in item["evidence"]]
    sources += legacy["analysis"]["conclusion_evidence"]
    if legacy["analysis"].get("independent_audit"):
        sources.append(legacy["analysis"]["independent_audit"]["evidence"])
    for identity, item in ledger["tasks"].items():
        if item["status"] != "done":
            continue
        result = item["result"]
        path = ROOT / item["definition"]["result"]
        require(path.is_file(), f"{identity} requires its saved dated result file")
        body = path.read_text()
        for marker in (f"# {identity}", result["completed_at"], result["outcome"],
                       "## Check performed", "## Evidence", "## Result", "## Limits"):
            require(marker in body, f"{identity} result is missing {marker}")
        sources.extend(result["evidence"])
    decision = ledger["research"]["zero_decision"]
    if decision:
        sources.extend(decision["evidence"])
    for source in sources:
        require((ROOT / source).is_file(), f"missing evidence: {source}")
