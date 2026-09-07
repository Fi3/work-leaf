# Independent C15 fixed-artifact replay review

Result: **no material finding** in the reviewed source reconstruction, selected-test
identity, or stated qualification limits. This review reads retained sources and
receipts only; it does not rerun Cargo, apply patches, launch providers, or inspect
current benchmark costs. Only this new review note is written.

Reviewed notes:

- `EVIDENCE-TEST-FIRST-REPLAY.md`, SHA-256
  `faef2f12816ed330b4b1d0d3ebf8d623b365bd61d2a9cea4b990b2150aab9890`.
- `DESIGN-TEST-FIRST-ISOLATION.md`, SHA-256
  `99d82be2c35c24dff89bded7eb786a9a18bf955ede11c0e7f68133ba6ffe2493`.

## Cohort identity

The selected observation originated as `direct-002` in
`efficiency-points8-9-20260828T145556Z`. It is **included in H's six-Direct
comparator population**, not a separate newly selected comparator observation:
`efficiency-exact-normal-work-leaf-20260829T181318Z/evidence.json`, JSON pointer
`/observations/2`, explicitly records `group: direct`, `run_id: direct-002`, and
that points8–9 study's `evidence.json` as its source. H evidence SHA-256 is
`4fc87b42b1a5f4a12f74195c6a16754e46e6a6f2605de32cdc79002d361ee246`.

H's separate `score-manifest.json` lists only its six `exact-normal-*` WL workflows
(SHA `f46ec7dfa267d50fa9904a5a9a50ae782011209b56dae5cc6f79badcd603b452`).
Thus points8–9 is the original source study, H is the later analytical population,
and this September replay is a new provider-free diagnostic—not another historical
or model-generated observation. It is not one of the six H WL runs.

## Checked source chain

Native source is
`/home/user/.codex/sessions/2026/08/28/rollout-2026-08-28T20-10-15-01a04990-b481-7702-b8b3-0ee09945eb44.jsonl`,
SHA `d61f6577a67a6bf0f92912c06ec1e9c6efec563378cd76f2562d0d64a241020b`.
N1 matches the declared thread, original points8–9 cwd, CLI `0.149.1` and base
`c92a0b7060a36eac6db2d869b85e589a7a9480f9`; its source hash also matches at endpoint.
Only public call/output and session-identity records were inspected.

All fifteen listed accepted patch inputs were independently hashed and joined by
their exact call IDs to their successful outputs: N120/174/180 before RED and
N208/216/224/232/240/248/256/290/296/304/312/328 after RED. Every listed patch-text
hash and input/output locator matches. Rejected N126, N156 and N264 instead have
explicit verification failures at N127, N157 and N265. N328 changes only the
separate harness block expectation; it does not alter the selected terminal test.
The pre-test public executor commands are reads, including N77's status check.

The retained Direct002 base-checkpoint manifest hash
`5b2ebcc7d228dac6ab36e6938d54ec8a54c00185b28a61d728cd1b8266ef4c15`
and all six declared base files match; head agrees and index/worktree/status are
empty. The retained private checkout `/tmp/work-leaf-c15-replay.TWeEqK/repo`
has that exact base HEAD/tree. Its raw Git commit/tree object hashes, both Cargo
files, three final implementation files and harness-test hash match the report.

The entire terminal test file is 44,052 bytes with SHA
`3a625e94afb8e69e15ee386e4807fc719c6dfb51775b4a3a1e371cde9ff05787`.
The selected block through its closing brace and terminating LF is 451 bytes,
SHA `f38f85cb3aa18bfba00b99d277d60a0ebeaca76960d56102237c4d7e50643c21`;
the separate blank-line separator before the next `#[test]` is not part of that
block. The complete body still asserts decoded clipboard text `aft` and uses
`FakeBackend::from_replies(VecDeque::new())`. Later accepted deltas do not target
this test file. The inspected selected scenario does not invoke a provider.

N196 and N342 have byte-identical complete executor arguments (including the same
command/cwd), SHA `81aad073f1bb4886c7fd6e897e76b10d82fc5c1858c143b7d038c2fd94e24425`.
Their exact output IDs, hashes and one-test behavioral failure/pass agree with
the report. The failure is missing OSC52 output, not compilation or zero tests.

## Qualified conclusion

The retained present-compiler replay receipts support this one fixed-source
RED→captured-implementation→GREEN case with an unchanged selected test. This
review verifies their source chain and quoted outcomes; it does not independently
repeat those executions or certify the complete historical environment.
The reported current rustc/Cargo envelope is explicitly distinct from historical
CLI `0.149.1`. Unavailable provisional trees and the alternative with a changed
test remain excluded from this strict replay qualification, not discarded from
the historical evidence.

This establishes a privately executable dependency fixture, not a natural WL
test-preview implementation, model response to private RED feedback, isolated
timing effect, quality equivalence, or token-saving share. The design correctly
retains the shared-tree safety boundary and names private source snapshots plus
new truthful feedback transport as additional implementation requirements.
