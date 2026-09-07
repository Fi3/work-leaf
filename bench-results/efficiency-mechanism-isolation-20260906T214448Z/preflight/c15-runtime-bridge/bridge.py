"""Private C15 bridge: selected objects, held tests, factual confined feedback.

Only main's validate/capture/test operations belong to the runtime transport.
Implementation continuation is qualification-only. No provider, promotion, ACK,
ordinary-tree writes, retry, implicit environment discovery, or semantic RED verdict.
Source work is O(P*B) across proposals, with constant source passes and sorted maps;
the unchanged executor has bounded O(M**2) mount checks (M <= 32). Git/history work
and postcreation bundle limits retain predecessor qualifications. Deadlines are
cooperative between bounded Git children and during executor runs, not a hard
bound on kernel/filesystem calls or trusted source hashing.
"""
import copy
import hashlib
import json
import math
import os
from pathlib import Path
import re
import stat
import subprocess
import sys
import threading
import time
import types
import unicodedata


HERE = Path(__file__).resolve().parent
__compiled_sha256__ = globals().get('__compiled_sha256__', hashlib.sha256(Path(__file__).read_bytes()).hexdigest())
HELPER_FILES = {
    'executor': ('c15-private-executor/executor.py', '16e3238e1bd647c9d4780db97c554ddf9579226c913995e2e18873bf7fcb24ae'),
    'materializer': ('c15-private-materialization/materialize.py', '87546793b76bc7324af55479be47a1e34abbeffb8f2787547d021575a51efab2'),
    'project_inputs': ('c15-project-qualification/project_inputs.py', 'b2f2905b3ffe9a0e356577d78eae2249b6c16a6919b0a359d731ab2e2ed0f862'),
    'live_selection': ('c15-live-selection/live_selection.py', '0614db9a5c874cd5f623997ce3433edbd2bc0ff3585c23913dd9ede9705d3a55')}
DEFAULT_HELPERS = {key: {'path': str(HERE.parent / name), 'sha256': digest}
                   for key, (name, digest) in HELPER_FILES.items()}
MAX_INPUT = 32 * 1024 * 1024
MAX_FILE = 512 * 1024 * 1024
SCHEMA = 'work-leaf-private-preview-bridge-v1'


def require(value, message):
    if not value: raise ValueError(message)


def sha(body): return hashlib.sha256(body).hexdigest()


def canonical(raw, *, exists=True):
    require(type(raw) is str and raw.startswith('/'), 'absolute path required')
    path = Path(raw)
    require(str(path) == raw and '..' not in path.parts and path != Path('/'), 'normalized nonroot path required')
    require(path.resolve(strict=exists) == path, 'path alias forbidden')
    if not exists: require(path.parent.is_dir(), 'missing owned parent')
    return path


def overlaps(a, b): return a == b or a in b.parents or b in a.parents


def read(path, maximum=MAX_FILE, *, independent=True):
    path = canonical(str(path)); before = path.lstat()
    require(stat.S_ISREG(before.st_mode) and (not independent or before.st_nlink == 1) and before.st_size <= maximum,
            'nonregular/aliased/oversized input')
    with path.open('rb') as stream:
        body = stream.read(maximum + 1); after = os.fstat(stream.fileno())
    fields = ('st_dev', 'st_ino', 'st_size', 'st_mtime_ns', 'st_mode', 'st_nlink')
    require(len(body) <= maximum and all(getattr(before, k) == getattr(after, k) for k in fields), 'input drift/bound')
    return body


def reference(value, *, independent=True):
    require(type(value) is dict and set(value) == {'path', 'sha256'}, 'exact source reference required')
    require(type(value['sha256']) is str and re.fullmatch('[0-9a-f]{64}', value['sha256']), 'invalid source hash')
    body = read(value['path'], independent=independent); require(sha(body) == value['sha256'], 'source hash differs')
    return body


def decode(body):
    def pairs(rows):
        result = {}
        for key, value in rows:
            require(key not in result, 'duplicate JSON key'); result[key] = value
        return result
    return json.loads(body, object_pairs_hook=pairs, parse_constant=lambda _: (_ for _ in ()).throw(ValueError('nonfinite JSON')))


