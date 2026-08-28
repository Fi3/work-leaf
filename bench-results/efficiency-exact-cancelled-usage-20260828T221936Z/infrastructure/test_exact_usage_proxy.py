#!/usr/bin/env python3

import http.client
import json
import sys
import tempfile
import threading
import time
import unittest
from http.server import BaseHTTPRequestHandler, ThreadingHTTPServer
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))

from exact_usage_proxy import ExactUsageProxyServer, summarize_records


COMPLETED_USAGE = {
    "input_tokens": 100,
    "input_tokens_details": {"cached_tokens": 40},
    "output_tokens": 50,
    "output_tokens_details": {"reasoning_tokens": 20},
    "total_tokens": 150,
}

CANCELLED_USAGE = {
    "input_tokens": 200,
    "input_tokens_details": {"cached_tokens": 25},
    "output_tokens": 75,
    "output_tokens_details": {"reasoning_tokens": 30},
    "total_tokens": 275,
}


class FakeOpenAIHandler(BaseHTTPRequestHandler):
    protocol_version = "HTTP/1.0"

    def log_message(self, _format, *_args):
        pass

    def do_POST(self):
        if self.path == "/v1/responses":
            body = json.loads(self.rfile.read(int(self.headers["Content-Length"])))
            self.server.created_requests.append(body)
            response_id = "resp_complete" if body.get("metadata", {}).get("mode") == "complete" else "resp_cancel"
            self.send_response(200)
            self.send_header("Content-Type", "text/event-stream")
            self.end_headers()
            self._send_event(
                "response.created",
                {"type": "response.created", "response": {"id": response_id, "status": "in_progress"}},
            )
            if response_id == "resp_complete":
                self._send_event(
                    "response.completed",
                    {
                        "type": "response.completed",
                        "response": {
                            "id": response_id,
                            "status": "completed",
                            "usage": COMPLETED_USAGE,
                        },
                    },
                )
                return

            sequence = 0
            while not self.server.cancel_seen.is_set():
                try:
                    self._send_event(
                        "response.output_text.delta",
                        {"type": "response.output_text.delta", "delta": str(sequence)},
                    )
                except (BrokenPipeError, ConnectionResetError):
                    self.server.stream_closed.set()
                    return
                sequence += 1
                time.sleep(0.01)
            self.server.stream_closed.set()
            return

        if self.path == "/v1/responses/resp_cancel/cancel":
            self.server.cancel_saw_closed_stream = self.server.stream_closed.wait(timeout=1.0)
            self.server.cancel_seen.set()
            self._send_json({"id": "resp_cancel", "status": "cancelled", "usage": None})
            return

        self.send_error(404)

    def do_GET(self):
        if self.path.startswith("/v1/models"):
            self.server.models_authorization = self.headers.get("Authorization")
            self._send_json({"object": "list", "data": [{"id": "gpt-5.5"}]})
            return
        if self.path != "/v1/responses/resp_cancel":
            self.send_error(404)
            return
        self.server.retrieve_count += 1
        usage = CANCELLED_USAGE if self.server.retrieve_count >= 2 else None
        self._send_json({"id": "resp_cancel", "status": "cancelled", "usage": usage})

    def _send_event(self, event_type, payload):
        data = f"event: {event_type}\ndata: {json.dumps(payload, separators=(',', ':'))}\n\n"
        self.wfile.write(data.encode())
        self.wfile.flush()

    def _send_json(self, payload):
        data = json.dumps(payload, separators=(",", ":")).encode()
        self.send_response(200)
        self.send_header("Content-Type", "application/json")
        self.send_header("Content-Length", str(len(data)))
        self.end_headers()
        self.wfile.write(data)


class FakeOpenAIServer(ThreadingHTTPServer):
    def __init__(self):
        super().__init__(("127.0.0.1", 0), FakeOpenAIHandler)
        self.created_requests = []
        self.cancel_seen = threading.Event()
        self.stream_closed = threading.Event()
        self.cancel_saw_closed_stream = False
        self.retrieve_count = 0
        self.models_authorization = None


