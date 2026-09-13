#!/usr/bin/env python3
"""Private standalone author custody; no Work Leaf runtime or provider SDK imports.

Only an invocation-owned final message grants operations. Commands have repository
cwd and monitored source custody, NOT an OS write sandbox. All artifacts are local.
"""
import argparse
from collections import deque
import hashlib
import json
import os
from pathlib import Path
import shlex
import signal
import stat
import subprocess
import sys
import time

MAX_FINAL_BYTES = 4 * 1024 * 1024
COMMAND_SECONDS = 300
OUTPUT_CHARS = 12000


class Rejected(Exception):
    """A proposal grants no valid action; the author may correct it."""


class Fatal(Exception):
    """Retained custody/transport failure; no automatic retry or success ACK."""


def save_json(path, value):
    with Path(path).open("x", encoding="utf-8") as out:
        json.dump(value, out, ensure_ascii=False, indent=2)
        out.write("\n")


def digest(data):
    return hashlib.sha256(data).hexdigest()


def git(repo, *args):
    env = dict(os.environ, GIT_OPTIONAL_LOCKS="0")
    result = subprocess.run(["git", "-C", str(repo), *args], stdout=subprocess.PIPE,
                            stderr=subprocess.PIPE, env=env, check=False)
    if result.returncode:
        raise Fatal(f"git {args[0]} failed ({result.returncode}): {result.stderr.decode(errors='replace')}")
    return result.stdout


def relative_path(repo, value, allow_root=False):
    if not value or "\x00" in value:
        raise Rejected("empty or NUL path")
    path = Path(value)
    if ".." in path.parts or ".git" in path.parts:
        raise Rejected("parent or Git administrative path")
    if path.is_absolute():
        try:
            path = path.relative_to(repo)
        except ValueError as exc:
            raise Rejected("path outside repository") from exc
    if str(path) == "." and not allow_root:
        raise Rejected("file path required")
    current = repo
    for part in path.parts:
        current /= part
        if current.is_symlink():
            raise Rejected("symlink path")
    return path.as_posix()


def parse_operations(text):
    if len(text.encode("utf-8")) > MAX_FINAL_BYTES:
        raise Rejected("final/proposal exceeds 4 MiB")
    lines = text.replace("\r\n", "\n").split("\n")
    result, index = [], 0
    while index < len(lines):
        line = lines[index]
        index += 1
        if line.strip(" \t") == "":
            continue
        if line.startswith("@standalone edit ") and line[17:].strip():
            reason = line[len("@standalone edit "):]
            body = []
            while index < len(lines) and lines[index] != "@standalone end":
                body.append(lines[index])
                index += 1
            if index == len(lines):
                raise Rejected("edit lacks exact end marker")
            index += 1
            result.append(("edit", reason, "\n".join(body)))
        elif line.startswith("@standalone run "):
            request = line[len("@standalone run "):]
            if request.startswith("-- "):
                paths, separator, command = "", " -- ", request[3:]
            else:
                paths, separator, command = request.partition(" -- ")
            try:
                paths = shlex.split(paths)
            except ValueError as exc:
                raise Rejected("malformed declared paths") from exc
            if not separator or not command.strip():
                raise Rejected("run needs exact -- command separator; writes require declared paths")
            result.append(("run", paths, command))
        elif line.startswith("@standalone discard ") and line[len("@standalone discard "):].strip():
            result.append(("discard", line[len("@standalone discard "):]))
        elif line == "@standalone done":
            result.append(("done",))
        else:
            raise Rejected("unknown, mixed, quoted or malformed outer operation")
    if not result or any(op[0] == "done" for op in result[:-1]):
        raise Rejected("DONE must be the sole terminal operation")
    return result


def file_state(path):
    try:
        info = path.lstat()
    except FileNotFoundError:
        return ("missing", b"", 0)
    if stat.S_ISLNK(info.st_mode):
        return ("symlink", os.readlink(path).encode("utf-8"), stat.S_IMODE(info.st_mode))
    if not stat.S_ISREG(info.st_mode):
        return ("special", b"", stat.S_IMODE(info.st_mode))
    return ("file", path.read_bytes(), stat.S_IMODE(info.st_mode))


