# Work-unit phase post-run command map

This operational note applies to the admitted `work-units-01` phase, whose manifest SHA-256 is
`5282364e43839e8932abfcec440d07b87467e6e7024f99f66b236e662cbdf950`. It records source inspection and
provider-free compatibility checks performed on 2026-09-07, without reading any prospective
work-unit outcome. It is not an amendment to the frozen protocol or an interim analysis.

## Order and admission boundaries

1. Wait for all twelve planned workflows and their observer captures to close, and for the
   supervisor to publish `PHASE-RESULT.json`, `score-manifest.json`, and final configuration/trust
   evidence. Preserve failures, unlaunched rows, source incidents and original configuration flags.
   A stopped/incomplete phase is retained and reported, not completed with replacement observations.
2. Verify the complete frozen file inventory and that the terminal score manifest names this
   admitted manifest. Run the two supplemental audits and canonical final-quality scoring below.
   These commands inventory individual outcomes; they do not compute condition contrasts.
3. Prepare the exact accepted-ACK inventory through `analyze_work_units.py::prompt_inventory`
   only, not its whole-manifest entry point. Supply every ACK exactly once to independent semantic
   coding. Freeze the complete classifications and supporting evidence before any condition contrast.
   The protocol's unresolved zero-to-one labels remain unresolved; quality is not an inclusion filter.
4. Only after that freeze, run `analyze_work_units.py` with the classification file, followed by
   `report_work_unit_tokens.py`. The latter independently invokes the mediator's whole-manifest
   analysis and therefore also computes a primary contrast internally; it is not a pre-coding route.
5. Interpret and report the phase facts together, retaining unknown coverage, raw-token bounds and
   every quality result. Phase closure does not itself require a final pause. No automatic replacement
   or extension is admitted; further investigation follows the latest user authority and separately
   frozen prospective admission.

## Exact commands after closure

Use the phase's preserved evidence tree. Relative dependency paths inside `SCORER.json` resolve
against that tree through the frozen helper's `ROOT`; no configuration rewrite is needed. A fresh
cache prefix and `-B` prevent timestamp-based cached bytecode from supplying an old dependency.

```sh
WU_STUDY=/home/user/src/work-leaf/bench-results/efficiency-mechanism-isolation-20260906T214448Z
WU_PHASE="$WU_STUDY/phases/work-units-01"
WU_FROZEN="$WU_PHASE/infrastructure/evidence/bench-results/efficiency-mechanism-isolation-20260906T214448Z"
WU_CACHE=$(mktemp -d /tmp/work-leaf-v2-postrun.XXXXXX)

python3 -B -X "pycache_prefix=$WU_CACHE/compaction" "$WU_FROZEN/audit_compaction.py" \
  --manifest "$WU_PHASE/score-manifest.json" \
  --sessions-root /home/user/.codex/sessions \
  --output "$WU_PHASE/COMPACTION-AUDIT.json"

python3 -B -X "pycache_prefix=$WU_CACHE/input-attribution" "$WU_FROZEN/audit_input_attribution.py" \
  --manifest "$WU_PHASE/score-manifest.json" \
  --sessions-root /home/user/.codex/sessions \
  --output "$WU_PHASE/INPUT-ATTRIBUTION.json"

python3 -B -X "pycache_prefix=$WU_CACHE/quality" "$WU_FROZEN/analyze.py" score \
  --manifest "$WU_PHASE/score-manifest.json" \
  --scorer-config "$WU_PHASE/infrastructure/SCORER.json" \
  --timeout-seconds 900 \
  --output "$WU_PHASE/QUALITY.json"
```

These reports are create-new. Do not rerun over a materialized result or silently replace a partial
report. Canonical scoring also writes its logs under `scorer/logs` beside `QUALITY.json`; preserve
those logs. Scoring materializes saved bundles and diffs in temporary checkouts, runs the three
frozen feature fixtures, and does not modify the candidate artifact or start an agent.

The following commands have the additional classification-freeze prerequisite. The classification
filename below is an operational output name, not a new schema or inference rule.

