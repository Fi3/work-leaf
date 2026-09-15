from types import SimpleNamespace
import unittest
from unittest import mock

import readonly_catalog


class ReadonlyCatalogTests(unittest.TestCase):
    def run_view(self, fail=False):
        module = readonly_catalog.load()
        calls = []
        def mount(source, target, filesystem, flags, data):
            calls.append((source, target, flags))
            return -1 if fail and flags == 4129 else 0
        lib = SimpleNamespace(mount=mount, prctl=mock.Mock(return_value=0),
                              capset=mock.Mock(return_value=0))
        view = {'uid':1000, 'mounts':[['/tmp/cache', '/fixture/cache'],
                                    ['/tmp/config', '/fixture/config']]}
        with mock.patch.object(module.ctypes, 'CDLL', return_value=lib), \
             mock.patch.object(module.os, 'stat', return_value=SimpleNamespace(st_ino=2)), \
             mock.patch.object(module.os, 'getuid', return_value=1000), \
             mock.patch.dict(module.os.environ, {'WORK_LEAF_P01_PARENT_MNT':'1'}):
            if fail:
                with self.assertRaises(OSError):
                    module.enter_view(view)
            else:
                module.enter_view(view)
        return calls, lib

    def test_all_private_mounts_are_readonly_before_capability_drop(self):
        calls, lib = self.run_view()
        self.assertEqual(calls, [(b'/tmp/cache', b'/fixture/cache', 4096),
                                 (b'/tmp/config', b'/fixture/config', 4096),
                                 (None, b'/fixture/cache', 4129),
                                 (None, b'/fixture/config', 4129)])
        lib.prctl.assert_called_once()
        lib.capset.assert_called_once()

    def test_failed_readonly_remount_never_proceeds_to_provider_ready(self):
        _, lib = self.run_view(fail=True)
        lib.prctl.assert_not_called()
        lib.capset.assert_not_called()

    def test_repair_has_one_exact_insertion_and_preserves_original_source(self):
        original = readonly_catalog.original_source()
        repaired = readonly_catalog.source()
        self.assertEqual(repaired.replace(readonly_catalog.INSERTION, '', 1), original)


if __name__ == '__main__':
    unittest.main()
