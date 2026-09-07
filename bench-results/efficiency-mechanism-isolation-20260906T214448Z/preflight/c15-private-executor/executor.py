"""Private Linux-only preview executor qualification; no provider or WL protocol.

Trusted admission supplies readonly mount roots. They must contain public runtime,
cache, or fixture data, never credentials. No host root/home/config mount or inherited
environment is supplied automatically. Unsupported setup has no unconfined fallback.
"""
import hashlib
import math
import os
from pathlib import Path
import re
import selectors
import signal
import stat
import subprocess
import time


SYSTEM_DIRS = ('/usr', '/bin', '/lib', '/lib64')
PRIVATE_TARGETS = ('/build', '/tmp')


def require(value, message):
    if not value:
        raise ValueError(message)


def canonical(raw, directory=True):
    require(isinstance(raw, str) and raw.startswith('/'), 'absolute source path required')
    path = Path(raw)
    require(str(path) == raw and '..' not in path.parts and path.resolve(strict=True) == path,
            'canonical nonsymlink source path required')
    require(path.is_dir() if directory else path.is_file(), 'wrong source kind')
    return path


def destination(raw):
    require(isinstance(raw, str) and raw.startswith('/'), 'absolute destination required')
    path = Path(raw)
    require(str(path) == raw and '..' not in path.parts and raw != '/', 'normalized nonroot destination required')
    return path


def overlaps(a, b):
    return a == b or a in b.parents or b in a.parents


def signature(path):
    meta = path.stat()
    return [meta.st_dev, meta.st_ino, meta.st_mode, meta.st_size, meta.st_mtime_ns]


def validate_owned_trees(roots):
    """Reject external hardlink aliases/special files without following symlinks."""
    links = {}
    stack = [(root, 0) for root in roots]
    count = 0
    while stack:
        path, depth = stack.pop()
        require(depth <= 64, 'owned tree exceeds depth bound')
        with os.scandir(path) as entries:
            for entry in entries:
                count += 1
                require(count <= 200000, 'owned tree exceeds entry bound')
                meta = entry.stat(follow_symlinks=False)
                if stat.S_ISDIR(meta.st_mode):
                    stack.append((Path(entry.path), depth + 1))
                elif stat.S_ISREG(meta.st_mode):
                    key = (meta.st_dev, meta.st_ino)
                    observed, expected = links.get(key, (0, meta.st_nlink))
                    require(expected == meta.st_nlink, 'hardlink count changed')
                    links[key] = (observed + 1, expected)
                else:
                    require(stat.S_ISLNK(meta.st_mode), 'special file in owned tree')
    require(all(observed == expected for observed, expected in links.values()),
            'hardlink reaches outside owned writable trees')
    return count


