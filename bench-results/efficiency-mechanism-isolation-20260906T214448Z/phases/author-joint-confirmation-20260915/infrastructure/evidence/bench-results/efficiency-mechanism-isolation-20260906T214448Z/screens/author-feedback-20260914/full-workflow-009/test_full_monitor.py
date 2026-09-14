from pathlib import Path
import tempfile
import unittest
from unittest.mock import patch

import full_monitor as m


class Process:
    pid = 321

    def poll(self):
        return None


class MonitorTests(unittest.TestCase):
    def test_public_population_has_no_observer_or_host_sidecar_duplicates(self):
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            runs = root/'runtime'/'owned'/'runs'
            runs.mkdir(parents=True)
            direct = runs/'review.jsonl'; direct.touch()
            host = runs/'author.host'/'invocation-0001'/'stdout.jsonl'
            host.parent.mkdir(parents=True); host.touch()
            (runs/'author.host'/'events.jsonl').touch()
            row = {'runtime_dir':str(root/'runtime'),'artifact':str(root/'archive')}
            self.assertEqual(set(m.public_files(row)), {direct, host})
            (root/'runtime'/'other'/'runs').mkdir(parents=True)
            with self.assertRaises(ValueError):
                m.public_files(row)

    def test_budget_stop_uses_owned_group_once_then_escalates(self):
        with tempfile.TemporaryDirectory() as tmp:
            manifest = {'schedule':[{'run_id':'sample','runtime_dir':tmp+'/runtime','artifact':tmp+'/archive'}],
                        'monitor_root':tmp+'/monitor'}
            value = {'native_recorded_raw':45_000_001,'public_completed_raw':2,'distinct_responses':1}
            process = Process(); active = [(process, {'id':'sample'}, None)]
            watcher = m.ResourceMonitor()
            with patch.object(m, 'sample', return_value=value), patch.object(m.time, 'monotonic', side_effect=[10, 25, 40]), patch.object(m.os,'getpgid',return_value=321), patch.object(m.os,'killpg') as kill:
                watcher(active, manifest)
                watcher(active, manifest)
                watcher(active, manifest)
            self.assertEqual([c.args for c in kill.call_args_list],[(321,m.signal.SIGINT),(321,m.signal.SIGKILL)])
            self.assertEqual(len(list((Path(tmp)/'monitor').glob('sample-*.json'))),3)

    def test_source_population_error_propagates_to_supervisor_finally(self):
        watcher = m.ResourceMonitor()
        with tempfile.TemporaryDirectory() as tmp:
            manifest = {'schedule':[{'run_id':'sample'}],'monitor_root':tmp}
            with patch.object(m,'sample',side_effect=ValueError('ambiguous source')):
                with self.assertRaises(ValueError):
                    watcher([(Process(), {'id':'sample'}, None)],manifest)


if __name__ == '__main__':
    unittest.main()
