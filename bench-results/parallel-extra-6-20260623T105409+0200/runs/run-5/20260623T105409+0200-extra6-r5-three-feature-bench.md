# Three-Feature Smoke Bench

- result: pass
- started_at: 2026-06-23T10:54:09+02:00
- finished_at: 2026-06-23T11:26:55+02:00
- duration_seconds: 1966
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
- web_ui_url: http://127.0.0.1:43485
- base_commit: f35d37e1d7fca8f13a57af707978c08d3891d977
- temp_checkout: /tmp/work-leaf-3feature-bench.FnkzAb
- temp_checkout_kept: 0
- review_completed: yes
- linearize_completed: yes
- commits_after_base: 3
- changed_files: 11
- changed_lines_added: 512
- changed_lines_deleted: 97
- changed_lines_total: 609
- token_usage: linearize: input=7982395 cached_input=7567616 output=26228 reasoning_output=8482; review-user-1: input=402990 cached_input=302208 output=5133 reasoning_output=3884; review-user-2: input=1764361 cached_input=1586688 output=9509 reasoning_output=4217; review-user-3: input=281983 cached_input=237440 output=4165 reasoning_output=3162; user-1: input=872056 cached_input=819968 output=7261 reasoning_output=4615; user-2: input=2485233 cached_input=2150144 output=20369 reasoning_output=12888; user-3: input=714039 cached_input=580096 output=5760 reasoning_output=2140
- code_quality: passed cargo fmt -- --check; cargo clippy --all-targets --all-features -- -D warnings; cargo test --all-targets --all-features
- comment: review and linearize completed; final repository checks passed
- operator_notes: review and linearize completed; final repository checks passed
- artifacts: /home/user/src/work-leaf/bench-results/parallel-extra-6-20260623T105409+0200/runs/run-5/20260623T105409+0200-extra6-r5-three-feature-bench-artifacts
- binaries: /home/user/src/work-leaf/bench-results/parallel-extra-6-20260623T105409+0200/runs/run-5/20260623T105409+0200-extra6-r5-three-feature-bench-artifacts/bin
- binaries_produced: work-leaf work-leaf-orchestrator
- patch_artifacts: /home/user/src/work-leaf/bench-results/parallel-extra-6-20260623T105409+0200/runs/run-5/20260623T105409+0200-extra6-r5-three-feature-bench-artifacts/patches/pass

## Sessions

```
linearize	-	-	369	linearize reviewed patches
review-user-1	-	-	31	review user-agent
review-user-2	-	-	61	review user-agent
review-user-3	-	-	25	review user-agent
user-1	-	NeedsDecision	96	vim-visual-mode-for-panes
user-2	-	NeedsDecision	201	strict-agent-slash-commands
user-3	-	NeedsDecision	115	review-done-chat-confirm
```

## Recent Commits

```
78d5233 ADD backend slash-command execution for selected-agent chats
de0f922 FIX plain visual mode to select text in the focused pane
17474ec ADD closed-chat reopen coverage so user follow-ups resume backend turns
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
