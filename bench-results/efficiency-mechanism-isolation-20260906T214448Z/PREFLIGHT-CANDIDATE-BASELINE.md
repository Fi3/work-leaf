# Saved WL baseline compatibility for the candidate screen

## Result and scope

Read-only review cutoff: 2026-09-07 11:53:49 UTC. The six declared saved controls are
compatible with a same-scope, descriptive candidate comparison on the inspected
driver, task, subscription transport, observer and nonfactor runtime sources.
There is no requirement for a fresh control or Direct run in this check.
The candidate release build, final source freeze and bounded real-agent verification
remain separate admission gates; this note is not their completion receipt.

The scope is `PROTOCOL-CANDIDATE-SCREEN.md` SHA-256
`7b1865f2d7a21af6075a1b40148beb46120f54ecc1b815857f5aa8a0e3f50fdb`:
one complete workflow for each of `requested-repeat-full`,
`unified-diff-preferred` and `review-fix-request-resupply`, concurrently in one
wave, with saved W controls 002/004/006/009/010/012. Only launch order is randomized.
Historical reuse does not provide a fresh randomized control contrast or eliminate
unobserved temporal/provider-side differences.

No provider was launched, no global setting or old report was written, no parked
read-phase costs were inspected, and no token totals, percentages or contributions
were calculated for presentation. The read-phase material used here consists only
of already retained safe trust-provenance metadata.

## Frozen workflow and transport identity

Exact-byte compilation of `runner_work_units.py::verify_manifest` verified all 121
W frozen file identities, the clean driver source and the declared schedule.
W `phases/work-units-01/PHASE-MANIFEST.json` has SHA-256
`5282364e43839e8932abfcec440d07b87467e6e7024f99f66b236e662cbdf950`;
its `score-manifest.json` has
`04f46dfb0492423020a24247731e015d1f61c7f8a0f9d8c214aa95b4603c7543`.

The reusable driver checkout is `infrastructure/driver-source`, clean at
`3c907f7266b63efa1558213f3e49587e999b899d`. Its five scripts and the W frozen
copies have these exact hashes:

| Driver | SHA-256 |
| --- | --- |
| `bench-three-features` | `d2487780c63c14021904b8a3c882d54fe231c5846f4a7f57fe955f50201f5644` |
| `bench-candidate-common` | `2971cb9729626ddc7a12bd688df8d80ee20f904bb48d52ab846c1e9f9751711f` |
| `bench-validation-common` | `235ef644d692098a670c4e61c3572f60f8830c445896f3639a56430c57781dca` |
| `bench-agent-profile-common` | `de9803658bc8ac41a9356436be2c8b8edfce9d7bef9176841e067b3478633d4f` |
| `bench-progress-common` | `8d8977f3d79ad721f09973f883b662f0aaf2f804b522390ff564337398b292f5` |

Task base is `c92a0b7060a36eac6db2d869b85e589a7a9480f9`; exact task-list hash is
`45bee25a4b929182d36612fc5a159597e7770f25dba9c95760713a401d45598a`.
The W frozen `infrastructure/SCORER.json` hash is
`4e35a729f53f030eb1ebf73eaa8f27bd6b66fed67411a94f5339dcfb0562ab70`.
The candidate protocol retains this driver/task/check structure, stage 7200 s,
busy-stall 1800 s, idle-stall 300 s and outer 86400 s limits.

Transport identities are unchanged:

| Component | SHA-256 |
| --- | --- |
| Canonical `bench-results/efficiency-measurement-gate-20260906/subscription-codex` | `0977db361b4477e2bf68ea08c05571a4a35fb1e5c958cc9a953f2cc738de7078` |
| `/usr/lib/node_modules/@openai/codex/bin/codex.js` | `61b0194f3bb6534439c8d26a3ed57d0805f84b884588b761795323eeb92fcf70` |
| Native `@openai/codex-linux-x64/vendor/x86_64-unknown-linux-musl/bin/codex` | `56ef98ab4032d317ab26e9b5e5a175650717351edb16ed9cde0cb6d1734d62da` |
| Release `bench-observer` | `238bdc610a28edd328d91046f9674dbe59ad2f2d7520fedf681c1ae95a56e386` |

