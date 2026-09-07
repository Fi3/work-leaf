# Context mechanism evidence

This Stage B pass follows [the active plan](PLAN-CANDIDATE-INVENTORY.md) and
[C01–C09, C31, C34–C35](CANDIDATE-MAP.md). It uses only the closed September 6 pilot (P),
three-workflow screen (S), twelve-workflow work-unit phase (W), and previously recorded bounded
diagnostics (D). The earlier accepted six-pair cohort (H) is not silently substituted by these
cohorts. Parked read-factor outcomes/costs (R) are not analyzed. No new model generation, token
retotal, percentage estimate, outcome exclusion, or frozen-artifact change is part of this pass.

## Scope, provenance and physical locators

The new census reads each original client/server stream once per extraction, indexes strict typed
RPC replies and public completed `agentMessage` items by thread/turn, and links each accepted
`turn/start` to its unique successful reply. Every inspected start has one text input and a
nonempty accepted turn ID; no start is discarded as unverified. All 273 file-text handoffs have
exactly one public read directive in the preceding accepted same-thread turn. Its paths are
ordinary relative paths without parent components or quoting; normalization agrees with
`FileLockTable::normalize_relative_path`. This avoids inferring coalescing from file counts or
counting a quoted directive inside supplied source as a generated request.

Only public agent messages, user prompts, request/reply metadata and public native tool items are
projected. Private reasoning bodies are not inspected or reproduced. Stream identities and hashes
are checked against the existing attribution reports; no attribution arithmetic is rerun:

- P: [HISTORICAL-INPUT-ATTRIBUTION.json](preflight/HISTORICAL-INPUT-ATTRIBUTION.json),
  SHA-256 `586533b10c4ffa94b9b2dbaf5ebf91762f8ea103b500238ae6b6e1265f0100c6`.
- S: [INPUT-ATTRIBUTION.json](phases/screen-01/INPUT-ATTRIBUTION.json),
  SHA-256 `7cfd44d2427394bdb143f0bb01478672d93d66be011c9f54408aef6c7ac8229b`.
- W: [INPUT-ATTRIBUTION.json](phases/work-units-01/INPUT-ATTRIBUTION.json),
  SHA-256 `ed171a20cec3017a75d84219ce81f759725e86b95f3494daedc27868cecf92cd`.

Each report's `runs[]/source_sha256` supplies the exact absolute source paths and expected hashes.
All 32 raw stream files match those recorded hashes. Native files used below independently match
the P source inventory. Unsupported whole-workflow accounting in those reports remains unsupported;
passing a source hash or delivery join does not repair it.

In the table, `C<n>` means physical line n of `client-to-server.raw`; `S<n>` means physical line n
of the companion `server-to-client.raw`, not the screen cohort. Every listed RPC ID is a string.
Paths resolve as follows:

- P capture root: `../efficiency-raw-token-pilot-20260906T185328Z/runs/work-leaf-001/work-leaf-001-three-feature-bench-artifacts/observation/app-server/<invocation>/`.
- S root: `phases/screen-01/runs/screen-01-00n/screen-01-00n-three-feature-bench-artifacts/observation/app-server/<invocation>/`.
- W root: `phases/work-units-01/runs/work-units-01-workflow-nnn/work-units-01-workflow-nnn-three-feature-bench-artifacts/observation/app-server/<invocation>/`.

These are complete accepted file-text, automatic-refresh and full-review-context locator lists,
not selected successful examples. W010's failed terminal outcome remains included with its actual
two review-context launches; there is no invented third reviewer.

