# Single-attempt launcher independent review

The complete new launcher and six tests have no introduced code blocker for the
declared trusted, source-frozen diagnostic admission. This review neither admits
nor launches the real diagnostic. No additional execution belongs to this receipt.

| Reviewed source | SHA-256 |
| --- | --- |
| `bounded_launch.py` | `f181729b5c26e4cfc85477c82918fb9468fb4a9482d8700c36134ecd3a175db1` |
| `test_bounded_launch.py` | `d127e4fd8c70b4c462a6673cd9aacfd86885c7893f94f4001e2fdd962f2e111b` |
| `LAUNCHER-VALIDATION.md` | `0d6bf5500a639a5f929f40537d16487efadf68e39698a9970ba58a8de2f426e7` |
| `POSTCAPTURE-OPERATOR-SCOPE.md` | `c5f5c78a9bb530533431f1005e4d828006e205ed90b7a3e5ddc7da58163b50f0` |

Independent command already completed:

```sh
python -B -m unittest discover -s bench-results/efficiency-mechanism-isolation-20260906T214448Z/preflight/c15-real-diagnostic -p test_bounded_launch.py -v
```

Receipt `375878`: **6 passed**, 0.033 seconds. Only local synthetic Python children
ran. The source/validation hashes above were rechecked without repeating tests.

`run` parses the exact admission and environment bytes whose hashes were checked;
it does not reread a mutable file to choose another argv or environment. The
source inventory is checked before the attempt and again after the waited command,
including the admission itself at the final endpoint. A post-execution integrity
failure stays separate from the actual supervised command return code.

Existing attempt, terminal, stdout or stderr paths—including dangling symlinks—
reject another execution. `publish` uses exclusive create-new and fsync. This is
not atomic rename or a crash transaction: partial publication remains a retained,
unretriable attempt. The full stdout/stderr files are separate from the terminal
metadata. A real spawn error has no fabricated child exit, while a nonzero waited
command remains terminally published with its actual integer return code.

The typed terminal IDs, factor condition versus observer condition, timezone-aware
start/finish and launcher exit satisfy the plan's generic publication contract.
Their time scope is the supervised command, not a provider's internal turn. An
outer timeout return code is not independently proved private-child closure.

Root's final admission must bind the intended complete executable/source chain,
environment values, canonical roots and literal argv containing
`/usr/bin/timeout --kill-after=5s 300s <frozen-harness> …`. The launcher deliberately
does not infer a timeout for arbitrary argv, construct provider arguments or
authorize replacements. It is not a hostile-input parser or continuous source
monitor. Its additional source work is linear in checked bytes; no introduced
O(n²) algorithm was found.

The selected postcapture method is the finite operator witness and existing pinned
pure helpers in `POSTCAPTURE-OPERATOR-SCOPE.md`. Original observer reports remain
separate; the v5 C21 collector is inapplicable. No general collector, token ledger,
provider call or accounting replay is created or executed by this review.
