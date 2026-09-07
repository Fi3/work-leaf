# Work-unit observer verification

The final inventory implementation is committed with the v2 runtime at
`2a69f863a294c224d137120ea0a21834cc88210d`. Required observer format, all-target/all-feature clippy
with denied warnings, and all-target/all-feature tests pass: 34 library, 29 analyzer, 22 project
inventory and 45 proxy tests (130 total). The public-crate dependency cache is
`/tmp/work-leaf-inventory-cargo.nqdcWE`; it contains no copied subscription credentials.

Independent review found and rechecked two resource-accounting cases: rejected directory children
consume the visited-entry limit, and partially read or subsequently rejected files consume actual
read bytes. RED tests precede the corresponding fixes. Collection is bounded and O(B + F log F),
with an indexed audit of at most 64 snapshots. No provider argument, prompt, stream, cancellation
decision or response-wait policy is altered by the inventory.

## Final-build real scenario

`work-units-trust/ADMISSION.json` admits one separate two-turn real diagnostic. The final debug
observer hash is `818267f7b487aa950b302d4d7acb0bff56e5449359775f1296fbe7b677475e60`.
The real `CodexBackend` launches a read-only author, records the existing pre-linearize checkpoint,
and launches its normal `linearize` identity with the benchmark-prescribed full-access sandbox.
Both complete the exact requested reply without tools or file modifications. The diagnostic passes
in 14.48 seconds and the observer stops and flushes successfully.

All four base/pre-spawn/pre-linearize/final inventory records are valid and have the same digest:
`3a9d4059f7d14d645028f39c877f9f5f09cb65720fef23e573922cd2aa35671d`.
The final `work-units-trust/observation/analysis.json` has `capture_complete: true` and no errors;
native rollout extraction also has no errors. Recorded usage is 28,089 input (21,248 cached) plus
405 output, or 28,494 raw tokens. Reasoning is 378 of those output tokens, not an additional charge.
These are diagnostic totals, not a causal comparison or benchmark observation.

Exact captured permissions, project-trust difference and the prospective classification/replay
contract are additionally checked by the study's separate trust adapter. This real scenario does
not itself waive any configuration integrity rule.

## Earlier diagnostics and post-capture executable identity

The four earlier work-unit handoff diagnostics were initialized against the previous debug observer
hash `87fad5858302ca0ae93e0fc9e23db42914c699d714a031fa958c1df8b6460191`. Their captures closed before
the inventory resource fixes rebuilt that same debug executable path. Current offline analysis
therefore retains `observer executable SHA-256 changed after observer initialization` for all four.
Their configurations are not rewritten and this flag is not removed.

The corrected control/incremental workflows still passed their actual five-response plumbing
scenario, and the independent typed-request/full-prompt audit still validates their captured
delivery. That narrower verification is not full current accounting-readiness. In addition to
the executable-identity flag, corrected incremental has four unreported interrupted turns. Original
incremental has four such turns and a retained rollout-inventory error; its recorded zero usage is
an empty measured prefix, not zero generation cost. Original control/incremental remain failed
scenarios because of the fixture's three-round cap. All generated analysis and extraction reports
are retained without overwriting their outcomes.

The admitted benchmark phase uses exclusively frozen copies of its runtime and observer binaries,
with immutable phase-owned executable paths. Rebuilding a workspace debug path is not its runtime
selection or evidence source. `BUILD-ATTESTATION-WORK-UNITS.json` identifies the intended release
inputs; the phase manifest supplies the actual frozen admission authority.
