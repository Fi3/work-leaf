"""Generic, provider-free accepted-tree/held-proposal qualification."""
import copy
import hashlib
import json
import os
from pathlib import Path
import subprocess
import tempfile
import unittest
from unittest import mock

import materialize as M


HERE = Path(__file__).resolve().parent
DRIVER = HERE / 'target/debug/c15-patch-driver'
BWRAP_SHA = 'eabbccb0f7f755b96d30834026a9b5d941c606400d097d87c1ff16622edaf68c'


class MaterializationTests(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory(prefix='c15-materialize-')
        self.root = Path(self.temp.name)
        self.shared = self.root / 'shared'
        self.shared.mkdir()
        M.git(self.shared, ['init', '-q'])
        M.git(self.shared, ['config', 'user.name', 'Fixture'])
        M.git(self.shared, ['config', 'user.email', 'fixture@example.invalid'])
        (self.shared / 'source.txt').write_text('old\n')
        (self.shared / 'execute.sh').write_text('#!/bin/sh\nexit 0\n')
        (self.shared / 'execute.sh').chmod(0o755)
        M.git(self.shared, ['add', '--', '.'])
        M.git(self.shared, ['commit', '-qm', 'fixture source'])
        self.head = M.git(self.shared, ['rev-parse', 'HEAD']).decode().strip()
        self.tree = M.git(self.shared, ['rev-parse', 'HEAD^{tree}']).decode().strip()
        self.owned = self.root / 'private'
        self.owned.mkdir()

    def tearDown(self):
        self.temp.cleanup()

    def snapshot(self):
        return M.materialize(self.shared, self.owned, self.head, self.tree)

    def driver(self):
        return {'path': str(DRIVER), 'sha256': hashlib.sha256(DRIVER.read_bytes()).hexdigest(),
                'bwrap_sha256': BWRAP_SHA}

    def proposal(self, ident='p1', body=None, format='edit'):
        return {'id': ident, 'format': format, 'agent_id': 'fixture-author',
                'feature': 'generic behavior', 'reason': 'held test proposal',
                'declared_test_units': ['declared behavior unit'],
                'body': body or '*** Begin Patch\n*** Update File: source.txt\n@@\n-old\n+new\n*** End Patch\n'}

    def test_exact_independent_tree_modes_and_shared_identity(self):
        before = M.source_identity(self.shared)
        snap = self.snapshot()
        copied = Path(snap['repo'])
        self.assertEqual(M.source_identity(copied)['tree'], self.tree)
        self.assertEqual((copied / 'source.txt').read_bytes(), b'old\n')
        self.assertEqual((copied / 'execute.sh').stat().st_mode & 0o777, 0o755)
        self.assertTrue((copied / '.git').is_dir())
        self.assertFalse((copied / '.git/commondir').exists())
        self.assertFalse((copied / '.git/objects/info/alternates').exists())
        self.assertNotEqual((copied / '.git/index').stat().st_ino, (self.shared / '.git/index').stat().st_ino)
        self.assertEqual(M.source_identity(self.shared), before)
        with self.assertRaises(ValueError): self.snapshot()

    def test_existing_edit_and_unified_semantics_only_private(self):
        before = M.source_identity(self.shared)
        snap = self.snapshot()
        one = M.apply_proposal(snap, self.proposal(), self.driver())
        self.assertTrue(one['applied_privately'], one)
        self.assertFalse(one['shared_accepted'])
        self.assertEqual(one['files']['source.txt']['before']['sha256'], hashlib.sha256(b'old\n').hexdigest())
        self.assertEqual(one['files']['source.txt']['after']['sha256'], hashlib.sha256(b'new\n').hexdigest())
        patch = '--- a/source.txt\n+++ b/source.txt\n@@ -1 +1 @@\n-new\n+final\n'
        two = M.apply_proposal(snap, self.proposal('p2', patch, 'patch'), self.driver())
        self.assertTrue(two['applied_privately'], two)
        self.assertEqual((Path(snap['repo']) / 'source.txt').read_text(), 'final\n')
        self.assertEqual(M.source_identity(self.shared), before)
        self.assertEqual(one['proposal']['body'], self.proposal()['body'])

    def test_failed_exact_block_and_path_escape_leave_shared_untouched(self):
        before = M.source_identity(self.shared)
        snap = self.snapshot()
        body = self.proposal()['body'].replace('-old', '-missing')
        failure = M.apply_proposal(snap, self.proposal(body=body), self.driver())
        self.assertFalse(failure['applied_privately'])
        self.assertIn('not found', failure['execution']['stdout'])
        escape = body.replace('source.txt', '../shared/source.txt')
        failure2 = M.apply_proposal(snap, self.proposal('p2', escape), self.driver())
        self.assertFalse(failure2['applied_privately'])
        self.assertEqual(M.source_identity(self.shared), before)

    def test_dirty_index_wrong_identity_and_untracked_fail(self):
        for case in ('dirty', 'index', 'untracked', 'ignored', 'commit', 'tree'):
            with self.subTest(case=case):
                changed = self.shared / 'extra'
                if case == 'dirty': (self.shared / 'source.txt').write_text('dirty\n')
                if case == 'index':
                    (self.shared / 'source.txt').write_text('staged\n')
                    M.git(self.shared, ['add', '--', 'source.txt'])
                if case in ('untracked', 'ignored'): changed.write_text('untracked')
                if case == 'ignored': (self.shared / '.git/info/exclude').write_text('extra\n')
                with self.assertRaises(ValueError):
                    M.materialize(self.shared, self.owned,
                                  '0'*40 if case == 'commit' else self.head,
                                  '0'*40 if case == 'tree' else self.tree)
                self.assertFalse((self.owned / 'repo').exists())
                M.git(self.shared, ['reset', '--hard', self.head])
                if changed.exists(): changed.unlink()

    def test_alias_hardlink_symlink_submodule_filter_unsupported(self):
        for case in ('alias', 'hardlink', 'symlink', 'submodule', 'filter'):
            with self.subTest(case=case):
                repo = self.shared
                if case == 'alias':
                    repo = self.root / 'alias'; repo.symlink_to(self.shared)
                if case == 'hardlink': os.link(self.shared / 'source.txt', self.root / 'outside')
                if case == 'symlink':
                    (self.shared / 'link').symlink_to('/outside')
                    M.git(self.shared, ['add', '--', 'link'])
                if case == 'submodule':
                    M.git(self.shared, ['update-index', '--add', '--cacheinfo', f'160000,{self.head},module'])
                if case == 'filter':
                    (self.shared / '.gitattributes').write_text('source.txt filter=external\n')
                    M.git(self.shared, ['add', '--', '.gitattributes'])
                if case in ('symlink', 'submodule', 'filter'):
                    M.git(self.shared, ['commit', '-qm', 'unsupported materialization'])
                head = M.git(self.shared, ['rev-parse', 'HEAD']).decode().strip()
                tree = M.git(self.shared, ['rev-parse', 'HEAD^{tree}']).decode().strip()
                with self.assertRaises(ValueError): M.materialize(repo, self.owned, head, tree)
                self.assertFalse((self.owned / 'repo').exists())
                M.git(self.shared, ['reset', '--hard', self.head])
                if case == 'hardlink': (self.root / 'outside').unlink()

    def test_source_drift_during_clone_is_retained_failure(self):
        original = M.git
        def drift(root, args, **kwargs):
            value = original(root, args, **kwargs)
            if args and args[0] == 'clone':
                (self.shared / 'source.txt').write_text('concurrent drift\n')
            return value
        with mock.patch.object(M, 'git', side_effect=drift):
            with self.assertRaises(ValueError): self.snapshot()
        self.assertTrue((self.owned / 'repo').exists())
        self.assertEqual((self.shared / 'source.txt').read_text(), 'concurrent drift\n')
        self.assertTrue((self.owned / 'failure.json').exists())

    def test_duplicate_proposal_and_evidence_failure_never_apply(self):
        snap = self.snapshot()
        (self.owned / 'evidence/p1').mkdir()
        with self.assertRaises(FileExistsError): M.apply_proposal(snap, self.proposal(), self.driver())
        self.assertEqual((Path(snap['repo']) / 'source.txt').read_text(), 'old\n')
        with mock.patch.object(M, 'publish', side_effect=OSError('evidence failure')):
            with self.assertRaises(OSError): M.apply_proposal(snap, self.proposal('p2'), self.driver())
        self.assertEqual((Path(snap['repo']) / 'source.txt').read_text(), 'old\n')

    def test_later_accepted_shared_commit_does_not_replace_held_base(self):
        snap = self.snapshot()
        (self.shared / 'source.txt').write_text('other accepted work\n')
        M.git(self.shared, ['add', '--', 'source.txt'])
        M.git(self.shared, ['commit', '-qm', 'later accepted work'])
        latest = M.source_identity(self.shared)
        outcome = M.apply_proposal(snap, self.proposal(), self.driver())
        self.assertTrue(outcome['applied_privately'])
        self.assertEqual(outcome['private_before']['head'], self.head)
        self.assertEqual((Path(snap['repo']) / 'source.txt').read_text(), 'new\n')
        self.assertEqual(M.source_identity(self.shared), latest)
        self.assertTrue(outcome['shared_advanced_since_snapshot'])

    def test_unsupported_private_after_image_retains_execution_failure(self):
        snap = self.snapshot()
        diff = 'diff --git a/link b/link\nnew file mode 120000\n--- /dev/null\n+++ b/link\n@@ -0,0 +1 @@\n+/outside\n'
        with self.assertRaises(ValueError):
            M.apply_proposal(snap, self.proposal(body=diff, format='patch'), self.driver())
        folder = self.owned / 'evidence/p1'
        self.assertTrue((folder / 'EXECUTION.json').exists())
        self.assertTrue((folder / 'failure.json').exists())
        self.assertFalse((folder / 'RESULT.json').exists())

    def test_oversize_source_rejected_before_reading_body(self):
        original = Path.read_bytes
        def guarded(path):
            if path in {self.shared / 'source.txt', self.shared / 'execute.sh'}: raise AssertionError('oversized body read')
            return original(path)
        with mock.patch.object(M, 'MAX_BYTES', 1), mock.patch.object(Path, 'read_bytes', guarded):
            with self.assertRaisesRegex(ValueError, 'byte bound'): self.snapshot()

    def test_index_hidden_flags_are_unsupported_even_when_status_clean(self):
        for flag, undo in (('--skip-worktree', '--no-skip-worktree'),
                           ('--assume-unchanged', '--no-assume-unchanged')):
            with self.subTest(flag=flag):
                M.git(self.shared, ['update-index', flag, '--', 'source.txt'])
                self.assertEqual(M.git(self.shared, ['status', '--porcelain']), b'')
                with self.assertRaisesRegex(ValueError, 'index flags'):
                    M.source_identity(self.shared)
                (self.shared / 'source.txt').write_text('hidden tracked overlay\n')
                self.assertEqual(M.git(self.shared, ['status', '--porcelain']), b'')
                with self.assertRaises(ValueError): self.snapshot()
                self.assertFalse((self.owned / 'repo').exists())
                (self.shared / 'source.txt').write_text('old\n')
                M.git(self.shared, ['update-index', undo, '--', 'source.txt'])

    def test_held_test_red_then_unchanged_test_private_implementation_green(self):
        (self.shared / 'src').mkdir()
        (self.shared / 'Cargo.toml').write_text('[package]\nname="held-proposal-fixture"\nversion="0.0.0"\nedition="2021"\n')
        (self.shared / 'Cargo.lock').write_text('version = 4\n\n[[package]]\nname = "held-proposal-fixture"\nversion = "0.0.0"\n')
        (self.shared / 'src/lib.rs').write_text('pub fn value() -> u8 { 1 }\n')
        M.git(self.shared, ['add', '--', '.'])
        M.git(self.shared, ['commit', '-qm', 'generic Rust baseline'])
        self.head = M.git(self.shared, ['rev-parse', 'HEAD']).decode().strip()
        self.tree = M.git(self.shared, ['rev-parse', 'HEAD^{tree}']).decode().strip()
        shared = M.source_identity(self.shared)
        snap = self.snapshot()
        test_body = '#[test]\nfn required_behavior() { assert_eq!(held_proposal_fixture::value(), 2); }\n'
        edit = '*** Begin Patch\n*** Add File: tests/behavior.rs\n' + ''.join('+'+line+'\n' for line in test_body.splitlines()) + '*** End Patch\n'
        held = M.apply_proposal(snap, self.proposal(body=edit), self.driver())
        self.assertTrue(held['applied_privately'], held)
        sysroot = Path(subprocess.check_output(['rustc', '--print', 'sysroot'], text=True).strip()).resolve()
        spec = M.execution_spec(snap, [{'source': str(sysroot), 'target': '/toolchain'}])
        command = ['/usr/bin/env', 'RUSTC=/toolchain/bin/rustc', '/toolchain/bin/cargo',
                   'test', '--offline', '--locked', '--test', 'behavior', 'required_behavior', '--', '--exact']
        red = M.executor().run_preview(spec, command, timeout=20)
        self.assertEqual(red['exit_code'], 101, red)
        self.assertIn('required_behavior ... FAILED', red['stdout'])
        self.assertIn('left: 1', red['stdout'])
        impl = '*** Begin Patch\n*** Update File: src/lib.rs\n@@\n-pub fn value() -> u8 { 1 }\n+pub fn value() -> u8 { 2 }\n*** End Patch\n'
        implemented = M.apply_proposal(snap, self.proposal('p2', impl), self.driver())
        self.assertTrue(implemented['applied_privately'], implemented)
        green = M.executor().run_preview(spec, command, timeout=20)
        self.assertEqual(green['exit_code'], 0, green)
        self.assertIn('required_behavior ... ok', green['stdout'])
        self.assertEqual((Path(snap['repo']) / 'tests/behavior.rs').read_text(), test_body)
        self.assertEqual(M.source_identity(self.shared), shared)
        self.assertFalse((self.shared / 'tests').exists())


if __name__ == '__main__': unittest.main()