def occurrences(lines, block, start):
    # KMP gives linear line-comparison work per search. Repeated hunk searches
    # and list rewrites can still be O(hunks * file size); not a global bound.
    if not block:
        return []
    prefix, matched = [0] * len(block), 0
    for index in range(1, len(block)):
        while matched and not block[index] == block[matched]:
            matched = prefix[matched - 1]
        if block[index] == block[matched]:
            matched += 1
        prefix[index] = matched
    result, matched = [], 0
    for index in range(start, len(lines)):
        while matched and not lines[index] == block[matched]:
            matched = prefix[matched - 1]
        if lines[index] == block[matched]:
            matched += 1
        if matched == len(block):
            result.append(index - len(block) + 1)
            matched = prefix[matched - 1]
    return result


def update_text(before, body):
    try:
        text = before.decode("utf-8")
    except UnicodeDecodeError as exc:
        raise Rejected("update requires UTF-8 source") from exc
    if "\r" in text.replace("\r\n", ""):
        raise Rejected("bare CR source is outside the text contract")
    newline = "\r\n" if "\r\n" in text else "\n"
    if newline == "\r\n" and "\n" in text.replace("\r\n", ""):
        raise Rejected("mixed source newline convention")
    terminal = text.endswith("\n")
    lines = text.replace("\r\n", "\n").split("\n")
    if terminal:
        lines.pop()
    if text == "":
        lines = []
    cursor, index, hunks = 0, 0, 0
    while index < len(body):
        header = body[index]
        if header != "@@" and not header.startswith("@@ "):
            raise Rejected("update requires @@ or exact-context @@ header")
        index += 1
        context = header[3:] if header.startswith("@@ ") else ""
        if context:
            matches = occurrences(lines, [context], cursor)
            if len(matches) != 1:
                raise Rejected("context header absent or ambiguous")
            cursor = matches[0] + 1
        old, new, eof = [], [], False
        while index < len(body) and not body[index].startswith("@@"):
            line = body[index]
            index += 1
            if line == "*** End of File":
                eof = True
                if index != len(body) and not body[index].startswith("@@"):
                    raise Rejected("EOF must terminate its hunk")
                break
            if not line or line[0] not in " +-":
                raise Rejected("unsupported hunk line")
            if line[0] in " -":
                old.append(line[1:])
            if line[0] in " +":
                new.append(line[1:])
        if not old:
            raise Rejected("hunk requires nonempty exact old block")
        matches = occurrences(lines, old, cursor if context else 0)
        if eof:
            matches = [i for i in matches if i + len(old) == len(lines)]
        if len(matches) != 1:
            raise Rejected("old block absent or ambiguous in declared range")
        location = matches[0]
        lines[location:location + len(old)] = new
        cursor = location + len(new)
        hunks += 1
    if not hunks:
        raise Rejected("empty update")
    return (newline.join(lines) + (newline if terminal and lines else "")).encode("utf-8")


def plan_edit(repo, patch):
    if len(patch.encode("utf-8")) > MAX_FINAL_BYTES:
        raise Rejected("patch exceeds 4 MiB")
    lines = patch.replace("\r\n", "\n").split("\n")
    if not lines or lines[0] != "*** Begin Patch" or lines[-1] != "*** End Patch":
        raise Rejected("exact patch envelope required")
    changes, seen, index = {}, set(), 1
    while index < len(lines) - 1:
        header = lines[index]
        index += 1
        kind = next((kind for kind in ("Add", "Update", "Delete") if header.startswith(f"*** {kind} File: ")), None)
        if kind is None:
            raise Rejected("unsupported file header")
        name = relative_path(repo, header[len(f"*** {kind} File: "):])
        if name in seen:
            raise Rejected("duplicate normalized file identity")
        seen.add(name)
        before = file_state(repo / name)
        body = []
        while index < len(lines) - 1 and not any(lines[index].startswith(f"*** {k} File: ") for k in ("Add", "Update", "Delete")):
            body.append(lines[index])
            index += 1
        if kind == "Add":
            if before[0] != "missing" or not body or any(not line.startswith("+") for line in body):
                raise Rejected("add requires absent regular path and + lines")
            after = ("file", ("\n".join(line[1:] for line in body) + "\n").encode("utf-8"), 0o644)
        elif kind == "Delete":
            if before[0] != "file" or body:
                raise Rejected("delete requires existing regular file and no body")
            after = ("missing", b"", 0)
        else:
            if before[0] != "file":
                raise Rejected("update requires existing regular file")
            after = ("file", update_text(before[1], body), before[2])
        if before != after:
            changes[name] = (before, after)
    if not changes:
        raise Rejected("edit has no actual change" if seen else "empty patch")
    return changes