class ExactUsageProxyTest(unittest.TestCase):
    def setUp(self):
        self.temp_dir = tempfile.TemporaryDirectory()
        self.records = Path(self.temp_dir.name) / "responses.jsonl"
        self.upstream = FakeOpenAIServer()
        self.upstream_thread = threading.Thread(target=self.upstream.serve_forever, daemon=True)
        self.upstream_thread.start()
        upstream_url = f"http://127.0.0.1:{self.upstream.server_port}/v1"
        self.proxy = ExactUsageProxyServer(
            ("127.0.0.1", 0),
            upstream_url=upstream_url,
            record_path=self.records,
            poll_interval_seconds=0.01,
            poll_timeout_seconds=2.0,
        )
        self.proxy_thread = threading.Thread(target=self.proxy.serve_forever, daemon=True)
        self.proxy_thread.start()

    def tearDown(self):
        self.proxy.shutdown()
        self.proxy.server_close()
        self.upstream.shutdown()
        self.upstream.server_close()
        self.temp_dir.cleanup()

    def test_completed_and_disconnected_responses_have_exact_usage(self):
        completed = self._post({"model": "gpt-5.5", "stream": True, "metadata": {"mode": "complete"}})
        self.assertIn(b"response.completed", completed)

        connection = http.client.HTTPConnection("127.0.0.1", self.proxy.server_port, timeout=2)
        body = json.dumps({"model": "gpt-5.5", "stream": True}).encode()
        connection.request(
            "POST",
            "/v1/responses",
            body=body,
            headers={"Authorization": "Bearer test-key", "Content-Type": "application/json"},
        )
        response = connection.getresponse()
        self.assertEqual(response.status, 200)
        created_event = b"".join(response.readline() for _ in range(3))
        self.assertIn(b"response.created", created_event)
        self.assertIn(b"resp_cancel", created_event)
        response.close()
        connection.close()

        records = self._wait_for_records(2)
        by_id = {record["response_id"]: record for record in records}
        self.assertEqual(by_id["resp_complete"]["termination"], "completed")
        self.assertIsNone(by_id["resp_complete"]["recovery_trigger"])
        self.assertEqual(by_id["resp_complete"]["usage"], COMPLETED_USAGE)
        self.assertEqual(by_id["resp_cancel"]["termination"], "cancelled")
        self.assertEqual(by_id["resp_cancel"]["recovery_trigger"], "downstream_disconnected")
        self.assertEqual(by_id["resp_cancel"]["usage"], CANCELLED_USAGE)
        self.assertEqual(
            by_id["resp_cancel"]["recovery_steps"],
            ["upstream_stream_closed", "cancel_requested", "usage_retrieved"],
        )
        self.assertTrue(self.upstream.cancel_saw_closed_stream)

        for request in self.upstream.created_requests:
            self.assertTrue(request["background"])
            self.assertTrue(request["store"])
            self.assertTrue(request["stream"])

        summary = summarize_records(self.records)
        self.assertTrue(summary["complete"])
        self.assertEqual(summary["response_count"], 2)
        self.assertEqual(summary["input_tokens"], 300)
        self.assertEqual(summary["cached_input_tokens"], 65)
        self.assertEqual(summary["output_tokens"], 125)
        self.assertEqual(summary["reasoning_output_tokens"], 50)
        self.assertEqual(summary["raw_input_plus_output"], 425)
        self.assertEqual(summary["uncached_input_plus_output"], 360)

    def test_summary_rejects_a_response_without_usage(self):
        self.records.write_text(
            json.dumps(
                {
                    "schema_version": 1,
                    "response_id": "resp_missing",
                    "termination": "recovery-failed",
                    "usage": None,
                }
            )
            + "\n"
        )
        summary = summarize_records(self.records)
        self.assertFalse(summary["complete"])
        self.assertEqual(summary["missing_usage_response_ids"], ["resp_missing"])

    def test_models_endpoint_is_forwarded_without_becoming_usage(self):
        connection = http.client.HTTPConnection("127.0.0.1", self.proxy.server_port, timeout=2)
        connection.request(
            "GET",
            "/v1/models?client_version=test",
            headers={"Authorization": "Bearer test-key"},
        )
        response = connection.getresponse()
        payload = json.loads(response.read())
        connection.close()
        self.assertEqual(response.status, 200)
        self.assertEqual(payload["data"][0]["id"], "gpt-5.5")
        self.assertEqual(self.upstream.models_authorization, "Bearer test-key")
        self.assertFalse(self.records.exists())

    def _post(self, payload):
        connection = http.client.HTTPConnection("127.0.0.1", self.proxy.server_port, timeout=2)
        body = json.dumps(payload).encode()
        connection.request(
            "POST",
            "/v1/responses",
            body=body,
            headers={"Authorization": "Bearer test-key", "Content-Type": "application/json"},
        )
        response = connection.getresponse()
        data = response.read()
        connection.close()
        self.assertEqual(response.status, 200)
        return data

    def _wait_for_records(self, count):
        deadline = time.monotonic() + 3
        while time.monotonic() < deadline:
            if self.records.exists():
                records = [json.loads(line) for line in self.records.read_text().splitlines()]
                if len(records) >= count:
                    return records
            time.sleep(0.01)
        self.fail(f"timed out waiting for {count} proxy records")


if __name__ == "__main__":
    unittest.main()