| Cohort/run | Invocation directory | File-text C lines | Automatic-refresh C lines | Full-review-context C lines |
| --- | --- | --- | --- | --- |
| P work-leaf-001 | `00000422153032163608-2899396` | 9,12,14,16,18,20,33,37,54,60,74,77,81,84,94,99,112,118,126,128 | 35,96 | 31,58,72 |
| S S001 | `00000433355029717882-3253359` | 10,12,14,16,18,20,22,30,41,48,84,122,138,141,144,158,171,191 | 46,50,82,152 | 38,114,136 |
| S S002 | `00000433355029374750-3253355` | 10,12,14,16,18,20,22,24,35,52,56,58,61,67,73,83,87,99,111,115,133 | 45,54,113 | 33,50,81 |
| S S003 | `00000433355029867482-3253361` | 8,12,14,16,18,20,22,24,26,37,43,53,66,69,79,83,95,105,108 | 71 | 35,64,103 |
| W W001 | `00000441344929586777-3537090` | 10,12,14,16,18,20,22,24,26,37,50,63,73,76 | 52,61 | 35,48,71 |
| W W002 | `00000441344930516444-3537099` | 8,12,14,16,18,20,22,33,43,49,57,67,81,83 | 61 | 31,41,79 |
| W W003 | `00000441344929587964-3537091` | 8,12,14,16,18,20,22,33,52,56,58,73,76,86,89,98,111,117,125,138,147,163 | 38,42,54 | 31,50,71 |
| W W004 | `00000446260578462469-3719532` | 10,12,14,16,18,20,22,35,41,58,69,76,94,106 | 71,74,78,96 | 33,56,104 |
| W W005 | `00000446260581883146-3719549` | 10,12,14,16,18,20,22,28,39,62,66,69,93,96,110 | 48,64 | 37,60,91 |
| W W006 | `00000446260578466031-3719533` | 8,12,14,16,18,20,22,24,35,51,56,72,75 | 53 | 33,49,70 |
| W W007 | `00000448924886790377-3844443` | 10,12,14,16,18,20,35,48,51,57,65,75,83,95 | — | 33,46,73 |
| W W008 | `00000448924902156465-3844489` | 10,12,14,16,18,20,22,24,37,41,44,46,56,66,70,85,87,89,91,93,95,97,100 | 35,39,58,68 | 33,54,83 |
| W W009 | `00000448924886506612-3844440` | 10,12,14,16,18,20,31,52,57,69,81,84,94,97 | 33,38,54,71 | 29,50,79 |
| W W010 | `00000452179764996037-4005148` | 9,12,14,16,18,20,22,24,26,51,61,64,69,77,90,98,101,116,119 | — | 49,59 |
| W W011 | `00000452179770293816-4005169` | 10,12,14,16,18,20,22,24,35,58,62,69,77,87,89,98 | 37,48,60 | 33,56,75 |
| W W012 | `00000452179765332046-4005151` | 10,12,14,16,18,20,31,35,38,48,60,64,69,87,99,104,119 | 33,50,62 | 29,46,97 |

## C03/C04 and explicit-bundle part of C05: named-cohort nonexposure

P has 20 accepted file-text handoffs, S has 58, and W has 195. Across those 273 boundaries:

- No preceding accepted same-thread turn contains two read directives to coalesce.
- No read directive contains duplicate normalized project paths.
- No read directive requests an explicit owned bundle path. Native tools opening bundles are a
  different route and remain present.

Thus adjacent-read coalescing, within-request path deduplication, and mediated explicit-bundle
read-back cannot explain savings *through an exercised delivered boundary in these cohorts*.
This does not establish their absence from H or from arbitrary workloads. Multi-file single
requests are common and are not evidence of adjacent-directive coalescing.

Concrete positive-link control: P `S54`, item
`msg_0c79ac7c5de691aa016a9dbbb37edc87d2860d2228b44931ce`, has one read directive requesting six
distinct paths. It leads to `C9`, string RPC `8`, accepted at `S64`, thread
`01a07825-10a9-77b3-bd6a-f75800663294`, turn `01a07825-37bf-7793-b533-674e10dc8d7a`.
The six manifest paths correspond to that request; one multi-file request was not six exchanges.

**Disposition/next:** nonexposure for these P/S/W delivery populations; inspect H's available
public requests before considering either runtime toggle. Do not create redundant requests merely
to activate a favorable mechanism.

## C05/C08: exercised invalidation and refresh interactions

The refresh census finds 2 accepted automatic-refresh messages in P, 8 in S and 27 in W.
All contain one file section. Of these, 35 carry a changed-file diff; two have no prior tracked
snapshot and omit its full text because it exceeds 8 KiB. None exercises the >48-KiB automatic-diff
omission, unchanged-refresh status, or diff-unavailable branch. These are delivery counts, not
counts of independent defects or estimated avoided responses.

**Own-edit invalidation is exposed before completion.** In P, the same author thread
`01a07825-10a9-77b3-bd6a-f75800663294` receives `src/ui.rs` and `tests/ui_harness.rs` in
`C9`'s bundle. `C44` acknowledges an accepted edit of both paths (string RPC `43`, reply
`S23625`, turn `01a0782f-2108-7bf0-b7dd-070a33e4b829`). The next read of those paths at
`C54` delivers another untracked bundle, not a diff (RPC `53`, `S23889`, turn
`01a0782f-8ace-7780-b10d-99c1939aef16`; request item `S23882`). No intervening public
`done` occurs. This matches the accepted-edit `clear_files` branch rather than first-ever access.

