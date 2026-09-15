"""Actual app-server launch arguments keep the frozen catalog and project trust."""
import json
from pathlib import Path
import sys
import unittest

sys.path.insert(0, str(Path(__file__).resolve().parent))
import appserver_provider as route


class ImmutableProjectTests(unittest.TestCase):
    def test_v2_project_table_preserves_profile_and_stdio_arguments(self):
        argv = ['-c', 'model="gpt-5.5"', '-c', 'model_reasoning_effort="xhigh"',
                'app-server', '--listen', 'stdio://']
        cwd = '/tmp/project.with.dots/quoted"directory'
        plan = {'schema': 'work-leaf-p06-appserver-input-v2', 'cwd': cwd}
        expected = 'projects={' + json.dumps(cwd) + '={trust_level="trusted"}}'
        self.assertEqual(route.provider_argv(plan, argv), ['-c', expected, *argv])
        self.assertEqual(argv[0:2], ['-c', 'model="gpt-5.5"'])

    def test_v2_rejects_missing_relative_or_broad_project(self):
        for cwd in (None, '', 'relative', '/', '/tmp/../project'):
            with self.subTest(cwd=cwd), self.assertRaises(ValueError):
                route.provider_argv({'schema': 'work-leaf-p06-appserver-input-v2',
                                     'cwd': cwd}, ['app-server', '--listen', 'stdio://'])

    def test_unadmitted_v1_argument_shape_remains_unchanged(self):
        argv = ['app-server', '--listen', 'stdio://']
        self.assertEqual(route.provider_argv(
            {'schema': 'work-leaf-p06-appserver-input-v1'}, argv), argv)

    def test_catalog_entry_uses_the_qualified_read_only_remount(self):
        self.assertIn('private catalog read-only remount failed',
                      route.qualified.enter_view.__code__.co_consts)


if __name__ == '__main__':
    unittest.main()
