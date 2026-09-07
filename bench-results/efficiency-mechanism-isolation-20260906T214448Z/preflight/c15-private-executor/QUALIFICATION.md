# Private C15 executor qualification

Scope: a standalone provider-free Linux executor prerequisite, not the C15 protocol,
runtime integration, a new control, a benchmark or an agent-facing readiness claim.
The existing C15 designs remain unchanged. Independent review is pending.

## Actual result and boundary

All 11 tests passed on Linux `7.1.8-1-cachyos`, bubblewrap `0.11.2`, Python `3.14`,
and Rust/Cargo `1.95.0`. `QUALIFICATION-001.json` retains the exact executed helper
and test identities, test log, executable hashes and all 11 returned execution
receipts. The receipt-producing test wrapper compiled the captured helper/test
bytes directly after hash verification and rechecked both sources afterward;
it only recorded returned results. It did not import cached helper bytecode.

The minimal executor builds an initially empty filesystem view with explicit
read-only system/runtime mounts, an isolated `/proc`, a read-only minimal `/dev`,
and exactly three writable owned trees: private source at the requested project
path, `/build`, and scratch `/tmp`. Other root entries are read-only. `/dev/null`
and analogous devices retain their normal device semantics; this is not a claim
that device I/O writes regular files. The host root, real home, provider state
and global configuration are not automatically mounted. The trusted launcher
alone supplies additional read-only **public** runtime/cache/fixture roots;
this interface must not receive agent-selected mount paths or credentials.

Namespaces are requested explicitly; user namespaces cannot be nested, network
is unshared, capabilities are dropped, and a new session plus die-with-parent
behavior accompanies the private PID namespace. Unsupported setup returns its
actual failure with no unconfined fallback. The executable is hash-checked before
and after invocation, not merely selected by PATH. Mounts use held directory
descriptors; those host descriptors are absent from the executed command.

The helper does not inherit the caller's environment. It supplies only a fixed
PATH, private HOME/TMPDIR/Cargo directories and locale. The credential-surrogate
environment variable was absent and the real-home auth path was absent inside
the view; no credential contents were read, copied or exported by qualification.
This is not a claim that a malicious trusted launcher cannot deliberately mount
sensitive data. Root-owned executable/system mounts and absence of hostile host
mutation remain explicit trust assumptions, not a new hardened host boundary.

## Concrete tests and retained REDs

Initial command, before `executor.py` existed:

```sh
python -B -m unittest discover -s bench-results/efficiency-mechanism-isolation-20260906T214448Z/preflight/c15-private-executor -p test_executor.py -v
```

Observed RED: `ModuleNotFoundError: No module named 'executor'`. The first
implementation's seven execution cases failed closed with
`bwrap: --disable-userns requires --unshare-user`; one source-guard test passed.
Explicit mandatory `--unshare-user` corrected that setup requirement, without
dropping namespace restrictions or executing an unconfined command. Eight tests
then passed.

A later pre-spawn external-hardlink regression observed RED: the helper reached
mocked `Popen` instead of rejecting a private-file inode linked to the shared
sentinel. The owned-tree inventory verifies that every regular-file hardlink is
contained within the three writable trees; FIFOs/other special files are rejected
and symlinks are not followed. In-tree links are not rejected solely because
their link count exceeds one. The same command above then passed all 11 tests,
including the following actual cases:

| Case | Observed result |
| --- | --- |
| Source/build/scratch writes | All succeed; the original project absolute cwd names the private source view, not the host shared directory. |
| Absolute shared/evidence/config surrogate writes | Rejected; all three retained sentinel bodies remain `UNCHANGED`. |
| Symlink and spawned-child writes to shared surrogate | Rejected, with the child returning nonzero. |
| Writes to read-only system/device directories | Rejected; no regular file is created there. |
| Host descriptors, inherited secret-surrogate environment and home auth path | Absent from the executed program. |
| Network connection and nested user namespace | Unavailable in the confined process. |
| Detached child on timeout, cancellation and ordinary parent return | No matching tagged process survives; output/process boundary closes. |
| Bad executable hash, missing binary, symlinked root, overlapping roots, external hardlink and FIFO | Rejected before process launch. |
| Output exceeds the explicit limit | Process is stopped with `output_limit`; the retained prefix is bounded and not reported as an unqualified success. |

