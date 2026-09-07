# Source-representation mechanism experiment

Status: prospective protocol, not provider admission. The phase requires a separately drawn fixed
allocation, a frozen manifest, completed implementation/tests/review and real subscription checks.
The future-only accounting adapter and its exact source identity must pass the admission requirements
below before any study generation. No observation is authorized by an unfinished prelaunch gate.

## Hypothesis and factor

The directional hypothesis is that eager inline delivery of the exact untracked project snapshots
which WL ordinarily bundles increases whole-workflow raw input plus output. The proposed mechanism
is full requested source entering the conversation, then being charged again as retained input on
later responses, instead of the agent retrieving selected bundle evidence on demand. Fewer retrieval
responses or other behavioral changes can counterbalance that input increase; the net direction is
an empirical question.

`DESIGN-UNTRACKED-READ-INLINE.md` defines the single factor. Conditions are `control` and
`untracked-read-inline`, using one feature-enabled binary and private v3 manifests. All baseline
bundle creation runs once in both conditions. Treatment selects the full-inline candidate only for
a successfully bundled untracked-project component. The complete original and alternative prompts
are recorded in both conditions, with a selected-candidate identity. Other bytes and workflow
mechanisms are unchanged. Untracked means absent from the existing agent/path snapshot tracker,
including paths cleared after an accepted authored edit; it does not mean first-ever read.

No treatment is applied to tracked changed/unchanged reads, explicit bundle-file reads, small inline
reads, bundle-write failures, automatic refreshes, command output, work-unit policy or validation.
No padding, forced retrieval, additional task, artificial truncation or experiment label is sent to
the agent. Large input/context failures and compaction are retained possible treatment outcomes.

This distinct hypothesis follows source/exposure inspection, not selection of a favorable comparison
from earlier phases. The original screen's no-candidate result and the unsuccessful work-unit test
remain retained. They are not confirmation observations and are not repeated or pooled here.

## Fixed population, allocation and stopping

The confirmation consists of exactly twelve fresh complete WL workflows, six per condition. There
are two blocks of six slots, each split into two concurrent waves of three. In each block draw
uniformly from the eighteen allocations with three labels of each condition and both conditions
present in each wave. Independent block draws produce 324 possible joint allocations. The draw
uses a create-new claim and cannot be repeated. Launch order within each wave is separately drawn.

All twelve identities, assignments, block/wave/slot positions and launch order are frozen before
generation. Three complete workflows run concurrently in separate checkouts/artifact directories.
Failures, missing measurements, unfinished attempts and withheld slots remain in the record. No
replacement, redraw, favorable stopping or sample-size extension is allowed. An incomplete schedule
does not support the complete twelve-workflow randomization test.

Do not inspect interim condition contrasts or token rankings. Operational checks may inspect
source/configuration integrity, delivery, process activity, failures and required coverage. The
primary analysis runs only after all admitted identities have terminal retained outcomes and the
phase closes. Infrastructure integrity failure stops new admissions while active observations
reach their bounded outcomes.

The single primary directional test receives the remaining alpha 0.025 of the investigation's
declared 0.05 family-wise budget. The work-unit phase already consumes its allocated 0.025; failure
does not refund it. No additional confirmatory endpoint, subgroup, replication or candidate receives
an unallocated significance threshold. Exploratory findings retain that label.

## Frozen task and execution

The task remains the original three-feature benchmark, base
`c92a0b7060a36eac6db2d869b85e589a7a9480f9`, task-list SHA-256
`45bee25a4b929182d36612fc5a159597e7770f25dba9c95760713a401d45598a`.
The normal concurrent driver performs implementation, review, fixes, linearization and final
format/clippy/test checks. Canonical feature scoring is offline. The user's no-quality-loss
assumption remains the interpretation boundary; no scorer result is erased or used for inclusion.

Generation uses the existing ChatGPT subscription, Codex CLI 0.153.4, GPT-5.5 and xhigh. No API-key
authentication, API credits, copied credentials, provider/model switch or Direct sequential run
belongs to this phase. Existing provider-isolation rules prevent recursive unmeasured generation.

The observer uses the same raw-response metadata option and maximum 1,000 ms pre-forward usage
grace, with immediate release on resumed output (`forward`). Stage/busy/idle/outer/termination-grace
limits remain 7,200/1,800/300/86,400/20 seconds. No added generation wait belongs to the factor.
The existing bounded project-layer inventories and future-only own-workflow trust attestation
policy apply unchanged, with original hashes/classifications and independent final replay retained.
Unknown drift stops new admissions; pending native permission publication follows the existing
fixed 30-second rule. No global configuration contents or credentials are archived.

## Primary accounting and test

Raw tokens mean input plus output, including cached input. Reasoning is a subset of output, never
added twice. Scope includes every implementation/review/fix/linearization/title/workflow generation,
retry, failure and compaction. Retain the original observer ledger separately from any proved
response-identity scope correction. No completed response is counted twice and missing usage is
never zero. Cached/uncached/input/output/reasoning components are separately reported.

