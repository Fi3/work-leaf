# Independent review of historical test timing and validation scope

## Disposition and scope

No introduced factual, source-isolation or causal-attribution finding was identified in the bounded review of [historical test timing](EVIDENCE-H-TEST-TIMING.md) and [validation cadence](DESIGN-VALIDATION-CADENCE.md).

Both documents were read completely. Independent checks cover the four public-action examples below, all 18 historical WL author launch-policy inputs, the identified historical/current source boundaries, and the stated inference limits. This is **not a second complete classification of all 18 Direct sequences or all 53 WL patch groups**. Those complete-census claims remain the evidence author's audit, with its published physical locators and source inventory.

Reviewed document SHA-256:

- `EVIDENCE-H-TEST-TIMING.md`: `894130bacdce03adb1b86173a05e03162a57f1b3c91ebbefc54205a692659901`.
- `DESIGN-VALIDATION-CADENCE.md`: `4723e0abfa11ebfd60578e6980146e15d768ee31105c11f11610b95e87854b20`.

## Independently checked public-action chains

Physical `N`, `C` and `S` locators use the exact native/client/server paths and SHA-256 inventory in the reviewed timing document. Samples are its named examples and one separately classified compiler-RED row, not cost-ranked selections.

| Historical sample | Checked chain | Supported distinction |
| --- | --- | --- |
| Direct `step4-direct-003`, slash author `01a04de5-49d9-72f1-a87d-af9f50224ac8` | N114→116 test-only accepted write; N120→122 executes the new controller test and fails on absent `/status` send; N128→130 and N136→138 production routing/helper writes; N144→146 repeats the same focused check and passes. | An actual pre-implementation behavioral failure, not an invalid command or a claim about separate provider responses. This review samples the first controller increment, not the document's later terminal/harness increments. |
| Direct `point7-exact-direct`, visual author `01a04842-cbbf-7952-97d1-98fc8afc784a` | N138→140 adds the new harness tests; N144→146 reports missing `TerminalUi::clipboard_text`; N159→161 first adds production visual-state/API handling. | Compiler-contract RED is correctly separate from executed behavioral assertions. The output never reaches the new visual assertions. |
| WL H004, slash author `01a04ee8-0c81-7b72-a923-f14e71ee8c69` | Complete S3478 edit includes production routing and four tests; C24/S3484 accepts the ACK. S3513→C26/S3519 returns status101: the positive route test passes, the negative test's selected-frame error assertion fails. Complete S3681 changes that assertion to the controller transcript; C28/S3687 accepts it; S3716→C30/S3722 passes; S3733 is done. | Cohesive delivery does not eliminate repair. This particular failure and test-only repair follow implementation, so neither is initial tests-first RED. |
| WL H002, visual author `01a04ebc-2c3d-7741-8e87-8e1d30282cad` | S54889's `tests/ui_harness.rs` section has exactly zero added/deleted lines; C65/S54895 accepts the receipt; S54918→C67/S54927 passes the existing 31-test harness. C82/S56071 delivers the actual reviewer finding about visual-mode routing and missing harness tests; S56102 requests source, C84/S56108 delivers the bundle. Complete S57956 supplies mode/escape repairs and four new tests; C86/S57962 accepts it; S57984→C88/S57990 passes 35 tests; S58001 is done. | A receipt naming a test file is not proof that tests were added. Later review repair does not retroactively make the initial group tests-first. The omission and actual coverage limitation remain in the evidence. |

Eight Direct input/output pairs have matching nonempty string `call_id` and explicit same-turn metadata. Eight sampled WL ACK/command-result requests have matching **string-typed** RPC replies, no RPC error member, and the same author thread. The command directives belong to their accepted ACK turns; preceding edit turns have their own matching accepted requests. The additional H002 reviewer request C82 is string RPC `81`, accepted at S56071 as turn `01a04ed1-b628-7a20-addd-bee736723017`; its public source-read directive is S56102. No commit-to-ACK association is invented.

Rehashed sample native sources match the timing inventory:

