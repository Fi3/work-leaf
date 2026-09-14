# Non-generating dry-run custody

Checked: 2026-09-14T23:53:04+00:00.

The prepared first row was used with --dry-run before generation admission.
The retained driver creates a checkout/report even in dry-run mode, then removes
its own fresh checkout. Its report records workflow_result=skipped,
measurement_status=not-started, observer not initialized, no agent CLI version,
zero commits and no feature/review/integration. No provider was launched.

That setup call occupied the row's future artifact/runtime directories. They
are preserved by moving the exact report directory to ./dry-run-artifacts and
the empty runtime parent to
/home/user/.codex/wl-author-joint-confirmation.AyyYIG/dry-run-qualification.
No report content, manifest, schedule or admitted observation was rewritten.
The original report paths remain historical, with this relocation map. This is
operator setup cleanup, not a replaced benchmark or a changed scientific count.
Future dry runs require their own preflight output locations from the start.