```sh
python3 -B -X "pycache_prefix=$WU_CACHE/mediator" "$WU_FROZEN/analyze_work_units.py" \
  --manifest "$WU_PHASE/score-manifest.json" \
  --classifications "$WU_PHASE/WORK-UNIT-CLASSIFICATIONS.json" \
  --output "$WU_PHASE/WORK-UNIT-ANALYSIS.json"

python3 -B -X "pycache_prefix=$WU_CACHE/tokens" "$WU_FROZEN/report_work_unit_tokens.py" \
  --manifest "$WU_PHASE/score-manifest.json" \
  --output "$WU_PHASE/WORK-UNIT-TOKENS.json"
```

Do not invoke the old helper's `analyze` mode or the canonical scorer's standalone comparison
entry point. `analyze.py score` calls `STRICT.score_all` directly, avoiding both v1 prompt analysis
and the scorer's old Direct-versus-WL comparison functions.

## Compatibility and limitations

- `audit_compaction.py::audit_manifest` / `audit_run` consume the frozen model, effort and `runs`
  array, then `run_id`/`id`, condition, artifact and launch outcome. The experiment version,
  prompt-trace schema and new condition name are not parsed or transformed.
- `audit_input_attribution.py::audit_manifest` / `audit_run` consume the same generic rows. Their
  pinned dependencies read native rollout metadata and exact raw response metadata. The old
  `analyze.py` dependency is used only for capture provenance, not v1 prompt matching.
- `batch_analysis.py::score_all` iterates all manifest runs, retaining scorer errors. Its canonical
  `score.py::score_entry` reads `id`, workflow, condition, report and artifact, plus the fixed source
  repository/base/model/effort from the manifest. The v2 runner publishes these fields unchanged.
- Quality's legacy `measurement.usable` and usage display are diagnostic fields, not this phase's
  token-accounting admission or an exclusion rule. The separate secondary token bridge supplies the
  unchanged prospective accounting envelopes; the quality scorer supplies the feature checks.
- Supplemental exact-prefix agreement is not exhaustive hidden-call coverage. Native-only response
  identities, compaction-marker adjacency, partial tails and nonzero reconciliation residuals remain
  visible. Neither supplemental audit automatically retotals the primary/secondary token ledger.
- Native sources must remain under the supplied sessions root and match the observer's saved SHA.
  An altered/missing rollout produces unknown coverage, not an inferred zero. Full source/configuration
  and prompt-delivery readiness comes from the independent v2 gate, not supplemental status alone.

## Provider-free checks

The phase-frozen copies passed all 60 supplemental/base tests and all 22 strict accounting/scorer
tests. A separate synthetic twelve-row v2 manifest retained every identity and condition in both
audits, including a failed and an unlaunched outcome; absent artifacts stayed unknown. No prompt
trace was read. A mocked canonical `score_entry` accepted the v2 row shape, while the actual frozen
`load_scorer` verified the scorer and fixture hashes and their phase-local paths. This validates
parsing and path compatibility, not future quality results; no fixture process or provider ran.

Frozen dependencies checked:

| Component | SHA-256 |
| --- | --- |
| Compaction audit | `dd5127f937ec1cc4d41a443ba7640fca93e82484f345ac8c56aeef5325d68153` |
| Input attribution audit | `ad6861e834c09f0027305abe223dbe80ae03401740abdae6288580d880bb0740` |
| Shared legacy analyzer | `2bb28891a2e158d51cf577bcf7c1fc2781065e38dc5ec4c6c30f7d4c146f2f78` |
| Strict accounting/scorer adapter | `dacbfc8416467312c8a447ac1cd846da3e1f78da96f3733ee16c3dad1781d7c3` |
| Canonical scorer | `c0a4e951e96d7da53a6d414a7677176183cae6e30d0bbfab92069d5082865162` |
| Phase scorer configuration | `4e35a729f53f030eb1ebf73eaa8f27bd6b66fed67411a94f5339dcfb0562ab70` |