**Completion/resume is also exposed.** The author emits `done` at P `S25064`, item
`msg_0c79ac7c5de691aa016a9dbeb09ad887d29b67036d75c85064`, turn
`01a07830-dbad-7ce2-8bf0-a5e9386b4f16` (the `C68` turn). Reviewer findings return to that
same thread at `C82/S26981`. Its next request `S27101` leads to `C84/S27108`, another
bundle for the same two paths. This does not prove a causal token effect or isolate completion
clearing from all intervening own edits; it proves that provider-thread reuse does not mean the
file-read tracker remains populated.

### Two omitted-refresh → unchanged-read chains

| Cohort | Exact accepted refresh | File and held digest | Subsequent public request and accepted response |
| --- | --- | --- | --- |
| S001 | `C82`, RPC string `81`, `S34670`, turn `01a078e1-381e-7650-91b0-3723aacd7cac` | `tests/terminal_app.rs`, 47,516 bytes, `fnv64:18b22c81c7a6f37f` | `S34999` requests that path; `C84`, RPC string `83`, `S35007`, turn `01a078e1-4e18-7293-8476-70651584e6f6` returns unchanged status only. |
| W004 | `C74`, RPC string `73`, `S42165`, turn `01a079a6-d294-7223-8f03-ee646059ce1b` | `src/terminal_app.rs`, 47,372 bytes, `fnv64:ffb1405626dce66c` | `S42322` requests that path; `C76`, RPC string `75`, `S42333`, turn `01a079a6-dcbb-7313-bea6-b8dbe170e143` returns unchanged status only. |

S001's thread is `01a078d0-2539-77c0-87e4-261188ee933a`; W004's is
`01a07995-1600-75c1-8f05-508bedca97f9`. In both, the rejected-refresh prompt explicitly says
the current full text is omitted and instructs a mediated read if needed. The immediate read
returns the identical digest but says the exact text was already sent. The first refresh is
1,263/1,254 bytes and the repeat message is 268/266 bytes, respectively; these are prompt bytes,
not token prices or evidence that the missing source was unnecessary.

The source explains this exact sequence: the conflict branch records every held snapshot after
rendering the refresh, including omitted text; `split_repeated_file_reads` then sees its unchanged
digest. It is not a snapshot-consumption tracker. This observation neither asserts that the agent
never knew an earlier version nor infers a quality failure; it retains an actual omitted delivery
and the extra request that did not restore full text.

Refresh prompt SHA-256: S001 `6ea6329db00b12177578c64c721bd4b91c85e281cc455aaf4168793e514a81a9`;
W004 `33ca2618639bd91ebcebb8cc0555080ed12dea5bb41f77a39de8277ad10789f8`.

**Disposition/next:** C05 state transitions and C08 compact refresh are exercised, with possible
saved input and recovery overhead. They are not demonstrated net savings. A C02 full-current-repeat
factor can affect the second handoff without modifying automatic refresh; a separate C08 test must
leave actual rejection and tracker behavior fixed. No >48-KiB-diff toggle is motivated by exposure
in this census.

## C09: native instruction duplication is actual in P, not merely hypothetical

Every one of P's eight WL native sessions has the provider-added user instruction item at physical
line 4 and WL's policy-injected launch item at line 7. Both contain the identical 10,938-byte
repository-instruction body. Every one of P's seven Direct native sessions has the provider-added
copy, but no second explicit WL copy. The comparison covers the complete recorded P session
inventory, including auxiliary WL work; it does not generalize to other CLI versions or H.

The compared body is extracted between the provider's `<INSTRUCTIONS>` markers and, separately,
between WL's `--- AGENTS.md ---` delimiter and `Agent-ID:`. Both extracted byte strings have
SHA-256 `5f6ba65e9708299c97fa6856d6eb4e9ac09434ca19d7add22b90862fb20d652c`.
This is equality of the actual supplied instruction body, not a similar-heading match.

Representative WL source:
`/home/user/.codex/sessions/2026/09/06/rollout-2026-09-06T21-14-50-01a07825-10a9-77b3-bd6a-f75800663294.jsonl`,
SHA-256 `7fbac978405e8b800da2a86bfe44e52f8684c97d483dcc168f196718937756c5`.

- Line 4: user item `msg_01a07825-1923-7721-91fb-ab686da263a0`; whole-text SHA-256
  `0abf23efba417d7bd38d5e9f104c9dd514c1875ee246e64a8d264e317e929679`.
