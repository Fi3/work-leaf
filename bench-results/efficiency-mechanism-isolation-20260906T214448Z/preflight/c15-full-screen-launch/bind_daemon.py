"""Frozen C15 launch template -> actual-cwd manifest, before daemon execution.

No provider, source selection, private command, shared mutation or retry. The
qualified bridge supplies only its read-only configuration and live-census checks.
Input hashing is linear in bytes per fixed pass; census keeps its predecessor
bounds. Filesystem endpoint checks assume a non-hostile single operator, not
atomic snapshots or adversarial descriptor-exec containment.
"""
from datetime import datetime, timezone
import hashlib
import json
import os
from pathlib import Path
import re
import stat
import subprocess
import sys
import types

SCHEMA = 'work-leaf-c15-launch-template-v1'
BRIDGE_SHA = '6af6c3f24f96a7b95514a0db702a775b895a8d89f5dc8a2db9aa1329db9d4d26'
MAX_JSON = 32 * 1024 * 1024
__compiled_sha256__ = globals().get('__compiled_sha256__', hashlib.sha256(Path(__file__).read_bytes()).hexdigest())


def require(value, message):
    if not value: raise ValueError(message)


def sha(body): return hashlib.sha256(body).hexdigest()
def encoded(value): return (json.dumps(value, sort_keys=True, indent=2, allow_nan=False) + '\n').encode()
def value_sha(value): return sha(json.dumps(value, sort_keys=True, separators=(',', ':'), allow_nan=False).encode())
def now(): return datetime.now(timezone.utc).isoformat()


def canonical(raw, exists=True):
    require(type(raw) is str and raw.startswith('/'), 'absolute path required')
    path = Path(raw)
    require(str(path) == raw and '..' not in path.parts and path != Path('/'), 'normalized nonroot path required')
    require(path.resolve(strict=exists) == path, 'path alias forbidden')
    if not exists: require(path.parent.is_dir(), 'owned parent missing')
    return path


def file_sha(path):
    path = canonical(str(path))
    before = path.stat(follow_symlinks=False)
    require(stat.S_ISREG(before.st_mode), 'regular input required')
    digest = hashlib.sha256()
    with path.open('rb') as stream:
        for block in iter(lambda: stream.read(1024 * 1024), b''): digest.update(block)
        held = os.fstat(stream.fileno())
    after = path.stat(follow_symlinks=False)
    fields = ('st_dev', 'st_ino', 'st_size', 'st_mtime_ns', 'st_mode', 'st_nlink')
    require(all(getattr(before, k) == getattr(held, k) == getattr(after, k) for k in fields), 'input changed while hashing')
    return digest.hexdigest()


def ref(path): return {'path': str(path), 'sha256': file_sha(path)}


def verify_reference(value):
    require(type(value) is dict and set(value) == {'path', 'sha256'}, 'exact source reference required')
    require(type(value['sha256']) is str and re.fullmatch('[0-9a-f]{64}', value['sha256']), 'invalid digest')
    require(file_sha(value['path']) == value['sha256'], 'source digest differs')
    return canonical(value['path'])


def read_reference(value):
    path = verify_reference(value)
    require(path.stat().st_nlink == 1 and path.stat().st_size <= MAX_JSON, 'aliased or oversized source')
    body = path.read_bytes()
    require(len(body) <= MAX_JSON and sha(body) == value['sha256'], 'source bytes differ')
    return body


def decode(body):
    def pairs(rows):
        result = {}
        for key, value in rows:
            require(key not in result, 'duplicate JSON key')
            result[key] = value
        return result
    return json.loads(body, object_pairs_hook=pairs,
                      parse_constant=lambda _: (_ for _ in ()).throw(ValueError('nonfinite JSON')))


