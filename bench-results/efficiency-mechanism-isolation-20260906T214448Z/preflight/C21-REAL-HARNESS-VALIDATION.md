# C21 bounded real-agent harness qualification

Status: automatic tests pass; the real-provider test is ignored and **unlaunched**.
This record is not admission, real-agent verification, a token result or permission
for a control/replacement. Independent source review is closed with no remaining finding.

Source: [`tests/bench_review_evidence_subscription_smoke.rs`](../../../tests/bench_review_evidence_subscription_smoke.rs),
SHA-256 `b94bcae247d704ecc3c913522848d2597a1a1dcdde132c4da9f66d10f4cda345`.
The prospective contract is [the C21 design](../DESIGN-REVIEW-EVIDENCE-ON-DEMAND.md).
Existing committed fixtures, runtime and frozen helpers are not edited by this task.

## Declared scenario

The real ignored test drives `CommandChat::launch_prepared_agent_streaming` and the
ordinary `handle_line("review")` path through the actual configured `CodexBackend`.
It does not substitute a fake reviewer, fake read route or post-adapter prompt.

1. An author submits one complete `@work-leaf edit` changing the supplied fixture's
   `VALUE` from 0 to 1. The host performs its normal apply/commit/ACK flow.
2. The author acknowledges that supplied initial text was used and no native tool/check
   was run, then emits `@work-leaf done`. This exact public agent-role evidence is
   retained in the archive, separately from the same instruction in the launch input.
3. The real reviewer reads the entire issued opaque archive with one native
   `exec_command` call: `cat -- '<issued path>'`, `login=false`,
   `max_output_tokens=20000`. It then requests exactly one normal locked check,
   `@work-leaf locks run fixture.rs -- sh ./check-fixture.sh`. The preexisting script
   checks the exact requested value and emits `C21_CHECK_OK`; it is not model-created
   verification. No other native tool, agent, poll, project read or command is permitted
   by this diagnostic. These bounded instructions are in the fixture's feature label
   before ordinary review rendering and archive selection.
4. After the successful actual check result, the reviewer returns `NO_FINDINGS` if the
   requested edit and archived evidence are present. The ordinary review reports one
   resolved round. A failure or unexpected action remains a failed diagnostic.

Expected generation is four outer turns across two threads. An independent six-call
ceiling is a backstop; the exact planned handoff/reply contract rejects extra calls
earlier. A 118-second watchdog bounds the whole workflow/settling interval. Native work
inside one outer turn is not counted as an additional outer turn or silently assumed
absent. The final accepted author and reviewer turns must each have a terminal record;
matching interrupts must be forwarded and acknowledged. The capture is explicitly
stopped before strict closed-frame verification.

## Guard and evidence boundaries

Pre-provider guards require schema 5 condition `review-evidence-native`, matching
nonempty run IDs and primary marker, observer condition `work-leaf`, declared
`gpt-5.5`/`xhigh`, raw response usage `1`, grace `1000`, resumed-output policy `forward`
and project-layer inventory `1`. The test requires its explicit real-smoke marker.
API key/base/access-token overrides, inherited parent invocation and Codex trace
overrides are rejected, including empty-but-present values. It does not copy credentials
or establish a new login. Root/operator qualification owns the exact subscription
launcher, executable hashes and effective native model/effort proof.

The fixture requires a canonical empty project checkout and the separately admitted
opaque root outside it. Schema-5 storage validation owns ordinary bundle separation
and publication. The verifier checks all six trace records: activation, unchanged
author policy, unchanged ACK, the selected owned review-context replacement, unchanged
reviewer policy, and unchanged command result. The review-context record and reviewer
policy refer to two stages of the same launch, not two provider calls.

Archive bytes equal the original source-owned context span exactly. The typed manifest,
read-only regular payload, source/reviewer identities, prefix/suffix preservation and
baseline policy injection are checked. Fixed ordered trace sites join their planned
accepted inputs; accepted `(thread, turn)` and user-item identities are consume-once,
so repeated text is not required to be globally unique. Original/forwarded turn-start
payloads and typed public user input text agree.

The public capture permits one `commandExecution` start/completion pair on the first
reviewer turn, an exact full archive output and a single exact `cat` read action. It
rejects extra tools or shell suffixes, including a same-output extra command. Supported
public wrappers are the literal command or plain `bash -c` with `/usr/bin/bash`,
`/bin/bash` or `bash`; other wrappers are unsupported, not accepted by guessing.
Root's separately pinned collector must still verify exact native function-call/output
identities, actual model/effort, whole source hashes and accounting scope. The public
fixture alone does not establish those separate native/provenance facts.

The test prints whether the actual scenario returned successfully **before** postcapture
assertions. Terminal-settling status is printed separately, and capture-close failure
retains the observer command's stdout/stderr before a later settling assertion. A later
verifier failure cannot replace the original observation or justify a provider retry.
Raw captures, archive and root-owned process receipts remain retained.

## Automatic checks and limits

- RED: the initial guard accepted a missing primary marker and the initial budget
  accepted an unplanned agent. Both new automatic tests failed before implementation.
- RED: the initial capture verifier accepted a missing final author terminal. The
  full-read/two-thread/typed-identity regression failed before implementation.
- RED: a public command containing `cat ...; true` passed with identical output and
  read-action metadata. The exact command-wrapper assertion then made it fail closed.
- RED: a generic capture-close failure discarded the operator's stop diagnostic. Its
  new regression failed before the diagnostic-preserving formatter was implemented.
- GREEN: the new target reports four automatic tests passed and one real test ignored.
  Tests also reject missing/wrong settings, API inheritance, wrong/extra handoffs,
  mismatched archive bytes, extra native tools, duplicate accepted turns, mixed
  result/error replies and incomplete final closed JSONL frames.
- `cargo fmt`/`cargo fmt --check`, `cargo clippy --all-targets --all-features -- -D warnings`
  and `cargo test --all-targets --all-features` pass, including the final source hash's
  whole-suite recheck. Independent review reran all four automatic tests and closed
  the sole failure-observability finding; no remaining introduced blocker was reported.

Automatic checks validate the harness, not real native archive accessibility. Root must
separately admit the ignored `real_subscription_review_evidence_handoffs` test after the
current screen's complete outcomes/audits and then retain every actual outcome. No real
agent scenario was launched during this implementation. Agent-facing C21 readiness
remains conditional on that real scenario and independent source/native/accounting gates.
