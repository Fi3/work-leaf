"""Private operator receipts; no measured agent input or host behavior changes."""
from datetime import datetime, timezone
import json
import os
from pathlib import Path
import sys

sys.path.insert(0, str(Path(__file__).resolve().parents[1] / "author-inverse-20260914"))
import resource_sample as resources


class MonitorFailure(RuntimeError):
    pass


def publish(path, value):
    with Path(path).open("x", encoding="utf-8") as output:
        json.dump(value, output, indent=2, sort_keys=True, allow_nan=False)
        output.write("\n")
        output.flush()
        os.fsync(output.fileno())


def prepare(root):
    """Call before admitting/starting a provider; existing receipts reject reuse."""
    root = Path(root)
    (root / "resources").mkdir(parents=True, exist_ok=True)
    publish(root / "resources/start.json", {"prepared_at": datetime.now(timezone.utc).isoformat(),
                                           "scope": "operator monitor, no provider admitted"})


def step(root, index, sample, stop, writer=publish):
    """Any sampling/publication error stops exactly the caller-owned supervisor."""
    root = Path(root)
    try:
        value = sample()
        writer(root / "resources" / f"sample-{index:04d}.json", value)
        return value
    except Exception as error:
        failure = {"at": datetime.now(timezone.utc).isoformat(),
                   "error": type(error).__name__ + ": " + str(error)}
        try:
            failure["stop"] = stop()
        except Exception as stop_error:
            failure["stop_error"] = type(stop_error).__name__ + ": " + str(stop_error)
        # The base artifact directory is independent of the failed resource path/writer.
        try:
            publish(root / f"monitor-failure-{index:04d}.json", failure)
        except Exception as publication_error:
            failure["failure_receipt_error"] = str(publication_error)
        print(json.dumps(failure, sort_keys=True), file=sys.stderr, flush=True)
        raise MonitorFailure(json.dumps(failure, sort_keys=True)) from error
