# Private preview selection with declared derived outputs

Status: prospective source design, not an amendment to `test-first-01`, an
executable qualification, or permission to repeat any admitted identity. The
three current workflows keep their frozen v1 selector/configuration and every
original outcome. No live checkout, build output, provider setting or ordinary
WL behavior is changed by this document.

## Observed boundary and intended factor

The v1 `preflight/c15-live-selection/live_selection.py::census` rejects every
`git ls-files --others -z` entry. That includes ignored generated files as well
as genuinely untracked source. Current workflow003/user-3 preview2 retains the
actual `untracked or ignored input unsupported` preparation failure; its source
receipt is SHA `fb826c1f1294f85d7e82e93bc9a6eb4a4e539bb6fc0cb6cb5f67408e67292c03`.
A later read-only directory sample at10:16UTC returns only ignored `target/`.
That later sample does not establish the exact failed-capture population.

The approved C15 factor already includes a private, cold build environment and
extra feedback transport. It does not require copying ordinary build caches,
deleting normal outputs, moving ordinary checks or declaring arbitrary ignored
files irrelevant. A prospective selector can distinguish explicitly declared
derived-output roots from accepted source without changing those normal paths.
This is a different qualified source contract; the original failure remains a
failure, not a retrospectively corrected preview or a successful exposure.

## Explicit source contract

An independently pinned bridge configuration v2 supplies an exact list of
project-relative derived-output directory roots. There is no default inferred
from `.gitignore`, a model reply, an error string, or a familiar filename.
The operator must establish each root from the frozen task's actual build
configuration and ordinary command environment before a future admission.
For a Cargo project, the tool's declared target directory and effective target
override are evidence; a string named `target` by itself is not a general rule.
No model prompt or extra model call is needed to discover these host facts.

Each declared root must be normalized, nonempty, nonabsolute and strictly
inside the canonical project. Root/ancestor `.git` paths, symlinked ancestors,
aliases, overlapping declarations, regular-file roots and declarations covering
any accepted/staged source or overlay are rejected. Declarations are finite and
bounded before execution. Unknown untracked files, and ignored files outside
the declared roots, retain the ordinary rejection. Being ignored alone is not
proof of being a disposable build input.

Only actual Git-ignored, untracked entries beneath a declared root may be
classified as excluded derived outputs. A nonignored entry inside a declared
root remains unsupported. An ignored rule that changes during source selection
does not authorize a different classification. Git configuration, index and
tracked ignore-rule identities retain the normal before/after source checks.
No excluded body is read, copied into a preview, executed, promoted, deleted or
rewritten. The private command still starts with its existing empty private
build/scratch state, using the same pinned public dependencies and toolchain.

The accepted tree, index equality, exact effective instruction overlay, tracked
file bytes/modes/flags, Git administration and canonical ownership are checked
as before. Each sampled endpoint records the declared exclusions and the actual
classified path inventory separately from accepted-source identity. Generated
output creation/growth/removal between endpoints may be recorded without
pretending those derived bytes were immutable source. Every other drift fails.
Canonical root safety and absence of accepted files under exclusions are checked
at both endpoints, not trusted from one old directory sample.

This contract does not prove that an arbitrary test never reads a generated
file. If the command depends on absent cold-build output, it may fail in the
already-declared private environment; that actual result stays visible. No
successful preparation is a claim of test correctness, behavioral RED or
equivalent live/private build state.

## Implementation ownership and immutable predecessors

Use new sibling private selector and bridge sources and new configuration /
selected-source schema identities. The original selector, bridge, all active
manifests, original receipts and default runtime remain byte-identical. Reuse
the qualified materializer, namespace executor, public dependency capsule and
ordinary Git patch semantics; do not add another execution or provider path.
The existing v6 runtime descriptor can pin the distinct bridge/configuration;
no public provider/controller API or normal prompt, lock, grace or ACK change
is needed for this source-selection contract.

Filtering belongs in the selector's explicit classification logic, not a fake
Git executable, forged command output, alternate index, modified ignore file,
working-tree cleanup or context-dependent monkey patch. Candidate history is
not scanned to choose exclusions. Inventory and path classification use indexed
membership and bounded ancestor checks, not a pairwise file-history search.
Retain the existing explicit path-depth and Git/process/resource qualifications.

## Test-first and actual qualification gates

Before implementation, retain a real-Git reproduction of the original strict
rejection with a normal accepted tree and declared ignored generated output.
New tests then cover the intended opt-in source contract, not a replacement of
the committed v1 rejection tests:

- Default/undeclared output remains rejected; accepted source and overlays are
  unchanged; only declared ignored output is excluded from private materialization.
- Nonignored files, unknown ignored files, staged/dirty/hidden source, tracked
  descendants, aliases, administrative paths and malformed declarations fail.
- Actual output growth during bundle creation is recorded separately while
  accepted-source drift still fails; no output bytes are copied or cleaned up.
- New bridge validation binds the exact declaration, helper and configuration
  identities; unsupported or stale records cannot become completed feedback.
- A real post-check project state must pass source capture and the existing
  confined private command, with all selected source/overlay, output, environment,
  same-author delivery, later ordinary patch/check/DONE and terminal joins.

Provider-free tests and source preparation may proceed independently, but Cargo,
private executor qualification and any additional provider wait until the current
three-workflow wave is terminal. A future real scenario and any modified-only
effect screen require their own fixed count, source freeze and independent
review. There are no automatic retries, replacements, controls, Direct runs or
after-the-fact changes to current token accounting. Failed current preparations
cannot establish the intended complete-workflow effect of a corrected selector.
