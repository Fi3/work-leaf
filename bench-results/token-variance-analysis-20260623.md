# Abstract

This document analyzes the current Work Leaf three-feature benchmark baseline group and evaluates whether the observed token variance is expected for this specific multi-agent workflow. The candidate baseline group contains 32 passing bench reports. One passing report is removed from parameter fitting as an outlier: `bench-results/parallel-followup-6-20260623T150613+0200/runs/run-5/three-feature-bench.jsonl`, which used `29.05M` `input + output` tokens. The fitted baseline therefore uses 31 successful full-workflow runs.

The fitted 31-run baseline has mean `input + output = 13.45M`, standard deviation `2.15M`, coefficient of variation `16.0%`, and observed included range `9.49M` to `17.89M`. A simple 6-session/2-commit versus 7-session/3-commit mixture is useful but incomplete. The current preferred model is a work-shape mixture over compact 6-session runs, compact 7-session runs, and high-change runs. Under this model, the expected central 95% interval for successful full-workflow `input + output` is approximately `9.73M` to `18.28M` tokens.

The outlier is excluded from ordinary parameter fitting because it is structurally extreme relative to the fitted baseline: it is `7.3` fitted standard deviations above the included mean, `62%` above the next-highest included run, and its linearize phase alone used `22.61M` input tokens, while the largest included linearize input was `11.57M`. It remains an outlier observation and should be investigated separately from the ordinary baseline.

# Scope And Data

This analysis uses saved bench result artifacts only.

Candidate result roots:

- `bench-results/parallel-baseline-9-20260623T083811+0200`
- `bench-results/parallel-extra-6-20260623T105409+0200`
- `bench-results/parallel-baseline-6-20260623T135939+0200`
- `bench-results/parallel-followup-6-20260623T150613+0200`
- `bench-results/parallel-followup-6-20260623T161812+0200`

Candidate baseline group:

```text
passing candidate reports: 32
excluded fitted outliers:   1
included fitted baseline:  31
```

The earlier stalled report at `bench-results/parallel-extra-6-20260623T105409+0200/runs/run-4/three-feature-bench.jsonl` is not part of this passing candidate group because it failed before linearize and is structurally incomparable to full successful runs.

Valid successful result definition:

- `result == pass`
- `token_usage` is not `null`, not `unavailable`, and not empty
- every reported session has strictly positive `input`, `cached_input`, `output`, and `reasoning_output`
- the run reached final code quality checks and passed them

Primary token metric:

```text
total = input + output
```

`reasoning_output` is reported separately and is not added a second time.

# Literature Assumptions

All external claims in this section are grounded in research papers or arXiv preprints. Vendor docs, forum posts, and anecdotal reports are not used as literature evidence.

