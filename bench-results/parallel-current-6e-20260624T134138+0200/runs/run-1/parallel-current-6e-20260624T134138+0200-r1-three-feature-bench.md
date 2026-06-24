# Three-Feature Smoke Bench

- result: pass
- started_at: 2026-06-24T13:41:38+02:00
- finished_at: 2026-06-24T14:20:02+02:00
- duration_seconds: 2304
- benched_binary_commit: 65157b102e1fdebca6264286fdcb6bf38b1d92c9
- benched_binary_dirty: no
- worktree_source_commit: c92a0b7060a36eac6db2d869b85e589a7a9480f9
- worktree_source_dirty: no
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
- web_ui_url: http://127.0.0.1:46629
- base_commit: c92a0b7060a36eac6db2d869b85e589a7a9480f9
- temp_checkout: /tmp/work-leaf-3feature-bench.qjYCzJ
- temp_checkout_kept: 0
- review_completed: yes
- linearize_completed: yes
- commits_after_base: 3
- changed_files: 12
- changed_lines_added: 1647
- changed_lines_deleted: 52
- changed_lines_total: 1699
- token_usage: linearize: input=3658971 cached_input=3444864 output=23100 reasoning_output=10064; review-user-1: input=923582 cached_input=815232 output=9923 reasoning_output=6756; review-user-2: input=541152 cached_input=455168 output=8938 reasoning_output=5882; review-user-3: input=1274212 cached_input=1134976 output=15928 reasoning_output=12268; user-1: input=1100303 cached_input=989824 output=13841 reasoning_output=11265; user-2: input=781374 cached_input=680192 output=18618 reasoning_output=13847; user-3: input=1433551 cached_input=1257088 output=8881 reasoning_output=4667
- code_quality: passed cargo fmt -- --check; cargo clippy --all-targets --all-features -- -D warnings; cargo test --all-targets --all-features
- comment: review and linearize completed; final repository checks passed
- operator_notes: parallel baseline batch parallel-current-6e-20260624T134138+0200 run 1
- artifacts: /home/user/src/work-leaf/bench-results/parallel-current-6e-20260624T134138+0200/runs/run-1/parallel-current-6e-20260624T134138+0200-r1-three-feature-bench-artifacts
- binaries: /home/user/src/work-leaf/bench-results/parallel-current-6e-20260624T134138+0200/runs/run-1/parallel-current-6e-20260624T134138+0200-r1-three-feature-bench-artifacts/bin
- binaries_produced: work-leaf work-leaf-orchestrator
- patch_artifacts: /home/user/src/work-leaf/bench-results/parallel-current-6e-20260624T134138+0200/runs/run-1/parallel-current-6e-20260624T134138+0200-r1-three-feature-bench-artifacts/patches/pass

## Sessions

```
linearize	-	-	250	linearize reviewed patches
review-user-1	-	-	54	review user-agent
review-user-2	-	-	61	review user-agent
review-user-3	-	-	49	review user-agent
user-1	-	NeedsDecision	79	vim-visual-mode-for-panes
user-2	-	NeedsDecision	124	strict-slash-command-execution
user-3	-	NeedsDecision	168	review-done-feature-confirmation
```

## Recent Commits

```
ae474c4 ADD reviewed patch completion prompts to close or reopen agents
4f57ab6 ADD terminal visual selection for copying pane text without expanding UI modes
75ac12a ADD selected-agent slash commands through backend execution to avoid prompt fallback
c92a0b7 FIX compact orchestrator and UI traffic for concurrent agents
cb5c388 FIX keep Codex resume prompts compact to avoid context blowups
2673db7 ADD localhost orchestrator daemon for CLI isolation
b831ebf UPDATE command-mode typing hints to ignore pure navigation bursts
d731958 UPDATE apply user-agent patch from user-1
9a2e3a6 UPDATE apply user-agent patch from user-1
358999c UPDATE apply user-agent patch from user-1
114c939 FIX review full patch-agent scopes before acceptance
41b4167 UPDATE document Codex slash-command resume policy exception
d9a1176 UPDATE format slash-command regression test so cargo fmt stays clean
db00ed5 UPDATE apply user-agent patch from user-1
50db6e2 UPDATE apply user-agent patch from user-1
cdf31a5 agent
e97dc14 FIX preserve exact reviewed commits for linearize scope
0ae881e FIX preserve new session snapshots before worker polling
bbef6e1 UPDATE apply user-agent patch from user-1
81634c9 UPDATE apply user-agent patch from user-1
cb4e212 UPDATE apply user-agent patch from user-1
0ccfe09 UPDATE apply user-agent patch from user-1
427a5c6 FIX block dirty command output before review and scope linearize
d504abf UPDATE document terminal ready notifications
a5f8a15 FIX require patch-agent readiness before review and cap locked commands
c37e302 UPDATE apply mouse-scrollable-chat-pane patch from user-1
cb349f9 UPDATE apply user-agent patch from user-1
bba96a6 ADD locked command execution so agents can run required checks safely
df67f96 UPDATE apply user-agent patch from user-1
82facd9 UPDATE keep repo checks and chat titles in backend agents
```

## Final Status

```

```
