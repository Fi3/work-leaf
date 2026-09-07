# Accepted historical WL context evidence

This is a provider-free Stage B exposure audit of **H's six accepted normal-WL workflows only**,
`exact-normal-001` through `006`. It does not pool P/S/W, inspect parked R costs, rerun a model,
retotal usage, estimate percentages, or assign a share of the historical Direct–WL difference.
Failures and unavailable-file results remain in the delivery inventory.

## Sources and identity gate

The cohort is fixed by `../efficiency-exact-normal-work-leaf-20260829T181318Z/score-manifest.json`
(SHA-256 `f46ec7dfa267d50fa9904a5a9a50ae782011209b56dae5cc6f79badcd603b452`). Its
`evidence.json` (`4fc87b42b1a5f4a12f74195c6a16754e46e6a6f2605de32cdc79002d361ee246`)
supplies all twelve original stream paths and hashes in `accounting.capture_bound_audits[]`.
Every stream matches its recorded hash. Every `turn/start` has one exact typed successful reply
and one same-thread/turn public `userMessage` with byte-equal text. Historical user content has
an additional `text_elements: []`; this empty metadata is checked separately, not mistaken for
a text difference. There are 67/50/58/42/65/73 accepted turns respectively and eight threads per
workflow. Private reasoning bodies are neither inspected nor reproduced.

`infrastructure/manifest.json` (`121c4ed265e967e517aaa14f1b6c71a123ddc3a5ca6c5ebd8d84e5a47cfaaa03`)
pins WL source commit `5b1d1ef9590850faed26052f909ddff7ff8f127d`. This audit uses that Git object,
not current experimental code, for renderer/tracker behavior. At that commit:

- `src/orchestrator.rs` SHA-256 `f88aeea1e910504eeb4434291e662ff2455e64edb5b2d45f87f5c316c6aafe24`;
  `send_file_read_response:995`, `split_repeated_file_reads:1066`, `read_requested_files:1553`,
  `render_file_read_response:2048`, repeat rendering `2073/2132`, thresholds `2172`, bundle
  rendering `2183`, automatic refresh `2213`, command output `2478`.
- `src/locks.rs` SHA-256 `0590b3409c5f8ad9cd44b0320d24b153d4485a26a8c065a877f8742f0b029697`;
  `FileLockTable::normalize_path` and `normalize_relative_path` govern project-path identity.
- `ContextBundleStore::write:271` serializes the held snapshots; `read:296` is the separate
  explicit-bundle route. `FileReadTracker::record_snapshots:365`, `clear_files:391`,
  `clear_agent:403`, accepted-write branches `795/867`, and `done:969` govern lifetime.

Each H capture's `S1` initialize reply identifies `work_leaf/0.150.1`, not the later diagnostic transport. Public app-server
item identity and returned text here are not a native per-item token-attribution assertion.

`C<n>`/`S<n>` below are physical client/server JSONL lines. All cited RPC IDs are strings.
For run `nnn`, the capture root is
`../efficiency-exact-normal-work-leaf-20260829T181318Z/runs/exact-normal-nnn/exact-normal-nnn-three-feature-bench-artifacts/observation/app-server/<invocation>/`.
The filenames are `client-to-server.raw` and `server-to-client.raw`.

| Run | Invocation | Client SHA-256 | Server SHA-256 |
| --- | --- | --- | --- |
| 001 | `00002676642578624406-641` | `90d9e5a624d9f04bb2c7809cb73d1af18b787a0cd2d270718c64c659bf906e54` | `42379ffc5bd1c929738a69982f70f1439ed20a3c2774a751d9d1b29fbbb7d339` |
| 002 | `00002676642578623219-639` | `3bb9f3541b8f5d89ddf5ea15dfcd9de817a432f6b9aa9d54ed7e0d21e8861d42` | `670459b291165a08996e8db5943a0be3b801d1d198d4fd065650ec725528154f` |
| 003 | `00002676642578620844-642` | `784dc35bb6bd618896304125ee891448b396a0c3dd37c0731bee32cbe0e55c31` | `d7c9e8b7f35ff152e56c0ebe25240ec5de6f95ce0440667b7a5fd5cda0a689ba` |
| 004 | `00002679513471323065-646` | `598ce84b96e05232df0257f8b7304746c8d1aaa219d64d264c4ea1412e50b61a` | `353021d33cb79748afa810f1d128cd09bf18ed93d13f7b583e0f0e8347053eed` |
| 005 | `00002679513471141406-639` | `8c444063cad8ad04e1f2be9224266e7947660621380c32ad08dc6c70ad418bdd` | `c35711c5398eff5bff47bc3c763db5813ae4ae18606854f6d8870927d1cdcac1` |
| 006 | `00002679513471141406-640` | `e2b4ed1812205251473e8e012da25e1fd329c10d39dfe492f02f184cb369b113` | `e0e473c2b3c548dbfafd0abad88abe71466e7c3138bc4f329889cf53d12bd454` |

