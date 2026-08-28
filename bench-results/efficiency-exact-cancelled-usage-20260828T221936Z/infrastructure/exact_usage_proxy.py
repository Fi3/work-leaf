#!/usr/bin/env python3

import argparse
import hashlib
import http.client
import json
import os
import signal
import socket
import sys
import threading
import time
import uuid
from collections import Counter
from http.server import BaseHTTPRequestHandler, ThreadingHTTPServer
from pathlib import Path
from urllib.parse import urlsplit


SCHEMA_VERSION = 1
FORWARDED_HEADERS = (
    "authorization",
    "content-type",
    "openai-organization",
    "openai-project",
    "user-agent",
    "x-stainless-arch",
    "x-stainless-lang",
    "x-stainless-os",
    "x-stainless-package-version",
    "x-stainless-runtime",
    "x-stainless-runtime-version",
)
TERMINAL_EVENTS = {
    "response.completed": "completed",
    "response.failed": "failed",
    "response.incomplete": "incomplete",
    "response.cancelled": "cancelled",
}


def _unix_ns():
    return time.time_ns()


def _compact_json(value):
    return json.dumps(value, separators=(",", ":"), sort_keys=True)


class ExactUsageProxyServer(ThreadingHTTPServer):
    daemon_threads = False
    block_on_close = True
    allow_reuse_address = True

    def __init__(
        self,
        address,
        *,
        upstream_url,
        record_path,
        poll_interval_seconds=1.0,
        poll_timeout_seconds=180.0,
    ):
        super().__init__(address, ExactUsageProxyHandler)
        parsed = urlsplit(upstream_url)
        if parsed.scheme not in {"http", "https"} or not parsed.hostname:
            raise ValueError(f"invalid upstream URL: {upstream_url}")
        self.upstream_scheme = parsed.scheme
        self.upstream_host = parsed.hostname
        self.upstream_port = parsed.port
        self.upstream_path = parsed.path.rstrip("/")
        self.record_path = Path(record_path)
        self.request_path = self.record_path.with_name("requests.jsonl")
        self.poll_interval_seconds = poll_interval_seconds
        self.poll_timeout_seconds = poll_timeout_seconds
        self.record_lock = threading.Lock()
        self.record_path.parent.mkdir(parents=True, exist_ok=True)

    def upstream_connection(self):
        connection_type = (
            http.client.HTTPSConnection if self.upstream_scheme == "https" else http.client.HTTPConnection
        )
        return connection_type(self.upstream_host, self.upstream_port, timeout=60)

    def upstream_endpoint(self, suffix):
        return f"{self.upstream_path}{suffix}"

    def append_request_event(self, record):
        self._append_json(self.request_path, record)

    def append_final_record(self, record):
        self._append_json(self.record_path, record)

    def _append_json(self, path, record):
        data = (_compact_json(record) + "\n").encode()
        with self.record_lock:
            descriptor = os.open(path, os.O_WRONLY | os.O_CREAT | os.O_APPEND, 0o600)
            try:
                os.write(descriptor, data)
                os.fsync(descriptor)
            finally:
                os.close(descriptor)