def load_helper(name, source):
    require(name in HELPER_FILES and source['sha256'] == HELPER_FILES[name][1], 'unqualified helper')
    body = reference(source); module = types.ModuleType('c15_bridge_' + name)
    module.__file__ = source['path']; exec(compile(body, source['path'], 'exec'), module.__dict__)
    return module


def publish(path, value):
    path = canonical(str(path), exists=False)
    body = (json.dumps(value, sort_keys=True, indent=2, allow_nan=False) + '\n').encode()
    require(len(body) <= MAX_INPUT, 'published metadata exceeds bound')
    fd = os.open(path, os.O_RDWR | os.O_CREAT | os.O_EXCL | os.O_NOFOLLOW, 0o600)
    with os.fdopen(fd, 'w+b') as out:
        out.write(body); out.flush(); os.fsync(out.fileno()); out.seek(0)
        require(out.read() == body, 'publication readback differs')
        held = os.fstat(out.fileno()); named = path.lstat()
        require((held.st_dev, held.st_ino) == (named.st_dev, named.st_ino), 'published identity replaced')
    return {'path': str(path), 'sha256': sha(body)}


def validate_config(request, qualification):
    require(sha(read(__file__)) == __compiled_sha256__, 'executing bridge source identity differs')
    config_ref = {'path': request['config_path'], 'sha256': request['config_sha256']}
    body = reference(config_ref); require(len(body) <= MAX_INPUT, 'config too large'); c = decode(body)
    require(type(c) is dict and set(c) == {'schema', 'helpers', 'executables', 'toolchain_root', 'cargo_capsule',
            'overlays', 'timeout_seconds', 'max_output_bytes', 'operation_timeout_seconds', 'qualification_implementation'}, 'config fields differ')
    require(c['schema'] == 'c15-runtime-bridge-config-v1' and type(c['qualification_implementation']) is bool
            and c['qualification_implementation'] is qualification, 'runtime/qualification mode mismatch')
    require(type(c['helpers']) is dict and set(c['helpers']) == set(HELPER_FILES), 'helper inventory differs')
    modules = {k: load_helper(k, v) for k, v in c['helpers'].items()}
    m, p, l = (modules[k] for k in ('materializer', 'project_inputs', 'live_selection'))
    require(str(m.EXECUTOR) == c['helpers']['executor']['path']
            and str(p.MATERIALIZER) == str(l.MATERIALIZER) == c['helpers']['materializer']['path'], 'helper dependency layout differs')
    require(type(c['executables']) is dict and set(c['executables']) == {'git', 'bwrap', 'patch_driver', 'toolchain_cargo', 'toolchain_rustc'}, 'executable inventory differs')
    for value in c['executables'].values():
        reference(value, independent=False); require(os.access(value['path'], os.X_OK), 'input executable unavailable')
    require(c['executables']['git'] == {'path': str(m.GIT), 'sha256': m.GIT_SHA}, 'qualified Git differs')
    sample = m.execution_spec({'owned': request['operation_root'], 'repo': request['project_root'], 'shared': {'root': request['project_root']}})
    require(c['executables']['bwrap'] == {'path': sample['bwrap'], 'sha256': sample['bwrap_sha256']}, 'qualified executor differs')
    toolchain = canonical(c['toolchain_root']); require(toolchain.is_dir(), 'toolchain directory required')
    require(all(Path(c['executables']['toolchain_' + name]['path']) == toolchain / 'bin' / name for name in ('cargo', 'rustc')), 'toolchain executable ownership differs')
    capsule = decode(reference(c['cargo_capsule'])); capsule_root = canonical(capsule['root'])
    require(c['cargo_capsule']['path'] == str(capsule_root / 'CAPSULE.json')
            and capsule['schema'] == 'c15-public-cargo-capsule-v1'
            and capsule['credentials_or_config_copied'] is False, 'capsule identity differs')
    require(type(capsule['files']) is dict and 0 < len(capsule['files']) <= 100000, 'capsule inventory empty/bound')
    for name, digest in capsule['files'].items():
        path = l.normalized(name); require(path.parts[0] in {'index', 'cache'}, 'unsupported capsule input')
        reference({'path': str(capsule_root / name), 'sha256': digest})
    actual = set()
    expected_dirs = {str(parent) for name in capsule['files'] for parent in Path(name).parents}
    for root, dirs, files in os.walk(capsule_root, followlinks=False):
        for name in dirs:
            require(not (Path(root) / name).is_symlink(), 'capsule directory alias')
            require(str((Path(root) / name).relative_to(capsule_root)) in expected_dirs, 'undeclared capsule directory')
        for name in files:
            actual.add(str((Path(root) / name).relative_to(capsule_root)))
            require(len(actual) <= len(capsule['files']) + 1, 'undeclared capsule file')
    require(actual == set(capsule['files']) | {'CAPSULE.json'}, 'capsule file population differs')
    l.declarations(c['overlays'])
    require(all(any(row['index_flags'].values()) for row in c['overlays']), 'overlay without an observed hidden flag unsupported')
    for key, maximum in (('timeout_seconds', 300), ('operation_timeout_seconds', 900)):
        require(type(c[key]) in (int, float) and math.isfinite(c[key]) and 0 < c[key] <= maximum, 'invalid time bound')
    require(type(c['max_output_bytes']) is int and 0 < c['max_output_bytes'] <= 16 * 1024 * 1024, 'invalid output bound')
    live, owned = canonical(request['project_root']), canonical(request['operation_root'])
    require(live.is_dir() and owned.is_dir() and not overlaps(live, owned), 'invalid/overlapping owned roots')
    protected = [toolchain, capsule_root, Path(c['executables']['patch_driver']['path']).parent,
                 Path(config_ref['path']), *[Path(x['path']) for x in c['helpers'].values()]]
    require(all(not overlaps(root, item) for root in (live, owned) for item in protected), 'source/view overlaps protected input')
    pins = {x['path']: x['sha256'] for x in [config_ref, *c['helpers'].values(), *c['executables'].values(), c['cargo_capsule']]}
    pins[str(Path(__file__))] = __compiled_sha256__
    pins.update({str(capsule_root / name): digest for name, digest in capsule['files'].items()})
    return c, modules, capsule, pins