| ID | Assumption | Citation | How The Assumption Is Used |
| --- | --- | --- | --- |
| L1 | LLM evaluation variance must be measured before deciding whether score differences are meaningful. | Madaan et al., [Quantifying Variance in Evaluation Benchmarks](https://arxiv.org/abs/2406.10229), arXiv:2406.10229. | Supports repeated-run empirical variance instead of comparing isolated runs. |
| L2 | Agentic coding token consumption is highly stochastic across same-task trajectories. | Bai et al., [How Do AI Agents Spend Your Money? Analyzing and Predicting Token Consumption in Agentic Coding Tasks](https://arxiv.org/abs/2604.22750), arXiv:2604.22750. | Directly supports expecting large token variation in repeated Work Leaf coding-agent runs. |
| L3 | Agentic coding token consumption is mostly input-token driven. | Bai et al., [How Do AI Agents Spend Your Money?](https://arxiv.org/abs/2604.22750), arXiv:2604.22750. | Supports focusing on `input` and `input + output`; in this baseline, input dominates total tokens. |
| L4 | Same-prompt LLM inference can be nondeterministic because of system-level effects such as dynamic batching and floating-point reduction behavior. | Gond et al., [LLM-42: Enabling Determinism in LLM Inference with Verified Speculation](https://arxiv.org/abs/2601.17768), arXiv:2601.17768. | Supports nondeterminism at the model-call layer before agentic feedback loops amplify path differences. |
| L5 | General-purpose agents operating over tools and long interaction histories can have path-dependent trajectories. | Li et al., [Benchmark Test-Time Scaling of General LLM Agents](https://arxiv.org/abs/2602.18998), arXiv:2602.18998. | Supports treating earlier patch/review behavior as context that affects later linearize behavior. |
| L6 | Token efficiency is a separate evaluation axis and should be measured explicitly. | Du et al., [OckBench: Measuring the Efficiency of LLM Reasoning](https://arxiv.org/abs/2511.05722), arXiv:2511.05722. | Supports maintaining a token-usage baseline and not treating token use as incidental logging. |

# Workflow Shape

The Work Leaf bench is a multi-agent coding workflow. It is not a single LLM completion.

The relevant control flow is in `bench-three-features`:

- Lines 673-675 post three `new ...` patch-agent commands.
- Line 704 counts review sessions.
- Line 706 counts patch agents whose transcript contains `agent user-N reported done`.
- Line 709 counts distinct patch-agent IDs in commit messages.
- Lines 716-728 launch `force-linearize` only after the completion gate is satisfied.
- Lines 733-739 accept the linearizer plan.
- Lines 742-770 wait for linearize completion, run final checks, collect token usage, and write the report.

A successful run token total is therefore produced by a trajectory:

```text
T = patch_agent_tokens + review_agent_tokens + linearize_tokens
```

The phases are not independent. Patch and review paths change commits, transcript history, file reads, review content, and finalizer context. The linearizer then consumes that accumulated state.

# Included Baseline Runs

The fitted baseline includes these 31 successful full-workflow runs. `total` is `input + output`.

```text
batch                                      run    sessions  commits  changed  duration_s  total
parallel-baseline-9-20260623T083811+0200  run-1  6         2        616      2970        12479863
parallel-baseline-9-20260623T083811+0200  run-2  7         3        639      1933        15158144
parallel-baseline-9-20260623T083811+0200  run-3  7         3        571      1747        12220762
parallel-baseline-9-20260623T083811+0200  run-4  7         3        728      1703        15966409
parallel-baseline-9-20260623T083811+0200  run-5  7         3        742      2345        15163003
parallel-baseline-9-20260623T083811+0200  run-6  7         3        672      1639        12712101
parallel-baseline-9-20260623T083811+0200  run-7  7         3        792      1845        14098341
parallel-baseline-9-20260623T083811+0200  run-8  6         2        583      1849        11176624
parallel-baseline-9-20260623T083811+0200  run-9  7         3        502      1844        13335734
parallel-extra-6-20260623T105409+0200    run-1  6         2        603      1960        12786553
parallel-extra-6-20260623T105409+0200    run-2  6         2        608      1678        10969104
parallel-extra-6-20260623T105409+0200    run-3  7         3        597      1971        13366390
parallel-extra-6-20260623T105409+0200    run-5  7         3        609      1966        14581482
parallel-extra-6-20260623T105409+0200    run-6  7         3        689      1795        13154495
parallel-baseline-6-20260623T135939+0200 run-1  6         2        445      1902        12074260
parallel-baseline-6-20260623T135939+0200 run-2  7         3        530      1908        13482785
parallel-baseline-6-20260623T135939+0200 run-3  7         3        653      1792        10703647
parallel-baseline-6-20260623T135939+0200 run-4  7         3        584      1701        11571609
parallel-baseline-6-20260623T135939+0200 run-5  6         2        646      1538        10021134
parallel-baseline-6-20260623T135939+0200 run-6  7         3        549      1765        15219437
parallel-followup-6-20260623T150613+0200 run-1  7         3        760      1782        16201988
parallel-followup-6-20260623T150613+0200 run-2  7         3        881      2274        17885915
parallel-followup-6-20260623T150613+0200 run-3  7         3        463      1615        14547785
parallel-followup-6-20260623T150613+0200 run-4  6         2        491      1691        10941356
parallel-followup-6-20260623T150613+0200 run-6  7         3        734      1803        13485156
parallel-followup-6-20260623T161812+0200 run-1  7         3        647      1544        11863342
parallel-followup-6-20260623T161812+0200 run-2  6         2        482      1378         9485647
parallel-followup-6-20260623T161812+0200 run-3  7         3        680      2298        16671875
parallel-followup-6-20260623T161812+0200 run-4  7         3        572      1934        14981788
parallel-followup-6-20260623T161812+0200 run-5  6         2        640      1843        13123039
parallel-followup-6-20260623T161812+0200 run-6  6         2        815      1964        17431669
```

# Removed Outlier

The following run is part of the 32-run passing candidate group but is removed from fitted baseline parameters:

```text
path:             bench-results/parallel-followup-6-20260623T150613+0200/runs/run-5/three-feature-bench.jsonl
result:           pass
sessions:         7
commits:          3
changed_lines:    707
duration_s:       3136
input:            28,940,264
output:              111,590
input + output:   29,051,854
linearize_input:  22,605,239
```

It is removed from parameter fitting for these reasons:

- It is `7.3` standard deviations above the fitted 31-run mean.
- It is `62%` higher than the next-highest included run, `17.89M`.
- Its linearize input alone is `22.61M`, while the largest included linearize input is `11.57M`.
- Including it would make the fitted distribution describe a rare incident rather than the recurring baseline.

The outlier is not discarded as invalid data. It remains evidence that rare large-agent trajectories can occur. It is excluded only from the ordinary baseline parameter fit used to detect recurring regressions.

# Descriptive Statistics

Fitted successful-run totals, excluding the outlier:

```text
n:                 31
mean total:        13,447,143
stddev total:       2,146,906
coefficient var:        16.0%
min total:          9,485,647
median total:      13,335,734
max total:         17,885,915
```

Input and output:

```text
metric            mean       stddev      coefficient var
input        13,364,789   2,142,179      16.0%
output           82,354      10,557      12.8%
reasoning        45,289       7,496      16.6%
```

Phase input statistics:

```text
phase       mean input   stddev input  coefficient var
patch       3,790,618      827,395     21.8%
review      2,036,387      537,783     26.4%
linearize   7,537,784    1,813,581     24.1%
```

Session topology split:

```text
topology                  n   mean total   stddev total  coefficient var
6 sessions / 2 commits   10   12,048,925    2,228,454    18.5%
7 sessions / 3 commits   21   14,112,961    1,798,921    12.8%
```

The latest benches show that session count is not enough by itself. One 6-session/2-commit run with `815` changed lines used `17.43M` tokens, mostly because linearize input reached `11.57M`. The model therefore includes a changed-line work-shape bucket.

# Distribution Derivation

## Modeling Choice

Token totals are positive and right-skewed. A Gamma distribution is a practical moment-matched distribution for positive overdispersed totals. This document does not claim that all LLM-agent costs are Gamma-distributed. The Gamma fit is a local approximation used to derive explicit `alpha` and `theta` values from Work Leaf bench data.

For a Gamma distribution with shape `alpha` and scale `theta`:

```text
mean = alpha * theta
variance = alpha * theta^2
alpha = mean^2 / variance
theta = variance / mean
```

## Pooled Successful-Run Fit

Using all 31 included successful runs:

```text
T_valid ~= Gamma(alpha = 39.231, theta = 342,765)
```

This pooled fit has:

```text
mean:   13.45M
stddev:  2.15M
```

The pooled fit is useful for a quick baseline, but it hides workflow shape. The work-shape model below is preferred.

## Session-Only Mixture Fit

The session-only model is:

```text
T_session ~= 0.323 * Gamma(alpha = 29.234, theta = 412,154)
          + 0.677 * Gamma(alpha = 61.548, theta = 229,301)
```

This captures the mean gap between 6-session and 7-session runs, but it is too crude for high-change 6-session runs.

## Work-Shape Mixture Fit

The recommended distribution for this specific bench is:

```text
T_bench ~= 0.290 * Gamma(alpha = 83.913, theta = 136,462)   # 6 sessions, changed <= 650
        + 0.355 * Gamma(alpha = 104.085, theta = 131,300)  # 7 sessions, changed <= 650
        + 0.355 * Gamma(alpha = 44.675, theta = 332,657)   # changed > 650
```

Where:

- `0.290 = 9 / 31`, compact 6-session/2-commit successful runs.
- `0.355 = 11 / 31`, compact 7-session/3-commit successful runs.
- `0.355 = 11 / 31`, high-change successful runs.
- `changed <= 650` is a coarse empirical compact-work bucket.
- `changed > 650` captures high-change trajectories, including the 6-session/2-commit `17.43M` run with `815` changed lines.

The `changed_lines` bucket is a post-run work-shape covariate. It should be used for diagnosis and stratified regression checks, not as a pre-run predictor. `linearize token share` is also diagnostic rather than predictive because it is part of the measured outcome.

Simulated quantiles from the fitted work-shape mixture:

```text
1%:      9.25M
2.5%:    9.73M
5%:     10.19M
10%:    10.77M
25%:    11.90M
50%:    13.29M
75%:    14.77M
90%:    16.28M
95%:    17.32M
97.5%:  18.28M
99%:    19.38M
```

Operational interpretation:

```text
normal successful run:        about 9.73M to 18.28M  (central 95%)
watch zone:                   below 9.73M or above 18.28M
strong single-run anomaly:    below 9.25M or above 19.38M  (outside central 98%)
separate outlier handling:    around 29M until repeated high-tail samples exist
```

Regression checks should compare post-patch successful runs against this work-shape mixture, not against a single absolute number. A single high run around `17M` can be normal when changed lines are high. A repeated mean shift, or repeated runs beyond the watch zone, is stronger evidence.

# Phase Dependency And Covariance

The model must not derive total variance by linearly adding independent patch, review, and linearize variances. The workflow is path-dependent: patch and review behavior changes the linearizer's context.

Let:

```text
U = patch + review
L = linearize
T = U + L
```

Then:

```text
Var(T) = Var(U) + Var(L) + 2 * Cov(U, L)
```

Observed fitted-baseline covariance diagnostics:

```text
group                      n   corr(U,L)  sd if independent  observed sd
all included runs         31    0.047          2.10M          2.15M
6-session topology        10   -0.049          2.27M          2.23M
7-session topology        21   -0.113          1.90M          1.80M
6-session compact bucket   9   -0.411          1.61M          1.25M
7-session compact bucket  11   -0.497          1.81M          1.34M
high-change bucket        11    0.114          2.12M          2.22M
```

This shows that phase dependency is real and bucket-dependent. Within compact buckets, upstream and linearize token totals are negatively correlated; assuming independence would overestimate standard deviation. In high-change runs, the correlation turns slightly positive and observed variance is larger.

The fitted distribution handles this by fitting whole successful trajectories by work-shape bucket. It does not assume independent phase noise. The phase decomposition is diagnostic; the fitted parameters come from full-run totals.

# Why The Variance Is Expected

The expected-variance argument has two kinds of premises: literature premises and local Work Leaf premises. The literature does not provide Work Leaf's numeric `theta`; it establishes which effects must be expected and measured. The local artifacts provide the fitted parameters.

1. Benchmark results must be treated as random variables, not as single deterministic facts. Madaan et al., [Quantifying Variance in Evaluation Benchmarks](https://arxiv.org/abs/2406.10229), argue that benchmark comparisons need variance estimates rather than isolated scores. This justifies using repeated Work Leaf bench trajectories as the unit of analysis instead of asking whether one `10M`, `17M`, or `29M` run is enough to prove a regression.

2. Token usage is itself a valid efficiency metric, not incidental logging. Du et al., [OckBench](https://arxiv.org/abs/2511.05722), treat token efficiency as a first-class measurement axis for reasoning systems. This justifies modeling `input + output` directly instead of only checking pass/fail behavior.

3. Agentic coding token usage is expected to be highly stochastic. Bai et al., [How Do AI Agents Spend Your Money?](https://arxiv.org/abs/2604.22750), analyze token consumption in agentic coding tasks and support the premise that same-task coding-agent runs can consume substantially different tokens. Work Leaf's bench is in that class because `bench-three-features` starts coding agents, review agents, and a linearizer rather than one fixed completion.

4. The token variance should be mostly input-token driven. Bai et al., [How Do AI Agents Spend Your Money?](https://arxiv.org/abs/2604.22750), support focusing on input-token-heavy agent costs. The local artifacts match this premise: the fitted 31-run baseline has mean `input = 13.36M` and mean `output = 82.4K`, so output is less than `1%` of `input + output`.

5. Base model-call nondeterminism can seed different trajectories. Gond et al., [LLM-42](https://arxiv.org/abs/2601.17768), discuss nondeterminism in LLM inference from system-level effects such as batching and numerical behavior. This does not by itself explain the whole Work Leaf spread, but it supports the first branching point: two runs with the same prompt and code can begin to diverge.

6. Multi-turn agent trajectories can amplify early differences through context and tool history. Li et al., [Benchmark Test-Time Scaling of General LLM Agents](https://arxiv.org/abs/2602.18998), study general LLM agents in long tool-using trajectories, which supports treating path history as part of the system being evaluated. In Work Leaf, patch/review history becomes linearize input; therefore earlier variation can affect later token cost.

7. The local workflow shape changes between valid runs. This is measured from the artifacts. Compact 6-session runs average `11.45M`, compact 7-session runs average `13.67M`, and high-change runs average `14.86M`. The latest six benches demonstrate why this extra work-shape bucket is necessary: one 6-session/2-commit high-change run used `17.43M` tokens.

8. The local downstream linearize phase is the largest token consumer, so upstream path differences matter. This is measured from the artifacts: linearize input averages `7.54M`, patch input averages `3.79M`, and review input averages `2.04M`. The phase dependency is also measured locally through covariance diagnostics. Therefore the model cannot be a linear sum of independent patch/review/linearize variances.

9. The fitted distribution therefore has to be empirical and work-shape-aware. Madaan et al. justify estimating variance from repeated runs; Bai et al. justify expecting stochastic agentic coding token consumption; Li et al. justify path-dependent trajectories; the Work Leaf artifacts determine the parameters. This is why the fitted model uses whole-run totals by work-shape bucket:

```text
T_bench ~= 0.290 * Gamma(83.913, 136,462)
        + 0.355 * Gamma(104.085, 131,300)
        + 0.355 * Gamma(44.675, 332,657)
```

Therefore, assuming the cited literature is accurate, the included fitted-baseline range of `9.49M` to `17.89M` successful-run tokens is expected for this bench. It is not by itself evidence of a regression. The `29.05M` run is a separate high-tail observation because it is not explained by the ordinary fitted baseline: its linearize input alone is `22.61M`, while the largest included linearize input is `11.57M`.

# Expected Distribution For Work Leaf Orchestrator Generally

The general Work Leaf orchestrator distribution should be modeled as a work-shape mixture, not as one universal distribution.

Let `g` be a workflow shape:

```text
g = {
  patch_agent_count,
  review_agent_count,
  finalizer_count,
  final_commit_count,
  tool_use_count,
  retry_count,
  file_read_volume,
  transcript_volume,
  changed_lines_bucket,
  check_command_volume,
  benchmark_version_or_benched_commit
}
```

For each shape `g`, define:

```text
T_g = U_g + L_g(H_g) + O_g
```

Where:

- `U_g` is upstream patch/review token use.
- `H_g` is the accumulated history created by upstream agents.
- `L_g(H_g)` is finalizer/linearizer token use as a function of that history.
- `O_g` is any other agent token use.

Then model:

```text
T_orchestrator ~= sum_g P(g) * Distribution(T_g | g)
```

A Gamma approximation can be used per shape when enough samples exist:

```text
T_g ~= Gamma(alpha_g, theta_g)
alpha_g = mu_g^2 / sigma_g^2
theta_g = sigma_g^2 / mu_g
```

But `theta_g` must be fitted from Work Leaf data. It cannot be imported from the literature. The literature justifies expecting stochasticity and path dependence; the local bench artifacts define the parameters.

# Regression Interpretation

For the current fitted bench baseline:

```text
included runs:      31
baseline mean:      13.45M
baseline stddev:     2.15M
baseline CV:          16.0%
central 95%:        9.73M to 18.28M
```

Approximate post-patch detection sensitivity using this variance scale:

```text
new successful samples   rough detectable upward shift
1                        about +35%
3                        about +20%
6                        about +15%
9                        about +13%
```

This means:

- A single successful run around `17M` can be normal when changed lines are high.
- A single successful run around `18M` is a watch signal.
- A single successful run around `29M` is an outlier relative to the current baseline and should be investigated separately.
- Failed/stalled runs must not be mixed into successful-run token baselines.
- Repeated post-patch successful runs should be compared by work-shape bucket when possible.

# Source Quality Notes

The strongest source for this exact question is Bai et al., because it studies token consumption in agentic coding tasks and reports high same-task stochasticity. Madaan et al. supports the repeated-measurement discipline. Gond et al. supports base inference nondeterminism. Li et al. supports path-dependent multi-turn agent behavior. Du et al. supports treating token efficiency as a first-class metric.

No claim in this document depends on vendor blog posts, forum posts, or anecdotal reports.

# References

- Lovish Madaan, Aaditya K. Singh, Rylan Schaeffer, Andrew Poulton, Sanmi Koyejo, Pontus Stenetorp, Sharan Narang, Dieuwke Hupkes. [Quantifying Variance in Evaluation Benchmarks](https://arxiv.org/abs/2406.10229). arXiv:2406.10229, 2024.
- Longju Bai, Zhemin Huang, Xingyao Wang, Jiao Sun, Rada Mihalcea, Erik Brynjolfsson, Alex Pentland, Jiaxin Pei. [How Do AI Agents Spend Your Money? Analyzing and Predicting Token Consumption in Agentic Coding Tasks](https://arxiv.org/abs/2604.22750). arXiv:2604.22750, 2026.
- Raja Gond, Aditya K. Kamath, Ramachandran Ramjee, Ashish Panwar. [LLM-42: Enabling Determinism in LLM Inference with Verified Speculation](https://arxiv.org/abs/2601.17768). arXiv:2601.17768, 2026.
- Xiaochuan Li, Ryan Ming, Pranav Setlur, Abhijay Paladugu, Andy Tang, Hao Kang, Shuai Shao, Rong Jin, Chenyan Xiong. [Benchmark Test-Time Scaling of General LLM Agents](https://arxiv.org/abs/2602.18998). arXiv:2602.18998, 2026.
- Zheng Du, Hao Kang, Song Han, Tushar Krishna, Ligeng Zhu. [OckBench: Measuring the Efficiency of LLM Reasoning](https://arxiv.org/abs/2511.05722). arXiv:2511.05722, 2025.

# Conclusion

The current Work Leaf three-feature baseline group contains 32 passing candidate reports. One report, the `29.05M`-token follow-up `run-5`, is a clear high-tail outlier driven by an unusually large `22.61M`-input linearize phase. It is retained as an outlier observation but excluded from ordinary baseline parameter fitting.

The fitted 31-run baseline supports the conclusion that substantial token variance is expected for this workflow. The preferred model is a work-shape-aware mixture over compact 6-session runs, compact 7-session runs, and high-change runs:

```text
T_bench ~= 0.290 * Gamma(83.913, 136,462)
        + 0.355 * Gamma(104.085, 131,300)
        + 0.355 * Gamma(44.675, 332,657)
```

This model gives a central 95% expected range of approximately `9.73M` to `18.28M` `input + output` tokens for successful full-workflow runs. It includes the latest six successful benches and accounts for the observed fact that high changed-line runs can be expensive even when they finish with only 6 sessions and 2 commits.

The operational conclusion is to treat successful runs inside the fitted interval as ordinary baseline variation, to treat repeated mean shifts as regression evidence, and to investigate rare extreme runs such as `29M` separately rather than allowing them to define the normal baseline.
