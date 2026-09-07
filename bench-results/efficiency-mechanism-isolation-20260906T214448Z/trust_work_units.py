#!/usr/bin/env python3
"""Prospective, read-only classification of one prescribed native trust transition.

This is not a Codex configuration resolver. It accepts only an exact new trusted
entry for an admitted primary invocation, with unchanged complete parsed remainder
and bounded project-layer evidence. Prefix hashes attest what existed at the live
decision. Later replay needs the matching external final global configuration; no
configuration contents or credentials are copied. With H snapshots, P proofs and B
configuration bytes, replay costs O(H*(B+P)) plus bounded capture/inventory scans.
The H*P metadata term can be quadratic as a study grows; this study caps P at 12.
"""

from copy import deepcopy
from datetime import datetime, timezone
import hashlib
import json
import os
from pathlib import Path
import stat
import tomllib


ROOT_POLICY = "canonical-cwd-is-git-root; default-markers-only; no-profile-or-resource-reference"
SAFE_LEGACY = {"unchanged", "formatting_only", "known_bookkeeping_only"}
MAX_BYTES = 128 * 1024 * 1024
MAX_LINE = 8 * 1024 * 1024
REQUIRED = (".codex", ".codex/config.toml", ".codex/rules", ".codex/hooks.json", ".codex/hooks")


class IntegrityError(ValueError):
    pass


class NativePending(IntegrityError):
    pass


def require(condition, code):
    if not condition:
        raise IntegrityError(code)


def now():
    return datetime.now(timezone.utc).isoformat()


def digest(value, *, unicode=False):
    return hashlib.sha256(json.dumps(value, sort_keys=True, separators=(",", ":"),
                                     ensure_ascii=not unicode, default=str, allow_nan=False).encode()).hexdigest()


def inventory_digest(entries):
    return digest(entries, unicode=True)


def timestamp_ns(value):
    return int(datetime.fromisoformat(value.replace("Z", "+00:00")).timestamp() * 1e9)


def read_regular(path, limit=MAX_LINE):
    path = Path(path)
    before = path.lstat()
    require(stat.S_ISREG(before.st_mode) and before.st_size <= limit, "unsupported-evidence-file")
    fd = os.open(path, os.O_RDONLY | os.O_NOFOLLOW | os.O_NONBLOCK)
    with os.fdopen(fd, "rb") as handle:
        opened = os.fstat(handle.fileno())
        value = handle.read(limit + 1)
        after = os.fstat(handle.fileno())
    identity = lambda s: (s.st_dev, s.st_ino, s.st_size, s.st_mtime_ns, s.st_ctime_ns, s.st_mode)
    require(identity(before) == identity(opened) == identity(after) and len(value) <= limit,
            "evidence-changed-during-read")
    return value


def project_entries(repo):
    """Rehash the observer's already-supported scope; no Git calls or config writes."""
    repo = Path(repo)
    require(repo.is_absolute() and repo.resolve() == repo and stat.S_ISDIR((repo / ".git").lstat().st_mode),
            "noncanonical-project-root")
    entries, visited, byte_count = {}, 0, 0

    def walk(relative, depth):
        nonlocal visited, byte_count
        visited += 1
        require(visited <= 512 and depth <= 16, "project-inventory-bound")
        name = relative.as_posix()
        name.encode("utf-8", errors="strict")
        path = repo / relative
        try:
            before = path.lstat()
        except FileNotFoundError:
            entries[name] = {"path": name, "kind": "absent"}
            return
        record = {"path": name, "mode": stat.S_IMODE(before.st_mode)}
        if stat.S_ISDIR(before.st_mode):
            record["kind"] = "directory"
            entries[name] = record
            with os.scandir(path) as children:
                for child in children:
                    require(visited < 512, "project-inventory-bound")
                    walk(relative / child.name, depth + 1)
        elif stat.S_ISREG(before.st_mode):
            require(path.name not in {"auth.json", "credentials.json", "credentials", "id_rsa", "id_ed25519"},
                    "credential-project-layer")
            require(before.st_size <= 1024 * 1024 and byte_count + before.st_size <= 8 * 1024 * 1024,
                    "project-inventory-bound")
            content = read_regular(path, 1024 * 1024)
            byte_count += len(content)
            record.update(kind="file", size=len(content), sha256=hashlib.sha256(content).hexdigest())
            entries[name] = record
        else:
            raise IntegrityError("unsupported-project-layer-type")
        after = path.lstat()
        require((before.st_ino, before.st_dev, before.st_mode, before.st_mtime_ns, before.st_ctime_ns, before.st_size)
                == (after.st_ino, after.st_dev, after.st_mode, after.st_mtime_ns, after.st_ctime_ns, after.st_size),
                "project-layer-changed-during-read")

    walk(Path(".codex"), 0)
    for name in REQUIRED:
        if name not in entries:
            require(len(entries) < 512, "project-inventory-bound")
            require(not (repo / name).exists(), "unvisited-required-project-layer")
            entries[name] = {"path": name, "kind": "absent"}
    return [entries[key] for key in sorted(entries)]


