import tomllib
import unittest

import integration_probe
import trust_table_probe


class TrustTableProbeTests(unittest.TestCase):
    def test_cli_dotted_split_leaves_path_in_toml_value(self):
        repo = '/tmp/a.b-"c/repo'
        command = trust_table_probe.command('/provider', repo, None)
        self.assertEqual(command[1], '-c')
        key, value = command[2].split('=', 1)
        self.assertEqual(key.split('.'), ['projects'])
        parsed = tomllib.loads('value=' + value)['value']
        self.assertEqual(parsed, {repo: {'trust_level': 'trusted'}})
        self.assertEqual(command[3:], integration_probe.command('/provider', repo, None)[1:])

    def test_resume_and_invalid_targets_retain_guards(self):
        thread = '11111111-1111-7111-8111-111111111111'
        command = trust_table_probe.command('/provider', '/tmp/a.b/repo', thread)
        self.assertEqual(command[-4:], ['resume', '--json', thread, '-'])
        for path in ('/', '/tmp', 'relative', '/tmp/../repo'):
            with self.assertRaises(ValueError):
                trust_table_probe.command('/provider', path, None)


if __name__ == '__main__':
    unittest.main()
