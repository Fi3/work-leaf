"""Fail-first checks for a tool-free local initialization-order fixture."""
import unittest
from unittest.mock import patch

import empty_startup_mcp as server


class EmptyMcpTests(unittest.TestCase):
    def test_initialization_wait_has_no_instructions_or_capabilities(self):
        with patch.object(server.time, "sleep") as wait:
            reply = server.reply({"jsonrpc": "2.0", "id": 1, "method": "initialize",
                                  "params": {"protocolVersion": "2025-03-26"}})
        wait.assert_called_once_with(2)
        self.assertEqual(reply["result"]["capabilities"], {})
        self.assertEqual(reply["result"]["protocolVersion"], "2025-03-26")
        self.assertNotIn("instructions", reply["result"])

    def test_fixture_advertises_no_tools_resources_or_prompts(self):
        for method, key in (("tools/list", "tools"), ("resources/list", "resources"),
                            ("resources/templates/list", "resourceTemplates"),
                            ("prompts/list", "prompts")):
            self.assertEqual(server.reply({"id": 2, "method": method})["result"], {key: []})

    def test_notification_has_no_reply(self):
        self.assertIsNone(server.reply({"method": "notifications/initialized"}))

    def test_unknown_request_is_rejected(self):
        self.assertEqual(server.reply({"id": 3, "method": "tools/call"})["error"]["code"], -32601)


if __name__ == "__main__":
    unittest.main()