class ExactUsageProxyHandler(BaseHTTPRequestHandler):
    protocol_version = "HTTP/1.0"
    server_version = "work-leaf-exact-usage-proxy/1"

    def log_message(self, _format, *_args):
        pass

    def do_GET(self):
        if self.path == "/health":
            self._send_json(200, {"status": "ok"})
            return
        parsed = urlsplit(self.path)
        if parsed.path.rstrip("/") in {"/models", "/v1/models"}:
            self._forward_models(parsed.query)
            return
        self.send_error(404)

    def do_POST(self):
        if self.path.rstrip("/") not in {"/responses", "/v1/responses"}:
            self.send_error(404)
            return
        self._handle_response_create()

    def _handle_response_create(self):
        request_id = f"request-{uuid.uuid4()}"
        try:
            length = int(self.headers.get("Content-Length", ""))
            if length <= 0:
                raise ValueError("missing or invalid Content-Length")
            raw_body = self.rfile.read(length)
            payload = json.loads(raw_body)
            if not isinstance(payload, dict):
                raise ValueError("request body must be a JSON object")
            if payload.get("stream") is not True:
                raise ValueError("exact-usage benchmark requires stream=true")
        except (ValueError, json.JSONDecodeError) as error:
            self._send_json(400, {"error": {"message": str(error)}})
            return

        original_background = payload.get("background")
        original_store = payload.get("store")
        payload["background"] = True
        payload["store"] = True
        body = _compact_json(payload).encode()
        self.server.append_request_event(
            {
                "schema_version": SCHEMA_VERSION,
                "event": "request_started",
                "request_id": request_id,
                "started_unix_ns": _unix_ns(),
                "request_sha256": hashlib.sha256(body).hexdigest(),
                "model": payload.get("model"),
                "reasoning_effort": (payload.get("reasoning") or {}).get("effort"),
                "stream": True,
                "original_background": original_background,
                "original_store": original_store,
                "effective_background": True,
                "effective_store": True,
            }
        )

        upstream = self.server.upstream_connection()
        response = None
        response_id = None
        terminal_payload = None
        disconnected = False
        upstream_error = None
        try:
            upstream.request(
                "POST",
                self.server.upstream_endpoint("/responses"),
                body=body,
                headers=self._upstream_headers(len(body)),
            )
            response = upstream.getresponse()
            if response.status != 200:
                error_body = response.read()
                self._forward_fixed_response(response.status, response.getheaders(), error_body)
                self._record_failure(request_id, None, f"upstream HTTP {response.status}")
                return

            self.send_response(200)
            self.send_header("Content-Type", response.getheader("Content-Type", "text/event-stream"))
            self.send_header("Cache-Control", "no-cache")
            self.send_header("Connection", "close")
            self.end_headers()

            event_lines = []
            while True:
                line = response.readline()
                if not line:
                    break
                event_lines.append(line)
                try:
                    self.wfile.write(line)
                    self.wfile.flush()
                except (BrokenPipeError, ConnectionResetError, socket.error):
                    disconnected = True
                    break
                if line not in {b"\n", b"\r\n"}:
                    continue
                event = self._parse_sse_event(event_lines)
                event_lines = []
                if event is None:
                    continue
                event_type, event_payload = event
                candidate_id = (event_payload.get("response") or {}).get("id")
                if candidate_id:
                    response_id = candidate_id
                if event_type == "response.created" and response_id:
                    self.server.append_request_event(
                        {
                            "schema_version": SCHEMA_VERSION,
                            "event": "response_identified",
                            "request_id": request_id,
                            "response_id": response_id,
                            "recorded_unix_ns": _unix_ns(),
                        }
                    )
                if event_type in TERMINAL_EVENTS:
                    terminal_payload = event_payload.get("response") or {}
                    break
        except (OSError, http.client.HTTPException) as error:
            upstream_error = str(error)
        finally:
            if response is not None:
                response.close()
            upstream.close()

        if terminal_payload is not None and terminal_payload.get("usage") is not None:
            self._record_final(
                request_id=request_id,
                response_id=response_id,
                termination=TERMINAL_EVENTS.get(event_type, terminal_payload.get("status", "terminal")),
                usage=terminal_payload["usage"],
                model=terminal_payload.get("model"),
                recovery_trigger=None,
                recovery_steps=[],
                error=None,
            )
            return

        if response_id is None:
            reason = upstream_error or "stream ended before response.created"
            self._record_failure(request_id, None, reason)
            return

        recovery_steps = ["upstream_stream_closed"]
        usage, status, model, recovery_error = self._recover_usage(
            response_id,
            request_id,
            recovery_steps,
            request_cancel=disconnected or terminal_payload is None,
        )
        termination = status or ("downstream-disconnected" if disconnected else "stream-ended")
        recovery_trigger = (
            "downstream_disconnected"
            if disconnected
            else "terminal_without_usage"
            if terminal_payload is not None
            else "upstream_stream_ended"
        )
        self._record_final(
            request_id=request_id,
            response_id=response_id,
            termination=termination,
            usage=usage,
            model=model,
            recovery_trigger=recovery_trigger,
            recovery_steps=recovery_steps,
            error=recovery_error or upstream_error,
        )

    def _recover_usage(self, response_id, request_id, recovery_steps, *, request_cancel):
        if request_cancel:
            cancel = self.server.upstream_connection()
            try:
                cancel.request(
                    "POST",
                    self.server.upstream_endpoint(f"/responses/{response_id}/cancel"),
                    body=b"",
                    headers=self._upstream_headers(0),
                )
                response = cancel.getresponse()
                body = response.read()
                if response.status not in {200, 400}:
                    return None, None, None, f"cancel returned HTTP {response.status}"
                if response.status == 400:
                    error = json.loads(body or b"{}").get("error", {}).get("message", "")
                    if "completed" not in error.lower():
                        return None, None, None, f"cancel returned HTTP 400: {error}"
                recovery_steps.append("cancel_requested")
                self.server.append_request_event(
                    {
                        "schema_version": SCHEMA_VERSION,
                        "event": "cancel_requested",
                        "request_id": request_id,
                        "response_id": response_id,
                        "recorded_unix_ns": _unix_ns(),
                    }
                )
            except (OSError, http.client.HTTPException, json.JSONDecodeError) as error:
                return None, None, None, f"cancel failed: {error}"
            finally:
                cancel.close()

        deadline = time.monotonic() + self.server.poll_timeout_seconds
        last_status = None
        last_model = None
        last_error = None
        while time.monotonic() < deadline:
            retrieve = self.server.upstream_connection()
            try:
                retrieve.request(
                    "GET",
                    self.server.upstream_endpoint(f"/responses/{response_id}"),
                    headers=self._upstream_headers(None),
                )
                response = retrieve.getresponse()
                body = response.read()
                if response.status != 200:
                    last_error = f"retrieve returned HTTP {response.status}"
                else:
                    payload = json.loads(body)
                    last_status = payload.get("status")
                    last_model = payload.get("model")
                    if payload.get("usage") is not None:
                        recovery_steps.append("usage_retrieved")
                        return payload["usage"], last_status, last_model, None
            except (OSError, http.client.HTTPException, json.JSONDecodeError) as error:
                last_error = f"retrieve failed: {error}"
            finally:
                retrieve.close()
            time.sleep(self.server.poll_interval_seconds)
        return None, last_status, last_model, last_error or "usage did not appear before poll timeout"

    def _upstream_headers(self, content_length):
        headers = {}
        for name in FORWARDED_HEADERS:
            value = self.headers.get(name)
            if value is not None:
                headers[name] = value
        headers.setdefault("content-type", "application/json")
        if content_length is not None:
            headers["content-length"] = str(content_length)
        return headers

    def _forward_models(self, query):
        upstream = self.server.upstream_connection()
        endpoint = self.server.upstream_endpoint("/models")
        if query:
            endpoint = f"{endpoint}?{query}"
        try:
            upstream.request("GET", endpoint, headers=self._upstream_headers(None))
            response = upstream.getresponse()
            body = response.read()
            self._forward_fixed_response(response.status, response.getheaders(), body)
        except (OSError, http.client.HTTPException) as error:
            self._send_json(502, {"error": {"message": f"model-list proxy failed: {error}"}})
        finally:
            upstream.close()

    def _record_failure(self, request_id, response_id, error):
        self._record_final(
            request_id=request_id,
            response_id=response_id or request_id,
            termination="recovery-failed",
            usage=None,
            model=None,
            recovery_trigger="request_failed_before_response_id",
            recovery_steps=[],
            error=error,
        )

    def _record_final(
        self,
        *,
        request_id,
        response_id,
        termination,
        usage,
        model,
        recovery_trigger,
        recovery_steps,
        error,
    ):
        self.server.append_final_record(
            {
                "schema_version": SCHEMA_VERSION,
                "request_id": request_id,
                "response_id": response_id,
                "finished_unix_ns": _unix_ns(),
                "termination": termination,
                "model": model,
                "usage": usage,
                "recovery_trigger": recovery_trigger,
                "recovery_steps": recovery_steps,
                "error": error,
            }
        )

    @staticmethod
    def _parse_sse_event(lines):
        event_type = None
        data = []
        for raw_line in lines:
            line = raw_line.decode("utf-8", errors="replace").rstrip("\r\n")
            if line.startswith("event:"):
                event_type = line[6:].strip()
            elif line.startswith("data:"):
                data.append(line[5:].strip())
        if not data or data == ["[DONE]"]:
            return None
        try:
            payload = json.loads("\n".join(data))
        except json.JSONDecodeError:
            return None
        if not event_type:
            event_type = payload.get("type")
        return event_type, payload

    def _forward_fixed_response(self, status, headers, body):
        self.send_response(status)
        for name, value in headers:
            if name.lower() in {"content-type", "openai-request-id"}:
                self.send_header(name, value)
        self.send_header("Content-Length", str(len(body)))
        self.end_headers()
        self.wfile.write(body)

    def _send_json(self, status, payload):
        body = _compact_json(payload).encode()
        self.send_response(status)
        self.send_header("Content-Type", "application/json")
        self.send_header("Content-Length", str(len(body)))
        self.end_headers()
        self.wfile.write(body)


