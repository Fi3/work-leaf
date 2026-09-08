"""Caller-owned selection with explicit ignored derived-output exclusions.

The caller must hold its actual shared FileLockTable root read lock. This process
cannot prove or manufacture that lock; a standalone qualification caller is separate.
"""
import hashlib
import json
from pathlib import Path
import re
import stat
import subprocess
import sys
import time
import types


HERE = Path(__file__).resolve().parent
MATERIALIZER = HERE.parent / 'c15-private-materialization/materialize.py'
MATERIALIZER_SHA = '87546793b76bc7324af55479be47a1e34abbeffb8f2787547d021575a51efab2'
MAX_BUNDLE_BYTES = 512 * 1024 * 1024  # Postcreation admission, not an in-flight storage limit.
SCHEMA = 'c15-caller-owned-selected-source-v2'
MAX_DERIVED_ROOTS = 64
MAX_PATH_DEPTH = 64
MAX_PATH_BYTES = 4096
MAX_DERIVED_PATHS = 100000


def require(value, message):
    if not value: raise ValueError(message)


def sha(body): return hashlib.sha256(body).hexdigest()


def materializer():
    require(MATERIALIZER.resolve() == MATERIALIZER, 'materializer source alias')
    body = MATERIALIZER.read_bytes()
    require(sha(body) == MATERIALIZER_SHA, 'qualified materializer source changed')
    module = types.ModuleType('qualified_materializer'); module.__file__ = str(MATERIALIZER)
    exec(compile(body, str(MATERIALIZER), 'exec'), module.__dict__)
    return module


M = materializer()
git = M.git


def normalized(name):
    require(isinstance(name, str) and bool(name), 'missing relative path')
    path = Path(name)
    require(str(path) == name and not path.is_absolute() and name != '.' and '..' not in path.parts,
            'unsupported source path')
    return path


def declarations(overlays):
    fields = {'path', 'base_blob', 'base_sha256', 'base_mode', 'effective_text',
              'effective_sha256', 'effective_mode', 'index_flags'}
    require(isinstance(overlays, list) and len(overlays) <= M.MAX_FILES, 'invalid overlay inventory')
    result = {}; total = 0
    for row in overlays:
        require(isinstance(row, dict) and set(row) == fields, 'unsupported overlay schema')
        name = row['path']; normalized(name)
        require(name not in result, 'duplicate overlay declaration')
        for key, pattern in (('base_blob', '[0-9a-f]{40}|[0-9a-f]{64}'),
                             ('base_sha256', '[0-9a-f]{64}'), ('effective_sha256', '[0-9a-f]{64}')):
            require(isinstance(row[key], str) and re.fullmatch(pattern, row[key]), 'invalid overlay digest')
        require(row['base_mode'] in {'100644', '100755'} and row['effective_mode'] in {'100644', '100755'}, 'unsupported overlay mode')
        flags = row['index_flags']
        require(isinstance(flags, dict) and set(flags) == {'skip_worktree', 'assume_unchanged'}
                and all(type(value) is bool for value in flags.values()), 'invalid overlay index flags')
        require(isinstance(row['effective_text'], str), 'missing exact overlay text')
        body = row['effective_text'].encode(); total += len(body)
        require(total <= M.MAX_BYTES and sha(body) == row['effective_sha256'], 'overlay bytes differ or exceed bound')
        result[name] = row
    return result


def derived_path(name):
    path = normalized(name)
    require(len(path.parts) <= MAX_PATH_DEPTH and len(name.encode()) <= MAX_PATH_BYTES,
            'derived path depth/byte bound exceeded')
    require('.git' not in path.parts and '\0' not in name, 'administrative derived path unsupported')
    return path


