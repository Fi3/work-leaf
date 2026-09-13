#!/usr/bin/env python3
"""One disarmed full-workflow completion pilot; no replacements or retries."""
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
SELF = Path(__file__).resolve()
ADAPTER_SHA = hashlib.sha256(SELF.read_bytes()).hexdigest()
ENGINE_PATH = HERE.parents[2] / 'efficiency-measurement-gate-20260906' / 'first_batch.py'
ENGINE_SHA = 'e21dcdde038630cc9a6e6bd5048fd78af60e59799c33377ce2acec8e85e55132'
IDS = ['standalone-global-hunk-pilot-001']
GENERATED_NAME = 'bench-three-features-serialized-host-custody'
CONDITION = 'standalone-global-hunk-pilot'
PHASE = 'standalone-global-hunk-pilot-01'


def digest(path):
    h = hashlib.sha256()
    with Path(path).open('rb') as handle:
        for block in iter(lambda: handle.read(1024*1024), b''):
            h.update(block)
    return h.hexdigest()


def loaded_identity():
    if digest(SELF) != ADAPTER_SHA or digest(ENGINE_PATH) != ENGINE_SHA:
        raise ValueError('loaded adapter/engine source identity changed')


def configuration(path):
    path = Path(path)
    if not path.is_absolute() or path.is_symlink() or not path.is_file():
        raise ValueError('global configuration must be an explicit regular absolute file')
    return {'path': str(path), 'sha256': digest(path), 'size': path.stat().st_size}


def load_engine():
    data = ENGINE_PATH.read_bytes()
    if hashlib.sha256(data).hexdigest() != ENGINE_SHA:
        raise ValueError('original engine does not match its reviewed source')
    source = data.decode('utf-8')
    substitutions = [
        ('        freeze(source / name, infrastructure / "drivers" / name, "frozen-driver")',
         '        if name == "bench-candidate-common":\n'
         '            freeze(source / name, infrastructure / "original-drivers" / name, "original-artifact-common")\n'
         '            record(args.artifact_common_source, "artifact-common-source")\n'
         '            freeze(args.artifact_common_source, infrastructure / "drivers" / name, "private-artifact-common")\n'
         '        else:\n'
         '            freeze(source / name, infrastructure / "drivers" / name, "frozen-driver")', 1),
        ('"supervisor_wall_timeout_seconds": 86400', '"supervisor_wall_timeout_seconds": 5400', 1),
        ('manifest.get("supervisor_wall_timeout_seconds") != 86400',
         'manifest.get("supervisor_wall_timeout_seconds") != 5400', 1),
        ('if sha256(args.subscription_wrapper) != sha256(canonical_wrapper):',
         'if sha256(args.subscription_wrapper) != "8ad1d261979029fec24cf4e143aaf6afc6c688056c6bc702c824b3538b92ff32":', 1),
        ('FIRST-BATCH-MANIFEST.json', 'PHASE-MANIFEST.json', 5),
        ('FIRST-BATCH-RESULT.json', 'PHASE-RESULT.json', 1),
        ('"maximum_concurrent_workflows": 2', '"maximum_concurrent_workflows": 1', 1),
        ('manifest.get("maximum_concurrent_workflows") != 2', 'manifest.get("maximum_concurrent_workflows") != 1', 1),
        ('"pause_after_pair"', '"pause_after_wave"', 2),
        ('["direct-001", "work-leaf-001"]', repr(IDS), 2),
        ('"workflow_count": 2,', '"workflow_count": 1,', 1),
        ('popen([row["driver"]],', 'popen(row["argv"],', 1),
        ('    manifest = {"schema_version": 1, "study": batch.name, "prepared_at": now(),',
         '    record(args.generated_driver, "generated-source-driver")\n'
         '    freeze(args.generated_driver, infrastructure / "drivers" / GENERATED_NAME, "generated-driver", True)\n'
         '    record(args.host_source, "host-source")\n'
         '    freeze(args.host_source, infrastructure / "drivers" / "host_custody.py", "standalone-host")\n'
         '    manifest = {"schema_version": 1, "study": batch.name, "prepared_at": now(),', 1),
        ('    for row in manifest["schedule"]:\n        row["environment_overrides"]',
         '    manifest.update(execution_authorized=False, adapter_sha256=ADAPTER_SHA,\n'
         '                    adapter_path=str(SELF), engine_path=str(ENGINE_PATH),\n'
         '                    generated_source=str(args.generated_driver.resolve()),\n'
         '                    host_source=str(args.host_source.resolve()),\n'
         '                    artifact_common_source=str(args.artifact_common_source.resolve()),\n'
         '                    configuration=configuration(args.global_config))\n'
         '    for row in manifest["schedule"]:\n        row["environment_overrides"]', 1),
    ]
    for old, new, count in substitutions:
        if source.count(old) != count:
            raise ValueError(f'original source anchor count differs: {old!r}')
        source = source.replace(old, new)
    module = types.ModuleType('pinned_three_direct_engine')
    module.__file__ = str(ENGINE_PATH)
    module.__dict__.update(GENERATED_NAME=GENERATED_NAME, ADAPTER_SHA=ADAPTER_SHA,
                           SELF=SELF, ENGINE_PATH=ENGINE_PATH, configuration=configuration,
                           ENGINE_SUBSTITUTIONS=substitutions, ENGINE_SOURCE=source)
    exec(compile(source, str(ENGINE_PATH), 'exec'), module.__dict__)
    return module


