# Corrected C15 diagnostic 002: independent preadmission review

Recorded 2026-09-07 20:07:46 UTC; read-only checks completed before admission. **No preadmission source/settings blocker found.** This review performs no initialization, version/authentication probe, provider call, private operation, fixture test or launch. Root owns the final admission union and the one user-approved observation. Original diagnostic 001 remains failed and is not rewritten.

## Exact source and command

| Input | SHA-256 |
| --- | --- |
| [FINAL-SOURCES.json](FINAL-SOURCES.json), 172 immutable pins | `2669892f733d16103d05a7b17e283832f92acaaccd8ebb75ed6ee821ce18aea1` |
| [FINAL-COMMAND.json](FINAL-COMMAND.json) | `f6af5bea8352caf6f30dd6d0e479fa4220b28fe60dce13c0b596416e3d9c97a8` |
| [BUILD-AND-EXECUTION-SCOPE.md](BUILD-AND-EXECUTION-SCOPE.md) | `811442b20226dc36fad22c1bdf51729b993b5739a596e68b65c40def1e60b065` |
| [EXECUTABLE-CHAIN.json](EXECUTABLE-CHAIN.json) | `4f0d1ec383b5741e1934d13b9bd84cef2b3f37860432705f70793225a0c08532` |
| [PRELAUNCH-PROJECT.json](PRELAUNCH-PROJECT.json) | `db48bea0cc9c6a3cf8312e3ef6d9fc3e64a9f1472a904441b4f4e9df92448b02` |
| [PREPARATION-SCOPE.json](PREPARATION-SCOPE.json) | `c6b4f333f3deefb69dd90377c97452a4680f01b313a27fa2a84c49e9d5a585d7` |
| [PREPARATION.md](PREPARATION.md) | `6dfbf2c9746a744ca5bab0d1ee7b7118f18cc33a6615e1aa0652cd25f17932f7` |
| [RESOURCE-CHECK.json](RESOURCE-CHECK.json) | `5912df3628630caade2608aa6cce72ad45786c01b13e835f3afd550bd5f5a9b6` |

All 172 endpoints independently match before and after this check (`cae310`, `ebdac1`). Relative to the original final 159-source inventory, only the bridge's existing hash differs, at reviewed correction `a8395dfe…`; four original mutable project files and the original binary are deliberately omitted, and 18 declared new source/config/qualification inputs are present. The eight later original-admission provenance pins also independently match. Root will carry those eight as predecessor provenance and pin the new inventory/command/preparation/project/chain/resource/review documents in its final union. They are not confused with effective 002 settings.

The selected source is `0756c6efc6e2d3ea44462327e0bccbd3b04014fa`, including correction commit `05e43d9`. All 36 source/Cargo/architecture entries checked against that commit match; the five private harness/guard/Cargo files equal both selected commit and diagnostic 001 byte-for-byte (`f8c1de`, `92e9d5`). No diagnostic prompt or guard change is hidden in the correction. The two-file source review and its independent 38-test result remain in [BUNDLE-NAMESPACE-INDEPENDENT-REVIEW.md](../c15-runtime-bridge/BUNDLE-NAMESPACE-INDEPENDENT-REVIEW.md). That review also reconciles the owner-executed full correction gates: 534 passed, 0 failed, 22 ignored, fmt/clippy green, and 50 default-library passes. The current private-crate 9-pass/1-ignored, clippy and fmt results are root-recorded in the build scope, not a second test execution by this review.

The exact executable `/tmp/c15-real-runtime-002.DlZ5tz/private-test-first-diagnostic`, SHA `c8383d2bbfa3d98f01d81220984cd357d2228872737b80a3aa89469015a92c10`, matches the declared completed build output. It is a regular, non-symlink 0555 file with one link and an inode distinct from that build output. The original admitted binary remains separate. Literal argv is:

```text
/usr/bin/timeout --kill-after=5s 300s /tmp/c15-real-runtime-002.DlZ5tz/private-test-first-diagnostic --ignored --exact real_subscription_private_test_first --nocapture
```

The launcher is the unchanged pinned diagnostic-001 `bounded_launch.py`, invoked by the declared Python `-I -B` command after root supplies the new exact admission path/hash. The command differs only in the executable path; limits remain one attempt, eight outer calls, two intended roles, 30-second private command, 180-second private operation, 240-second harness cancellation, 300-second outer timeout and five-second final kill grace. No automatic retry or additional benchmark/control/Direct/C21 observation is admitted here.

## Effective settings and mutable baseline

The environment comparison is exact after the declared new owned paths, run identity and primary marker are substituted (`707d92`). Variable key set, unset list, PATH suffix, timing and instrumentation are unchanged. Observer config/manifest agree on run/source/project identities and marker digest. Subscription wrapper, gpt-5.5/xhigh, raw capture, 1000-ms grace, forward mode, primary marker and project-inventory setting remain selected. No API credential is copied or displayed; global configuration hash remains `21bcb4825009b9561fa200a806ccc7d68993cbd4e718342b257a594dfaf31908`.

All 16 executable/entrypoint records, complete bytes, symlink targets, canonical resolutions and six effective PATH resolutions independently match (`92e9d5`). This includes the subscription wrapper → `/usr/bin/codex` → package JavaScript/native executable chain and both owned observer proxies. The recorded CLI 0.153.4 is explicitly carried from the prior observed version under that unchanged chain, not freshly probed. The bridge config is byte-identical (`d01e198e…`), with empty overlays and `qualification_implementation: false`; all nine helper/executable references plus the public capsule match the immutable inventory. No helper is executed by those checks.

The clean initial project `/tmp/c15-real-diagnostic-002.aD9RpS/project` has HEAD `3181e41560e66d4a533e4eca3a871fd8df9790cf`, tree `0d2d02ef2859890ccc2a7113fd27e3b457f90c6b`, normal index flags and exactly four tracked files. Their bytes/lengths match the separate project receipt and original initial tree: `.gitignore`, `Cargo.lock`, `Cargo.toml`, and the incomplete `src/lib.rs`; no test body is supplied. The receipt is immutable evidence, but its project-file paths are **not** immutable postlaunch endpoints. Intended accepted edits must be inspected as outcomes, not treated as source-pin corruption.

The canonical sibling preview and ordinary-bundle roots are precreated, empty and disjoint. Admission, attempt, terminal, harness result, prompt trace, app-server captures and invocation inventory were absent at the independent check. This verifies the formerly uncovered precreated-directory shape without running the initializer or changing the fixture.

## Remaining real-work gates

No passing real-agent C15 result exists at this cutoff. Original 001 startup exit 101, analyzer flags and extracted zero-work evidence remain intact. Synthetic validation-only startup coverage is not confinement/provider qualification. Root's selected postcapture plan preserves actual scenario/timeout/closure/source statuses, every accepted public/native input including usage-less threads, private proposal/result/delivery identity, same-test semantics, normal patch/check/review work and all unresolved errors. Copied observer ELF marker warnings remain original analyzer flags; any exact-file explanation is separate and cannot waive other locations or `capture_complete`. No accounting re-extraction, cost claim, favorable-color test rerun or token contribution estimate follows from this preadmission review.