def derived_declarations(root, roots, protected):
    require(type(roots) is list and len(roots) <= MAX_DERIVED_ROOTS, 'invalid derived root inventory')
    names = set(); parsed = []
    for name in roots:
        path = derived_path(name)
        require(name not in names, 'duplicate derived root'); names.add(name); parsed.append(path)
    for path in parsed:
        require(not any(str(parent) in names for parent in path.parents), 'overlapping derived roots')
    for name in protected:
        path = derived_path(name)
        require(name not in names and not any(str(parent) in names for parent in path.parents),
                'derived root covers accepted/staged/overlay source')
    # Metadata only, one bounded ancestor walk per declaration. Missing roots
    # are permitted; neither creation nor cleanup belongs to this selector.
    for path in parsed:
        current = root
        for part in path.parts:
            current = current / part
            try: meta = current.lstat()
            except FileNotFoundError: continue
            require(stat.S_ISDIR(meta.st_mode), 'derived root/ancestor is not a real directory')
    return sorted(names)


def ignore_authority(root, entries):
    names = ['.git/config', '.git/info/exclude']
    names.extend(name for name in entries if Path(name).name == '.gitignore')
    authority = {}
    for name in names:
        path = root / name
        try: path.lstat()
        except FileNotFoundError:
            authority[name] = None
            continue
        require(path.resolve(strict=True) == path, 'ignore authority alias')
        authority[name] = M.file_identity(path, M.MAX_BYTES)[0]
    return authority


def derived_inventory(root, roots, entries, staged, declared):
    names = derived_declarations(root, roots, set(entries) | set(staged) | set(declared))
    root_set = set(names)
    authority = ignore_authority(root, entries)
    keys = git(root, ['config', '--local', '--name-only', '--list']).decode().splitlines()
    require('core.excludesfile' not in {key.lower() for key in keys}, 'external ignore authority unsupported')
    others = git(root, ['ls-files', '--others', '-z']).split(b'\0')
    others = [value.decode() for value in others if value]
    require(len(others) <= MAX_DERIVED_PATHS and len(set(others)) == len(others), 'derived path inventory bound/duplicates')
    require(not any(Path(name).name == '.gitignore' for name in others), 'untracked ignore administration unsupported')
    ignored = {value.decode() for value in git(root, ['ls-files', '--others', '--ignored', '--exclude-standard', '-z']).split(b'\0') if value}
    require(set(others) == ignored, 'nonignored untracked input unsupported')
    metadata = {}; checked_dirs = set()
    for name in others:
        path = derived_path(name)
        require(any(str(parent) in root_set for parent in path.parents), 'undeclared ignored input unsupported')
        current = root
        for part in path.parts[:-1]:
            current = current / part
            if current not in checked_dirs:
                require(stat.S_ISDIR(current.lstat().st_mode), 'derived entry ancestor alias')
                checked_dirs.add(current)
        meta = (root/path).lstat()
        require(stat.S_ISREG(meta.st_mode) and meta.st_nlink == 1, 'derived entry alias/special file unsupported')
        metadata[name] = {'bytes': meta.st_size, 'mtime_ns': meta.st_mtime_ns}
    rules = {}
    if others:
        records = git(root, ['check-ignore', '-v', '-z', '--stdin'],
                      data=b''.join(name.encode()+b'\0' for name in sorted(others))).split(b'\0')
        require(records[-1] == b'' and (len(records)-1) % 4 == 0, 'malformed Git ignore proof')
        for i in range(0, len(records)-1, 4):
            source, line, pattern, name = (value.decode() for value in records[i:i+4])
            require(name in ignored and name not in rules and line.isdecimal() and int(line) > 0
                    and pattern and not pattern.startswith('!'), 'nonignored/ambiguous Git rule proof')
            # Ignored output bodies are never read. Rule authority must already
            # belong to the accepted/effective census or pinned Git administration.
            allowed = source == '.git/info/exclude' or (source in entries and Path(source).name == '.gitignore')
            require(allowed, 'untracked/external ignore rule source unsupported')
            rules[name] = {'source': source, 'line': int(line), 'pattern': pattern}
        require(rules.keys() == ignored, 'incomplete Git ignore provenance')
    require(ignore_authority(root, entries) == authority, 'ignore authority drift during classification')
    return {'roots': names, 'ignored_paths': sorted(others), 'metadata': metadata, 'rules': rules,
            'rule_authority': authority,
            'bodies_read_or_copied': False}


