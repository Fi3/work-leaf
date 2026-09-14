"""Approved same-task allowance and a separate unforeseen-bug event counter."""
import hashlib
import json
import re

import progress_counter as core

AUTHORITY = "bench-results/efficiency-mechanism-isolation-20260906T214448Z/AUTHORITY-P01-EXTRA-AND-BUGFIX-20260914.md"
AUTHORITY_SHA = "b321e20d9e784dea8ee560e1dd695c67b1e0d132b7161969f429a77dcd42c7f2"
EXCEPTION_SHA = "f17a9c912531368db2ad3f1c9a3e7a95d5142c4d5839b654a2a0634204efc484"


def approved_exception():
    raw = (core.STUDY / "P01-BUDGET-EXCEPTION-20260914.json").read_bytes()
    core.require(hashlib.sha256(raw).hexdigest() == EXCEPTION_SHA,
                 "preserve the exact approved extra allowance")
    core.require(hashlib.sha256((core.ROOT / AUTHORITY).read_bytes()).hexdigest() == AUTHORITY_SHA,
                 "preserve the exact user approval and bug-fix-only authority")
    return json.loads(raw)


def preparation_limits(ledger):
    limits = {key: item["definition"].get("preparation_limit_seconds", 0)
              for key, item in ledger["tasks"].items()}
    if "budget_exceptions" in ledger:
        exception = approved_exception()
        core.require(ledger["budget_exceptions"] == [exception["id"]],
                     "no repeated or unapproved budget exception")
        limits[exception["task"]] += exception["additional_preparation_seconds"]
    return limits


def validate_bugs(ledger):
    approved_exception()
    core.keys(ledger, "schema authority events")
    core.require(type(ledger["schema"]) is int and ledger["schema"] == 1
                 and ledger["authority"] == AUTHORITY and isinstance(ledger["events"], list),
                 "the bug counter requires the recorded standing authority")
    discovered, fixed = {}, set()
    previous_at = core.timestamp(approved_exception()["resumed_at"])
    for event in ledger["events"]:
        if event.get("event") == "discovered":
            core.keys(event, "event id at title related_task already_accounted evidence")
            identity = event["id"]
            core.require(identity == f"BUG{len(discovered) + 1:03d}"
                         and identity not in discovered and core.nonempty(event["title"])
                         and event["already_accounted"] is False
                         and re.fullmatch(r"P(0[1-9]|1[0-6])", event["related_task"]),
                         "count each genuinely unaccounted defect once, outside research tasks")
            evidence = event["evidence"]
            discovered[identity] = event["at"]
        elif event.get("event") == "fixed":
            core.keys(event, "event id at reproduction verification result")
            identity = event["id"]
            core.require(identity in discovered and identity not in fixed,
                         "only an outstanding bug can be completed, once")
            core.require(core.strings(event["reproduction"])
                         and core.strings(event["verification"]) and core.nonempty(event["result"]),
                         "a fixed bug needs reproduction, verification and its saved result")
            evidence = event["reproduction"] + event["verification"] + [event["result"]]
            fixed.add(identity)
        else:
            core.require(False, "no removal, reopen or unlisted bug-counter event")
        at = core.timestamp(event["at"])
        core.require(at >= previous_at, "preserve chronological bug events")
        previous_at = at
        core.require(core.strings(evidence)
                     and all((core.ROOT / path).is_file() for path in evidence),
                     "bug progress requires retained evidence files")
    return len(fixed), len(discovered) - len(fixed), len(discovered)