def observation_for(row, study):
    results = Path(row["results_dir"])
    require(results.is_absolute() and results.resolve() == results, "noncanonical-results-root")
    matches = []
    if results.exists():
        with os.scandir(results) as children:
            for index, child in enumerate(children):
                require(index < 64, "results-discovery-bound")
                if not child.is_dir(follow_symlinks=False):
                    continue
                candidate = Path(child.path) / "observation"
                config = candidate / "observer-config.json"
                if config.is_file():
                    parsed = json.loads(read_regular(config))
                    if parsed.get("run_id") == row["run_id"] and parsed.get("study_id") == study:
                        require(parsed.get("condition") == "work-leaf", "wrong-observation-condition")
                        require(candidate.resolve() == candidate, "symlinked-observation-root")
                        matches.append(candidate)
    require(len(matches) == 1, "missing-or-ambiguous-own-observation")
    return matches[0]


class Evidence:
    def __init__(self, observation, cutoff, replay=None):
        self.observation, self.cutoff = observation, cutoff
        self.sources = {}
        self.replay = {(item["location"], item["relative"]): item for item in replay or []}

    def lines(self, relative, *, external=False):
        location = "external-native" if external else "observation"
        path = Path(relative) if external else self.observation / relative
        require(path.resolve() == path and stat.S_ISREG(path.lstat().st_mode), "unsafe-evidence-path")
        key = (location, str(relative))
        expected = self.replay.get(key)
        if self.replay:
            require(expected is not None, "unrecorded-replay-source")
        maximum = expected["length"] if expected else MAX_BYTES
        require(0 < maximum <= MAX_BYTES, "evidence-prefix-bound")
        hashed, length = hashlib.sha256(), 0
        fd = os.open(path, os.O_RDONLY | os.O_NOFOLLOW | os.O_NONBLOCK)
        with os.fdopen(fd, "rb") as handle:
            while length < maximum:
                line = handle.readline(min(MAX_LINE + 1, maximum - length))
                if not line:
                    break
                require(len(line) <= MAX_LINE, "oversized-evidence-frame")
                if not line.endswith(b"\n") and external and not self.replay:
                    # A live append tail is not evidence; the completed prefix remains available.
                    break
                require(line.endswith(b"\n"), "incomplete-evidence-frame")
                hashed.update(line)
                length += len(line)
                self.sources[key] = {"location": location, "relative": str(relative), "path": str(path),
                                     "length": length, "sha256": hashed.hexdigest(), "captured_at": self.cutoff}
                parsed = json.loads(line)
                require(isinstance(parsed, dict), "nonobject-evidence-frame")
                yield parsed
            if expected:
                require(length == expected["length"] and hashed.hexdigest() == expected["sha256"],
                        "evidence-prefix-identity-mismatch")

    def one(self, relative):
        # start/child/config are pretty-printed JSON, not necessarily one JSONL frame.
        path = self.observation / relative
        require(path.resolve() == path, "unsafe-evidence-path")
        content = read_regular(path)
        item = {"location": "observation", "relative": str(relative), "path": str(path),
                "length": len(content), "sha256": hashlib.sha256(content).hexdigest(), "captured_at": self.cutoff}
        if self.replay:
            expected = self.replay.get(("observation", str(relative)))
            require(expected is not None and item["length"] == expected["length"] and item["sha256"] == expected["sha256"],
                    "evidence-prefix-identity-mismatch")
        self.sources[("observation", str(relative))] = item
        parsed = json.loads(content)
        require(isinstance(parsed, dict), "nonobject-evidence-metadata")
        return parsed

    def validate_prefixes(self):
        # Generators stopped after a selected frame do not run their end checks.
        for key, expected in self.replay.items():
            actual = self.sources.get(key)
            require(actual and actual["length"] == expected["length"] and actual["sha256"] == expected["sha256"],
                    "evidence-prefix-identity-mismatch")


