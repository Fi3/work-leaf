#!/usr/bin/env python3
"""One bounded, subscription-only synthetic Codex dynamic-tool experiment."""

import argparse
from collections import deque
import hashlib
import json
import os
from pathlib import Path
import queue
import signal
import subprocess
import tempfile
import threading
import time


FIELDS = (
    "inputTokens", "cachedInputTokens", "outputTokens", "reasoningOutputTokens", "totalTokens"
)
REMOVED_ENV = {
    "OPENAI_API_KEY", "CODEX_API_KEY", "OPENAI_BASE_URL", "OPENAI_API_BASE", "CODEX_BASE_URL",
    "CODEX_ACCESS_TOKEN", "CODEX_THREAD_ID", "CODEX_CI", "CODEX_MANAGED_BY_NPM",
    "CODEX_MANAGED_PACKAGE_ROOT", "WORK_LEAF_CODEX_TRACE", "WORK_LEAF_COMMAND_TMPDIR",
    "WORK_LEAF_CONTEXT_BUNDLE_DIR", "WORK_LEAF_CODEX_LINEARIZE_SANDBOX",
}
TOOL = "handoff_probe"
ANSWER = "WORK_LEAF_SUBSCRIPTION_HANDOFF_OK"
PROMPT = (
    'This is a synthetic protocol check, not a repository task. Invoke handoff_probe exactly '
    'once with {"request":"ping"}. After receiving its result, reply with exactly the text '
    'returned by that tool and stop. Do not invoke any other tool, run commands, access the '
    'network yourself, read files, or modify files. Do not send a commentary message.'
)


def subscription_account(response):
    return (
        isinstance(response, dict)
        and isinstance(response.get("account"), dict)
        and response["account"].get("type") == "chatgpt"
    )


def child_environment(env):
    return {key: value for key, value in env.items() if key not in REMOVED_ENV}


def valid_usage(usage):
    if not isinstance(usage, dict):
        return False
    if any(type(usage.get(key)) is not int or usage[key] < 0 for key in FIELDS):
        return False
    return (
        usage["cachedInputTokens"] <= usage["inputTokens"]
        and usage["reasoningOutputTokens"] <= usage["outputTokens"]
        and usage["totalTokens"] == usage["inputTokens"] + usage["outputTokens"]
    )


def reconcile_usage(raw_records, total):
    if not isinstance(raw_records, list) or not raw_records or not valid_usage(total):
        return False
    seen = set()
    sums = dict.fromkeys(FIELDS, 0)
    for record in raw_records:
        if not isinstance(record, dict):
            return False
        identity = record.get("responseId")
        usage = record.get("usage")
        if not isinstance(identity, str) or not identity or identity in seen or not valid_usage(usage):
            return False
        seen.add(identity)
        for key in FIELDS:
            sums[key] += usage[key]
    return sums == {key: total[key] for key in FIELDS}


def usage_only(value):
    if not isinstance(value, dict):
        return None
    return {key: value[key] for key in FIELDS if key in value}


def thread_start_params(scratch):
    return {
        "model": "gpt-5.5", "modelProvider": "openai", "cwd": scratch,
        "approvalPolicy": "never", "sandbox": "read-only", "experimentalRawEvents": True,
        "dynamicTools": [{"type": "function", "name": TOOL,
                          "description": "Return a fixed diagnostic string without side effects.",
                          "deferLoading": False,
                          "inputSchema": {"type": "object", "properties": {"request": {"type": "string"}},
                                          "required": ["request"], "additionalProperties": False}}],
    }


