# P10 — all actual new admissions accounted and qualified

The complete retained roster is ADMISSION-ROSTER.json: four actual admissions,
43 distinct recorded responses, 3,567,272 recorded raw tokens and two unknown
unfinished tails. There is no finite asserted upper bound for those tails.
This is a complete audit deliverable, not four successful experiments.

| Admission | Recorded raw | Completed responses | Input / outcome |
| --- | ---: | ---: | --- |
| P01 input-001 | 15,709 | 1 | Completed diagnostic, missing startup catalog; failed qualification |
| P01 input-002 | 15,873 | 1 | Completed diagnostic, exact initial base/developer objects |
| P03 B-only | 1,507,326 | 19 | Five complete outer turns plus stopped sixth; resumed-input mismatch |
| P04 A+B | 2,028,364 | 22 | Five complete outer turns plus stopped sixth; resumed-input mismatch |

The unchanged audit_compaction.py::audit_rollout core was invoked once for each
closed new native thread. Existing P01 audit files are reused; P03/P04 use the
single saved NATIVE-AUDIT.json. The roster extraction invokes the core zero
times. No original-baseline native usage audit is rerun.

All 43 recorded response IDs are unique across the four observations. Every
completed public usage record matches its native distinct-response sum in input,
cached input, output and reasoning output. No compaction markers or duplicate
response records occur. Raw equals input plus output; cached/reasoning fields
are subsets, not additional charges. Monitor/public/native totals are not added
together. Failed and partial outcomes remain included.

The two new screens have 181 saved resource samples each and independently stop
at their 900-second walls. Native/public ownership and exact prompt joins pass;
all 82 admitted source endpoints per screen match. Their last native turns lack
a completion/usage record. Recorded zero for those unfinished turns is not zero
actual cost, and an earlier turn's exact usage cannot price their tails.

## Input and factor qualification

Actual base objects, initial developer contents, model/effort, source/CLI, native
tools and core context settings match the saved reference. The context_window
metadata difference in FINAL-INPUT-SOURCES.json is its unique window_id, not a
changed numeric token limit. Run/thread/path/date identities and environmental
timing remain explicit observation differences.

The final corrected CATALOG-BOUNDARY-QUALIFICATION.json rejects full resumed-
input equivalence: two optional skill descriptions disappear at new author
turn 2, versus reference author turn 5. P04 restores them at turn 3, but matching
later text does not remove prior context differences. BUG005's result supersedes
the interpretive claim of the preserved pooled-profile receipts.

P03's actual host results omit B guidance and retain commits, command statuses,
failure details and allowed repair. P04 additionally accepts real test-only
publication followed by executed RED. Those activation facts and exact source
spans survive; qualified whole-episode attribution does not. No observation is
dropped or relabelled as a completed null effect.

## Scope and limits

P06–P08 and P09's three workflows are unlaunched, not unreported observations.
Earlier R13 usage belongs to its retained R17 audit, outside this roster.
P10 is complete for every actual admission under the current frozen allocation.
Any subsequently approved new observations require their own explicit audit
coverage; this completed receipt is not a pre-audit of future runs.

The historical 45.38%–51.62% saving is not re-audited or refuted here. Its combined
causally explained share remains NOT ESTABLISHED. The audit supports preserving
these costs and rejecting causal escalation, not declaring the study complete.