def typed_id(value):
    require(type(value) in (str, int), "invalid-rpc-id-type")
    return type(value).__name__, value


def prove_transition(manifest, row, trusted, cutoff, *, replay=None, native_required=True):
    observation = observation_for(row, manifest["study"])
    evidence = Evidence(observation, cutoff, replay.get("source_identities") if replay else None)
    config = evidence.one("observer-config.json")
    require(config.get("run_id") == row["run_id"] and config.get("study_id") == manifest["study"], "wrong-observer-identity")
    runtime = Path(row["runtime_dir"])
    repo = Path(trusted)
    require(repo.is_absolute() and repo.parent.parent == runtime and runtime.resolve() == runtime,
            "trust-entry-not-exact-owned-checkout")
    candidates = []
    with os.scandir(observation / "invocations") as children:
        for index, child in enumerate(children):
            require(index < 4096, "invocation-discovery-bound")
            if child.is_dir(follow_symlinks=False):
                start_path = Path(child.path) / "start.json"
                if start_path.is_file():
                    value = json.loads(read_regular(start_path))
                    if value.get("primary") is True and value.get("capture_kind") == "app-server":
                        candidates.append(child.name)
    require(len(candidates) == 1, "missing-or-ambiguous-primary-invocation")
    invocation = candidates[0]
    start = evidence.one(f"invocations/{invocation}/start.json")
    child = evidence.one(f"invocations/{invocation}/child.json")
    require(start.get("cwd") == trusted and start.get("invocation_id") == invocation
            and start.get("parent_invocation_id") is None and start.get("project_layer_inventory_required") is True,
            "wrong-primary-project-identity")
    capture = f"app-server/{invocation}"
    request, preceding = None, 0
    for frame in evidence.lines(f"{capture}/client-to-server.raw"):
        if frame.get("method") != "thread/start":
            continue
        params = frame.get("params", {})
        require(params.get("cwd") == trusted and params.get("approvalPolicy") == "never"
                and params.get("model") == manifest["model"], "unexpected-prescribed-thread-settings")
        if params.get("sandbox") == "danger-full-access":
            request = frame
            break
        require(params.get("sandbox") == "read-only", "unexpected-preceding-thread-sandbox")
        preceding += 1
    require(request is not None and preceding > 0, "missing-prescribed-full-access-transition")
    request_id = typed_id(request["id"])
    forwarded = None
    for frame in evidence.lines(f"{capture}/client-to-server.forwarded.raw"):
        if frame.get("method") == "thread/start" and typed_id(frame.get("id")) == request_id:
            forwarded = frame
            break
    expected = deepcopy(request)
    expected["params"]["experimentalRawEvents"] = True
    require(forwarded == expected, "missing-or-modified-forwarded-thread-start")
    reply, notification = None, None
    for frame in evidence.lines(f"{capture}/server-to-client.raw"):
        if "id" in frame and "method" not in frame and typed_id(frame["id"]) == request_id:
            require("error" not in frame and isinstance(frame.get("result"), dict), "full-access-thread-start-not-accepted")
            reply = frame["result"]
        if frame.get("method") == "thread/started" and reply:
            if frame.get("params", {}).get("thread", {}).get("id") == reply.get("thread", {}).get("id"):
                notification = frame
        if reply and notification:
            break
    require(reply and notification, "missing-typed-reply-or-thread-start-time")
    thread = reply.get("thread", {})
    require(type(thread.get("id")) is str and bool(thread["id"])
            and type(notification["params"]["thread"].get("id")) is str,
            "invalid-thread-identity-type")
    require(reply.get("cwd") == trusted and thread.get("cwd") == trusted and reply.get("sandbox") == {"type": "dangerFullAccess"}
            and reply.get("approvalPolicy") == "never" and reply.get("model") == manifest["model"]
            and reply.get("reasoningEffort") == manifest["reasoning_effort"], "accepted-thread-settings-mismatch")
    emitted_ns = notification.get("emittedAtMs", 0) * 1_000_000
    require(0 < emitted_ns <= timestamp_ns(cutoff), "thread-start-after-classification")
    native = Path(thread.get("path", ""))
    session_root = Path(manifest["global_config_baseline"]["path"]).parent / "sessions"
    require(native.is_absolute() and native.is_relative_to(session_root), "native-source-outside-subscription-sessions")
    if native_required:
        session_ok, permission_ok, context_ok = False, False, False
        try:
            for frame in evidence.lines(str(native), external=True):
                if timestamp_ns(frame["timestamp"]) > timestamp_ns(cutoff) and not replay:
                    # Publication during this read is not evidence available at its earlier cutoff.
                    raise NativePending("native-attestation-not-yet-published")
                require(timestamp_ns(frame["timestamp"]) <= timestamp_ns(cutoff), "native-evidence-after-classification")
                payload = frame.get("payload", {})
                if frame.get("type") == "session_meta":
                    require(type(payload.get("id")) is str and bool(payload["id"])
                            and payload["id"] == thread["id"] and payload.get("cwd") == trusted
                            and payload.get("cli_version") == "0.153.4", "native-session-identity-mismatch")
                    session_ok = True
                if frame.get("type") == "response_item" and payload.get("role") == "developer":
                    texts = [item.get("text", "") for item in payload.get("content", []) if isinstance(item, dict)]
                    for value in texts:
                        if "<permissions instructions>" in value:
                            require("`sandbox_mode` is `danger-full-access`" in value
                                    and "Network access is enabled." in value
                                    and "Approval policy is currently never." in value,
                                    "native-permission-context-mismatch")
                            permission_ok = True
                if frame.get("type") == "turn_context":
                    require(payload.get("cwd") == trusted and payload.get("sandbox_policy") == {"type": "danger-full-access"}
                            and payload.get("approval_policy") == "never" and payload.get("model") == manifest["model"]
                            and payload.get("effort") == manifest["reasoning_effort"], "native-permission-context-mismatch")
                    context_ok = True
                    break
        except FileNotFoundError as error:
            raise NativePending("native-attestation-not-yet-published") from error
        if not context_ok:
            raise NativePending("native-attestation-not-yet-published")
        require(session_ok and permission_ok, "native-permission-context-mismatch")
    records = list(evidence.lines("project-layer-inventory/manifest.jsonl"))
    prespawn, prelinearize = [], []
    for record in records:
        require(record.get("schema") == "work-leaf-project-layer-inventory-v1" and record.get("run_id") == row["run_id"]
                and record.get("study_id") == manifest["study"] and record.get("repository") == trusted
                and record.get("valid") is True and record.get("errors") == [] and record.get("root_policy") == ROOT_POLICY
                and inventory_digest(record["entries"]) == record.get("inventory_sha256"), "invalid-project-inventory")
        require(int(record["completed_unix_ns"]) <= timestamp_ns(cutoff), "inventory-after-classification")
        if record.get("label") == "pre-spawn" and record.get("invocation_id") == invocation:
            prespawn.append(record)
        if record.get("label") == "checkpoint:pre-linearize":
            prelinearize.append(record)
    require(len(prespawn) == 1 and len(prelinearize) == 1, "missing-or-ambiguous-required-inventory")
    before, after = prespawn[0], prelinearize[0]
    require(int(before["completed_monotonic_ns"]) <= child["started_monotonic_ns"]
            and int(before["completed_unix_ns"]) <= int(after["completed_unix_ns"]) <= emitted_ns,
            "inventory-transition-order-unverified")
    expected_inventory = before["inventory_sha256"]
    require(all(item["inventory_sha256"] == expected_inventory for item in records)
            and after.get("matches_pre_spawn") is True, "project-layer-drift")
    if replay:
        require(replay["transition_inventory_sha256"] == expected_inventory, "transition-inventory-mismatch")
        evidence.validate_prefixes()
    else:
        require(inventory_digest(project_entries(repo)) == expected_inventory, "actual-project-layer-drift")
    return {"run_id": row["run_id"], "trusted_project": trusted, "invocation_id": invocation,
            "request_id": {"type": request_id[0], "value": request_id[1]}, "thread_id": thread["id"],
            "classified_at": cutoff, "thread_started_unix_ns": str(emitted_ns),
            "native_attestation": "verified" if native_required else "pending",
            "transition_inventory_sha256": expected_inventory,
            "transition_inventory_checked_at": replay["transition_inventory_checked_at"] if replay else now(),
            "source_identities": list(evidence.sources.values())}


