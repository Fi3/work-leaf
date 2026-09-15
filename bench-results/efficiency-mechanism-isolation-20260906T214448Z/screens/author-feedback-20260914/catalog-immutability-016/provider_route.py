"""Integration-only provider route with an immutable private catalog view."""
import os
from pathlib import Path
import sys

HERE = Path(__file__).resolve().parent
sys.path.insert(0, str(HERE.parent / 'integration-continuation-015'))
import continuation
import readonly_catalog
import integration_probe

_probe_command = integration_probe.command


def arguments(argv, stage):
    continuation.require(stage in ('linearize-plan', 'linearize-accept'),
                         'only integration stages are admitted')
    if argv[:1] == ['--inside']:
        return list(argv)
    return continuation.provider_args(argv, stage)


def configured_provider():
    module = continuation.repair.original
    # The original provider's private re-exec must return here, not bypass the
    # immutable mount seam. All original profile and pin checks remain intact.
    module.HERE = HERE
    module.dispatch.qualified.enter_view = readonly_catalog.load().enter_view
    return module


def probe_command(provider, repo, thread):
    del provider
    return _probe_command(HERE / 'input_provider', repo, thread)


def main():
    continuation.require(os.environ.get('WORK_LEAF_BENCH_CATALOG_IMMUTABLE') == '1',
                         'explicit immutable benchmark view opt-in required')
    argv = arguments(sys.argv[1:], os.environ.get('WORK_LEAF_OBSERVER_ROLE'))
    return configured_provider().provider_main(argv)
