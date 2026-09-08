# C08 exact-cut preadmission review

No remaining preadmission source/configuration blocker is identified at the cut below. This is a read-only preadmission review, **not admission or real-agent qualification**. The reviewer implemented the C08 runtime and the earlier harness draft; this record does not independently certify that work. It checks the final configuration, executable/source closure and final other-owner harness/guard integration. The separate runtime and guard reviews retain their own scopes.

## Exact identities

| Artifact | SHA-256 |
|---|---|
| `FINAL-SOURCES.json` | `4194034619c417eadb0a3a8f387c85eef83f8cd0ab49a6cc1c169b8a6778a0d3` |
| `FINAL-COMMAND.json` | `0ffe481308acaa0b4d12eb9c5fca596f8df7cb1569c480abb6d662b0dc05700c` |
| `ENVIRONMENT-PROPOSED.json` | `d7a2c79f8efc659ebc415e090f7daea3c12a9e8d50a353e729cf29d7e659db2c` |
| `PREPARATION.md` | `8f1ad49abca89051f3c0e86d41b0cd76103b8286bf4093258a38477bd77c097f` |
| Real diagnostic ELF | `061a601e830c02021a7514794c26efcc4899a71a3101735a5b7208aae80fbf8b` |
| Final `diagnostic/harness.rs` | `1c3a210418d280b6ef993c1a9d4d18b02d866508d036c710c38497cf18c53f31` |
| Final `diagnostic/guards.rs` | `3bc5cbda8042aab4b70d92d6bc9cd04385180b657317c41d8d191fee46e62785` |

All **151** declared endpoints independently match exact bytes, SHA256, canonical resolution and declared symlink/target identity. All54 build-to-published source pairs independently match. The endpoint roles are146 immutable, four initial-project and one configuration. The final source-list bytes also match after that check. Earlier149-endpoint verification is retained in tool `5d0bba`; the final recheck includes the two additional explicit executable identities.

The concrete closure finding was that `harness.rs::git` and `GitPatcher` invoke `git`, which the prospective PATH resolves to `/usr/bin/git`, but the original149 list omitted it. The final151 list binds Git SHA256 `292115a21c70326a0fa239e6d9bdee32f750cbd23d12fdef2e518ad4b451a8b3` and explicitly records `/home/user/.cargo/bin/rustc`'s `rustup` symlink. Its resolved rustup and actual stable cargo/rustc executables were already pinned. No runtime source or command changed to close this dependency finding.

The actual executable is the regular ELF `/tmp/c08-real-qualification.kWTyzw/automatic-refresh-diagnostic`, mode0555. Build cut `e497ff5df5bf84e169aa4460953383639df956f8` and published cut `d0a22e78326fccd60e1639fef34da75cfca8947c` have the54 attested source pairs. This review did not rebuild or execute that ELF. Root's retained source-qualification receipts report546 repository tests, seven private tests and53 default checks, with clean formatting and Clippy; these are not newly executed independent checks here.

## Fresh state and environment

The canonical fresh project is `/tmp/c08-real-qualification.kWTyzw/project`. Its four tracked files are exactly `.gitignore`, `Cargo.toml`, `Cargo.lock`, and `src/lib.rs`. Their bytes equal the declared dependency-free guard fixture and `/target/` ignore rule. Git status is clean and HEAD is `6d7fc01a063f4dcd88204d3125cf86bcded0fbd7`, matching the observer's actual `base_commit`; `experiment_commit` separately names the C08 build source. The sibling ordinary-bundle parent is canonical, separate and empty. Evidence is outside the project.

Before publication of this review, no admission, HARNESS attempt/result, mutation attempt/acceptance, call receipt, prompt trace or terminal receipt existed. Observation contains only its configuration/manifest and two proxy executables; no app-server or invocation directory exists. These are prelaunch facts, not promises about postlaunch task state. Initial project files are intentionally mutable after launch, the global configuration is monitored separately without a waiver, and immutable endpoints must retain their declared identities.

