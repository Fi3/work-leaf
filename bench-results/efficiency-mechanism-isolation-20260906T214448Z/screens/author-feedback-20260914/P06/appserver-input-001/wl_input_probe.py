"""Two tiny turns through the saved WL RPC shape; no workflow or tool task."""
import argparse
import hashlib
import importlib.util
import json
import os
from pathlib import Path
import signal
import time

HERE = Path(__file__).resolve().parent
SOURCE = HERE.parents[3] / "preflight/reasoning-client-continuity-20260912/continuity_probe.py"
EXPECTED = "e30f327526cf8a4f9953f54382617dd679a7cc2908a460e5c8ab6e471ee69923"
if hashlib.sha256(SOURCE.read_bytes()).hexdigest() != EXPECTED:
    raise ValueError("pinned continuity transport differs")
spec = importlib.util.spec_from_file_location("p06_continuity", SOURCE)
previous = importlib.util.module_from_spec(spec)
spec.loader.exec_module(previous)
previous.RAW_LIMIT = 100000
previous.WORK_SECONDS = 60
previous.TURN_SECONDS = 30
CLIENT = {"name":"work_leaf","title":"Work Leaf","version":"0.1.3"}

class WlProbe(previous.ContinuityProbe):
    def __init__(self, provider, repo):
        super().__init__(provider, repo)
        self.command = [provider, "app-server", "--listen", "stdio://"]
        self.evidence.update(command=self.command, client_info=CLIENT,
            experiment="p06-wl-input-qualification", rpc_requests=[])

    def request(self, identity, method, params):
        params = dict(params)
        if method == "initialize":
            params = {"clientInfo":CLIENT,"capabilities":{"experimentalApi":True,
                "optOutNotificationMethods":["rawResponseItem/completed"]}}
        if method == "thread/start":
            params.pop("modelProvider", None)
        if method == "turn/start":
            params.pop("effort", None)
        self.evidence["rpc_requests"].append({"id":identity,"method":method,"params":params})
        return super().request(identity, method, params)

    def run_connection(self, first, second, thread_id=None):
        if thread_id is not None:
            raise ValueError("no existing thread resume is admitted")
        return super().run_connection(first, second)

    def run_turn(self, prompt):
        if len(self.evidence["turns"]) >= 2:
            raise ValueError("two diagnostic turns maximum")
        return super().run_turn(prompt)

def main():
    parser = argparse.ArgumentParser()
    for key in ("provider", "repo", "output"):
        parser.add_argument("--"+key, required=True)
    args = parser.parse_args()
    if os.environ.get("WORK_LEAF_BENCH_P06_INPUT") != "1" or os.environ.get("CODEX_HOME"):
        raise ValueError("explicit private P06 input and original auth home required")
    probe = WlProbe(args.provider, str(Path(args.repo).resolve()))
    previous.ACTIVE_PROBE = probe
    signal.signal(signal.SIGINT, previous.stop_on_signal)
    signal.signal(signal.SIGTERM, previous.stop_on_signal)
    with Path(args.output).open("x") as output:
        try:
            probe.run_connection(previous.CONTINUATION, previous.CONTINUATION)
            probe.evidence["completed"] = True
        except Exception as error:
            probe.evidence["completed"] = False
            probe.evidence["failure"] = str(error) if type(error) in (
                RuntimeError, TimeoutError, ValueError) else type(error).__name__
        finally:
            probe.draining = True
            probe.deadline = max(probe.deadline, time.monotonic()+5)
            try:
                probe.cleanup()
            except Exception as error:
                probe.evidence["completed"] = False
                probe.evidence["cleanup_error"] = type(error).__name__
            previous.ACTIVE_PROBE = None
            probe.evidence.update(observed_raw=probe.budget["raw"],
                elapsed_seconds=probe.stamp(), transport_sha256=EXPECTED)
            json.dump(probe.evidence, output, separators=(",",":"))
            output.write("\n")
    print(json.dumps({k:probe.evidence.get(k) for k in
        ("completed","failure","thread_id","observed_raw","elapsed_seconds")}))
    return 0 if probe.evidence["completed"] else 2

if __name__ == "__main__":
    raise SystemExit(main())
