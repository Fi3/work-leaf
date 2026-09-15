"""Actual proc-fd/user-namespace regression, with no model invocation."""
import os
from pathlib import Path
import subprocess
import sys
import tempfile
import unittest

import repair_driver as r


class GuardPathTests(unittest.TestCase):
    def test_actual_guard_path_survives_private_user_namespace(self):
        with tempfile.TemporaryDirectory(prefix="wl-observer-guard-test.") as folder:
            root = Path(folder)
            target = root / "observation/proxy-bin"
            target.mkdir(parents=True)
            (target / "codex").symlink_to("/usr/bin/true")
            guard = subprocess.Popen([sys.executable, "-c",
                "import os,sys; fd=os.open(sys.argv[1],os.O_RDONLY|os.O_DIRECTORY); "
                "print('/proc/%s/fd/%s'%(os.getpid(),fd),flush=True); sys.stdin.read()",
                str(root)], stdin=subprocess.PIPE, stdout=subprocess.PIPE, text=True)
            try:
                guarded = guard.stdout.readline().strip()
                self.assertTrue(os.access(guarded+"/observation/proxy-bin/codex", os.X_OK))
                source = r.driver_source()
                start = source.index('  observer_root="$artifact_dir/observation"')
                end = source.index('  "$observer_bin" init ', start)
                shell = 'set -euo pipefail\nartifact_dir="$1"\nfail_bench() { return 86; }\n'
                shell += 'prepare() {\n' + source[start:end] + '\n}\nprepare\nprintf "%s" "$observer_root"\n'
                prepared = subprocess.run(["bash", "-c", shell, "fixture", guarded],
                    capture_output=True, text=True, check=True)
                observed = subprocess.run(["unshare", "--user", "--map-current-user",
                    "--keep-caps", "--mount", "--propagation", "private",
                    sys.executable, "-c",
                    "import os,sys; sys.exit(0 if os.access(sys.argv[1],os.X_OK) else 91)",
                    prepared.stdout+"/proxy-bin/codex"], capture_output=True, text=True)
                self.assertEqual(observed.returncode, 0, observed.stderr)
                self.assertEqual(Path(prepared.stdout).stat().st_ino,
                                 (root/"observation").stat().st_ino)
            finally:
                guard.communicate("", timeout=5)


if __name__ == "__main__":
    unittest.main()
