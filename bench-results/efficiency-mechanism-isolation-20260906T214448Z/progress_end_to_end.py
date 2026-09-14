"""Append-only complete-known-work ledger, independent of benchmark execution."""
import hashlib
import json
import re

import progress_counter as core

SCOPE_SHA256 = "865ecf8b6ed7269119bcc7f4d751c34f16942aac52feb95935d7d9a81e90449b"
CONDITIONAL = {"P06", "P07", "P08", "P15"}
ANSWERS = {"causal_explanation", "benchmark_error"}


def load_contract():
    raw = (core.STUDY / "PROGRESS-END-TO-END-CONTRACT-20260914.json").read_bytes()
    core.require(hashlib.sha256(raw).hexdigest() == SCOPE_SHA256,
                 "the approved sixteen tasks, budgets and novelty rules are frozen")
    scope = json.loads(raw)
    records = {}
    for name in ("parent_ledger", "parent_publications", "plan", "authority"):
        item = scope[name]
        data = (core.ROOT / item["path"]).read_bytes()
        core.require(hashlib.sha256(data).hexdigest() == item["sha256"],
                     "preserve the prior zero, exact plan and conditional user authority")
        if name.startswith("parent_"):
            records[name] = json.loads(data)
    parent, pubs = records["parent_ledger"], records["parent_publications"]
    core.require(core.validate_published_progress(parent, pubs) == (74, 0),
                 "the new scope starts after the immutable 74/0 checkpoint")
    return scope, parent, pubs


def checkpoint(scope, parent):
    return {"event": "approved_scope_extension", "approval_id": scope["id"],
            "at": scope["approved_on"], "done": 74, "todo": 16, "total": 90,
            "completed": list(parent["completed"])}


def supported(ledger, identity):
    item = ledger["tasks"][identity]
    return item["status"] == "done" and item["result"]["outcome"] == "supported"


def validate_gate(ledger, identity, item):
    result = item["result"]
    core.keys(result, "completed_at outcome checked evidence finding limits gate")
    core.require(result["outcome"] in core.OUTCOMES - {"failed_setup"}
                 and core.strings(result["evidence"])
                 and all(core.nonempty(result[key]) for key in ("checked", "finding", "limits")),
                 "setup failure or unavailable work cannot finish a research obligation")
    core.require(core.timestamp(result["completed_at"]) >= core.timestamp(item["started_at"]),
                 "completion must follow its recorded start")
    gate = result["gate"]
    core.keys(gate, "mode setup_valid performed deliverable_complete answer_kind")
    core.require(all(type(gate[key]) is bool for key in
                     ("setup_valid", "performed", "deliverable_complete"))
                 and gate["setup_valid"] and gate["deliverable_complete"],
                 "completion requires qualified setup and the actual required deliverable")
    core.require(gate["answer_kind"] is None or gate["answer_kind"] in ANSWERS,
                 "answer kind must name a supported causal answer or benchmark error")
    mode = gate["mode"]
    if mode == "performed":
        core.require(gate["performed"] and result["outcome"] != "justified_not_needed",
                     "an unrun experiment cannot be a performed check")
        if item["definition"]["kind"] in {"screen", "confirmation"}:
            core.require(supported(ledger, "P01"),
                         "generated checks require completed input/setup qualification")
        if identity == "P01":
            core.require(result["outcome"] == "supported",
                         "qualification stays unfinished until the actual gate passes")
    else:
        core.require(not gate["performed"] and result["outcome"] == "justified_not_needed",
                     "a discharged conditional obligation is not a performed experiment")
        if mode == "absent_or_shared":
            core.require(identity in {"P06", "P07", "P08"},
                         "only the declared conditional mechanisms use source exclusions")
        elif mode == "covered_by_verified_joint":
            core.require(identity in CONDITIONAL and supported(ledger, "P13")
                         and ledger["tasks"]["P13"]["definition"]["result"] in result["evidence"],
                         "conditional discharge needs linked verified historical attribution")
        elif mode == "verified_benchmark_error":
            error = ledger["tasks"]["P15"]
            core.require(identity not in {"P10", "P15", "P16"}
                         and supported(ledger, "P15")
                         and error["result"]["gate"]["answer_kind"] == "benchmark_error"
                         and error["definition"]["result"] in result["evidence"],
                         "error-outcome discharge needs the linked demonstrated benchmark error")
        else:
            core.require(False, "no unlisted completion or skip basis")
    if identity == "P16":
        core.require(mode == "performed" and result["outcome"] == "supported"
                     and gate["answer_kind"] in ANSWERS,
                     "final acceptance cannot close on an unavailable answer")
        required = ("P13", "P14") if gate["answer_kind"] == "causal_explanation" else ("P15",)
        core.require(all(supported(ledger, key) and ledger["tasks"][key]["definition"]["result"]
                         in result["evidence"] for key in required),
                     "final acceptance needs linked supported attribution/recipe or corrected error")
        if gate["answer_kind"] == "benchmark_error":
            core.require(ledger["tasks"]["P15"]["result"]["gate"]["answer_kind"] == "benchmark_error",
                         "a no-error-found audit is not a demonstrated error")


