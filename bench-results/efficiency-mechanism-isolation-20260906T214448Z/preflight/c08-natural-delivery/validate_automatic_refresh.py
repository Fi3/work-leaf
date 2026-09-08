"""Pure v7 source-consistency checks, not delivery, Git or accounting proof.

Only in-memory JSON-shaped values are accepted. No source bodies are exported.
Literal contracts are from the source identities recorded in DESIGN.md.
"""
import hashlib
import re


SCHEMA = "work-leaf-bench-experiment-v7"
CONDITION = "automatic-changed-refresh-full"
COMMON = {"event", "schema", "sequence", "run_id", "condition", "process_id",
          "unix_time_ns", "site", "agent_id", "original_prompt", "original_bytes", "changed"}
AUTO = COMMON | {"candidate_prompt", "candidate_bytes", "eligible", "selected_candidate", "components", "metadata"}
IDENTITY = COMMON | {"forwarded_prompt", "forwarded_bytes", "byte_delta", "spans"}
ACTIVATION = {"event", "schema", "run_id", "condition", "process_id"}
PART = {"id", "baseline_start", "baseline_end", "candidate_start", "candidate_end",
        "snapshot_index", "body_start", "body_end"}
SNAPSHOT = {"path", "bytes", "digest", "previous_bytes", "previous_digest", "class",
            "diff_disposition", "diff_bytes", "baseline_section_start", "baseline_section_end",
            "baseline_body_start", "baseline_body_end"}
SPAN = {"id", "cue_start", "cue_end", "original", "replacement", "changed", "byte_delta"}
INTRO_OLD = "This is a compact refresh, not a patch to submit. It shows changes from the last file text this agent received. Repeated full-text refreshes are intentionally avoided to keep the session compact.\n"
INTRO_NEW = "This is a file refresh, not a patch to submit. Sections marked current full text contain the complete current file; all other sections keep their stated snapshot or diagnostic meaning.\n"
FORMATS = {
    "patch": "Resend the complete unified diff through `@work-leaf patch <reason>` followed by the patch body and `@work-leaf end`. Do not use placeholder `@@` hunk headers. Every hunk must use real unified-diff line ranges such as `@@ -old_start,old_count +new_start,new_count @@` from the current file text.",
    "edit": "Resend the complete edit through `@work-leaf edit <reason>` followed by an apply-patch-style body and `@work-leaf end`. Use `*** Begin Patch`, one or more `*** Update File: path` sections, `@@` hunk separators without line numbers, exact unchanged context lines prefixed with a space, old lines prefixed with `-`, new lines prefixed with `+`, and `*** End Patch`. Include enough unchanged context for each old block to match exactly one place in the current file.",
}
ACK = "run at most one focused validation step that is relevant to files you touched or checks you added."
REMAINING = "After the focused validation passes, or after you report an external blocker, emit a top-level `@work-leaf done` so review can start. Send another edit only if validation found a concrete issue in your own patch."
ACK_BEFORE = "The orchestrator has already saved this patch as a provisional git commit. Do not resend this patch, do not rebase this same diff, and do not restate the patch body.\nNext step: "
ACK_BETWEEN = " Use `@work-leaf locks run <path>... -- <command>` when that command may write files.\nDo not run another patch agent's focused tests as local validation. If a broad check is blocked only by another patch agent's owned files or tests, report that exact blocker once.\nIf validation fails in another feature's test or behavior, do not edit that test or unrelated implementation unless your patch clearly caused the failure.\n"
GUIDANCE = "\nnext: Reply with the next Work Leaf directive, such as `@work-leaf done`, `@work-leaf edit`, `@work-leaf read`, or another `@work-leaf locks run`. Keep any non-directive explanation brief."


class Invalid(ValueError):
    """Contains a fixed, non-payload error code only."""


def require(condition, code):
    if not condition:
        raise Invalid(code)


def keys(value, expected):
    require(type(value) is dict and value.keys() == expected, "invalid-fields")


def integer(value, minimum=0):
    require(type(value) is int and value >= minimum, "invalid-integer")
    return value


