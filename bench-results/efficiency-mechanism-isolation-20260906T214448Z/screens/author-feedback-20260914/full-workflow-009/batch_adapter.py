"""Exactly three A+B reversals through the retained one-shot benchmark engine."""
import hashlib
from pathlib import Path
import sys
import types

HERE = Path(__file__).resolve().parent
STUDY = HERE.parents[2]
ORIGINAL = STUDY / 'preflight/standalone-global-hunk-pilot-20260913/runner_global_hunk.py'
ORIGINAL_SHA = '7e4a988d4aa96549846927a0976d1efd7c323b2008ac03444dc7a665e8feb3d0'
PHASE = 'author-joint-confirmation-20260915'
CONDITION = 'author-publication-guidance-off'
IDS = ['author-joint-confirmation-001', 'author-joint-confirmation-002', 'author-joint-confirmation-003']


def digest(path):
    return hashlib.sha256(Path(path).read_bytes()).hexdigest()


def load_runner(monitor=None):
    raw = ORIGINAL.read_bytes()
    if hashlib.sha256(raw).hexdigest() != ORIGINAL_SHA:
        raise ValueError('retained one-shot runner changed')
    runner = types.ModuleType('retained_runner_for_joint_confirmation')
    runner.__file__ = str(ORIGINAL)
    exec(compile(raw, str(ORIGINAL), 'exec'), runner.__dict__)
    source = runner.E.ENGINE_SOURCE
    changes = [
        ('"maximum_concurrent_workflows": 1', '"maximum_concurrent_workflows": 3', 1),
        ('manifest.get("maximum_concurrent_workflows") != 1', 'manifest.get("maximum_concurrent_workflows") != 3', 1),
        (repr(runner.IDS), repr(IDS), 2),
        ('"workflow_count": 1,', '"workflow_count": 3,', 1),
        ('        while any(process.poll() is None for process, _, _ in active):\n',
         '        while any(process.poll() is None for process, _, _ in active):\n'
         '            P09_RESOURCE_MONITOR(active, manifest)\n', 1),
    ]
    for before, after, count in changes:
        if source.count(before) != count:
            raise ValueError('retained engine source anchor changed: ' + before)
        source = source.replace(before, after)
    engine = types.ModuleType('three_modified_one_shot_engine')
    engine.__file__ = str(runner.ENGINE_PATH)
    for key in ('GENERATED_NAME', 'ADAPTER_SHA', 'SELF', 'ENGINE_PATH', 'configuration'):
        engine.__dict__[key] = runner.__dict__[key]
    engine.ENGINE_SOURCE = source
    engine.ENGINE_SUBSTITUTIONS = runner.E.ENGINE_SUBSTITUTIONS + changes
    engine.P09_RESOURCE_MONITOR = monitor
    exec(compile(source, str(runner.ENGINE_PATH), 'exec'), engine.__dict__)
    runner.E = engine
    runner._prepare, runner._verify, runner._run = engine.prepare, engine.verify_manifest, engine.run_batch
    runner._environment = engine.run_environment
    runner.IDS, runner.PHASE, runner.CONDITION = IDS, PHASE, CONDITION

    def validate(rows):
        if (not isinstance(rows, list) or len(rows) != 3 or [r.get('run_id') for r in rows] != IDS
                or any(r.get('condition') != CONDITION for r in rows)):
            raise ValueError('exactly three declared modified workflows are required')

    previous_environment = runner.run_environment

    def environment(manifest, row, inherited):
        env = previous_environment(manifest, row, inherited)
        env.update(WORK_LEAF_BENCH_FULL_INVERSE='1', WORK_LEAF_BENCH_INVERSE='1',
                   WORK_LEAF_BENCH_P01_CATALOG='1',
                   WORK_LEAF_BENCH_FULL_HOST=str(HERE/'host_custody.py'),
                   WORK_LEAF_BENCH_FULL_PROVIDER=str(HERE/'input_provider'),
                   WORK_LEAF_BENCH_INPUT_PLAN=str(HERE/'REFERENCE-PLAN.json'),
                   WORK_LEAF_BENCH_INPUT_PLAN_SHA256=digest(HERE/'REFERENCE-PLAN.json'),
                   WORK_LEAF_BENCH_INPUT_RECEIPTS=str(Path(row['results_dir'])/'input-receipts'))
        return env

    runner.validate_schedule = engine.validate_schedule = validate
    runner.run_environment = engine.run_environment = environment
    engine.schedule = runner.schedule
    engine.verify_manifest = runner.verify_manifest
    run = runner.run_batch

    def monitored_run(batch, popen=None):
        if monitor is None:
            raise ValueError('resource monitor must be ready before generation')
        return run(batch, popen=popen)

    runner.run_batch = monitored_run
    return runner


if __name__ == '__main__':
    if sys.argv[1:2] == ['run']:
        from full_monitor import ResourceMonitor
        runner = load_runner(ResourceMonitor())
    else:
        runner = load_runner()
    raise SystemExit(runner.main())