def validate_issues(ledger, scope):
    core.require(isinstance(ledger["scope_issues"], list), "scope issues are explicit")
    for issue in ledger["scope_issues"]:
        core.keys(issue, "case status classification discovered_at reason new_fact why_not_known "
                        "why_promising evidence proposed_tasks")
        classification = issue["classification"]
        core.require(classification in {"unexpected_issue", "new_promising_mechanism", "known_omission"}
                     and issue["status"] == "unapproved",
                     "a discovery record never grants itself extra scope")
        core.require(type(issue["case"]) is int and
                     issue["case"] == (2 if classification == "known_omission" else 1),
                     "known omissions are case (2), not unexpected discoveries")
        core.require(core.timestamp(issue["discovered_at"]) >= core.timestamp(scope["approved_on"])
                     and core.strings(issue["evidence"])
                     and all(core.nonempty(issue[key]) for key in
                             ("reason", "new_fact", "why_not_known", "why_promising")),
                     "record dated new evidence, what was unknown, and explanatory relevance")
        core.validate_proposals(issue["proposed_tasks"])
        core.require(not {task["id"] for task in issue["proposed_tasks"]} & set(ledger["tasks"]),
                     "do not rename or reopen an existing task as new work")


def validate_ledger(ledger):
    scope, parent, _ = load_contract()
    from progress_repairs import preparation_limits
    fields = "schema scope_id fixed_total completed pending tasks history research scope_issues"
    core.keys(ledger, fields + (" budget_exceptions" if "budget_exceptions" in ledger else ""))
    core.require(type(ledger["schema"]) is int and ledger["schema"] == 5
                 and type(ledger["fixed_total"]) is int and ledger["fixed_total"] == 90
                 and ledger["scope_id"] == scope["id"], "only the approved fixed-90 scope is active")
    definitions = {item["id"]: item for item in scope["tasks"]}
    completed, pending = ledger["completed"], ledger["pending"]
    core.require(core.strings(completed) and core.strings(pending, allow_empty=True),
                 "stable completed/pending identities are required")
    core.require(completed[:74] == parent["completed"]
                 and len(completed) + len(pending) == 90
                 and len(set(completed + pending)) == 90
                 and set(completed + pending) == set(parent["completed"]) | set(definitions),
                 "preserve all 74 completions; no extra, removed or reopened task")
    core.require(pending == [key for key in definitions if key not in completed],
                 "TODO is exactly the remaining approved obligations")
    core.require(isinstance(ledger["tasks"], dict)
                 and set(ledger["tasks"]) == set(parent["tasks"]) | set(definitions),
                 "no hidden supplementary work")
    limits = preparation_limits(ledger)
    for identity, record in parent["tasks"].items():
        core.require(ledger["tasks"][identity] == record, "old completed records are immutable")
    for identity, definition in definitions.items():
        item = ledger["tasks"][identity]
        core.keys(item, "definition status started_at result blocker preparation_seconds")
        core.require(item["definition"] == definition, "task scope, question, gates and limits are pinned")
        spent = item["preparation_seconds"]
        core.require(type(spent) is int and spent >= 0,
                     "preparation spending must be explicit and nonnegative")
        if spent > limits[identity]:
            core.require(item["status"] == "blocked",
                         "budget exhaustion stops work; retain any actual overrun as blocked")
        if item["started_at"] is not None:
            core.require(core.timestamp(item["started_at"]) >= core.timestamp(scope["approved_on"]),
                         "new work starts after its approval")
        if item["status"] == "blocked":
            blocker = item["blocker"]
            core.keys(blocker, "detected_at reason evidence")
            core.timestamp(blocker["detected_at"])
            core.require(core.nonempty(blocker["reason"]) and core.strings(blocker["evidence"]),
                         "blocked work needs an exact evidenced reason")
        else:
            core.require(item["blocker"] is None, "nonblocked status cannot hide a blocker")
        if identity in pending:
            core.require(item["status"] in {"pending", "checking", "blocked"} and item["result"] is None,
                         "blocked/unrun work stays TODO, with no completion result")
            if item["status"] in {"checking", "blocked"}:
                core.timestamp(item["started_at"])
            if item["status"] == "checking" and definition["kind"] in {"screen", "confirmation"}:
                core.require(supported(ledger, "P01"),
                             "generated work cannot start behind unfinished setup")
            continue
        core.require(item["status"] == "done", "completed IDs require completed obligations")
        validate_gate(ledger, identity, item)
    history = ledger["history"]
    boundary = len(parent["history"])
    core.require(isinstance(history, list) and len(history) >= boundary + 1
                 and history[:boundary] == parent["history"]
                 and history[boundary] == checkpoint(scope, parent),
                 "retain every old publication and the exact approved 74/16 extension")
    previous = list(parent["completed"])
    for point in history[boundary + 1:]:
        core.keys(point, "event scope_id at done todo total completed completion")
        core.keys(point["completion"], "id record")
        identity = point["completion"]["id"]
        core.require(identity in definitions and identity not in previous,
                     "advance exactly one new approved obligation")
        if identity == "P16":
            core.require(set(definitions) - {"P16"} <= set(previous),
                         "final acceptance requires every input obligation first")
        expected = previous + [identity]
        core.require(point["event"] == "task_completed" and point["scope_id"] == scope["id"]
                     and all(type(point[key]) is int for key in ("done", "todo", "total"))
                     and point["total"] == 90 and point["done"] == len(expected)
                     and point["todo"] == 90 - len(expected) and point["completed"] == expected,
                     "each completion is exactly DONE +1 / TODO -1")
        item = ledger["tasks"][identity]
        core.require(point["completion"]["record"] == item
                     and point["at"] == item["result"]["completed_at"],
                     "a published completed result cannot be rewritten")
        previous = expected
    core.require(previous == completed, "every completion needs its retained checkpoint")
    validate_issues(ledger, scope)
    basic_issues = [{key: issue[key] for key in ("case", "status", "reason", "proposed_tasks")}
                    for issue in ledger["scope_issues"]]
    core.validate_zero_decision(dict(ledger, scope_issues=basic_issues),
                               {key: item["definition"] for key, item in ledger["tasks"].items()},
                               final_id="P16")
    return len(completed), len(pending)


