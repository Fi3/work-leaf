# Three-Feature Smoke Bench

- result: pass
- started_at: 2026-06-23T13:59:39+02:00
- finished_at: 2026-06-23T14:31:27+02:00
- duration_seconds: 1908
- benched_binary_commit: 7014e00d15ecb242e3848478da86f5b8dcbbf114
- benched_binary_dirty: yes
- worktree_source_commit: 7014e00d15ecb242e3848478da86f5b8dcbbf114
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
- web_ui_url: http://127.0.0.1:44143
- base_commit: 3e343a14a57d5170aacc2b1d3abb5dc1123c8b4b
- temp_checkout: /tmp/work-leaf-3feature-bench.kNuOYX
- temp_checkout_kept: 0
- review_completed: yes
- linearize_completed: yes
- commits_after_base: 3
- changed_files: 10
- changed_lines_added: 455
- changed_lines_deleted: 75
- changed_lines_total: 530
- token_usage: linearize: input=7486095 cached_input=7156096 output=27138 reasoning_output=12385; review-user-1: input=445693 cached_input=365696 output=4387 reasoning_output=2693; review-user-2: input=1022865 cached_input=934272 output=7547 reasoning_output=4123; review-user-3: input=446299 cached_input=399104 output=5422 reasoning_output=3772; user-1: input=1229408 cached_input=1120000 output=9130 reasoning_output=5422; user-2: input=1859311 cached_input=1718016 output=16703 reasoning_output=11947; user-3: input=915662 cached_input=805376 output=7125 reasoning_output=3430
- code_quality: passed cargo fmt -- --check; cargo clippy --all-targets --all-features -- -D warnings; cargo test --all-targets --all-features
- comment: review and linearize completed; final repository checks passed
- operator_notes: post-timeout-gate regression baseline run
- artifacts: /home/user/src/work-leaf/bench-results/parallel-baseline-6-20260623T135939+0200/runs/run-2/20260623T135939+0200-baseline6-r2-three-feature-bench-artifacts
- binaries: /home/user/src/work-leaf/bench-results/parallel-baseline-6-20260623T135939+0200/runs/run-2/20260623T135939+0200-baseline6-r2-three-feature-bench-artifacts/bin
- binaries_produced: work-leaf work-leaf-orchestrator
- patch_artifacts: /home/user/src/work-leaf/bench-results/parallel-baseline-6-20260623T135939+0200/runs/run-2/20260623T135939+0200-baseline6-r2-three-feature-bench-artifacts/patches/pass

## Sessions

```
linearize	-	-	339	linearize reviewed patches
review-user-1	-	-	33	review user-agent
review-user-2	-	-	46	review user-agent
review-user-3	-	-	31	review user-agent
user-1	-	NeedsDecision	83	vim-visual-mode-for-both-panes
user-2	-	NeedsDecision	121	strict-selected-agent-slash-commands
user-3	-	NeedsDecision	122	review-done-patch-chat-confirm
```

## Recent Commits

```
7723b36 ADD selected-agent slash command routing so backend commands avoid normal prompts
8aafde7 ADD terminal visual selection yanks so focused panes can copy text
0fd475e ADD reviewed completion regression coverage so closed patch-agent chats reopen normally
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
```

## Final Status

```

```
