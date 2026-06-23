# Three-Feature Smoke Bench

- result: pass
- started_at: 2026-06-23T13:59:39+02:00
- finished_at: 2026-06-23T14:25:17+02:00
- duration_seconds: 1538
- benched_binary_commit: 7014e00d15ecb242e3848478da86f5b8dcbbf114
- benched_binary_dirty: yes
- worktree_source_commit: 7014e00d15ecb242e3848478da86f5b8dcbbf114
- worktree_source_dirty: yes
- agent_backend: codex
- agent_transport: app-server
- agent_model: unknown
- agent_model_source: not requested
- requested_agent_model: default
- no_read_permission: 0
- read_permission_mode: orchestrator-mediated file reads
- web_ui_url: http://127.0.0.1:41523
- base_commit: 3e343a14a57d5170aacc2b1d3abb5dc1123c8b4b
- temp_checkout: /tmp/work-leaf-3feature-bench.geJxAD
- temp_checkout_kept: 0
- review_completed: yes
- linearize_completed: yes
- commits_after_base: 2
- changed_files: 10
- changed_lines_added: 517
- changed_lines_deleted: 129
- changed_lines_total: 646
- token_usage: linearize: input=5046980 cached_input=4731904 output=23448 reasoning_output=7808; review-user-1: input=1099183 cached_input=1053568 output=9922 reasoning_output=6103; review-user-2: input=1291705 cached_input=1169024 output=8767 reasoning_output=4366; user-1: input=892819 cached_input=827776 output=9302 reasoning_output=5516; user-2: input=1232575 cached_input=1109760 output=10526 reasoning_output=6995; user-3: input=392758 cached_input=307072 output=3149 reasoning_output=1448
- code_quality: passed cargo fmt -- --check; cargo clippy --all-targets --all-features -- -D warnings; cargo test --all-targets --all-features
- comment: review and linearize completed; final repository checks passed
- operator_notes: post-timeout-gate regression baseline run
- artifacts: /home/user/src/work-leaf/bench-results/parallel-baseline-6-20260623T135939+0200/runs/run-5/20260623T135939+0200-baseline6-r5-three-feature-bench-artifacts
- binaries: /home/user/src/work-leaf/bench-results/parallel-baseline-6-20260623T135939+0200/runs/run-5/20260623T135939+0200-baseline6-r5-three-feature-bench-artifacts/bin
- binaries_produced: work-leaf work-leaf-orchestrator
- patch_artifacts: /home/user/src/work-leaf/bench-results/parallel-baseline-6-20260623T135939+0200/runs/run-5/20260623T135939+0200-baseline6-r5-three-feature-bench-artifacts/patches/pass

## Sessions

```
linearize	-	-	327	linearize reviewed patches
review-user-1	-	-	54	review user-agent
review-user-2	-	-	62	review user-agent
user-1	-	NeedsDecision	115	vim-visual-select-panes
user-2	-	NeedsDecision	131	strict-agent-slash-commands
user-3	-	-	45	review-done-feature-confirmation
```

## Recent Commits

```
0a100f7 ADD backend slash command routing because selected-agent commands must bypass prompt sends
0a96dbb ADD focused-pane visual selection because v should select text immediately
3e343a1 WORK_LEAF_BENCH_WORKTREE_SNAPSHOT
7014e00 v0.1.1
faf17c7 ADD Claude benchmark failure artifacts to preserve quota-limited run
efcabba FIX preserve provider selection API and make Claude interrupts real
371c38f ADD Leaf blame Neovim plugin to expose Work Leaf provenance in editor
15fd27c ADD Claude provider selection and mediated bundle reads for portable agent workflows
cf50907 ADD release publishing to binary workflow so builds produce GitHub releases
9a99771 UPGRADE Drive Codex through app-server JSON-RPC to remove SDK runtime
b93e585 FIX keep command and title system agents persistent for contextual orchestration
23a638a ADD reviewed and reviewing sections to the terminal left pane
3be0d3f FIX wrapped chat prompt cursor at the visible line edge
4a66811 FIX completion prompts after clean patch-agent reviews
7ef94b9 ADD patch lifecycle sections to the terminal left menu
15b9c51 FIX command-mode comma to focus the left pane from chat panes
80e6e8d FIX dependent waiting-chat launches to carry the first task metadata
2ffa3cd ADD per-window chat message folds for terminal panes
9e77538 ADD automatic Rust target installation for release packaging
ec06f79 ADD daemon and remote CLI modes for packaged launches
97ecf21 ADD focus selected chat with Enter from the left pane
d7971f5 UPDATE release packaging to avoid host cross-linking failures
b2b1329 UPDATE apply when-i-m-prompting-i-ctr patch from user-2
b49856a FIX insert-mode multiline input so terminal line breaks do not submit unfinished prompts
6269daa UPDATE derive compact chat titles through the title agent so left-pane rows stay readable
b2ee5b9 FIX prompt cursor row calculation so full-width chat lines keep the cursor on the prompt
dac6b2e UPDATE right-pane chat navigation so gg and G reach history edges
973b2eb UPDATE reviewer context to use commit logs and patch-agent evidence
723eb7e ADD grouped terminal chat sections and suppress review ready highlighting
2795a9a FIX route patch follow-up fixes through a reviewed done cycle
```

## Final Status

```

```
