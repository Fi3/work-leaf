# Independent phase mediator review

Verdict: PASS for reported arithmetic and protocol adherence. The primary test
does not establish a causal increase in accepted handoffs; the fixed-sequence
secondary test correctly remains untested. No alternate secondary test was run.

## Identities and scope

- `WORK-UNIT-ANALYSIS.json`: SHA-256
  `d27a14b34b5ce6d56173cb82af2c48a2afd836c604c43543d9b36722f4953216`.
- Frozen protocol, `infrastructure/PROTOCOL.md`: SHA-256
  `8d7f29516d37a1e050e539d7098249a2550d61b7eb3b9c30c1f18f82086b9964`.
  The study's `PROTOCOL-WORK-UNITS.md` has identical bytes.
- `PHASE-MANIFEST.json`: SHA-256
  `5282364e43839e8932abfcec440d07b87467e6e7024f99f66b236e662cbdf950`.
- Frozen classification assembly: SHA-256
  `0503a570f2f6c66ad615c1b8ab7f0f8e31786792cb4f147784707e415cdbfdef`.

The final protocol's primary, fixed-sequence, accounting and reporting rules
govern interpretation. The prospective design review provides background,
not an alternative testing rule. No provider call, helper/report edit,
semantic relabeling, token/quality analysis or additional hypothesis test is
part of this review.

## Independent primary arithmetic

Counts below follow frozen run-ID order within each condition. Each count
matches both the report's exact delivered inventory and its reviewed packet.

| Condition | Workflow suffixes | Delivered ACK counts | Total | Mean |
| --- | --- | --- | ---: | ---: |
| Control | 002, 004, 006, 009, 010, 012 | 6, 7, 7, 7, 10, 9 | 46 | 23/3 |
| Incremental policy | 001, 003, 005, 007, 008, 011 | 5, 15, 10, 9, 5, 6 | 50 | 25/3 |

The observed treatment-minus-control mean is exactly `2/3` ACK per workflow.
A separate integer/rational calculation enumerated every choice of three
treatment slots per six-slot block, excluding either single-condition wave.
There are exactly 18 allowed choices per block and 324 distinct joint choices.
The observed allocation is included. Keeping every workflow's observed count
fixed, 117 assignments have a strictly larger contrast and 18 tie it. Thus the
specified inclusive one-sided p-value is `(117 + 18) / 324 = 5/12`, approximately
`0.4166667`, exactly matching the report. No unconstrained allocation space,
Monte Carlo approximation, midpoint, agent-level unit or asymptotic test is used.

The positive observed contrast does not pass alpha `0.025`.
`primary_passes: false`, `secondary.status: not_tested` and its stated
fixed-sequence reason are correct. This phase's alpha allocation is not
refunded by a nonsignificant result. No post-hoc substitution of a semantic
subset, extension or replacement is justified by this outcome.

Workflow 010 remains a completed recorded attempt with exit 1, workflow result
`fail`, and all ten verified ACKs. Its pre-linearization contribution-gate
termination neither deletes its deliveries nor becomes a zero. All twelve
frozen identities remain in the primary allocation and arithmetic.

## Exposure and integrity checks

Independent read-only replay used the actual raw app-server captures for every
run, not only captures listed by individual exposures. Typed accepted request
and reply identities, thread/turn IDs, physical source lines, exact forwarded
text, trace ordering and source hashes agree. Eligible renderer boundaries and
matched trace requests have complete bidirectional set equality. Accepted ACK
counts agree with the observed primary inventory for every workflow.

Exact UTF-8 splicing was reconstructed from the pinned declarative A/B/C and
unchanged span constants in analyzer source SHA-256
`e59e6a22bf1ab09e0aabec86791cefb54cd3f3d38167a4c9822cdce1d09b50d3`.
No analyzer function was executed for the independent arithmetic or replay.
Treatment has 42 delivered A spans, 42 delivered B spans and 50 delivered C
spans, all with the prescribed changes. Control's 41 A, 41 B and 46 C spans
are unchanged. The 96 ACK-validation spans and 118 command-guidance spans
across both arms are unchanged; linearizer policy identity is preserved.
All remaining bytes outside the owned spans are identical to original prompts.
These exposure counts are observed deliveries, not separately tested effects.

The report's integrity and classification error lists are empty. It retains
legacy behavioral configuration drift, while unexplained drift and pending
attestation at finish are false. Its successful trust replay still explicitly
depends on the matching external final configuration; it is not self-contained.
This review independently replays prompt delivery and transformations, not the
entire separately reviewed global-configuration attestation algorithm again.
Interrupted-turn usage gaps remain visible. They do not invalidate an exactly
witnessed ACK delivery and are not treated as zero usage or full token coverage.

## Supported interpretation

The intended policy reached agents, its non-factor prompt bytes were preserved,
and this sample has four more total accepted ACK deliveries in the incremental
arm. The preregistered test does not establish the hypothesized policy-to-handoff
increase. It also does not prove that the policy has no effect, and the untested
secondary cannot establish a causal increase in genuinely remaining work.

Individual reviewed edit/check/repair chains remain descriptive evidence.
ACKs are not completed response counts, and extra local handoffs do not imply
a net whole-workflow response or token increase. This result neither explains
a percentage of the historical approximately 50% savings nor overturns that
separate endpoint. Shared-capacity interference limits interpretation to the
frozen mixed-wave assignment regime; direct effects and spillovers are not
separately identified. The exact causal mechanism remains unestablished by
this particular confirmatory mediator test.
