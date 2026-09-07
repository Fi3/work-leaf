"""Provider-free accepted-tree materialization and held patch qualification.

The caller owns source serialization and precreated private roots. This standalone
helper cannot acquire a live WL daemon's in-memory FileLockTable. It never promotes
private Git commits or emits a shared PatchApplied/ACK. Unsupported inputs fail closed.
"""
import hashlib
import json
import os
from pathlib import Path
import re
import stat
import subprocess
import types


HERE = Path(__file__).resolve().parent
EXECUTOR = HERE.parent / 'c15-private-executor/executor.py'
EXECUTOR_SHA = '16e3238e1bd647c9d4780db97c554ddf9579226c913995e2e18873bf7fcb24ae'
GIT = Path('/usr/bin/git')
GIT_SHA = '292115a21c70326a0fa239e6d9bdee32f750cbd23d12fdef2e518ad4b451a8b3'
MAX_FILES = 100000
MAX_BYTES = 256 * 1024 * 1024


def require(value, message):
    if not value: raise ValueError(message)


def sha(data): return hashlib.sha256(data).hexdigest()


def canonical(path):
    path = Path(path)
    require(path.is_absolute() and path.resolve(strict=True) == path and '..' not in path.parts,
            'canonical nonsymlink path required')
    return path


def git(root, args, *, data=None):
    require(sha(GIT.read_bytes()) == GIT_SHA, 'Git executable changed')
    env = {'PATH': '/usr/bin:/bin', 'LC_ALL': 'C.UTF-8', 'GIT_OPTIONAL_LOCKS': '0',
           'GIT_CONFIG_NOSYSTEM': '1', 'GIT_CONFIG_GLOBAL': '/dev/null', 'HOME': '/nonexistent'}
    result = subprocess.run([str(GIT), '-c', 'core.hooksPath=/dev/null', '-c', 'core.fsmonitor=false',
                             '-c', 'core.attributesFile=/dev/null', '-c', 'core.autocrlf=false',
                             '-c', 'protocol.allow=never', '-c', 'protocol.file.allow=always',
                             '-C', str(root), *args], input=data, stdout=subprocess.PIPE,
                            stderr=subprocess.PIPE, env=env, timeout=30)
    require(result.returncode == 0, f'Git operation failed: {args[0]} (status {result.returncode})')
    return result.stdout


def publish(path, value):
    content = (json.dumps(value, sort_keys=True, indent=2, allow_nan=False) + '\n').encode()
    with path.open('xb') as out:
        out.write(content); out.flush(); os.fsync(out.fileno())
    require(path.read_bytes() == content, 'evidence readback differs')
    return sha(content)


def file_identity(path, remaining):
    meta = path.lstat()
    require(stat.S_ISREG(meta.st_mode), 'unsupported symlink or special source file')
    require(meta.st_nlink == 1, 'unsupported source hardlink')
    require(meta.st_size <= remaining, 'source file exceeds byte bound')
    with path.open('rb') as source:
        data = source.read(remaining + 1)
        end = os.fstat(source.fileno())
    require(len(data) <= remaining, 'source file exceeds byte bound')
    require((meta.st_dev, meta.st_ino, meta.st_size, meta.st_mtime_ns, meta.st_mode, meta.st_nlink)
            == (end.st_dev, end.st_ino, end.st_size, end.st_mtime_ns, end.st_mode, end.st_nlink),
            'source changed during read')
    mode = '100755' if meta.st_mode & 0o111 else '100644'
    return {'sha256': sha(data), 'bytes': len(data), 'mode': mode}, data


