# Three-Feature Smoke Bench

- result: pass
- started_at: 2026-06-23T09:28:06+02:00
- finished_at: 2026-06-23T09:58:50+02:00
- duration_seconds: 1844
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
- web_ui_url: http://127.0.0.1:36253
- base_commit: 69f1bd7d02bf4f406db9a86d1e5fdec28ee5c5c3
- temp_checkout: /tmp/work-leaf-3feature-bench.rpvk1X
- temp_checkout_kept: 0
- review_completed: yes
- linearize_completed: yes
- commits_after_base: 3
- changed_files: 10
- changed_lines_added: 454
- changed_lines_deleted: 48
- changed_lines_total: 502
- token_usage: linearize: input=7222440 cached_input=6764288 output=28725 reasoning_output=10394; review-user-1: input=582443 cached_input=413696 output=9626 reasoning_output=7508; review-user-2: input=419847 cached_input=335744 output=5076 reasoning_output=3201; review-user-3: input=233627 cached_input=201472 output=4244 reasoning_output=3483; user-1: input=1092558 cached_input=1019392 output=5761 reasoning_output=3157; user-2: input=1469920 cached_input=1273728 output=7631 reasoning_output=2945; user-3: input=2243774 cached_input=1980032 output=10062 reasoning_output=5759
- code_quality: passed cargo fmt -- --check; cargo clippy --all-targets --all-features -- -D warnings; cargo test --all-targets --all-features
- comment: review and linearize completed; final repository checks passed
- operator_notes: review and linearize completed; final repository checks passed
- artifacts: /home/user/src/work-leaf/bench-results/parallel-baseline-9-20260623T083811+0200/remaining-6/run-9/20260623T083811+0200-remaining6-r9-three-feature-bench-artifacts
- binaries: /home/user/src/work-leaf/bench-results/parallel-baseline-9-20260623T083811+0200/remaining-6/run-9/20260623T083811+0200-remaining6-r9-three-feature-bench-artifacts/bin
- binaries_produced: work-leaf work-leaf-orchestrator
- patch_artifacts: /home/user/src/work-leaf/bench-results/parallel-baseline-9-20260623T083811+0200/remaining-6/run-9/20260623T083811+0200-remaining6-r9-three-feature-bench-artifacts/patches/pass

## Sessions

```
linearize	-	-	334	linearize reviewed patches
review-user-1	-	-	45	review user-agent
review-user-2	-	-	45	review user-agent
review-user-3	-	-	21	review user-agent
user-1	-	NeedsDecision	103	vim-visual-mode-for-panes
user-2	-	NeedsDecision	105	strict-agent-slash-commands
user-3	-	NeedsDecision	115	review-done-chat-confirmation
```

## Recent Commits

```
b0a3b52 ADD selected-agent slash command routing so backend commands bypass prompts
9201fd0 UPDATE terminal visual selections to start and rebase from the focused pane
b8a1b47 ADD completion-decision regression coverage for reviewed patch chats
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
