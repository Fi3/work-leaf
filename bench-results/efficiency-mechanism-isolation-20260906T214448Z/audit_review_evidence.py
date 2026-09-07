"""Private opaque-review evidence primitives; no provider calls or token prices.

Snapshot validation establishes exact held/archive bytes and owned replacement
bounds. It does not independently establish the renderer's receipt semantics or
delivery/native ownership; those require the separately source-bound census.
Read classification is deliberately narrow: unsupported commands remain unresolved.
Work is linear in prompt/payload/output bytes, apart from shell tokenization.
"""
import hashlib
from collections import defaultdict, deque
from pathlib import PurePosixPath
import re
import shlex


CONDITIONS = ("review-evidence-native", "review-evidence-inline")


def require(predicate, message):
    if not predicate:
        raise ValueError(message)


def typed_equal(left, right):
    """JSON evidence equality retains number/boolean and integer/float types."""
    if type(left) is not type(right):
        return False
    if isinstance(left, dict):
        return left.keys() == right.keys() and all(typed_equal(value, right[key]) for key, value in left.items())
    if isinstance(left, list):
        return len(left) == len(right) and all(typed_equal(a, b) for a, b in zip(left, right))
    return left == right


def fnv64(data):
    value = 0xcbf29ce484222325
    for byte in data:
        value = ((value ^ byte) * 0x100000001b3) & 0xffffffffffffffff
    return f"fnv64:{value:016x}"


def byte_slice(text, start, end):
    data = text.encode("utf-8")
    require(type(start) is int and type(end) is int and 0 <= start <= end <= len(data),
            "invalid owned UTF-8 byte range")
    try:
        data[:start].decode("utf-8")
        data[start:end].decode("utf-8")
    except UnicodeDecodeError as error:
        raise ValueError("owned range cuts a UTF-8 character") from error
    return data[start:end]


def validate_snapshot(event, payload, manifest_entry, run_id, condition, archive_root):
    """No content-marker parsing: arbitrary recorded context remains opaque."""
    require(condition in CONDITIONS and event.get("condition") == condition,
            "wrong review condition")
    require(event.get("event") == "review-context"
            and event.get("schema") == "work-leaf-bench-experiment-v5"
            and event.get("site") == "review-source-context", "wrong review boundary")
    require(isinstance(run_id, str) and run_id and event.get("run_id") == run_id,
            "wrong review run")
    archive = event.get("archive")
    require(isinstance(archive, dict) and typed_equal(archive, manifest_entry),
            "typed archive manifest differs from issued identity")
    require(archive.get("kind") == "review-source-context", "not opaque review context")
    for key in ("run_id", "source_agent_id", "reviewer_id", "target_commit"):
        require(isinstance(event.get(key), str) and event[key]
                and archive.get(key) == event[key], "archive ownership differs: " + key)
    require(type(archive.get("sequence")) is int and archive["sequence"] > 0,
            "invalid independent archive sequence")
    root = PurePosixPath(archive_root)
    path = PurePosixPath(archive.get("path", ""))
    require(root.is_absolute() and path.is_absolute() and path.parent == root
            and ".." not in root.parts and ".." not in path.parts,
            "archive is outside its exact declared root")
    require(type(payload) is bytes and type(archive.get("bytes")) is int
            and archive["bytes"] == len(payload) and archive.get("digest") == fnv64(payload),
            "archive byte length/runtime checksum differs")
    original = event.get("original_prompt")
    candidate = event.get("candidate_prompt")
    require(isinstance(original, str) and isinstance(candidate, str), "missing full candidates")
    start, end = event.get("context_start"), event.get("context_end")
    replacement_start, replacement_end = event.get("candidate_start"), event.get("candidate_end")
    require(byte_slice(original, start, end) == payload, "held context/archive bytes differ")
    byte_slice(candidate, replacement_start, replacement_end)
    old, new = original.encode(), candidate.encode()
    require(old[:start] == new[:replacement_start] and old[end:] == new[replacement_end:],
            "candidate changes unowned prompt bytes")
    selected = "candidate" if condition == "review-evidence-native" else "baseline"
    require(event.get("selected_candidate") == selected
            and event.get("forwarded_prompt") == (candidate if selected == "candidate" else original),
            "wrong selected review delivery")
    return {"path": str(path), "sequence": archive["sequence"], "bytes": len(payload),
            "sha256": hashlib.sha256(payload).hexdigest(), "runtime_digest": archive["digest"],
            "held_context_exact": True, "unowned_bytes_exact": True}


def literal_shell_command(command):
    """Recognize only literal shell words; unsupported evaluation is unresolved."""
    require(isinstance(command, str), "command must be text")
    quote = None
    escaped = False
    for char in command:
        if escaped:
            require(char not in "\r\n", "shell line continuation is unsupported")
            escaped = False
        elif quote == "'":
            if char == "'":
                quote = None
        elif char == "\\":
            escaped = True
        elif quote == '"':
            require(char not in "$`", "shell expansion is unsupported")
            if char == '"':
                quote = None
        elif char in "'\"":
            quote = char
        else:
            require(char not in ";&|<>()$`*?[]{}~#!\r\n", "shell syntax is not a literal read")
    require(quote is None and not escaped, "incomplete shell word")


