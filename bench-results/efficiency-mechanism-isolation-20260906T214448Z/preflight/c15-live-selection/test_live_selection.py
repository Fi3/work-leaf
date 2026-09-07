"""Actual Git/overlay census and real caller-owned lock qualification; no providers."""
import copy
import hashlib
import json
import os
from pathlib import Path
import subprocess
import tempfile
import unittest
from unittest import mock

import live_selection as L


HERE = Path(__file__).resolve().parent
DRIVER = HERE / 'target/debug/c15-lock-selection'
STUDY = HERE.parent.parent
FROZEN = STUDY / 'infrastructure/driver-source'


class SelectionTests(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory(prefix='c15-live-select-')
        self.root = Path(self.temp.name); self.repo = self.root / 'live'; self.repo.mkdir()
        self.m = L.materializer()
        self.m.git(self.repo, ['init', '-q'])
        self.m.git(self.repo, ['config', 'user.name', 'Fixture'])
        self.m.git(self.repo, ['config', 'user.email', 'fixture@example.invalid'])
        (self.repo / 'guidance.txt').write_text('original\n')
        (self.repo / 'source.txt').write_text('old\n')
        self.m.git(self.repo, ['add', '--', '.']); self.m.git(self.repo, ['commit', '-qm', 'fixture'])
        self.head = self.m.git(self.repo, ['rev-parse', 'HEAD']).decode().strip()
        self.owned = self.root / 'selection'; self.owned.mkdir()

    def tearDown(self): self.temp.cleanup()

    def overlay(self, name='guidance.txt', skip=True, assume=False):
        blob = self.m.git(self.repo, ['rev-parse', f'{self.head}:{name}']).decode().strip()
        original = self.m.git(self.repo, ['cat-file', 'blob', blob])
        effective = original + b'public overlay\n'
        return {'path': name, 'base_blob': blob, 'base_sha256': hashlib.sha256(original).hexdigest(),
                'base_mode': '100644', 'effective_text': effective.decode(),
                'effective_sha256': hashlib.sha256(effective).hexdigest(), 'effective_mode': '100644',
                'index_flags': {'skip_worktree': skip, 'assume_unchanged': assume}}

    def install(self, row):
        name = row['path']
        (self.repo / name).write_text(row['effective_text'])
        if row['index_flags']['skip_worktree']: self.m.git(self.repo, ['update-index', '--skip-worktree', '--', name])
        if row['index_flags']['assume_unchanged']: self.m.git(self.repo, ['update-index', '--assume-unchanged', '--', name])

    def test_exact_generic_overlay_and_normal_bytes(self):
        row = self.overlay(); self.install(row)
        before_index = (self.repo / '.git/index').read_bytes()
        result = L.census(self.repo, self.head, [row])
        self.assertEqual(result['files']['guidance.txt']['live']['sha256'], row['effective_sha256'])
        self.assertEqual(result['files']['guidance.txt']['accepted']['sha256'], row['base_sha256'])
        self.assertTrue(result['files']['guidance.txt']['index_flags']['skip_worktree'])
        self.assertEqual((self.repo / '.git/index').read_bytes(), before_index)
        self.assertEqual((self.repo / 'guidance.txt').read_text(), row['effective_text'])

    def test_every_flag_and_dirty_byte_requires_exact_declaration(self):
        row = self.overlay(); self.install(row)
        cases = ('undeclared_flag', 'hidden_edit', 'dirty', 'staged', 'untracked', 'ignored', 'mode')
        for case in cases:
            with self.subTest(case=case):
                if case == 'undeclared_flag': self.m.git(self.repo, ['update-index', '--assume-unchanged', '--', 'source.txt'])
                if case == 'hidden_edit':
                    self.m.git(self.repo, ['update-index', '--skip-worktree', '--', 'source.txt'])
                    (self.repo / 'source.txt').write_text('hidden\n')
                if case in {'dirty', 'staged'}: (self.repo / 'source.txt').write_text('dirty\n')
                if case == 'staged': self.m.git(self.repo, ['add', '--', 'source.txt'])
                if case in {'untracked', 'ignored'}: (self.repo / 'other').write_text('untracked')
                if case == 'ignored': (self.repo / '.git/info/exclude').write_text('other\n')
                if case == 'mode': (self.repo / 'source.txt').chmod(0o755)
                with self.assertRaises(ValueError): L.census(self.repo, self.head, [row])
                self.m.git(self.repo, ['update-index', '--no-assume-unchanged', '--', 'source.txt'])
                self.m.git(self.repo, ['update-index', '--no-skip-worktree', '--', 'source.txt'])
                (self.repo / 'source.txt').write_text('old\n'); (self.repo / 'source.txt').chmod(0o644)
                self.m.git(self.repo, ['add', '--', 'source.txt'])
                if (self.repo / 'other').exists(): (self.repo / 'other').unlink()

    def test_overlay_identity_shape_and_duplicate_errors(self):
        row = self.overlay(); self.install(row)
        for case in ('missing', 'duplicate', 'path', 'blob', 'hash', 'body', 'flag', 'bool_alias', 'mode'):
            with self.subTest(case=case):
                changed = copy.deepcopy(row)
                if case == 'path': changed['path'] = '../guidance.txt'
                if case == 'blob': changed['base_blob'] = '0'*40
                if case == 'hash': changed['base_sha256'] = '0'*64
                if case == 'body': changed['effective_text'] += 'wrong'
                if case == 'flag': changed['index_flags']['skip_worktree'] = False
                if case == 'bool_alias': changed['index_flags']['skip_worktree'] = 1
                if case == 'mode': changed['effective_mode'] = '100755'
                rows = [] if case == 'missing' else [changed, changed] if case == 'duplicate' else [changed]
                with self.assertRaises(ValueError): L.census(self.repo, self.head, rows)

    def test_alias_hardlink_and_external_git_directory_refused(self):
        alias = self.root / 'alias'; alias.symlink_to(self.repo)
        with self.assertRaises(ValueError): L.census(alias, self.head, [])
        os.link(self.repo / 'source.txt', self.root / 'outside')
        with self.assertRaises(ValueError): L.census(self.repo, self.head, [])
        (self.root / 'outside').unlink()
        (self.repo / '.git').rename(self.root / 'git-store')
        (self.repo / '.git').write_text('gitdir: ../git-store\n')
        with self.assertRaises(ValueError): L.census(self.repo, self.head, [])

    def test_bundle_selection_survives_later_live_head(self):
        row = self.overlay(); self.install(row)
        selection = L.capture_selection(self.repo, self.owned, self.head, [row])
        (self.repo / 'source.txt').write_text('later accepted\n')
        self.m.git(self.repo, ['add', '--', 'source.txt']); self.m.git(self.repo, ['commit', '-qm', 'later work'])
        later = self.m.git(self.repo, ['rev-parse', 'HEAD']).decode().strip()
        output = self.root / 'materialized'; output.mkdir()
        snapshot = L.materialize_selected(selection, output)
        self.assertNotEqual(later, self.head)
        self.assertEqual(snapshot['shared']['head'], self.head)
        self.assertEqual(snapshot['selected_origin']['live_root'], str(self.repo))
        self.assertEqual(snapshot['selected_origin']['owned_accepted_root'], snapshot['shared']['root'])
        self.assertFalse(snapshot['selected_origin']['same_path_execution_established'])
        self.assertNotEqual(snapshot['shared']['root'], str(self.repo))
        self.assertEqual((Path(snapshot['repo']) / 'source.txt').read_text(), 'old\n')
        self.assertEqual(self.m.git(self.repo, ['rev-parse', 'HEAD']).decode().strip(), later)

    def test_bundle_and_selected_record_tampering_or_unsafe_destination(self):
        inside = self.repo / '.git/selection'; inside.mkdir()
        with self.assertRaises(ValueError): L.capture_selection(self.repo, inside, self.head, [])
        inside.rmdir()
        selection = L.capture_selection(self.repo, self.owned, self.head, [])
        output = self.root / 'materialized'; output.mkdir()
        changed = copy.deepcopy(selection); changed['commit'] = '0'*40
        with self.assertRaises(ValueError): L.materialize_selected(changed, output)
        bundle = Path(selection['bundle']['path']); bundle.chmod(0o600)
        with bundle.open('ab') as out: out.write(b'tampered')
        with self.assertRaises(ValueError): L.materialize_selected(selection, output)
        self.assertEqual(list(output.iterdir()), [])

    def test_selected_materialization_cannot_write_within_live_source(self):
        selection = L.capture_selection(self.repo, self.owned, self.head, [])
        output = self.repo / 'private-output'; output.mkdir()
        with self.assertRaisesRegex(ValueError, 'live source'):
            L.materialize_selected(selection, output)
        self.assertEqual(list(output.iterdir()), [])

    def test_drift_during_bundle_is_retained_failure_without_success(self):
        original = L.git
        def change(root, args, **kwargs):
            result = original(root, args, **kwargs)
            if args[:2] == ['bundle', 'create']: (self.repo / 'source.txt').write_text('concurrent transient work\n')
            return result
        with mock.patch.object(L, 'git', side_effect=change):
            with self.assertRaises(ValueError): L.capture_selection(self.repo, self.owned, self.head, [])
        self.assertTrue((self.owned / 'failure.json').exists())
        self.assertFalse((self.owned / 'SELECTED.json').exists())
        self.assertEqual((self.repo / 'source.txt').read_text(), 'concurrent transient work\n')

    def test_git_timeout_retains_partial_bundle_and_timing(self):
        original = L.git
        def timeout(root, args, **kwargs):
            if args[:2] == ['bundle', 'create']:
                (self.owned / 'accepted.bundle').write_bytes(b'partial')
                raise subprocess.TimeoutExpired(['git', 'bundle', 'create'], 30)
            return original(root, args, **kwargs)
        with mock.patch.object(L, 'git', side_effect=timeout):
            with self.assertRaises(subprocess.TimeoutExpired):
                L.capture_selection(self.repo, self.owned, self.head, [])
        failure = json.loads((self.owned / 'failure.json').read_bytes())
        self.assertEqual(failure['bundle_bytes_at_endpoint'], 7)
        self.assertGreaterEqual(failure['selection_operation_seconds'], 0)
        self.assertFalse(failure['in_flight_bundle_storage_bound'])
        self.assertFalse((self.owned / 'SELECTED.json').exists())

    def test_bundle_limit_is_postcreation_and_failure_retains_size(self):
        with mock.patch.object(L, 'MAX_BUNDLE_BYTES', 1):
            with self.assertRaises(ValueError): L.capture_selection(self.repo, self.owned, self.head, [])
        failure = json.loads((self.owned / 'failure.json').read_bytes())
        self.assertGreater(failure['bundle_bytes_at_endpoint'], 1)
        self.assertGreaterEqual(failure['selection_operation_seconds'], 0)
        self.assertFalse(failure['in_flight_bundle_storage_bound'])
        self.assertFalse((self.owned / 'SELECTED.json').exists())

    def test_real_same_table_lock_excludes_patch_then_releases(self):
        rows = self.root / 'overlays.json'; rows.write_text('[]')
        patch = self.root / 'patch.json'
        patch.write_text(json.dumps({'body': '*** Begin Patch\n*** Update File: source.txt\n@@\n-old\n+writer\n*** End Patch\n'}))
        result = subprocess.run([str(DRIVER), str(self.repo), str(self.owned), str(rows),
                                 str(HERE / 'live_selection.py'), str(patch)], capture_output=True, text=True, timeout=30,
                                env={'PATH': '/usr/bin:/bin', 'GIT_CONFIG_NOSYSTEM': '1',
                                     'GIT_CONFIG_GLOBAL': '/dev/null', 'GIT_CONFIG_COUNT': '2',
                                     'GIT_CONFIG_KEY_0': 'core.hooksPath', 'GIT_CONFIG_VALUE_0': '/dev/null',
                                     'GIT_CONFIG_KEY_1': 'core.fsmonitor', 'GIT_CONFIG_VALUE_1': 'false'})
        self.assertEqual(result.returncode, 0, result.stderr)
        receipt = json.loads(result.stdout)
        self.assertTrue(receipt['writer_blocked_inside_owner_closure'])
        self.assertTrue(receipt['writer_applied_after_release'])
        self.assertGreater(receipt['owner_lock_seconds'], 0)
        selection = json.loads((self.owned / 'SELECTED.json').read_bytes())
        self.assertEqual(selection['commit'], self.head)
        self.assertEqual((self.repo / 'source.txt').read_text(), 'writer\n')

    def test_actual_frozen_benchmark_installer_overlay_is_exact(self):
        script = FROZEN / 'bench-agent-profile-common'
        self.assertEqual(hashlib.sha256(script.read_bytes()).hexdigest(), 'de9803658bc8ac41a9356436be2c8b8edfce9d7bef9176841e067b3478633d4f')
        project = self.root / 'frozen-project'
        self.m.git(self.root, ['clone', '--no-checkout', '--no-hardlinks', '--local', str(FROZEN), str(project)])
        base = 'c92a0b7060a36eac6db2d869b85e589a7a9480f9'
        self.m.git(project, ['checkout', '--detach', base])
        policy = self.root / 'policy'
        subprocess.run(['/bin/bash', '-c', 'source "$1"; bench_install_no_recursive_agent_policy "$2" "$3"',
                        'fixture', str(script), str(project), str(policy)], check=True,
                       env={'PATH': '/usr/bin:/bin', 'GIT_CONFIG_NOSYSTEM': '1', 'GIT_CONFIG_GLOBAL': '/dev/null'})
        original = (policy / 'original-AGENTS.md').read_bytes(); effective = (policy / 'effective-AGENTS.md').read_bytes()
        row = {'path': 'AGENTS.md', 'base_blob': self.m.git(project, ['rev-parse', f'{base}:AGENTS.md']).decode().strip(),
               'base_sha256': hashlib.sha256(original).hexdigest(), 'base_mode': '100644',
               'effective_text': effective.decode(), 'effective_sha256': hashlib.sha256(effective).hexdigest(),
               'effective_mode': '100644', 'index_flags': {'skip_worktree': True, 'assume_unchanged': False}}
        result = L.census(project, base, [row])
        self.assertEqual(result['files']['AGENTS.md']['live']['sha256'], row['effective_sha256'])
        self.m.git(project, ['update-index', '--assume-unchanged', '--', 'src/ui.rs'])
        with (project / 'src/ui.rs').open('a') as out: out.write('\n// undeclared hidden edit\n')
        with self.assertRaises(ValueError): L.census(project, base, [row])

    def test_unsupported_attributes_symlink_and_submodule_entries(self):
        self.m.git(self.repo, ['config', 'filter.example.clean', 'false'])
        with self.assertRaises(ValueError): L.census(self.repo, self.head, [])
        self.m.git(self.repo, ['config', '--unset', 'filter.example.clean'])
        attrs = self.repo / '.git/info/attributes'; attrs.write_text('source.txt text\n')
        with self.assertRaises(ValueError): L.census(self.repo, self.head, [])
        attrs.unlink()
        (self.repo / 'link').symlink_to('source.txt')
        self.m.git(self.repo, ['add', '--', 'link']); self.m.git(self.repo, ['commit', '-qm', 'symlink fixture'])
        linked = self.m.git(self.repo, ['rev-parse', 'HEAD']).decode().strip()
        with self.assertRaises(ValueError): L.census(self.repo, linked, [])
        self.m.git(self.repo, ['update-index', '--add', '--cacheinfo', f'160000,{linked},module'])
        self.m.git(self.repo, ['commit', '-qm', 'submodule fixture'])
        submodule = self.m.git(self.repo, ['rev-parse', 'HEAD']).decode().strip()
        with self.assertRaises(ValueError): L.census(self.repo, submodule, [])

    def test_selection_failure_releases_real_owner_and_retains_caller_duration(self):
        rows = self.root / 'overlays.json'; rows.write_text('[{"unsupported":true}]')
        patch = self.root / 'patch.json'
        patch.write_text(json.dumps({'body': '*** Begin Patch\n*** Update File: source.txt\n@@\n-old\n+writer\n*** End Patch\n'}))
        result = subprocess.run([str(DRIVER), str(self.repo), str(self.owned), str(rows),
                                 str(HERE / 'live_selection.py'), str(patch)], capture_output=True, timeout=30,
                                env={'PATH': '/usr/bin:/bin', 'GIT_CONFIG_NOSYSTEM': '1',
                                     'GIT_CONFIG_GLOBAL': '/dev/null', 'GIT_CONFIG_COUNT': '2',
                                     'GIT_CONFIG_KEY_0': 'core.hooksPath', 'GIT_CONFIG_VALUE_0': '/dev/null',
                                     'GIT_CONFIG_KEY_1': 'core.fsmonitor', 'GIT_CONFIG_VALUE_1': 'false'})
        self.assertEqual(result.returncode, 1)
        caller = json.loads((self.owned / 'CALLER.json').read_bytes())
        self.assertFalse(caller['selection_succeeded'])
        self.assertTrue(caller['writer_applied_after_release'])
        self.assertGreater(caller['owner_lock_seconds'], 0)
        self.assertTrue((self.owned / 'failure.json').exists())
        self.assertFalse((self.owned / 'SELECTED.json').exists())
        self.assertEqual((self.repo / 'source.txt').read_text(), 'writer\n')


if __name__ == '__main__': unittest.main()
