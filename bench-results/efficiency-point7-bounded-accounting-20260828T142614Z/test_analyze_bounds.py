import json
import tempfile
import unittest
from pathlib import Path

from analyze_bounds import (
    audit_app_server,
    condition_bound,
    missing_usage_cap_audit,
    usage_window_evidence,
)


def write_jsonl(path: Path, rows: list[dict]) -> None:
    path.write_text("".join(json.dumps(row) + "\n" for row in rows))


class AppServerAuditTest(unittest.TestCase):
    def test_classifies_every_started_turn(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            client = root / "client.jsonl"
            server = root / "server.jsonl"
            write_jsonl(
                client,
                [
                    {
                        "id": 1,
                        "method": "turn/start",
                        "params": {"threadId": "thread-1"},
                    },
                    {
                        "id": 2,
                        "method": "turn/start",
                        "params": {"threadId": "thread-1"},
                    },
                    {
                        "id": 3,
                        "method": "turn/interrupt",
                        "params": {"threadId": "thread-1", "turnId": "turn-2"},
                    },
                ],
            )
            write_jsonl(
                server,
                [
                    {"id": 1, "result": {"turn": {"id": "turn-1"}}},
                    {"id": 2, "result": {"turn": {"id": "turn-2"}}},
                    {
                        "method": "turn/completed",
                        "params": {
                            "threadId": "thread-1",
                            "turn": {"id": "turn-1", "status": "completed"},
                        },
                    },
                    {
                        "method": "turn/completed",
                        "params": {
                            "threadId": "thread-1",
                            "turn": {"id": "turn-2", "status": "interrupted"},
                        },
                    },
                ],
            )

            self.assertEqual(
                audit_app_server(client, server),
                {
                    "started_turns": 2,
                    "completed_turns": 1,
                    "interrupted_turns": 1,
                    "interrupted_prompt_json_bytes": 2,
                    "rpc_errors": 0,
                },
            )

    def test_rejects_an_unclassified_started_turn(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            client = root / "client.jsonl"
            server = root / "server.jsonl"
            write_jsonl(
                client,
                [
                    {
                        "id": 1,
                        "method": "turn/start",
                        "params": {"threadId": "thread-1"},
                    }
                ],
            )
            write_jsonl(server, [{"id": 1, "result": {"turn": {"id": "turn-1"}}}])

            with self.assertRaisesRegex(ValueError, "started/completed turn mismatch"):
                audit_app_server(client, server)

    def test_rejects_an_interruption_without_matching_client_request(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            client = root / "client.jsonl"
            server = root / "server.jsonl"
            write_jsonl(
                client,
                [
                    {
                        "id": 1,
                        "method": "turn/start",
                        "params": {"threadId": "thread-1"},
                    }
                ],
            )
            write_jsonl(
                server,
                [
                    {"id": 1, "result": {"turn": {"id": "turn-1"}}},
                    {
                        "method": "turn/completed",
                        "params": {
                            "threadId": "thread-1",
                            "turn": {"id": "turn-1", "status": "interrupted"},
                        },
                    },
                ],
            )

            with self.assertRaisesRegex(ValueError, "interrupt request/outcome mismatch"):
                audit_app_server(client, server)

    def test_requires_one_consistent_usage_window(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            server = Path(directory) / "server.jsonl"
            write_jsonl(
                server,
                [
                    {
                        "method": "thread/tokenUsage/updated",
                        "params": {
                            "tokenUsage": {"modelContextWindow": 258_400}
                        },
                    },
                    {
                        "method": "thread/tokenUsage/updated",
                        "params": {
                            "tokenUsage": {"modelContextWindow": 258_400}
                        },
                    },
                ],
            )

            self.assertEqual(
                usage_window_evidence(server),
                {"usage_events": 2, "effective_context_window": 258_400},
            )


class BoundTest(unittest.TestCase):
    def test_computes_a_conservative_minimum_reduction(self) -> None:
        self.assertEqual(
            condition_bound(
                direct_raw_tokens=1_000,
                observed_raw_tokens=200,
                interrupted_turns=2,
                maximum_tokens_per_interrupted_turn=300,
            ),
            {
                "observed_raw_tokens": 200,
                "interrupted_turns": 2,
                "maximum_tokens_per_interrupted_turn": 300,
                "raw_token_upper_bound": 800,
                "minimum_raw_tokens_saved": 200,
                "minimum_raw_reduction_percent": 20.0,
            },
        )

    def test_reports_aggregate_prompt_headroom(self) -> None:
        self.assertEqual(
            missing_usage_cap_audit(
                interrupted_turns=2,
                interrupted_prompt_json_bytes=100,
                effective_context_window=250,
                maximum_output_tokens=100,
                declared_tokens_per_interruption=400,
            ),
            {
                "declared_missing_raw_token_cap": 800,
                "context_output_and_prompt_upper_bound": 800,
                "remaining_headroom": 0,
            },
        )


if __name__ == "__main__":
    unittest.main()
