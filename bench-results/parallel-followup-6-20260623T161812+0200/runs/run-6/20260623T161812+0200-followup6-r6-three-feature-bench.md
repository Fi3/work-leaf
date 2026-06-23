# Three-Feature Smoke Bench

- result: pass
- started_at: 2026-06-23T16:18:12+02:00
- finished_at: 2026-06-23T16:50:56+02:00
- duration_seconds: 1964
- benched_binary_commit: e084ec9cb1d6b4b1e00c79952f04879d587b7be9
- benched_binary_dirty: no
- worktree_source_commit: e084ec9cb1d6b4b1e00c79952f04879d587b7be9
- worktree_source_dirty: yes
- agent_backend: codex
- agent_transport: app-server
- codex_cli_path: /usr/bin/codex
- codex_cli_version: codex-cli 0.141.0
- agent_model: gpt-5.5
- agent_model_source: codex config /home/user/.codex/config.toml
- agent_reasoning_effort: xhigh
- agent_reasoning_effort_source: codex config /home/user/.codex/config.toml
- requested_agent_model: default
- no_read_permission: 0
- read_permission_mode: orchestrator-mediated file reads
- web_ui_url: http://127.0.0.1:44807
- base_commit: f53786df9fc47a62913085c1dfddcf348d0490aa
- temp_checkout: /tmp/work-leaf-3feature-bench.2zRjsL
- temp_checkout_kept: 0
- review_completed: yes
- linearize_completed: yes
- commits_after_base: 2
- changed_files: 9
- changed_lines_added: 679
- changed_lines_deleted: 136
- changed_lines_total: 815
- token_usage: linearize: input=11570272 cached_input=11260288 output=30148 reasoning_output=9847; review-user-1: input=1123874 cached_input=1064192 output=14915 reasoning_output=9891; review-user-2: input=709008 cached_input=627456 output=8336 reasoning_output=5182; user-1: input=787208 cached_input=657792 output=9840 reasoning_output=5769; user-2: input=2667641 cached_input=2476544 output=16255 reasoning_output=10207; user-3: input=491005 cached_input=381184 output=3167 reasoning_output=1597
- code_quality: passed cargo fmt -- --check; cargo clippy --all-targets --all-features -- -D warnings; cargo test --all-targets --all-features
- comment: review and linearize completed; final repository checks passed
- operator_notes: six-run parallel follow-up batch 20260623T161812+0200 run 6; current worktree snapshot
- artifacts: /home/user/src/work-leaf/bench-results/parallel-followup-6-20260623T161812+0200/runs/run-6/20260623T161812+0200-followup6-r6-three-feature-bench-artifacts
- binaries: /home/user/src/work-leaf/bench-results/parallel-followup-6-20260623T161812+0200/runs/run-6/20260623T161812+0200-followup6-r6-three-feature-bench-artifacts/bin
- binaries_produced: work-leaf work-leaf-orchestrator
- patch_artifacts: /home/user/src/work-leaf/bench-results/parallel-followup-6-20260623T161812+0200/runs/run-6/20260623T161812+0200-followup6-r6-three-feature-bench-artifacts/patches/pass

## Sessions

```
linearize	-	-	392	linearize reviewed patches
review-user-1	-	-	81	review user-agent
review-user-2	-	-	41	review user-agent
user-1	-	NeedsDecision	151	vim-visual-mode-for-panes
user-2	-	NeedsDecision	164	strict-selected-agent-slash-commands
user-3	-	-	39	patch-chat-done-confirmation
```

## Recent Commits

```
8161d53 ADD selected-agent slash routing so backend commands bypass prompt sends
0954ed3 ADD terminal visual selection so focused panes can be yanked
f53786d WORK_LEAF_BENCH_WORKTREE_SNAPSHOT
e084ec9 FIX require resolved Codex metadata for benchmark reports
9b899a2 ADD Codex benchmark baseline artifacts for regression detection
877de09 FIX make agent workflows recover and benchmark gates state-based
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
```

## Final Status

```

```
