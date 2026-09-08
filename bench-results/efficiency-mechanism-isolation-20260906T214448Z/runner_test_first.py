#!/usr/bin/env python3
"""Three modified-only C15 workflows over the exact retained supervisor.

The static experiment files are immutable launch templates. A create-new host
binding supplies the actual driver checkout without altering scheduling, provider
environments, observation, trust, terminal outcomes or the original driver bytes.
"""
import argparse
import copy
import hashlib
import json
import os
from pathlib import Path
import subprocess
import sys
import types

HERE = Path(__file__).resolve().parent
REPO = HERE.parents[1]
ADAPTER_PATH = Path(__file__).resolve()
ENGINE_PATH = HERE / 'runner_work_units.py'
ENGINE_SHA256 = '2f019c297a2e436c54241ad0643d4f105db6ec8bc782b1cac456f3acde0bc98e'
BINDING_PATH = HERE / 'preflight/c15-full-screen-launch/bind_daemon.py'
BRIDGE_PATH = HERE / 'preflight/c15-runtime-bridge/bridge.py'
QUALIFIED_CONFIG_SHA256 = '5e1e5a204445f7ccaec057f446d87ce060fe98783f560dfd577ea0f3f0fd982a'
EXPERIMENT_SCHEMA = 'work-leaf-bench-experiment-v6'
CONDITIONS = ('private-test-first',)
METHOD = 'fixed three modified workflows; only launch order randomized; six saved baselines reused'


def load_source(path, expected=None):
    data = path.read_bytes(); digest = hashlib.sha256(data).hexdigest()
    if path.resolve() != path or path.is_symlink() or expected is not None and digest != expected:
        raise ValueError('executing source identity differs: ' + str(path))
    module = types.ModuleType('test_first_' + path.stem)
    module.__file__ = str(path); module.__compiled_sha256__ = digest
    exec(compile(data, str(path), 'exec'), module.__dict__)
    return module, digest


ADAPTER_SHA256 = hashlib.sha256(ADAPTER_PATH.read_bytes()).hexdigest()
_engine, _ = load_source(ENGINE_PATH, ENGINE_SHA256)
binding, BINDING_SHA256 = load_source(BINDING_PATH)
_base_validate = _engine.validate_plan
_base_verify = _engine.verify_manifest
DRIVERS = _engine.DRIVERS
CANONICAL_WRAPPER = _engine.CANONICAL_WRAPPER


def make_plan(phase):
    if not _engine.identifier(phase) or len(phase) > 67:
        raise ValueError('phase must leave room for explicit run and block identifiers')
    block = phase + '-block-01'
    return {'phase': phase, 'phase_kind': 'screening',
        'runs': [{'run_id': f'{phase}-workflow-{index:03d}', 'condition': CONDITIONS[0],
                  'wave': 1, 'block_id': block} for index in range(1, 4)],
        'randomization': {'method': METHOD, 'unit': 'workflow', 'scheme': 'within_block',
            'mixed_waves': False, 'block_condition_counts': {block: {CONDITIONS[0]: 3}}}}


def validate_plan(plan):
    _base_validate(plan)
    if plan != make_plan(plan['phase']):
        raise ValueError('exactly three C15 modified workflows required; no controls or replacements')


def dependency_pins():
    return {ADAPTER_PATH: ADAPTER_SHA256, ENGINE_PATH: ENGINE_SHA256, BINDING_PATH: BINDING_SHA256}


def verify_current_dependencies():
    for path, digest in dependency_pins().items():
        if binding.file_sha(path) != digest: raise ValueError('loaded adapter dependency changed')


def copy_new(source, destination, executable=False):
    original = binding.ref(source)
    with source.open('rb') as incoming, destination.open('xb') as outgoing:
        for chunk in iter(lambda: incoming.read(1024 * 1024), b''): outgoing.write(chunk)
        outgoing.flush(); os.fsync(outgoing.fileno())
    destination.chmod(0o500 if executable else 0o400)
    if binding.file_sha(destination) != original['sha256'] or binding.file_sha(source) != original['sha256']:
        raise ValueError('frozen copy differs from captured source')
    return binding.ref(destination)


def bootstrap(index, python, helper):
    # The immutable bootstrap binds template digests; templates need not contain
    # their bootstrap's hash, avoiding a self-referential hash dependency.
    return (f'#!{python["path"]} -I\n'
        'import hashlib, pathlib, sys, types\n'
        'sys.dont_write_bytecode = True\n'
        f'p = {helper["path"]!r}\n'
        f'expected = {helper["sha256"]!r}\n'
        'path = pathlib.Path(p)\n'
        'if str(path.resolve()) != p or path.is_symlink(): raise SystemExit("binding source alias")\n'
        'body = path.read_bytes()\n'
        'if hashlib.sha256(body).hexdigest() != expected: raise SystemExit("binding source changed")\n'
        'm = types.ModuleType("frozen_c15_binding"); m.__file__ = p; m.__compiled_sha256__ = expected\n'
        'exec(compile(body, p, "exec"), m.__dict__)\n'
        f'raise SystemExit(m.main(sys.argv, {index!r}, {python!r}))\n').encode()


