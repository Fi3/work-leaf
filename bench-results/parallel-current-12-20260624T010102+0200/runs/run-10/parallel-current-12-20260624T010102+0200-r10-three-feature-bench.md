# Three-Feature Smoke Bench

- result: pass
- started_at: 2026-06-24T01:01:08+02:00
- finished_at: 2026-06-24T01:31:54+02:00
- duration_seconds: 1846
- benched_binary_commit: d4cb33d9cae99387831c690ca3b5201450558634
- benched_binary_dirty: yes
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
- web_ui_url: http://127.0.0.1:35661
- base_commit: c92a0b7060a36eac6db2d869b85e589a7a9480f9
- temp_checkout: /tmp/work-leaf-3feature-bench.W0vkqZ
- temp_checkout_kept: 0
- review_completed: yes
- linearize_completed: yes
- commits_after_base: 3
- changed_files: 12
- changed_lines_added: 1475
- changed_lines_deleted: 98
- changed_lines_total: 1573
- token_usage: linearize: input=6699657 cached_input=6513792 output=27597 reasoning_output=10478; review-user-1: input=1240163 cached_input=1183488 output=11064 reasoning_output=7688; review-user-2: input=638101 cached_input=570624 output=10943 reasoning_output=7640; review-user-3: input=722900 cached_input=631936 output=9336 reasoning_output=6589; user-1: input=1867465 cached_input=1691136 output=10502 reasoning_output=5993; user-2: input=2006954 cached_input=1796736 output=9282 reasoning_output=6372; user-3: input=1202900 cached_input=1037440 output=6494 reasoning_output=2420
- code_quality: passed cargo fmt -- --check; cargo clippy --all-targets --all-features -- -D warnings; cargo test --all-targets --all-features
- comment: review and linearize completed; final repository checks passed
- operator_notes: current worktree parallel batch 12 run 10
- artifacts: /home/user/src/work-leaf/bench-results/parallel-current-12-20260624T010102+0200/runs/run-10/parallel-current-12-20260624T010102+0200-r10-three-feature-bench-artifacts
- binaries: /home/user/src/work-leaf/bench-results/parallel-current-12-20260624T010102+0200/runs/run-10/parallel-current-12-20260624T010102+0200-r10-three-feature-bench-artifacts/bin
- binaries_produced: work-leaf work-leaf-orchestrator
- patch_artifacts: /home/user/src/work-leaf/bench-results/parallel-current-12-20260624T010102+0200/runs/run-10/parallel-current-12-20260624T010102+0200-r10-three-feature-bench-artifacts/patches/pass

## Sessions

```
linearize	-	-	287	linearize reviewed patches
review-user-1	-	-	48	review user-agent
review-user-2	-	-	53	review user-agent
review-user-3	-	-	66	review user-agent
user-1	-	NeedsDecision	142	vim-visual-mode-pane-selection
user-2	-	NeedsDecision	125	strict-selected-slash-execution
user-3	-	NeedsDecision	151	patch-agent-feature-done-prompt
```

## Recent Commits

```
92a21ba ADD Vim visual yanks in terminal panes for clipboard copying
d861a5e ADD selected-agent backend slash commands to avoid prompt resumes
a6306e6 ADD reviewed-feature closure confirmation so clean patch work can close
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
