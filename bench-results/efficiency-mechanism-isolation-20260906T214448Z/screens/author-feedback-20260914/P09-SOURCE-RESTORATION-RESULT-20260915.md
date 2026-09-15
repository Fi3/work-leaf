# Exact original pre-integration source is recoverable for all three runs

Checked 2026-09-15 03:25–03:40 UTC, within the twenty-minute local screen.
No model generation or original-artifact mutation. Research remains 80/10/90.

| Run | Accepted commits | Original pre-integration HEAD | Source tree |
| --- | ---: | --- | --- |
| 004 | 19 | `7b16e5fbfa3f2cfd604c31405771a916f29cb938` | `f4b66255bb657c0c75d1f25cfbefa28ea3d1fcc1` |
| 005 | 19 | `138e1b51ecd7c989c8f48b37db387fbbe3fe6dac` | `5512929971221609a63ff011208bcdda9df1af4c` |
| 006 | 17 | `67fe0fd538d138b9fe41c2e84a8e93342c26966b` | `f3b76e57000f5407de646ad7f2221f288ffc558c` |

All 55 accepted commit identities, their parent chain, checkpoint pins and clean
tracked state match. Run 004's 19 objects were available. The 36 missing objects
from 005/006 are recovered from their original accepted proposals and receipts
using the exact qualified host parser, not regenerated implementation work.

The frozen driver's `Host.apply` constructs the message from feature and retained
reason, and its bootstrap sets `Work Leaf Bench <bench@example.com>`. An available
same-batch object confirms the timestamp timezone. Searching only the original
00:15–01:24 UTC interval recovers the timestamp only when the complete commit
object hashes to the already-recorded original SHA. A different tree, parent,
message or metadata fails this identity gate. This is exact object recovery, not
an assumption that a similar tree or guessed metadata is equivalent.

Four fail-first/local identity and rejection tests pass. Three fresh bundles,
76,355 bytes total, retain the recovered chains with the original base as their
declared prerequisite; `git bundle verify` passes for each. The working recovery
checkouts are temporary, and their aggregate refs are not asserted to be the
original clone's refs. A future integration checkout must clone the same retained
`infrastructure/driver-source` and import its original chain before generation.

The first graph comparison used the wrong text format; the second incorrectly
compared an ancestor-only graph with the archived all-refs graph. A command
construction also failed to parse. All commands and failures remain alongside
the final accepted ancestor-record comparison. None launched a model or changed
an admitted source artifact.

Evidence: [whole roster](source-restoration-014/WHOLE-ROSTER-RESULT-20260915.json),
[first exact recovery](source-restoration-014/FIRST-EXACT-RESULT-20260915.json),
[commands and failures](source-restoration-014/COMMANDS-20260915.json),
[bundle identities and verification](source-restoration-014/BUNDLES-20260915.json).
`cargo fmt`, warning-free all-target/all-feature Clippy and Cargo tests pass.
This local object-restoration helper affects no agent launch or workflow; no
real-agent verification is claimed or needed for its hashing operation.

## Next bounded action

Prepare integration-only continuation for the same three qualified prefixes,
with BUG009's verified input repair. Preserve every old partial/unqualified
integration result and cost. No author/review rerun, selected-only 004 run,
ordinary control or full replacement is admitted by this restoration result.
The fresh integration attempts need a separate prospective admission, faithful
plan/acceptance prompts, actual input qualification and complete accounting.