def census(root, expected_commit, overlays, *, derived_output_roots=None):
    root = M.canonical(root); declared = declarations(overlays)
    require((root/'.git').is_dir() and not (root/'.git').is_symlink(), 'linked Git directory unsupported')
    require(git(root, ['rev-parse', '--show-toplevel']).decode().strip() == str(root), 'not actual Git root')
    require(git(root, ['rev-parse', '--absolute-git-dir']).decode().strip() == str(root/'.git'), 'external Git directory unsupported')
    require(not (root/'.git/commondir').exists() and not (root/'.git/objects/info/alternates').exists(), 'linked or alternate Git objects unsupported')
    keys = git(root, ['config', '--local', '--name-only', '--list']).decode().splitlines()
    require(not any(key.lower().startswith(('include.', 'includeif.', 'filter.', 'extensions.', 'submodule.'))
                    or key.lower() in {'core.worktree', 'core.sparsecheckout'} for key in keys), 'unsupported Git configuration')
    commit = git(root, ['rev-parse', '--verify', 'HEAD^{commit}']).decode().strip()
    require(commit == expected_commit, 'accepted HEAD differs')
    tree = git(root, ['rev-parse', '--verify', 'HEAD^{tree}']).decode().strip()
    algorithm = git(root, ['rev-parse', '--show-object-format']).decode().strip()
    require(algorithm in {'sha1', 'sha256'}, 'unsupported Git object format')
    entries = {}; objects = {}; total = 0
    for record in filter(None, git(root, ['ls-tree', '-rlz', '--full-tree', commit]).split(b'\0')):
        raw, name_bytes = record.split(b'\t', 1); mode, kind, oid, size = raw.decode().split()
        require(mode in {'100644', '100755'} and kind == 'blob', 'unsupported submodule/symlink entry')
        name = name_bytes.decode(); normalized(name); size = int(size)
        require(name not in entries and size >= 0, 'duplicate or invalid tree entry')
        total += size
        require(len(entries) < M.MAX_FILES and total <= M.MAX_BYTES, 'accepted source bound exceeded')
        entries[name] = (mode, oid); objects[oid] = size
    require(set(declared) <= set(entries), 'unused/untracked overlay declaration')
    raw = git(root, ['cat-file', '--batch'], data=b''.join(oid.encode()+b'\n' for oid in objects))
    accepted = {}; cursor = 0
    for oid, size in objects.items():
        end = raw.index(b'\n', cursor); header = raw[cursor:end].decode().split()
        require(header == [oid, 'blob', str(size)], 'accepted object response differs')
        body = raw[end+1:end+1+size]; cursor = end+size+2
        require(len(body) == size and raw[cursor-1:cursor] == b'\n', 'incomplete accepted blob')
        require(hashlib.new(algorithm, b'blob '+str(size).encode()+b'\0'+body).hexdigest() == oid, 'accepted object hash differs')
        accepted[oid] = {'sha256': sha(body), 'bytes': size}
    require(cursor == len(raw), 'unexpected accepted object tail')
    staged = {}
    for record in filter(None, git(root, ['ls-files', '--stage', '-z']).split(b'\0')):
        info, name = record.split(b'\t', 1); mode, oid, stage = info.decode().split()
        name = name.decode(); require(stage == '0' and name not in staged, 'unmerged/duplicate index entry')
        staged[name] = (mode, oid)
    require(staged == entries, 'index differs from accepted tree')
    flags = {}
    for record in filter(None, git(root, ['ls-files', '-v', '-z']).split(b'\0')):
        tag, name = record[:1], record[2:].decode()
        require(record[1:2] == b' ' and tag in {b'H', b'S', b'h', b's'} and name not in flags, 'unsupported index flag record')
        flags[name] = {'skip_worktree': tag.upper() == b'S', 'assume_unchanged': tag.islower()}
    require(flags.keys() == entries.keys(), 'index flag inventory differs')
    if derived_output_roots is None:
        require(not git(root, ['ls-files', '--others', '-z']), 'untracked or ignored input unsupported')
        excluded = None
    else:
        excluded = derived_inventory(root, derived_output_roots, entries, staged, declared)
    attrs = git(root, ['check-attr', '-z', '--all', '--stdin'], data=b''.join(n.encode()+b'\0' for n in entries)).split(b'\0')
    require(not any(attrs[i] in {b'filter', b'working-tree-encoding', b'text', b'eol', b'ident'}
                    for i in range(1, len(attrs)-1, 3)), 'checkout conversion/filter attributes unsupported')
    files = {}; total = 0
    for name, (mode, oid) in entries.items():
        path = root / name; require(path.resolve(strict=True) == path, 'live file alias')
        live, body = M.file_identity(path, M.MAX_BYTES-total); total += len(body)
        baseline = {**accepted[oid], 'mode': mode}
        row = declared.get(name)
        if row is None:
            require(flags[name] == {'skip_worktree': False, 'assume_unchanged': False}, 'undeclared hidden index flag')
            require(live == baseline, 'undeclared live source difference')
        else:
            require(row['base_blob'] == oid and row['base_sha256'] == baseline['sha256']
                    and row['base_mode'] == mode, 'declared accepted overlay base differs')
            require(flags[name] == row['index_flags'], 'declared index flags differ')
            require(body == row['effective_text'].encode() and live['sha256'] == row['effective_sha256']
                    and live['mode'] == row['effective_mode'], 'declared effective overlay differs')
        files[name] = {'blob': oid, 'accepted': baseline, 'live': live, 'index_flags': flags[name]}
    administrative = {}
    for name in ('HEAD', 'index', 'config', 'info/exclude'):
        path = root / '.git' / name
        if path.exists():
            require(path.resolve() == path and path.is_file(), 'Git administrative alias')
            administrative[name] = sha(path.read_bytes())
        else: administrative[name] = None
    require(git(root, ['rev-parse', '--verify', 'HEAD^{commit}']).decode().strip() == commit, 'HEAD drift during census')
    if excluded is not None:
        require(ignore_authority(root, entries) == excluded['rule_authority'], 'ignore authority drift during census')
    result = {'root': str(root), 'commit': commit, 'tree': tree, 'files': files,
              'administrative_sha256': administrative, 'object_format': algorithm}
    if excluded is not None: result['derived_outputs'] = excluded
    return result