def validate_request(request, qualification):
    common = {'schema', 'operation', 'config_path', 'config_sha256', 'project_root', 'operation_root'}
    role = {'run_id', 'agent_id', 'launch_generation', 'proposal_id', 'revision_of', 'proposal', 'cancel_path'}
    require(type(request) is dict and request.get('schema') == SCHEMA, 'invalid bridge schema')
    op = request.get('operation'); require(op in ({'validate', 'capture', 'test', 'implementation'} if qualification else {'validate', 'capture', 'test'}), 'unsupported runtime operation')
    extra = {'selection'} if op == 'test' else {'prior_test', 'implementation'} if op == 'implementation' else set()
    require(set(request) == common | (role if op != 'validate' else set()) | extra, 'request fields differ')
    if op == 'validate': return
    for key in ('run_id', 'proposal_id'):
        require(type(request[key]) is str and re.fullmatch('[A-Za-z0-9_.-]{1,80}', request[key]), 'invalid owner/proposal identity')
    agent = request['agent_id']
    require(type(agent) is str and agent and all(not x.isspace() and unicodedata.category(x) != 'Cc' for x in agent), 'invalid canonical agent identity')
    require(type(request['launch_generation']) is int and request['launch_generation'] > 0, 'invalid launch generation')
    require(request['revision_of'] is None or (type(request['revision_of']) is str and re.fullmatch('[A-Za-z0-9_.-]{1,80}', request['revision_of']) and request['revision_of'] != request['proposal_id']), 'invalid revision identity')
    p = request['proposal']
    require(type(p) is dict and set(p) == {'format', 'reason', 'body', 'test_purpose', 'test_paths', 'command', 'lock_paths'}, 'proposal fields differ')
    require(p['format'] in {'edit', 'patch'} and all(type(p[k]) is str and p[k] and '\0' not in p[k] for k in ('reason', 'body', 'test_purpose', 'command')), 'invalid proposal text')
    require(type(p['lock_paths']) is list and p['lock_paths'] and all(type(x) is str and x and not Path(x).is_absolute() and '..' not in Path(x).parts for x in p['lock_paths']), 'invalid declared locks')
    paths = p['test_paths']; require(type(paths) is list and 0 < len(paths) <= 10000, 'missing/bounded declared test paths')
    seen = set()
    for path in paths:
        require(type(path) is str and str(Path(path)) == path and path != '.' and not Path(path).is_absolute() and '..' not in Path(path).parts, 'invalid test path')
        require(path not in seen, 'duplicate test path'); seen.add(path)