The wrapper forces ChatGPT subscription login, OpenAI provider, `gpt-5.5` and
`xhigh`; the captured reports identify CLI 0.153.4 and app-server backend.
`runner_candidates.py` SHA
`ecac7753fb535c59974deedf6a04f0ce11b6e05a19f8c4112416ab7e29b7cb03`
exact-compiles the unchanged supervisor
`runner_work_units.py` SHA
`2f019c297a2e436c54241ad0643d4f105db6ec8bc782b1cac456f3acde0bc98e`.
Its private overrides are the v4 experiment schema, fixed condition list and
three-row plan validation. `runner_work_units.py::run_environment` retains
raw-response metadata 1, interrupt grace 1000 ms/forward, project inventory 1,
normal read permission, the same model/effort, isolated runtime directories and
the same provider/observer invocation. It removes inherited benchmark/auth-key
overrides; it does not copy credentials.

## Runtime source qualification

W build attestation `preflight/BUILD-ATTESTATION-WORK-UNITS.json` SHA
`aefdc03b08740ee1d6ec93a5a9f17251ac6c2d9f565d8e5ac52fb35f8e86534e`
pins compiled source commit `2a69f863a294c224d137120ea0a21834cc88210d`.
W release binary hashes are `work-leaf`
`07c403c5290371a7e1700cb9d03127b9b9a8d384c55a98692115b2fbf01fe1db`
and `work-leaf-orchestrator`
`9f9a6c88a45bd39a99b6c7eeae82e0037575592ed75d8a41a013d3792ade95fa`.
The report-header `benched_binary_commit` names the clean driver commit, not this
experimental runtime commit; the attestation and binary hashes are the runtime
authority.

Of the 31 existing attested inputs, 27 match current bytes. The four differing
inputs and the additional private candidate module are:

| Input | W SHA-256 | Candidate source SHA-256 at cutoff |
| --- | --- | --- |
| `src/agent.rs` | `a6bd797e780b1751026c141cfff508c036ea45af7f37b085336f8a34c8a85838` | `a9e6065a450d05bf299202a2d6f44dd2dae33a3324e8c70e9f91d36a22b242fd` |
| `src/bench_experiment.rs` | `a06ee31112e7b4d8bb13f1a4a0e58063629c7a36caba6be9598e1ebc832a16eb` | `8c9db45eda3a10acabe441259e901617d44bf3d92f800231099ba84036de9a89` |
| `src/cli.rs` | `70d5107a5b022f4dba118e8567a5c2f54330a51319f5d97d84299380e6246c0d` | `e68988f004ac9a61d25c0565b41dd46a371ebcf62a2415c0aabc4d6d83a96763` |
| `src/orchestrator.rs` | `11b1e131b8d6e6f50100575c4d43c5e245243fe501f9280b8f899f197f947ae0` | `027c35a9eaf24e99335001e68f7d0612a262831af80f8677874525db95e9332b` |
| `src/bench_candidate_experiment.rs` | absent from W runtime inventory | `e327d28fb05ceaa8f9c9465a78fc61069eddb56faa767b4909f29f83684d8b94` |

The differences occupy benchmark-gated delivery hooks: policy rendering, requested
repeat-read representation and actual review-fix request resupply, plus private
manifest/evidence support. Normal provider launch/resume, review/linearize
coordinators, locking, patch machinery, title worker, workspace and instructions
sources remain byte-identical. All observer source/Cargo inputs are identical.
`src/bench_read_experiment_tests.rs` is an additional cfg(test)-only input, not
production runtime logic.

The web assets omitted from the old 31-file inventory were independently compared
with `git show 2a69f863…:<path>`; all are identical:
`web-ui/app.js` `355f10869a9661c3fa77e1b420b5b3f08cd16a78394804c364cad05e84a6c3f7`,
`index.html` `19a46e7a42b981968adfc6e047faffe3ef7e72bf5fa0d5e3bdbb5496d8a5edcb`,
`styles.css` `9bdffa2129d1c90708991db903c2165750587233a711841fa5bcd92987965290`.