def snapshot(repo):
    index = git(repo, "ls-files", "--stage", "-z")
    names = []
    for row in index.split(b"\x00"):
        if row:
            metadata, name = row.split(b"\t", 1)
            if metadata.split()[-1] != b"0":
                raise Fatal("unmerged index")
            names.append(name.decode("utf-8"))
    files = {}
    for name in names:
        try:
            relative_path(repo, name)
        except Rejected:
            files[name] = ("unsafe-path", b"", 0)
        else:
            files[name] = file_state(repo / name)
    return {"head": git(repo, "rev-parse", "HEAD").decode().strip(), "index": index,
            "index_flags": git(repo, "ls-files", "-v", "-z"),
            "files": files,
            "untracked": git(repo, "ls-files", "--others", "--exclude-standard", "-z")}


def state_summary(state):
    return {"head": state["head"], "index_sha256": digest(state["index"]),
            "index_flags_sha256": digest(state["index_flags"]),
            "untracked": state["untracked"].decode("utf-8").split("\x00")[:-1],
            "files": {name: {"kind": value[0], "sha256": digest(value[1]), "mode": value[2]}
                      for name, value in state["files"].items()}}


def write_state(path, value):
    if value[0] == "missing":
        path.unlink()
    elif value[0] == "file":
        path.parent.mkdir(parents=True, exist_ok=True)
        path.write_bytes(value[1])
        path.chmod(value[2])
    else:
        raise Fatal("cannot publish nonregular source")


def execute_child(argv, cwd, stdin_path, stdout_path, stderr_path, seconds, env):
    if seconds <= 0:
        raise Fatal("workflow deadline exhausted before child")
    started = time.monotonic()
    child, cancelled_signal, timed_out = None, None, False

    def stop_group():
        if child is None:
            return
        try:
            os.killpg(child.pid, signal.SIGTERM)
        except ProcessLookupError:
            pass
        try:
            child.wait(timeout=1)
        except subprocess.TimeoutExpired:
            pass
        try:
            os.killpg(child.pid, signal.SIGKILL)
        except ProcessLookupError:
            pass
        child.wait(timeout=1)

    def cancelled(signum, _frame):
        nonlocal cancelled_signal
        if cancelled_signal is None:
            cancelled_signal = signum
            stop_group()

    previous = {kind: signal.getsignal(kind) for kind in (signal.SIGTERM, signal.SIGINT)}
    with Path(stdout_path).open("xb") as out, Path(stderr_path).open("xb") as err:
        incoming = Path(stdin_path).open("rb") if stdin_path else subprocess.DEVNULL
        for kind in previous:
            signal.signal(kind, cancelled)
        try:
            child = subprocess.Popen(argv, cwd=cwd, stdin=incoming, stdout=out, stderr=err,
                                     env=env, start_new_session=True)
            if cancelled_signal is not None:
                stop_group()
            try:
                child.wait(timeout=seconds)
            except subprocess.TimeoutExpired:
                timed_out = True
                stop_group()
        finally:
            for kind, handler in previous.items():
                signal.signal(kind, handler)
            if stdin_path:
                incoming.close()
    return {"argv": argv, "exit_code": child.returncode, "timed_out": timed_out, "cancelled_signal": cancelled_signal,
            "duration_seconds": time.monotonic() - started}


def inclusive_lines(text):
    pieces = text.split("\n")
    for piece in pieces[:-1]:
        yield piece + "\n"
    if pieces[-1]:
        yield pieces[-1]


