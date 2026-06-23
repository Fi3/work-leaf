# Three-Feature Smoke Bench

- result: pass
- started_at: 2026-06-23T10:54:09+02:00
- finished_at: 2026-06-23T11:24:04+02:00
- duration_seconds: 1795
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
- web_ui_url: http://127.0.0.1:36703
- base_commit: f35d37e1d7fca8f13a57af707978c08d3891d977
- temp_checkout: /tmp/work-leaf-3feature-bench.ohbyZ8
- temp_checkout_kept: 0
- review_completed: yes
- linearize_completed: yes
- commits_after_base: 3
- changed_files: 11
- changed_lines_added: 582
- changed_lines_deleted: 107
- changed_lines_total: 689
- token_usage: linearize: input=7452573 cached_input=7205504 output=29822 reasoning_output=13367; review-user-1: input=823830 cached_input=689792 output=9840 reasoning_output=7505; review-user-2: input=752168 cached_input=650880 output=6176 reasoning_output=3655; review-user-3: input=409930 cached_input=353920 output=4313 reasoning_output=2704; user-1: input=720747 cached_input=643200 output=8764 reasoning_output=4880; user-2: input=2029775 cached_input=1844736 output=16198 reasoning_output=8886; user-3: input=884488 cached_input=773120 output=5871 reasoning_output=2855
- code_quality: passed cargo fmt -- --check; cargo clippy --all-targets --all-features -- -D warnings; cargo test --all-targets --all-features
- comment: review and linearize completed; final repository checks passed
- operator_notes: review and linearize completed; final repository checks passed
- artifacts: /home/user/src/work-leaf/bench-results/parallel-extra-6-20260623T105409+0200/runs/run-6/20260623T105409+0200-extra6-r6-three-feature-bench-artifacts
- binaries: /home/user/src/work-leaf/bench-results/parallel-extra-6-20260623T105409+0200/runs/run-6/20260623T105409+0200-extra6-r6-three-feature-bench-artifacts/bin
- binaries_produced: work-leaf work-leaf-orchestrator
- patch_artifacts: /home/user/src/work-leaf/bench-results/parallel-extra-6-20260623T105409+0200/runs/run-6/20260623T105409+0200-extra6-r6-three-feature-bench-artifacts/patches/pass

## Sessions

```
linearize	-	-	276	linearize reviewed patches
review-user-1	-	-	51	review user-agent
review-user-2	-	-	42	review user-agent
review-user-3	-	-	43	review user-agent
user-1	-	NeedsDecision	120	vim-visual-mode-pane-selection
user-2	-	NeedsDecision	194	selected-agent-slash-execution
user-3	-	NeedsDecision	80	patch-chat-done-confirmation
```

## Recent Commits

```
0bf8f02 ADD provider command routing so selected-agent slash commands bypass prompt sends
960bd97 ADD focused-pane visual selection so yanks copy visible pane text
406ee40 ADD completion-gate coverage so closed reviewed chats reopen on later work
f35d37e WORK_LEAF_BENCH_WORKTREE_SNAPSHOT
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
