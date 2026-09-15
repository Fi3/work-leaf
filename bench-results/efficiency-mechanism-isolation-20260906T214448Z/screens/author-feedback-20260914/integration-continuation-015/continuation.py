"""Private integration-only seam over the retained, qualified complete driver."""
import argparse
import hashlib
import json
import os
from pathlib import Path
import re
import shlex
import subprocess
import sys
import time

HERE = Path(__file__).resolve().parent
sys.path.insert(0, str(HERE.parent / 'guard-path-011'))
sys.path.insert(0, str(HERE.parent / 'permissions-input-013'))
import repair_driver as repair
import trust_table_probe


def require(value, message):
    if not value:
        raise ValueError(message)


def replacements(source):
    start = source.index('  for feature_index in 1 2 3; do\n',
                         source.index('run_sequential_bench() {\n'))
    end = source.index('  review_completed="yes"\n', start)
    anchor = 'git -C "$checkout_dir" config user.name "Work Leaf Bench"\n'
    restore_command = ('python3 ' + shlex.quote(str(HERE / 'continuation.py'))
        + ' restore --repo "$checkout_dir" --run-id "$run_id"'
        + ' --seeds "${WORK_LEAF_BENCH_CONTINUATION_SEEDS:?required}"'
        + ' --seeds-sha "${WORK_LEAF_BENCH_CONTINUATION_SEEDS_SHA256:?required}"'
        + ' --result "$tmp_root/runs/CONTINUATION-SOURCE.json"'
        + ' || fail_bench "retained prefix source qualification failed"\n')
    return [
        ('set -euo pipefail\n', 'set -euo pipefail\n'
         '[[ "${WORK_LEAF_BENCH_INTEGRATION_CONTINUATION:-}" == "1" ]] || exit 86\n'),
        (anchor, anchor + restore_command),
        (source[start:end],
         '  [[ -f "$tmp_root/runs/CONTINUATION-SOURCE.json" ]] '
         '|| fail_bench "qualified retained prefix is missing"\n'
         '  [[ -f "$tmp_root/runs/CONTINUATION-CACHE.json" ]] '
         '|| fail_bench "retained-prefix cache preparation is missing"\n'),
    ]


def driver_source():
    original = repair.driver_source()
    result = original
    for before, after in replacements(original):
        result = repair.original.inverse.replace_once(result, before, after)
    return result


def provider_args(argv, stage):
    require(stage in ('linearize-plan', 'linearize-accept'),
            'only integration stages are admitted')
    require(argv.count('--cd') == 1 and '-c' not in argv and '--config' not in argv,
            'ambiguous cwd or extra configuration')
    index = argv.index('--cd')
    require(index + 1 < len(argv), 'missing exact cwd')
    repo = Path(argv[index + 1])
    # Reuse the actual-agent-qualified table-value implementation unchanged.
    qualified = trust_table_probe.command(HERE / 'input_provider', repo, None)
    return qualified[1:3] + list(argv)


def select_seed(manifest, run_id):
    require(manifest.get('schema') == 1 and isinstance(manifest.get('rows'), list),
            'unsupported seed manifest')
    rows = [row for row in manifest['rows'] if row.get('run_id') == run_id]
    require(len(rows) == 1, 'foreign or ambiguous continuation identity')
    row = rows[0]
    require(row.get('original_run_id') != run_id, 'original outcome cannot be overwritten')
    for key in ('base', 'head', 'tree'):
        require(isinstance(row.get(key), str) and re.fullmatch(r'[0-9a-f]{40}', row[key]),
                'invalid retained Git identity')
    require(isinstance(row.get('pins'), dict) and row['pins'], 'missing retained evidence')
    for filename, expected in row['pins'].items():
        path = Path(filename)
        require(path.is_absolute() and path.is_file() and not path.is_symlink(),
                'nonregular retained evidence')
        require(hashlib.sha256(path.read_bytes()).hexdigest() == expected,
                'retained source evidence differs: ' + filename)
    return row


