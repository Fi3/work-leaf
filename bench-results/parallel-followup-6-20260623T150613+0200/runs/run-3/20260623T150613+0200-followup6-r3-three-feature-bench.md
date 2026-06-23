# Three-Feature Smoke Bench

- result: pass
- started_at: 2026-06-23T15:06:13+02:00
- finished_at: 2026-06-23T15:33:08+02:00
- duration_seconds: 1615
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
- web_ui_url: http://127.0.0.1:34251
- base_commit: 9b899a276b6693fb6eb66a3c835b3806d8a8e6af
- temp_checkout: /tmp/work-leaf-3feature-bench.IRz54x
- temp_checkout_kept: 0
- review_completed: yes
- linearize_completed: yes
- commits_after_base: 3
- changed_files: 11
- changed_lines_added: 400
- changed_lines_deleted: 63
- changed_lines_total: 463
- token_usage: linearize: input=8607205 cached_input=8293120 output=25846 reasoning_output=9533; review-user-1: input=542448 cached_input=449920 output=6365 reasoning_output=4636; review-user-2: input=366946 cached_input=253952 output=5175 reasoning_output=2594; review-user-3: input=800175 cached_input=614912 output=7986 reasoning_output=5521; user-1: input=1614259 cached_input=1328128 output=14127 reasoning_output=10349; user-2: input=1685688 cached_input=1408768 output=14191 reasoning_output=10187; user-3: input=848673 cached_input=694528 output=8701 reasoning_output=6052
- code_quality: passed cargo fmt -- --check; cargo clippy --all-targets --all-features -- -D warnings; cargo test --all-targets --all-features
- comment: review and linearize completed; final repository checks passed
- operator_notes: follow-up six-run variance baseline after model document review
- artifacts: /home/user/src/work-leaf/bench-results/parallel-followup-6-20260623T150613+0200/runs/run-3/20260623T150613+0200-followup6-r3-three-feature-bench-artifacts
- binaries: /home/user/src/work-leaf/bench-results/parallel-followup-6-20260623T150613+0200/runs/run-3/20260623T150613+0200-followup6-r3-three-feature-bench-artifacts/bin
- binaries_produced: work-leaf work-leaf-orchestrator
- patch_artifacts: /home/user/src/work-leaf/bench-results/parallel-followup-6-20260623T150613+0200/runs/run-3/20260623T150613+0200-followup6-r3-three-feature-bench-artifacts/patches/pass

## Sessions

```
linearize	-	-	325	linearize reviewed patches
review-user-1	-	-	43	review user-agent
review-user-2	-	-	53	review user-agent
review-user-3	-	-	41	review user-agent
user-1	-	NeedsDecision	122	vim-visual-mode-pane-text-selection
user-2	-	NeedsDecision	136	strict-selected-agent-slash-commands
user-3	-	NeedsDecision	69	review-done-patch-chat-confirm
```

## Recent Commits

```
b4628c1 ADD visual selection yanks for focused panes to make copied text consistent
13d4a7d ADD backend slash-command dispatch to keep selected-agent commands off prompt sends
bdab8e8 ADD UI harness completion fixtures to cover close and reopen decisions
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
```

## Final Status

```

```
