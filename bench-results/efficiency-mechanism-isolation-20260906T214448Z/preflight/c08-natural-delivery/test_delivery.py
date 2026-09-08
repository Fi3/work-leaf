"""Pure source-consistency regressions; no saved workflow payloads are read.

Synthetic renderer bytes are independent fixtures, not saved workflow payloads.
No imports from a provider, runtime executor, scorer or accounting helper.
"""
import copy
import hashlib
import unittest
from unittest.mock import patch

from validate_automatic_refresh import census_trace, validate_event


RUN = "synthetic-v7-run"
SCHEMA = "work-leaf-bench-experiment-v7"
CONDITION = "automatic-changed-refresh-full"
INTRO_OLD = "This is a compact refresh, not a patch to submit. It shows changes from the last file text this agent received. Repeated full-text refreshes are intentionally avoided to keep the session compact.\n"
INTRO_NEW = "This is a file refresh, not a patch to submit. Sections marked current full text contain the complete current file; all other sections keep their stated snapshot or diagnostic meaning.\n"
FORMATS = {
    "patch": "Resend the complete unified diff through `@work-leaf patch <reason>` followed by the patch body and `@work-leaf end`. Do not use placeholder `@@` hunk headers. Every hunk must use real unified-diff line ranges such as `@@ -old_start,old_count +new_start,new_count @@` from the current file text.",
    "edit": "Resend the complete edit through `@work-leaf edit <reason>` followed by an apply-patch-style body and `@work-leaf end`. Use `*** Begin Patch`, one or more `*** Update File: path` sections, `@@` hunk separators without line numbers, exact unchanged context lines prefixed with a space, old lines prefixed with `-`, new lines prefixed with `+`, and `*** End Patch`. Include enough unchanged context for each old block to match exactly one place in the current file.",
}


def fnv_fixture(body):
    value = 0xcbf29ce484222325
    for byte in body:
        value = ((value ^ byte) * 0x100000001b3) & ((1 << 64) - 1)
    return f"fnv64:{value:016x}; bytes:{len(body)}"


def snapshot(path="alpha.rs", current="new\n", previous="old\n", disposition="available", diff="-old\n+new\n"):
    return dict(path=path, current=current, previous=previous, disposition=disposition, diff=diff)