def utf8(value):
    require(type(value) is str, "invalid-string")
    try:
        return value.encode("utf-8")
    except UnicodeEncodeError as exc:
        raise Invalid("invalid-utf8") from exc


def check_range(raw, start, end):
    integer(start); integer(end)
    require(start <= end <= len(raw), "invalid-range")
    require(all(offset == len(raw) or raw[offset] & 0xc0 != 0x80 for offset in (start, end)), "split-utf8")


def byte_range(raw, start, end):
    check_range(raw, start, end)
    return raw[start:end]


def digest(value, size):
    integer(size)
    require(type(value) is str and re.fullmatch(r"fnv64:[0-9a-f]{16}; bytes:(0|[1-9][0-9]*)", value) is not None,
            "invalid-digest")
    require(value.rsplit(":", 1)[1] == str(size), "digest-length")


def body_proof(raw, row):
    require(len(raw) == row["bytes"], "body-length")
    value = 0xcbf29ce484222325
    for byte in raw:
        value = ((value ^ byte) * 0x100000001b3) & 0xffffffffffffffff
    require(row["digest"] == f"fnv64:{value:016x}; bytes:{len(raw)}", "body-digest")
    return {"status": "verified", "bytes": len(raw), "sha256": hashlib.sha256(raw).hexdigest()}


def common(event, expected_run_id):
    require(type(expected_run_id) is str and bool(expected_run_id), "invalid-expected-run")
    require(event["schema"] == SCHEMA and event["condition"] == CONDITION, "foreign-schema-condition")
    require(type(event["run_id"]) is str and event["run_id"] == expected_run_id, "foreign-run")
    integer(event["process_id"], 1)
    integer(event["sequence"], 1)
    require(type(event["unix_time_ns"]) is str and re.fullmatch(r"[0-9]+", event["unix_time_ns"]) is not None,
            "invalid-time")
    require(bool(utf8(event["agent_id"])), "invalid-agent")
    require(type(event["changed"]) is bool, "invalid-boolean")
    original = utf8(event["original_prompt"])
    require(integer(event["original_bytes"]) == len(original), "original-length")
    return original


class Reader:
    def __init__(self, raw):
        self.raw = raw
        self.cursor = 0

    def expect(self, value):
        value = utf8(value) if type(value) is str else value
        end = self.cursor + len(value)
        require(self.raw[self.cursor:end] == value, "renderer-bytes")
        self.cursor = end

    def body(self, size):
        raw = byte_range(self.raw, self.cursor, self.cursor + size)
        self.cursor += size
        return raw