## Complete delivery census

There are **98 file-text deliveries**: 84 include the owned initial bundle-manifest prefix;
13 contain actual renderer repeat sections; two have small untracked inline text. These are
overlapping categories: 005 `C58` has a manifest plus repeats. Twelve repeat deliveries are
changed-bearing, one is unchanged-only; five changed-bearing deliveries also include unchanged
files. Repeat identification uses the actual header/explanatory paragraph and owned component
position, not a marker found inside copied source. The mixed manifest prefix is independently
reconstructed from its archived snapshot inventory before checking the following repeat section.

| Run | All file-text C lines | Repeat C lines | Automatic refresh C lines | Delivered command-result C lines |
| --- | --- | --- | --- | --- |
| 001 | 10,12,14,16,18,20,22,24,28,39,60,63,73,76,100,109,117,125 | 73,125 | 65,78 | 32,48,50,54,69,86,92,94,104,113,115,121 |
| 002 | 10,12,14,16,18,20,31,58,60,77,84,92 | 92 | 36,62 | 24,44,48,52,67,75,79,88 |
| 003 | 10,12,14,16,18,20,22,24,26,41,49,58,72,78,88,91,97,105 | 49,105 | 43,47 | 30,34,56,62,70,76,82,95,101,110 |
| 004 | 10,12,14,16,18,20,22,37,50,54,56,66,69,80 | 54,56,80 | 52 | 26,30,44,60,76 |
| 005 | 10,12,14,16,18,20,22,24,35,56,58,77,79,84,104,107,115,118 | 56,58,115 | 37,39,48,67 | 28,46,52,65,71,88,94,98,111,122 |
| 006 | 10,12,14,16,18,20,22,24,26,37,68,73,77,91,93,101,113,124 | 77,101 | 39,60,70 | 30,46,52,56,62,87,89,97,105,107,118,128,137 |

## C03/C04/C05 explicit-bundle route: H nonexposure

All 98 accepted file-text handoffs have exactly one public top-level read directive in the
immediately preceding accepted same-thread turn. Structured-edit/patch bodies are excluded from
directive scanning. The actual read arguments are ordinary relative paths; no duplicate normalized
project path, adjacent second read to coalesce, or explicit owned-bundle read request occurs.
Multi-file requests are not coalescing exposure. Native tools opening bundles are a separate route.

For example, 001 `S93` requests six distinct paths in one directive. `C10`, RPC `9`, is accepted
at `S103`, with exact public input at `S135`, thread `01a04ebc-2a9e-7662-8f06-5d0cfe7c08fd`,
turn `01a04ebc-691d-7c10-a638-c0e6b22e3955`. No merge/dedup saving should be attributed to this
handoff or the other 97. A future toggle for these mechanisms is not motivated by this H exposure
census; manufacturing redundant reads would test a different workload.

## C01/C06/C35: manifest, archived version, and actual selective retrieval

All 84 delivered manifest prefixes exactly reconstruct from their corresponding archived
`context-bundles/manifest.jsonl` record: source bundle path, ordered project paths, and digests.
All 84 archived bundle SHA-256 values and lengths match; walking the source-defined bundle
serialization by recorded body lengths independently reproduces each snapshot's FNV digest and
exact boundary/newline framing. This establishes captured-version identity, not later filesystem
freshness or that every file was read by the model. The 15/11/16/11/16/15 archive records respectively
match the 84 delivered manifests without an unused/missing archive in this cohort.

Bundle manifest paths are relative to each run's `observation/` directory:

