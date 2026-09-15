from pathlib import Path
import subprocess
import sys
import unittest

import provider_route


class ProviderRouteTests(unittest.TestCase):
    def test_outer_prepends_trust_once_and_inside_preserves_it(self):
        args = ['--cd', '/tmp/exact/repo', '--sandbox', 'danger-full-access',
                '--ask-for-approval', 'never', 'exec', '--json', '-']
        outer = provider_route.arguments(args, 'linearize-plan')
        self.assertEqual(outer[2:], args)
        inside = ['--inside', *outer]
        self.assertEqual(provider_route.arguments(inside, 'linearize-plan'), inside)
        self.assertEqual(outer.count('-c'), 1)

    def test_private_reentry_keeps_this_provider_and_immutable_view(self):
        module = provider_route.configured_provider()
        self.assertEqual(module.HERE, Path(provider_route.__file__).resolve().parent)
        self.assertEqual(module.dispatch.qualified.enter_view.__module__, 'private_immutable_catalog')

    def test_foreign_role_is_rejected_before_private_reentry(self):
        with self.assertRaises(ValueError):
            provider_route.arguments(['--inside', 'exec'], 'foreign')

    def test_real_probe_uses_immutable_provider_without_preinjected_trust(self):
        argv = provider_route.probe_command('/unused/provider', Path('/tmp/exact/repo'), None)
        self.assertEqual(argv[0], str(Path(provider_route.__file__).with_name('input_provider')))
        self.assertNotIn('-c', argv)

    def test_actual_probe_entrypoint_imports_without_generation(self):
        result = subprocess.run([sys.executable, str(Path(__file__).with_name('real_probe.py')), '--help'],
                                capture_output=True, text=True)
        self.assertEqual(result.returncode, 0, result.stderr)


if __name__ == '__main__':
    unittest.main()