E = load_engine()
_prepare = E.prepare
_verify = E.verify_manifest
_run = E.run_batch
_environment = E.run_environment


def schedule(batch, source, runtime):
    del source
    rows = []
    driver = str(batch / 'infrastructure' / 'drivers' / GENERATED_NAME)
    for run_id in IDS:
        results = batch / 'runs' / run_id
        artifact = results / f'{run_id}-three-feature-sequential-bench-artifacts'
        rows.append({'run_id': run_id, 'condition': CONDITION,
                     'workflow': 'direct-sequential-serialized-host-custody', 'driver': driver,
                     'argv': [driver, 'sequential'], 'results_dir': str(results),
                     'runtime_dir': str(runtime/run_id), 'artifact': str(artifact),
                     'report': str(artifact/'report.json')})
    return rows


def validate_schedule(rows):
    if (not isinstance(rows, list) or len(rows) != 1
            or [r.get('run_id') for r in rows] != IDS
            or any(r.get('condition') != CONDITION for r in rows)):
        raise ValueError('exactly the one declared standalone completion-pilot row is required')


def run_environment(manifest, row, inherited):
    env = _environment(manifest, {**row, 'condition': 'direct'}, inherited)
    env['WORK_LEAF_BENCH_PAIR_ID'] = manifest['study']
    env['WORK_LEAF_DIAGNOSTIC_SOURCE_REPO'] = manifest['source_repo']
    return env


E.schedule = schedule
E.validate_schedule = validate_schedule
E.run_environment = run_environment


def prepare(args):
    loaded_identity()
    args = copy.copy(args)
    if args.batch_root.resolve().name != PHASE:
        raise ValueError('fresh completion-pilot phase identity is required')
    if args.artifact_common_source.is_symlink() or not args.artifact_common_source.is_file():
        raise ValueError('artifact common source must be an ordinary regular file')
    args.artifact_common_source = args.artifact_common_source.resolve(strict=True)
    args.generated_driver = args.generated_driver.resolve()
    if args.host_source.is_symlink() or not args.host_source.is_file():
        raise ValueError('host source must be an ordinary regular file')
    args.host_source = args.host_source.resolve(strict=True)
    args.evidence = list(args.evidence) + [SELF]
    args.identity_file = list(dict.fromkeys(list(args.identity_file) + [SELF, ENGINE_PATH]))
    config = configuration(args.global_config)
    if any(str(Path(p).resolve()) == config['path']
           for p in args.evidence + args.identity_file + [args.artifact_common_source]):
        raise ValueError('mutable global configuration must remain outside immutable files')
    if not os.access(args.generated_driver, os.X_OK):
        raise ValueError('explicit generated driver is not executable')
    result = _prepare(args)
    loaded_identity()
    return result


