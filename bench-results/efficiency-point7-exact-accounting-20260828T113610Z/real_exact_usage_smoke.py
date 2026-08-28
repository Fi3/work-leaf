#!/usr/bin/env python3
"""Verify real Codex app-server exact usage survives Work Leaf-style interruption."""

import argparse
import json
import os
import selectors
import subprocess
import sys
import time
from pathlib import Path


def parse_args():
    parser = argparse.ArgumentParser()
    parser.add_argument("--codex", required=True)
    parser.add_argument("--cwd", required=True)
    parser.add_argument("--transcript", required=True)
    parser.add_argument("--stderr", required=True)
    parser.add_argument("--summary", required=True)
    parser.add_argument("--timeout", type=int, default=300)
    parser.add_argument("--codex-version", required=True)
    parser.add_argument("--actual-codex-sha256", required=True)
    parser.add_argument("--profiled-codex-sha256", required=True)
    parser.add_argument(
        "--completion-mode",
        choices=(
            "interrupt-after-directive",
            "wait-for-completion",
            "interrupt-after-usage",
        ),
        required=True,
    )
    return parser.parse_args()


def main():
    args = parse_args()
    for path in (args.transcript, args.stderr, args.summary):
        if Path(path).exists():
            raise RuntimeError(f"refusing to replace existing evidence: {path}")

    process = subprocess.Popen(
        [args.codex, "app-server", "--listen", "stdio://"],
        cwd=args.cwd,
        stdin=subprocess.PIPE,
        stdout=subprocess.PIPE,
        stderr=open(args.stderr, "w", encoding="utf-8"),
        text=True,
        bufsize=1,
    )
    assert process.stdin is not None
    assert process.stdout is not None
    selector = selectors.DefaultSelector()
    selector.register(process.stdout, selectors.EVENT_READ)
    transcript = open(args.transcript, "w", encoding="utf-8")
    deadline = time.monotonic() + args.timeout
    messages = []

    def send(message):
        process.stdin.write(json.dumps(message, separators=(",", ":")) + "\n")
        process.stdin.flush()
        transcript.write(json.dumps({"direction": "client", "message": message}) + "\n")
        transcript.flush()

    def receive():
        remaining = deadline - time.monotonic()
        if remaining <= 0:
            raise TimeoutError("real exact-usage smoke timed out")
        if not selector.select(remaining):
            raise TimeoutError("real exact-usage smoke timed out")
        line = process.stdout.readline()
        if not line:
            raise RuntimeError(f"Codex app-server exited early with {process.poll()}")
        message = json.loads(line)
        transcript.write(json.dumps({"direction": "server", "message": message}) + "\n")
        transcript.flush()
        messages.append(message)
        if "id" in message and "method" in message:
            send({"id": message["id"], "result": {}})
        return message

    def wait_for_response(request_id):
        while True:
            message = receive()
            if str(message.get("id")) == str(request_id) and "method" not in message:
                if message.get("error") is not None:
                    raise RuntimeError(f"request {request_id} failed: {message['error']}")
                return message.get("result", {})

    directive_seen = False
    interrupted = False
    interrupt_acknowledged = False
    try:
        send(
            {
                "id": "1",
                "method": "initialize",
                "params": {
                    "clientInfo": {
                        "name": "work_leaf_benchmark_smoke",
                        "title": "Work Leaf benchmark smoke",
                        "version": "1",
                    },
                    "capabilities": {"experimentalApi": True},
                },
            }
        )
        wait_for_response("1")
        send({"method": "initialized"})
        send(
            {
                "id": "2",
                "method": "thread/start",
                "params": {
                    "approvalPolicy": "never",
                    "cwd": str(Path(args.cwd).resolve()),
                    "sandbox": "read-only",
                    "model": "gpt-5.5",
                    "experimentalRawEvents": True,
                },
            }
        )
        thread_result = wait_for_response("2")
        thread_id = thread_result["thread"]["id"]
        send(
            {
                "id": "3",
                "method": "turn/start",
                "params": {
                    "approvalPolicy": "never",
                    "cwd": str(Path(args.cwd).resolve()),
                    "sandboxPolicy": {"type": "readOnly"},
                    "threadId": thread_id,
                    "model": "gpt-5.5",
                    "input": [
                        {
                            "type": "text",
                            "text": "Reply exactly with @work-leaf done and do not modify files.",
                        }
                    ],
                },
            }
        )
        turn_result = wait_for_response("3")
        turn_id = turn_result["turn"]["id"]

        exact = []
        finished = False
        while not finished:
            message = receive()
            method = message.get("method")
            params = message.get("params", {})
            if (
                method == "rawResponse/completed"
                and params.get("threadId") == thread_id
                and params.get("turnId") == turn_id
            ):
                exact.append(params)
            if method == "item/completed":
                item = params.get("item", {})
                text = item.get("text", "")
                if item.get("type") == "agentMessage" and "@work-leaf done" in text:
                    directive_seen = True
                    if args.completion_mode == "interrupt-after-directive":
                        send(
                            {
                                "id": "4",
                                "method": "turn/interrupt",
                                "params": {"threadId": thread_id, "turnId": turn_id},
                            }
                        )
                        interrupted = True
            if (
                args.completion_mode == "interrupt-after-usage"
                and directive_seen
                and exact
                and not interrupted
            ):
                send(
                    {
                        "id": "4",
                        "method": "turn/interrupt",
                        "params": {"threadId": thread_id, "turnId": turn_id},
                    }
                )
                interrupted = True
            if (
                str(message.get("id")) == "4"
                and "method" not in message
                and interrupted
            ):
                if message.get("error") is not None:
                    raise RuntimeError(f"interrupt failed: {message['error']}")
                interrupt_acknowledged = True
                finished = True
            if method == "turn/completed" and params.get("turnId") == turn_id:
                finished = True
        if not directive_seen:
            raise RuntimeError("the smoke never observed the complete Work Leaf directive")
        if args.completion_mode != "wait-for-completion" and not interrupted:
            raise RuntimeError("the smoke never sent the Work Leaf-style interrupt")
        if not exact:
            raise RuntimeError("the interrupted turn emitted no exact response event")
        if any(event.get("usage") is None for event in exact):
            raise RuntimeError("the interrupted turn emitted null exact usage")

        usage_keys = (
            "inputTokens",
            "cachedInputTokens",
            "outputTokens",
            "reasoningOutputTokens",
        )
        totals = {
            key: sum(int(event["usage"].get(key, 0)) for event in exact)
            for key in usage_keys
        }
        summary = {
            "schema_version": 1,
            "result": "passed",
            "model": "gpt-5.5",
            "reasoning_effort": "xhigh",
            "completion_mode": args.completion_mode,
            "codex_cli_version": args.codex_version,
            "actual_codex_sha256": args.actual_codex_sha256,
            "profiled_codex_sha256": args.profiled_codex_sha256,
            "thread_id": thread_id,
            "turn_id": turn_id,
            "interrupt_sent": interrupted,
            "interrupt_acknowledged": interrupt_acknowledged,
            "exact_response_count": len(exact),
            "exact_usage": totals,
        }
        temporary = Path(args.summary + ".tmp")
        temporary.write_text(json.dumps(summary, indent=2, sort_keys=True) + "\n")
        os.replace(temporary, args.summary)
    finally:
        transcript.close()
        if process.stdin:
            process.stdin.close()
        process.terminate()
        try:
            process.wait(timeout=5)
        except subprocess.TimeoutExpired:
            process.kill()
            process.wait(timeout=5)


if __name__ == "__main__":
    try:
        main()
    except Exception as error:
        print(f"error: {error}", file=sys.stderr)
        raise