def capture_selection(root, owned, expected_commit, overlays, *, derived_output_roots=None):
    root, owned = M.canonical(root), M.canonical(owned)
    require(root != owned and root not in owned.parents and owned not in root.parents, 'selection overlaps shared source')
    require(owned.is_dir() and not any(owned.iterdir()), 'selection root must be empty and exclusively owned')
    M.publish(owned/'CLAIM.json', {'expected_commit': expected_commit, 'overlays': overlays,
                                 'derived_output_roots': derived_output_roots, 'schema': SCHEMA,
                                 'caller_lock_proved_by_python': False})
    started = time.monotonic()
    bundle = owned / 'accepted.bundle'
    def endpoint():
        try:
            value = bundle.lstat()
            size = value.st_size if stat.S_ISREG(value.st_mode) else None
        except FileNotFoundError: size = None
        return {'selection_operation_seconds': time.monotonic()-started,
                'bundle_bytes_at_endpoint': size, 'in_flight_bundle_storage_bound': False,
                'postcreation_bundle_admission_bytes': MAX_BUNDLE_BYTES}
    try:
        before = census(root, expected_commit, overlays, derived_output_roots=derived_output_roots)
        git(root, ['bundle', 'create', str(bundle), 'HEAD'])
        identity, _ = M.file_identity(bundle, MAX_BUNDLE_BYTES)
        require(git(root, ['bundle', 'list-heads', str(bundle)]) == (expected_commit+' HEAD\n').encode(), 'bundle HEAD identity differs')
        after = census(root, expected_commit, overlays, derived_output_roots=derived_output_roots)
        # Output creation/growth/removal is not accepted-source drift. Both full
        # metadata inventories remain separate, including their actual rule proof.
        source_before = {k: v for k, v in before.items() if k != 'derived_outputs'}
        source_after = {k: v for k, v in after.items() if k != 'derived_outputs'}
        require(source_before == source_after, 'live census drift during selection')
        bundle.chmod(0o400)
        record = {'schema': SCHEMA, 'commit': before['commit'],
                  'tree': before['tree'], 'live_census': before, 'overlays': overlays,
                  'bundle': {'path': str(bundle), 'sha256': identity['sha256'], 'bytes': identity['bytes']},
                  'receipt_path': str(owned/'SELECTED.json'), 'caller_lock_proved_by_python': False,
                  'live_atomicity_claimed': False, 'materializer_source_sha256': MATERIALIZER_SHA,
                  'selector_source_sha256': sha(Path(__file__).read_bytes()),
                  'derived_output_roots': derived_output_roots,
                  'derived_output_endpoints': {'before': before.get('derived_outputs'), 'after': after.get('derived_outputs')},
                  'operation': endpoint()}
        receipt_sha = M.publish(owned/'SELECTED.json', record)
        return {**record, 'receipt_sha256': receipt_sha}
    except (ValueError, OSError, subprocess.SubprocessError) as error:
        M.publish(owned/'failure.json', {'error': str(error), 'error_type': type(error).__name__,
                                       'retained_root': str(owned), 'selection_succeeded': False, **endpoint()})
        raise


