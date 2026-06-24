# Three-Feature Smoke Bench

- result: fail
- started_at: 2026-06-24T05:37:54+02:00
- finished_at: 2026-06-24T06:10:26+02:00
- duration_seconds: 1952
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
- web_ui_url: http://127.0.0.1:46053
- base_commit: c92a0b7060a36eac6db2d869b85e589a7a9480f9
- temp_checkout: /tmp/work-leaf-3feature-bench.5jYvuM
- temp_checkout_kept: 0
- review_completed: yes
- linearize_completed: yes
- commits_after_base: 3
- changed_files: 11
- changed_lines_added: 1277
- changed_lines_deleted: 48
- changed_lines_total: 1325
- token_usage: linearize: input=5657893 cached_input=5372544 output=21519 reasoning_output=7846; review-user-1: input=1069866 cached_input=939776 output=12690 reasoning_output=9713; review-user-2: input=1189922 cached_input=1054592 output=9571 reasoning_output=5644; review-user-3: input=842363 cached_input=779648 output=6375 reasoning_output=4121; user-1: input=1108103 cached_input=978304 output=8110 reasoning_output=5284; user-2: input=1182889 cached_input=998784 output=2717 reasoning_output=766; user-3: input=304708 cached_input=123776 output=4862 reasoning_output=2506
- code_quality: failed; see checks.log
- comment: final repository checks failed
- operator_notes: current worktree parallel batch 6b retry run 4 after Codex quota reset
- artifacts: /home/user/src/work-leaf/bench-results/parallel-current-6b-retry-20260624T053754+0200/runs/run-4/parallel-current-6b-retry-20260624T053754+0200-r4-three-feature-bench-artifacts
- binaries: /home/user/src/work-leaf/bench-results/parallel-current-6b-retry-20260624T053754+0200/runs/run-4/parallel-current-6b-retry-20260624T053754+0200-r4-three-feature-bench-artifacts/bin
- binaries_produced: work-leaf work-leaf-orchestrator
- patch_artifacts: /home/user/src/work-leaf/bench-results/parallel-current-6b-retry-20260624T053754+0200/runs/run-4/parallel-current-6b-retry-20260624T053754+0200-r4-three-feature-bench-artifacts/patches/fail

## Sessions

```
linearize	-	-	268	linearize reviewed patches
review-user-1	-	-	44	review user-agent
review-user-2	-	-	50	review user-agent
review-user-3	-	-	43	review user-agent
user-1	-	NeedsDecision	94	vim-visual-mode-for-panes
user-2	-	NeedsDecision	103	strict-agent-slash-command-execution
user-3	-	NeedsDecision	74	patch-agent-feature-done-prompt
```

## Recent Commits

```
640019f ADD Vim visual yanking for both terminal panes
1e31815 ADD selected-agent slash dispatch so backend commands avoid prompt wrapping
16a8a1b ADD reviewed-feature confirmation prompts so clean reviews close explicitly
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
