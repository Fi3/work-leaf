# C15 diagnostic 001: retained startup failure

The only admitted command ended at 2026-09-07T19:42:55.833120+00:00 with
exit 101, before the first provider launch or private bridge invocation.
This is not a successful real-agent or private-test qualification. Its original
terminal and both original postcapture outcomes remain unchanged.

## Exact failure mechanism

The source is the admitted compiled commit
`22a38282d2ae12f184c833de11d377241a18fca0`, retained at
`/tmp/c15-startup-failed-source.IsbptR`. All 40 corresponding archived
source/Cargo/doc/harness files match their original admission digests.

In that archive, `harness.rs:170` unwraps
`CommandChat::prepare_agent_launch` (`src/cli.rs:962`).
The benchmark-only `private_test_first::active` enters
`bench_experiment::load`, which initializes the private bridge before creating
the prompt-evidence file (`src/bench_experiment.rs:147`).
`bench_private_test_first_bridge.rs::initialize` calls
`validate_bundle_separation` before making the private validation directory or
invoking Python.

At `src/bench_private_test_first_bridge.rs:125`, an already-existing
canonical ordinary-bundle parent is its own nearest existing ancestor.
Its relative suffix is empty; joining that suffix appends a trailing separator.
The raw `OsStr` equality check at line 135 therefore rejects the resolved path
against the un-suffixed parent even though the admitted preview and ordinary
bundle namespaces are disjoint. The exact retained panic is in
[PROCESS.stderr](../PROCESS.stderr:2).
The actual provider launch at `harness.rs:188` is never reached.

The initial bundle parent was empty and pre-created, as recorded in
[PREPARATION.md](../PREPARATION.md:17) and
[PREADMISSION-REVIEW.md](../PREADMISSION-REVIEW.md:32).
It is absent after unwind, not unchanged-empty:
`CommandChat::new` owns a `ContextBundleStore`, whose
`ContextBundleStoreInner::drop` attempts to remove the store directory and its
empty parent (`src/orchestrator.rs:337`). It was not recreated.

## Once-only original postcapture

Scope [POSTCAPTURE-SCOPE.json](POSTCAPTURE-SCOPE.json) was published before either
command; SHA-256
`a5dc894ede80a4f76fddb3bc625dbbe9e662fbc9352b9fdeef0fbaded21588a3`.
Each exact command used the unchanged observer, an external 180-second timeout
plus five-second final kill grace, exclusive output streams and one retained
attempt. No command was repeated.

| Original command | Actual UTC interval | Exit | Retained result |
| --- | --- | --- | --- |
| analyze | 19:52:54.809588–19:52:54.862637 | 2 | capture_complete false; eight binary marker errors |
| extract-rollouts | 19:53:53.966398–19:53:54.253960 | 0 | zero observed/matched threads; no extraction errors |

Analyze accepts the absent invocation directory as an empty inventory
(`bench-observer/src/lib.rs:5050`) and published the required summary.
Extraction therefore met its explicit prerequisite (`lib.rs:7317`).
No reanalysis consumed the later rollout result.

The eight analyzer flags name four marker classes in each of
`observation/proxy-bin/codex` and `proxy-bin/sh`. Both are complete ELF copies
of the admitted observer binary with SHA-256
`238bdc610a28edd328d91046f9674dbe59ad2f2d7520fedf681c1ae95a56e386`.
The unchanged `collect_artifact_files_for_secret_scan` recursively scans regular
files, including those binaries (`lib.rs:6889`); its marker detector is byte-based.
These hits are in pinned executable bytes, not generated model/native content.
This explanation does not clear the flags or replace `capture_complete: false`.
Any future source qualification must separately prove exact binary identity and
must not waive other marker locations or unrelated errors.

The analyzer stdout/summary SHA is
`d817c2bb76912d3ecedcc5543331e28a43d349ccaba82607c2cb368f1086d20d`;
extraction stdout/audit SHA is
`20abaa3caa5fdc3f8658b63f84fe80ae197d8ccff3f1d0bef226d797ff0f95ed`.
Both complete stderr streams are retained and empty.
The execution-receipt SHAs are
`531b50815017ca75fff7f384e6158f95ae5efdc925194a104771ccdc87c98ff3`
and
`745badb1f2dbc8f448871780d5a763c35fc66b0c2f31d191fad55c1a6c7ca23c`.

## Finite startup witness and limits

[STARTUP-WITNESS.json](STARTUP-WITNESS.json), SHA-256
`2a0cd161f0a4457c4097cf590b7c0fef4057db1f1cd71b2bea54e4b97b8a8fe1`,
retains complete safe source identities, physical locators, original errors,
execution receipts and final observation inventory.

At its 19:57:48 UTC cutoff, there is no invocation/app-server/locked-command
capture, prompt trace, private validation directory or `HARNESS-RESULT.json`.
The preview root is empty. No accepted author or reviewer thread, private test,
test color, shared patch/ACK, ordinary validation or review was reached.
The four-file project is clean at the original HEAD
`3f1565cf784f324b0fe9af1a9f30e59f331bc7fc`, tree
`0d2d02ef2859890ccc2a7113fd27e3b457f90c6b`.
No test was run to manufacture RED or GREEN.

The original terminal SHA
`316e10157dbd68f113ddc562210857771fa83ebfb5ff1d1ecb85a627671ece0c`
reports all 167 admitted source endpoints matching at actual closure.
All 18 retained postcapture inputs and 40 archived original files match at both
postcapture checks. The later live namespace fix has a different source hash;
the witness records it explicitly and does not assert that current live source
equals the failed compiled source.

The v5 C21 collector remains inapplicable. No rewritten collector, full-workflow
accounting, provider call, private execution, replacement observation or retry
was performed here. Extraction exit 0 over an empty expected population is not
a green real-agent result. Publication is exclusive, not a transactional
multi-file guarantee.
