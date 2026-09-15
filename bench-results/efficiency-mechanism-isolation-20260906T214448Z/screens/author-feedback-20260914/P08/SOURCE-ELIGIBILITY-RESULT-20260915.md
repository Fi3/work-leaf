# P08 — native route shares downstream code, not complete historical coverage

Checked 2026-09-15T06:53:04+00:00; selected at06:45:49, 435 local seconds.
Outcome: the complete P08 generation gate remains unqualified. Zero model
responses, no runtime changes and no audit-core execution.

## Source comparison

The complete source diff compares these frozen study-relative files:

- infrastructure/driver-source/bench-three-features-direct-common,
  SHA256489289165601e00a545f76ab0631d5e0f48d644b928376488677e1875651e386.
- phases/standalone-global-hunk-pilot-01/infrastructure/drivers/bench-three-features-serialized-host-custody,
  SHA256a621b4ee2ae74dbef03d6c3b29b8dbaaf5842d27e186f61089ce16d658d1526f.

The standalone driver changes author implementation/fix custody and policy,
source-helper paths, artifact retention and associated checks. It retains the
native review, review-clean parser, validation guidance, sequential scheduler
and plan/accept integration functions. The source diff does not prove identical
realized later work: the inputs to those shared functions depend on authors.

The exact chain is implementation_prompt → run_host_author → Host.apply/command
→ run_feature_cycle → review_prompt. A finding resumes the original author with
fix_prompt, then review_prompt receives that fix's text.txt. The standalone
run_host_author copies Host.evidence into text.txt; Host.evidence (host_custody.py
504) accumulates actual edit/check receipts and output. The native path instead
uses the author's actual final response and a post-author commit. Consequently
changing custody also changes later review evidence and commit/history shape.
Those are declared package consequences, not independently held-fixed results.

The source locations in the standalone driver are run_host_author937,
implementation_prompt1075, fix_prompt1098, review_prompt1124,
linearize_plan_prompt_sequential1158, linearize_accept_prompt_sequential1182,
run_feature_cycle1213 and run_sequential_bench1281. The full diff, not matching
function names alone, establishes shared source.

## Why this does not admit another R06

R06 is a real completed initial author: 2,312,888raw, 117,639 below R04, with a
catalog mismatch. The qualified P09 route resolves a launch problem but supplies
no new positive conditional native-custody signal. An otherwise identical repeat
of that weak initial screen would still omit the evidenced later-stage paths.
No such repeat is selected or renamed P08.

The historical J10 package does expose complete reviews/integration, but retains
the author policy and couples scheduling, direct reads and bounded continuation.
Its35,659,265 versus19,311,710 means do not isolate the residual package. Its
actual105 interrupt requests remain interruptions after advancing usage; they
are not natural native final turns. EVIDENCE-PRIOR-PROTOCOL-AND-INTERRUPTION.md
provides that source/activation chain. Source reuse in the newer standalone
driver cannot erase these actual historical boundaries.

EVIDENCE-H-LIFECYCLE.md also retains actual historical review/fix and title
work, plus specific absent paths. Those absent/shared exclusions remain valid;
the exercised paths do not become absent because the available small screen
does not reach them. P06/P07's connected refresh/continuation boundaries remain
separate unfinished obligations.

## Disposition

P08 remains BLOCKED in TODO; no causal zero, complete residual bound or completed
research count is claimed. This source-only attempt stops early instead of using
its30-minute ceiling to construct an unapproved interception/state-replay engine.
The next useful retained-data task is P12's exact accounting-component breakdown
of the qualified A+B net, not another generic history census or new baseline.

Additional source pins: P02/FACTOR-REFERENCE-MANIFEST.md
74c4581e1e6906b064c13176fe7353a9cb15b32c70eaa6762322a68ec6186e4f;
progress-results/R06.md
6b2915e9a4a3b37c25d0db52876334f56368a07d3d579c62aa7d6880a37f812a;
EVIDENCE-PRIOR-PROTOCOL-AND-INTERRUPTION.md
54bd7755382d0976a2dd4306c50fbb49d001f4259dcfb6860875de47c3d1dbe1;
EVIDENCE-H-LIFECYCLE.md
556da269740ff8a33edd25d351ea913ec1a4f77bd2524f099eb26bd8753c55b2.
