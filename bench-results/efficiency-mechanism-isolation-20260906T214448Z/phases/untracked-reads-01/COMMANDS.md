# Untracked-read phase operation

This phase has twelve fixed workflows, six per condition, in four mixed waves of three.
The one-time allocation is `ALLOCATION.json`; no redraw or replacement is permitted.
The prepared manifest SHA-256 is
`706c0b13d62376b3ea10c4604bb63957c271a1a7224d39b27db20fa24e2da263`.
The schedule SHA-256 is
`05fa9c780d2e2203af0454990597fb6369082ddefb49e46887b663341ed41687`.
All 114 manifest files pass root endpoint hashing and the frozen runner's `verify` mode.
Preparation is not generation: `RUN-ONCE/admission.json` belongs only to the explicit run command.

## Execution paths and admission gates

```sh
V3_PHASE=/home/user/src/work-leaf/bench-results/efficiency-mechanism-isolation-20260906T214448Z/phases/untracked-reads-01
V3_HELPERS="$V3_PHASE/infrastructure/evidence/bench-results/efficiency-mechanism-isolation-20260906T214448Z"
V3_CACHE=$(mktemp -d /tmp/work-leaf-v3-execution-cache.XXXXXX)
```

The original clean driver snapshot remains the manifest's executable driver source; its five
scripts and Git identity are checked. Runtime, observer and subscription wrapper execute from the
phase's frozen paths. The runtime root is `/tmp/work-leaf-untracked-reads.jUcFKF`.
Every Python command uses `-B` and an initially empty dedicated cache prefix: `-B` alone does not
prevent loading existing bytecode. No credentials or global configuration contents are copied.

Admission requires completed independent prepared-source review, passing exact-source tests,
unchanged manifest/binary/helper identities, and the subscription resource recheck after
2026-09-07 07:53:49 UTC. `QUOTA-PREWAIT.json` retains the read-only pre-reset check, with zero
generation requests and unchanged global configuration. The quota check uses only logged-in CLI
initialization and `account/rateLimits/read`, never login, purchases, earned-reset consumption,
messages or model turns. Any later check has a separate create-new receipt.

```sh
python3 -B -X "pycache_prefix=$V3_CACHE" "$V3_HELPERS/runner_untracked_reads.py" verify --phase-root "$V3_PHASE"
# Execute once, only after all admission gates pass:
python3 -B -X "pycache_prefix=$V3_CACHE" "$V3_HELPERS/runner_untracked_reads.py" run --phase-root "$V3_PHASE"
```

The supervisor launches three independent complete workflows concurrently, then waits for that
wave's terminal outcomes. Its unchanged limits, observer raw-response capture, 1,000 ms maximum
usage grace and forward-on-resumed-output behavior remain frozen. Operational monitoring may inspect
processes, workflow status and integrity; it may not compute interim contrasts or token rankings.
All outcomes remain retained, including failed and withheld rows.

## Closed-phase derivations

Only after all identities are terminal and the phase result is published, independently replay
`trust_work_units.py::audit` from its verified frozen source against the retained final global
configuration. The primary also performs that replay. Its dependence on the matching external
configuration is explicit; no configuration archive is substituted.

```sh
python3 -B -X "pycache_prefix=$V3_CACHE" "$V3_HELPERS/analyze_untracked_reads.py" \
  --manifest "$V3_PHASE/score-manifest.json" --sessions-root /home/user/.codex/sessions \
  --output "$V3_PHASE/UNTRACKED-READ-ANALYSIS.json"
python3 -B -X "pycache_prefix=$V3_CACHE" "$V3_HELPERS/audit_read_mechanism.py" \
  --manifest "$V3_PHASE/score-manifest.json" --analysis "$V3_PHASE/UNTRACKED-READ-ANALYSIS.json" \
  --sessions-root /home/user/.codex/sessions --output "$V3_PHASE/READ-MECHANISM-CENSUS.json"
python3 -B -X "pycache_prefix=$V3_CACHE" "$V3_HELPERS/analyze.py" score \
  --manifest "$V3_PHASE/score-manifest.json" --scorer-config "$V3_PHASE/infrastructure/SCORER.json" \
  --output "$V3_PHASE/QUALITY.json" --timeout-seconds 900
```

The single primary retains the complete fixed population, accounting intervals and conservative
randomization test. The census supplies exact delivery/native-item/response charge references,
unknowns and public retrieval candidates, not another endpoint or a causal-share estimate. Public
tool arguments/results still require semantic review before claiming an actual bundle read.
The old token-comparison and ranked attribution CLIs are not v3 replacements. Quality is retained
without selecting observations; the user's no-quality-loss assumption remains the interpretation
boundary. Earlier phases and reports remain unchanged.