- Line 7: user item `msg_01a07825-1940-73d3-9991-e44a1fe6e16a`; whole-text SHA-256
  `14975b677e3c7e88328303977bba56708b3fc4f84b922a0cf1d22d1f955393f5`,
  exactly equal to accepted P `C4` launch text.

The saved attribution ledger links both items to the first exact response
`resp_0c79ac7c5de691aa016a9dbbad171887d2a847b3546c0ab358`, P server physical line 55.
Its recorded whole-item input charges are 3,854 and 3,963, respectively. Each whole item is also
listed as charged in 45 completed responses. These existing item records establish an actual
retained-input pathway, but do **not** assign either whole-item charge to the duplicated substring.

Representative Direct source:
`/home/user/.codex/sessions/2026/09/06/rollout-2026-09-06T21-14-50-01a07825-1052-7cd0-a87e-507552c13d67.jsonl`,
SHA-256 `301f3c9d4b3b8ea9bf063bfa549f8ad1767e40757b05ba7abb41827f94b3474f`.
Line 4 item `msg_01a07825-1923-7541-b0e4-b7c329cdcbfe` contains the same instruction body;
line 7 item `msg_01a07825-1949-7233-b59f-10af89c7786c` is the Direct task prompt, without a
second copy. Direct has no exact per-item attribution ledger here, so no invented Direct price
is supplied.

**Disposition/next:** exposed WL input overhead, not a saving. An equivalent-information
packaging test would remove only a verified redundant body while retaining its native copy and
WL-specific semantic translations. It cannot assume the provider always supplies that body;
the effective instruction layers must be verified for every admitted run. This is distinct from
removing validation or repeated-read guidance.

## C31: exact repeated reviewer metadata, not only repeated field names

Every full-review-context prompt listed in the locator table contains the same complete
`Latest commit / Feature / Reason / Review scope` block twice: once in the outer review request
and once in the host-collected `Git metadata` section. Byte equality of the two extracted blocks
was checked, not just a count of headings. There are 3 such accepted prompts in P, 9 in S and 35
in W. The W010 failure remains included; no missing reviewer is zero-filled.

P `C31` is accepted at `S7723`, string RPC `30`, thread
`01a0782b-8100-71f1-a935-ca076774eabb`, turn `01a0782b-81ad-7010-b9eb-497aa20a1d7d`.
Its repeated block is 886 bytes, SHA-256
`af6da2cd0ff1abd0d9a52783eb5bb58306fca4c9b377d5ed366a98e6d5757edb`.
The complete prompt is 47,976 bytes with SHA-256
`5114f99057db290f203b7abd5bba14c6b118eec6e3faf18d73f5b64078cd81ac`.
P's other duplicate blocks occur at `C58` and `C72`; all remaining physical locations are in
the table. No whole-review-context token price is assigned to this one repeated block.

**Disposition/next:** demonstrated duplicate delivery and an exposed WL overhead, not a net-saving
mechanism. Removing one exact metadata copy is a narrower directional test than lazy delivery of
all recorded evidence (C21). Preserve the remaining copy, review scope, evidence, routing and
policy. The two effects would overlap and must not be added as independent savings.

## C01/C02/C06/C34/C35: retained findings and bounded new checks