def source_identity(root):
    root = canonical(root)
    require((root / '.git').is_dir() and not (root / '.git').is_symlink(), 'linked Git directory unsupported')
    require(git(root, ['rev-parse', '--show-toplevel']).decode().strip() == str(root), 'not actual Git root')
    require(Path(git(root, ['rev-parse', '--absolute-git-dir']).decode().strip()) == root / '.git', 'external Git directory unsupported')
    require(not (root / '.git/commondir').exists(), 'linked Git common directory unsupported')
    require(not (root / '.git/objects/info/alternates').exists(), 'Git alternates unsupported')
    # No effective include/filter/worktree configuration is silently reconstructed.
    keys = git(root, ['config', '--local', '--name-only', '--list']).decode().splitlines()
    require(not any(k.lower().startswith(('include.', 'includeif.', 'filter.', 'extensions.', 'submodule.'))
                    or k.lower() in {'core.worktree', 'core.sparsecheckout'} for k in keys),
            'unsupported Git configuration/include/filter/worktree')
    head = git(root, ['rev-parse', '--verify', 'HEAD^{commit}']).decode().strip()
    tree = git(root, ['rev-parse', '--verify', 'HEAD^{tree}']).decode().strip()
    algorithm = git(root, ['rev-parse', '--show-object-format']).decode().strip()
    require(algorithm in {'sha1', 'sha256'}, 'unsupported Git object format')
    tree_rows = git(root, ['ls-tree', '-rz', '--full-tree', head]).split(b'\0')
    entries = {}
    for row in filter(None, tree_rows):
        info, name = row.split(b'\t', 1)
        mode, kind, oid = info.decode().split()
        require(mode in {'100644', '100755'} and kind == 'blob', 'unsupported submodule/symlink/tree entry')
        path = name.decode('utf-8')
        require(path not in entries and path and not Path(path).is_absolute() and '..' not in Path(path).parts,
                'unsupported or duplicate tree path')
        entries[path] = (mode, oid)
    require(len(entries) <= MAX_FILES, 'source file bound exceeded')
    staged = {}
    for row in filter(None, git(root, ['ls-files', '--stage', '-z']).split(b'\0')):
        info, name = row.split(b'\t', 1)
        mode, oid, stage = info.decode().split()
        require(stage == '0', 'unmerged index unsupported')
        staged[name.decode('utf-8')] = (mode, oid)
    require(staged == entries, 'index differs from accepted tree')
    flags = git(root, ['ls-files', '-v', '-z']).split(b'\0')
    require(all(row.startswith(b'H ') for row in flags if row),
            'unsupported skip-worktree/assume-unchanged index flags')
    require(not git(root, ['ls-files', '--others', '-z']), 'untracked or ignored source unsupported')
    names = b''.join(name.encode() + b'\0' for name in entries)
    attrs = git(root, ['check-attr', '-z', '--all', '--stdin'], data=names).split(b'\0')
    require(not any(attrs[i] in {b'filter', b'working-tree-encoding', b'text', b'eol', b'ident'}
                    for i in range(1, len(attrs)-1, 3)), 'checkout conversion/filter attributes unsupported')
    files = {}; total = 0
    for name, (mode, oid) in entries.items():
        path = root / name
        require(path.resolve(strict=True) == path, 'source path traverses symlink')
        identity, data = file_identity(path, MAX_BYTES - total)
        total += len(data)
        require(total <= MAX_BYTES, 'source byte bound exceeded')
        object_bytes = b'blob ' + str(len(data)).encode() + b'\0' + data
        require(hashlib.new(algorithm, object_bytes).hexdigest() == oid and identity['mode'] == mode,
                'worktree differs from accepted blob or mode')
        files[name] = identity
    administrative = {}
    for name in ('HEAD', 'index', 'config', 'info/exclude'):
        path = root / '.git' / name
        if path.exists():
            require(path.resolve() == path and path.is_file(), 'Git administration alias unsupported')
            administrative[name] = sha(path.read_bytes())
        else: administrative[name] = None
    return {'root': str(root), 'head': head, 'tree': tree, 'object_format': algorithm,
            'files': files, 'administrative_sha256': administrative}


def materialize(shared, owned, expected_commit, expected_tree):
    shared, owned = canonical(shared), canonical(owned)
    require(shared not in owned.parents and owned not in shared.parents and shared != owned,
            'private/shared roots overlap')
    require(owned.is_dir() and not any(owned.iterdir()), 'private root must be empty and exclusively owned')
    before = source_identity(shared)
    require(before['head'] == expected_commit and before['tree'] == expected_tree, 'accepted source identity differs')
    # Caller-held FileLockTable `.` selection is an integration prerequisite, not
    # a property manufactured by this out-of-process qualification.
    publish(owned / 'CLAIM.json', {'source': before, 'promotion': False,
                                  'in_process_wl_lock_proved': False})
    try:
        git(owned, ['clone', '--no-checkout', '--no-hardlinks', '--local', str(shared), str(owned / 'repo')])
        repo = owned / 'repo'
        git(repo, ['checkout', '--detach', expected_commit])
        copied = source_identity(repo)
        require(copied['tree'] == before['tree'] and copied['files'] == before['files'], 'private materialization differs')
        require((repo / '.git/index').stat().st_ino != (shared / '.git/index').stat().st_ino
                or (repo / '.git/index').stat().st_dev != (shared / '.git/index').stat().st_dev,
                'private index aliases shared index')
        for path in (repo / '.git').rglob('*'):
            require(not path.is_symlink(), 'private Git metadata symlink unsupported')
            if path.is_file(): require(path.stat().st_nlink == 1, 'private Git metadata hardlink unsupported')
        for name in ('evidence', 'build', 'scratch'): (owned / name).mkdir()
        require(source_identity(shared) == before, 'shared source drift during materialization')
        snapshot = {'schema': 'work-leaf-private-snapshot-qualification-v1', 'repo': str(repo),
                    'owned': str(owned), 'shared': before, 'initial_private': copied,
                    'exact_accepted_tree': True, 'in_process_wl_lock_proved': False,
                    'unsupported_external_inputs': 'not materialized; no dependency completeness claim'}
        publish(owned / 'SNAPSHOT.json', snapshot)
        return snapshot
    except (ValueError, OSError, subprocess.SubprocessError) as error:
        publish(owned / 'failure.json', {'error': str(error), 'shared_source_before': before,
                                       'retained_private_root': str(owned), 'promoted': False})
        raise


