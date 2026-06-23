# Three-Feature Smoke Bench

- result: pass
- started_at: 2026-06-23T10:54:09+02:00
- finished_at: 2026-06-23T11:27:00+02:00
- duration_seconds: 1971
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
- web_ui_url: http://127.0.0.1:41967
- base_commit: f35d37e1d7fca8f13a57af707978c08d3891d977
- temp_checkout: /tmp/work-leaf-3feature-bench.BzYSjR
- temp_checkout_kept: 0
- review_completed: yes
- linearize_completed: yes
- commits_after_base: 3
- changed_files: 10
- changed_lines_added: 515
- changed_lines_deleted: 82
- changed_lines_total: 597
- token_usage: linearize: input=6891253 cached_input=6576512 output=30965 reasoning_output=14860; review-user-1: input=525429 cached_input=425856 output=8480 reasoning_output=5143; review-user-2: input=1242874 cached_input=1098752 output=10706 reasoning_output=6696; review-user-3: input=1020941 cached_input=937216 output=11876 reasoning_output=7472; user-1: input=663320 cached_input=571904 output=7186 reasoning_output=3982; user-2: input=2024767 cached_input=1726080 output=11902 reasoning_output=6607; user-3: input=908540 cached_input=793088 output=8151 reasoning_output=4586
- code_quality: passed cargo fmt -- --check; cargo clippy --all-targets --all-features -- -D warnings; cargo test --all-targets --all-features
- comment: review and linearize completed; final repository checks passed
- operator_notes: review and linearize completed; final repository checks passed
- artifacts: /home/user/src/work-leaf/bench-results/parallel-extra-6-20260623T105409+0200/runs/run-3/20260623T105409+0200-extra6-r3-three-feature-bench-artifacts
- binaries: /home/user/src/work-leaf/bench-results/parallel-extra-6-20260623T105409+0200/runs/run-3/20260623T105409+0200-extra6-r3-three-feature-bench-artifacts/bin
- binaries_produced: work-leaf work-leaf-orchestrator
- patch_artifacts: /home/user/src/work-leaf/bench-results/parallel-extra-6-20260623T105409+0200/runs/run-3/20260623T105409+0200-extra6-r3-three-feature-bench-artifacts/patches/pass

## Sessions

```
linearize	-	-	326	linearize reviewed patches
review-user-1	-	-	37	review user-agent
review-user-2	-	-	77	review user-agent
review-user-3	-	-	78	review user-agent
user-1	-	NeedsDecision	104	vim-visual-mode-pane-selection
user-2	-	NeedsDecision	177	strict-agent-slash-execution
user-3	-	NeedsDecision	82	patch-agent-done-prompt
```

## Recent Commits

```
993fd28 ADD backend command execution so selected-agent slash inputs bypass prompt sends
8ce13ff ADD vim visual selection copying for both terminal panes
c8cea2c ADD UI harness coverage for reviewed completion decisions
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
