"""Exactly three retained-prefix integrations with independent resource limits."""
import hashlib
from pathlib import Path
import sys
import types

HERE = Path(__file__).resolve().parent
PRIOR = HERE.parent / 'full-workflow-009'
sys.path.insert(0, str(PRIOR))


def pinned(path, expected):
    raw = path.read_bytes()
    if hashlib.sha256(raw).hexdigest() != expected:
        raise ValueError('retained continuation infrastructure changed')
    return raw.decode()


def load_runner(monitor=None):
    path = PRIOR / 'batch_adapter.py'
    source = pinned(path, '4de0f92a67aefd58d84f341a2ae14d319ca03e412ab6b277978ceed276c4b84c')
    anchor = '    changes = [\n'
    if source.count(anchor) != 1:
        raise ValueError('retained batch extension seam changed')
    source = source.replace(anchor, anchor +
        '        (\'"supervisor_wall_timeout_seconds": 5400\', \'"supervisor_wall_timeout_seconds": 1800\', 1),\n'
        '        (\'manifest.get("supervisor_wall_timeout_seconds") != 5400\', '
        '\'manifest.get("supervisor_wall_timeout_seconds") != 1800\', 1),\n')
    prior = types.ModuleType('retained_three_integration_engine')
    prior.__file__ = str(path)
    exec(compile(source, str(path), 'exec'), prior.__dict__)
    prior.PHASE = 'author-joint-integration-20260915'
    prior.IDS = ['author-joint-integration-004', 'author-joint-integration-005',
                 'author-joint-integration-006']
    runner = prior.load_runner(monitor)
    previous = runner.run_environment

    def environment(manifest, row, inherited):
        env = previous(manifest, row, inherited)
        seed = HERE / 'SEEDS.json'
        env.update(WORK_LEAF_BENCH_INTEGRATION_CONTINUATION='1',
                   WORK_LEAF_BENCH_CONTINUATION_WARM_CACHE='1',
                   WORK_LEAF_BENCH_CATALOG_IMMUTABLE='1',
                   WORK_LEAF_BENCH_CONTINUATION_SEEDS=str(seed),
                   WORK_LEAF_BENCH_CONTINUATION_SEEDS_SHA256=prior.digest(seed),
                   WORK_LEAF_BENCH_FULL_PROVIDER=str(HERE.parent / 'catalog-immutability-016/input_provider'),
                   WORK_LEAF_DIRECT_BENCH_TIMEOUT_SECS='1800')
        return env

    runner.run_environment = runner.E.run_environment = environment
    return runner


def load_monitor():
    path = PRIOR / 'full_monitor.py'
    source = pinned(path, 'b4e0ceb51678e75367bbf1416fd481ddc80c6574c2825ddd126e4d9bea04d9d4')
    if source.count('>= 45_000_000') != 1:
        raise ValueError('retained resource threshold seam changed')
    source = source.replace('>= 45_000_000', '>= 12_000_000')
    module = types.ModuleType('retained_integration_resource_monitor')
    module.__file__ = str(path)
    exec(compile(source, str(path), 'exec'), module.__dict__)
    return module.ResourceMonitor(), source


if __name__ == '__main__':
    monitor = load_monitor()[0] if sys.argv[1:2] == ['run'] else None
    raise SystemExit(load_runner(monitor).main())