def render_output(text, _path=None):
    if not text:
        return "<empty>\n"
    # Rust char::is_whitespace (Unicode White_Space), excluding Python's
    # additional U+001C..U+001F classification; preserve CR and LF bytes.
    whitespace = "\t\n\v\f\r \u0085\u00a0\u1680\u2000\u2001\u2002\u2003\u2004\u2005\u2006\u2007\u2008\u2009\u200a\u2028\u2029\u202f\u205f\u3000"
    parts, blanks = [], 0
    for line in inclusive_lines(text):
        if not line.strip(whitespace):
            blanks += 1
            if blanks <= 8:
                parts.append(line)
            continue
        if blanks > 8:
            parts.append(f"[standalone compacted {blanks - 8} whitespace-only output lines]\n")
        blanks = 0
        parts.append(line)
    if blanks > 8:
        parts.append(f"[standalone compacted {blanks - 8} whitespace-only output lines]\n")
    rendered, parts = "".join(parts), []
    for line in inclusive_lines(rendered):
        newline = line.endswith("\n")
        content = line[:-1] if newline else line
        if len(content) <= 4096:
            parts.append(line)
        else:
            parts.append(content[:1600] + f"\n[standalone compacted {len(content) - 3200} characters from one long output line]\n" + content[-1600:] + ("\n" if newline else ""))
    rendered = "".join(parts)
    if len(rendered) > OUTPUT_CHARS:
        rendered = rendered[:6000] + f"\n[standalone compacted {len(rendered) - 10000} characters from command output]\n" + rendered[-4000:]
    return rendered if rendered.endswith("\n") else rendered + "\n"


def public_texts(event):
    # Same text-key selection/order as the frozen Direct extractor. This is
    # public evidence only; never an authorization parser or native reasoning read.
    keys = {"text", "message", "content", "response", "reply", "delta", "summary"}
    stack, result = [(event, "")], []
    while stack:
        value, parent = stack.pop()
        if isinstance(value, dict):
            stack.extend((child, key) for key, child in reversed(list(value.items())))
        elif isinstance(value, list):
            stack.extend((child, parent) for child in reversed(value))
        elif isinstance(value, str) and parent in keys:
            result.append(value)
    return result


def owned_final(raw_path, final_path, expected_thread):
    final_path = Path(final_path)
    if final_path.is_symlink() or not final_path.is_file():
        raise Fatal("missing or nonregular fresh final output")
    with final_path.open("rb") as source:
        final_bytes = source.read(MAX_FINAL_BYTES + 1)
    if len(final_bytes) > MAX_FINAL_BYTES:
        raise Fatal("final output exceeds 4 MiB")
    try:
        final = final_bytes.decode("utf-8")
    except UnicodeDecodeError as exc:
        raise Fatal("final output is not UTF-8") from exc
    thread, starts, ends, active, seen, author, last = None, 0, 0, set(), set(), [], None
    with Path(raw_path).open(encoding="utf-8") as raw:
        for line in raw:
            if not line.strip():
                continue
            try:
                event = json.loads(line)
            except (ValueError, TypeError) as exc:
                raise Fatal("invalid native public JSONL") from exc
            if not isinstance(event, dict):
                raise Fatal("native public event must be an object")
            author.extend(public_texts(event))
            kind = event.get("type")
            if kind == "thread.started":
                if thread is not None or starts or ends or not isinstance(event.get("thread_id"), str):
                    raise Fatal("ambiguous thread ownership")
                thread = event["thread_id"]
            elif kind == "turn.started":
                if not thread or starts or ends:
                    raise Fatal("turn lacks preceding unique thread identity")
                starts += 1
            elif kind == "turn.completed":
                ends += 1
            elif kind in ("error", "turn.failed"):
                raise Fatal("native invocation failed")
            elif kind in ("item.started", "item.completed"):
                if starts != 1 or ends:
                    raise Fatal("item outside owned active turn")
                item = event.get("item", {})
                identity = item.get("id")
                if not isinstance(identity, str) or not identity:
                    raise Fatal("missing item identity")
                if kind == "item.started":
                    if identity in active or identity in seen:
                        raise Fatal("duplicate item start")
                    active.add(identity)
                else:
                    if identity in seen:
                        raise Fatal("duplicate completed item")
                    active.discard(identity)
                    seen.add(identity)
                    last = item
                    if item.get("type") == "agent_message":
                        if not isinstance(item.get("text"), str):
                            raise Fatal("invalid author text")
    if not thread or starts != 1 or ends != 1 or active or not last or last.get("type") != "agent_message":
        raise Fatal("incomplete final-item ownership")
    if expected_thread is not None and thread != expected_thread:
        raise Fatal("foreign resumed thread")
    if last.get("text") != final:
        raise Fatal("fresh final file differs from owned completed final item")
    return thread, final, author