def event_fixture(items=None, kind="edit", diagnostic="genuine fixture conflict", failures=None):
    items = items if items is not None else [snapshot()]
    failures = failures if failures is not None else []
    files = [x["path"] for x in items] + [x["path"] for x in failures]
    label = "Git diagnostic" if kind == "patch" else "Diagnostic"
    old = f"The orchestrator could not apply your {kind}.\nFiles: {', '.join(files) or '-'}\n\n{label}:\n{diagnostic}\n\n".encode()
    cue_old = ("Rebase your patch" if kind == "patch" else "Rebase your exact edit blocks") + " against the compact file refresh below."
    cue_new = cue_old.replace("compact file refresh", "file refresh")
    replacements = [("refresh-guidance", len(old), len(old) + len(cue_old.encode()), cue_new.encode(), None, None)]
    old += cue_old.encode() + b"\n" + FORMATS[kind].encode() + b"\n\nwork-leaf file refresh\n"
    replacements.append(("refresh-intro", len(old), len(old) + len(INTRO_OLD.encode()), INTRO_NEW.encode(), None, None))
    old += INTRO_OLD.encode()
    rows = []
    for index, item in enumerate(items):
        body = item["current"].encode()
        previous = None if item["previous"] is None else item["previous"].encode()
        cls = "untracked" if previous is None else "unchanged" if body == previous else "changed"
        row = dict(path=item["path"], bytes=len(body), digest=fnv_fixture(body),
                   previous_bytes=None if previous is None else len(previous),
                   previous_digest=None if previous is None else fnv_fixture(previous),
                   **{"class": cls}, diff_disposition="not-applicable", diff_bytes=None,
                   baseline_section_start=len(old), baseline_body_start=None, baseline_body_end=None)
        old += f"\n--- {item['path']} ---\ncurrent digest: {row['digest']}\n".encode()
        if cls == "untracked":
            old += b"status: no previous snapshot recorded for this agent\n"
            if len(body) <= 8192:
                old += f"work-leaf file text\n\n--- {item['path']} ---\n".encode() + body
                if not body.endswith(b"\n"): old += b"\n"
            else:
                old += f"current file text omitted: file is {len(body)} bytes. Request mediated file text with `@work-leaf read {item['path']}` if this file is still needed.\n".encode()
        else:
            old += f"previous digest: {row['previous_digest']}\n".encode()
            old += f"status: {cls} since this agent's last snapshot\n".encode()
            if cls == "changed":
                disposition = item["disposition"]
                row["diff_disposition"] = disposition
                if disposition in ("available", "empty"):
                    diff = item["diff"].encode()
                    row.update(diff_bytes=len(diff), baseline_body_start=len(old), baseline_body_end=len(old) + len(diff))
                    begin = len(old); old += diff
                    if not diff.endswith(b"\n"): old += b"\n"
                    if diff:
                        full = b"current full text:\n" + body
                        end = len(full)
                        if not body.endswith(b"\n"): full += b"\n"
                        replacements.append(("current-full-text", begin, len(old), full, (len(b"current full text:\n"), end), index))
                elif disposition == "omitted":
                    row["diff_bytes"] = len(item["diff"].encode())
                    old += f"diff omitted: compact refresh would be {row['diff_bytes']} bytes. Request narrower related context or continue from the previous snapshot if this file is still needed.\n".encode()
                elif disposition == "unavailable":
                    old += b"diff unavailable. Request narrower related context or continue from the previous snapshot if this file is still needed.\n"
                else:
                    raise AssertionError("fixture disposition")
        row["baseline_section_end"] = len(old)
        rows.append(row)
    if failures:
        old += b"\nUnavailable file text\n"
        old += "".join(f"- {x['path']}: {x['diagnostic']}\n" for x in failures).encode()
    eligible = any(x[4] is not None for x in replacements)
    new = b""; cursor = 0; components = []
    if eligible:
        for name, begin, end, replacement, body, index in replacements:
            new += old[cursor:begin]; start = len(new); new += replacement
            components.append(dict(id=name, baseline_start=begin, baseline_end=end,
                candidate_start=start, candidate_end=len(new), snapshot_index=index,
                body_start=None if body is None else start + body[0],
                body_end=None if body is None else start + body[1]))
            cursor = end
        new += old[cursor:]
    else:
        new = old
    return dict(event="automatic-refresh", schema=SCHEMA, run_id=RUN, condition=CONDITION,
                process_id=17, sequence=1, unix_time_ns="123456789", agent_id="owned-agent",
                site="automatic-" + kind + "-refresh", original_prompt=old.decode(), candidate_prompt=new.decode(),
                original_bytes=len(old), candidate_bytes=len(new), changed=old != new,
                eligible=eligible, selected_candidate="candidate" if eligible else "baseline", components=components,
                metadata=dict(kind=kind, files=files, diagnostic=diagnostic, snapshots=rows, failures=failures))