def summary(evidence):
    reconciled = reconcile_usage(evidence["responses"], evidence.get("thread_total"))
    count = evidence.get("pre_result_response_count")
    phase_usage_valid = (
        type(count) is int and count == 1 and len(evidence["responses"]) == 2
        and all(valid_usage(row.get("usage")) and row["usage"]["inputTokens"] > 0
                and row["usage"]["outputTokens"] > 0 for row in evidence["responses"])
    )
    passed = (
        evidence.get("account_type") == "chatgpt"
        and evidence.get("resolved_model") == "gpt-5.5"
        and evidence.get("resolved_effort") == "xhigh"
        and evidence.get("tool_request_count") == 1
        and evidence.get("tool_arguments_valid") is True
        and evidence.get("tool_result_sent") is True
        and evidence.get("tool_completed_count") == 1
        and evidence.get("tool_completed_successfully") is True
        and evidence.get("usage_before_tool_result") is True
        and evidence.get("final_answer_matches") is True
        and evidence.get("turn_status") == "completed"
        and phase_usage_valid
        and reconciled
        and not evidence.get("interrupt_sent")
        and not evidence.get("failure")
        and not evidence.get("provider_error_seen")
        and not evidence.get("unexpected_activity")
    )
    return {"single_handoff_pass": passed, "response_totals_reconcile": reconciled}


