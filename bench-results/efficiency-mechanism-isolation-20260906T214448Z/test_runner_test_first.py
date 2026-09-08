"""Fixed C15 launch adapter; synthetic preparation only, never run mode."""
import copy
import hashlib
import json
import os
from pathlib import Path
import sys
import tempfile
import types
import unittest
from unittest.mock import patch

HERE = Path(__file__).resolve().parent


def load(name):
    path = HERE / (name + '.py')
    body = path.read_bytes()
    module = types.ModuleType(name + '_test')
    module.__file__ = str(path)
    exec(compile(body, str(path), 'exec'), module.__dict__)
    return module


class RunnerTests(unittest.TestCase):
    def setUp(self): self.r = load('runner_test_first')

    def test_three_same_treatment_one_wave_no_control(self):
        plan = self.r.make_plan('private-test-first-01')
        self.r.validate_plan(plan)
        self.assertEqual(len(plan['runs']), 3)
        self.assertEqual({r['condition'] for r in plan['runs']}, {'private-test-first'})
        self.assertEqual({r['wave'] for r in plan['runs']}, {1})
        self.assertIs(plan['randomization']['mixed_waves'], False)
        self.assertEqual(self.r.EXPERIMENT_SCHEMA, 'work-leaf-bench-experiment-v6')

    def test_allocation_expansion_control_and_confirmation_rejected(self):
        for mutate in (lambda p: p['runs'].pop(), lambda p: p['runs'].reverse(),
                       lambda p: p['runs'][0].update(condition='control'),
                       lambda p: p.update(phase_kind='confirmation'),
                       lambda p: p['randomization'].update(mixed_waves=True)):
            plan = self.r.make_plan('fixture')
            mutate(plan)
            with self.assertRaises(ValueError): self.r.validate_plan(plan)

    def test_run_ids_fit_the_qualified_bridge_eighty_character_bound(self):
        self.assertEqual(len(self.r.make_plan('x' * 67)['runs'][0]['run_id']), 80)
        for length in (68, 76):
            with self.assertRaises(ValueError): self.r.make_plan('x' * length)

    def test_original_environment_and_supervisor_bytes_unchanged(self):
        old = load('runner_work_units')
        manifest = {'provider_dir': '/frozen/provider', 'bin_dir': '/frozen/bin', 'study': 'fixture'}
        row = {'run_id': 'fixture', 'block_id': 'block', 'runtime_dir': '/run',
               'results_dir': '/results', 'experiment_manifest': '/static/template'}
        env = {'PATH': '/usr/bin', 'CODEX_HOME': '/original', 'OPENAI_API_KEY': 'synthetic',
               'WORK_LEAF_OBSERVER_PROVIDER_USAGE_GRACE_MS': '0'}
        self.assertEqual(self.r.run_environment(manifest, {**row, 'condition': 'private-test-first'}, env),
                         old.run_environment(manifest, {**row, 'condition': 'control'}, env))
        self.assertEqual(old.CONDITIONS, ('control', 'buildable-work-unit-incremental'))
        self.assertEqual(hashlib.sha256((HERE / 'runner_work_units.py').read_bytes()).hexdigest(), self.r.ENGINE_SHA256)

    def prepare_fixture(self, root):
        from argparse import Namespace
        source, binaries = root / 'source', root / 'bin'
        source.mkdir(); binaries.mkdir()
        for name in self.r.DRIVERS:
            (source / name).write_text('#!/bin/sh\nexit 0\n')
            (source / name).chmod(0o700)
        for name in ('work-leaf', 'work-leaf-orchestrator'):
            # Synthetic ELF-shaped input. Neither file is ever executed.
            (binaries / name).write_bytes(b'\x7fELFsynthetic-' + name.encode())
            (binaries / name).chmod(0o700)
        (binaries / 'observer').write_text('#!/bin/sh\nexit 0\n')
        (binaries / 'observer').chmod(0o700)
        for name, content in (('protocol.md', '# Synthetic\n'), ('scorer.json', '{}\n'),
            ('global.toml', 'model="gpt-5.5"\n'), ('bridge-config.json', '{}\n'),
            ('schedule.json', json.dumps(self.r.make_plan('fixture')))):
            (root / name).write_text(content)
        return Namespace(phase_root=root / 'phase', source_repo=source, bin_dir=binaries,
            observer_bin=binaries / 'observer', runtime_root=root / 'runtime',
            subscription_wrapper=self.r.CANONICAL_WRAPPER, evidence=[], identity_file=[],
            task_list_sha256='a' * 64, protocol=root / 'protocol.md', scorer_config=root / 'scorer.json',
            global_config=root / 'global.toml', bridge_config=root / 'bridge-config.json',
            python=Path(sys.executable).resolve(), schedule=root / 'schedule.json', evidence_root=self.r.REPO)

    def test_full_prepare_templates_and_real_elf_pins_exist_before_manifest_publication(self):
        with tempfile.TemporaryDirectory() as tmp:
            args = self.prepare_fixture(Path(tmp).resolve())
            publications = []
            original_write = self.r._engine.write_new
            def record(path, value):
                if Path(path).name == 'PHASE-MANIFEST.json':
                    publications.append(copy.deepcopy(value))
                    entries = {r['path']: r for r in value['files']}
                    for row in value['schedule']:
                        template = json.loads(Path(row['experiment_manifest']).read_text())
                        self.assertEqual(template['schema'], self.r.binding.SCHEMA)
                        self.assertIn(template['daemon']['path'], entries)
                        self.assertIn(template['config']['path'], entries)
                        self.assertTrue(Path(template['private_root']).is_dir())
                        self.assertFalse(Path(template['output_root']).exists())
                        self.assertEqual(Path(template['daemon']['path']).read_bytes(), b'\x7fELFsynthetic-work-leaf-orchestrator')
                return original_write(path, value)
            def inspect(config, bridge, *_):
                return {}, {}, {}, {config['path']: config['sha256'], bridge['path']: bridge['sha256']}
            with patch.object(self.r, 'QUALIFIED_CONFIG_SHA256', self.r.binding.file_sha(args.bridge_config)), \
                 patch.object(self.r.binding, 'inspect_config', side_effect=inspect), \
                 patch.object(self.r._engine, 'write_new', side_effect=record), \
                 patch.object(self.r._engine, 'git', side_effect=lambda _root, *a: '1' * 40 if a[0] == 'rev-parse' else ''):
                manifest = self.r.prepare(args)
                self.assertEqual(self.r.verify_manifest(args.phase_root), manifest)
                self.assertEqual(len(publications), 1)
                self.assertFalse((args.phase_root / 'RUN-ONCE').exists())
                self.assertEqual(list((args.phase_root / 'launches').iterdir()), [])
                with self.assertRaises(FileExistsError): self.r.prepare(args)
                elf = args.phase_root / 'launch-inputs/work-leaf-orchestrator-real'
                elf.chmod(0o700); elf.write_bytes(b'\x7fELFchanged')
                with self.assertRaises(ValueError): self.r.verify_manifest(args.phase_root)

    def test_nonqualified_configuration_fails_before_creating_phase(self):
        with tempfile.TemporaryDirectory() as tmp:
            args = self.prepare_fixture(Path(tmp).resolve())
            with self.assertRaises(ValueError): self.r.prepare(args)
            self.assertFalse(args.phase_root.exists())

    def test_loaded_binding_drift_rejects_before_preparation(self):
        original = self.r.binding.file_sha
        with patch.object(self.r.binding, 'file_sha', side_effect=lambda p:
                          '0' * 64 if Path(p) == self.r.BINDING_PATH else original(p)):
            with self.assertRaises(ValueError): self.r.verify_current_dependencies()


if __name__ == '__main__': unittest.main()