def automatic(event, original):
    candidate = utf8(event["candidate_prompt"])
    require(integer(event["candidate_bytes"]) == len(candidate), "candidate-length")
    require(type(event["eligible"]) is bool, "invalid-boolean")
    meta = event["metadata"]
    keys(meta, {"files", "diagnostic", "kind", "snapshots", "failures"})
    kind = meta["kind"]
    require(type(kind) is str and kind in FORMATS and event["site"] == f"automatic-{kind}-refresh", "invalid-site-kind")
    require(type(meta["files"]) is list and all(type(x) is str for x in meta["files"]), "invalid-files")
    require(type(meta["diagnostic"]) is str, "invalid-diagnostic")
    require(type(meta["snapshots"]) is list and type(meta["failures"]) is list, "invalid-inventory")
    parts = event["components"]
    require(type(parts) is list, "invalid-components")
    prior_baseline = 0; prior_candidate = 0
    for part in parts:
        keys(part, PART)
        check_range(original, part["baseline_start"], part["baseline_end"])
        check_range(candidate, part["candidate_start"], part["candidate_end"])
        require(part["baseline_start"] >= prior_baseline and part["candidate_start"] >= prior_candidate,
                "overlapping-components")
        prior_baseline = part["baseline_end"]; prior_candidate = part["candidate_end"]
        if part["snapshot_index"] is None:
            require(part["body_start"] is None and part["body_end"] is None, "bodyless-component")
        else:
            integer(part["snapshot_index"])
            check_range(candidate, part["body_start"], part["body_end"])
            require(part["candidate_start"] <= part["body_start"] <= part["body_end"] <= part["candidate_end"], "body-outside-component")
    reader = Reader(original)
    label = "Git diagnostic" if kind == "patch" else "Diagnostic"
    reader.expect(f"The orchestrator could not apply your {kind}.\nFiles: {', '.join(meta['files']) or '-'}\n\n{label}:\n{meta['diagnostic']}\n\n")
    cue = ("Rebase your patch" if kind == "patch" else "Rebase your exact edit blocks") + " against the compact file refresh below."
    begin = reader.cursor; reader.expect(cue)
    replacements = [("refresh-guidance", begin, reader.cursor, utf8(cue.replace("compact file refresh", "file refresh")), None, None)]
    reader.expect("\n" + FORMATS[kind] + "\n\nwork-leaf file refresh\n")
    begin = reader.cursor; reader.expect(INTRO_OLD)
    replacements.append(("refresh-intro", begin, reader.cursor, utf8(INTRO_NEW), None, None))
    paths = set(); proofs = []; eligible_count = 0
    for index, row in enumerate(meta["snapshots"]):
        keys(row, SNAPSHOT)
        path = row["path"]; utf8(path)
        require(bool(path) and path != "." and all(x not in ("", ".", "..") for x in path.split("/")) and "\0" not in path,
                "noncanonical-path")
        require(path not in paths, "duplicate-snapshot-path"); paths.add(path)
        digest(row["digest"], row["bytes"])
        integer(row["baseline_section_start"]); integer(row["baseline_section_end"])
        require(row["baseline_section_start"] == reader.cursor, "section-start")
        reader.expect(f"\n--- {path} ---\ncurrent digest: {row['digest']}\n")
        cls = row["class"]
        require(type(cls) is str and cls in ("changed", "unchanged", "untracked"), "invalid-class")
        proof = {"snapshot_index": index, "class": cls, "current_body": {"status": "not-carried"},
                 "previous_body": {"status": "not-carried"}}
        if cls == "untracked":
            require(row["previous_digest"] is None and row["previous_bytes"] is None, "untracked-previous")
            reader.expect("status: no previous snapshot recorded for this agent\n")
            if row["bytes"] <= 8192:
                reader.expect(f"work-leaf file text\n\n--- {path} ---\n")
                body = reader.body(row["bytes"])
                proof["current_body"] = body_proof(body, row)
                if not body.endswith(b"\n"): reader.expect(b"\n")
            else:
                reader.expect(f"current file text omitted: file is {row['bytes']} bytes. Request mediated file text with `@work-leaf read {path}` if this file is still needed.\n")
        else:
            digest(row["previous_digest"], row["previous_bytes"])
            reader.expect(f"previous digest: {row['previous_digest']}\nstatus: {cls} since this agent's last snapshot\n")
            if cls == "unchanged":
                require(row["digest"] == row["previous_digest"] and row["bytes"] == row["previous_bytes"], "unchanged-metadata")
        disposition = row["diff_disposition"]
        if cls != "changed":
            require(disposition == "not-applicable" and row["diff_bytes"] is None and
                    row["baseline_body_start"] is None and row["baseline_body_end"] is None, "ineligible-diff-fields")
        elif disposition in ("available", "empty"):
            size = integer(row["diff_bytes"])
            require((disposition == "available" and 1 <= size <= 49152) or (disposition == "empty" and size == 0), "diff-threshold")
            integer(row["baseline_body_start"]); integer(row["baseline_body_end"])
            require(row["baseline_body_start"] == reader.cursor and row["baseline_body_end"] == reader.cursor + size, "diff-body-range")
            begin = reader.cursor; diff = reader.body(size)
            if not diff.endswith(b"\n"): reader.expect(b"\n")
            if disposition == "available":
                require(2 + eligible_count < len(parts), "missing-body-component")
                part = parts[2 + eligible_count]
                require(part["snapshot_index"] == index, "snapshot-component-link")
                body = byte_range(candidate, part["body_start"], part["body_end"])
                proof["current_body"] = body_proof(body, row)
                replacement = b"current full text:\n" + body
                body_range = (len(b"current full text:\n"), len(replacement))
                if not body.endswith(b"\n"): replacement += b"\n"
                replacements.append(("current-full-text", begin, reader.cursor, replacement, body_range, index))
                eligible_count += 1
        elif disposition == "omitted":
            require(integer(row["diff_bytes"]) > 49152, "diff-threshold")
            reader.expect(f"diff omitted: compact refresh would be {row['diff_bytes']} bytes. Request narrower related context or continue from the previous snapshot if this file is still needed.\n")
            require(row["baseline_body_start"] is None and row["baseline_body_end"] is None, "omitted-body-range")
        elif disposition == "unavailable":
            require(row["diff_bytes"] is None and row["baseline_body_start"] is None and row["baseline_body_end"] is None, "unavailable-body-range")
            reader.expect("diff unavailable. Request narrower related context or continue from the previous snapshot if this file is still needed.\n")
        else:
            raise Invalid("invalid-disposition")
        require(row["baseline_section_end"] == reader.cursor, "section-end")
        proofs.append(proof)
    if meta["failures"]:
        reader.expect("\nUnavailable file text\n")
        for failure in meta["failures"]:
            keys(failure, {"path", "diagnostic"}); utf8(failure["path"]); utf8(failure["diagnostic"])
            reader.expect(f"- {failure['path']}: {failure['diagnostic']}\n")
    require(reader.cursor == len(original), "baseline-suffix")
    eligible = eligible_count > 0
    require(event["eligible"] is eligible and event["selected_candidate"] == ("candidate" if eligible else "baseline"), "selection")
    require(event["changed"] is (original != candidate), "changed-metadata")
    if not eligible:
        require(not parts and candidate == original, "ineligible-identity")
    else:
        require(len(parts) == len(replacements), "component-count")
        old_cursor = 0; new_cursor = 0
        for part, (name, begin, end, replacement, body_range, index) in zip(parts, replacements):
            unchanged = original[old_cursor:begin]
            require(candidate[new_cursor:new_cursor + len(unchanged)] == unchanged, "unowned-bytes")
            new_cursor += len(unchanged)
            expected = dict(id=name, baseline_start=begin, baseline_end=end, candidate_start=new_cursor,
                            candidate_end=new_cursor + len(replacement), snapshot_index=index,
                            body_start=None if body_range is None else new_cursor + body_range[0],
                            body_end=None if body_range is None else new_cursor + body_range[1])
            require(part == expected, "component-ownership")
            require(candidate[new_cursor:new_cursor + len(replacement)] == replacement, "replacement-bytes")
            new_cursor += len(replacement); old_cursor = end
        require(candidate[new_cursor:] == original[old_cursor:], "unowned-suffix")
    return candidate, eligible, proofs