class Budget:
    def __init__(self, seconds, cancel):
        self.deadline = time.monotonic() + seconds; self.cancel_path = cancel
        self.execution_pending = False
        self.event = threading.Event(); self.done = threading.Event(); self.reason = None
        self.thread = threading.Thread(target=self.watch, daemon=True); self.thread.start()

    def watch(self):
        while not self.done.wait(.02):
            if self.cancel_path is not None and self.cancel_path.exists(): self.reason = 'cancelled'
            elif time.monotonic() >= self.deadline: self.reason = 'operation_timeout'
            if self.reason: self.event.set(); return

    def check(self):
        if self.event.is_set() or time.monotonic() >= self.deadline:
            raise TimeoutError(self.reason or 'operation_timeout')

    def close(self): self.done.set(); self.thread.join()


def bind_modules(modules, budget, live):
    m, e, p, l = (modules[k] for k in ('materializer', 'executor', 'project_inputs', 'live_selection'))
    old_git, old_spec = m.git, m.execution_spec
    def git(*args, **kwargs):
        budget.check(); result = old_git(*args, **kwargs); budget.check(); return result
    def spec(snapshot, readonly=()):
        result = old_spec(snapshot, readonly)
        origin = snapshot.get('selected_origin')
        require(origin and origin['live_root'] == str(live) and origin['owned_accepted_root'] == snapshot['shared']['root'], 'original-live/accepted provenance differs')
        require(canonical(origin['live_root']) == live, 'original live path changed')
        result['view'] = str(live); return result
    def run(specification, argv, **kwargs):
        budget.check(); kwargs['cancel'] = budget.event
        kwargs['timeout'] = min(kwargs.get('timeout', 300), max(.001, budget.deadline - time.monotonic()))
        budget.execution_pending = True
        result = e.run_preview(specification, argv, **kwargs)
        require(result['closed'] is True, 'executor did not prove closure')
        budget.execution_pending = False
        return result
    m.git = git; m.execution_spec = spec; m.executor = lambda: types.SimpleNamespace(run_preview=run)
    l.M = m; l.git = git; p.materializer = lambda: m
    return m, p, l, run


def owned_identity(request):
    return {key: request[key] for key in ('run_id', 'agent_id', 'launch_generation', 'proposal_id', 'revision_of', 'project_root', 'operation_root', 'proposal', 'config_path', 'config_sha256')}


def validate_prior(request, name, operation):
    body = reference(request[name]); saved = decode(body)
    require(saved['request']['operation'] == operation and
            json.dumps(owned_identity(saved['request']), sort_keys=True) == json.dumps(owned_identity(request), sort_keys=True), 'prior operation owner/proposal differs')
    require(saved.get('status') == 'completed' and saved.get('closed') is True, 'prior operation incomplete')
    root = canonical(request['operation_root'])
    require(Path(request[name]['path']) == root / (operation.upper() + '-RESULT.json'), 'prior receipt outside exact owner')
    return saved


def declared_images(repo, paths):
    result = []; total = 0
    for name in paths:
        path = canonical(str(repo / name)); body = read(path, 256 * 1024 * 1024 - total); total += len(body)
        result.append({'path': name, 'text': body.decode(), 'sha256': sha(body), 'bytes': len(body),
                       'mode': '100755' if path.stat().st_mode & 0o111 else '100644'})
    return result


