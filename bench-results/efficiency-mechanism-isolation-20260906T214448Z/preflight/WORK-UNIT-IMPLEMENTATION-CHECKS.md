# Work-unit implementation gates

The v2 control/treatment runtime contains one private work-unit policy factor at three owned
surfaces. `src/agent.rs::PromptPolicy::inject` remains baseline and infallible; the eight existing
Codex/Claude injection sites use private fallible delivery. Known-session followups remain raw.
The `bench-experiments` feature is nondefault, and v1 schemas/continuation evidence stay compatible.

## Automatic verification

- `cargo fmt` passes.
- `cargo clippy --all-targets --all-features -- -D warnings` passes.
- `cargo test --all-targets --all-features --quiet` passes after the diagnostic round-budget fix
  (root verification sessions `59769` and `61093`, exit 0; the latter includes the final trust fixture).
- `cargo test --test bench_work_unit --no-default-features` passes three tests.
- The feature matrix exercises all eight existing provider policy-injection branches, known-session
  raw sends, non-linearizer/reviewer/hidden/linearizer identities, both read permissions, instruction
  translations, Unicode and matching cue text inside copied data. V2 admission was RED before its
  implementation; invalid manifests fail before provider generation. Evidence-write errors and
  malformed/overlapping spans fail closed. V1 evidence has no extra policy records.
- The independent reference harness compiles the verbatim pre-patch renderer and instruction loader
  from `518673c0b54943cde34cd1bd8c1376a5d498d460` beside the resulting library. Seventy-two entire-prompt
  comparisons pass across default and feature builds. `WORK-UNIT-POLICY-IDENTITY.json` retains
  old/new source, library, fixture and complete-prompt hashes; this is not merely new-control versus
  new-baseline comparison.
- The new allocation helper has four passing tests, including all 324 allowed allocations,
  exact six-per-arm counts, three-per-wave mixing, and durable claim-before-draw/no-redraw behavior.
  The v2 supervisor's four initial tests pass; its future trust adapter has a separate gate.

## Real subscription verification

`WORK-UNIT-SMOKE-ADMISSION.json` retains the first two failed diagnostic attempts. The copied
three-round fixture limit prevented execution of the second check. The automatic regression
exercises actual `CommandChat` and shell checks, fails before the fixture fix and passes afterward.
No normal WL limit or workflow implementation is altered by that fixture correction.

The two separately admitted corrected diagnostics in `WORK-UNIT-SMOKE-ADMISSION-V2.json` pass:

| Diagnostic | Provider turns | Sessions | Accepted edits / successful checks | Duration |
| --- | ---: | ---: | ---: | ---: |
| Control | 5 | 1 | 2 / 2 | 15.92 seconds |
| Buildable incremental policy | 5 | 1 | 2 / 2 | 14.48 seconds |

Each settles its actually forwarded final interrupt, matching typed RPC acknowledgment and terminal
turn before stopping the observer. Both have closed capture metadata and valid pre-spawn project
inventories. The independent v2 delivery audit verifies 5/5 actual accepted prompt boundaries and
two genuine delivered ACKs per corrected diagnostic. Control is byte-identical. Treatment changes
A by 184 bytes and each of two C exposures by 219 bytes; command-result text and validation
cardinality remain identical. These fixtures contain no instruction files, so B is covered by the
provider-free matrix rather than claimed as real-smoke exposure.

All generation uses the existing ChatGPT subscription, Codex CLI 0.153.4, GPT-5.5 and `xhigh`.
No API authentication, copied credentials or extra diagnostic provider calls are used to obtain
these outcomes. The explicitly prescribed two-increment scaffolding verifies plumbing, not a
natural work-unit choice, causal mechanism effect or token reduction. Diagnostic usage is excluded
from every benchmark observation.

## Review and remaining admission scope

Root source review finds no introduced public API break, provider-specific policy ownership,
non-target prompt transformation or default-byte drift. Renderer span assembly and application
are linear in prompt/replacement bytes and span count. The exact allocation test is bounded at
324 allocations and twelve workflows. The original supervisor's O(waves × frozen evidence bytes)
metadata verification remains a documented potentially quadratic unrestricted-schedule cost;
this phase is fixed at twelve workflows.

The observer-inventory component and future-only trust classifier have their own review/checks
and real verification gate. The corrected work-unit handoffs used the observer hash in their
admission record; they are not falsely relabeled as executions of a later observer build.
`WORK-UNIT-OBSERVER-VERIFICATION.md` records the final-build real scenario and preserves the earlier
fixtures' post-capture executable-identity and incomplete-usage findings.
No benchmark admission is implied by this report. Source/binary attestation, complete v2 analyzer
review, trust-transition replay and the final phase manifest remain separate launch requirements.

## Prospective trust supervisor verification

The future-only classifier and runner pass these offline checks:

- `python3 -B bench-results/efficiency-mechanism-isolation-20260906T214448Z/test_trust_work_units.py`: 25 tests pass.
- `python3 -B bench-results/efficiency-mechanism-isolation-20260906T214448Z/test_runner_work_units.py`: 4 tests pass.
- Independent review by `bench_feature` closes at classifier SHA256
  `37122d613c96b21fc4cd078ad8dfd7784e0fa60c1f430bb2ff78cf6d82ea88b8` and runner SHA256
  `2f019c297a2e436c54241ad0643d4f105db6ec8bc782b1cac456f3acde0bc98e`.

RED-to-GREEN regressions cover exact owned admission versus unrelated/preexisting configuration
changes, typed RPC and nonempty string session identities, missing forwarding/inventories, actual
rules drift, retained legacy flags with a distinct admission gate, and final evidence tampering.
The native-publication tests exercise missing, partial and after-cutoff attestation: the original
snapshot stays unverified, subsequent admission waits at most 30 seconds, and a later proof has a
separate cutoff. Offline replay rejects workflow starts during that pending interval. A matching
timestamp/size stale Python bytecode fixture verifies execution of exactly the hash-pinned source
bytes. Parsed bookkeeping changes without an exact historical reconstruction path fail the future
gate while retaining their legacy label.

`WORK-UNIT-TRUST-REPLAY.json` records the read-only verification at
`2026-09-07T00:24:49.501088+00:00` of the excluded actual two-session diagnostic in
`work-units-trust`. Its author read-only start precedes the prescribed full-access linearizer
start; typed string RPC ID `4`, native permissions, eight capture-prefix source identities and
unchanged project inventory `3a9d4059f7d14d645028f39c877f9f5f09cb65720fef23e573922cd2aa35671d`
pass both transition proof and independent prefix replay. The matching external final global
file was available at that check. The replay itself makes zero provider calls and zero global
configuration writes. It is not a fabricated whole-phase historical replay or a causal observation.

Full future phase audit additionally requires matching final external global bytes and reconstructs
every historical parsed hash in memory; no configuration contents, prompt bodies or credentials are
copied into the proof report. Snapshot equality is not continuous immutability. The bounded
historical replay has an O(H × P) metadata component for H snapshots and P proofs, potentially
quadratic in an unrestricted study; the prospective phase fixes P at twelve.
