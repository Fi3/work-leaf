import unittest
from pathlib import Path
import verify_routes as v


class RouteVerificationTests(unittest.TestCase):
    def test_commands_bind_actual_host_stage_and_original_resume(self):
        command = v.host_command(Path('/tmp/repo'), Path('/tmp/artifacts'), 'probe-fix',
                                 Path('/tmp/prompt'), 'original-thread', 42.5)
        self.assertEqual(command[command.index('--resume-thread')+1], 'original-thread')
        self.assertEqual(command[command.index('--stage')+1], 'probe-fix')
        self.assertIn('--serialized-feedback', command)
        initial = v.host_command(Path('/tmp/repo'), Path('/tmp/artifacts'), 'probe-initial',
                                 Path('/tmp/prompt'), None, 42.5)
        self.assertNotIn('--resume-thread', initial)


if __name__ == '__main__':
    unittest.main()