The final candidate build must retain these comparisons as an explicit attestation.
This cutoff comparison alone is not a claim that an unbuilt or subsequently edited
candidate binary has passed the nonfactor identity/real-agent gates.

## Global and actual project-layer configuration

`runner_work_units.py::config_snapshot` read the existing global file at
2026-09-07T11:44:18.037865+00:00. Only safe hashes and the helper's whitelisted
summary were retained. The whitelisted global summary is `{}`; this is not a
claim that the global file contains no settings. Declared wrapper overrides are
ChatGPT/OpenAI/`gpt-5.5`/`xhigh`, not a general effective-config resolver.

| Identity | W admission | Current read-only snapshot |
| --- | --- | --- |
| Raw SHA-256 | `5f846b530bcbca1095b739b8bd86c977b7775d5426f80b0e6169ac0109809406` | `2487e9d4652b2e52fe3e8862ce4526db592facc4e9eee7b2013a7de210e5df10` |
| Parsed SHA-256 | `e91e44495343edc3d4ced57caa5bb296c01f0447688d27a1577f8d1243a3a389` | `af37b3fe34dcc4f368f6eb7b985953429d7516642c7da59bfb7f5cabf664dbdb` |
| Behavioral SHA-256 | `78cd609bb3fb021622bfb6cf84afd8954c4169d18b7f2f8c4de6156ef0735512` | `c4e6b45b1aea7d54906796b899f5b8271dc731ba02b83ba34864b5bb66c3d5d7` |

The original legacy comparison is `behavioral_or_unknown`, retained unchanged.
A separate in-memory reconstruction removed only the 14 exact project keys named
by the retained proofs below, after requiring every value to equal precisely
`{"trust_level":"trusted"}`. It reproduced the **entire** W parsed-config hash
`e91e4449…`, with no other differences and no raw-file change during the check.
No blanket exclusion of `projects`, no global write and no retrospective
relabeling of the stopped phase occurred.

Proof source identities are W `trust-evidence.jsonl`
`47666e6ab1c5d87b9c1132d1876064ae168cd28c791c18c2aadd62bf375b1be6`,
R `trust-evidence.jsonl`
`0500a33e4d2b121a548632db0a60844e7fa7e43c8a37fe97cc393eb245d35cd2`,
and R `OPERATOR-HOLD-005-TRUST.json`
`41a838326d6870629d3ecd43347d2a3ae10251b5d9a21d5471184c0e7b0c2fce`.
Their exact `trusted_project` keys are the only reconstruction targets:

| Owning workflow | Proof ID |
| --- | --- |
| work-units-01-workflow-001 | `0e15fdeba140bbd5615b0d2a7ae3a37c93afad2034c5410ddbc06e895d1882c7` |
| work-units-01-workflow-002 | `e09a1bb76e87708c578f7c6f738e6c9bd3db0b0238a78b89a6db132ed60c99fc` |
| work-units-01-workflow-003 | `8a564d11c4635669faac163692ad69c862667f69277fdb6c299960232663cf08` |
| work-units-01-workflow-006 | `6fd87e7241401a7dbe6b30b4ef652ba62c1a77dc20b19bc960059af35dcc9780` |
| work-units-01-workflow-005 | `50ef771978c9a079840913817de2bc7108c0528eeaf09d320cf600df1b182a5e` |
| work-units-01-workflow-004 | `3d9b96f518e13d0d9681c0e0d9b0491093b43cb805d7bfd24c3cbc76a4fc1cc8` |
| work-units-01-workflow-007 | `cb624971c592d2e43586e0d28aa1f052f494b7e2c7c7ae21aa28f30ec2810004` |
| work-units-01-workflow-008 | `cce004974c65c2317dc36117f8faeafd44d144a1608cfd2bccbd213ace707dc9` |
| work-units-01-workflow-009 | `e936cd1f777cb680df741f838234fad7b789418533c19603c4e4bcc200664198` |
| work-units-01-workflow-012 | `74fcad4865365cbee877e65959d234e2d14d601ce74c8f5152af5de50e603f61` |
| work-units-01-workflow-011 | `0a2775ef62d6a21b6ed0624f0b5b7aeabd1df4c86faddbfd1d8b3e7e0bf50b21` |
| untracked-reads-01-workflow-001 | `3d7f9a8698b0ba46a7b42b70e18c051b13db6ed1daa0f1b776eea0d5415f06f6` |
| untracked-reads-01-workflow-006 | `b203b1a75b7a5023d954329ca464111b024059fbaf764e4b61e1dfc6bfec088a` |
| untracked-reads-01-workflow-005 | `c6a906cf8edd42a29d8b7a09ab9133931d654e848253baf25c2ac7be6e6933cc` |

