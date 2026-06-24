# Three-Feature Smoke Bench

- result: pass
- started_at: 2026-06-24T12:14:34+02:00
- finished_at: 2026-06-24T12:53:45+02:00
- duration_seconds: 2351
- benched_binary_commit: 65a71fe8bd1a9a2adc4173d0775526514c01a76e
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
- web_ui_url: http://127.0.0.1:36419
- base_commit: c92a0b7060a36eac6db2d869b85e589a7a9480f9
- temp_checkout: /tmp/work-leaf-3feature-bench.toIS5M
- temp_checkout_kept: 0
- review_completed: yes
- linearize_completed: yes
- commits_after_base: 3
- changed_files: 13
- changed_lines_added: 1210
- changed_lines_deleted: 101
- changed_lines_total: 1311
- token_usage: linearize: input=5928326 cached_input=5549952 output=27787 reasoning_output=12708; review-user-1: input=1261824 cached_input=1132928 output=24337 reasoning_output=20387; review-user-2: input=723654 cached_input=606336 output=8010 reasoning_output=5970; review-user-3: input=326980 cached_input=273536 output=9285 reasoning_output=7705; user-1: input=1725583 cached_input=1544704 output=13458 reasoning_output=9443; user-2: input=655962 cached_input=498560 output=6739 reasoning_output=2844; user-3: input=826821 cached_input=715520 output=7423 reasoning_output=5529
- code_quality: passed cargo fmt -- --check; cargo clippy --all-targets --all-features -- -D warnings; cargo test --all-targets --all-features
- comment: review and linearize completed; final repository checks passed
- operator_notes: parallel baseline batch parallel-current-6d-20260624T121434+0200 run 2
- artifacts: /home/user/src/work-leaf/bench-results/parallel-current-6d-20260624T121434+0200/runs/run-2/parallel-current-6d-20260624T121434+0200-r2-three-feature-bench-artifacts
- binaries: /home/user/src/work-leaf/bench-results/parallel-current-6d-20260624T121434+0200/runs/run-2/parallel-current-6d-20260624T121434+0200-r2-three-feature-bench-artifacts/bin
- binaries_produced: work-leaf work-leaf-orchestrator
- patch_artifacts: /home/user/src/work-leaf/bench-results/parallel-current-6d-20260624T121434+0200/runs/run-2/parallel-current-6d-20260624T121434+0200-r2-three-feature-bench-artifacts/patches/pass

## Sessions

```
linearize	-	-	272	linearize reviewed patches
review-user-1	-	-	55	review user-agent
review-user-2	-	-	44	review user-agent
review-user-3	-	-	38	review user-agent
user-1	-	NeedsDecision	156	vim-visual-pane-selection-copy
user-2	-	NeedsDecision	83	strict-selected-agent-slash-commands
user-3	-	NeedsDecision	86	patch-agent-done-confirmation
```

## Recent Commits

```
9b3d336 ADD Vim visual yanking so terminal panes support keyboard copy
0c3ccc5 ADD selected-agent slash command routing so backend commands bypass prompt sends
a4f6031 ADD review-complete confirmation so patch chats highlight only while awaiting closure
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