def _read_json_lines(path):
    if not path.exists():
        return []
    records = []
    for line_number, line in enumerate(path.read_text().splitlines(), start=1):
        if not line.strip():
            continue
        try:
            records.append(json.loads(line))
        except json.JSONDecodeError as error:
            raise ValueError(f"invalid JSON in {path}:{line_number}: {error}") from error
    return records


def summarize_records(record_path):
    record_path = Path(record_path)
    records = _read_json_lines(record_path)
    request_path = record_path.with_name("requests.jsonl")
    request_events = _read_json_lines(request_path)
    started = {
        event["request_id"]
        for event in request_events
        if event.get("event") == "request_started" and event.get("request_id")
    }
    finalized = {record.get("request_id") for record in records if record.get("request_id")}
    missing_final_records = sorted(started - finalized)
    response_ids = [record.get("response_id") for record in records]
    response_id_counts = Counter(response_ids)
    duplicate_response_ids = sorted(
        response_id for response_id, count in response_id_counts.items() if response_id and count > 1
    )
    missing_usage = sorted(
        str(record.get("response_id")) for record in records if record.get("usage") is None
    )

    totals = {
        "input_tokens": 0,
        "cached_input_tokens": 0,
        "output_tokens": 0,
        "reasoning_output_tokens": 0,
    }
    invalid_usage_response_ids = []
    for record in records:
        usage = record.get("usage")
        if usage is None:
            continue
        try:
            input_tokens = _nonnegative_int(usage["input_tokens"])
            output_tokens = _nonnegative_int(usage["output_tokens"])
            cached_tokens = _nonnegative_int(
                (usage.get("input_tokens_details") or {}).get("cached_tokens", 0)
            )
            reasoning_tokens = _nonnegative_int(
                (usage.get("output_tokens_details") or {}).get("reasoning_tokens", 0)
            )
            total_tokens = _nonnegative_int(usage["total_tokens"])
            if total_tokens != input_tokens + output_tokens:
                raise ValueError("total_tokens does not equal input_tokens plus output_tokens")
            if cached_tokens > input_tokens or reasoning_tokens > output_tokens:
                raise ValueError("usage detail exceeds its parent total")
        except (KeyError, TypeError, ValueError):
            invalid_usage_response_ids.append(str(record.get("response_id")))
            continue
        totals["input_tokens"] += input_tokens
        totals["cached_input_tokens"] += cached_tokens
        totals["output_tokens"] += output_tokens
        totals["reasoning_output_tokens"] += reasoning_tokens

    complete = bool(records) and not (
        missing_final_records
        or duplicate_response_ids
        or missing_usage
        or invalid_usage_response_ids
    )
    return {
        "schema_version": SCHEMA_VERSION,
        "complete": complete,
        "response_count": len(records),
        "started_request_count": len(started) if request_path.exists() else None,
        "missing_final_request_ids": missing_final_records,
        "duplicate_response_ids": duplicate_response_ids,
        "missing_usage_response_ids": missing_usage,
        "invalid_usage_response_ids": sorted(invalid_usage_response_ids),
        **totals,
        "uncached_input_tokens": totals["input_tokens"] - totals["cached_input_tokens"],
        "raw_input_plus_output": totals["input_tokens"] + totals["output_tokens"],
        "uncached_input_plus_output": (
            totals["input_tokens"] - totals["cached_input_tokens"] + totals["output_tokens"]
        ),
    }