def template_for(phase, row, sources, refs):
    env = _engine.run_environment({'study': phase.parent.parent.name if phase.parent.name == 'phases' else phase.name,
        'provider_dir': str(phase / 'infrastructure/provider'), 'bin_dir': str(phase / 'infrastructure/bin')},
        row, {'PATH': '<inherited PATH>'})
    return dict(schema=binding.SCHEMA, run_id=row['run_id'], condition=CONDITIONS[0],
        evidence_path=row['prompt_trace'], runtime_root=row['runtime_dir'],
        private_root=str(phase / 'private-preview' / row['run_id']),
        output_root=str(phase / 'launches' / row['run_id']), base_commit=_engine.BASE_COMMIT,
        **refs, source_sha256=sources, required_environment={k: v for k, v in env.items() if k != 'PATH'})


def prepare(args):
    verify_current_dependencies()
    if args.evidence_root.resolve() != REPO: raise ValueError('evidence root must preserve actual dependency paths')
    phase = args.phase_root.resolve(); source = args.source_repo.resolve()
    plan = binding.decode(args.schedule.read_bytes()); validate_plan(plan)
    if binding.file_sha(args.bridge_config) != QUALIFIED_CONFIG_SHA256:
        raise ValueError('full-project qualified CONFIG differs')
    python = binding.ref(binding.canonical(str(args.python)))
    if not os.access(python['path'], os.X_OK): raise ValueError('Python is not executable')
    for name in ('work-leaf', 'work-leaf-orchestrator'):
        path = binding.canonical(str(args.bin_dir / name))
        with path.open('rb') as handle:
            if handle.read(4) != b'\x7fELF' or not os.access(path, os.X_OK):
                raise ValueError('actual Work Leaf runtime must be native ELF')
    phase.mkdir(parents=True, exist_ok=True)
    for name in ('PHASE-MANIFEST.json', 'SCHEDULE.json', 'infrastructure', 'experiments', 'RUN-ONCE',
                 'launch-inputs', 'launches', 'private-preview', 'launch-validation'):
        if (phase / name).exists() or (phase / name).is_symlink(): raise FileExistsError('phase already prepared: ' + name)
    inputs = phase / 'launch-inputs'; inputs.mkdir()
    validation = phase / 'launch-validation'; validation.mkdir()
    (phase / 'launches').mkdir(); (phase / 'private-preview').mkdir()
    config = copy_new(args.bridge_config, inputs / 'BRIDGE-CONFIG.json')
    helper = copy_new(BINDING_PATH, inputs / 'bind_daemon.py')
    daemon = copy_new(args.bin_dir / 'work-leaf-orchestrator', inputs / 'work-leaf-orchestrator-real', True)
    bridge = binding.ref(BRIDGE_PATH)
    _c, _modules, _capsule, sources = binding.inspect_config(config, bridge, source, validation)
    sources.update({ref['path']: ref['sha256'] for ref in (python, helper, daemon)})
    binding.verify_inputs(sources)
    rows = _engine.schedule(phase, source, args.runtime_root.resolve(), plan)
    refs = dict(config=config, daemon=daemon, bridge=bridge, python=python)
    templates = {Path(row['experiment_manifest']): template_for(phase, row, sources, refs) for row in rows}
    index = {value['run_id']: {'template': {'path': str(path), 'sha256': binding.sha(binding.encoded(value))},
                             'output_root': value['output_root']} for path, value in templates.items()}
    for value in templates.values(): Path(value['private_root']).mkdir()
    staged = inputs / 'driver-bin'; staged.mkdir()
    copy_new(args.bin_dir / 'work-leaf', staged / 'work-leaf', True)
    shim = staged / 'work-leaf-orchestrator'
    with shim.open('xb') as stream: stream.write(bootstrap(index, python, helper)); stream.flush(); os.fsync(stream.fileno())
    shim.chmod(0o500)
    prepared = copy.copy(args); prepared.bin_dir = staged
    prepared.evidence = list(args.evidence) + list(dependency_pins())
    generated = [config['path'], helper['path'], daemon['path'], str(staged / 'work-leaf'), str(shim)]
    prepared.identity_file = sorted({Path(p).resolve() for p in [*args.identity_file, *sources, *generated]})
    ordinary_write = _engine.write_new
    published = set()
    def write_template(path, value):
        path = Path(path)
        if path in templates:
            expected = {k: templates[path][k] for k in ('run_id', 'condition', 'evidence_path')}
            expected['schema'] = EXPERIMENT_SCHEMA
            if value != expected or path in published: raise ValueError('unexpected engine template publication')
            published.add(path)
            return ordinary_write(path, templates[path])
        return ordinary_write(path, value)
    _engine.write_new = write_template
    try: manifest = _engine.prepare(prepared)
    finally: _engine.write_new = ordinary_write
    if published != templates.keys(): raise ValueError('missing engine template publication')
    verify_manifest(phase)
    return manifest


