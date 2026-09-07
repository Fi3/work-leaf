# Independent historical validation-cadence review

The reviewed report is `EVIDENCE-H-VALIDATION-CADENCE.md` at SHA-256
`b7057881b40f684d8c002172a80ea1280a930d4f362aa22fdce054c9d28886ae`;
the exact source index is `EVIDENCE-H-VALIDATION-CADENCE.json` at
`c9f7b02c439843d1d76b3782bea4c437f84db25acfc014917b11b19d80ce6918`.

Independent replay verifies all 37 source hashes, 219 indexed native call/output
witnesses, all 32 exact-command initial RED→GREEN chains, all 92 initial-window
checks, all 16 repair windows and all 58 WL command/result/accepted-implementation
joins. Native witnesses match physical lines, typed call/item identities, explicit
turn metadata, command/patch and output hashes, and recorded exit statuses.
The WL joins preserve exact delivered-input hashes and typed RPC/turn identities;
directive parsing trims outer whitespace without changing delivered input bytes.

All 16 distinct accepted repair bodies were inspected: 14 change test fixtures or
assertions and two repair missing production match arms. The inline test change
inside `src/terminal_app.rs` is not misclassified as a production repair. These
records support actual recovery-work paths, not a claim of identical test bodies
or automatic equivalence of every intermediate test assertion.

Independent native enumeration reproduces all 501 leading-Cargo calls and their
phase/subcommand counts: initial author 288 tests, 36 format and 15 clippy; later
author fixes 132 tests, 20 format and 10 clippy. Compound shell commands are not
expanded into assumed subprocess executions or provider response counts.

No introduced finding in the evidence claims. The 92-call Direct window and 58
whole-author WL results have different scopes and are not subtracted. The report
retains incomplete post-first-GREEN purpose classification, formatter-separated
rechecks and absence of complete tree identity for purported redundant checks.
It does not turn action counts into exact token shares or an isolated C15 effect.

This documentation-only audit affects no agent-facing workflow. No provider,
runtime change, public API change, new control or real-agent verification is
needed for this evidence review. The linked timing, action and fixed-artifact
replay reports remain the supporting context rather than rewritten source reports.
