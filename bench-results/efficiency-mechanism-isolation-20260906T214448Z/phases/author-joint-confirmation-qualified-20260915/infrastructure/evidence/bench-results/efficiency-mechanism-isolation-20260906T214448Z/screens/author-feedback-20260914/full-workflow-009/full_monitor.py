"""Per-owned-workflow resource trips inside the retained supervisor loop."""
from datetime import datetime, timedelta, timezone
import json
import os
from pathlib import Path
import signal
import sys
import time

sys.path.insert(0, str(Path(__file__).resolve().parent.parent/'resume-input-003'))
import output_monitor


def public_files(row):
    roots = sorted(Path(row['runtime_dir']).glob('*/runs'))
    if len(roots) > 1:
        raise ValueError('ambiguous owned runtime output directory')
    root = roots[0] if roots else Path(row['artifact'])/'runs'
    return sorted(set(root.glob('*.jsonl')) | set(root.glob('*.host/invocation-*/stdout.jsonl')))


def sample(row):
    # The model's local calendar can differ from UTC around midnight. Only exact
    # thread IDs discovered in the owned public files select native rollouts.
    today = datetime.now(timezone.utc).date()
    sessions = Path.home()/'.codex/sessions'
    days = [sessions/(today + timedelta(days=offset)).strftime('%Y/%m/%d')
            for offset in (-1, 0, 1)]
    return output_monitor.sample(Path(row['artifact']), public_files(row), days)


class ResourceMonitor:
    def __init__(self):
        self.next_sample = 0
        self.index = 0
        self.stopped = {}

    def __call__(self, active, manifest):
        now = time.monotonic()
        if now < self.next_sample:
            return
        self.next_sample = now + 15
        self.index += 1
        root = (Path(manifest['monitor_root']) if 'monitor_root' in manifest else
                Path(manifest['schedule'][0]['results_dir']).parents[1]/'resource-samples')
        root.mkdir(parents=True, exist_ok=True)
        rows = {r['run_id']:r for r in manifest['schedule']}
        for process, result, _ in active:
            run_id = result['id']
            value = sample(rows[run_id])
            value['stop_threshold_reached'] = max(value['native_recorded_raw'], value['public_completed_raw']) >= 45_000_000
            action = None
            if process.poll() is None:
                if run_id not in self.stopped and value['stop_threshold_reached']:
                    self.stopped[run_id] = now
                    action = signal.SIGINT
                elif run_id in self.stopped and now - self.stopped[run_id] > 20:
                    action = signal.SIGKILL
                if action is not None:
                    if os.getpgid(process.pid) != process.pid:
                        raise ValueError('owned workflow lost its original process group')
                    os.killpg(process.pid, action)
            receipt = {'at':datetime.now(timezone.utc).isoformat(), 'run_id':run_id,
                       'resource':value, 'signalled_pid':process.pid if action else None,
                       'signal':action.name if action else None,
                       'scope':'live alternate resource representations, never added together'}
            with (root/f'{run_id}-{self.index:06d}.json').open('x') as handle:
                json.dump(receipt, handle, indent=2)
            print(json.dumps(receipt), flush=True)