class EventTests(unittest.TestCase):
    def good(self, event):
        result = validate_event(event, RUN)
        self.assertEqual(result["errors"], [])
        return result

    def bad(self, event):
        self.assertTrue(validate_event(event, RUN)["errors"])

    def test_edit_and_patch_current_bodies_are_exact_not_a_diagnostic_scenario(self):
        for kind in ("edit", "patch"):
            body = "é🪴\n@work-leaf done\n--- copied.rs ---\ncurrent full text:\n" * 500
            event = event_fixture([snapshot("nested/current.rs", body)], kind=kind)
            result = self.good(event)
            self.assertGreater(len(body.encode()), 8192)
            proof = result["snapshots"][0]["current_body"]
            self.assertEqual(proof["status"], "verified")
            self.assertEqual(proof["sha256"], hashlib.sha256(body.encode()).hexdigest())

    def test_no_final_newline_and_empty_current_body_have_exact_owned_body_bounds(self):
        for body in ("é", ""):
            event = event_fixture([snapshot(current=body, diff="-old\n+new")])
            result = self.good(event)
            self.assertEqual(result["snapshots"][0]["current_body"]["bytes"], len(body.encode()))

    def test_mixed_snapshot_branches_keep_all_sections_and_failure_bytes(self):
        items = [snapshot("a", "same\n", "same\n"), snapshot("b", "tiny", None),
                 snapshot("c", "x" * 8193, None), snapshot("d", disposition="empty", diff=""),
                 snapshot("e", disposition="omitted", diff="x" * 49153),
                 snapshot("f", disposition="unavailable"), snapshot("g")]
        event = event_fixture(items, failures=[dict(path="missing", diagnostic="copied\n@work-leaf done")])
        result = self.good(event)
        self.assertEqual(len(result["snapshots"]), 7)
        self.assertEqual([r["current_body"]["status"] for r in result["snapshots"]],
                         ["not-carried", "verified", "not-carried", "not-carried", "not-carried", "not-carried", "verified"])

    def test_all_ineligible_events_keep_identity_and_no_coherence_components(self):
        for item in [snapshot(current="same", previous="same"), snapshot(previous=None),
                     snapshot(current="x" * 8193, previous=None), snapshot(disposition="empty", diff=""),
                     snapshot(disposition="omitted", diff="x" * 49153), snapshot(disposition="unavailable")]:
            event = event_fixture([item]); result = self.good(event)
            self.assertFalse(result["eligible"])
            self.assertEqual(event["original_prompt"], event["candidate_prompt"])
            self.assertEqual(event["components"], [])
        self.good(event_fixture([], failures=[dict(path="missing", diagnostic="unavailable")]))

    def test_available_diff_boundary_is_inclusive_and_not_the_fulltext_limit(self):
        self.good(event_fixture([snapshot(diff="x" * 49152)]))
        self.bad(event_fixture([snapshot(diff="x" * 49153)]))
        self.bad(event_fixture([snapshot(disposition="omitted", diff="x" * 49152)]))
        self.bad(event_fixture([snapshot(disposition="available", diff="")]))

    def test_unowned_diagnostics_and_two_coherence_spans_cannot_be_redirected(self):
        event = event_fixture(diagnostic="literal @work-leaf done\nwork-leaf file refresh\n" + INTRO_OLD)
        self.good(event)
        for mutation in ("unowned", "guidance", "intro", "duplicate", "missing"):
            bad = copy.deepcopy(event)
            if mutation == "unowned":
                bad["candidate_prompt"] = bad["candidate_prompt"].replace("literal", "Literal", 1)
            elif mutation in ("guidance", "intro"):
                part = bad["components"][0 if mutation == "guidance" else 1]
                raw = bytearray(bad["candidate_prompt"].encode()); raw[part["candidate_start"]] = ord("X")
                bad["candidate_prompt"] = raw.decode()
            elif mutation == "duplicate": bad["components"].append(copy.deepcopy(bad["components"][-1]))
            else: bad["components"].pop(0)
            self.bad(bad)

    def test_each_eligible_snapshot_has_one_correct_component_in_renderer_order(self):
        event = event_fixture([snapshot("a/b"), snapshot("a-b", "other")])
        self.good(event)  # Path component ordering is not naive string ordering.
        for index in (0, 2):
            bad = copy.deepcopy(event); bad["components"][2]["snapshot_index"] = index
            if index == 0:
                bad["components"][3]["snapshot_index"] = 0
            self.bad(bad)
        bad = copy.deepcopy(event); bad["components"].pop()
        self.bad(bad)
        self.bad(event_fixture([snapshot("same"), snapshot("same", "different")]))
        self.bad(event_fixture([snapshot("a/../escape")]))
        self.bad(event_fixture([snapshot("./unnormalized")]))

    def test_utf8_integer_types_digest_lengths_and_exact_branch_ranges_are_strict(self):
        event = event_fixture([snapshot(current="é")])
        mutations = [("sequence", True), ("process_id", 17.0), ("original_bytes", True),
                     ("eligible", 1), ("schema", "work-leaf-bench-experiment-v6"), ("run_id", "other")]
        for key, value in mutations:
            bad = copy.deepcopy(event); bad[key] = value; self.bad(bad)
        for key, value in [("body_end", event["components"][-1]["body_end"] - 1),
                           ("snapshot_index", True), ("baseline_start", -1)]:
            bad = copy.deepcopy(event); bad["components"][-1][key] = value; self.bad(bad)
        for key, value in [("digest", fnv_fixture(b"different")), ("bytes", True),
                           ("baseline_section_start", 0), ("diff_bytes", None)]:
            bad = copy.deepcopy(event); bad["metadata"]["snapshots"][0][key] = value; self.bad(bad)

    def test_metadata_only_current_or_previous_bytes_are_never_invented(self):
        result = self.good(event_fixture([snapshot(disposition="unavailable")]))
        self.assertEqual(result["snapshots"][0]["current_body"], {"status": "not-carried"})
        self.assertEqual(result["snapshots"][0]["previous_body"], {"status": "not-carried"})

    def test_overlapping_components_are_rejected_before_any_body_hashing(self):
        event = event_fixture([snapshot("first"), snapshot("second")])
        event["components"][3] = dict(event["components"][2], snapshot_index=1)
        with patch("validate_automatic_refresh.body_proof") as verify:
            self.bad(event)
            verify.assert_not_called()

    def test_malformed_json_types_unknown_fields_and_utf8_fail_closed(self):
        event = event_fixture()
        for path in [("metadata",), ("components",), ("metadata", "snapshots"),
                     ("metadata", "failures"), ("metadata", "kind"), ("metadata", "files"),
                     ("process_id",), ("sequence",), ("agent_id",), ("original_prompt",)]:
            for value in (None, False, [], {}, 1.0):
                bad = copy.deepcopy(event); target = bad
                for key in path[:-1]: target = target[key]
                target[path[-1]] = value
                # Empty failures is already a valid inventory.
                if path == ("metadata", "failures") and value == []: continue
                self.bad(bad)
        for path in [(), ("metadata",), ("components", 0), ("metadata", "snapshots", 0)]:
            bad = copy.deepcopy(event); target = bad
            for key in path: target = target[key]
            target["unexpected"] = "private body"
            self.bad(bad)
        bad = copy.deepcopy(event); bad["original_prompt"] = "\ud800"
        self.bad(bad)
        result = self.good(event_fixture())
        self.assertEqual(result["snapshots"][0]["previous_body"], {"status": "not-carried"})