class Probe:
    def __init__(self, codex, scratch):
        self.started = time.monotonic()
        self.deadline = self.started + 150
        self.events = queue.Queue()
        self.deferred_events = deque()
        self.awaiting_turn_id = False
        self.raw_tool_calls = []
        self.pending_tool = None
        self.tool_received_at = None
        self.result_sent_at = None
        self.evidence = {
            "schema_version": 1, "experiment": "subscription_dynamic_tool_handoff",
            "model": "gpt-5.5", "effort": "xhigh", "model_turns_requested": 0,
            "tool_request_count": 0, "responses": [], "trace": [],
            "usage_before_tool_result": False, "interrupt_sent": False,
            "bounds_seconds": {"total": 150, "request": 15, "tool_request": 45,
                               "withheld_result": 10, "continuation": 45},
            "scratch_directory": scratch,
        }
        self.command = [
            codex, "--disable", "apps", "--disable", "multi_agent", "--disable", "image_generation",
            "--disable", "shell_tool", "--disable", "browser_use", "--disable", "computer_use",
            "--disable", "hooks", "--disable", "plugins", "--cd", scratch, "--sandbox", "read-only",
            "--ask-for-approval", "never", "-c", 'model="gpt-5.5"',
            "-c", 'model_reasoning_effort="xhigh"', "-c", 'model_provider="openai"',
            "-c", 'forced_login_method="chatgpt"', "app-server", "--listen", "stdio://",
        ]
        self.evidence["command"] = self.command
        self.process = None
        self.readers = []

    def stamp(self):
        return round(time.monotonic() - self.started, 6)

    def note(self, event, **fields):
        record = {"elapsed_seconds": self.stamp(), "event": event, **fields}
        self.evidence["trace"].append(record)
        return record

    def progress(self, event, **fields):
        print(json.dumps({"elapsed_seconds": self.stamp(), "event": event, **fields}), flush=True)

    def read_stdout(self):
        for line in self.process.stdout:
            try:
                message = json.loads(line)
                if not isinstance(message, dict):
                    raise ValueError("not an object")
                self.events.put((time.monotonic(), message))
            except (ValueError, UnicodeError):
                self.events.put((time.monotonic(), {"probe_error": "invalid_json"}))
        self.events.put((time.monotonic(), {"probe_error": "stdout_closed"}))

    def read_stderr(self):
        for line in self.process.stderr:
            lowered = line.lower()
            if "read-only file system" in lowered:
                self.evidence["stderr_read_only_error"] = True
            if "401 unauthorized" in lowered or "missing bearer" in lowered:
                self.evidence["stderr_auth_error"] = True
            if "failed to initialize in-process app-server client" in lowered:
                self.evidence["stderr_app_server_initialization_error"] = True

    def send(self, message):
        self.process.stdin.write(json.dumps(message, separators=(",", ":")) + "\n")
        self.process.stdin.flush()
        self.note("client_message", method=message.get("method"), id=message.get("id"))

    def receive(self, timeout):
        remaining = min(timeout, self.deadline - time.monotonic())
        if remaining <= 0:
            return None
        if self.deferred_events and not self.awaiting_turn_id:
            received_at, message = self.deferred_events.popleft()
        else:
            try:
                received_at, message = self.events.get(timeout=remaining)
            except queue.Empty:
                return None
        if "probe_error" in message:
            raise RuntimeError(message["probe_error"])
        method = message.get("method")
        params = message.get("params") or {}
        if (self.awaiting_turn_id and method is not None
                and params.get("threadId") == self.evidence.get("thread_id")):
            self.deferred_events.append((received_at, message))
            return message
        record = self.note("server_message", method=method, id=message.get("id"),
                           received_seconds=round(received_at - self.started, 6))
        # Account responses and arbitrary content deliberately never enter the evidence.
        for key in ("threadId", "turnId", "responseId", "callId"):
            if isinstance(params.get(key), str):
                record[key] = params[key]
        if message.get("error") is not None:
            record["rpc_error_code"] = message["error"].get("code")
        if method == "account/updated" and params.get("authMode") != "chatgpt":
            raise RuntimeError("authentication_changed_away_from_subscription")
        if method in ("error", "model/rerouted"):
            self.evidence["provider_error_seen"] = True
            raise RuntimeError("provider_error_or_model_reroute")
        matching = params.get("threadId") == self.evidence.get("thread_id")
        same_turn = matching and params.get("turnId") == self.evidence.get("turn_id")
        if (matching and method in ("turn/started", "turn/completed")
                and (params.get("turn") or {}).get("id") == self.evidence.get("turn_id")):
            turn = params.get("turn") or {}
            record["turn_status"] = turn.get("status")
            record["turnId"] = turn.get("id")
            if method == "turn/completed":
                self.evidence["turn_status"] = turn.get("status")
        if same_turn and method in ("item/started", "item/completed"):
            item = params.get("item") or {}
            record["item_type"] = item.get("type")
            record["item_status"] = item.get("status")
            if item.get("type") not in ("userMessage", "reasoning", "agentMessage", "dynamicToolCall"):
                self.evidence["unexpected_activity"] = True
                raise RuntimeError("unexpected_builtin_tool")
            if method == "item/completed" and item.get("type") == "agentMessage":
                text = item.get("text", "")
                record["expected_answer"] = text.strip() == ANSWER
                self.evidence["final_answer_matches"] = (
                    self.evidence.get("tool_result_sent") is True and record["expected_answer"]
                )
                if not self.evidence.get("tool_result_sent"):
                    self.evidence["unexpected_activity"] = True
            if method == "item/completed" and item.get("type") == "dynamicToolCall":
                record["success"] = item.get("success")
                record["tool_item_id"] = item.get("id")
                self.evidence["tool_completed_count"] = self.evidence.get("tool_completed_count", 0) + 1
                self.evidence["tool_completed_successfully"] = (
                    item.get("status") == "completed" and item.get("success") is True
                    and item.get("tool") == TOOL and item.get("namespace") is None
                    and item.get("arguments") == {"request": "ping"}
                    and self.evidence["tool_completed_count"] == 1
                )
                if not self.evidence["tool_completed_successfully"]:
                    self.evidence["unexpected_activity"] = True
                    raise RuntimeError("unexpected_or_failed_tool_completion")
        if same_turn and method == "rawResponseItem/completed":
            item = params.get("item") or {}
            record["raw_item_type"] = item.get("type")
            if item.get("type") == "function_call":
                record["raw_call_id"] = item.get("call_id")
                record["expected_tool"] = item.get("name") == TOOL and item.get("namespace") is None
                if record["expected_tool"] and isinstance(item.get("call_id"), str):
                    self.raw_tool_calls.append(item["call_id"])
        if same_turn and method == "rawResponse/completed":
            response = {"responseId": params.get("responseId"), "usage": usage_only(params.get("usage"))}
            if self.raw_tool_calls:
                response["tool_call_ids"] = self.raw_tool_calls
                self.raw_tool_calls = []
            self.evidence["responses"].append(response)
            record["usage"] = response["usage"]
            self.progress("exact_response_completed", responseId=response["responseId"], usage=response["usage"])
        if same_turn and method == "thread/tokenUsage/updated":
            token_usage = params.get("tokenUsage") or {}
            record["last"] = usage_only(token_usage.get("last"))
            record["total"] = usage_only(token_usage.get("total"))
            self.evidence["thread_total"] = record["total"]
        if method == "item/tool/call":
            self.evidence["tool_request_count"] += 1
            valid = (same_turn and params.get("tool") == TOOL and params.get("namespace") is None
                     and params.get("arguments") == {"request": "ping"}
                     and self.evidence["tool_request_count"] == 1)
            self.evidence["tool_arguments_valid"] = valid
            record["arguments_valid"] = valid
            if not valid:
                self.send({"id": message["id"], "result": {"success": False, "contentItems": []}})
                raise RuntimeError("unexpected_tool_request")
            self.pending_tool = message["id"]
            self.tool_received_at = time.monotonic()
            self.evidence["call_id"] = params.get("callId")
            self.progress("tool_request_received")
        elif "method" in message and "id" in message:
            self.send({"id": message["id"], "error": {"code": -32601, "message": "Unsupported in isolated diagnostic"}})
            raise RuntimeError("unexpected_server_request")
        return message

    def request(self, identity, method, params):
        self.send({"id": identity, "method": method, "params": params})
        deadline = min(time.monotonic() + 15, self.deadline)
        while time.monotonic() < deadline:
            message = self.receive(max(0, deadline - time.monotonic()))
            if message and message.get("id") == identity and "method" not in message:
                if message.get("error") is not None:
                    raise RuntimeError("rpc_rejected_" + method.replace("/", "_"))
                return message.get("result", {})
        raise TimeoutError("request_timeout_" + method.replace("/", "_"))

    def run(self):
        self.process = subprocess.Popen(
            self.command, cwd=self.evidence["scratch_directory"], env=child_environment(os.environ),
            stdin=subprocess.PIPE, stdout=subprocess.PIPE, stderr=subprocess.PIPE,
            text=True, encoding="utf-8", errors="replace", bufsize=1, start_new_session=True,
        )
        for reader in (self.read_stdout, self.read_stderr):
            thread = threading.Thread(target=reader, daemon=True)
            thread.start()
            self.readers.append(thread)
        self.evidence["stage"] = "initialize"
        self.request("init", "initialize", {
            "clientInfo": {"name": "work_leaf_handoff_probe", "version": "1"},
            "capabilities": {"experimentalApi": True},
        })
        self.send({"method": "initialized"})
        account = self.request("account", "account/read", {"refreshToken": False})
        if not subscription_account(account):
            raise RuntimeError("subscription_authentication_required")
        self.evidence["account_type"] = "chatgpt"
        self.progress("subscription_authentication_verified")
        self.evidence["stage"] = "thread_start"
        started = self.request("thread", "thread/start", thread_start_params(self.evidence["scratch_directory"]))
        self.evidence.update(thread_id=started["thread"]["id"], resolved_model=started.get("model"),
                             resolved_effort=started.get("reasoningEffort"), model_provider=started.get("modelProvider"))
        if (started.get("model"), started.get("reasoningEffort"), started.get("modelProvider")) != ("gpt-5.5", "xhigh", "openai"):
            raise RuntimeError("resolved_configuration_mismatch")
        self.evidence["stage"] = "tool_request"
        self.evidence["model_turns_requested"] = 1
        self.awaiting_turn_id = True
        turn = self.request("turn", "turn/start", {
            "threadId": self.evidence["thread_id"], "model": "gpt-5.5", "effort": "xhigh",
            "input": [{"type": "text", "text": PROMPT}],
        })
        self.evidence["turn_id"] = turn["turn"]["id"]
        self.awaiting_turn_id = False
        request_deadline = time.monotonic() + 45
        while time.monotonic() < self.deadline:
            if self.pending_tool is not None:
                has_usage = any(valid_usage(row["usage"])
                                and self.evidence.get("call_id") in row.get("tool_call_ids", [])
                                for row in self.evidence["responses"])
                if has_usage or time.monotonic() - self.tool_received_at >= 10:
                    self.evidence["usage_before_tool_result"] = has_usage
                    self.evidence["pre_result_response_count"] = len(self.evidence["responses"])
                    self.evidence["tool_result_wait_seconds"] = round(time.monotonic() - self.tool_received_at, 6)
                    self.send({"id": self.pending_tool, "result": {
                        "contentItems": [{"type": "inputText", "text": ANSWER}], "success": True,
                    }})
                    self.pending_tool = None
                    self.evidence["tool_result_sent"] = True
                    self.result_sent_at = time.monotonic()
                    self.evidence["stage"] = "continuation"
                    self.progress("tool_result_sent", usage_before_result=has_usage)
            if self.evidence.get("turn_status") is not None:
                self.evidence["stage"] = "finished"
                return
            if self.tool_received_at is None and time.monotonic() > request_deadline:
                raise TimeoutError("tool_request_timeout")
            if self.result_sent_at is not None and time.monotonic() - self.result_sent_at > 45:
                raise TimeoutError("continuation_timeout")
            self.receive(0.1)
        raise TimeoutError("overall_timeout")

    def cleanup(self):
        if self.process is None:
            return
        if self.evidence.get("turn_id") and self.evidence.get("turn_status") is None:
            self.evidence["interrupt_sent"] = True
            try:
                self.send({"id": "cleanup", "method": "turn/interrupt", "params": {
                    "threadId": self.evidence["thread_id"], "turnId": self.evidence["turn_id"],
                }})
                end = time.monotonic() + 5
                while time.monotonic() < end and self.evidence.get("turn_status") is None:
                    self.receive(0.1)
            except (OSError, RuntimeError):
                self.evidence["cleanup_interrupt_error"] = True
        try:
            self.process.stdin.close()
        except OSError:
            pass
        try:
            self.process.wait(timeout=5)
        except subprocess.TimeoutExpired:
            try:
                os.killpg(self.process.pid, signal.SIGTERM)
            except ProcessLookupError:
                pass
            try:
                self.process.wait(timeout=5)
            except subprocess.TimeoutExpired:
                try:
                    os.killpg(self.process.pid, signal.SIGKILL)
                except ProcessLookupError:
                    pass
                self.process.wait(timeout=5)
        for thread in self.readers:
            thread.join(timeout=1)
        self.evidence["process_returncode"] = self.process.returncode


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--codex", default="/usr/bin/codex")
    parser.add_argument("--output", required=True)
    args = parser.parse_args()
    def interrupted(signum, frame):
        raise RuntimeError("parent_signal_" + str(signum))
    signal.signal(signal.SIGTERM, interrupted)
    signal.signal(signal.SIGINT, interrupted)
    # Exclusive creation occurs before any provider process or model work.
    with Path(args.output).open("x", encoding="utf-8") as output:
        scratch = tempfile.mkdtemp(prefix="work-leaf-handoff-")
        probe = Probe(args.codex, scratch)
        probe.evidence["codex_version"] = subprocess.check_output(
            [args.codex, "--version"], env=child_environment(os.environ), text=True, timeout=10,
        ).strip()
        probe.evidence["helper_sha256"] = hashlib.sha256(Path(__file__).read_bytes()).hexdigest()
        probe.evidence["prompt"] = PROMPT
        try:
            probe.run()
        except (RuntimeError, TimeoutError, OSError, KeyError) as error:
            # Only our controlled error labels are retained, never raw provider messages.
            probe.evidence["failure"] = str(error) if type(error) in (RuntimeError, TimeoutError) else type(error).__name__
            probe.progress("probe_failed", failure=probe.evidence["failure"])
        finally:
            try:
                probe.cleanup()
            except (OSError, RuntimeError, subprocess.TimeoutExpired) as error:
                probe.evidence["failure"] = "cleanup_" + type(error).__name__
            probe.evidence["elapsed_seconds"] = probe.stamp()
            probe.evidence.update(summary(probe.evidence))
            json.dump(probe.evidence, output, indent=2)
            output.write("\n")
        print(json.dumps(summary(probe.evidence)), flush=True)
        return 0 if probe.evidence["single_handoff_pass"] else 2


if __name__ == "__main__":
    raise SystemExit(main())
