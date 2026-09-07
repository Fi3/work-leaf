"""One frozen diagnostic command; original exit and publication are distinct.

The admitted argv owns its external timeout. This launcher neither retries nor
constructs provider arguments. Source checks are one linear pass at each endpoint.
Timestamps bracket the supervised command, not the provider's internal turn.
"""
from datetime import datetime, timezone
import hashlib
import json
import os
from pathlib import Path
import subprocess
import sys
import time


def sha(path):
    return hashlib.sha256(Path(path).read_bytes()).hexdigest()


def require(value, message):
    if not value: raise ValueError(message)


def publish(path, value):
    with Path(path).open('x', encoding='utf-8') as out:
        json.dump(value, out, indent=2, sort_keys=True, allow_nan=False)
        out.write('\n'); out.flush(); os.fsync(out.fileno())


def now():
    return datetime.now(timezone.utc).isoformat()


def run(admission_path, expected_sha256):
    admission_path = Path(admission_path)
    admission_bytes = admission_path.read_bytes()
    require(hashlib.sha256(admission_bytes).hexdigest() == expected_sha256, 'admission digest differs')
    admitted = json.loads(admission_bytes)
    require(admitted['schema'] == 'work-leaf-single-diagnostic-admission-v1', 'unsupported admission')
    require(type(admitted['run_id']) is str and admitted['run_id'], 'missing run identity')
    root = Path(admitted['artifact_root']); cwd = Path(admitted['cwd'])
    require(root.is_absolute() and root.resolve() == root and root.is_dir(), 'invalid evidence root')
    require(cwd.is_absolute() and cwd.resolve() == cwd and cwd.is_dir(), 'invalid cwd')
    argv = admitted['argv']
    require(type(argv) is list and argv and all(type(x) is str and x and '\0' not in x for x in argv), 'invalid argv')
    sources = admitted['source_sha256']
    require(type(sources) is dict and sources, 'source pins required')
    require(admitted['environment_path'] in sources, 'environment is not pinned')
    environment_bytes = Path(admitted['environment_path']).read_bytes()
    for path, expected in sources.items():
        observed = hashlib.sha256(environment_bytes).hexdigest() if path == admitted['environment_path'] else sha(path)
        require(observed == expected, 'source digest differs: ' + path)
    environment = json.loads(environment_bytes)
    child_environment = dict(os.environ)
    for key in environment['unset']: child_environment.pop(key, None)
    child_environment.update(environment['variables'])
    for name in ('ATTEMPT.json', 'TERMINAL.json', 'PROCESS.stdout', 'PROCESS.stderr'):
        require(not os.path.lexists(root / name), 'existing publication or attempt: ' + name)
    started = now(); monotonic = time.monotonic()
    publish(root / 'ATTEMPT.json', {'run_id': admitted['run_id'], 'admission_sha256': expected_sha256,
        'started_at': started, 'retry_authorized': False, 'argv': argv})
    result = {'schema': 'work-leaf-single-diagnostic-terminal-v1',
        'id': admitted['run_id'], 'run_id': admitted['run_id'],
        'condition': admitted['condition'], 'observer_condition': admitted['observer_condition'],
        'started_at': started, 'launch_status': 'spawn_failed', 'launcher_exit_code': None,
        'admission_sha256': expected_sha256, 'timestamp_scope': 'supervised command, not provider turn'}
    try:
        with (root / 'PROCESS.stdout').open('xb') as stdout, (root / 'PROCESS.stderr').open('xb') as stderr:
            completed = subprocess.run(argv, cwd=cwd, env=child_environment, stdin=subprocess.DEVNULL,
                stdout=stdout, stderr=stderr, check=False)
        result['launch_status'] = 'completed'
        result['launcher_exit_code'] = completed.returncode
    except OSError as error:
        result['error'] = type(error).__name__ + ': ' + str(error)
    result['finished_at'] = now(); result['duration_seconds'] = time.monotonic() - monotonic
    errors = []
    for path, expected in (sources | {str(admission_path): expected_sha256}).items():
        try: require(sha(path) == expected, 'digest differs')
        except (OSError, ValueError) as error: errors.append({'path': path, 'error': str(error)})
    result['source_endpoint_errors'] = errors
    result['source_endpoints_match'] = not errors
    publish(root / 'TERMINAL.json', result)
    return result


if __name__ == '__main__':
    require(len(sys.argv) == 3, 'usage: bounded_launch.py ADMISSION SHA256')
    result = run(sys.argv[1], sys.argv[2])
    print(json.dumps(result, sort_keys=True))
    code = result['launcher_exit_code']
    raise SystemExit(code if type(code) is int and code >= 0 else 1)