The proposed PATH resolves sh/codex to the owned observer proxies and bash/env/node/timeout/git/cargo/rustc to the attested chain. The unchanged subscription wrapper removes API/endpoint overrides and forces ChatGPT login, OpenAI provider, gpt-5.5 and xhigh. The harness independently requires the forbidden names absent before provider creation. A names-only ambient check found `OPENAI_API_KEY`; the declared `unset` list removes it and all seven other forbidden override/instrumentation names before the child starts. No value, authentication file or credential was read/copied; no CODEX_HOME override is present. Root must apply the exact environment rather than merely inherit the ambient shell.

Observer settings retain raw-response capture,1000-ms usage grace, output-resume forwarding and project-layer inventory. Explicit proxy-bin PATH instrumentation is declared, not treated as a causal environment-equivalence claim. The ordinary C08 read/edit/patch/check/done path does not use v6 test-preview syntax. Original observer failures and any postcapture source gaps remain reportable.

## Call-chain and bounds

The complete final harness, guards, five guard cases, actual-wrapper cases, proposed diagnostic scope, amended design, operator policy and relevant unchanged D3 Budget/settled functions were read. No Cargo, command executor, provider probe, collector or accounting operation was run during this review.

`BoundedBackend::before` admits at most eight real launch/send attempts for the prepared author before forwarding; D3 `Budget` never enters review phase here. Other author identities, extra launches and nonstreaming calls fail. `diagnostic_chat` uses ordinary default CommandChat rounds and a30-second locked-command timeout. The real model receives the natural task, no test body, forced reply or prescribed repair patch.

The single mutation requires a real prior sole read and the exact whole ordinary old-text delivery. The wrapper retains the unchanged prompt, verifies initial bytes/clean state, and uses normal `GitPatcher::apply_edit` with distinct `qualification-host-mutator` identity. Acceptance/source/commit evidence is retained before its postcondition assertion. The read-lock release and original record-after-send ordering are established by the runtime source, not a timing sleep or synthetic tracker. The fixture uses a separate lock table and admits no concurrent writer. Natural benchmark workflows must not inherit this manufactured stimulus.

A240-second watchdog shuts down a cloned CommandChat handle. The literal outer argv is `/usr/bin/timeout --kill-after=5s 300s <pinned ELF> --ignored --exact real_subscription_automatic_refresh --nocapture`. The observer stop has its separate10-second bound and the original exit/stdout/stderr is preserved. Create-new publications are not a multi-file transaction; failed startup or partial publication remains a retained failed attempt. No automatic retry or replacement is authorized.

`recovery_chain` requires one owned eligible full-current trace span, exact non-target bytes, ordered actual local input occurrences, a later ordinary ACK, the exact focused status-zero non-timeout check, and sole final DONE. The harness additionally requires CommandChat's processed-DONE transcript, real mutation acceptance, fixed-call settlement and observer closure. Repeated equal ACK texts consume distinct input occurrences. Indexed queues avoid an unbounded quadratic join; remaining whole-input scans are bounded by the eight-call scenario. No general peak-memory or hostile-concurrency guarantee is inferred.

These predicates establish only local harness facts. After the sole admitted attempt, source/public/native full-input and lifecycle joins must still verify the old read, genuine rejection, actual full-current delivery, repair commit, executed check and processed completion. Native actions are not assumed absent when public tools are absent; usage-less threads, compaction and unknowns remain retained. Final behavior and author-written tests require separate semantic review. Exit0 alone cannot establish real-agent readiness, a natural benchmark effect or token savings.

The separately selected unchanged `preflight/c15-real-diagnostic/bounded_launch.py` was read completely (SHA256 `f181729b5c26e4cfc85477c82918fb9468fb4a9482d8700c36134ecd3a175db1`). It verifies exact admission and environment bytes, applies the recorded unset/variable map, creates one attempt and exclusive stdout/stderr, waits the literal admitted argv, preserves its original exit or spawn failure, and rehashes admission/source endpoints before terminal publication. It neither constructs model arguments nor retries. The external timeout is an admission-argv responsibility, not inferred from this launcher. Root's admission must pin the launcher, this review, final source/command/environment documents and all147 non-initial-project endpoints; the four task files are checked before admission and retained as authorized mutable state. Publication remains nontransactional and configuration drift remains an explicit endpoint error, not a waiver.

Only this new review record was written. Root retains sole authority to publish admission and launch the one bounded scenario.
