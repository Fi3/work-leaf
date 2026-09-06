# Repository verification

## Current pilot verification

`PROTOCOL.md` governs the approved one-direct/one-normal-WL batch followed by a pause. Raw tokens
are primary; the previous dual-metric five-point cutoff is not a launch requirement. Completed
response capture is verified, but cancellation gaps still require explicit inventory and limits.

The single committed report assertion has explicit user approval. Its expected evidence status is
`incomplete_normal_endpoint_measurement`; all 24 mechanism-study tests and all four predecessor
report tests pass. No product runtime file under `src/` is modified. Pilot manifests identify the
actual observer source and binary separately from the clean production source snapshot.

The admitted pair in `../efficiency-raw-token-pilot-20260906T185328Z` is complete and collection is
paused at 2026-09-06 20:58:49 UTC. The exact real-agent scenario is one Direct sequential three-feature
workflow and one normal concurrent WL three-feature workflow, both using the existing ChatGPT
subscription with GPT-5.5 / `xhigh` and CLI 0.153.4. Launch, same-thread resume, reviews, fixes,
linearization, final repository checks, and candidate-startup validation finish successfully in both.
The canonical offline feature scorer passes Direct 3/3 and WL 2/3; WL's left character-selection
status assertion fails. `BATCH-RESULT.md` links the retained reports and full interpretation.

The exact launch command is:

```sh
python3 -B bench-results/efficiency-measurement-gate-20260906/first_batch.py run \
  --batch-root bench-results/efficiency-raw-token-pilot-20260906T185328Z
```

Direct's observer capture is complete (37/37 invocations, zero errors); WL has 36/36 complete
invocations but four missing interruption tails. The frozen WL bound is unbounded, while the
separate post-hoc conditional envelope remains explicit. Real workflow success is not exact
whole-workflow accounting verification.

All 76 provider-free measurement-gate Python tests pass, including 13 first-batch runner and 22
batch-analysis tests. The frozen scorer and analyzer run once against both retained observations.
The final `first_batch.py verify` command exits 2 because the global Codex configuration identity
differs from admission. All other pinned artifacts match. The batch's `POST-RUN-INTEGRITY-AUDIT.md`
retains this unresolved drift; full post-run identity verification is not green. No configuration
or credential restoration, copying, provider retry, or benchmark replacement is part of verification.

Independent final review covers the resulting reports and relevant architecture, observer, and
operator documentation. A separate replay verifies 20 supplemental source pins, 15 rollout hashes,
all six scorer logs, stage sums, raw/secondary arithmetic, both retained outcomes and the paused flag.
Report links resolve and `git diff --check` passes. These checks do not waive the configuration
identity failure or the frozen unbounded WL result.

## Observer-only measurement attempt

The current plan and outcome are in `OBSERVER-PLAN.md`, `OBSERVER-RESULT.md`, and
`NORMAL-SMOKE-RESULT.md`. The external observer's metadata capture, strict response-ID ledger, and
provenance checks pass independent review. Product runtime files under `src/` are unchanged.

Final required checks pass for both crates:

```sh
cargo fmt
cargo clippy --all-targets --all-features -- -D warnings
cargo test --all-targets --all-features
cargo fmt --manifest-path bench-observer/Cargo.toml
cargo clippy --manifest-path bench-observer/Cargo.toml --all-targets --all-features -- -D warnings
cargo test --manifest-path bench-observer/Cargo.toml --all-targets --all-features
```

The observer suite has 107 passing tests, including 13 response-ledger and 18 capture/provenance
tests. Reproduced failures precede implementation and the review fixes. The main crate includes
five passing guard tests for the opt-in real smoke; ordinary test runs leave its provider call
ignored. The precision helper has 14 passing tests; the complete provider-free measurement-gate
Python suite has 76 passing tests, including the pilot runner and analyzer. The one approved report
assertion is the only modified committed test.

The explicitly enabled real subscription smoke passes in 11.53 seconds using the unchanged
`CommandChat` directive detector and `CodexBackend`. Its exact command is recorded in
`NORMAL-SMOKE-RESULT.md`. First turn: genuinely interrupted. Second turn: raw same-session follow-up
to normal completion. The observer captures one exact response and verifies its original/forwarded
request provenance; no tools or project writes occur. GPT-5.5/`xhigh` and CLI 0.153.4 match rollout
metadata. Independent evidence review verifies the source/binary/protocol and capture hashes.

