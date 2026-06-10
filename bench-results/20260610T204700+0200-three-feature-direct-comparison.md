# Three-Feature Benchmark Comparison

- compared_at: 2026-06-10T21:31:34+02:00
- base_commit: c92a0b7060a36eac6db2d869b85e589a7a9480f9
- model: gpt-5.5
- codex_cli: 0.138.0
- bench_driver_commit: d04d3e3253b037e41289347b995371b869c39f2b

## Results

| mode | report | result | duration | review | linearize | final checks | commits | changed files | lines |
| --- | --- | --- | ---: | --- | --- | --- | ---: | ---: | ---: |
| Work Leaf orchestrator | `20260610T152953+0200-three-feature-bench.md` | pass | 4557s | yes | yes | pass | 4 | 19 | +1746/-146 |
| Direct Codex sequential | `20260610T194209+0200-three-feature-sequential-bench.md` | pass | 3867s | yes | yes | pass | 3 | 16 | +1549/-125 |
| Direct Codex worktree | `20260610T204700+0200-three-feature-worktree-bench.md` | pass | 2673s | yes | yes | pass | 3 | 15 | +1760/-192 |
| Direct Codex worktree first attempt | `20260610T194209+0200-three-feature-worktree-bench.md` | fail | 1500s | yes | no | not run | 3 | 16 | +1341/-93 |

## Token Totals

| mode | input | cached input | uncached input | output | reasoning output |
| --- | ---: | ---: | ---: | ---: | ---: |
| Work Leaf orchestrator | 25,170,611 | 23,895,680 | 1,274,931 | 112,556 | 60,016 |
| Direct Codex sequential | 54,057,480 | 52,143,104 | 1,914,376 | 281,482 | 140,786 |
| Direct Codex worktree pass | 126,364,880 | 122,669,824 | 3,695,056 | 587,557 | 264,581 |
| Direct Codex worktree first attempt | 50,847,223 | 48,858,752 | 1,988,471 | 321,610 | 161,825 |

## Notes

Work Leaf used substantially fewer total and uncached tokens than both direct Codex baselines. The direct sequential run was about 690 seconds faster than the saved Work Leaf baseline while using about 2.1x input tokens and 2.5x output tokens. The passing direct worktree run was the fastest wall-clock baseline, about 1884 seconds faster than Work Leaf, but it used about 5.0x input tokens and 5.2x output tokens.

The passing direct worktree run spent most of its extra review cost on feature 1. Review found several distinct terminal visual-selection edge cases: prompt text yanking, multiline draft filtering, user-entered `chat> ` sentinel handling, and multiline prompt row counting. This improved code quality, but it also made the worktree run token-heavy.

The first direct worktree attempt failed after review while entering linearize because the temporary checkout was under `/tmp` and hit environment storage/quota pressure. The rerun used a repo-local temporary parent and completed.

For the current data set, Work Leaf is the token-efficient baseline, direct worktree is the wall-clock-efficient baseline, and direct sequential is in the middle on time while still much more token-expensive than Work Leaf.
