# Three-Feature Smoke Bench

- result: pass
- started_at: 2026-06-23T09:28:06+02:00
- finished_at: 2026-06-23T09:56:29+02:00
- duration_seconds: 1703
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
- web_ui_url: http://127.0.0.1:46193
- base_commit: 69f1bd7d02bf4f406db9a86d1e5fdec28ee5c5c3
- temp_checkout: /tmp/work-leaf-3feature-bench.3p6RQ9
- temp_checkout_kept: 0
- review_completed: yes
- linearize_completed: yes
- commits_after_base: 3
- changed_files: 12
- changed_lines_added: 540
- changed_lines_deleted: 188
- changed_lines_total: 728
- token_usage: linearize: input=9758063 cached_input=9408640 output=29923 reasoning_output=11561; review-user-1: input=990877 cached_input=809856 output=9451 reasoning_output=6465; review-user-2: input=420335 cached_input=301824 output=7466 reasoning_output=3792; review-user-3: input=628555 cached_input=513024 output=6516 reasoning_output=3330; user-1: input=1244348 cached_input=1130112 output=5741 reasoning_output=3247; user-2: input=1782895 cached_input=1591808 output=9558 reasoning_output=2901; user-3: input=1064951 cached_input=913664 output=7730 reasoning_output=4177
- code_quality: passed cargo fmt -- --check; cargo clippy --all-targets --all-features -- -D warnings; cargo test --all-targets --all-features
- comment: review and linearize completed; final repository checks passed
- operator_notes: review and linearize completed; final repository checks passed
- artifacts: /home/user/src/work-leaf/bench-results/parallel-baseline-9-20260623T083811+0200/remaining-6/run-4/20260623T083811+0200-remaining6-r4-three-feature-bench-artifacts
- binaries: /home/user/src/work-leaf/bench-results/parallel-baseline-9-20260623T083811+0200/remaining-6/run-4/20260623T083811+0200-remaining6-r4-three-feature-bench-artifacts/bin
- binaries_produced: work-leaf work-leaf-orchestrator
- patch_artifacts: /home/user/src/work-leaf/bench-results/parallel-baseline-9-20260623T083811+0200/remaining-6/run-4/20260623T083811+0200-remaining6-r4-three-feature-bench-artifacts/patches/pass

## Sessions

```
linearize	-	-	344	linearize reviewed patches
review-user-1	-	-	44	review user-agent
review-user-2	-	-	76	review user-agent
review-user-3	-	-	79	review user-agent
user-1	-	NeedsDecision	80	vim-visual-selection-panes
user-2	-	NeedsDecision	98	strict-agent-slash-commands
user-3	-	NeedsDecision	119	patch-chat-feature-done-confirm
```

## Recent Commits

```
ca2f174 ADD selected-agent backend slash commands so provider actions bypass prompts
97f175e ADD Vim visual selections for focused terminal panes
8e3b010 ADD review-done patch chats to ready highlighting for completion decisions
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