def command_words(command):
    literal_shell_command(command)
    words = shlex.split(command, posix=True)
    # One explicit shell wrapper is supported; no shell evaluation is performed.
    shells = {name for shell in ("bash", "sh") for name in (shell, "/bin/" + shell, "/usr/bin/" + shell)}
    if len(words) == 3 and words[0] in shells and words[1] in {"-c", "-lc"}:
        literal_shell_command(words[2])
        words = shlex.split(words[2], posix=True)
    return words


def lf_lines(payload):
    pieces = payload.split(b"\n")
    return [part + b"\n" for part in pieces[:-1]] + ([pieces[-1]] if pieces[-1] else [])


def selected_bytes(words, path, payload):
    if not words:
        return None
    executable = PurePosixPath(words[0]).name
    if words[0] not in {executable, "/bin/" + executable, "/usr/bin/" + executable}:
        return None
    args = words[1:]
    if executable == "cat" and args in ([path], ["--", path]):
        return payload
    if executable == "sed" and len(args) in {3, 4} and args[0] == "-n":
        match = re.fullmatch(r"([1-9][0-9]*)(?:,([1-9][0-9]*))?p", args[1])
        if match and args[2:] in ([path], ["--", path]):
            first = int(match[1]); last = int(match[2] or match[1])
            if last >= first:
                return b"".join(lf_lines(payload)[first - 1:last])
    if executable == "head" and len(args) in {3, 4} and args[0] == "-n":
        if re.fullmatch(r"[1-9][0-9]*", args[1]) and args[2:] in ([path], ["--", path]):
            return b"".join(lf_lines(payload)[:int(args[1])])
    return None


def output_body(output):
    if not isinstance(output, str):
        return None
    header, separator, body = output.partition("\nOutput:\n")
    # Original token count is a normal metadata header, not truncation evidence.
    if not separator or not re.fullmatch(
        r"Chunk ID: [^\n]+\nWall time: [0-9.]+ seconds\nProcess exited with code 0"
        r"(?:\nOriginal token count: [0-9]+)?", header
    ):
        return None
    return body.encode("utf-8")


def classify_read(command, output, path, payload):
    """Caller must independently join issued path, native call/output and reviewer."""
    result = {"status": "unresolved", "reason": "unsupported command or non-exact successful output"}
    try:
        expected = selected_bytes(command_words(command), path, payload)
    except (TypeError, ValueError):
        return result
    body = output_body(output)
    if expected is None or (not expected and payload) or body != expected:
        return result
    return {"status": "complete" if expected == payload else "verified_partial",
            "bytes": len(body), "sha256": hashlib.sha256(body).hexdigest(),
            "reason": "supported read command and exact returned archive text"}


def text_digest(text):
    require(isinstance(text, str), "input text must be a string")
    return hashlib.sha256(text.encode()).hexdigest()


def rpc_key(frame):
    value = frame.get("id")
    require(type(value) in {str, int}, "RPC identity must retain string/integer type")
    return type(value).__name__, value


def one_text(content):
    require(isinstance(content, list) and len(content) == 1
            and isinstance(content[0], dict) and content[0].get("type") in {"text", "input_text"}
            and isinstance(content[0].get("text"), str), "unsupported input content")
    return content[0]["text"]


def native_user_index(sources):
    users = {}; threads = set()
    for source in sources:
        thread = source["thread_id"]
        require(isinstance(thread, str) and thread and thread not in threads, "duplicate native thread")
        threads.add(thread); metadata = []; seen = {}
        for line, row in source["rows"]:
            payload = row.get("payload", {})
            if row.get("type") == "session_meta":
                metadata.append(payload.get("id"))
            if row.get("type") != "response_item" or payload.get("role") != "user":
                continue
            require(payload.get("type") == "message", "unsupported native user item")
            passthrough = payload.get("internal_chat_message_metadata_passthrough")
            if passthrough is None:
                passthrough = {}
            require(isinstance(passthrough, dict), "malformed native passthrough metadata")
            explicit, nested = payload.get("turn_id"), passthrough.get("turn_id")
            for identity in (explicit, nested):
                require(identity is None or (isinstance(identity, str) and identity), "malformed present native turn identity")
            require(explicit is None or nested is None or explicit == nested, "contradictory native turn ownership")
            turn = explicit if explicit is not None else nested
            # Environment/AGENTS inputs are retained context, not the raw user request.
            if passthrough.get("content_item_kinds") not in (None, ["user.text"]):
                continue
            require(isinstance(turn, str) and turn, "native user item lacks explicit turn identity")
            item_id = payload.get("id")
            require(isinstance(item_id, str) and item_id, "native user item lacks identity")
            text = one_text(payload.get("content")); key = thread, turn, text_digest(text)
            identity = (turn, text)
            require(item_id not in seen or seen[item_id] == identity, "native item identity conflicts")
            if item_id in seen:
                continue
            seen[item_id] = identity
            require(key not in users, "ambiguous native user input")
            users[key] = {"text": text, "native_item_id": item_id, "native_line": line,
                          "native_source": source["source"]}
        require(metadata == [thread], "native session metadata differs from declared thread")
    return users, threads


