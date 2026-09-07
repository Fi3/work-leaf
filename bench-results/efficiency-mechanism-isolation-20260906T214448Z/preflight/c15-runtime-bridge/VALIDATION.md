# C15 bridge qualification

The bridge's provider-free automatic suite passes 21 cases; one separately
gated frozen-project Cargo qualification passes its single retained execution.
This is not a complete agent-facing readiness or benchmark-admission report.
No provider call, shared patch promotion, ACK, or token observation belongs to
these tests. The Rust owner retains the real lock/enrollment/framing/delivery
and required Rust gates; the root agent owns real-provider admission.

## Exact source cut

| File | SHA-256 |
| --- | --- |
| `bridge.py` | `6af6c3f24f96a7b95514a0db702a775b895a8d89f5dc8a2db9aa1329db9d4d26` |
| `test_bridge.py` | `2a604d681b5bde70ad76b077b556a7577af7995524daff3887ccc4e0349c3f69` |
| `CONTRACT.md` | `4a4c109da4624ce8238472961e455c95ad50178b4ecb02e0fc25aae6092c56c0` |

The unchanged prerequisite sources remain:

```text
executor.py       16e3238e1bd647c9d4780db97c554ddf9579226c913995e2e18873bf7fcb24ae
materialize.py    87546793b76bc7324af55479be47a1e34abbeffb8f2787547d021575a51efab2
project_inputs.py b2f2905b3ffe9a0e356577d78eae2249b6c16a6919b0a359d731ab2e2ed0f862
live_selection.py 0614db9a5c874cd5f623997ce3433edbd2bc0ff3585c23913dd9ede9705d3a55
```

The complete architecture/operator policy, initial/detailed C15 designs, runtime
boundary review, and each predecessor's complete source, contract/qualification,
independent review and available closeout were inspected before implementation.
Only this new private bridge directory is owned by this implementing agent.
No old helper, source artifact, committed test, normal provider/backend, or public
API is edited. The Rust owner coordinates the resulting architecture paragraph.

## Test-first evidence

Initial command before `bridge.py` existed:

```sh
python -B -m unittest discover \
  -s bench-results/efficiency-mechanism-isolation-20260906T214448Z/preflight/c15-runtime-bridge \
  -p test_bridge.py -v
```

Observed RED `89358e`: `ModuleNotFoundError: No module named 'bridge'`.
The first implementation run additionally identified a fixture helper named
`test_request` that unittest treated as a case and the qualified Cargo-produced
driver's ordinary two-name hardlink. The fixture helper is not a test; trusted
pinned executable hardlinks remain admitted, while independent source/receipt/
capsule requirements remain strict. Neither correction modifies a committed test.

Focused subsequent pre-fix REDs:

| Receipt | Reproduced missing rule |
| --- | --- |
| `313b37` | Executing helper identity must match the exact captured source, not only a later reread. |
| `7ac1d9` | A public capsule cannot contain an undeclared empty directory. |
| `fcc131` | A bounded Git `TimeoutExpired` must retain a failed result. |
| `b24174` | Failed operations still require endpoint-source verification. |
| `ee2315` | An executor cleanup exception cannot prove a closed process. |
| `3b4f84` | Typed owner IDs must match the agreed runtime alphabet/canonical AgentId. |
| `3590c9` | Source drift and the original operational failure must both remain visible. |
| `4e113c` | The approved live `test_paths` transport must work without model-computed ranges or duplicated text. |
| `cb8268` | A dangling internal result symlink must reject before private work or an attempt; every reserved attempt/result/execution path rejects existing entries and symlinks. |

The same complete command passes **21 cases, one explicitly gated case skipped**,
12.075 seconds, receipt `4e7784` (`22` discovered tests). The ordinary suite includes
actual Git, qualified private patch-driver and bubblewrap executions, not provider
or fake-command substitutions for the successful behavioral/isolation paths.

## Demonstrated behavior

- Exact selected source survives a later normal accepted live commit without
  rebasing the held test. The original live source remains unchanged by preview.