def run_preview(spec, argv, *, timeout=5, cancel=None, max_output_bytes=1024 * 1024):
    require(type(timeout) in (int, float) and math.isfinite(timeout) and 0 < timeout <= 300,
            'timeout must be finite, positive and at most 300 seconds')
    require(type(max_output_bytes) is int and 0 < max_output_bytes <= 16 * 1024 * 1024,
            'invalid output limit')
    require(isinstance(argv, list) and argv and all(isinstance(x, str) and '\x00' not in x for x in argv),
            'exact nonempty argument list required')
    destination(argv[0])
    require(set(spec) == {'bwrap', 'bwrap_sha256', 'source', 'build', 'scratch', 'view', 'readonly'},
            'unsupported executor specification')
    binary = canonical(spec['bwrap'], False)
    require(isinstance(spec['bwrap_sha256'], str) and re.fullmatch('[0-9a-f]{64}', spec['bwrap_sha256']),
            'invalid executable digest')
    binary_bytes = binary.read_bytes()
    require(hashlib.sha256(binary_bytes).hexdigest() == spec['bwrap_sha256'], 'executable digest mismatch')
    require(not binary.stat().st_mode & (stat.S_IWGRP | stat.S_IWOTH), 'writable shared executor')
    binary_identity = signature(binary)
    writable = [canonical(spec[key]) for key in ('source', 'build', 'scratch')]
    for i, path in enumerate(writable):
        require(all(not overlaps(path, other) for other in writable[:i]), 'overlapping writable sources')
    owned_entries = validate_owned_trees(writable)
    view = destination(spec['view'])
    require(all(not overlaps(view, Path(x)) for x in (*SYSTEM_DIRS, '/proc', '/dev', '/build')),
            'source view overlaps reserved runtime mount')
    require(view != Path('/tmp') and view not in Path('/tmp').parents, 'source view hides scratch')
    require(isinstance(spec['readonly'], list) and len(spec['readonly']) <= 32, 'invalid readonly mount list')
    mounts = [(Path(x).resolve(strict=True), Path(x), False) for x in SYSTEM_DIRS if Path(x).exists()]
    extras = []
    for row in spec['readonly']:
        require(isinstance(row, dict) and set(row) == {'source', 'target'}, 'invalid readonly mount')
        source, target = canonical(row['source']), destination(row['target'])
        require(all(not overlaps(source, other) for other in writable), 'readonly source aliases writable source')
        require(all(not overlaps(target, other) for other in [view, Path('/proc'), Path('/dev'), *map(Path, PRIVATE_TARGETS), *[m[1] for m in mounts], *extras]),
                'overlapping readonly destination')
        require(source != Path('/') and source != Path.home(), 'broad readonly host root/home forbidden')
        mounts.append((source, target, False))
        extras.append(target)
    mounts.extend([(writable[1], Path('/build'), True), (writable[2], Path('/tmp'), True), (writable[0], view, True)])
    # Scratch is owned execution output; never source a home or credential directory.
    (writable[2] / 'home').mkdir(exist_ok=True)
    require(not (writable[2] / 'home').is_symlink(), 'private home is symlinked')
    fds = []
    identities = []
    child = None
    selector = selectors.DefaultSelector()
    start = time.monotonic()
    try:
        command = [str(binary), '--unshare-all', '--unshare-user', '--die-with-parent', '--new-session',
                   '--disable-userns', '--assert-userns-disabled', '--cap-drop', 'ALL', '--clearenv']
        for source, target, write in mounts:
            fd = os.open(source, os.O_RDONLY | os.O_DIRECTORY | os.O_NOFOLLOW)
            fds.append(fd)
            meta = os.fstat(fd)
            require((meta.st_dev, meta.st_ino) == (source.stat().st_dev, source.stat().st_ino), 'mount identity changed')
            identities.append({'source': str(source), 'target': str(target), 'writable': write,
                               'device': meta.st_dev, 'inode': meta.st_ino})
            command.extend(['--bind-fd' if write else '--ro-bind-fd', str(fd), str(target)])
        command += ['--proc', '/proc', '--remount-ro', '/proc', '--dev', '/dev', '--remount-ro', '/dev',
                    '--remount-ro', '/', '--chdir', str(view),
                    '--setenv', 'PATH', '/usr/bin:/bin', '--setenv', 'HOME', '/tmp/home',
                    '--setenv', 'TMPDIR', '/tmp', '--setenv', 'CARGO_TARGET_DIR', '/build',
                    '--setenv', 'CARGO_HOME', '/tmp/cargo-home', '--setenv', 'LC_ALL', 'C.UTF-8',
                    '--', *argv]
        child = subprocess.Popen(command, stdin=subprocess.DEVNULL, stdout=subprocess.PIPE,
                                 stderr=subprocess.PIPE, pass_fds=tuple(fds), close_fds=True,
                                 start_new_session=True, env={'PATH': '/usr/bin:/bin'})
        for fd in fds:
            os.close(fd)
        fds.clear()
        for name, pipe in (('stdout', child.stdout), ('stderr', child.stderr)):
            os.set_blocking(pipe.fileno(), False)
            selector.register(pipe, selectors.EVENT_READ, name)
        captured = {'stdout': bytearray(), 'stderr': bytearray()}
        total = 0
        stop = None
        kill_at = None
        while selector.get_map() or child.poll() is None:
            elapsed = time.monotonic() - start
            reason = 'cancelled' if cancel is not None and cancel.is_set() else 'timeout' if elapsed >= timeout else None
            if stop is None and reason:
                stop, kill_at = reason, time.monotonic()
                try: os.killpg(child.pid, signal.SIGKILL)
                except ProcessLookupError: pass
            if kill_at is not None and time.monotonic() - kill_at > 2:
                raise RuntimeError('preview process/output boundary did not close; preserve execution tree')
            for key, _ in selector.select(.02):
                chunk = os.read(key.fileobj.fileno(), 65536)
                if not chunk:
                    selector.unregister(key.fileobj)
                    key.fileobj.close()
                    continue
                remaining = max_output_bytes - total
                captured[key.data].extend(chunk[:remaining])
                total += min(len(chunk), remaining)
                if len(chunk) > remaining and stop is None:
                    stop, kill_at = 'output_limit', time.monotonic()
                    try: os.killpg(child.pid, signal.SIGKILL)
                    except ProcessLookupError: pass
        code = child.wait(timeout=1)
        require(signature(binary) == binary_identity and hashlib.sha256(binary.read_bytes()).hexdigest() == spec['bwrap_sha256'],
                'executor changed during execution')
        for row in identities:
            meta = Path(row['source']).stat()
            require((meta.st_dev, meta.st_ino) == (row['device'], row['inode']), 'mount source path changed; preserve execution tree')
        return {'schema': 'work-leaf-private-preview-executor-qualification-v1',
                'exit_code': code, 'timed_out': stop == 'timeout', 'cancelled': stop == 'cancelled',
                'stop_reason': stop, 'closed': True, 'elapsed_seconds': time.monotonic() - start,
                'stdout': captured['stdout'].decode('utf-8', errors='replace'),
                'stderr': captured['stderr'].decode('utf-8', errors='replace'),
                'stdout_sha256': hashlib.sha256(captured['stdout']).hexdigest(),
                'stderr_sha256': hashlib.sha256(captured['stderr']).hexdigest(),
                'captured_bytes': total, 'argv': list(argv), 'cwd': str(view),
                'initial_owned_entries': owned_entries,
                'bwrap_sha256': spec['bwrap_sha256'], 'mounts': identities,
                'scope': 'closed sandbox pipes/process; no test-purpose, source-equivalence or provider claim'}
    finally:
        if child is not None and child.poll() is None:
            try: os.killpg(child.pid, signal.SIGKILL)
            except ProcessLookupError: pass
            child.wait(timeout=2)
        for fd in fds:
            os.close(fd)
        for key in list(selector.get_map().values()):
            key.fileobj.close()
        selector.close()