def publish(path, value, mode=0o400):
    path = canonical(str(path), exists=False)
    body = encoded(value)
    require(len(body) <= MAX_JSON, 'metadata publication bound')
    fd = os.open(path, os.O_CREAT | os.O_EXCL | os.O_NOFOLLOW | os.O_RDWR, 0o600)
    with os.fdopen(fd, 'w+b') as stream:
        stream.write(body); stream.flush(); os.fsync(stream.fileno()); stream.seek(0)
        require(stream.read() == body, 'publication readback differs')
        held, named = os.fstat(stream.fileno()), path.lstat()
        require((held.st_dev, held.st_ino) == (named.st_dev, named.st_ino), 'publication path changed')
        os.fchmod(stream.fileno(), mode)
    return {'path': str(path), 'sha256': sha(body)}


def verify_inputs(pins):
    require(type(pins) is dict and bool(pins), 'source inventory missing')
    for path, digest in pins.items(): verify_reference({'path': path, 'sha256': digest})


def load_bridge(source):
    require(source['sha256'] == BRIDGE_SHA, 'unqualified bridge source')
    body = read_reference(source)
    module = types.ModuleType('c15_launch_qualified_bridge')
    module.__file__ = source['path']; module.__compiled_sha256__ = sha(body)
    exec(compile(body, source['path'], 'exec'), module.__dict__)
    return module


def inspect_config(config, bridge, project, owned):
    return load_bridge(bridge).validate_config({'config_path': config['path'], 'config_sha256': config['sha256'],
        'project_root': str(project), 'operation_root': str(owned)}, False)


def validate_template(value):
    fields = {'schema', 'run_id', 'condition', 'evidence_path', 'runtime_root', 'private_root', 'output_root',
              'base_commit', 'bridge', 'python', 'config', 'daemon', 'source_sha256', 'required_environment'}
    require(type(value) is dict and set(value) == fields and value['schema'] == SCHEMA, 'launch template schema differs')
    require(value['condition'] == 'private-test-first' and type(value['run_id']) is str
            and re.fullmatch('[A-Za-z0-9][A-Za-z0-9_-]{0,95}', value['run_id']), 'undeclared condition/run')
    require(type(value['base_commit']) is str and re.fullmatch('[0-9a-f]{40}', value['base_commit']), 'base identity differs')
    require(type(value['required_environment']) is dict and all(type(k) is str and type(v) is str
            for k, v in value['required_environment'].items()), 'environment contract differs')
    return value


def overlaps(a, b): return a == b or a in b.parents or b in a.parents


