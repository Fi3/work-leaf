"""New public-input/overlay qualification, without providers or shared source edits."""
import hashlib
import json
import os
from pathlib import Path
import tempfile
import unittest
from unittest import mock

import project_inputs as P


class InputTests(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory(prefix='c15-project-input-')
        self.root = Path(self.temp.name)
        self.registry = self.root / 'registry'; self.registry.mkdir()
        self.label = 'index.crates.io-test'
        self.index = self.registry / 'index' / self.label
        self.cache = self.registry / 'cache' / self.label
        self.index.mkdir(parents=True); self.cache.mkdir(parents=True)
        self.payload = b'public crate fixture'
        self.checksum = hashlib.sha256(self.payload).hexdigest()
        self.lock = ('version = 4\n[[package]]\nname="alpha"\nversion="1.0.0"\n'
                     'source="registry+https://github.com/rust-lang/crates.io-index"\n'
                     f'checksum="{self.checksum}"\n').encode()
        (self.index / 'config.json').write_text(json.dumps({'dl': 'https://static.crates.io/crates', 'api': 'https://crates.io'}))
        self.record = self.index / '.cache/al/ph/alpha'
        self.record.parent.mkdir(parents=True)
        value = {'name': 'alpha', 'vers': '1.0.0', 'cksum': self.checksum}
        self.record.write_bytes(b'\x03\x02\x00\x00\x00etag: fixture\x001.0.0\x00' + json.dumps(value).encode() + b'\x00')
        self.archive = self.cache / 'alpha-1.0.0.crate'
        self.archive.write_bytes(self.payload)
        self.destination = self.root / 'capsule'; self.destination.mkdir()

    def tearDown(self): self.temp.cleanup()

    def capsule(self):
        return P.prepare_capsule(self.lock, self.registry, self.label, self.destination)

    def test_exact_public_allowlist_and_missing_archive_retention(self):
        (self.registry / 'credentials.toml').write_text('SECRET_CARGO_CREDENTIALS')
        (self.registry / 'config.toml').write_text('SECRET_CONFIGURATION')
        result = self.capsule()
        contents = b''.join(p.read_bytes() for p in self.destination.rglob('*') if p.is_file())
        self.assertNotIn(b'SECRET_', contents)
        self.assertEqual(len(result['sources']), 3)
        self.assertEqual(result['missing_archives'], [])
        self.assertEqual((self.destination / 'cache' / self.label / self.archive.name).read_bytes(), self.payload)
        with self.assertRaises(ValueError): self.capsule()
        self.archive.unlink()
        other = self.root / 'missing'; other.mkdir()
        missing = P.prepare_capsule(self.lock, self.registry, self.label, other)
        self.assertEqual(missing['missing_archives'], ['alpha-1.0.0'])

    def test_bad_archive_registry_config_or_index_rejected_before_publication(self):
        cases = ('checksum', 'registry', 'config', 'index', 'index_checksum')
        for case in cases:
            with self.subTest(case=case):
                lock = self.lock
                original_archive = self.archive.read_bytes()
                original_config = (self.index / 'config.json').read_bytes()
                original_index = self.record.read_bytes()
                if case == 'checksum': self.archive.write_bytes(b'wrong bytes')
                if case == 'registry': lock = lock.replace(b'github.com/rust-lang/crates.io-index', b'private.invalid/index')
                if case == 'config': (self.index / 'config.json').write_text('{"dl":"https://secret.invalid","token":"SECRET"}')
                if case == 'index': self.record.unlink()
                if case == 'index_checksum': self.record.write_bytes(original_index.replace(self.checksum.encode(), b'0'*64))
                with self.assertRaises((ValueError, FileNotFoundError)):
                    P.prepare_capsule(lock, self.registry, self.label, self.destination)
                self.assertEqual(list(self.destination.iterdir()), [])
                self.archive.write_bytes(original_archive)
                (self.index / 'config.json').write_bytes(original_config)
                self.record.write_bytes(original_index)

    def test_alias_and_hardlink_source_rejected(self):
        alias = self.root / 'alias'; alias.symlink_to(self.registry)
        with self.assertRaises(ValueError): P.prepare_capsule(self.lock, alias, self.label, self.destination)
        os.link(self.archive, self.root / 'outside')
        with self.assertRaises(ValueError): self.capsule()

    def test_private_cargo_links_only_to_explicit_readonly_capsule(self):
        receipt = self.capsule()
        scratch = self.root / 'scratch'; scratch.mkdir()
        P.prepare_cargo_home(scratch, receipt)
        home = scratch / 'cargo-home'
        self.assertEqual(os.readlink(home / 'registry/cache'), '/cargo-public/cache')
        self.assertEqual(os.readlink(home / 'registry/index'), '/cargo-public/index')
        self.assertFalse((home / 'credentials.toml').exists())
        with self.assertRaises(FileExistsError): P.prepare_cargo_home(scratch, receipt)

    def test_capsule_receipt_cannot_introduce_unlisted_parent_read(self):
        receipt = self.capsule()
        secret = self.root / 'credentials.toml'; secret.write_bytes(b'FIXTURE_SECRET')
        receipt['files']['../credentials.toml'] = hashlib.sha256(secret.read_bytes()).hexdigest()
        scratch = self.root / 'scratch'; scratch.mkdir()
        original = P.read_public
        def guarded(path, *args, **kwargs):
            if Path(path).resolve() == secret: raise AssertionError('unlisted parent read')
            return original(path, *args, **kwargs)
        with mock.patch.object(P, 'read_public', side_effect=guarded):
            with self.assertRaises(ValueError): P.prepare_cargo_home(scratch, receipt)
        self.assertFalse((scratch / 'cargo-home').exists())


class OverlayTests(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory(prefix='c15-overlay-')
        self.root = Path(self.temp.name); self.shared = self.root / 'shared'; self.shared.mkdir()
        self.m = P.materializer()
        self.m.git(self.shared, ['init', '-q'])
        self.m.git(self.shared, ['config', 'user.name', 'Fixture'])
        self.m.git(self.shared, ['config', 'user.email', 'fixture@example.invalid'])
        (self.shared / 'guidance.txt').write_bytes(b'public original\n')
        self.m.git(self.shared, ['add', '--', '.'])
        self.m.git(self.shared, ['commit', '-qm', 'fixture'])
        self.before = self.m.source_identity(self.shared)
        owned = self.root / 'owned'; owned.mkdir()
        self.snapshot = self.m.materialize(self.shared, owned, self.before['head'], self.before['tree'])
        self.overlay = {'path': 'guidance.txt', 'base_sha256': hashlib.sha256(b'public original\n').hexdigest(),
                        'base_mode': '100644', 'effective_text': 'public original\npublic addition\n',
                        'effective_sha256': hashlib.sha256(b'public original\npublic addition\n').hexdigest(),
                        'effective_mode': '100644', 'observed_index_flag': 'skip-worktree'}

    def tearDown(self): self.temp.cleanup()

    def test_generic_declared_overlay_preserves_accepted_base_and_shared_source(self):
        result = P.install_private_overlays(self.snapshot, [self.overlay])
        self.assertEqual(self.m.source_identity(self.shared), self.before)
        self.assertEqual(result['accepted_commit'], self.before['head'])
        self.assertNotEqual(result['private_after']['head'], self.before['head'])
        self.assertFalse(result['shared_accepted'])
        self.assertFalse(result['live_overlay_census_proved'])
        self.assertEqual((Path(self.snapshot['repo']) / 'guidance.txt').read_text(), self.overlay['effective_text'])

    def test_duplicate_mismatch_alias_and_unknown_overlay_refused(self):
        for case in ('duplicate', 'base', 'body', 'escape', 'flag', 'unknown'):
            with self.subTest(case=case):
                row = dict(self.overlay)
                if case == 'base': row['base_sha256'] = '0'*64
                if case == 'body': row['effective_sha256'] = '0'*64
                if case == 'escape': row['path'] = '../shared/guidance.txt'
                if case == 'flag': row['observed_index_flag'] = 'unknown'
                if case == 'unknown': row['extra'] = True
                with self.assertRaises(ValueError): P.install_private_overlays(self.snapshot, [row, row] if case == 'duplicate' else [row])
                self.assertEqual((Path(self.snapshot['repo']) / 'guidance.txt').read_bytes(), b'public original\n')


if __name__ == '__main__': unittest.main()
