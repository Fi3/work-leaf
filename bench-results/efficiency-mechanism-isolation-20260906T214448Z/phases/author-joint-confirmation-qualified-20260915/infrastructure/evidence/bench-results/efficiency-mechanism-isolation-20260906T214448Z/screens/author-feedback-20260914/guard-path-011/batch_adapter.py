"""Bounded P09 continuation after a verified pre-provider path repair."""
from pathlib import Path
import sys

HERE = Path(__file__).resolve().parent
sys.path.insert(0, str(HERE.parent / "full-workflow-009"))
import importlib.util

# Use a distinct module name: the original source and its namespace stay intact.
spec = importlib.util.spec_from_file_location("retained_full_batch", HERE.parent / "full-workflow-009/batch_adapter.py")
prior = importlib.util.module_from_spec(spec)
spec.loader.exec_module(prior)


def qualified_runner(monitor=None):
    prior.PHASE = "author-joint-confirmation-qualified-20260915"
    prior.IDS = ["author-joint-confirmation-004", "author-joint-confirmation-005", "author-joint-confirmation-006"]
    return prior.load_runner(monitor)


if __name__ == "__main__":
    if sys.argv[1:2] == ["run"]:
        from full_monitor import ResourceMonitor
        runner = qualified_runner(ResourceMonitor())
    else:
        runner = qualified_runner()
    raise SystemExit(runner.main())
