"""Private route accepts only the frozen profile and app-server shape."""
import sys
from pathlib import Path
import unittest
sys.path.insert(0, str(Path(__file__).resolve().parent))
import appserver_provider as route

class RouteTests(unittest.TestCase):
    def test_exact_profile_prefix_and_stdio_only(self):
        args = ["-c", 'model="gpt-5.5"', "-c",
                'model_reasoning_effort="xhigh"', "app-server", "--listen", "stdio://"]
        self.assertEqual(route.classify(args), "app-server")
        self.assertEqual(route.classify(["app-server", "--listen", "stdio://"]), "app-server")
        self.assertEqual(route.classify(args[:4] + ["--version"]), "version")
        for bad in (["exec"], ["app-server"], ["app-server", "--listen", "tcp://localhost:1"],
                    ["-c", 'model="other"', "--version"], args+["--extra"],
                    ["--", "app-server", "--listen", "stdio://"], ["-c"]):
            with self.subTest(args=bad), self.assertRaises(ValueError):
                route.classify(bad)


class RouteRejectionTests(unittest.TestCase):
    def test_missing_opt_in_and_outer_guard_never_spawn(self):
        from unittest import mock
        import os
        with mock.patch.object(sys, "argv", ["route","app-server","--listen","stdio://"]), \
                mock.patch.object(os, "execvp") as spawn:
            for env in ({}, {"WORK_LEAF_BENCH_P06_INPUT":"1"}):
                with mock.patch.dict(os.environ, env, clear=True), self.assertRaises(ValueError):
                    route.main()
            spawn.assert_not_called()

    def test_stale_plan_or_reused_receipt_never_spawn(self):
        from unittest import mock
        import os, tempfile, json, hashlib
        with tempfile.TemporaryDirectory() as tmp:
            root=Path(tmp); source=root/"source"; source.write_text("pin")
            receipt=root/"receipt"; receipt.write_text("old")
            plan=root/"plan.json"
            plan.write_text(json.dumps({"schema":"work-leaf-p06-appserver-input-v1",
                "sources":{str(source):hashlib.sha256(b"pin").hexdigest()},
                "provider":sys.executable,"view":{}}))
            env={"WORK_LEAF_BENCH_P06_INPUT":"1","WORK_LEAF_BENCH_CODEX_ACTIVE":"1",
                "WORK_LEAF_BENCH_P06_INPUT_PLAN":str(plan),
                "WORK_LEAF_BENCH_P06_INPUT_PLAN_SHA256":hashlib.sha256(plan.read_bytes()).hexdigest(),
                "WORK_LEAF_BENCH_P06_INPUT_RECEIPT":str(receipt)}
            with mock.patch.object(sys,"argv",["route","app-server","--listen","stdio://"]), \
                    mock.patch.dict(os.environ, env, clear=True), mock.patch.object(os,"execvp") as spawn:
                with self.assertRaises(FileExistsError): route.main()
                source.write_text("changed")
                with self.assertRaises(ValueError): route.main()
                os.environ["WORK_LEAF_BENCH_P06_INPUT_PLAN_SHA256"]="0"*64
                with self.assertRaises(ValueError): route.main()
                spawn.assert_not_called()

if __name__ == "__main__":
    unittest.main()
