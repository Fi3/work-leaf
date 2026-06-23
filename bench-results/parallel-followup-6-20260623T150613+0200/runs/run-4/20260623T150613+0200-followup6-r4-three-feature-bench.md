# Three-Feature Smoke Bench

- result: pass
- started_at: 2026-06-23T15:06:13+02:00
- finished_at: 2026-06-23T15:34:24+02:00
- duration_seconds: 1691
- benched_binary_commit: 9b899a276b6693fb6eb66a3c835b3806d8a8e6af
- benched_binary_dirty: yes
- worktree_source_commit: 9b899a276b6693fb6eb66a3c835b3806d8a8e6af
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
- web_ui_url: http://127.0.0.1:37067
- base_commit: 9b899a276b6693fb6eb66a3c835b3806d8a8e6af
- temp_checkout: /tmp/work-leaf-3feature-bench.bgJ9Rc
- temp_checkout_kept: 0
- review_completed: yes
- linearize_completed: yes
- commits_after_base: 2
- changed_files: 11
- changed_lines_added: 397
- changed_lines_deleted: 94
- changed_lines_total: 491
- token_usage: linearize: input=5054708 cached_input=4690816 output=23347 reasoning_output=9154; review-user-1: input=1222050 cached_input=1101824 output=11517 reasoning_output=8517; review-user-2: input=563715 cached_input=449920 output=6443 reasoning_output=3696; user-1: input=1424878 cached_input=1261440 output=11315 reasoning_output=8314; user-2: input=1919603 cached_input=1701504 output=23379 reasoning_output=15268; user-3: input=675687 cached_input=479616 output=4714 reasoning_output=2429
- code_quality: passed cargo fmt -- --check; cargo clippy --all-targets --all-features -- -D warnings; cargo test --all-targets --all-features
- comment: review and linearize completed; final repository checks passed
- operator_notes: follow-up six-run variance baseline after model document review
- artifacts: /home/user/src/work-leaf/bench-results/parallel-followup-6-20260623T150613+0200/runs/run-4/20260623T150613+0200-followup6-r4-three-feature-bench-artifacts
- binaries: /home/user/src/work-leaf/bench-results/parallel-followup-6-20260623T150613+0200/runs/run-4/20260623T150613+0200-followup6-r4-three-feature-bench-artifacts/bin
- binaries_produced: work-leaf work-leaf-orchestrator
- patch_artifacts: /home/user/src/work-leaf/bench-results/parallel-followup-6-20260623T150613+0200/runs/run-4/20260623T150613+0200-followup6-r4-three-feature-bench-artifacts/patches/pass

## Sessions

```
linearize	-	-	265	linearize reviewed patches
review-user-1	-	-	67	review user-agent
review-user-2	-	-	58	review user-agent
user-1	-	NeedsDecision	93	vim-visual-mode-for-panes
user-2	-	NeedsDecision	161	strict-selected-agent-slash-commands
user-3	-	-	58	review-done-chat-confirmation
```

## Recent Commits

```
f271b19 ADD selected-agent backend slash commands so provider actions bypass chat sends
cfdb7cf ADD immediate terminal visual selection so focused panes can yank text
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
973b2eb UPDATE reviewer context to use commit logs and patch-agent evidence
723eb7e ADD grouped terminal chat sections and suppress review ready highlighting
```

## Final Status

```

```
