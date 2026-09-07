"""Public-only Cargo capsule and explicit private overlay qualification.

No live flagged source census, provider, resolver rewrite or runtime integration.
"""
import hashlib
import json
import os
from pathlib import Path
import re
import stat
import tomllib
import types


HERE = Path(__file__).resolve().parent
MATERIALIZER = HERE.parent / 'c15-private-materialization/materialize.py'
MATERIALIZER_SHA = '87546793b76bc7324af55479be47a1e34abbeffb8f2787547d021575a51efab2'
REGISTRY = 'registry+https://github.com/rust-lang/crates.io-index'
MAX_BYTES = 256 * 1024 * 1024


def require(condition, message):
    if not condition: raise ValueError(message)


def digest(body): return hashlib.sha256(body).hexdigest()


def canonical(path):
    path = Path(path)
    require(path.is_absolute() and path.resolve(strict=True) == path and '..' not in path.parts,
            'canonical nonsymlink path required')
    return path


def read_public(path, remaining=MAX_BYTES):
    path = canonical(path); before = path.stat()
    require(stat.S_ISREG(before.st_mode) and before.st_nlink == 1, 'public input is not independent regular file')
    require(before.st_size <= remaining, 'public input byte bound exceeded')
    with path.open('rb') as source:
        body = source.read(remaining + 1); after = os.fstat(source.fileno())
    require(len(body) <= remaining and (before.st_dev, before.st_ino, before.st_size, before.st_mtime_ns)
            == (after.st_dev, after.st_ino, after.st_size, after.st_mtime_ns), 'public input drift or bound exceeded')
    return body


def materializer():
    body = read_public(MATERIALIZER)
    require(digest(body) == MATERIALIZER_SHA, 'qualified materializer source differs')
    module = types.ModuleType('qualified_materializer'); module.__file__ = str(MATERIALIZER)
    exec(compile(body, str(MATERIALIZER), 'exec'), module.__dict__)
    return module


def shard(name):
    if len(name) < 3: return f'{len(name)}/{name}'
    if len(name) == 3: return f'3/{name[0]}/{name}'
    return f'{name[:2]}/{name[2:4]}/{name}'


def prepare_capsule(lock_bytes, registry_root, registry_id, destination):
    registry_root, destination = canonical(registry_root), canonical(destination)
    require(destination.is_dir() and not any(destination.iterdir()), 'capsule destination must be empty')
    require(destination != registry_root and destination not in registry_root.parents
            and registry_root not in destination.parents, 'capsule overlaps registry')
    require(isinstance(registry_id, str) and re.fullmatch(r'index\.crates\.io-[A-Za-z0-9_-]+', registry_id), 'unsupported registry identity')
    packages = tomllib.loads(lock_bytes.decode())['package']; expected = {}
    require(len(packages) <= 10000, 'package count bound')
    for row in packages:
        if 'source' not in row: continue
        require(row['source'] == REGISTRY, 'unsupported registry or external dependency')
        name, version, checksum = row['name'], row['version'], row['checksum']
        require(re.fullmatch('[A-Za-z0-9_-]+', name) and re.fullmatch('[A-Za-z0-9.+-]+', version)
                and re.fullmatch('[0-9a-f]{64}', checksum), 'invalid locked package identity')
        require((name, version) not in expected, 'duplicate locked package')
        expected[name, version] = checksum
    require(expected, 'no locked public registry packages')
    held = {}; sources = {}; total = 0
    def capture(relative):
        nonlocal total
        source = registry_root / relative
        body = read_public(source, MAX_BYTES-total); total += len(body)
        held[relative] = body; sources[str(source)] = digest(body)
        return body
    config = capture(f'index/{registry_id}/config.json')
    require(json.loads(config) == {'dl': 'https://static.crates.io/crates', 'api': 'https://crates.io'},
            'unsupported public registry configuration')
    indexed = {}
    for name in sorted({name for name, _ in expected}):
        body = capture(f'index/{registry_id}/.cache/{shard(name)}')
        require(body.startswith(b'\x03\x02\x00\x00\x00'), 'unsupported sparse cache format')
        parts = body[5:].split(b'\0')
        require(parts[-1] == b'' and (len(parts)-2) % 2 == 0, 'malformed sparse cache records')
        for index in range(1, len(parts)-1, 2):
            version = parts[index].decode(); row = json.loads(parts[index+1])
            require(row.get('name') == name and row.get('vers') == version, 'index identity mismatch')
            require((name, version) not in indexed, 'duplicate index version')
            indexed[name, version] = row.get('cksum')
    require(all(indexed.get(key) == value for key, value in expected.items()), 'locked checksum absent from index')
    missing = []
    for (name, version), checksum in sorted(expected.items()):
        relative = f'cache/{registry_id}/{name}-{version}.crate'
        path = registry_root / relative
        if not path.exists() and not path.is_symlink(): missing.append(f'{name}-{version}'); continue
        require(digest(capture(relative)) == checksum, 'crate archive differs from locked checksum')
    # No Cargo home/config/source-directory enumeration: only explicit public files.
    for relative, body in held.items():
        target = destination / relative; target.parent.mkdir(parents=True, exist_ok=True)
        with target.open('xb') as out: out.write(body); out.flush(); os.fsync(out.fileno())
        target.chmod(0o400)
    require(all(digest(read_public(Path(path))) == value for path, value in sources.items()), 'registry input changed')
    result = {'schema': 'c15-public-cargo-capsule-v1', 'root': str(destination),
              'lock_sha256': digest(lock_bytes), 'sources': sources,
              'files': {name: digest(body) for name, body in held.items()},
              'locked_versions': len(expected), 'index_names': len({name for name, _ in expected}),
              'missing_archives': missing, 'credentials_or_config_copied': False}
    materializer().publish(destination / 'CAPSULE.json', result)
    return result


