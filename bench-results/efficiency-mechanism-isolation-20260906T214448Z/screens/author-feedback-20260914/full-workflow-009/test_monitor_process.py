import subprocess
import sys
import tempfile
from pathlib import Path
import unittest
from unittest.mock import patch

import full_monitor as m


class RealProcessMonitorTests(unittest.TestCase):
    def test_resource_stop_reaps_actual_owned_process_group(self):
        with tempfile.TemporaryDirectory() as tmp:
            child = subprocess.Popen([sys.executable, '-c',
                                      'import time; print("READY", flush=True); time.sleep(30)'],
                                     start_new_session=True, stdout=subprocess.PIPE,
                                     stderr=subprocess.PIPE, text=True)
            try:
                self.assertEqual(child.stdout.readline().strip(), 'READY')
                row = {'run_id':'owned','runtime_dir':tmp+'/runtime','artifact':tmp+'/archive'}
                manifest = {'schedule':[row],'monitor_root':tmp+'/monitor'}
                value = {'native_recorded_raw':45_000_000,'public_completed_raw':0}
                with patch.object(m, 'sample', return_value=value):
                    m.ResourceMonitor()([(child, {'id':'owned'}, None)], manifest)
                child.communicate(timeout=5)
                self.assertEqual(child.returncode, -m.signal.SIGINT)
                self.assertTrue((Path(tmp)/'monitor/owned-000001.json').is_file())
            finally:
                if child.poll() is None:
                    child.kill()
                child.communicate(timeout=5)


if __name__ == '__main__':
    unittest.main()