def join_review_inputs(trace, captures, native_sources):
    """Exact accepted/raw/public/native identity, including usage-less threads.

    The caller owns activation/source/terminal/archive verification. This routine
    never treats public item IDs as native item IDs or infers a user turn from
    the preceding native context. Dictionaries/queues keep the join linear.
    """
    policies = {}; policy_counts = defaultdict(int); reviews = defaultdict(deque)
    previous = 0
    for event in trace:
        if event.get("event") == "activation":
            continue
        sequence = event.get("sequence")
        require(type(sequence) is int and sequence > previous, "unordered trace sequence")
        previous = sequence
        if event.get("event") == "review-context":
            reviews[event["reviewer_id"]].append(event)
        elif event.get("site") == "policy-injection":
            text = event["forwarded_prompt"]
            require(text == event["original_prompt"], "nonfactor policy bytes differ")
            key = text_digest(text); owner = event["agent_id"]
            require(key not in policies or policies[key] == (owner, text), "ambiguous policy ownership")
            policies[key] = owner, text; policy_counts[key] += 1
    native, native_threads = native_user_index(native_sources)
    owners = {}; joined = []; accepted = []; captured_turns = set(); seen_public = set()
    for capture in captures:
        require(typed_equal(capture["clients"], capture["forwarded"]), "original/forwarded frames differ")
        replies = {}; public = {}
        for line, frame in enumerate(capture["servers"], 1):
            if "method" not in frame and "id" in frame:
                key = rpc_key(frame)
                require(key not in replies, "duplicate RPC reply")
                replies[key] = frame
            if frame.get("method") != "item/completed":
                continue
            params = frame["params"]; item = params["item"]
            if item.get("type") != "userMessage":
                continue
            key = params.get("threadId"), params.get("turnId")
            item_id = item.get("id")
            require(all(isinstance(v, str) and v for v in (*key, item_id)), "missing public user identity")
            require(key not in public and (key[0], item_id) not in seen_public, "duplicate public input")
            seen_public.add((key[0], item_id))
            public[key] = one_text(item.get("content")), item_id, line
        seen_requests = set()
        for line, frame in enumerate(capture["clients"], 1):
            if frame.get("method") != "turn/start":
                continue
            rpc = rpc_key(frame)
            require(rpc not in seen_requests, "duplicate turn-start request")
            seen_requests.add(rpc)
            reply = replies.get(rpc, {})
            turn = reply.get("result", {}).get("turn", {}).get("id")
            require("error" not in reply and isinstance(turn, str) and turn, "turn lacks unique acceptance")
            thread = frame["params"].get("threadId"); text = one_text(frame["params"].get("input"))
            key = thread, turn
            require(key not in captured_turns and key in public, "accepted/public turn mismatch")
            captured_turns.add(key)
            public_text, public_id, public_line = public.pop(key)
            require(public_text == text, "accepted/public input bytes differ")
            first = thread not in owners
            if first:
                policy_key = text_digest(text)
                owner, policy_text = policies.get(policy_key, (None, None))
                require(owner is not None and policy_text == text and policy_counts[policy_key] > 0,
                        "first thread input lacks an owned policy occurrence")
                policy_counts[policy_key] -= 1; owners[thread] = owner
            source = native.pop((thread, turn, text_digest(text)), None)
            require(source is not None and source["text"] == text, "exact explicit-turn native input missing")
            locator = {"capture": capture["path"], "client_line": line, "rpc_id": frame["id"],
                       "thread_id": thread, "turn_id": turn, "agent_id": owners[thread],
                       "public_item_id": public_id, "public_line": public_line,
                       **{k: v for k, v in source.items() if k != "text"}}
            accepted.append(locator)
            queue = reviews[owners[thread]]
            if queue:
                event = queue[0]; selected = event["forwarded_prompt"]
                if text == selected or (first and text.endswith("\n\nUser prompt:\n" + selected)):
                    joined.append({**locator, "trace_sequence": event["sequence"],
                                   "selected_prompt_sha256": text_digest(selected)})
                    queue.popleft()
        require(not public, "public user input has no accepted request")
    require(not native and native_threads == set(owners), "native/captured user or thread inventory differs")
    require(not any(policy_counts.values()), "policy occurrence was not delivered")
    require(not any(reviews.values()), "review-context occurrence was not delivered")
    return {"accepted_inputs": len(accepted), "threads": owners, "reviews": joined, "inputs": accepted}
