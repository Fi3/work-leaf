# Three-Feature Smoke Bench

- result: pass
- started_at: 2026-06-23T08:38:11+02:00
- finished_at: 2026-06-23T09:27:41+02:00
- duration_seconds: 2970
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
- web_ui_url: http://127.0.0.1:39129
- base_commit: 6ecb9bc5936422c6a5b9c729c070730d67b102a6
- temp_checkout: /tmp/work-leaf-3feature-bench.SQB7lH
- temp_checkout_kept: 0
- review_completed: yes
- linearize_completed: yes
- commits_after_base: 2
- changed_files: 9
- changed_lines_added: 556
- changed_lines_deleted: 60
- changed_lines_total: 616
- token_usage: linearize: input=6913139 cached_input=6543616 output=28450 reasoning_output=10481; review-user-1: input=368425 cached_input=283008 output=6320 reasoning_output=4381; review-user-2: input=1074282 cached_input=952448 output=12100 reasoning_output=8200; user-1: input=357241 cached_input=300672 output=5252 reasoning_output=3245; user-2: input=2843438 cached_input=2691456 output=20719 reasoning_output=12950; user-3: input=844664 cached_input=721152 output=5833 reasoning_output=4001
- code_quality: passed cargo fmt -- --check; cargo clippy --all-targets --all-features -- -D warnings; cargo test --all-targets --all-features
- comment: review and linearize completed; final repository checks passed
- operator_notes: review and linearize completed; final repository checks passed
- artifacts: /home/user/src/work-leaf/bench-results/parallel-baseline-9-20260623T083811+0200/first-3/run-1/20260623T083811+0200-first3-r1-three-feature-bench-artifacts
- binaries: /home/user/src/work-leaf/bench-results/parallel-baseline-9-20260623T083811+0200/first-3/run-1/20260623T083811+0200-first3-r1-three-feature-bench-artifacts/bin
- binaries_produced: work-leaf work-leaf-orchestrator
- patch_artifacts: /home/user/src/work-leaf/bench-results/parallel-baseline-9-20260623T083811+0200/first-3/run-1/20260623T083811+0200-first3-r1-three-feature-bench-artifacts/patches/pass

## Sessions

```
linearize	-	-	359	linearize reviewed patches
review-user-1	-	-	50	review user-agent
review-user-2	-	-	99	review user-agent
user-1	-	NeedsDecision	87	vim-visual-mode-pane-selection
user-2	-	NeedsDecision	280	strict-selected-agent-slash-commands
user-3	-	-	52	patch-agent-done-confirmation
```

## Recent Commits

```
3cd2729 ADD strict selected-agent slash routing so backend commands bypass prompt turns
db751e4 ADD immediate visual selection for focused panes so Vim-style selection starts on the active row
6ecb9bc WORK_LEAF_BENCH_WORKTREE_SNAPSHOT
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
2795a9a FIX route patch follow-up fixes through a reviewed done cycle
```

## Final Status

```

```