Both timeout and cancellation fixtures spawn a child in a new session. The tests
check the host `/proc` census for the unique generated child tag, rather than
treating pipe closure alone as proof that that detached child disappeared.
These are demonstrated supported lifetime cases, not a proof against a kernel
vulnerability or hostile privileged host actor.

## Offline Rust and read-only cache feasibility

The generated fixture has no external dependencies. A pinned installed toolchain
is read-only at `/toolchain`; the real user Cargo home is not exposed. The same
command runs twice with `--offline`, the same named test and the exact test-body
bytes. First, `value()` returns 1 and the assertion expecting 2 fails with status
101. Only the private implementation return value becomes 2; the unchanged test
then passes with status 0. Its actual command, stdout/stderr and status are in
the JSON receipts. This is a real test execution, not a mocked shell response.

The private view uses the same absolute cwd as the host project surrogate, while
host files remain unchanged. A separate read-only cache surrogate is readable,
rejects writes, and keeps its exact content digest. That demonstrates the mount
mechanism, **not** a dependency-resolved build of the current project, an actual
Cargo registry-cache replay, a network-dependent test, or historical compiler
equivalence. Extra toolchain/configuration/path dependencies need explicitly
declared read-only inputs before a project can be admitted. No `.git` metadata,
shared index or source materialization equivalence is asserted by this executor.

## Reuse decision and remaining integration boundary

The demonstrated Linux executor is a concrete candidate for the private C15
preview; a cwd-only host shell is not required. No generic provider executor,
new provider call or public API is necessary for this prerequisite. The new
protocol still needs separate approval and implementation for exact snapshot
materialization, proposal/test identity, patch preparation, truthful result
delivery and unchanged final shared apply/ACK/review behavior.

This helper never removes execution directories or evidence. The caller owns
their creation and eventual cleanup. On uncertain closure or replaced source
identity it raises and requires preservation, not recursive cleanup or retry.
The tests remove only their own disposable fixture directories after the tested
process boundary closes. A hostile concurrent host writer is outside the present
qualification; descriptor mounts and endpoint checks are not claimed as full
continuous immutable-source attestation.

The owned-tree walk is bounded to 200,000 entries and depth 64, O(F) metadata work
with keyed inode counts. Mount validation is pairwise in a fixed at-most-32
additional mounts: O(M²), explicitly bounded independently of conversation size.
Output collection is linear in captured/read bytes and stops at its explicit
limit. No archive-count × native-transcript scan exists. These bounds and private
HOME/build/cache behavior are non-target environment qualifications to carry into
any future treatment, not facts to conceal in an equivalence claim.

No Rust or agent-facing runtime source was changed. No real-agent workflow is
affected by this standalone provider-free qualification; actual C15 runtime and
real-agent verification remain separate future gates. No costs, percentages or
new causal observations were produced.

## Source identities

- `executor.py`: `16e3238e1bd647c9d4780db97c554ddf9579226c913995e2e18873bf7fcb24ae`.
- `test_executor.py`: `7972e12652552252a29d1f87ca840b3fe17f484d424ec97c0ecdcdcad621f1f9`.
- `/usr/bin/bwrap`: `eabbccb0f7f755b96d30834026a9b5d941c606400d097d87c1ff16622edaf68c`.
- Existing `DESIGN-TEST-FIRST-ISOLATION.md`: `99d82be2c35c24dff89bded7eb786a9a18bf955ede11c0e7f68133ba6ffe2493`.
- Existing `DESIGN-TEST-FIRST-ISOLATION-DETAIL.md`: `9e4911e63cbd5fac54475552dc883d15c4448eaf149a4e8e2a134df8328240ff`.
