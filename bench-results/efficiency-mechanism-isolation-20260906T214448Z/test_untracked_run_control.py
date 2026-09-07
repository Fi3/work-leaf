#!/usr/bin/env python3
"""Provider-free regression gates for the v3 allocation/control adapters."""
import copy
import hashlib
import importlib.util
import json
from pathlib import Path
import tempfile
import unittest
from unittest.mock import patch

HERE = Path(__file__).resolve().parent


def load(name):
    spec = importlib.util.spec_from_file_location('test_v3_' + name, HERE / (name + '.py'))
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


class UntrackedControlTests(unittest.TestCase):
    def setUp(self):
        self.allocator = load('allocate_untracked_reads')
        self.runner = load('runner_untracked_reads')

    def plan(self):
        return self.allocator.make_plan('read-inline-fixture', 'workflow', lambda choices: choices[0])

    def test_exact_population_and_v3_only_conditions(self):
        choices = self.allocator.block_population()
        self.assertEqual(len(choices), 18)
        for left in choices:
            for right in choices:
                draws = iter((left, right))
                plan = self.allocator.make_plan('read-inline-fixture', 'workflow', lambda _: next(draws))
                self.runner.validate_plan(plan)
                self.assertEqual(len(plan['runs']), 12)
                self.assertEqual(sum(r['condition'] == 'untracked-read-inline' for r in plan['runs']), 6)
        self.assertEqual(self.runner.EXPERIMENT_SCHEMA, 'work-leaf-bench-experiment-v3')
        self.assertEqual(self.runner.CONDITIONS, ('control', 'untracked-read-inline'))
        for bad in ('buildable-work-unit-incremental', 'ack-validation-unlimited', 'direct'):
            plan = self.plan(); plan['runs'][0]['condition'] = bad
            with self.assertRaises(ValueError): self.runner.validate_plan(plan)

    def test_schedule_cannot_shrink_expand_relabel_or_change_population(self):
        original = self.plan()
        for mutate in (
            lambda p: p['runs'].pop(),
            lambda p: p['runs'].append(copy.deepcopy(p['runs'][0])),
            lambda p: p.update(phase_kind='screening'),
            lambda p: p['randomization'].update(joint_population_size=400),
            lambda p: p['randomization']['actual_variant_slots_by_block'].update({'wrong': [1, 2, 4]}),
            lambda p: p['runs'].reverse(),
        ):
            plan = copy.deepcopy(original); mutate(plan)
            with self.assertRaises(ValueError): self.runner.validate_plan(plan)

    def test_allocate_claim_precedes_draw_and_forbids_reuse(self):
        with tempfile.TemporaryDirectory() as tmp:
            phase = Path(tmp) / 'read-inline-fixture'
            def choose(choices):
                self.assertTrue((phase / 'ALLOCATION-CLAIM.json').is_file())
                return choices[-1]
            with patch.object(self.allocator.random.SystemRandom, 'choice', side_effect=choose):
                plan = self.allocator.allocate(phase, phase.name, 'workflow')
            self.runner.validate_plan(plan)
            self.assertFalse((phase / 'RUN-ONCE').exists())
            self.assertEqual(plan['allocation_provenance']['schema'], 'work-leaf-untracked-read-allocation-v3')
            with self.assertRaises(FileExistsError): self.allocator.allocate(phase, phase.name, 'workflow')

    def test_failed_draw_keeps_its_claim(self):
        with tempfile.TemporaryDirectory() as tmp:
            phase = Path(tmp) / 'read-inline-fixture'
            with patch.object(self.allocator.random.SystemRandom, 'choice', side_effect=RuntimeError('fixture')):
                with self.assertRaises(RuntimeError): self.allocator.allocate(phase, phase.name, 'workflow')
            self.assertTrue((phase / 'ALLOCATION-CLAIM.json').is_file())
            self.assertFalse((phase / 'ALLOCATION.json').exists())
            with self.assertRaises(FileExistsError): self.allocator.allocate(phase, phase.name, 'workflow')

    def test_nonfactor_environment_is_identical_to_pinned_engine(self):
        row = {**self.plan()['runs'][0], 'runtime_dir': '/fixture/runtime',
               'results_dir': '/fixture/results', 'experiment_manifest': '/fixture/manifest.json'}
        manifest = {'provider_dir': '/fixture/provider', 'bin_dir': '/fixture/bin', 'study': 'fixture'}
        inherited = {'PATH': '/usr/bin', 'HOME': '/existing', 'CODEX_HOME': '/existing/.codex',
                     'OPENAI_API_KEY': 'synthetic', 'WORK_LEAF_UNKNOWN': 'discard'}
        control = self.runner.run_environment(manifest, {**row, 'condition': 'control'}, inherited)
        variant = self.runner.run_environment(manifest, {**row, 'condition': 'untracked-read-inline'}, inherited)
        self.assertEqual(control, variant)
        self.assertEqual(control['CODEX_HOME'], inherited['CODEX_HOME'])
        self.assertNotIn('OPENAI_API_KEY', control)
        self.assertNotIn('WORK_LEAF_UNKNOWN', control)
        self.assertEqual(control['WORK_LEAF_OBSERVER_PROVIDER_USAGE_GRACE_MS'], '1000')
        self.assertEqual(control['WORK_LEAF_OBSERVER_PROVIDER_USAGE_GRACE_OUTPUT_RESUME'], 'forward')
        self.assertEqual(control['WORK_LEAF_OBSERVER_PROJECT_LAYER_INVENTORY'], '1')
        old = load('runner_work_units')
        expected = old.run_environment(manifest, {**row, 'condition': 'control'}, inherited)
        self.assertEqual(control, expected)

    def test_old_engines_stay_byte_pinned_and_unmodified_in_other_instances(self):
        for name, expected in (
            ('runner_work_units.py', self.runner.ENGINE_SHA256),
            ('allocate_confirmation.py', self.allocator.LEGACY_SHA256),
        ):
            self.assertEqual(hashlib.sha256((HERE / name).read_bytes()).hexdigest(), expected)
        old = load('runner_work_units')
        self.assertEqual(old.EXPERIMENT_SCHEMA, 'work-leaf-bench-experiment-v2')
        self.assertEqual(old.CONDITIONS, ('control', 'buildable-work-unit-incremental'))

    def test_prepare_declares_v3_and_freezes_both_control_and_engine_without_admission(self):
        from argparse import Namespace
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp); source = root / 'source'; binaries = root / 'bin'
            source.mkdir(); binaries.mkdir()
            for name in self.runner.DRIVERS:
                (source / name).write_text('#!/bin/sh\nexit 0\n'); (source / name).chmod(0o700)
            for name in ('work-leaf', 'work-leaf-orchestrator', 'bench-observer'):
                (binaries / name).write_text('#!/bin/sh\nexit 0\n'); (binaries / name).chmod(0o700)
            for name, content in (('PROTOCOL.md', '# Fixture\n'), ('SCORER.json', '{}'),
                                  ('config.toml', 'model = "gpt-5.5"\n'),
                                  ('schedule.json', json.dumps(self.plan()))):
                (root / name).write_text(content)
            args = Namespace(phase_root=root / 'phase', source_repo=source, bin_dir=binaries,
                             observer_bin=binaries / 'bench-observer', runtime_root=root / 'runtime',
                             subscription_wrapper=self.runner.CANONICAL_WRAPPER, evidence=[],
                             identity_file=[], task_list_sha256='a' * 64, protocol=root / 'PROTOCOL.md',
                             scorer_config=root / 'SCORER.json', schedule=root / 'schedule.json',
                             global_config=root / 'config.toml', evidence_root=self.runner.REPO)
            with patch.object(self.runner._engine, 'git', side_effect=lambda _root, *a: '1' * 40 if a[0] == 'rev-parse' else ''):
                manifest = self.runner.prepare(args)
                self.runner.verify_manifest(args.phase_root)
                self.assertEqual(manifest['runner_sha256'], self.runner.ENGINE_SHA256)
                frozen_names = {Path(e['path']).name for e in manifest['files']}
                self.assertTrue({'runner_untracked_reads.py', 'runner_work_units.py',
                                 'allocate_untracked_reads.py', 'allocate_confirmation.py'} <= frozen_names)
                for row in manifest['schedule']:
                    record = json.loads(Path(row['experiment_manifest']).read_text())
                    self.assertEqual(record['schema'], 'work-leaf-bench-experiment-v3')
                self.assertFalse((args.phase_root / 'RUN-ONCE').exists())
                with self.assertRaises(FileExistsError): self.runner.prepare(args)


if __name__ == '__main__':
    unittest.main()