The R005 evidence is a separate factual operator-hold proof, not uninterrupted
supervisor monitoring or an exception that turns R's final failed trust audit
green. Later independent full reconstruction requires the matching external global
configuration bytes; this safe note does not archive those bytes.

For all six W controls, retained actual project-layer inventories are valid and
have digest
`08b328252e86541a1d1343ad84f1e599eb6a2bb9fa9334a0564ed04e3eb29370`.
Controls 002/004/006/009/012 retain base, pre-spawn, pre-linearize and final
snapshots with identity; failed 010 retains only base and pre-spawn. The source is
each row's artifact `observation/project-layer-inventory/manifest.jsonl`.
These checkpoints are not proof of continuous immutability. Fresh candidate
pre-spawn inventories must match the same eligible layer state before the child
starts; unrelated old trusted paths are not eligible ancestors of those fresh roots.

W's closed external-global replay remains preserved at
`phases/work-units-01/coding/operational/FINAL-GLOBAL-TRUST-REPLAY.json`, SHA
`4f06fd3001838f620ec945b806cbd37cc95b79f9e1988f9b14318bd6b4706a87`.
A replay requiring that old final external raw hash must not be falsely described
as succeeding against a later global file merely because the model is unchanged.

## Existing accounting: actual six-row replay

One provider-free `accounting_untracked_reads.py::audit_run` invocation per saved
control completed successfully. It retained the genuine failed workflow 010.
All six have errors `[]`, finite bounds for `raw_input_plus_output` and
`uncached_input_plus_output`, and supported isolated normal-response tails.
None required an omitted-compaction correction, unusable-last warning or late
terminal recovery. These are bounded measurements, not exact-completeness claims.

| Run | Exit | Retained outcome | Audit / measurement | Retained gaps | Hashed sources |
| --- | --- | --- | --- | --- | --- |
| work-units-01-workflow-002 | 0 | recorded_workflow_success | validated / bounded | 5 | 23 |
| work-units-01-workflow-004 | 0 | recorded_workflow_success | validated / bounded | 5 | 23 |
| work-units-01-workflow-006 | 0 | recorded_workflow_success | validated / bounded | 1 | 23 |
| work-units-01-workflow-009 | 0 | recorded_workflow_success | validated / bounded | 3 | 23 |
| work-units-01-workflow-010 | 1 | recorded_workflow_failure | validated / bounded | 3 | 21 |
| work-units-01-workflow-012 | 0 | recorded_workflow_success | validated / bounded | 5 | 23 |

The following are hashes of each audit's complete source-identity map, using
sorted-key compact JSON. Original report/capture/native source paths and hashes are
the map inputs, not invented aggregate response records:

| W suffix | Source-identity-map SHA-256 |
| --- | --- |
| 002 | `27cb79a1e59f0e7a8eb6b7fca2a889be3cbd814507fa0488e7c647782aedc731` |
| 004 | `752048bbd8cb0c123eff41e2a5c091b9fd7ed1c9a2a4ffd5a9274fec85968069` |
| 006 | `4150994bd33b0a8327d149028f434ac5de978e43c3323c5cc3d074a3b2fc0aee` |
| 009 | `c7c66f4d20813c8c723ffd26042c2c25fd6c932c26d54ac7cc43fd09aee6d6ec` |
| 010 | `19ec9391ef814384620ff8ba337d41a03a91243ac6f1b7ff75c1658ea194f992` |
| 012 | `f91ba4e59fbeb63521a28c6d633aa5128bdf70347259d99d0545da79d18704bb` |