def write_new(path, value):
    with Path(path).open("x", encoding="utf-8") as handle:
        json.dump(value, handle, indent=2, sort_keys=True, allow_nan=False)
        handle.write("\n")
        handle.flush()
        os.fsync(handle.fileno())


class TrustClassifier:
    def __init__(self, manifest, phase):
        self.manifest, self.phase, self.proofs = manifest, Path(phase), {}
        self.pending = {}
        self.rows_by_runtime = {row["runtime_dir"]: row for row in manifest["schedule"]}
        self.baseline = None
        try:
            parsed = tomllib.loads(read_regular(manifest["global_config_baseline"]["path"]).decode())
            if digest(parsed) == manifest["global_config_baseline"]["parsed_sha256"]:
                self.baseline = parsed
        except (OSError, ValueError, KeyError, TypeError):
            pass

    def classify(self, snapshot, legacy, launched):
        result = {"verdict": "unexplained_config_drift", "allows_admission": False,
                  "legacy_verdict": legacy, "proof_ids": [], "pending_ids": [], "pending_transition": False, "errors": []}
        try:
            require(self.baseline is not None, "baseline-config-identity-mismatch")
            content = read_regular(snapshot["path"])
            require(hashlib.sha256(content).hexdigest() == snapshot.get("sha256"), "global-config-snapshot-race")
            current = tomllib.loads(content.decode())
            require(digest(current) == snapshot.get("parsed_sha256"), "global-config-parsed-identity-mismatch")
            normalized = deepcopy(current)
            original_projects = self.baseline.get("projects", {})
            projects = normalized.get("projects", {})
            require(isinstance(projects, dict) and isinstance(original_projects, dict), "unsupported-projects-shape")
            additions = sorted(set(projects) - set(original_projects))
            for trusted in additions:
                require(projects[trusted] == {"trust_level": "trusted"}, "unexpected-new-project-entry")
                del projects[trusted]
            if "projects" not in self.baseline and not projects:
                normalized.pop("projects", None)
            require(legacy != "known_bookkeeping_only", "parsed-bookkeeping-history-not-replayable")
            if not additions and legacy in SAFE_LEGACY:
                result.update(verdict="legacy_no_behavioral_drift", allows_admission=True)
                return result
            require(additions and digest(normalized) == digest(self.baseline), "full-config-remainder-changed")
            for trusted in additions:
                if trusted not in self.proofs:
                    row = self.rows_by_runtime.get(str(Path(trusted).parent.parent))
                    require(len(self.rows_by_runtime) == len(self.manifest["schedule"])
                            and row is not None and row["run_id"] in launched, "trust-entry-not-admitted-owned-workflow")
                    existing_pending = self.pending.get(trusted)
                    if existing_pending:
                        require(timestamp_ns(snapshot["captured_at"]) <= int(existing_pending["deadline_unix_ns"]),
                                "native-attestation-pending-deadline")
                    try:
                        proof = prove_transition(self.manifest, row, trusted, snapshot["captured_at"])
                    except NativePending:
                        if not existing_pending:
                            core = prove_transition(self.manifest, row, trusted, snapshot["captured_at"], native_required=False)
                            core["deadline_unix_ns"] = str(timestamp_ns(snapshot["captured_at"]) + 30_000_000_000)
                            core["pending_id"] = digest(core)
                            with (self.phase / "trust-pending.jsonl").open("a", encoding="utf-8") as handle:
                                handle.write(json.dumps(core, sort_keys=True, allow_nan=False) + "\n")
                                handle.flush()
                                os.fsync(handle.fileno())
                            self.pending[trusted] = core
                            existing_pending = core
                        result["pending_ids"].append(existing_pending["pending_id"])
                        continue
                    if existing_pending:
                        require(proof["thread_id"] == existing_pending["thread_id"]
                                and proof["request_id"] == existing_pending["request_id"]
                                and proof["transition_inventory_sha256"] == existing_pending["transition_inventory_sha256"],
                                "pending-transition-identity-changed")
                        proof["resolves_pending_id"] = existing_pending["pending_id"]
                    proof["proof_id"] = digest(proof)
                    with (self.phase / "trust-evidence.jsonl").open("a", encoding="utf-8") as handle:
                        handle.write(json.dumps(proof, sort_keys=True, allow_nan=False) + "\n")
                        handle.flush()
                        os.fsync(handle.fileno())
                    self.proofs[trusted] = proof
                result["proof_ids"].append(self.proofs[trusted]["proof_id"])
            is_pending = bool(result["pending_ids"])
            result.update(verdict="pending_own_workflow_native_attestation" if is_pending else "verified_own_workflow_trust_transition",
                          allows_admission=not is_pending, pending_transition=is_pending,
                          normalized_parsed_sha256=digest(normalized))
        except (OSError, ValueError, KeyError, TypeError, OverflowError, AttributeError, IndexError) as error:
            result["errors"] = [str(error) if isinstance(error, IntegrityError) else type(error).__name__]
        return result

    def finish(self, snapshots):
        write_new(self.phase / "trust-final.json", {
            "schema": "work-leaf-own-workflow-trust-v1", "snapshots": snapshots,
            "final_global_snapshot": snapshots[-1] if snapshots else None,
            "proof_ids": sorted(proof["proof_id"] for proof in self.proofs.values()),
            "pending_ids": sorted(proof["pending_id"] for proof in self.pending.values()),
            "legacy_flags_preserved": True, "self_contained_global_config_replay": False,
            "limitation": "Independent parsed-global replay requires the matching external final global file; no configuration contents are retained."})