- Direct slash: `f06ac070bb68fbaf3c11c137d1548193bf1c59840418a43b5dfb6d293ecd4217`.
- Direct visual compiler RED: `81d8532466ee761bc4f6a35cef95851b85eb14512eaca5578b0a8346e4539358`.
- H004 server: `353021d33cb79748afa810f1d128cd09bf18ed93d13f7b583e0f0e8347053eed`.
- H002 server: `670459b291165a08996e8db5943a0be3b801d1d198d4fd065650ec725528154f`.
- H002 S54889 complete public text: `5d1e40092fd476d3ffb400c1809610c08c58aa02fbd4e1e15ed7e78a2c5cd870`.

## C15 safety and C10 isolation

All six H client-stream hashes match the timing document's inventory. In each capture, author launches C4/C6/C8 contain exactly one copy of each buildable-test translation and exactly one copy of each literal repository RED-first requirement. This verifies actual delivered coexistence, not just source text. Retained historical `src/agent.rs` is byte-identical to Git blob `5b1d1ef9590850faed26052f909ddff7ff8f127d:src/agent.rs`, SHA-256 `dd4464f94d887b30ad603f331743d18e69936d0f898c74383595c34f43d4cb74`.

Current `src/agent.rs:346–349` preserves the known-red/compile-breaking shared-tree prohibition and cohesive test/implementation instruction; `:359` requires manual test changes through the edit protocol. The per-instruction translation at `:514–524` is consistent with those constraints. Thus mandatory published failing-test-only patches cannot be introduced merely by changing the test-design sentence while claiming all normal safety/ownership rules are intact. A private RED preview is a distinct execution/transport/snapshot design to review, not proof that safe test timing is impossible. Existing buildable-work-unit cues do not require executed RED; their unsuccessful mediator result does not test that requirement.

C10's proposed narrow boundary is also described accurately: preference changes at `agent.rs:466/507` and `orchestrator.rs::render_patch_applied_prompt:3017`, with consistency fragments at `agent.rs:316/470` and `orchestrator.rs:2815/3027`, leave substantive eligibility and stop conditions intact. `run_command_for_agent:1176 → PatchOwnershipTracker::other_agent_test_locks:495 → render_other_agent_test_command_prompt:3039` is a real enforcement path, not a dispensable breadth adjective. The formatter, ownership, external-blocker, known-red and final-gate constraints are separate factors. Calling this narrow preference treatment removal of all scope restrictions would be false; the design explicitly does not do that.

The saved point7 Direct `runs/sequential-feature-1-implement.prompt.txt:13` really contains the focused-check/fix/rerun guidance and broad-final-check deferral, SHA-256 `c715888b6da655322396c0f7f8c022994f53f0961c2ca93a4d5e6f73c3e20ff5`. Current `bench-three-features-direct-common::normal_validation_guidance:916`, injected at :941/:964, matches that relevant text. Final integration checks remain `bench-validation-common::bench_run_final_gate:3`. Current source hashes for these scripts, `agent.rs` and `orchestrator.rs` match the reviewed cadence document.

The documents correctly distinguish ordering exposure from a randomized cause, tool actions from provider responses, compiler errors from reached assertions, and purpose/timing from quality equivalence. Neither supplies a historical saving percentage or justifies another control, repeated cue test, or endpoint substitution.

## Verification limits and documentation

The docs catalog contains `architecture.md`, `benchmark-operator-policy.md` and `orchestrator-file-workflow.md`. Their relevant shared-tree ownership, mediated-write/validation and benchmark-admission descriptions do not require changes for this evidence-only review. Architecture's buildability description at :416 is consistent with the inspected source; its SHA-256 matches the timing document.

No runtime, helper, frozen report, protocol or original capture was edited. No algorithm is introduced, so there is no new quadratic-complexity concern. No private reasoning body, token/cost report, parked-read outcome, current benchmark outcome or provider generation was used. Rust checks and a fresh real-agent scenario are not applicable because no agent-facing behavior is changed.