The helper SHA is
`c365aa86ed956292f628d31ea79603196750744e5ecaec7e18ef1224b2a8d56a`.
Its exact-byte compiled dependencies are:

| Dependency | SHA-256 |
| --- | --- |
| `bench-results/efficiency-measurement-gate-20260906/batch_analysis.py` | `dacbfc8416467312c8a447ac1cd846da3e1f78da96f3733ee16c3dad1781d7c3` |
| Sibling `analyze.py` | `2bb28891a2e158d51cf577bcf7c1fc2781065e38dc5ec4c6c30f7d4c146f2f78` |
| Sibling `audit_compaction.py` | `dd5127f937ec1cc4d41a443ba7640fca93e82484f345ac8c56aeef5325d68153` |

`audit_run` has no intervention-condition dependency: its
`work-leaf-read-identity-accounting-v1` output schema names the accounting scope,
not a requirement that a workflow ran the read treatment. Its caller owns phase,
source/config and intervention-exposure gates. `reconcile_stream` independently
checks response identities/arithmetic, observer/native scope, named compactions
and the restrictive gap proof rules; it does not label nonadditive last metadata
as fresh billed usage or recover gaps from elapsed time.

The helper postdates W and is therefore **not** a W admission dependency.
Passing the original W manifest unchanged as the helper's `frozen` argument would
correctly fail its unique-helper-identity gate. This preparatory replay used a
separate in-memory analysis scope containing the exact helper pin, model/effort
and the original W manifest hash; it did not amend or impersonate W's admission.
The original W entries and source reports were read unchanged.

## Minimal common-scope invocation after candidate closure

No new accounting framework is necessary. Freeze this unchanged helper and its
three pinned dependencies in the candidate evidence tree with their original
repo-relative layout. After all three candidate outcomes are terminal, use one
separately named, create-new analysis artifact, not an old report rewrite:

1. Verify the candidate frozen manifest and its phase/config/exposure gates with
   the v4-aware source; retain the W admission/closure evidence separately.
2. Read the frozen helper bytes, require exactly one matching
   `role == "frozen-evidence"` file entry, and `exec(compile(bytes, path, "exec"))`
   in a module whose `__file__` is that exact frozen path. Do not import unchecked
   cached bytecode.
3. For each of the six unchanged W score-manifest entries and three new candidate
   entries, call the same `audit_run(entry, candidate_frozen_manifest,
   Path("/home/user/.codex/sessions"))`. Keep original run IDs, report/artifact
   locators and outcome receipts; do not turn them into v3 experiment rows.
4. Retain the complete returned diagnostics/source map, original observer ledger,
   exact response evidence, scope corrections and finite/unbounded/ineligible
   measurement state. Reject duplicate response IDs across distinct workflows;
   the per-run helper cannot perform that cross-run check.
5. Rehash admitted inputs and returned source identities at the endpoint. Write
   only the new analysis artifact with create-new semantics, preserving every
   failed/unknown row. Descriptive candidate-versus-saved-baseline intervals use
   the same raw metric and bounds, not a midpoint, silent zero-fill or current-price
   conversion.

The raw metric is input plus output, including cached input; reasoning is an output
subset, not an extra charge. Missing usage remains an accounting uncertainty, not
a demonstrated saving. The helper explicitly reports
`whole_workflow_hidden_call_completeness_proven: false`; it proves the captured
and reconciled identity scope, not absence of every possible hidden provider call.
Finite bounds depend on the existing supported tail rule. A new candidate with
unsupported tails must remain unbounded/ineligible, even though all six baseline
replays are bounded. No evidence here authorizes relaxing that rule.