A separately versioned future-only accounting adapter is an admission requirement. It must preserve
strict raw-response identity and input/output arithmetic, source hashes, cumulative validation and
the existing conditional tail-proof assumptions. A nonadditive `tokenUsage.last` notification is
retained as unusable notification metadata, not declared a completed or zero-cost response. It
cannot establish exact usage grace or recover a tail. Any exception to the predecessor's fatal
parsing rule requires a separately retained generic source/lifecycle proof and regression tests;
no observed run ID, filename, line number or numeric token value is an exception key.

Compaction response usage can enter the new phase's observed scope only through an explicit exact
native/raw response identity, valid usage, captured thread/turn and deduplication against the
recorded ledger. Same-time adjacency alone is insufficient. Retain the old total, corrected scope,
identity inventory and proof separately. Unproved scope, malformed completed usage, ambiguous
identity or unsupported missing-response cardinality stays unknown/ineligible; no parser recovery
may conceal it. Historical and work-unit reports are never overwritten or retotalled.

A saved unresolved pre-forward grace decision may be separately marked recovered only by exact
same-turn raw/native completed-response identity, matching additive fresh cumulative last usage
after the directive, and a captured interrupted terminal with no intervening later generation,
item or unknown same-thread boundary. Retain the original grace decision and physical source
locators. Compaction/unusable-last turns cannot use this recovery. Other unsupported tails retain
null upper bounds; no runtime wait or interruption rule is altered by offline reconciliation.

Each defensibly inventoried missing response retains the existing conditional ceiling of
1,178,000 raw tokens (1,050,000 input plus 128,000 output). This is not a measured maximum or proof
of hidden-call completeness. No midpoint substitution, observed-maximum bound or reconstructed
prompt-token estimate replaces missing generation usage. Admission requires automated and retained
completed-source replay checks for both ordinary and compaction paths under the new adapter.

For all twelve workflows report the accounting interval for mean treatment minus control. Use the
exact one-sided blocked randomization test over the 324 actual permitted allocations. With interval
outcomes, bound each assignment's contrast minus the observed assignment's contrast using the same
per-workflow intervals before counting definitely/possibly greater-or-equal assignments. The
resulting conservative p-value envelope is not an exact midpoint test or sampling confidence
interval. Integer weights preserve exact ties; enumeration costs O(A*n) for A=324 and n=12.

The primary passes only if the whole observed accounting contrast is positive and the conservative
p-value upper bound is at most 0.025, with complete allocation/source/configuration/delivery gates.
No unlaunched or unknown row is dropped. A failed primary does not prove zero effect and cannot be
replaced after inspection by a selected-input, response-count or pre-compaction endpoint.

## Required mechanism evidence

Verify each renderer-owned range and exact candidate reconstruction, byte-for-byte identity outside
the factor, and symmetric evidence construction. Require bidirectional coverage between trace rows
and actual supported read-response requests, distinguishing prepared, accepted, rejected and unsent
rows. Accepted delivery requires a typed matching success reply and exact thread/turn identity;
an error or ambiguous identity is not exposure. Non-read policies/ACKs/command results remain baseline.

Join selected delivered input to native user-item identity only where the exact evidence supports it.
Follow its measured per-response input charges and actual bundle-reading tool calls/results. Preserve
unlinked identities, output truncation, complete versus selected reads, and source changes. A repeated
item charge is not repeated execution, candidate byte size is not a token count, and every later
response is not automatically caused by one earlier read.

Report all observed response identities, first-exposure/pre-exposure diagnostics, eligible-read and
bundle-retrieval behavior, compaction, failures and retained-input charges. No additional significance
test or favorable episode selection is implied. Source-semantic case selection must precede cost
ranking; if examining all cases, retain the complete census. Mechanistic review and exact accounting
must connect delivered full source to observed context use rather than rely on a positive total alone.

Separate checkout ownership does not eliminate shared CPU/I/O/subscription-capacity interference.
Record resource incidents. Randomization concerns the specified mixed-wave assignment regime; it
does not separate isolated direct effects from spillovers or identify a unique historical share.

## Admission and reporting

New runtime and helper tests run RED before implementation. Existing committed tests remain intact.
Required format, all-target/all-feature clippy with warnings denied, all-target/all-feature tests,
default/control identity, v1/v2 compatibility and independent architecture/documentation/source review
must pass. A bounded real subscription-backed diagnostic in each active arm verifies actual read
delivery and ordinary continuation. Diagnostics are retained separately and are not observations.

Freeze actual runtime/observer/provider binaries, source inputs, lockfiles, clean driver snapshot,
wrapper, scorer, protocol/design, analysis/accounting/allocation/supervisor helpers and manifests
before admission. Fail closed on identity mismatch. Every outcome and original accounting diagnostic
remains visible, including a failed primary. Report exact source symbols, treatment delivery,
action/input-charge chain, net effect interval and assumptions. A confirmed component effect is not
automatically the entire historical approximately 50% WL saving or a unique percentage attribution.