def materialize_selected(selection, owned):
    require(selection.get('schema') == SCHEMA, 'unsupported selected-source schema')
    expected = selection.get('receipt_sha256')
    require(isinstance(expected, str) and re.fullmatch('[0-9a-f]{64}', expected), 'missing selected receipt digest')
    receipt_path = M.canonical(selection['receipt_path']); body = receipt_path.read_bytes()
    require(sha(body) == expected, 'selected receipt changed')
    saved = json.loads(body); supplied = {k: v for k, v in selection.items() if k != 'receipt_sha256'}
    require(json.dumps(saved, sort_keys=True) == json.dumps(supplied, sort_keys=True), 'selected record differs')
    bundle = M.canonical(selection['bundle']['path'])
    require(bundle.parent == receipt_path.parent, 'bundle outside selected source owner')
    identity, _ = M.file_identity(bundle, MAX_BUNDLE_BYTES)
    require(identity['sha256'] == selection['bundle']['sha256'] and identity['bytes'] == selection['bundle']['bytes'], 'selected bundle changed')
    owned = M.canonical(owned)
    live_root = Path(selection['live_census']['root'])
    require(owned != live_root and owned not in live_root.parents and live_root not in owned.parents,
            'materialization overlaps live source')
    require(owned.is_dir() and not any(owned.iterdir()) and owned not in bundle.parents
            and bundle.parent not in owned.parents, 'materialization owner overlaps selection or is nonempty')
    require(git(owned, ['bundle', 'list-heads', str(bundle)]) == (selection['commit']+' HEAD\n').encode(), 'selected bundle object identity differs')
    accepted_root = owned / 'accepted'
    git(owned, ['clone', '--no-checkout', str(bundle), str(accepted_root)])
    git(accepted_root, ['checkout', '--detach', selection['commit']])
    accepted = M.source_identity(accepted_root)
    require(accepted['tree'] == selection['tree'] and accepted['files'] == {
        name: row['accepted'] for name, row in selection['live_census']['files'].items()}, 'selected accepted image differs')
    private = owned / 'preview'; private.mkdir()
    result = M.materialize(accepted_root, private, selection['commit'], selection['tree'])
    require(M.file_identity(bundle, MAX_BUNDLE_BYTES)[0]['sha256'] == selection['bundle']['sha256'], 'bundle changed during materialization')
    origin = {'live_root': str(live_root), 'owned_accepted_root': str(accepted_root),
              'selected_receipt_path': str(receipt_path), 'selected_receipt_sha256': expected,
              'same_path_execution_established': False, 'live_overlay_installation_performed': False}
    M.publish(owned/'SELECTED-ORIGIN.json', origin)
    return {**result, 'selected_origin': origin}


if __name__ == '__main__':
    require(len(sys.argv) == 6 and sys.argv[1] == 'capture', 'usage: capture ROOT OWNED COMMIT OVERLAYS_JSON')
    rows = json.loads(Path(sys.argv[5]).read_bytes())
    result = capture_selection(sys.argv[2], sys.argv[3], sys.argv[4], rows)
    print(json.dumps({key: result[key] for key in ('commit', 'tree', 'receipt_path', 'receipt_sha256')}))
