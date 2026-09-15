"""Verify the exact retained WL RPC envelope without a provider call."""
import sys
from pathlib import Path
import unittest
from unittest import mock
sys.path.insert(0, str(Path(__file__).resolve().parent))
import wl_input_probe as route

class WlInputTests(unittest.TestCase):
    def test_original_transport_restores_wl_envelope_only(self):
        p = route.WlProbe("/provider", "/repo")
        self.assertEqual(p.command, ["/provider", "app-server", "--listen", "stdio://"])
        cases = [
            ("initialize", {"clientInfo":{},"capabilities":{}},
             {"clientInfo":{"name":"work_leaf","title":"Work Leaf","version":"0.1.3"},
              "capabilities":{"experimentalApi":True,"optOutNotificationMethods":["rawResponseItem/completed"]}}),
            ("thread/start", {"modelProvider":"openai","model":"gpt-5.5","experimentalRawEvents":True},
             {"model":"gpt-5.5","experimentalRawEvents":True}),
            ("turn/start", {"effort":"xhigh","threadId":"owned","input":[{"type":"text","text":"x"}]},
             {"threadId":"owned","input":[{"type":"text","text":"x"}]}),
        ]
        with mock.patch.object(route.previous.ContinuityProbe,"request",return_value={}) as request:
            for method, params, expected in cases:
                p.request("1", method, params)
                request.assert_called_with("1", method, expected)
        with self.assertRaises(ValueError):
            p.run_connection("one", "two", "existing-thread")
        p.evidence["turns"] = [{},{}]
        with self.assertRaises(ValueError):
            p.run_turn("third")
        p.evidence["turns"] = []
        with mock.patch.object(route.previous.ContinuityProbe,"run_turn") as turn:
            p.run_turn("first")
            turn.assert_called_once_with("first")

if __name__ == "__main__":
    unittest.main()