class Host:
    def __init__(self, repo, artifact_dir, feature, stage, deadline, serialized_feedback=False):
        self.repo = Path(repo).resolve(strict=True)
        self.artifacts = Path(artifact_dir).absolute()
        self.artifacts.mkdir(parents=True, exist_ok=False)
        self.feature, self.stage, self.deadline = feature, stage, deadline
        self.serialized_feedback = serialized_feedback
        try:
            self.expected = snapshot(self.repo)
            if self.expected["untracked"] or git(self.repo, "status", "--porcelain", "--untracked-files=no"):
                raise Fatal("initial checkout must have clean index/tracked source and no nonignored untracked files")
            if any(value[0] != "file" for value in self.expected["files"].values()):
                raise Fatal("initial tracked sources must be regular files")
        except (Fatal, OSError, ValueError) as exc:
            save_json(self.artifacts / "initialization-error.json", {"repo": str(self.repo), "feature": feature, "stage": stage, "error": str(exc)})
            raise
        self.initial_head = self.expected["head"]
        self.accepted_commits, self.pending, self.invocations = [], {}, []
        self.queued_replies = deque()
        self.completed, self.counter, self.thread = False, 0, None
        self.events_path = self.artifacts / "events.jsonl"
        self.events_path.touch(exist_ok=False)
        self.evidence_path = self.artifacts / "evidence.txt"
        self.evidence_path.touch(exist_ok=False)

    @property
    def pending_changes(self):
        return sorted(self.pending)

    def event(self, kind, **fields):
        with self.events_path.open("a", encoding="utf-8") as out:
            out.write(json.dumps({"kind": kind, **fields}, ensure_ascii=False) + "\n")

    def evidence(self, label, text):
        with self.evidence_path.open("a", encoding="utf-8") as out:
            out.write(f"[{label}]\n{text}\n\n")

    def unchanged(self):
        current = snapshot(self.repo)
        if current != self.expected:
            self.event("unexpected_source_state", expected=state_summary(self.expected), actual=state_summary(current))
            raise Fatal("unexpected source/index/HEAD state; retained without restoration")
        return current

    def operation_dir(self):
        self.counter += 1
        path = self.artifacts / f"operation-{self.counter:04d}"
        path.mkdir()
        return path

    def apply(self, reason, patch):
        self.unchanged()
        changes = plan_edit(self.repo, patch)
        folder = self.operation_dir()
        (folder / "proposal.txt").write_text(patch, encoding="utf-8")
        before_head = self.expected["head"]
        for name, (before, after) in changes.items():
            relative_path(self.repo, name)
            if file_state(self.repo / name) != before:
                raise Fatal("source drift before publication")
            write_state(self.repo / name, after)
        git(self.repo, "add", "--", *changes)
        message = f"UPDATE {self.feature}: {reason}"
        git(self.repo, "commit", "-m", message, "--", *changes)
        after = snapshot(self.repo)
        commit = after["head"]
        if commit == before_head or git(self.repo, "rev-parse", f"{commit}^").decode().strip() != before_head:
            raise Fatal("accepted commit does not extend exact preceding HEAD")
        expected_files = dict(self.expected["files"])
        for name, (_, value) in changes.items():
            if value[0] == "missing":
                expected_files.pop(name)
            else:
                expected_files[name] = value
        if after["files"] != expected_files or after["untracked"] or git(self.repo, "status", "--porcelain", "--untracked-files=no"):
            raise Fatal("commit left unexplained source effects")
        for name in changes:
            self.pending.pop(name, None)
        self.accepted_commits.append(commit)
        self.expected = after
        receipt = {"before_head": before_head, "commit": commit, "paths": list(changes), "reason": reason}
        save_json(folder / "receipt.json", receipt)
        self.event("edit_accepted", **receipt)
        self.evidence("HOST EDIT", f"accepted provisional commit {commit}; changed paths: {', '.join(changes)}")
        return list(changes)

    def command(self, paths, command):
        before = self.unchanged()
        declared = [relative_path(self.repo, path, allow_root=True) for path in paths]
        folder = self.operation_dir()
        save_json(folder / "request.json", {"command": command, "declared_paths": declared})
        seconds = min(COMMAND_SECONDS, self.deadline - time.monotonic())
        env = dict(os.environ, WORK_LEAF_BENCH_CODEX_ACTIVE="1")
        receipt = execute_child(["/bin/sh", "-c", command], self.repo, None, folder / "stdout.txt", folder / "stderr.txt", seconds, env)
        after = snapshot(self.repo)
        receipt.update(command=command, declared_paths=declared, before=state_summary(before), after=state_summary(after))
        changed = {name: (old, after["files"].get(name, ("missing", b"", 0)))
                   for name, old in before["files"].items() if old != after["files"].get(name)}
        pending_diffs = {}
        errors = []
        if after["head"] != before["head"] or after["index"] != before["index"] or after["index_flags"] != before["index_flags"]:
            errors.append("command changed HEAD or index")
        if after["untracked"] != before["untracked"]:
            errors.append("command created nonignored untracked source")
        for number, (name, (old, new)) in enumerate(changed.items()):
            retained = folder / "pending" / str(number)
            retained.mkdir(parents=True)
            (retained / "before").write_bytes(old[1])
            (retained / "after").write_bytes(new[1])
            save_json(retained / "identity.json", {"path": name, "before_kind": old[0], "after_kind": new[0], "before_mode": old[2], "after_mode": new[2]})
            with (retained / "diff.txt").open("xb") as output:
                subprocess.run(["git", "diff", "--no-index", "--", str(retained / "before"), str(retained / "after")], stdout=output, stderr=subprocess.STDOUT, check=False)
            if after["index"] == before["index"] and after["head"] == before["head"] and new[0] in ("file", "missing"):
                ordinary = git(self.repo, "diff", "--no-ext-diff", "--no-textconv", "--", name)
                indexed = git(self.repo, "show", f":{name}")
                if ordinary and indexed == old[1]:
                    pending_diffs[name] = ordinary.decode("utf-8", errors="replace")
                else:
                    # Index flags/overlays can hide a real working-byte change.
                    # Never silently omit it or claim a different beforeimage.
                    pending_diffs[name] = f"Tracked path: {name}; exact retained before/after diff (index view unavailable or differs); before {old[0]} mode {old[2]:o}, after {new[0]} mode {new[2]:o}:\n" + (retained / "diff.txt").read_bytes().decode("utf-8", errors="replace")
                (retained / "model-diff.txt").write_text(pending_diffs[name], encoding="utf-8")
            if new[0] not in ("file", "missing"):
                errors.append(f"nonregular changed source: {name}")
            if not any(path == "." or name == path or name.startswith(path + "/") for path in declared):
                errors.append(f"source change outside declared paths: {name}")
            try:
                relative_path(self.repo, name)
            except Rejected:
                errors.append(f"symlink source path: {name}")
        receipt["custody_errors"] = errors
        receipt["pending_paths"] = list(changed)
        save_json(folder / "receipt.json", receipt)
        self.event("command_result", receipt=str(folder / "receipt.json"), status=receipt["exit_code"], timed_out=receipt["timed_out"], custody_errors=errors)
        self.evidence("HOST CHECK", f"command: {command}\nstatus: {receipt['exit_code']}; timed_out: {receipt['timed_out']}\nstdout: {folder / 'stdout.txt'}\nstderr: {folder / 'stderr.txt'}")
        if errors:
            raise Fatal("; ".join(errors) + "; actual state retained")
        for name, (old, new) in changed.items():
            write_state(self.repo / name, old)
            self.pending[name] = {"after": new, "artifact": str(folder / "pending"), "diff": pending_diffs[name]}
        self.unchanged()
        if receipt["cancelled_signal"] or receipt["timed_out"] or time.monotonic() >= self.deadline:
            raise Fatal("command cancelled or command/workflow deadline reached; no retry")
        output = []
        for stream in ("stdout", "stderr"):
            path = folder / f"{stream}.txt"
            output.append(f"{stream}:\n" + render_output(path.read_bytes().decode("utf-8", errors="replace")))
        pending = "\n" + self.pending_feedback() if self.pending else ""
        return f"Host command completed.\ncommand: {command}\nstatus: {receipt['exit_code']}\n" + "\n".join(output) + pending + "\nUse another requested command or an ordinary edit for concrete remaining work. When the required work and focused validation are complete, send @standalone done."

    def pending_feedback(self):
        diff = "\n".join(self.pending[name]["diff"] for name in self.pending_changes)
        return (f"Standalone command captured tracked file changes\nfiles: {', '.join(self.pending_changes)}\n"
                "Review cannot start until command-produced tracked changes are saved in a provisional commit or explicitly discarded.\n"
                "These changes are pending and restored in the checkout. If required, submit an equivalent structured `@standalone edit <reason>` using the full diff below; the host applies and commits it through the ordinary edit path.\n"
                "If not required, emit `@standalone discard <reason>` to clear this pending output without writing or committing it.\n"
                "In your next response, emit one edit block or one discard directive for these files, then stop; do not repeat the block or include DONE until acceptance or explicit discard.\n"
                "Current tracked diff:\n" + diff + ("" if diff.endswith("\n") else "\n"))

    def consume(self, text, feedback=None):
        if self.completed:
            raise Fatal("completed stage cannot consume another reply")
        self.unchanged()
        if time.monotonic() >= self.deadline:
            raise Fatal("workflow deadline exhausted")
        accepted, prompts = [], []

        def deliver(prompt):
            prompts.append(prompt)
            if feedback is not None:
                reply = feedback(prompt)
                self.queued_replies.append({"text": reply, "invocation": len(self.invocations) or None})

        try:
            operations = parse_operations(text)
            if self.serialized_feedback and len(operations) != 1:
                raise Rejected("serialized feedback requires exactly one operative directive per complete final reply. No operation from this reply was executed; one edit may contain multiple files and hunks. Wait for its actual result before choosing the next operation")
        except Rejected as exc:
            self.event("proposal_rejected", reason=str(exc), accepted_paths=[])
            deliver(f"Host proposal rejected: {exc}. Inspect the actual source and submit a corrected complete operation.")
            return "\n\n".join(prompts)
        for operation in operations:
            try:
                kind = operation[0]
                if kind == "edit":
                    accepted.extend(self.apply(operation[1], operation[2]))
                elif kind == "run":
                    deliver(self.command(operation[1], operation[2]))
                elif kind == "discard":
                    self.evidence("HOST DISCARD", f"reason: {operation[1]}; discarded pending paths: {', '.join(self.pending_changes)}")
                    self.event("pending_discarded", reason=operation[1], paths=self.pending_changes)
                    self.pending.clear()
                else:
                    if prompts:
                        self.event("done_deferred_to_generated_reply", feedback_count=len(prompts))
                        break
                    if self.pending:
                        deliver("DONE rejected: tracked command output remains pending.\n" + self.pending_feedback())
                        break
                    self.completed = True
                    self.event("done", head=self.expected["head"])
                    break
            except Rejected as exc:
                self.event("proposal_rejected", reason=str(exc), accepted_paths=accepted)
                deliver(f"Host proposal rejected: {exc}. No success is claimed for the rejected operation. Previously accepted paths in this reply: {', '.join(accepted) or 'none'}. Inspect the actual source and submit a corrected complete operation.")
        if accepted and not self.completed:
            deliver(f"Host accepted files: {', '.join(dict.fromkeys(accepted))}. Each accepted edit has its provisional commit already saved. Do not resend or restate accepted patches. Run at most one relevant focused validation step for this accepted work; after it passes, send @standalone done. Send another edit for a concrete issue in your own patch. The complete requested work remains required; concrete unresolved requirements may still be addressed.")
        if not prompts and not self.completed:
            deliver("Pending output explicitly discarded. Continue concrete required work or send @standalone done when complete.")
        # This joined string is only a provider-free unit-test view. Production
        # invokes feedback separately at EACH deliver call, never as a bundle.
        return None if self.completed else "\n\n".join(prompts)