def identity(event, original):
    forwarded = utf8(event["forwarded_prompt"])
    require(integer(event["forwarded_bytes"]) == len(forwarded), "forwarded-length")
    require(integer(event["byte_delta"]) == 0 and event["changed"] is False and original == forwarded, "identity-bytes")
    site = event["site"]
    spans = event["spans"]
    require(type(spans) is list, "invalid-spans")
    if site == "patch-applied":
        suffix = utf8(ACK_BEFORE + ACK + ACK_BETWEEN + REMAINING)
        require(original.endswith(suffix), "ack-suffix")
        prefix = original[:len(original) - len(suffix)]
        require(prefix.startswith(b"work-leaf patch applied\nfiles: ") and prefix.endswith(b"\n") and
                len(prefix) > len(b"work-leaf patch applied\nfiles: \n"), "ack-header")
        start = len(prefix) + len(utf8(ACK_BEFORE))
        expected = [("patch-applied-validation", start, ACK),
                    ("patch-applied-remaining-work", start + len(utf8(ACK + ACK_BETWEEN)), REMAINING)]
    elif site == "command-result":
        require(len(spans) == 1 and type(spans[0]) is dict, "command-span-count")
        span = spans[0]; keys(span, SPAN)
        byte_range(original, span["cue_start"], span["cue_end"])
        prefix = original[:span["cue_start"]]
        # Commands/paths may contain newlines. This checks the renderer language,
        # not uniquely parsed command/status fields or independently owned output.
        header = b"work-leaf command result\ncommand: "
        require(prefix.startswith(header), "command-header")
        # Search fixed separator tokens once, without ambiguous nested wildcards.
        require(any(match.start() > len(header) and match.end() < len(prefix)
                    for match in re.finditer(rb"\nstatus: (?:-?[0-9]+|terminated)\nlocked paths: ", prefix)), "command-header")
        tail = original[span["cue_end"]:]
        require(tail.startswith(b"\nstdout:\n") and tail.endswith(b"\n") and b"\nstderr:\n" in tail, "command-output-framing")
        expected = [("command-result-guidance", span["cue_start"], GUIDANCE)]
    else:
        raise Invalid("invalid-identity-site")
    require(len(spans) == len(expected), "span-count")
    for span, (name, start, literal) in zip(spans, expected):
        keys(span, SPAN)
        body = byte_range(original, span["cue_start"], span["cue_end"])
        require(span["id"] == name and span["cue_start"] == start and body == utf8(literal), "span-ownership")
        require(span["original"] == literal and span["replacement"] == literal and span["changed"] is False and
                integer(span["byte_delta"]) == 0, "span-literal")
    return forwarded, False, []