def prepare_cargo_home(scratch, receipt):
    scratch = canonical(scratch); capsule = canonical(receipt['root'])
    require(all(digest(read_public(capsule/name)) == value for name, value in receipt['files'].items()),
            'capsule source differs')
    home = scratch / 'cargo-home'; home.mkdir()
    (home / 'registry').mkdir()
    for name in ('cache', 'index'):
        (home / 'registry' / name).symlink_to('/cargo-public/' + name, target_is_directory=True)
    return str(home)


def install_private_overlays(snapshot, overlays):
    m = materializer(); repo = canonical(snapshot['repo'])
    before = m.source_identity(repo)
    require(before == snapshot['initial_private'], 'overlays require the original private accepted image')
    require(isinstance(overlays, list) and overlays, 'explicit overlay list required')
    held = {}; fields = {'path', 'base_sha256', 'base_mode', 'effective_text', 'effective_sha256',
                         'effective_mode', 'observed_index_flag'}
    total = 0
    for row in overlays:
        require(isinstance(row, dict) and set(row) == fields, 'unknown overlay fields')
        name = row['path']; path = Path(name)
        require(isinstance(name, str) and name and str(path) == name and not path.is_absolute()
                and '..' not in path.parts and name not in held, 'invalid/duplicate overlay path')
        require(row['observed_index_flag'] in {'skip-worktree', 'assume-unchanged'}, 'unsupported index flag')
        require(name in before['files'] and before['files'][name]['sha256'] == row['base_sha256']
                and before['files'][name]['mode'] == row['base_mode'], 'overlay accepted base differs')
        require(row['effective_mode'] in {'100644', '100755'} and isinstance(row['effective_text'], str), 'invalid overlay image')
        body = row['effective_text'].encode(); total += len(body)
        require(total <= MAX_BYTES and digest(body) == row['effective_sha256'], 'overlay body differs or bound exceeded')
        held[name] = body
    folder = canonical(snapshot['owned']) / 'evidence/immutable-overlays'; folder.mkdir()
    m.publish(folder/'DECLARATION.json', overlays)
    try:
        for row in overlays:
            path = repo / row['path']
            with path.open('wb') as out: out.write(held[row['path']]); out.flush(); os.fsync(out.fileno())
            path.chmod(0o755 if row['effective_mode'] == '100755' else 0o644)
        names = list(held)
        m.git(repo, ['add', '--', *names])
        m.git(repo, ['-c', 'user.name=Private Overlay', '-c', 'user.email=preview@example.invalid',
                     'commit', '-qm', 'Record declared private source overlay'])
        after = m.source_identity(repo)
        result = {'schema': 'c15-private-source-overlay-v1', 'accepted_commit': snapshot['shared']['head'],
                  'accepted_tree': snapshot['shared']['tree'], 'overlays': overlays,
                  'private_before': before, 'private_after': after, 'shared_accepted': False,
                  'live_overlay_census_proved': False, 'promotion': False}
        m.publish(folder/'RESULT.json', result)
        return result
    except (ValueError, OSError) as error:
        m.publish(folder/'failure.json', {'error': str(error), 'promotion': False})
        raise