def verify_manifest(batch):
    loaded_identity()
    manifest = _verify(Path(batch).resolve())
    if Path(batch).resolve().name != PHASE or manifest.get('study') != PHASE:
        raise ValueError('fresh completion-pilot phase identity is required')
    if (manifest.get('adapter_sha256') != ADAPTER_SHA or manifest.get('adapter_path') != str(SELF)
            or manifest.get('engine_path') != str(ENGINE_PATH)
            or type(manifest.get('execution_authorized')) is not bool):
        raise ValueError('adapter identity or explicit authority flag differs')
    required = {str(SELF), str(ENGINE_PATH), manifest['generated_source'],
                str(Path(batch).resolve()/'infrastructure/drivers'/GENERATED_NAME),
                manifest['host_source'], str(Path(batch).resolve()/'infrastructure/drivers/host_custody.py'),
                manifest['artifact_common_source'],
                str(Path(batch).resolve()/'infrastructure/drivers/bench-candidate-common'),
                str(Path(batch).resolve()/'infrastructure/original-drivers/bench-candidate-common')}
    index = {r['path']: r for r in manifest['files']}
    if len(index) != len(manifest['files']) or not required <= index.keys():
        raise ValueError('generated/adapter source population is missing or duplicated')
    private_common = str(Path(batch).resolve()/'infrastructure/drivers/bench-candidate-common')
    original_common = str(Path(batch).resolve()/'infrastructure/original-drivers/bench-candidate-common')
    source_common = str(Path(manifest['source_repo'])/'bench-candidate-common')
    if (index[private_common]['sha256'] != index[manifest['artifact_common_source']]['sha256']
            or index[original_common]['sha256'] != index[source_common]['sha256']):
        raise ValueError('private or original artifact common copy differs from its source')
    # The adapter evidence is copied by the inherited preparation routine.
    # Its live source must independently remain represented in the manifest.
    if configuration(manifest['configuration']['path']) != manifest['configuration']:
        raise ValueError('global configuration differs from prelaunch snapshot')
    return manifest


E.verify_manifest = verify_manifest


def run_batch(batch, popen=None):
    batch = Path(batch).resolve()
    manifest = verify_manifest(batch)
    manifest_before = digest(batch/'PHASE-MANIFEST.json')
    if manifest['execution_authorized'] is not True:
        raise ValueError('phase is prepared but not authorized for generation')
    for name in ('RUN-ONCE', 'PHASE-RESULT.json', 'score-manifest.json', 'POST-RUN-IDENTITY.json'):
        if (batch/name).exists():
            raise FileExistsError(f'one-shot output already exists: {name}')
    try:
        return _run(batch, popen=popen)
    finally:
        if (batch/'RUN-ONCE').exists():
            errors = []
            for row in manifest['files']:
                try:
                    if digest(row['path']) != row['sha256']:
                        errors.append({'path': row['path'], 'error': 'source changed'})
                except OSError as error:
                    errors.append({'path': row['path'], 'error': type(error).__name__})
            try:
                manifest_after = digest(batch/'PHASE-MANIFEST.json')
            except OSError:
                manifest_after = None
            if manifest_after != manifest_before:
                errors.append({'path': str(batch/'PHASE-MANIFEST.json'), 'error': 'manifest changed'})
            try:
                after = configuration(manifest['configuration']['path'])
                config_error = None
            except (OSError, ValueError) as error:
                after, config_error = None, str(error)
            E.write_new(batch/'POST-RUN-IDENTITY.json', {
                'manifest_before_sha256': manifest_before, 'manifest_sha256': manifest_after,
                'immutable_errors': errors, 'configuration_before': manifest['configuration'],
                'configuration_after': after, 'configuration_error': config_error,
                'configuration_drift': after != manifest['configuration'],
                'configuration_drift_qualified': False,
                'interpretation': 'No trust-transition qualification is performed; original outcomes remain separate.'})


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    sub = parser.add_subparsers(dest='mode', required=True)
    prep = sub.add_parser('prepare')
    for name in ('batch-root','source-repo','bin-dir','observer-bin','subscription-wrapper',
                 'runtime-root','generated-driver','global-config','host-source','artifact-common-source'):
        prep.add_argument('--'+name,type=Path,required=True)
    prep.add_argument('--task-list-sha256',required=True)
    prep.add_argument('--evidence',type=Path,action='append',default=[])
    prep.add_argument('--identity-file',type=Path,action='append',default=[])
    for name in ('verify','run'):
        sub.add_parser(name).add_argument('--batch-root',type=Path,required=True)
    args = parser.parse_args()
    try:
        if args.mode == 'prepare':prepare(args)
        elif args.mode == 'verify':verify_manifest(args.batch_root)
        else:
            result=run_batch(args.batch_root)
            print(json.dumps({'runs':result['runs'],'collection_paused':True}))
            return 0 if all(r['launcher_exit_code']==0 for r in result['runs']) else 1
        print(json.dumps({'mode':args.mode,'batch_root':str(args.batch_root),'provider_work_started':False}))
        return 0
    except (OSError,ValueError,KeyError,subprocess.CalledProcessError) as error:
        print(f'standalone completion-pilot runner: {type(error).__name__}: {error}',file=sys.stderr)
        return 2


if __name__=='__main__':raise SystemExit(main())
