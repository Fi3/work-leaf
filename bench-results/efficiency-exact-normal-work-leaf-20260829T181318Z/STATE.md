# Study State

## Current Position

The six-run collection, exact token reconciliation, quality scoring, and final analysis are
complete.

## Completed Verification

- Observer unit and integration tests pass, including exact usage and timeout fallback paths.
- Repository format, Clippy, and all-target tests pass.
- A real GPT-5.5/xhigh app-server turn captured exact usage 46 milliseconds after a complete Work
  Leaf directive and then forwarded the original interrupt.
- The fixed source checkout and three runtime binaries match `infrastructure/manifest.json`.
- All six concurrent runs using the normal Work Leaf implementation completed with passing workflow
  reports and passing final format, Clippy, and test gates. Provider interrupt timing was instrumented
  as described in `README.md` and `FINAL-REPORT.md`.
- The original scorer retained scores of 2, 3, 3, 1, 2, and 2 completed features.
- Ten interrupted responses lacked an immediate usage event. Every affected provider thread later
  emitted a cumulative total, so all six final token totals include those responses exactly.
- The incoming and forwarded client byte streams are identical for all six runs. The observer
  changed only when the original interrupt was forwarded, after a complete directive.
- `evidence.json` compares these six observations with six exact direct Codex observations.

## Result

Normal Work Leaf averaged 17,471,532 raw tokens and 1,343,404 uncached tokens. Direct Codex averaged
36,116,382 raw tokens and 1,608,712 uncached tokens. These are exact descriptive reductions of
51.62% raw and 16.49% uncached tokens.

The average implementation quality was not equal: Work Leaf completed 13 of 18 scored features and
direct Codex completed 17 of 18. The study therefore does not claim that the headline average is an
equal-quality efficiency result. See `FINAL-REPORT.md` for the full interpretation.
