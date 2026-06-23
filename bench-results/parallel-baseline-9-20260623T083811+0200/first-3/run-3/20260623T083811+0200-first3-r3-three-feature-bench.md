# Three-Feature Smoke Bench

- result: pass
- started_at: 2026-06-23T08:38:11+02:00
- finished_at: 2026-06-23T09:07:18+02:00
- duration_seconds: 1747
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
- web_ui_url: http://127.0.0.1:37257
- base_commit: 6ecb9bc5936422c6a5b9c729c070730d67b102a6
- temp_checkout: /tmp/work-leaf-3feature-bench.JEfw1u
- temp_checkout_kept: 0
- review_completed: yes
- linearize_completed: yes
- commits_after_base: 3
- changed_files: 10
- changed_lines_added: 459
- changed_lines_deleted: 112
- changed_lines_total: 571
- token_usage: linearize: input=4528147 cached_input=4215424 output=20838 reasoning_output=8087; review-user-1: input=446033 cached_input=349824 output=5070 reasoning_output=2471; review-user-2: input=1384606 cached_input=1280896 output=12858 reasoning_output=6394; review-user-3: input=188544 cached_input=172288 output=3449 reasoning_output=2245; user-1: input=901755 cached_input=758016 output=10112 reasoning_output=7592; user-2: input=3144611 cached_input=2853504 output=17607 reasoning_output=10746; user-3: input=1546198 cached_input=1316480 output=10934 reasoning_output=6997
- code_quality: passed cargo fmt -- --check; cargo clippy --all-targets --all-features -- -D warnings; cargo test --all-targets --all-features
- comment: review and linearize completed; final repository checks passed
- operator_notes: review and linearize completed; final repository checks passed
- artifacts: /home/user/src/work-leaf/bench-results/parallel-baseline-9-20260623T083811+0200/first-3/run-3/20260623T083811+0200-first3-r3-three-feature-bench-artifacts
- binaries: /home/user/src/work-leaf/bench-results/parallel-baseline-9-20260623T083811+0200/first-3/run-3/20260623T083811+0200-first3-r3-three-feature-bench-artifacts/bin
- binaries_produced: work-leaf work-leaf-orchestrator
- patch_artifacts: /home/user/src/work-leaf/bench-results/parallel-baseline-9-20260623T083811+0200/first-3/run-3/20260623T083811+0200-first3-r3-three-feature-bench-artifacts/patches/pass

## Sessions

```
linearize	-	-	231	linearize reviewed patches
review-user-1	-	-	56	review user-agent
review-user-2	-	-	83	review user-agent
review-user-3	-	-	40	review user-agent
user-1	-	NeedsDecision	96	vim-visual-mode-for-panes
user-2	-	NeedsDecision	171	strict-slash-command-execution
user-3	-	NeedsDecision	81	feature-done-chat-confirmation
```

## Recent Commits

```
76f1b77 ADD selected-agent slash execution so backend commands bypass prompt sends
5b2739b ADD terminal harness coverage for reviewed completion decisions
7b006cf ADD visual terminal selection so focused pane text can be copied accurately
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
```

## Final Status

```

```
