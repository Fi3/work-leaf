# Three-Feature Direct Codex Bench

- result: pass
- bench_mode: worktree
- started_at: 2026-06-10T20:47:00+02:00
- finished_at: 2026-06-10T21:31:33+02:00
- duration_seconds: 2673
- benched_binary_commit: n/a-direct-codex-baseline
- bench_driver_commit: d04d3e3253b037e41289347b995371b869c39f2b
- bench_driver_dirty: no
- agent_backend: codex
- agent_transport: direct-codex-cli
- agent_conversation_mode: persistent-codex-resume-threads
- codex_version: codex_bin=/usr/bin/codex
codex-cli 0.138.0
- agent_model: gpt-5.5
- agent_model_source: codex sdk config/read
- requested_agent_model: default
- sdk_python: /home/user/src/work-leaf/target/work-leaf-codex-sdk-venv/bin/python
- no_read_permission: n/a-direct-codex
- web_ui_url: n/a-direct-codex
- read_permission_mode: direct Codex filesystem access (workspace-write)
- base_commit: c92a0b7060a36eac6db2d869b85e589a7a9480f9
- temp_checkout: /home/user/src/work-leaf/.bench-tmp/work-leaf-3feature-worktree-bench.if2eWj
- temp_checkout_kept: 0
- review_completed: yes
- linearize_completed: yes
- review_round_limit: 0
- commits_after_base: 3
- changed_files: 15
- changed_lines_added: 1760
- changed_lines_deleted: 192
- changed_lines_total: 1952
- token_usage: linearize: input=2529270 cached_input=2437760 output=19689 reasoning_output=7131; worktree-feature-1-fix-1: input=7806051 cached_input=7553024 output=44008 reasoning_output=18253; worktree-feature-1-fix-2: input=10958246 cached_input=10673920 output=50224 reasoning_output=20874; worktree-feature-1-fix-3: input=19220325 cached_input=18897920 output=63391 reasoning_output=25453; worktree-feature-1-fix-4: input=19942744 cached_input=19575936 output=67427 reasoning_output=26823; worktree-feature-1-implement: input=5116737 cached_input=4971264 output=37435 reasoning_output=15334; worktree-feature-1-review-1: input=957791 cached_input=841600 output=11091 reasoning_output=6745; worktree-feature-1-review-2: input=1756608 cached_input=1610880 output=14408 reasoning_output=8305; worktree-feature-1-review-3: input=2205930 cached_input=2043648 output=17875 reasoning_output=10927; worktree-feature-1-review-4: input=3057568 cached_input=2875008 output=21989 reasoning_output=13188; worktree-feature-1-review-5: input=3615457 cached_input=3415808 output=23773 reasoning_output=14263; worktree-feature-2-fix-1: input=14408303 cached_input=14029440 output=48801 reasoning_output=18657; worktree-feature-2-implement: input=11171561 cached_input=10955392 output=41820 reasoning_output=16704; worktree-feature-2-review-1: input=735089 cached_input=642688 output=10194 reasoning_output=5891; worktree-feature-2-review-2: input=1457875 cached_input=1323392 output=14267 reasoning_output=8035; worktree-feature-3-fix-1: input=10744232 cached_input=10538496 output=37499 reasoning_output=16139; worktree-feature-3-implement: input=8288489 cached_input=8102016 output=33573 reasoning_output=14994; worktree-feature-3-review-1: input=867660 cached_input=774784 output=11560 reasoning_output=6033; worktree-feature-3-review-2: input=1524944 cached_input=1406848 output=18533 reasoning_output=10832
- code_quality: passed cargo fmt -- --check; cargo clippy --all-targets --all-features -- -D warnings; cargo test --all-targets --all-features
- comment: direct Codex worktree benchmark completed review, linearize, and final checks
- operator_notes: Direct Codex worktree parallel baseline with persistent implementer/reviewer Codex chats after d04d3e3; rerun with repo-local temp parent after /tmp quota failure.
- artifacts: /home/user/src/work-leaf/bench-results/20260610T204700+0200-three-feature-worktree-bench-artifacts
- binaries: /home/user/src/work-leaf/bench-results/20260610T204700+0200-three-feature-worktree-bench-artifacts/bin
- binaries_produced: work-leaf work-leaf-orchestrator
- patch_artifacts: /home/user/src/work-leaf/bench-results/20260610T204700+0200-three-feature-worktree-bench-artifacts/patches/pass

## Recent Commits

```
4f8bdb8 ADD review-completion prompts for patch-agent done decisions
c920b2d ADD selected-agent slash commands for backend-only execution
50c9d14 ADD vim-style visual selection for terminal pane copying
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