| Run | `context-bundles/manifest.jsonl` SHA-256 |
| --- | --- |
| 001 | `20e90a358135c9fc7c5e606cc13b18bbe6bb51430d8d71cb8698eabdb0a04b43` |
| 002 | `b67121c64062886c3cdcd7a8b8219316bfd83f70f04db0e6228bfcbfa2c33c9e` |
| 003 | `210b91f525efac2cb1a796443976a96080099d3bface03553d2265a2ba57e0e6` |
| 004 | `0d012293436de2c31c02320b599463ea78b7c7c863d0659a50e25711ced57c7e` |
| 005 | `039287070742580bcd7634ca9f5c181677a3002d60308069bf6d63a68c9f9774` |
| 006 | `b64da86c122250698ba6a42bc8d1c107e02d59055ff4265a49b31989c0d18067` |

**Exact retrieval witness, selected in run/occurrence order rather than by price:** 001 `C12`
(RPC `11`, accepted `S131`, public input `S137`) delivers a 982-byte manifest to thread
`01a04ebc-3891-7172-bbb2-3bbdab8c9de7`, turn `01a04ebc-6be8-7672-b7d7-63c407e0067a`.
Manifest row 2 identifies `files/orchestrator-532-0/bundle-1.md`, 200,626 bytes, SHA-256
`da25d4253b05b61018af69212f003ebb6a2424ec6808b4eb6965e5b162e2c6a8`.
The same-turn completed public native command at `S192`, item
`call_rOf5Lwy9cw9TFhky0moED7mM`, executes `sed -n '1,220p'` on exactly that issued bundle
path and exits 0. Its 13,099-byte `aggregatedOutput` equals the archive's first 220 lines
byte-for-byte (SHA-256 `a7bb6d915251126b263a969e1cd1b03af17bb4f873a55bdf912fed6292d36eb4`).

This proves deferred delivery plus actual selective **returned text**, not merely a path mention.
It does not prove selective disk I/O, that later calls never retrieve the rest, that this public
output is the whole provider input, or a net/token saving. H's old captures lack the later exact
per-input-item attribution records; no diagnostic price is transplanted into H. C01 and C06 share
this pathway and cannot be independently added as two savings. C35's immutable-version property
is part of its correctness, not another counted saving. The already-collected large-read variant
remains parked; no further large-read comparison is performed here.

## C02/C05/C08: repeated snapshots and real invalidation/refresh

The 13 repeat rows are directly exercised. The five mixed changed/unchanged rows are 003 `C49`,
004 `C54`, 005 `C56/C58`, and 006 `C77`; only 004 `C56` is unchanged-only. At 005 `C58`, the
531-byte manifest for the untracked `src/http_controller.rs` is followed by changed/unchanged
components, producing one 6,605-byte delivery. This is not a repeat-free bundle or two handoffs.

For an exact unchanged chain, 004 `S17274` (item
`msg_07ec2d3bbc8dba8e016a932f68b6c887d28ca1570e7b57680f`) requests `src/workspace.rs` and
`src/terminal_app.rs`. `C56`, RPC `55`, accepted at `S17280`, turn
`01a04ef1-3303-7f63-9fef-47478b63ec1b`, delivers 323 bytes consisting of reminders and two
digests, rather than the 40,263/43,792-byte current file bodies. Prompt SHA-256 is
`22351e0c40706ba6fc50291d834ca1efa4e316adfa718200a6f1e532934da673`.
The same thread (`01a04ee8-1ac3-7112-9cb3-dc21d4737ee2`) already received the latter file's
changed refresh at `C52/S17091`, RPC `51`, turn `01a04ef0-411a-7550-bc9d-f6e12a5f2e20`.
Its current digest is the identical `fnv64:01941b7bcff57ad2; bytes:43792`; the refresh prompt
is 5,114 bytes, SHA-256 `4a8af825a29c2b965b3a15e943a4551375b2578fe7e50ab9aa0434c27c265a54`.
These exact representation differences establish a local avoided-full-text input channel, not
an avoided-call count, token price, or proof that the compact context was sufficient.

