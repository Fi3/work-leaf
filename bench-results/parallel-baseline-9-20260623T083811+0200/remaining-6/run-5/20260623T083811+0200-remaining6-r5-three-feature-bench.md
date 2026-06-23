# Three-Feature Smoke Bench

- result: pass
- started_at: 2026-06-23T09:28:06+02:00
- finished_at: 2026-06-23T10:07:11+02:00
- duration_seconds: 2345
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
- web_ui_url: http://127.0.0.1:40935
- base_commit: 69f1bd7d02bf4f406db9a86d1e5fdec28ee5c5c3
- temp_checkout: /tmp/work-leaf-3feature-bench.39ewG9
- temp_checkout_kept: 0
- review_completed: yes
- linearize_completed: yes
- commits_after_base: 3
- changed_files: 10
- changed_lines_added: 704
- changed_lines_deleted: 38
- changed_lines_total: 742
- token_usage: linearize: input=8891695 cached_input=8464512 output=28601 reasoning_output=9517; review-user-1: input=483059 cached_input=429184 output=9532 reasoning_output=6783; review-user-2: input=1414249 cached_input=1258240 output=15419 reasoning_output=10669; review-user-3: input=1259174 cached_input=1026944 output=16929 reasoning_output=11689; user-1: input=488748 cached_input=414080 output=7963 reasoning_output=5878; user-2: input=1889946 cached_input=1592960 output=14903 reasoning_output=8703; user-3: input=634588 cached_input=492288 output=8197 reasoning_output=4556
- code_quality: passed cargo fmt -- --check; cargo clippy --all-targets --all-features -- -D warnings; cargo test --all-targets --all-features
- comment: review and linearize completed; final repository checks passed
- operator_notes: review and linearize completed; final repository checks passed
- artifacts: /home/user/src/work-leaf/bench-results/parallel-baseline-9-20260623T083811+0200/remaining-6/run-5/20260623T083811+0200-remaining6-r5-three-feature-bench-artifacts
- binaries: /home/user/src/work-leaf/bench-results/parallel-baseline-9-20260623T083811+0200/remaining-6/run-5/20260623T083811+0200-remaining6-r5-three-feature-bench-artifacts/bin
- binaries_produced: work-leaf work-leaf-orchestrator
- patch_artifacts: /home/user/src/work-leaf/bench-results/parallel-baseline-9-20260623T083811+0200/remaining-6/run-5/20260623T083811+0200-remaining6-r5-three-feature-bench-artifacts/patches/pass

## Sessions

```
linearize	-	-	393	linearize reviewed patches
review-user-1	-	-	72	review user-agent
review-user-2	-	-	84	review user-agent
review-user-3	-	-	100	review user-agent
user-1	-	NeedsDecision	115	vim-visual-mode-panes
user-2	-	NeedsDecision	147	strict-agent-slash-commands
user-3	-	NeedsDecision	117	review-done-feature-confirmation
```

## Recent Commits

```
cb44c46 ADD selected-agent slash dispatch so backend commands stay ordered
003ebd9 ADD completion-decision UI harness coverage for reviewed patch chats
2af2e94 ADD terminal visual selection across both panes for Vim-style yanks
69f1bd7 WORK_LEAF_BENCH_WORKTREE_SNAPSHOT
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
```

## Final Status

```

```