def invoke(host, args, prompt):
    host.unchanged()
    number = len(host.invocations) + 1
    folder = host.artifacts / f"invocation-{number:04d}"
    folder.mkdir()
    prompt_path, final_path = folder / "prompt.txt", folder / "final.txt"
    prompt_path.write_text(prompt, encoding="utf-8")
    argv = [str(args.codex_bin), "--cd", str(host.repo), "--sandbox", "read-only", "--ask-for-approval", "never"]
    if args.model:
        argv += ["--model", args.model]
    argv += ["exec", "--color", "never"]
    if host.thread:
        argv += ["resume", "--json", "-o", str(final_path), host.thread, "-"]
    else:
        argv += ["--json", "-o", str(final_path), "-"]
    env = dict(os.environ)
    if env.get("WORK_LEAF_BENCH_CODEX_ACTIVE") == "1":
        raise Fatal("host cannot be launched from an active measured Codex child")
    role = f"{env.get('WORK_LEAF_OBSERVER_ROLE', args.stage)}-host-{number:04d}"
    env["WORK_LEAF_OBSERVER_ROLE"] = role
    request = {"role": role, "artifact_dir": str(folder), "argv": argv, "expected_thread": host.thread, "prompt_sha256": digest(prompt.encode("utf-8"))}
    save_json(folder / "request.json", request)
    host.invocations.append(request)
    result = execute_child(argv, host.repo, prompt_path, folder / "stdout.jsonl", folder / "stderr.txt", args.deadline - time.monotonic(), env)
    save_json(folder / "exit.json", result)
    host.unchanged()
    if result["exit_code"] != 0 or result["timed_out"] or result["cancelled_signal"]:
        raise Fatal("native child failed or timed out; retained without retry")
    host.thread, final, author = owned_final(folder / "stdout.jsonl", final_path, host.thread)
    for text in author:
        host.evidence("AUTHOR PUBLIC TEXT", text)
    host.event("owned_final", invocation=number, thread_id=host.thread, final_sha256=digest(final.encode("utf-8")))
    return final