def execute(request, *, qualification=False):
    validate_request(request, qualification)
    c, modules, capsule, pins = validate_config(request, qualification)
    op = request['operation']; owned = canonical(request['operation_root']); live = canonical(request['project_root'])
    cancel = None
    if op != 'validate':
        cancel = canonical(request['cancel_path'], exists=False)
        require(cancel.parent == owned and not cancel.exists(), 'cancel file must be absent and owned')
    prior = validate_prior(request, 'selection', 'capture') if op == 'test' else validate_prior(request, 'prior_test', 'test') if op == 'implementation' else None
    attempt = owned / (op.upper() + '-ATTEMPT.json'); result_path = owned / (op.upper() + '-RESULT.json')
    reserved = [attempt, result_path]
    if op in {'test', 'implementation'}: reserved.append(owned / (op.upper() + '-EXECUTION.json'))
    require(all(not path.exists() and not path.is_symlink() for path in reserved), 'operation already attempted/published or unsafe publication path')
    publish(attempt, {'request_sha256': sha(json.dumps(request, sort_keys=True, separators=(',', ':')).encode()), 'retry_authorized': False})
    budget = Budget(c['operation_timeout_seconds'], cancel)
    record = {'schema': 'c15-runtime-bridge-result-v1', 'request': copy.deepcopy(request), 'qualification_only': qualification,
              'status': 'failed', 'closed': True, 'exit_code': None, 'stop_reason': None, 'stdout': '', 'stderr': '',
              'promoted': False, 'shared_accepted': False, 'source_sha256': pins, 'attempt': str(attempt)}
    started = time.monotonic()
    try:
        m, p, l, run = bind_modules(modules, budget, live)
        if op == 'validate': record['status'] = 'completed'
        elif op == 'capture':
            target = owned / 'capture'; target.mkdir()
            head = m.git(live, ['rev-parse', '--verify', 'HEAD^{commit}']).decode().strip()
            selected = l.capture_selection(live, target, head, c['overlays'])
            record['selected'] = selected; record['status'] = 'completed'
        else:
            if op == 'test':
                materialization = owned / 'test'; materialization.mkdir()
                selection = prior['selected']; snapshot = l.materialize_selected(selection, materialization)
                require(snapshot['selected_origin']['live_root'] == str(live)
                        and selection['live_census']['root'] == str(live), 'selected original live path differs')
                projected = []
                for row in selection['overlays']:
                    projected.append({key: value for key, value in row.items() if key not in {'base_blob', 'index_flags'}} |
                                     {'observed_index_flag': 'skip-worktree' if row['index_flags']['skip_worktree'] else 'assume-unchanged'})
                overlay = p.install_private_overlays(snapshot, projected) if projected else None
                record['overlay_adapter'] = {'original_declarations': selection['overlays'], 'single_flag_projection': projected,
                    'projection_not_exhaustive_index_census': True, 'installation': overlay}
                require(m.source_identity(snapshot['repo'])['files'] == {name: value['live'] for name, value in selection['live_census']['files'].items()}, 'private effective image differs from selected live census')
            else:
                snapshot = prior['snapshot']; require(prior['declared_test_afterimages'] == declared_images(Path(snapshot['repo']), request['proposal']['test_paths']), 'prior declared-path image changed')
                require(m.source_identity(snapshot['repo']) == prior['post_command_source'], 'qualification private image changed')
            record['snapshot'] = snapshot
            proposal = request['proposal'] if op == 'test' else request['implementation']
            if op == 'implementation':
                require(type(proposal) is dict and set(proposal) == {'format', 'reason', 'body'}
                        and proposal['format'] in {'edit', 'patch'} and all(type(proposal[k]) is str and proposal[k] for k in ('reason', 'body')), 'invalid qualification implementation')
            adapter_proposal = {'id': 'held-test' if op == 'test' else 'qualification-implementation',
                'format': proposal['format'], 'agent_id': request['agent_id'], 'feature': 'Private test preview',
                'reason': proposal['reason'], 'body': proposal['body'],
                'declared_test_units': [request['proposal']['test_purpose']]}
            driver = {**c['executables']['patch_driver'], 'bwrap_sha256': c['executables']['bwrap']['sha256']}
            record['patch'] = m.apply_proposal(snapshot, adapter_proposal, driver)
            require(record['patch']['applied_privately'], 'held patch was not applied privately')
            record['declared_test_afterimages'] = declared_images(Path(snapshot['repo']), request['proposal']['test_paths'])
            record['test_identity_scope'] = 'Whole declared-path images only; no semantic test boundary, test-only purpose, or assertion equivalence inferred.'
            if op == 'implementation': require(record['declared_test_afterimages'] == prior['declared_test_afterimages'], 'qualification changes declared test file image')
            scratch = Path(snapshot['owned']) / 'scratch'
            if op == 'test': p.prepare_cargo_home(scratch, capsule)
            spec = m.execution_spec(snapshot, [{'source': c['toolchain_root'], 'target': '/toolchain'},
                                               {'source': capsule['root'], 'target': '/cargo-public'}])
            argv = ['/usr/bin/env', 'RUSTC=/toolchain/bin/rustc', 'PATH=/toolchain/bin:/usr/bin:/bin',
                    '/bin/sh', '-c', request['proposal']['command']]
            command = run(spec, argv, timeout=c['timeout_seconds'], max_output_bytes=c['max_output_bytes'])
            record['command_execution'] = command
            publish(owned / (op.upper() + '-EXECUTION.json'), command)
            record.update({key: command[key] for key in ('closed', 'exit_code', 'stop_reason', 'stdout', 'stderr')})
            record['post_command_source'] = m.source_identity(snapshot['repo'])
            require(record['declared_test_afterimages'] == declared_images(Path(snapshot['repo']), request['proposal']['test_paths']), 'command changed declared-path bytes')
            record['status'] = 'completed'
    except (ValueError, OSError, RuntimeError, subprocess.SubprocessError) as error:
        record['error'] = {'type': type(error).__name__, 'message': str(error)}
        record['stop_reason'] = budget.reason or record['stop_reason'] or 'preparation_failed'
        if budget.execution_pending: record['closed'] = False
        record['stderr'] = record['stderr'] or str(error)
    finally: budget.close()
    endpoint_errors = []
    executable_paths = {x['path'] for x in c['executables'].values()}
    for path, expected in pins.items():
        try: require(sha(read(path, independent=path not in executable_paths)) == expected, 'source changed during bridge operation')
        except (ValueError, OSError) as error: endpoint_errors.append({'path': path, 'error': str(error)})
    record['source_endpoint_errors'] = endpoint_errors
    record['source_endpoints_match'] = not endpoint_errors
    if endpoint_errors:
        record['status'] = 'failed'; record['stop_reason'] = record['stop_reason'] or 'source_integrity_failure'
    record['elapsed_seconds'] = time.monotonic() - started
    result = publish(result_path, record)
    reply = {key: record[key] for key in ('status', 'closed', 'exit_code', 'stop_reason', 'stdout', 'stderr')}
    reply.update({key: request[key] for key in ('run_id', 'agent_id', 'launch_generation', 'proposal_id') if key in request})
    reply.update({'result_path': result['path'], 'result_sha256': result['sha256']})
    if op == 'capture' and record['status'] == 'completed': reply['selection'] = result
    return reply


def main(argv=None):
    argv = sys.argv[1:] if argv is None else argv
    try:
        require(len(argv) == 2, 'usage: bridge INPUT_JSON OUTPUT_JSON')
        inp = canonical(argv[0]); out = canonical(argv[1], exists=False)
        require(not out.exists(), 'output exists; retry forbidden')
        body = read(inp, MAX_INPUT); request = decode(body); owned = canonical(request['operation_root'])
        require(inp.parent == out.parent == owned and inp != out, 'input/output must be direct owned children')
        result = execute(request)
        publish(out, result)
        return 0 if result['status'] == 'completed' else 1
    except (ValueError, OSError, RuntimeError) as error:
        print(f'private bridge failed: {error}', file=sys.stderr); return 2


if __name__ == '__main__': raise SystemExit(main())