def audit(phase_root):
    """Independently replay saved evidence prefixes and external matching global config."""
    result = {"valid": False, "errors": [], "source_identities": [], "verified_snapshot_indices": [],
              "legacy_flags_preserved": True, "self_contained_global_config_replay": False}
    try:
        phase = Path(phase_root)
        manifest = json.loads(read_regular(phase / "PHASE-MANIFEST.json"))
        rows_by_id = {row["run_id"]: row for row in manifest["schedule"]}
        require(len(rows_by_id) == len(manifest["schedule"]), "duplicate-scheduled-run-identity")
        ledger = json.loads(read_regular(phase / "trust-final.json"))
        snapshots = ledger["snapshots"]
        history_path = phase / "config-history.json"
        history = json.loads(read_regular(history_path))
        require(history["snapshots"] == snapshots and history["baseline"] == manifest["global_config_baseline"], "config-history-ledger-mismatch")
        require([json.loads(line) for line in read_regular(phase / "config-history.jsonl").splitlines()] == snapshots,
                "config-live-journal-ledger-mismatch")
        phase_result = json.loads(read_regular(phase / "PHASE-RESULT.json"))
        launched = {row["run_id"]: row["started_at"] for row in phase_result["runs"] if row.get("started_at")}

        def admitted_before(record):
            require(record["run_id"] in launched and timestamp_ns(launched[record["run_id"]]) < timestamp_ns(record["classified_at"]),
                    "proof-run-not-launched-before-classification")
        final = ledger["final_global_snapshot"]
        require(final == snapshots[-1], "final-global-ledger-mismatch")
        content = read_regular(final["path"])
        require(hashlib.sha256(content).hexdigest() == final["sha256"], "external-final-global-snapshot-unavailable")
        parsed = tomllib.loads(content.decode())
        require(digest(parsed) == final["parsed_sha256"], "final-global-parsed-mismatch")
        result["global_snapshot_replay"] = {"path": final["path"], "raw_sha256": final["sha256"],
                                             "parsed_sha256": final["parsed_sha256"], "checked_at": now()}
        pending = {}
        pending_file = phase / "trust-pending.jsonl"
        for line in read_regular(pending_file).splitlines() if pending_file.exists() else []:
            record = json.loads(line)
            identity = record.pop("pending_id")
            require(identity == digest(record) and identity not in pending, "pending-proof-identity-mismatch")
            record["pending_id"] = identity
            require(int(record["deadline_unix_ns"]) == timestamp_ns(record["classified_at"]) + 30_000_000_000,
                    "pending-deadline-changed")
            row = rows_by_id[record["run_id"]]
            admitted_before(record)
            replayed = prove_transition(manifest, row, record["trusted_project"], record["classified_at"],
                                        replay=record, native_required=False)
            require(all(replayed[key] == record[key] for key in replayed if key != "source_identities"), "pending-core-semantic-mismatch")
            result["source_identities"].extend(replayed["source_identities"])
            pending[identity] = record
        require(sorted(pending) == ledger.get("pending_ids", []), "pending-proof-ledger-mismatch")
        proofs = {}
        proof_file = phase / "trust-evidence.jsonl"
        for line in read_regular(proof_file).splitlines() if proof_file.exists() else []:
            proof = json.loads(line)
            identity = proof.pop("proof_id")
            require(identity == digest(proof) and identity not in proofs, "trust-proof-identity-mismatch")
            proof["proof_id"] = identity
            row = rows_by_id[proof["run_id"]]
            admitted_before(proof)
            replayed = prove_transition(manifest, row, proof["trusted_project"], proof["classified_at"], replay=proof)
            require(all(replayed[key] == proof[key] for key in replayed if key != "source_identities"), "trust-proof-semantic-mismatch")
            result["source_identities"].extend(replayed["source_identities"])
            observation = observation_for(row, manifest["study"])
            inventory_path = observation / "project-layer-inventory/manifest.jsonl"
            inventory_bytes = read_regular(inventory_path)
            inventories = [json.loads(line) for line in inventory_bytes.splitlines()]
            require(inventories and inventories[-1].get("label") == "checkpoint:final", "missing-final-project-inventory")
            require(all(item.get("schema") == "work-leaf-project-layer-inventory-v1"
                        and item.get("run_id") == row["run_id"] and item.get("study_id") == manifest["study"]
                        and item.get("repository") == proof["trusted_project"] and item.get("root_policy") == ROOT_POLICY
                        and item.get("valid") is True and item.get("errors") == []
                        and inventory_digest(item["entries"]) == item.get("inventory_sha256") == proof["transition_inventory_sha256"]
                        for item in inventories), "final-project-inventory-drift")
            result["source_identities"].append({"location": "final-observation-inventory", "path": str(inventory_path),
                                                "length": len(inventory_bytes), "sha256": hashlib.sha256(inventory_bytes).hexdigest()})
            proofs[identity] = proof
        require(sorted(proofs) == ledger["proof_ids"], "trust-proof-ledger-mismatch")
        resolution_index = {}
        for proof in proofs.values():
            if "resolves_pending_id" in proof:
                identity = proof["resolves_pending_id"]
                require(identity in pending and identity not in resolution_index, "ambiguous-pending-resolution")
                resolution_index[identity] = proof
        for pending_id, core in pending.items():
            require(pending_id in resolution_index, "pending-native-attestation-unresolved")
            resolution = resolution_index[pending_id]
            require(all(resolution[key] == core[key] for key in ("run_id", "trusted_project", "thread_id", "request_id",
                        "invocation_id", "transition_inventory_sha256"))
                    and timestamp_ns(core["classified_at"]) <= timestamp_ns(resolution["classified_at"]) <= int(core["deadline_unix_ns"]),
                    "pending-resolution-identity-or-deadline-mismatch")
            require(not any(timestamp_ns(core["classified_at"]) <= timestamp_ns(started) <= timestamp_ns(resolution["classified_at"])
                            for started in launched.values()), "workflow-admitted-during-pending-attestation")
        baseline = deepcopy(parsed)
        for proof in proofs.values():
            require(baseline.get("projects", {}).pop(proof["trusted_project"], None) == {"trust_level": "trusted"}, "final-own-trust-entry-mismatch")
        baseline_hash = manifest["global_config_baseline"]["parsed_sha256"]
        if digest(baseline) != baseline_hash and baseline.get("projects") == {}:
            baseline.pop("projects")
        require(digest(baseline) == baseline_hash, "external-global-baseline-reconstruction-failed")
        for index, snapshot in enumerate(snapshots):
            verdict = snapshot["trust_classification"]
            require(verdict["legacy_verdict"] == snapshot["drift_from_baseline"], "legacy-verdict-not-preserved")
            restored = deepcopy(baseline)
            for identity in verdict["proof_ids"]:
                require(identity in proofs, "unknown-snapshot-trust-proof")
                proof = proofs[identity]
                require(timestamp_ns(proof["classified_at"]) <= timestamp_ns(snapshot["captured_at"]), "proof-after-snapshot")
                restored.setdefault("projects", {})[proof["trusted_project"]] = {"trust_level": "trusted"}
            for identity in verdict.get("pending_ids", []):
                require(identity in pending, "unknown-snapshot-pending-proof")
                core = pending[identity]
                require(timestamp_ns(core["classified_at"]) <= timestamp_ns(snapshot["captured_at"]) <= int(core["deadline_unix_ns"]),
                        "pending-snapshot-outside-deadline")
                restored.setdefault("projects", {})[core["trusted_project"]] = {"trust_level": "trusted"}
            require(digest(restored) == snapshot["parsed_sha256"], "historical-global-reconstruction-failed")
            is_pending = bool(verdict.get("pending_ids"))
            require(verdict["allows_admission"] is not is_pending and verdict["errors"] == [], "unexplained-live-config-drift")
            if snapshot["drift_from_baseline"] not in SAFE_LEGACY:
                require(verdict["verdict"] == ("pending_own_workflow_native_attestation" if is_pending else "verified_own_workflow_trust_transition")
                        and (verdict["proof_ids"] or verdict.get("pending_ids"))
                        and verdict["normalized_parsed_sha256"] == baseline_hash, "unverified-live-config-exception")
            result["verified_snapshot_indices"].append(index)
        result["valid"] = True
    except (OSError, ValueError, KeyError, TypeError, OverflowError, StopIteration, AttributeError, IndexError) as error:
        result["errors"].append(str(error) if isinstance(error, IntegrityError) else type(error).__name__)
    return result