- The private source is actually mounted at the original absolute project cwd;
  `snapshot.shared.root` still identifies the independent accepted clone.
- Both simultaneous hidden index flags and full effective overlay bytes are
  retained. The private installation equals the selected effective file census;
  the older installer's single-flag argument is explicitly a projection, not a
  replacement live census or a false unobserved flag.
- A held test-only shell file fails against `value.txt = old`, returning actual
  exit 1. Qualification-only implementation changes the separate private value
  to `new`; the identical command and complete test file return exit 0. The
  shared value and test absence remain unchanged. This generic behavioral
  fixture is not a frozen-project Cargo or timing-effect claim.
- Missing declared paths, invalid path metadata, owner changes, unsupported
  runtime implementation mode and changed dependency pins fail closed. A failed
  preparation never masquerades as an executed test. Duplicate operations and
  output/attempt paths cannot execute again.
- Actual cancel-file polling stops the confined execution and reports cancellation.
  A synthetic executor exception tests conservative `closed: false`; it does
  not claim an actual unclosed process was created during qualification.
- Success/failure endpoint checks preserve both original failure and independent
  source drift. A Git timeout publishes its failure without a command result.
- The exact-source Python bootstrap runs `validate` under `-I -B` with an empty
  environment. Its source pin agrees with the compiled input; duplicate JSON
  keys and an already-published output are rejected before another attempt.

Declared path images are host-derived full-file evidence. They do not mechanically
identify assertions, prove test-only semantics, or resolve a co-located production
edit. These semantic limits remain explicit in every bridge test result. Live
transport requires only natural test purpose/path declarations and the held body,
not manual UTF-8 offsets or a second copy of the test.

## Separate frozen-project qualification and limits

`test_actual_frozen_project_cargo_preview` requires a fresh, empty, explicitly
supplied `C15_BRIDGE_PROJECT_QUALIFICATION_ROOT`. It retains every output there and
does not run in ordinary test discovery. Its prescribed scenario clones frozen
base `c92a0b7060a36eac6db2d869b85e589a7a9480f9`, verifies tree
`acb6583fbb0e25347f70ee44cbdacafd6df95e37`, applies the exact retained public overlay
only to the new fixture, and submits a new provider-free `AgentId` test through
the bridge. The exact offline/locked Cargo target is expected initially GREEN;
that result qualifies project execution, not missing-behavior RED or a saving.
The earlier project's Cargo execution is not substituted for this new bridge gate.

The explicit execution in `/tmp/c15-bridge-project.TaPaRh` passes one test in
7.961 seconds (tool receipts `d54b3a`, `4aa9c0`). `PROJECT-QUALIFICATION.json`
binds the retained capture, selected bundle receipt, test result, configuration
and complete execution result. The actual private Cargo command exits 0 with
closed pipes/processes; the live census is byte/metadata-equal before and after
and the held test remains absent from the live root. All 65 returned source pins
match at the operation endpoint and a separate post-run rehash. Its command cwd
and writable private-source mount establish the original-live-root view, while
the predecessor's accepted-root fields and unqualified view flag remain intact.
There is exactly one retained project qualification, no rerun or provider call.

The operation budget is cooperative between 30-second-bounded Git children and
during the executor; it is not a hard kernel-I/O or filesystem quota. Reachable
Git history and postcreation bundle admission retain their original scope. The
installed public toolchain and system distribution are trusted inputs; explicit
Cargo/rustc/capsule pins do not prove all-command dependency completeness or host
environment equivalence. Private HOME, configuration, cache/build state, namespace
setup, snapshot and result traffic are part of the private-feedback package.

The bridge performs constant byte passes per proposal and keyed/sorted joins, not
all-history pairwise work. Repeated source validation is O(P × B); Git history
operations are additional. The unchanged executor's O(M²) mount admission is
bounded to at most 32 additional mounts and remains explicitly flagged. No
general peak-memory, hostile concurrent host, or continuous live-immutability
claim follows. Evidence/execution roots are retained on failure; no broad cleanup,
retry, provider call, or new normal-control observation is authorized here.