def validate_publications(ledger, publications):
    counts = validate_ledger(ledger)
    scope, parent, parent_pubs = load_contract()
    core.keys(publications, "schema scope_id fixed_total checkpoints")
    core.require(type(publications["schema"]) is int and publications["schema"] == 4
                 and publications["scope_id"] == scope["id"]
                 and type(publications["fixed_total"]) is int and publications["fixed_total"] == 90,
                 "publication history must use the approved complete-plan scope")
    core.require(publications["checkpoints"] == parent_pubs["checkpoints"]
                 + ledger["history"][len(parent["history"]):],
                 "preserve all earlier checkpoints, including the 74/0 zero")
    return counts


def validate_visible_progress(ledger, hypothesis, note):
    counts = validate_ledger(ledger)
    lines = hypothesis.splitlines()
    core.require(bool(lines) and core.read_counts(lines[0]) == counts, "header and ledger disagree")
    note_lines = note.splitlines()
    core.require(len(note_lines) > 2 and lines[0][2:] in note_lines[2], "note and header disagree")
    rows = re.findall(r"^\| (P\d\d) \| (TODO|CHECKING|BLOCKED|DONE) \| ([^|]+) \|$",
                      "\n".join(lines[:35]), re.MULTILINE)
    scope, _, _ = load_contract()
    statuses = {"pending": "TODO", "checking": "CHECKING", "blocked": "BLOCKED", "done": "DONE"}
    expected = [(item["id"], statuses[ledger["tasks"][item["id"]]["status"]], item["question"])
                for item in scope["tasks"]]
    core.require(rows == expected, "every approved P task, including blocked checks, is visible")


def validate_auxiliary_evidence(ledger):
    for identity, item in ledger["tasks"].items():
        blocker = item.get("blocker")
        if blocker:
            for path in blocker["evidence"]:
                core.require((core.ROOT / path).is_file(), f"{identity} blocker evidence is missing")
    for issue in ledger["scope_issues"]:
        for path in issue["evidence"]:
            core.require((core.ROOT / path).is_file(), "new-discovery evidence is missing")