def run_stage(args):
    host, errors = None, []
    try:
        host = Host(args.repo, args.artifact_dir, args.feature, args.stage, args.deadline,
                    serialized_feedback=getattr(args, "serialized_feedback", False))
        prompt = Path(args.prompt_file).read_text(encoding="utf-8")
        host.thread = args.resume_thread
        first = invoke(host, args, prompt)
        host.queued_replies.append({"text": first, "invocation": 1})
        while host.queued_replies and not host.completed:
            current = host.queued_replies.popleft()
            host.event("reply_processing", invocation=current["invocation"], final_sha256=digest(current["text"].encode("utf-8")))
            host.consume(current["text"], lambda prompt: invoke(host, args, prompt))
        if host.queued_replies:
            host.event("queued_replies_retained_unprocessed", replies=[{"invocation": row["invocation"], "final_sha256": digest(row["text"].encode("utf-8"))} for row in host.queued_replies])
    except (Fatal, Rejected, OSError, ValueError) as exc:
        errors.append(str(exc))
        print(f"standalone host failed: {exc}", file=sys.stderr)
    if host is None:
        return 1
    try:
        current = snapshot(host.repo)
        clean = current == host.expected and not current["untracked"] and not git(host.repo, "status", "--porcelain", "--untracked-files=no")
        final_head = current["head"]
    except (Fatal, OSError) as exc:
        errors.append(str(exc))
        clean, final_head = False, None
    result = {"schema_version": "standalone-host-result-v1", "repo": str(host.repo), "feature": args.feature, "stage": args.stage,
              "serialized_feedback": host.serialized_feedback,
              "thread_id": host.thread, "initial_head": host.initial_head, "final_head": final_head,
              "accepted_commits": host.accepted_commits, "completed": host.completed,
              "pending_changes": host.pending_changes, "clean": clean, "invocations": host.invocations,
              "events_path": str(host.events_path), "evidence_file": "evidence.txt", "errors": errors,
              "unprocessed_replies": [{"invocation": row["invocation"], "final_sha256": digest(row["text"].encode("utf-8"))} for row in host.queued_replies]}
    save_json(host.artifacts / "result.json", result)
    return 0 if host.completed and clean and not host.pending and not errors else 1


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    for name in ("repo", "feature", "stage", "prompt-file", "artifact-dir", "codex-bin"):
        parser.add_argument("--" + name, required=True)
    parser.add_argument("--deadline", required=True, type=float)
    parser.add_argument("--resume-thread")
    parser.add_argument("--model")
    parser.add_argument("--serialized-feedback", action="store_true",
                        help="require one complete operative directive per owned final, with actual feedback before the next operation")
    return run_stage(parser.parse_args())


if __name__ == "__main__":
    raise SystemExit(main())