class CensusTests(unittest.TestCase):
    def test_malformed_rows_and_invalid_line_maps_never_truncate_population(self):
        activation = dict(event="activation", schema=SCHEMA, condition=CONDITION, run_id=RUN, process_id=17)
        rows = [activation, None, "private body", [], event_fixture()]
        for lines in (None, [], [1], [1, 2, 3, 4, 5.0]):
            result = census_trace(rows, RUN, lines)
            self.assertEqual(len(result["events"]), 4)
            self.assertTrue(result["errors"])
            self.assertNotIn("private body", repr(result))
    def test_activation_run_process_sequence_and_physical_line_identity_are_required(self):
        activation = dict(event="activation", schema=SCHEMA, condition=CONDITION, run_id=RUN, process_id=17)
        event = event_fixture()
        for rows in [[], [event], [activation, activation, event],
                     [dict(activation, run_id="foreign"), event],
                     [dict(activation, process_id=True), event],
                     [activation, dict(event, process_id=18)],
                     [activation, dict(event, run_id="foreign")],
                     [activation, dict(event, sequence=2)],
                     [activation, dict(event, sequence=True)]]:
            with self.subTest(rows=len(rows)):
                self.assertTrue(census_trace(rows, RUN)["errors"])
        for lines in [[], [1], [1, 1], [2, 1], [0, 2], [True, 2], [1, 2.0]]:
            self.assertTrue(census_trace([activation, event], RUN, lines)["errors"])
        # Blank physical lines are allowed; trace event sequences are contiguous.
        self.assertFalse(census_trace([activation, event], RUN, [1, 3])["errors"])

    def test_invalid_and_unknown_rows_stay_in_the_physical_event_population(self):
        activation = dict(event="activation", schema=SCHEMA, condition=CONDITION, run_id=RUN, process_id=17)
        good = event_fixture(); bad = copy.deepcopy(good); bad["sequence"] = 2; bad["eligible"] = 1
        unknown = dict(event="unknown", schema=SCHEMA, condition=CONDITION, run_id=RUN, process_id=17, sequence=3)
        result = census_trace([activation, good, bad, unknown], RUN, [1, 3, 7, 9])
        self.assertTrue(result["errors"])
        self.assertEqual([r["trace_line"] for r in result["events"]], [3, 7, 9])
        self.assertNotIn("candidate_prompt", repr(result))
        self.assertNotIn("original_prompt", repr(result))

    def test_zero_automatic_rows_are_a_trace_fact_not_proof_of_zero_natural_conflicts(self):
        activation = dict(event="activation", schema=SCHEMA, condition=CONDITION, run_id=RUN, process_id=17)
        result = census_trace([activation], RUN)
        self.assertEqual(result["events"], [])
        self.assertFalse(result["complete_delivery_proven"])


ACK = "run at most one focused validation step that is relevant to files you touched or checks you added."
REMAINING = "After the focused validation passes, or after you report an external blocker, emit a top-level `@work-leaf done` so review can start. Send another edit only if validation found a concrete issue in your own patch."
GUIDANCE = "\nnext: Reply with the next Work Leaf directive, such as `@work-leaf done`, `@work-leaf edit`, `@work-leaf read`, or another `@work-leaf locks run`. Keep any non-directive explanation brief."


