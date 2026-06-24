# Three-Feature Smoke Bench

- result: fail
- started_at: 2026-06-24T12:14:34+02:00
- finished_at: 2026-06-24T12:45:29+02:00
- duration_seconds: 1855
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
- web_ui_url: http://127.0.0.1:32815
- base_commit: c92a0b7060a36eac6db2d869b85e589a7a9480f9
- temp_checkout: /tmp/work-leaf-3feature-bench.fNY8Y6
- temp_checkout_kept: 0
- review_completed: no
- linearize_completed: no
- commits_after_base: 11
- changed_files: 10
- changed_lines_added: 1329
- changed_lines_deleted: 68
- changed_lines_total: 1397
- token_usage: review-user-1: input=1654379 cached_input=1501568 output=20907 reasoning_output=16748; review-user-3: input=756723 cached_input=635264 output=9532 reasoning_output=7361; user-1: input=1567872 cached_input=1395968 output=13833 reasoning_output=10491; user-2: input=424388 cached_input=344320 output=3560 reasoning_output=1316; user-3: input=739610 cached_input=572672 output=5134 reasoning_output=1365
- code_quality: not run
- comment: idle stalled for more than 300s without session state changes; user_count=3 terminal_users=2 done_users=2 ready_users=2 patch_agents_with_commits=3
- operator_notes: parallel baseline batch parallel-current-6d-20260624T121434+0200 run 5
- artifacts: /home/user/src/work-leaf/bench-results/parallel-current-6d-20260624T121434+0200/runs/run-5/parallel-current-6d-20260624T121434+0200-r5-three-feature-bench-artifacts
- binaries: /home/user/src/work-leaf/bench-results/parallel-current-6d-20260624T121434+0200/runs/run-5/parallel-current-6d-20260624T121434+0200-r5-three-feature-bench-artifacts/bin
- binaries_produced: work-leaf work-leaf-orchestrator
- patch_artifacts: /home/user/src/work-leaf/bench-results/parallel-current-6d-20260624T121434+0200/runs/run-5/parallel-current-6d-20260624T121434+0200-r5-three-feature-bench-artifacts/patches/fail

## Sessions

```
review-user-1	-	-	56	review user-agent
review-user-3	-	-	38	review user-agent
user-1	-	NeedsDecision	158	vim-visual-mode-for-panes
user-2	-	-	41	implement-strict-selected-agent-slash
user-3	-	NeedsDecision	118	review-done-chat-confirmation
```

## Recent Commits

```
7f54c48 UPDATE apply add-vim-visual-mode-both-panes patch from user-1
c7f3eb2 UPDATE apply add-vim-visual-mode-both-panes patch from user-1
032afe9 UPDATE apply add-vim-visual-mode-both-panes patch from user-1
b856bfe UPDATE apply add-vim-visual-mode-both-panes patch from user-1
93e8147 UPDATE apply user-agent patch from user-1
71e4d29 UPDATE apply user-agent patch from user-1
585b431 UPDATE apply when-review-process-done-patch-agent patch from user-3
03cb804 UPDATE apply user-agent patch from user-3
3353ed5 UPDATE apply user-agent patch from user-3
f519e1e UPDATE apply user-agent patch from user-3
4688f4b UPDATE apply user-agent patch from user-2
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
```

## Final Status

```

```
