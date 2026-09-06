# Supplemental pair analysis

This single pair has a **conditional, post-hoc raw-token saving envelope of 41.51%–51.53%**, but it does not establish equal-quality savings: the frozen independent scorer passes 3/3 checks for Direct and 2/3 for Work Leaf. The frozen primary Work Leaf classification remains **`unbounded_accounting_gap`**, without a finite lower bound on savings. External configuration identity also drifted during the run; see [post-run integrity audit](POST-RUN-INTEGRITY-AUDIT.md).

The [frozen pair analysis](pair-analysis.json), [quality results](quality.json), and [supplemental machine-readable comparison](supplemental-pair-analysis.json) preserve both observations and all outcomes. Scoring and analysis each ran once, without provider calls, benchmark retries, or frozen-helper changes.

## Full-workflow accounting

Direct's strict observer is complete: 37/37 invocations, no interrupted usage gaps, and exact full-workflow totals. Work Leaf has four missing tails; the [separate tail audit](SUPPLEMENTAL-TAIL-AUDIT.md) supplies its conditional post-hoc envelope while retaining the frozen rejection.

| Metric | Exact Direct total | Conditional Work Leaf envelope | Conditional saving versus Direct |
| --- | ---: | ---: | ---: |
| Raw input + output, primary | 47,013,757 | 22,786,969–27,498,969 | 41.5087%–51.5313% |
| Uncached input + output, secondary | 1,906,045 | 1,719,449–6,431,449 | −237.4238%–9.7897% |

For each metric, saving is `100 × (Direct − Work Leaf) / Direct`, using the **full-workflow Direct denominator**, not selected stages or successful-feature counts. The raw token difference is 19,514,788–24,226,788; the secondary difference is −4,525,404–186,596. Accounting widths are 10.0226 and 247.2135 percentage points respectively. Thus the secondary result does not establish a saving.

The conditional Work Leaf upper is its recorded lower plus four responses × 1,178,000 tokens. This requires the frozen model limits to apply, complete visibility of normal response/tool/compaction boundaries, and no hidden retries, opaque upstream subresponses, or unobserved preemption. Exact missing usage is not recovered. These envelopes are not sampling confidence intervals or unconditional billing guarantees.

## Quality is a separate outcome

Both workflow reports say `pass`. The frozen canonical scorer reports:

| Check | Direct | Work Leaf |
| --- | --- | --- |
| Completion | Pass | Pass |
| Status | Pass | Pass |
| Visual | Pass | Fail |

The retained [Work Leaf visual log](scorer/logs/work-leaf-001/visual.log) reports an assertion failure at `tests/quality_visual.rs:26`: `has_visual_status(&character.render_frame(), "left", "char")`. This is the observed failure, not a diagnosis that the entire visual feature is absent. There is no scorer rerun or repair in this batch.

## Stage accounting and activity

Direct stage totals are exact; Work Leaf stage totals are recorded lower bounds, without allocation of missing-tail usage. All stages below remain in the workflow denominator.

| Stage | Direct raw | Work Leaf recorded raw | Direct uncached + output | Work Leaf recorded uncached + output |
| --- | ---: | ---: | ---: | ---: |
| implementation | 30,149,576 | 14,444,366 | 1,033,544 | 1,024,974 |
| review | 9,500,669 | 4,319,091 | 601,341 | 504,947 |
| linearization | 7,363,512 | 3,962,985 | 271,160 | 187,497 |
| title | 0 | 60,527 | 0 | 2,031 |
| Total | 47,013,757 | 22,786,969 | 1,906,045 | 1,719,449 |

The next table uses **Direct / Work Leaf** counts. Reported usage advances are changes in retained cumulative-usage records, not an exhaustive count of model calls. Tool counts are recorded actions, not equal-cost units of work.

| Stage | Reported usage advances, D / WL | exec_command actions, D / WL | apply_patch actions, D / WL |
| --- | ---: | ---: | ---: |
| implementation | 247 / 133 | 391 / 93 | 70 / 0 |
| review | 98 / 58 | 239 / 82 | 0 / 0 |
| linearization | 43 / 48 | 115 / 75 | 3 / 5 |
| title | 0 / 3 | 0 / 0 | 0 / 0 |
| Total | 388 / 242 | 745 / 250 | 73 / 5 |

Observer command totals are 766 / 285, including 21 / 35 locked commands. Repeated-command counts are 182 / 12; validation-command counts are 69 / 23. Work Leaf additionally reports 16 structured edit submissions, so its five `apply_patch` actions do not represent its total editing activity. These measurements describe the paths taken; they do not assign percentages of savings to orchestration mechanisms, explain away the quality difference, or establish causation.

## Scope, integrity, and reproducibility

There is one observation per condition, not a repeated estimate. Unequal scorer outcomes, missing Work Leaf usage, post-hoc tail assumptions, and [configuration identity drift](POST-RUN-INTEGRITY-AUDIT.md) prevent an equal-quality, stable-configuration causal claim. The drift's behavioral impact is not established by this arithmetic report. The frozen unbounded result remains alongside the supplemental comparison.

The JSON pins the completed-run manifest/result, frozen scorer/policy/helper, both reports and strict analyses, canonical score/analysis, prior tail supplement, all six scorer logs, and the 15 external rollout sources used for stage proxies. Capture pins are retained transitively in the frozen pair analysis and tail audit. The following provider-free query, from this batch directory, verifies local pins, exact Direct eligibility, arithmetic, stage sums, and retained quality outcomes; it does not rerun scoring.

```sh
python3 -B - <<'PY'
import hashlib, json, math
from pathlib import Path
b = Path.cwd()
e = json.loads((b / "supplemental-pair-analysis.json").read_text())
for rel, digest in e["source_sha256"].items():
    assert hashlib.sha256((b / rel).read_bytes()).hexdigest() == digest, rel
p = json.loads((b / "pair-analysis.json").read_text())
rows = {r["condition"]: r for r in p["observations"]}
assert rows["direct"]["measurement"]["status"] == "exact"
assert rows["work-leaf"]["measurement"]["status"] == "unbounded_accounting_gap"
a = b / "runs/direct-001/direct-001-three-feature-sequential-bench-artifacts/observation/analysis.json"
assert json.loads(a.read_text())["capture_complete"] is True
for metric, c in e["comparison"].items():
    d = rows["direct"]["measurement"]["recorded_usage"][metric]
    assert c["direct"] == dict(lower=d, upper=d)
    lo, hi = c["work_leaf"]["lower"], c["work_leaf"]["upper"]
    assert hi - lo == 4 * 1178000
    assert c["saving_tokens"] == dict(lower=d-hi, upper=d-lo)
    assert math.isclose(c["saving_percent"]["lower"], 100*(d-hi)/d)
    assert math.isclose(c["saving_percent"]["upper"], 100*(d-lo)/d)
    for r in rows.values():
        assert sum(s[metric] for s in r["stage_recorded_usage"].values()) == r["measurement"]["recorded_usage"][metric]
q = {r["condition"]: r for r in json.loads((b / "quality.json").read_text())["runs"]}
assert q["direct"]["checks"] == dict(completion="pass", status="pass", visual="pass")
assert q["work-leaf"]["checks"] == dict(completion="pass", status="pass", visual="fail")
print(json.dumps({m: c["saving_percent"] for m, c in e["comparison"].items()}))
PY
```

The next action is pause. No additional observations are admitted by this report.