def validate_event(event, expected_run_id):
    """Return fixed errors and safe body proofs; never infer delivery from trace."""
    result = {"errors": [], "eligible": None, "snapshots": []}
    try:
        require(type(event) is dict, "invalid-event")
        if event.get("event") == "automatic-refresh":
            keys(event, AUTO); function = automatic
        elif event.get("event") == "prompt":
            keys(event, IDENTITY); function = identity
        else:
            raise Invalid("unsupported-event")
        original = common(event, expected_run_id)
        selected, eligible, proofs = function(event, original)
        result.update(eligible=eligible, snapshots=proofs, original_sha256=hashlib.sha256(original).hexdigest(),
                      selected_sha256=hashlib.sha256(selected).hexdigest(), original_bytes=len(original), selected_bytes=len(selected))
    except Invalid as exc:
        result["errors"].append(str(exc))
    return result


def census_trace(rows, expected_run_id, physical_lines=None):
    """Retain every noninitial-activation row, including malformed/foreign rows."""
    result = {"errors": [], "events": [], "complete_delivery_proven": False}
    if type(rows) is not list:
        result["errors"].append("invalid-rows")
        return result
    valid_lines = (physical_lines is None or (type(physical_lines) is list and len(physical_lines) == len(rows)
                   and all(type(n) is int and n > 0 for n in physical_lines)
                   and all(a < b for a, b in zip(physical_lines, physical_lines[1:]))))
    if not valid_lines:
        result["errors"].append("invalid-physical-lines")
    lines = list(range(1, len(rows) + 1)) if physical_lines is None else physical_lines
    first_activation = bool(rows) and type(rows[0]) is dict and rows[0].get("event") == "activation"
    process = None
    try:
        require(first_activation, "missing-activation")
        activation = rows[0]; keys(activation, ACTIVATION)
        require(activation["schema"] == SCHEMA and activation["condition"] == CONDITION, "foreign-activation")
        require(type(expected_run_id) is str and bool(expected_run_id) and activation["run_id"] == expected_run_id, "foreign-activation")
        process = integer(activation["process_id"], 1)
    except Invalid as exc:
        result["errors"].append(str(exc))
    for index in range(1 if first_activation else 0, len(rows)):
        row = rows[index]
        record = validate_event(row, expected_run_id)
        record["row_index"] = index
        record["trace_line"] = lines[index] if valid_lines else None
        expected_sequence = index if first_activation else index + 1
        if type(row) is not dict or type(row.get("sequence")) is not int or row["sequence"] != expected_sequence:
            record["errors"].append("sequence-gap")
        if process is None or type(row) is not dict or type(row.get("process_id")) is not int or row["process_id"] != process:
            record["errors"].append("process-mismatch")
        if record["errors"]:
            result["errors"].append("invalid-event-row")
        result["events"].append(record)
    return result
