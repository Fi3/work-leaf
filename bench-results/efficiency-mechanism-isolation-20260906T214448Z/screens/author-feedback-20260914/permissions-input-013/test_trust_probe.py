from pathlib import Path
import unittest

import integration_probe
import trust_probe


class TrustProbeTests(unittest.TestCase):
    def test_only_exact_checkout_trust_is_appended_on_both_calls(self):
        repo = Path('/tmp/source-state-example/repo')
        thread = '11111111-1111-7111-8111-111111111111'
        for owner in (None, thread):
            original = integration_probe.command('/provider', repo, owner)
            changed = trust_probe.command('/provider', repo, owner)
            self.assertEqual(changed, [original[0], '-c',
                'projects."/tmp/source-state-example/repo".trust_level="trusted"',
                *original[1:]])

    def test_invalid_or_broad_path_is_rejected(self):
        for repo in ('/', '/tmp', 'relative', '/tmp/../repo', '/tmp/path\nname'):
            with self.assertRaises(ValueError):
                trust_probe.command('/provider', repo, None)
        with self.assertRaises(ValueError):
            trust_probe.command('/provider', '/tmp/valid/repo', 'foreign')

    def test_path_is_quoted_as_data_not_toml_structure(self):
        self.assertEqual(trust_probe.trust_override('/tmp/a"b/repo'),
            'projects."/tmp/a\\"b/repo".trust_level="trusted"')


if __name__ == '__main__':
    unittest.main()