def verify_manifest(phase):
    verify_current_dependencies()
    phase = Path(phase).resolve(); manifest = _base_verify(phase)
    entries = {entry['path']: entry for entry in manifest['files']}
    for path, digest in dependency_pins().items():
        frozen = phase / 'infrastructure/evidence' / path.relative_to(REPO)
        entry = entries.get(str(frozen))
        if entry is None or entry['role'] != 'frozen-evidence' or entry['sha256'] != digest:
            raise ValueError('executing adapter dependency absent from frozen inventory')
    index = {}; shared = None
    for row in manifest['schedule']:
        path = Path(row['experiment_manifest'])
        template = binding.validate_template(binding.decode(binding.read_reference({'path': str(path), 'sha256': entries[str(path)]['sha256']})))
        refs = {k: template[k] for k in ('config', 'daemon', 'bridge', 'python')}
        if template != template_for(phase, row, template['source_sha256'], refs):
            raise ValueError('template identity/paths/environment differ')
        helper = {'path': str(phase / 'launch-inputs/bind_daemon.py'), 'sha256': BINDING_SHA256}
        required = {**template['source_sha256'], **{r['path']: r['sha256'] for r in [*refs.values(), helper]}}
        for source, digest in required.items():
            if source not in entries or entries[source]['sha256'] != digest:
                raise ValueError('launch dependency absent from immutable manifest')
        if template['config']['sha256'] != QUALIFIED_CONFIG_SHA256 or template['bridge']['sha256'] != binding.BRIDGE_SHA:
            raise ValueError('unqualified private input')
        identity = (refs, helper, template['source_sha256'])
        if shared is not None and identity != shared: raise ValueError('per-run nonfactor input mismatch')
        shared = identity
        index[row['run_id']] = {'template': {'path': str(path), 'sha256': entries[str(path)]['sha256']},
                               'output_root': template['output_root']}
    expected = bootstrap(index, shared[0]['python'], shared[1])
    if (phase / 'infrastructure/bin/work-leaf-orchestrator').read_bytes() != expected:
        raise ValueError('daemon bootstrap differs from exact admitted template bindings')
    return manifest


_engine.CONDITIONS = CONDITIONS
_engine.EXPERIMENT_SCHEMA = EXPERIMENT_SCHEMA
_engine.validate_plan = validate_plan
_engine.verify_manifest = verify_manifest
run_environment = _engine.run_environment
run_phase = _engine.run_phase


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    modes = parser.add_subparsers(dest='mode', required=True)
    plan = modes.add_parser('plan')
    plan.add_argument('--phase', required=True); plan.add_argument('--output', type=Path, required=True)
    prep = modes.add_parser('prepare')
    for name in ('phase-root', 'source-repo', 'bin-dir', 'observer-bin', 'subscription-wrapper', 'runtime-root',
                 'schedule', 'protocol', 'scorer-config', 'global-config', 'bridge-config', 'python'):
        prep.add_argument('--' + name, type=Path, required=True)
    prep.add_argument('--task-list-sha256', required=True)
    prep.add_argument('--evidence-root', type=Path, default=REPO)
    prep.add_argument('--evidence', type=Path, action='append', default=[])
    prep.add_argument('--identity-file', type=Path, action='append', default=[])
    for mode in ('verify', 'run'): modes.add_parser(mode).add_argument('--phase-root', type=Path, required=True)
    args = parser.parse_args()
    try:
        if args.mode == 'plan': _engine.write_new(args.output, make_plan(args.phase))
        elif args.mode == 'prepare': prepare(args)
        elif args.mode == 'verify': verify_manifest(args.phase_root)
        else:
            result = run_phase(args.phase_root)
            print(json.dumps(result, sort_keys=True))
            return 0 if (all(row['launcher_exit_code'] == 0 for row in result['runs'])
                and not result['unexplained_config_drift_detected']
                and not result['pending_config_attestation_at_finish']
                and not result['frozen_input_integrity_errors']) else 1
        print(json.dumps({'mode': args.mode, 'provider_work_started': False}))
        return 0
    except (OSError, ValueError, KeyError, subprocess.CalledProcessError) as error:
        print('C15 supervisor: ' + type(error).__name__ + ': ' + str(error), file=sys.stderr)
        return 2


if __name__ == '__main__': raise SystemExit(main())