def bind(template, template_ref, argv, environment, output):
    template = validate_template(template)
    require(template['output_root'] == str(output), 'output ownership differs')
    require(all(environment.get(k) == v for k, v in template['required_environment'].items()), 'required environment differs')
    require(environment.get('WORK_LEAF_BENCH_RUN_ID') == template['run_id']
            and environment.get('WORK_LEAF_BENCH_EXPERIMENT') == '1'
            and environment.get('WORK_LEAF_BENCH_EXPERIMENT_MANIFEST') == template_ref['path'], 'activation identity differs')
    project = canonical(str(Path.cwd()))
    runtime = canonical(template['runtime_root'])
    require(project.parent.parent == runtime and project.name == 'repo', 'cwd is not the declared driver checkout')
    private = canonical(template['private_root'])
    require(private.is_dir() and not any(private.iterdir()), 'private evidence root is not empty')
    require(not any(overlaps(private, p) or overlaps(output, p) for p in (runtime, project))
            and not overlaps(private, output), 'private/binding roots overlap driver cleanup or each other')
    bundle = project.parent / 'context-bundles'
    require(environment.get('WORK_LEAF_CONTEXT_BUNDLE_DIR') == str(bundle), 'ordinary bundle namespace differs')
    canonical(str(bundle), exists=bundle.exists())
    temporary = canonical(str(project.parent / 'tmp'))
    require(environment.get('TMPDIR') == str(temporary)
            and environment.get('WORK_LEAF_COMMAND_TMPDIR') == str(temporary), 'ordinary command temp differs')
    for key in ('python', 'daemon', 'bridge', 'config'): verify_reference(template[key])
    require(os.access(template['daemon']['path'], os.X_OK) and os.access(template['python']['path'], os.X_OK), 'executable permission missing')
    with open(template['daemon']['path'], 'rb') as stream: require(stream.read(4) == b'\x7fELF', 'actual daemon must be ELF')
    verify_inputs(template['source_sha256'])
    c, modules, _capsule, pins = inspect_config(template['config'], template['bridge'], project, output)
    require(c['qualification_implementation'] is False and c['timeout_seconds'] == 120
            and c['operation_timeout_seconds'] == 300, 'full-project private bounds differ')
    require(all(template['source_sha256'].get(path) == digest for path, digest in pins.items()), 'configuration dependency absent from template')
    require(not any(overlaps(private, Path(path)) for path in pins), 'private root overlaps immutable input')
    census = modules['live_selection'].census(project, template['base_commit'], c['overlays'])
    require(modules['live_selection'].census(project, template['base_commit'], c['overlays']) == census, 'project changed during startup census')
    descriptor = {'root': str(private), 'project_root': str(project)}
    for key in ('python', 'bridge', 'config'):
        descriptor[key + '_path'] = template[key]['path']; descriptor[key + '_sha256'] = template[key]['sha256']
    manifest = {key: template[key] for key in ('run_id', 'condition', 'evidence_path')}
    manifest.update(schema='work-leaf-bench-experiment-v6', private_preview=descriptor)
    pins = {**template['source_sha256'], **pins, template_ref['path']: template_ref['sha256']}
    for key in ('python', 'daemon', 'bridge', 'config'): pins[template[key]['path']] = template[key]['sha256']
    pins[str(Path(__file__).resolve())] = __compiled_sha256__
    verify_inputs(pins)
    manifest_ref = publish(output / 'EXPERIMENT.json', manifest)
    forwarded = dict(environment, WORK_LEAF_BENCH_EXPERIMENT_MANIFEST=manifest_ref['path'])
    receipt = dict(schema='work-leaf-c15-daemon-binding-v1', status='ready_for_exec', bound_at=now(),
        run_id=template['run_id'], condition=template['condition'], template=template_ref, manifest=manifest_ref,
        actual_daemon=template['daemon'], observed_wrapper=ref(canonical(argv[0])), project_census=census,
        private_root=str(private), environment_sha256=value_sha(environment),
        forwarded_environment_sha256=value_sha(forwarded),
        environment_changed_keys=['WORK_LEAF_BENCH_EXPERIMENT_MANIFEST'], argv_sha256=value_sha(argv),
        source_sha256=pins, provider_started_by_binding=False, private_execution_performed=False)
    return receipt, forwarded


def main(argv, templates, python_ref, environment=None, exec_fn=None):
    environment = dict(os.environ if environment is None else environment)
    output = None; claimed = False
    try:
        selected = templates.get(environment.get('WORK_LEAF_BENCH_RUN_ID'))
        require(selected is not None and set(selected) == {'template', 'output_root'}, 'run absent from frozen bootstrap')
        output = canonical(selected['output_root'], exists=False)
        output.mkdir(); claimed = True
        publish(output / 'ATTEMPT.json', dict(started_at=now(), template=selected['template'],
            environment_sha256=value_sha(environment), argv_sha256=value_sha(argv)))
        require(canonical(str(Path(sys.executable).resolve())) == verify_reference(python_ref), 'executing Python differs')
        template = decode(read_reference(selected['template']))
        require(template['python'] == python_ref, 'template Python differs')
        receipt, forwarded = bind(template, selected['template'], argv, environment, output)
        publish(output / 'BINDING.json', receipt)
        verify_inputs(receipt['source_sha256'])
        verify_reference(receipt['manifest'])
        (os.execve if exec_fn is None else exec_fn)(template['daemon']['path'], argv, forwarded)
        return 0  # Only a synthetic exec callback returns.
    except (OSError, ValueError, KeyError, TypeError, subprocess.SubprocessError) as error:
        if claimed:
            try: publish(output / 'ERROR.json', dict(failed_at=now(), error_type=type(error).__name__,
                error=str(error), daemon_exec_success_proven=False, attempt_retained=True))
            except (OSError, ValueError): pass
        print('C15 daemon binding failed: ' + type(error).__name__, file=sys.stderr)
        return 2