Whole-workflow accounting remains incomplete: the first response is unreported, and the exact
completed response's 15,871 raw / 2,175 uncached-plus-output tokens are only a workflow lower bound.
The analyzer correctly exits 2 with one unresolved-usage error, before and after rollout extraction.
That is an expected negative exact-accounting result, not proof that interrupted usage is zero.
Historical interval widths remain 6.24094 pp and 140.11206 pp, exceeding the telemetry attempt's
original 5 pp target. Current pilot admission follows `PROTOCOL.md`, and status is in `BATCH-RESULT.md`.

Historical sections below document the earlier analysis and diagnostic work; they do not supersede
the current pilot authority or completed 24/24 report-test verification.

## Analysis correction

`../efficiency-mechanism-attribution-20260830T081131Z/test_attribution_bounds.py` covers intervals
crossing zero, touching zero at either endpoint, and equal to zero, as well as entirely positive and
negative endpoint gaps. Both the full bridge and selected-mechanism coverage retain absolute token
ranges and represent undefined percentage attribution as JSON `null` with a reason. Point-only
`ordered_bridge` calls continue to reject a zero gap by default.

The new regression suite was run before implementation and failed with two failures and nine
subtest errors. Its five tests pass after implementation. All seven existing analyzer tests also
pass. The saved-evidence test requires incomplete normal-endpoint measurement and distinguishes
accounting bounds from statistical confidence and quality equivalence.

The regenerated evidence preserves the original groups, endpoint accounting, measured behavior,
stage attribution, counterchecks, and source evidence. A comparison of those complete JSON sections
against `HEAD` has no differences. The changed numerical fields are the invalid uncached percentage
bounds, which are null; raw-token ranges and absolute uncached-token ranges are preserved.

## Required repository checks

These commands pass in `/home/user/src/work-leaf`:

```sh
cargo fmt
cargo clippy --all-targets --all-features -- -D warnings
cargo test --all-targets --all-features
git diff --check
```

The full Rust suite includes `ui_harness` and `terminal_pty`. No Rust runtime or UI implementation is
part of this patch. Existing `.codex/config.toml` and `.gitignore` modifications belong to the user
and are outside this patch.

The predecessor report suite passes all four tests:

```sh
python3 -B -m unittest discover \
  -s bench-results/efficiency-causal-validation-20260829T210343Z \
  -p 'test_final_report.py' -v
```

The four tests in `test_api_probe.py` pass. They verify that saved diagnostics retain response
identity and safe error codes without response content or credentials, and that cancellation
acceptance rejects missing, negative, or inconsistent token totals. The arithmetic regression was
observed failing before its guard was implemented. These tests do not prove provider support for
cancelled-response usage.

## Approved committed-test expectation

The mechanism study's complete suite passes all 24 tests.
`test_final_report.py::test_report_includes_the_conservative_endpoint_scenario` requires the
corrected evidence status `incomplete_normal_endpoint_measurement`. The user explicitly authorized
this single expected-value correction. The obsolete expectation was reproduced failing before
the correction, and the full study suite passes afterward.

## Subscription handoff diagnostic

`test_handoff_probe.py` passes 23 provider-free tests covering authentication refusal, child
environment isolation, token arithmetic, unique-response reconciliation, schema fields, actual
tool-result delivery, nonzero pre/post-result usage, turn identity, early-event buffering, tool
identity/completion count, permitted activity, and rejection of premature final answers. These
checks were observed failing before their corresponding helper implementations or corrections.

The real subscription scenario invokes one synthetic dynamic tool, withholds its result until
exact upstream-response usage arrives, returns its fixed text, and observes normal continuation
and turn completion. It passes in 8.954498 seconds with two exact responses reconciled to the final
thread total, no interruption, and process exit status 0. `HANDOFF-RESULT.md` records the exact
command, event ordering, counts, and limits. Independent review confirms this narrow pass.

The required format, clippy, and complete Rust test commands pass after this diagnostic is present.
No product runtime, architecture, public API, or UI change is part of the diagnostic; successful
standalone provider verification does not constitute real Work Leaf integration verification.

## Review and real-agent scope

Independent reviews cover the analyzer, new regressions, regenerated evidence, study-documentation
consistency, and the first-batch protocol. They find no blocking implementation issue and no
algorithmic complexity increase. The study changes concern offline arithmetic and reporting; no
agent-facing workflow, runtime interface, launch policy, or interruption behavior is changed.

Provider diagnostics are separate from that implementation verification. Their commands and
outcomes are recorded in `TELEMETRY.md`. Automated checks and metadata queries do not satisfy the
real cancellation/resume accounting gate. The admitted pilot's launch evidence and completed result
are retained in its frozen manifest and `BATCH-RESULT.md`; collection is paused.