def executor():
    data = EXECUTOR.read_bytes()
    require(EXECUTOR.resolve() == EXECUTOR and sha(data) == EXECUTOR_SHA, 'qualified executor source differs')
    module = types.ModuleType('pinned_preview_executor'); module.__file__ = str(EXECUTOR)
    exec(compile(data, str(EXECUTOR), 'exec'), module.__dict__)
    return module


def execution_spec(snapshot, readonly=()):
    owned = canonical(snapshot['owned'])
    return {'bwrap': '/usr/bin/bwrap', 'bwrap_sha256': 'eabbccb0f7f755b96d30834026a9b5d941c606400d097d87c1ff16622edaf68c',
            'source': snapshot['repo'], 'build': str(owned / 'build'), 'scratch': str(owned / 'scratch'),
            'view': snapshot['shared']['root'], 'readonly': list(readonly)}


def apply_proposal(snapshot, proposal, driver):
    require(isinstance(proposal, dict) and set(proposal) == {'id', 'format', 'agent_id', 'feature', 'reason', 'body', 'declared_test_units'},
            'invalid proposal schema')
    require(all(isinstance(proposal[k], str) and proposal[k] for k in ('id', 'format', 'agent_id', 'feature', 'reason', 'body')), 'missing proposal text')
    require(re.fullmatch('[A-Za-z0-9_-]{1,80}', proposal['id']) and proposal['format'] in {'edit', 'patch'}, 'invalid proposal identity/format')
    require(isinstance(proposal['declared_test_units'], list) and proposal['declared_test_units']
            and all(isinstance(x, str) and x for x in proposal['declared_test_units']), 'missing declared test identity')
    binary = canonical(driver['path'])
    require(sha(binary.read_bytes()) == driver['sha256'], 'patch driver identity differs')
    repo, owned = canonical(snapshot['repo']), canonical(snapshot['owned'])
    require(repo == owned / 'repo', 'snapshot owner identity differs')
    shared_before = source_identity(snapshot['shared']['root'])
    before = source_identity(repo)
    destination = owned / 'evidence' / proposal['id']
    destination.mkdir()
    proposal_sha = publish(destination / 'proposal.json', proposal)
    spec = execution_spec(snapshot, [
        {'source': str(binary.parent), 'target': '/preview-tools'},
        {'source': str(destination), 'target': '/proposal'}])
    require(driver['bwrap_sha256'] == spec['bwrap_sha256'], 'executor admission digest differs')
    args = ['/usr/bin/env', 'GIT_CONFIG_NOSYSTEM=1', 'GIT_CONFIG_GLOBAL=/dev/null',
            'GIT_CONFIG_COUNT=2', 'GIT_CONFIG_KEY_0=core.hooksPath', 'GIT_CONFIG_VALUE_0=/dev/null',
            'GIT_CONFIG_KEY_1=core.fsmonitor', 'GIT_CONFIG_VALUE_1=false',
            'GIT_AUTHOR_NAME=Private Preview', 'GIT_AUTHOR_EMAIL=preview@example.invalid',
            'GIT_COMMITTER_NAME=Private Preview', 'GIT_COMMITTER_EMAIL=preview@example.invalid',
            '/preview-tools/' + binary.name, spec['view'], '/proposal/proposal.json']
    try:
        run = executor().run_preview(spec, args, timeout=30)
        # Persist the closed execution before inspecting an unsupported/partial
        # after-image; a failed qualification must not hide private side effects.
        publish(destination / 'EXECUTION.json', run)
        parsed = json.loads(run['stdout']) if run['exit_code'] == 0 and run['stop_reason'] is None else {}
        after = source_identity(repo)
        shared_after = source_identity(snapshot['shared']['root'])
        require(shared_after == shared_before, 'shared source drift during proposal qualification')
        files = {name: {'before': before['files'].get(name), 'after': after['files'].get(name)}
                 for name in sorted(before['files'].keys() | after['files'].keys())
                 if before['files'].get(name) != after['files'].get(name)}
        result = {'proposal': proposal, 'proposal_sha256': proposal_sha, 'execution': run,
                  'applied_privately': parsed.get('applied_privately') is True, 'shared_accepted': False,
                  'private_before': before, 'private_after': after, 'files': files,
                  'shared_at_proposal': shared_before,
                  'shared_advanced_since_snapshot': shared_before != snapshot['shared'],
                  'driver_sha256': driver['sha256'], 'executor_source_sha256': EXECUTOR_SHA,
                  'test_purpose': 'declared, not semantically proven', 'promotion': False}
        require(sha(binary.read_bytes()) == driver['sha256'], 'patch driver changed during proposal')
        publish(destination / 'RESULT.json', result)
        return result
    except (ValueError, OSError, subprocess.SubprocessError) as error:
        publish(destination / 'failure.json', {'error': str(error), 'proposal_sha256': proposal_sha,
                                               'retained_private_root': str(owned), 'promoted': False})
        raise