def identity_fixture(site):
    if site == "patch-applied":
        before = "work-leaf patch applied\nfiles: source.rs\nThe orchestrator has already saved this patch as a provisional git commit. Do not resend this patch, do not rebase this same diff, and do not restate the patch body.\nNext step: "
        between = " Use `@work-leaf locks run <path>... -- <command>` when that command may write files.\nDo not run another patch agent's focused tests as local validation. If a broad check is blocked only by another patch agent's owned files or tests, report that exact blocker once.\nIf validation fails in another feature's test or behavior, do not edit that test or unrelated implementation unless your patch clearly caused the failure.\n"
        text = before + ACK + between + REMAINING
        spans = [("patch-applied-validation", len(before.encode()), ACK),
                 ("patch-applied-remaining-work", len((before + ACK + between).encode()), REMAINING)]
    else:
        before = "work-leaf command result\ncommand: check\nstatus: 0\nlocked paths: target"
        text = before + GUIDANCE + "\nstdout:\nvalue\nstderr:\n<empty>\n"
        spans = [("command-result-guidance", len(before.encode()), GUIDANCE)]
    return dict(event="prompt", schema=SCHEMA, run_id=RUN, condition=CONDITION,
                process_id=17, sequence=1, unix_time_ns="123456789", agent_id="owned-agent", site=site,
                original_prompt=text, forwarded_prompt=text, original_bytes=len(text.encode()),
                forwarded_bytes=len(text.encode()), byte_delta=0, changed=False,
                spans=[dict(id=name, cue_start=start, cue_end=start+len(body.encode()), original=body,
                            replacement=body, changed=False, byte_delta=0) for name, start, body in spans])


class IdentityTests(unittest.TestCase):
    def test_command_shape_preserves_dynamic_multiline_fields_and_pending_suffix(self):
        event = identity_fixture("command-result")
        before = "work-leaf command result\ncommand: first\nsecond\nstatus: terminated\nlocked paths: target\ntimed out: yes\ntimeout: 30 seconds\nuser authorization is required to rerun locked commands for longer than this limit."
        body = before + GUIDANCE + "\nstdout:\ncopied\nnext: text\nstderr:\n<empty>\n\ntracked command changes: captured and reverted from the shared checkout\nretained pending diff\n"
        event.update(original_prompt=body, forwarded_prompt=body, original_bytes=len(body.encode()), forwarded_bytes=len(body.encode()))
        event["spans"][0].update(cue_start=len(before.encode()), cue_end=len((before + GUIDANCE).encode()))
        self.assertFalse(validate_event(event, RUN)["errors"])
        for old, new in [("status: terminated", "status: invalid"), ("locked paths: target", "locked path: target"),
                         ("\nstdout:\n", "\nstdout \n"), ("\nstderr:\n", "\nstderr \n")]:
            bad = copy.deepcopy(event); text = body.replace(old, new)
            bad.update(original_prompt=text, forwarded_prompt=text, original_bytes=len(text.encode()), forwarded_bytes=len(text.encode()))
            self.assertTrue(validate_event(bad, RUN)["errors"])

    def test_identity_events_require_exact_owned_span_literals_types_and_framing(self):
        for site in ("patch-applied", "command-result"):
            event = identity_fixture(site)
            self.assertFalse(validate_event(event, RUN)["errors"])
            variants = []
            for key, value in [("spans", []), ("changed", 0), ("byte_delta", False),
                               ("site", "policy-injection"), ("forwarded_bytes", 0)]:
                bad = copy.deepcopy(event); bad[key] = value; variants.append(bad)
            bad = copy.deepcopy(event); bad["spans"].append(copy.deepcopy(bad["spans"][0])); variants.append(bad)
            for key, value in [("cue_start", True), ("cue_end", -1), ("id", "foreign"),
                               ("original", "same-looking"), ("replacement", "changed"),
                               ("changed", 0), ("byte_delta", False)]:
                bad = copy.deepcopy(event); bad["spans"][0][key] = value; variants.append(bad)
            bad = copy.deepcopy(event)
            bad["original_prompt"] = "X" + bad["original_prompt"][1:]
            bad["forwarded_prompt"] = bad["original_prompt"]
            variants.append(bad)
            for bad in variants:
                with self.subTest(site=site, bad=bad.get("site")):
                    self.assertTrue(validate_event(bad, RUN)["errors"])

    def test_invalid_identity_row_is_retained_in_census_not_counted_as_success(self):
        activation = dict(event="activation", schema=SCHEMA, condition=CONDITION, run_id=RUN, process_id=17)
        event = identity_fixture("patch-applied"); event["spans"] = []
        result = census_trace([activation, event], RUN)
        self.assertTrue(result["errors"])
        self.assertEqual(len(result["events"]), 1)
        self.assertTrue(result["events"][0]["errors"])


if __name__ == "__main__":
    unittest.main()
