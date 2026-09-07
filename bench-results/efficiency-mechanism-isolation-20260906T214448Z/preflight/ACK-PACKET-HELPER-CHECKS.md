# ACK coding-packet helper checks

The private `build_ack_packet.py` derives manual coding evidence from one terminally
published workflow. It uses the frozen `prompt_inventory` and `capture_provenance`
functions, not whole-phase inference. It assigns no semantic labels or commit-to-ACK
joins and computes no contrasts or token totals. No actual packet is part of these checks.

Reviewed identities:

- Helper: `d00e77ba303c7e19ca02353487619245dce7d86021fef3dc29edce95b09dfc49`.
- Tests: `b0473660f08c139873c8c8dc73199b9b4c37999f69f6631ea9a240a8da209add`.

The initial feature test run was RED with `ModuleNotFoundError: build_ack_packet`.
The independent reviews covered these three concrete RED regressions and their GREEN gates:

1. The fixture uses `runner_work_units.schedule`'s actual `work-leaf-concurrent` row;
   the incompatible workflow gate produced a recorded failure before correction.
2. Two admitted files can share `analyze.py` as a basename. A duplicate-basename fixture
   produced a recorded failure; execution requires the exact expected phase-frozen
   repository-relative helper path and matching source bytes, not basename uniqueness.
3. Unsafe/oversized capture sources must fail before delegated provenance reads. A spy
   regression recorded an incorrect delegated call; source guards and byte budgets precede
   delegation, and the symlink/oversize test requires zero delegated calls.

The 14-test suite additionally covers exact frozen ACK IDs, typed request identities,
physical trace line maps including blanks, retained public edit/read/command/review context,
full locked-command evidence links, reasoning and A/B/C prose exclusion, explicit incomplete
native-body projection, create-new output, terminal receipts, provenance failures, source
hash rechecks, duplicate/conflicting items and absence of invented semantic labels/joins.

Commands run from the study directory:

```sh
python3 -B -m unittest test_build_ack_packet.py
python3 -B -m unittest test_build_ack_packet.py test_analyze_work_units.py test_runner_work_units.py test_trust_work_units.py
```

Results: 14 new tests GREEN; 71 combined targeted tests GREEN. Both new Python files also
pass compile-in-memory syntax and whitespace checks. The independent reviewer reran 14
new plus 28 frozen mediator tests and a synthetic physical server-line mapping check.
Review is closed at the identities above. Storage/indexing is linear in evidence bytes
and identities apart from bounded file sorting; there is no per-ACK full-stream rescan.

No frozen helper, protocol, runtime, provider, prompt, interruption or benchmark behavior
is affected. No provider generation or active-run outcome inspection occurs in verification.
This offline derivation has no agent-facing behavior requiring another real-agent scenario;
the phase's existing runtime verification remains independent. Heavy Cargo validation is
not part of these Python-only checks during active benchmark execution.