def _nonnegative_int(value):
    if isinstance(value, bool) or not isinstance(value, int) or value < 0:
        raise ValueError("expected a non-negative integer")
    return value


def _parse_listen(value):
    host, separator, port = value.rpartition(":")
    if not separator or not host:
        raise ValueError("listen address must be HOST:PORT")
    return host, int(port)


def _serve(args):
    server = ExactUsageProxyServer(
        _parse_listen(args.listen),
        upstream_url=args.upstream,
        record_path=args.records,
        poll_interval_seconds=args.poll_interval_seconds,
        poll_timeout_seconds=args.poll_timeout_seconds,
    )
    if args.ready_file:
        ready_path = Path(args.ready_file)
        ready_path.parent.mkdir(parents=True, exist_ok=True)
        temporary = ready_path.with_name(f".{ready_path.name}.{os.getpid()}.tmp")
        temporary.write_text(
            _compact_json(
                {
                    "schema_version": SCHEMA_VERSION,
                    "pid": os.getpid(),
                    "host": server.server_address[0],
                    "port": server.server_port,
                    "base_url": f"http://{server.server_address[0]}:{server.server_port}/v1",
                    "records": str(Path(args.records).resolve()),
                }
            )
            + "\n"
        )
        os.chmod(temporary, 0o600)
        os.replace(temporary, ready_path)

    def stop_server(_signum, _frame):
        threading.Thread(target=server.shutdown, daemon=True).start()

    signal.signal(signal.SIGTERM, stop_server)
    signal.signal(signal.SIGINT, stop_server)
    try:
        server.serve_forever(poll_interval=0.1)
    finally:
        server.server_close()


def main():
    parser = argparse.ArgumentParser()
    subparsers = parser.add_subparsers(dest="command", required=True)
    serve = subparsers.add_parser("serve")
    serve.add_argument("--listen", default="127.0.0.1:0")
    serve.add_argument("--upstream", default="https://api.openai.com/v1")
    serve.add_argument("--records", required=True)
    serve.add_argument("--ready-file")
    serve.add_argument("--poll-interval-seconds", type=float, default=1.0)
    serve.add_argument("--poll-timeout-seconds", type=float, default=180.0)
    serve.set_defaults(action=_serve)
    summarize = subparsers.add_parser("summarize")
    summarize.add_argument("--records", required=True)
    summarize.set_defaults(action=lambda args: print(_compact_json(summarize_records(args.records))))
    args = parser.parse_args()
    result = args.action(args)
    return 0 if result is None else result


if __name__ == "__main__":
    sys.exit(main())