| Candidate | Concrete evidence and disposition | Next directional boundary |
| --- | --- | --- |
| C01 | Existing [read-representation exposure](preflight/READ-REPRESENTATION-EXPOSURE.md) establishes 18 P manifests and 160 W manifests. Existing [D review](preflight/READ-INLINE-PRIMARY-DIAGNOSTIC-REVIEW.md) establishes exact representation/item-charge/recharge plumbing with retrieval prohibited. Local mechanism, not natural-workflow net saving. | The already collected large-untracked treatment remains parked. Do not recreate it or import its cost comparison here. |
| C02 unchanged | S001 `C84` and W004 `C76` above are exact unchanged delivery despite an immediately requested current file; D independently verifies unchanged-item charge plumbing. Existing W exposure includes unchanged/mixed requests. | Return already-held current repeat snapshots at the ordinary requested-read boundary; keep automatic refresh, first reads and tracker state fixed. |
| C02 changed | P `C94/S30366` (RPC `93`, turn `01a0783a-5608-7d33-a208-a787412b76af`) returns changed diffs after `S30359` requests five files; P `C126/S39788` similarly serves four repeated files. Existing exposure note retains all nine occurrences and their actual item identities. | Separate changed from unchanged exposure. Do not impose an inline-size eligibility cap that excludes every audited P/W repeat. A full-current representation can grow context substantially and induce compaction/failure; retain those outcomes. |
| C06 | The real P tool-output/prefix check below establishes actual selective returned text, not merely a bundle-path mention. Both Direct and WL have native tool truncation in the existing command-output note. | Inspect returned content and follow-up work, not command names or assumed disk I/O; changing permission is broader than representation. |
| C07 | The existing [command-output census](preflight/COMMAND-OUTPUT-MECHANISM-EXPOSURE.md) proves zero removed output in all 14 P mediated command results. This pass does not retotal or expand that conclusion to H/S/W. | No compactor toggle motivated by P exposure. Blank-run, long-line and total-stream limits are distinct prospective subfactors. |
| C34 | Exact eager-inline starts in W are W002 `C18`, W008 `C91/C95`, W010 `C24`, W011 `C22`. They retain small untracked text; some prompts also have unrelated repeat sections. P has no small-untracked-inline response. | Isolate untracked formatting/threshold choice separately from C02 and preserve above-threshold bundle-failure fallback. |
| C35 | The archived P bundle and actual consumed prefix below agree byte-for-byte. This is a concrete captured-version witness, not proof of every file version throughout the workflow. | Preserve snapshot identity when comparing representation. A freshness intervention changes what source the agent sees and is not merely a token-format switch. |

For C02, W001 `S29194` explicitly requests `--force`; accepted `C63/S29201`, RPC string
`62`, turn `01a0795a-d141-7cc0-a283-b23162153c85`, still contains changed/unchanged sections.
This is observed force behavior, not only source inspection. The actual prompt hash is
`5ca302c76a59278d22b2097ceda1e9afb27a6b408014997c7320ec63d37493a1`.

### Actual retrieval and archived-version witness

Use P's first manifest `C9/S64` and the WL native source cited under C09:

- Native line 27 is `exec_command`, call `call_sI2MBJG1qlOYRi5kcwHCxszH`, requesting
  `sed -n '1,220p'` of that exact issued `bundle-0.md`.
- Native line 30 is its matching output item
  `fco_01a07825-6e80-7383-acfa-13ab6676391c`, exit 0. Both carry explicit turn
  `01a07825-37bf-7793-b533-674e10dc8d7a`.
- Its output body after the native `Output:` wrapper is exactly the first 220 physical lines of
  the archived bundle: 13,099 bytes, SHA-256
  `a7bb6d915251126b263a969e1cd1b03af17bb4f873a55bdf912fed6292d36eb4`.
  The complete native output item is larger because of its wrapper; these are not token counts.
- Archive relative to P's observation root:
  `context-bundles/files/orchestrator-2899296-0/bundle-0.md`, SHA-256
  `740b7050b013136787a09e7a1a0462450cf3fde43cc53847fb7f8e3f6174697d`.
  `context-bundles/manifest.jsonl:1` records the same hash and exact original issued path;
  manifest SHA-256 is `5a1e80f3295b4da72f260fcc0bdd9a5b58c1f6ae8c1f641a705c0dad3fced79f`.

The returned prefix ends within the first file: the archive's architecture-document BEGIN/END
markers are at lines 5/435. Thus this *particular* retrieval did not return the whole first file,
let alone the entire bundle. It does not establish that later retrievals collectively remained
partial or that all archive bytes were immutable at every intermediate instant.

## Prospective C02 isolation requirements

The minimal renderer boundary uses the already separated changed and unchanged snapshots, not a
second filesystem read. Construct ordinary untracked rendering and ordinary bundle allocation once.
Splice only repeat-owned representation; preserve explicit-bundle prefixes, untracked eligibility,
bundle paths/counters, failures, tracker advancement/invalidation, automatic refresh and provider
timing. Never move repeated files into the ordinary untracked bucket to obtain bundling.

Replace repeat-section claims that only diffs/status are being delivered with truthful full-current
framing. If launch policy's repeat-only descriptions are also made coherent, identify the factor as
the requested-repeat delivery contract, not byte-only formatting: those descriptions can influence
requests before exposure. Keep the compact automatic-refresh policy and limited/relevant-reread
guidance separate. `--force` is not a secretly repurposed experimental switch.

No local byte reduction is promoted here to a whole-workflow saving, and no duplicated item's whole
charge is a counterfactual price for one substring. Any directional model experiment requires the
separate approved fixed-count admission and existing-baseline compatibility; this document launches
none and supplies no contribution percentages.

