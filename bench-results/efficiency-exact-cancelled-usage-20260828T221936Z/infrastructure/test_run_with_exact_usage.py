#!/usr/bin/env python3

import os
import subprocess
import sys
import tempfile
import unittest
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))

from run_with_exact_usage import codex_config_args, write_codex_wrapper


class ExactUsageLauncherTest(unittest.TestCase):
    def test_wrapper_routes_codex_without_persisting_the_api_key(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            argument_log = root / "arguments.txt"
            fake_codex = root / "real-codex"
            fake_codex.write_text(
                "#!/usr/bin/env python3\n"
                "import os, pathlib, sys\n"
                "pathlib.Path(os.environ['ARGUMENT_LOG']).write_text('\\n'.join(sys.argv[1:]) + '\\n')\n"
            )
            fake_codex.chmod(0o700)
            wrapper = root / "bin" / "codex"
            base_url = "http://127.0.0.1:43210/v1"
            write_codex_wrapper(wrapper, fake_codex, base_url)

            environment = os.environ.copy()
            environment["ARGUMENT_LOG"] = str(argument_log)
            environment["OPENAI_API_KEY"] = "must-not-be-recorded"
            subprocess.run(
                [str(wrapper), "-c", 'model="gpt-5.5"', "exec", "--json", "-"],
                check=True,
                env=environment,
            )

            arguments = argument_log.read_text().splitlines()
            expected = codex_config_args(base_url)
            self.assertIn('model="gpt-5.5"', expected)
            self.assertIn('model_reasoning_effort="xhigh"', expected)
            self.assertEqual(arguments[: len(expected)], expected)
            self.assertEqual(arguments[len(expected) :], ["-c", 'model="gpt-5.5"', "exec", "--json", "-"])
            self.assertNotIn("must-not-be-recorded", wrapper.read_text())
            self.assertNotIn("must-not-be-recorded", argument_log.read_text())


if __name__ == "__main__":
    unittest.main()
