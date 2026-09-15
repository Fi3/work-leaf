# BUG008 — observer guard path across the private namespace

Discovered 2026-09-15T00:04:00+00:00, existing P09 obligation.

All three author-joint-confirmation attempts start at 23:57:00 and finish by
23:57:03. Each first host child exits 86 with the same error: original observer
provider is not executable. Each public stdout is empty, native thread is null,
accepted commits are empty and retained rollout extraction finds no thread.
Their phase result, manifest, source-endpoint check and artifacts remain immutable.

The wrapper writes its outer-entry receipt, then enters its private user/mount
namespace. The recorded next provider is /proc/<artifact-guard-pid>/fd/5/
observation/proxy-bin/codex. The inside entry fails the identical accessibility
gate, before observer or native launch. The earlier real-route diagnostic used
a normal path and did not cover this full-driver artifact-guard boundary.

Reproduce the namespace boundary without provider generation first. A qualified
private repair must preserve the actual observer executable/configuration,
artifact target inode, model input, custody and all non-target mechanisms.
Normal WL and original admitted sources remain untouched. Local bound: twenty
minutes; actual verification, if eligible, receives its own short admission.
This is one previously unaccounted bug, not a scientific task or positive signal.

Evidence: phases/author-joint-confirmation-20260915/PHASE-RESULT.json,
POST-RUN-IDENTITY.json and each row's input-receipts and first host invocation
request/exit/stderr/stdout under the parent mechanism study.