def restore(repo, seed):
    repo = Path(repo).resolve()
    def git(*args):
        return subprocess.run(['git', '-C', str(repo), *args],
                              check=True, capture_output=True).stdout
    require(git('rev-parse', '--show-toplevel').decode().strip() == str(repo),
            'not an exact checkout root')
    require(git('rev-parse', 'HEAD').decode().strip() == seed['base'],
            'initial checkout differs from the fixed base')
    require(not git('status', '--porcelain', '--untracked-files=all'),
            'initial checkout is dirty')
    bundle = Path(seed['bundle'])
    require(str(bundle) in seed['pins'] and str(Path(seed['checkpoint_graph'])) in seed['pins'],
            'bundle and full checkpoint graph must be pinned')
    require(git('bundle', 'list-heads', str(bundle)).decode().strip() == seed['head'] + ' HEAD',
            'bundle contains a foreign retained head')
    refs_before = git('show-ref')
    git('bundle', 'verify', str(bundle))
    git('bundle', 'unbundle', str(bundle))
    git('checkout', '--detach', seed['head'])
    require(git('show-ref') == refs_before, 'source clone refs changed')
    require(git('rev-parse', 'HEAD^{tree}').decode().strip() == seed['tree'],
            'restored tree differs')
    require(not git('status', '--porcelain', '--untracked-files=all'),
            'restored checkout is dirty')
    graph = git('log', '--graph', '--format=%H%x09%P%x09%s', '--all')
    require(graph == Path(seed['checkpoint_graph']).read_bytes(),
            'restored all-ref commit graph differs')
    return dict(schema=1, original_run_id=seed['original_run_id'], run_id=seed['run_id'],
                head=seed['head'], tree=seed['tree'], clean=True,
                source_refs_preserved=True, all_ref_commit_graph_matches=True,
                graph_sha256=hashlib.sha256(graph).hexdigest(),
                refs_sha256=hashlib.sha256(refs_before).hexdigest(),
                scope='exact source chain; old model outcomes remain unchanged')


def warm_commands():
    return [
        ['cargo', 'clippy', '--all-targets', '--all-features', '--', '-D', 'warnings'],
        ['cargo', 'test', '--all-targets', '--all-features', '--no-run'],
    ]


def warm_cache(repo, root):
    host = repair.original.load_host()
    initial = host.snapshot(repo)
    deadline = time.monotonic() + 240
    receipts = []
    for ordinal, argv in enumerate(warm_commands(), 1):
        result = host.execute_child(argv, repo, None,
            root / f'cache-{ordinal}.stdout', root / f'cache-{ordinal}.stderr',
            deadline - time.monotonic(), dict(os.environ))
        host.save_json(root / f'cache-{ordinal}.json', result)
        require(result['exit_code'] in (0, 101) and not result['timed_out']
                and not result['cancelled_signal'], 'cache compilation did not complete')
        require(host.snapshot(repo) == initial, 'cache preparation changed source state')
        receipts.append(result)
    return dict(completed=True, model_generation=False, tests_executed=False,
                source_unchanged=True, commands=receipts,
                scope='compile retained source, not a quality gate; failures remain for integration and no output is sent to the model')


def main(argv=None):
    argv = list(sys.argv[1:] if argv is None else argv)
    if argv == ['driver']:
        print(driver_source(), end='')
        return 0
    parser = argparse.ArgumentParser()
    parser.add_argument('action', choices=['restore'])
    for key in ('repo', 'seeds', 'result'):
        parser.add_argument('--' + key, type=Path, required=True)
    parser.add_argument('--run-id', required=True)
    parser.add_argument('--seeds-sha', required=True)
    args = parser.parse_args(argv)
    require(os.environ.get('WORK_LEAF_BENCH_INTEGRATION_CONTINUATION') == '1',
            'explicit integration continuation opt-in required')
    raw = args.seeds.read_bytes()
    require(hashlib.sha256(raw).hexdigest() == args.seeds_sha, 'seed manifest differs')
    require(not args.result.exists(), 'continuation source result already exists')
    seed = select_seed(json.loads(raw), args.run_id)
    value = restore(args.repo, seed)
    with args.result.open('x') as output:
        json.dump(value, output, indent=2)
    if os.environ.get('WORK_LEAF_BENCH_CONTINUATION_WARM_CACHE') == '1':
        cache = warm_cache(args.repo.resolve(), args.result.parent)
        with args.result.with_name('CONTINUATION-CACHE.json').open('x') as output:
            json.dump(cache, output, indent=2)
    return 0


if __name__ == '__main__':
    raise SystemExit(main())