**Own accepted edits invalidate the tracker before completion.** In 001, author thread
`01a04ebc-2a9e-7662-8f06-5d0cfe7c08fd` receives `src/ui.rs` in `C109/S62117`'s manifest.
`C111/S63223`, RPC `110`, turn `01a04ed6-4e0b-7de2-94fd-342db20643e6`, acknowledges an
accepted edit of that file. Public `S63312` then requests it again; `C117/S63318`, RPC `116`,
turn `01a04ed6-7062-7a32-9e2f-76d78317430e`, supplies another full-snapshot bundle.
No public `done` occurs between that ACK and reread. This exercises own-edit clearing, not
first-ever access or a new provider thread. Other completion/review cycles also reuse a thread
with fresh manifests; they do not isolate completion clearing from intervening own edits.

All **14 actual automatic-refresh deliveries** in the census table contain one changed-file
diff. None exposes the >48-KiB diff omission, >8-KiB untracked-full-text omission, unchanged
status, or unavailable-diff branch. Launch/reviewer copies containing the refresh phrase are
excluded. This H-specific result differs from the omitted-refresh episodes in S/W; those cannot
be imported as H evidence. Requested-repeat diffs and automatic conflict-refresh diffs remain
separate boundaries. C02's current-text intervention can preserve C08 and all tracker mutations;
no omission-threshold toggle is motivated by H's actual refresh exposures.

## C07: mediated command compaction is not exercised in H

All **58 delivered command results** (12/8/10/5/10/13) have a unique archived locked-command
match by the exact wrapped command, exit status, and complete stdout/stderr representation.
Each archived raw output matches its own terminal metadata digest. None exceeds 12,000 Unicode
characters, has a line over 4,096 characters, or has more than eight consecutive whitespace-only
lines. Full nonempty output is preserved, with only the ordinary final newline; empty output uses
the ordinary `<empty>` marker. No `[work-leaf compacted ...]` marker is delivered.

For example, 001 `C32/S3659` matches
`observation/locked-commands/00002676927217163490-13768/`:
`meta.json` SHA-256 `e84f97375d5484665d9e20916006456e61845729749154204f45088c268d9196`,
`stdout.raw` `f37209193ba81aece8cb6aff504b6790830908c80dce29757d53f88fbcb2e6f4`,
`stderr.raw` `b4fa3287fe46415ea4b63e7f4d231d0a4f39d2bbc8fcc8ec205f00e2c75d85b5`.
Additional captured subprocesses without a delivered command-result counterpart are not counted
as extra agent handoffs. Native tool outputs bypass this WL renderer and are not covered by its
nonexposure conclusion. Thus this compactor does not account for H's saving through any delivered
mediated result; the earlier P-only zero-compaction claim is not the basis of this finding.

## C34 and disposition

The two small untracked inline deliveries independently reconstruct byte-for-byte from the
retained task base `c92a0b7060a36eac6db2d869b85e589a7a9480f9` and the ordinary renderer:

- 001 `C16/S425`, RPC `15`: 6,607 bytes, including Cargo.toml (454), the UI-harness example
  (4,199), src/bin/work-leaf.rs (49), src/lib.rs (1,706), and the preserved missing-src/main.rs
  diagnostic. Prompt SHA-256 `972e01355ed1b6155ffdc571be617806a5848acda48be7677f02138c9783dc57`.
- 006 `C22/S575`, RPC `21`: 99 bytes wrapping the same 49-byte src/bin/work-leaf.rs body.
  Prompt SHA-256 `7deaf8345e3c8dff28eff5ff75aac0b02cadccc7334e9fed4aeb1c37fe9edc85`.

These are real small-source inline exposures, not failed large-bundle fallbacks. Initial bundle
deferral, repeat representation, selective native retrieval, tracker invalidation, and conflict
refresh therefore have H exposure; none alone proves a net effect or a share of H's difference.
Adjacent-read merging, normalized path deduplication, mediated explicit-bundle rereads and mediated
command compaction have no exercised saving boundary in this H census. Small inline packaging is
exposed but not established as a saving. The next directional evidence for C02 is the already
specified full-current-repeat factor: owned changed/unchanged repeat representation plus its two
launch-policy coherence sentences, preserving first reads, snapshots and automatic refresh. It is
not identical to the older points 8/9 renderer-only repeat knobs.
No new provider request or benchmark is authorized by this evidence note.

Endpoint preservation check: all twelve evidence-pinned raw streams and all 84 selected archived
bundles were rehashed after inspection, without modifying any retained source or report. The
source-only document affects no agent-facing workflow and requires no new real-agent generation.
