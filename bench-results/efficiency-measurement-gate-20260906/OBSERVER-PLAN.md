# Observer-only accounting plan

This document records the completed telemetry attempt and its original acceptance threshold.
`PROTOCOL.md` governs the authorized raw-primary pilot; uncached precision and the earlier dual
five-point cutoff are not current admission gates. `BATCH-RESULT.md` records pilot status.

## Objective and authority

Measure the token difference between normal concurrent Work Leaf and direct sequential Codex for
the same work, with independently reported quality. Provider work uses only the existing Codex
ChatGPT subscription. API-key generation, credential copying, protocol redesign, and native-tool
handoff integration are outside this plan.

The accepted accounting precision target is an interval width of at most **5 percentage points**
for each reported saving metric: raw input plus output and uncached input plus output. This is an
accounting bound, not a sampling confidence interval or a requirement to find positive savings.

## Fixed experimental behavior

- Work Leaf runtime, prompts, tool protocol, command policy, and concurrency remain unchanged.
- The historical observer policy remains a 1,000 ms maximum pre-forward interrupt grace, with
  resumed output releasing the interrupt. Raw usage does not alter that state machine.
- Model and effort remain GPT-5.5 and `xhigh`, on the existing subscription route.
- Metadata opt-in belongs to the external benchmark observer. Its exact request changes and event
  filtering must be audited; instrumentation is not assumed to have zero timing overhead.
- Old observations retain their original versions and identities. A newer CLI verification is not
  retroactively pooled with them as an identical configuration.

## Execution and decision gates

1. Save this plan and precision threshold before new provider work.
2. Add regression tests that fail before implementation. Capture response-ID-scoped usage from
   `rawResponse/completed`, validate identities and all required counters, and reconcile against
   cumulative totals without adding the same response twice. Null, conflicting, duplicate, resumed,
   and genuinely interrupted cases require explicit coverage. A response earlier in a turn is not
   evidence for the response interrupted later in that turn.
3. Reprocess available captures and retain a reproducible before/after comparison. A missing raw
   record remains missing; a later cumulative notification counts only if its arithmetic proves
   inclusion. Historical response sizes and cache-hit rates are not hard upper bounds.
4. Run bounded real subscription verification through the unchanged Work Leaf backend, including a
   directive interrupt and a raw follow-up. Inventory every response and unresolved tail. Keep
   exact-response coverage separate from whole-workflow completeness. Stop on infrastructure or
   authentication failure; do not switch providers or silently extend waiting.
5. Run formatting, warning-free clippy, and all-target/all-feature tests for both the main crate and
   the separate observer crate, plus new accounting tests and a documentation/architecture review.
6. Launch the first benchmark pair only if the observer demonstrates useful accounting precision
   on the real unchanged workflow and all readiness gates pass. Freeze the manifest first. The
   pair contains one direct sequential and one concurrent Work Leaf observation, then collection
   pauses for the user. No replacement or second batch is authorized.

If the new metadata cannot meet the precision gate, save that result and pause before a full batch.
A captured batch with wider uncertainty remains an inconclusive result; it is never discarded or
replaced to obtain a narrow or favorable result.

## Arithmetic and baseline

For positive Direct bounds `[Dlo, Dhi]` and nonnegative Work Leaf bounds `[Wlo, Whi]`, saving is
bounded by `100 * [1 - Whi / Dlo, 1 - Wlo / Dhi]`. Width must be at most 5 percentage points for
each accepted metric. Undefined denominators or unsupported finite bounds fail the gate. Exact
accounting has zero accounting width, but does not establish repeatability or quality equivalence.

The six historical normal Work Leaf observations contain 35 unresolved responses. Using their
frozen 386,400-token per-response ceiling, the descriptive raw saving interval is approximately
45.3834%–51.6244% (6.24094 pp wide); uncached-input-plus-output saving is approximately
−123.6201%–16.4920% (140.11206 pp wide). These are the baseline, not useful precision passes.

The cumulative replay audit finds no recoverable residual for those tails: 32 later advances equal
the later response's `last` usage, and three have no later advancing total. Response-level records
are a candidate source of additional evidence, not a promise that cancelled responses have usage.

The separate native-tool probe in `HANDOFF-RESULT.md` only establishes that completed responses can
emit exact response-scoped counters. It does not verify the normal Work Leaf workflow or repair the
historical missing responses.

## Progress

- Plan, implementation, historical replay, and bounded real verification are saved.
- Observer metadata capture and reconciliation pass automated and real integration checks.
- Historical widths remain 6.24094 pp and 140.11206 pp; the real interruption remains unmeasured.
- The precision gate fails. First benchmark batch: not launched; collection paused.
- Results and remaining readiness issue: `OBSERVER-RESULT.md` and `NORMAL-SMOKE-RESULT.md`.
