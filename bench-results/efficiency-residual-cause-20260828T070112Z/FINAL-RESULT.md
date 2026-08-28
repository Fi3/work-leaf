# Result Status

The saved benchmark totals do not support an exact Work Leaf token-reduction percentage. A later
hash-locked bound does establish a raw-token saving for selected fair 3/3 observations; see
[`../efficiency-point7-bounded-accounting-20260828T142614Z/FINAL-RESULT.md`](../efficiency-point7-bounded-accounting-20260828T142614Z/FINAL-RESULT.md).

Static inspection found a Work Leaf-only accounting error: Codex is interrupted immediately after a
complete orchestrator directive, and many such model calls have no recorded current usage. The direct
Codex path waits for terminal usage. The earlier 57.33% raw and 35.66% uncached reductions are
therefore withdrawn, and the three-disabled comparison cannot allocate savings reliably.

The candidate-quality evidence remains useful. Normal Work Leaf passed 8 of 9 external feature
checks and direct Codex passed 7 of 9, with two fully passing candidates in each group. The bounded
Point 7 result proves a raw saving in one normal and one all-three-disabled 3/3 observation, but its
exact size remains unknown. The likely remaining source is Work Leaf's smaller number of iterative
model/tool cycles, not one isolated text compactor; that explanation still requires attribution.

The complete code-grounded audit, counter-hypotheses, proxy scale check, and likelihood ratings are
in [STATIC-AUDIT.md](STATIC-AUDIT.md). `evidence.json` preserves the original recorded values and
artifact references; those token values are incomplete for interrupted Work Leaf calls.
