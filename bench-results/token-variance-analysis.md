# Abstract

This document analyzes the Work Leaf Codex three-feature benchmark baseline and evaluates whether the observed token variance is expected for this specific multi-agent workflow. The candidate baseline group is the 30-report set listed by `bench-results/baseline-manifest.json`: the `parallel-current-12`, `parallel-current-6a`, `parallel-current-6b-retry`, and `parallel-current-6c` batches from June 24, 2026.

Only completed successful reports are used for token-distribution fitting. The fitted set contains 24 passing reports and excludes 6 failed reports. No successful report is removed as a token outlier in this baseline. Every fitted report contains the required reviewed patch work from `review-user-1`, `review-user-2`, and `review-user-3`, and every fitted report reached the final pass path with 3 commits after the base commit and successful final checks.

The primary regression model is the pooled successful-run Gamma fit:

```text
T_valid ~= Gamma(alpha = 39.171, theta = 328,808)
```

where `T_valid = input + output` tokens for a successful full benchmark run. The fitted mean is `12.88M`, the sample standard deviation is `2.06M`, the coefficient of variation is `16.0%`, and the fitted central 95% interval is approximately `9.16M` to `17.23M` tokens.

A changed-lines split is useful for post-run diagnosis:

```text
T_shape ~= 0.542 * Gamma(alpha = 33.583, theta = 358,247)   # changed <= 1500
        + 0.458 * Gamma(alpha = 76.289, theta = 181,971)   # changed > 1500
```

The changed-lines model is diagnostic rather than the primary gate because the fitted set has only 24 successful observations. The pooled model is the regression baseline; the split explains work-shape differences after a run completes.

# Scope And Data

This analysis uses saved bench result artifacts only.

Candidate result roots:

- `bench-results/parallel-current-12-20260624T010102+0200`
- `bench-results/parallel-current-6a-20260624T020421+0200`
- `bench-results/parallel-current-6b-retry-20260624T053754+0200`
- `bench-results/parallel-current-6c-20260624T104153+0200`

Candidate baseline group:

```text
candidate reports:             30
passing reports fitted:        24
failed reports excluded:        6
passing reports missing user-3: 0
successful token outliers:      0
```

The report schema is the JSON written by `bench-three-features` into each `three-feature-bench.jsonl` file. The candidate reports share `base_commit = c92a0b7060a36eac6db2d869b85e589a7a9480f9` in the saved reports. The bench binary reports `benched_binary_commit = d4cb33d9cae99387831c690ca3b5201450558634`; the saved reports also mark `benched_binary_dirty = yes`, so the reports themselves remain the authority for the exact executed artifacts.

Valid successful result definition:

- `result == pass`
- `token_usage` is present and parseable
- total `input > 0`
- total `output > 0`
- `review-user-1`, `review-user-2`, and `review-user-3` are present in token usage
- `commits_after_base == 3`
- `code_quality` records successful `cargo fmt -- --check`, `cargo clippy --all-targets --all-features -- -D warnings`, and `cargo test --all-targets --all-features`

Primary token metric:

```text
total = input + output
```

`reasoning_output` is reported separately and is not added a second time. Phase input totals use named token-usage sessions: patch input is from `user-N`, review input is from `review*` sessions, and linearize input is from `linearize`.

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

- Lines 690-692 post three `new ...` patch-agent commands.
- Lines 717-723 count review sessions, ready/done patch agents, terminal patch agents, total patch agents, and patch agents with commits.
- Lines 740-748 launch `force-linearize` only after 3 patch agents exist, terminal patch-agent state is reached, and all 3 patch agents have produced reviewed commits.
- Lines 753-759 wait for the linearizer and accept the linearization plan.
- Lines 762-790 require a clean final worktree, exactly 3 commits after the base commit, successful final checks, token-usage collection, and report writing.

A successful run token total is therefore produced by a trajectory:

```text
T = patch_agent_tokens + review_agent_tokens + linearize_tokens
```

The phases are not independent. Patch and review paths change commits, transcript history, file reads, review content, and finalizer context. The linearizer then consumes that accumulated state.

The `token_sessions` column in this document counts named token-usage entries, not required patch features. A normal successful run has 7 named token sessions: `user-1`, `user-2`, `user-3`, `review-user-1`, `review-user-2`, `review-user-3`, and `linearize`. One fitted successful report has 8 token sessions because it also includes `reviewer-1`; it still has the required three reviewed patch agents and 3 final commits, so it remains a valid successful run.

# Included Baseline Runs

The fitted baseline includes these 24 successful full-workflow runs. `total` is `input + output`.

```text
batch                                      run     token_sessions  commits  changed  duration_s  input     output  total     linearize_input
parallel-current-12-20260624T010102+0200  run-1   7               3        1425     3744        12012962   98433  12111395  3124507
parallel-current-12-20260624T010102+0200  run-2   7               3        1615     2065        12562920   94908  12657828  4760315
parallel-current-12-20260624T010102+0200  run-3   7               3        1357     2012        14658399  111641  14770040  5659064
parallel-current-12-20260624T010102+0200  run-5   7               3        1672     2039        14363412   94188  14457600  7786158
parallel-current-12-20260624T010102+0200  run-6   7               3        1401     2181        14534716   92935  14627651  6191600
parallel-current-12-20260624T010102+0200  run-7   7               3        1596     2292        12951230   80804  13032034  5809189
parallel-current-12-20260624T010102+0200  run-8   7               3        1477     1658         9318203   85969   9404172  4794514
parallel-current-12-20260624T010102+0200  run-9   7               3        1317     1576         9411265   66593   9477858  2744682
parallel-current-12-20260624T010102+0200  run-10  7               3        1573     1846        14378140   85218  14463358  6699657
parallel-current-12-20260624T010102+0200  run-11  7               3        1408     1790        12328965   93402  12422367  5453992
parallel-current-12-20260624T010102+0200  run-12  7               3        1222     1419        10492951   71201  10564152  4929642
parallel-current-6a-20260624T020421+0200  run-2   7               3        1462     3049        12968639  102158  13070797  3087499
parallel-current-6a-20260624T020421+0200  run-4   7               3        1646     2156        13150052   65731  13215783  8761554
parallel-current-6a-20260624T020421+0200  run-5   7               3        1375     2051        10463877   93551  10557428  5113128
parallel-current-6a-20260624T020421+0200  run-6   7               3        1552     2508        12623412   75811  12699223  4430434
parallel-current-6b-retry-20260624T053754+0200 run-1 7           3        1603     2122        15110638   96739  15207377  7556616
parallel-current-6b-retry-20260624T053754+0200 run-2 7           3        1347     1695         9873794   84079   9957873  4748670
parallel-current-6b-retry-20260624T053754+0200 run-5 7           3        1497     2075        14383955   92011  14475966  6307681
parallel-current-6b-retry-20260624T053754+0200 run-6 8           3        1365     2607        10416391   89838  10506229  3452774
parallel-current-6c-20260624T104153+0200  run-1   7               3        1532     2447        13748789   85812  13834601  7821595
parallel-current-6c-20260624T104153+0200  run-2   7               3        1330     2473        14370800   87779  14458579  5835175
parallel-current-6c-20260624T104153+0200  run-4   7               3        1533     2798        12112437   81066  12193503  5422644
parallel-current-6c-20260624T104153+0200  run-5   7               3        1518     2634        13039611  102405  13142016  3678071
parallel-current-6c-20260624T104153+0200  run-6   7               3        1617     3308        17680608  122009  17802617  4776943
```

# Excluded Failed Reports

The following reports are excluded from successful-run parameter fitting because they did not pass. They are not ignored as operational failures; they are outside the fitted token distribution because they are not completed successful trajectories.

```text
batch                                      run    changed  duration_s  total     commits  token_sessions  reason
parallel-current-12-20260624T010102+0200  run-4  1637     1987         9382573  3        7               final repository checks failed
parallel-current-6a-20260624T020421+0200  run-1  1457     3366         9096461  7        6               idle stalled after done_users=2 ready_users=2
parallel-current-6a-20260624T020421+0200  run-3  1743     2134        15259766  3        7               final repository checks failed
parallel-current-6b-retry-20260624T053754+0200 run-3 1330 1997       10775822  3        7               final repository checks failed
parallel-current-6b-retry-20260624T053754+0200 run-4 1325 1952       11421588  3        7               final repository checks failed
parallel-current-6c-20260624T104153+0200  run-3  1366     2343        10744811  3        7               final repository checks failed
```

Failed reports can still contain token usage, but using them to fit successful-run token totals would mix different stopping conditions. Failure rate and failure causes require a separate reliability analysis.

# Descriptive Statistics

Fitted successful-run totals:

```text
n:                 24
mean total:        12,879,602
stddev total:       2,057,890
coefficient var:        16.0%
min total:          9,404,172
median total:      13,051,416
max total:         17,802,617
```

Token fields:

```text
metric       n   min        median      mean        stddev     coefficient var  max
input       24   9,318,203  12,959,934  12,789,840  2,050,392  16.0%            17,680,608
output      24      65,731      90,924      89,762     13,090  14.6%               122,009
reasoning   24      28,034      52,038      53,144     10,881  20.5%                80,257
```

Workflow fields:

```text
metric             n   min        median     mean       stddev     coefficient var  max
changed lines     24       1222       1487       1477        121    8.2%               1672
duration seconds  24       1419       2139       2273        557   24.5%               3744
patch input       24  2,362,847  4,257,571  4,360,340  1,503,642  34.5%          9,334,863
review input      24  1,604,853  3,068,376  3,056,746    778,824  25.5%          4,689,532
linearize input   24  2,744,682  5,267,886  5,372,754  1,591,047  29.6%          8,761,554
```

Token usage is input-dominated. Mean output is `89,762`, which is less than `1%` of mean `input + output`.

Session count is not a useful primary split in this baseline. Twenty-three fitted reports have 7 token sessions and one fitted report has 8 token sessions. No successful 6-token-session report exists in this baseline, so a session-only mixture would either be degenerate or overfit a one-sample component.

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

Using all 24 successful runs:

```text
T_valid ~= Gamma(alpha = 39.171, theta = 328,808)
```

This pooled fit has:

```text
mean:   12.88M
stddev:  2.06M
```

Moment-matched Gamma quantiles:

```text
1%:      8.58M
2.5%:    9.16M
5%:      9.69M
10%:    10.32M
25%:    11.44M
50%:    12.77M
75%:    14.21M
90%:    15.59M
95%:    16.45M
97.5%:  17.23M
99%:    18.16M
```

Operational interpretation:

```text
normal successful run:        about 9.16M to 17.23M  (central 95%)
watch zone:                   below 9.16M or above 17.23M
strong single-run anomaly:    below 8.58M or above 18.16M  (outside central 98%)
```

## Changed-Line Diagnostic Fit

Changed lines are a post-run work-shape covariate, not a pre-run predictor. They remain useful for diagnosis because successful runs with more changed lines have a higher observed token mean.

```text
bucket            n   weight  mean total  stddev total  coefficient var  alpha   theta    observed min  observed max
changed <= 1500  13   0.542   12,031,116  2,076,081     17.3%            33.583  358,247   9,404,172    14,770,040
changed > 1500   11   0.458   13,882,358  1,589,397     11.4%            76.289  181,971  12,193,503    17,802,617
```

The diagnostic mixture is:

```text
T_shape ~= 0.542 * Gamma(alpha = 33.583, theta = 358,247)   # changed <= 1500
        + 0.458 * Gamma(alpha = 76.289, theta = 181,971)   # changed > 1500
```

Simulated quantiles from the fitted diagnostic mixture:

```text
1%:      8.11M
2.5%:    8.78M
5%:      9.38M
10%:    10.13M
25%:    11.47M
50%:    12.93M
75%:    14.30M
90%:    15.52M
95%:    16.25M
97.5%:  16.91M
99%:    17.67M
```

The diagnostic mixture has a narrower high tail than the pooled model because each bucket is fitted from fewer observations. For regression gating, the pooled successful-run model is the safer primary baseline. The changed-line mixture explains shape after the run and helps classify whether a high-token pass came from a larger final work product.

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
group                n   corr(U,L)  sd if independent  observed sd
all included runs   24   -0.321          2.48M          2.06M
changed <= 1500     13   -0.041          2.11M          2.08M
changed > 1500      11   -0.689          2.74M          1.59M
7 token sessions    23   -0.346          2.50M          2.04M
```

This shows that phase dependency is real and bucket-dependent. In this fitted baseline, upstream and linearize token totals are negatively correlated, especially in the `changed > 1500` bucket. Assuming independent phase noise would overestimate standard deviation for this sample.

The fitted distribution handles this by fitting whole successful trajectories. It does not assume independent phase noise. The phase decomposition is diagnostic; the fitted parameters come from full-run totals.

# Why The Variance Is Expected

The expected-variance argument has two kinds of premises: literature premises and local Work Leaf premises. The literature does not provide Work Leaf's numeric `theta`; it establishes which effects must be expected and measured. The local artifacts provide the fitted parameters.

1. Benchmark results must be treated as random variables, not as single deterministic facts. Madaan et al., [Quantifying Variance in Evaluation Benchmarks](https://arxiv.org/abs/2406.10229), argue that benchmark comparisons need variance estimates rather than isolated scores. This justifies using repeated Work Leaf bench trajectories as the unit of analysis instead of deciding from one `9M`, `13M`, or `18M` run.

2. Token usage is itself a valid efficiency metric, not incidental logging. Du et al., [OckBench](https://arxiv.org/abs/2511.05722), treat token efficiency as a first-class measurement axis for reasoning systems. This justifies modeling `input + output` directly instead of only checking pass/fail behavior.

3. Agentic coding token usage is expected to be highly stochastic. Bai et al., [How Do AI Agents Spend Your Money?](https://arxiv.org/abs/2604.22750), analyze token consumption in agentic coding tasks and support the premise that same-task coding-agent runs can consume substantially different tokens. Work Leaf's bench is in that class because `bench-three-features` starts coding agents, review agents, and a linearizer rather than one fixed completion.

4. The token variance should be mostly input-token driven. Bai et al., [How Do AI Agents Spend Your Money?](https://arxiv.org/abs/2604.22750), support focusing on input-token-heavy agent costs. The local artifacts match this premise: the fitted 24-run baseline has mean `input = 12.79M` and mean `output = 89.8K`, so output is less than `1%` of `input + output`.

5. Base model-call nondeterminism can seed different trajectories. Gond et al., [LLM-42](https://arxiv.org/abs/2601.17768), discuss nondeterminism in LLM inference from system-level effects such as batching and numerical behavior. This does not by itself explain the whole Work Leaf spread, but it supports the first branching point: two runs with the same prompt and code can begin to diverge.

6. Multi-turn agent trajectories can amplify early differences through context and tool history. Li et al., [Benchmark Test-Time Scaling of General LLM Agents](https://arxiv.org/abs/2602.18998), study general LLM agents in long tool-using trajectories, which supports treating path history as part of the system being evaluated. In Work Leaf, patch/review history becomes linearize input; therefore earlier variation can affect later token cost.

7. The local workflow has measured work-shape variation even when all successful reports have the required three reviewed patch agents. This is measured from the artifacts. The lower changed-line bucket averages `12.03M`; the higher changed-line bucket averages `13.88M`. This supports a post-run changed-line diagnostic split without replacing the primary pooled fit.

8. The local downstream linearize phase is a large token consumer, so upstream path differences matter. This is measured from the artifacts: linearize input averages `5.37M`, patch input averages `4.36M`, and review input averages `3.06M`. The phase dependency is measured locally through covariance diagnostics. Therefore the model cannot be a linear sum of independent patch/review/linearize variances.

9. The fitted distribution therefore has to be empirical and workflow-specific. Madaan et al. justify estimating variance from repeated runs; Bai et al. justify expecting stochastic agentic coding token consumption; Li et al. justify path-dependent trajectories; the Work Leaf artifacts determine the parameters. This is why the primary fitted model uses whole-run totals:

```text
T_valid ~= Gamma(alpha = 39.171, theta = 328,808)
```

Therefore, assuming the cited literature is accurate, the fitted-baseline observed range of `9.40M` to `17.80M` successful-run tokens is expected for this bench. It is not by itself evidence of a regression. Evidence of regression requires a repeated upward shift in successful-run means, repeated high-tail samples, or a reliability regression in pass/fail behavior.

# Expected Distribution For Work Leaf Orchestrator Generally

The general Work Leaf orchestrator distribution should be modeled as a workflow-shape mixture, not as one universal distribution.

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

For the current three-feature bench, 24 successful samples are enough for a pooled successful-run baseline and a diagnostic changed-line split. They are not enough for a stable high-dimensional orchestrator model over all fields in `g`.

# Regression Interpretation

For the current fitted bench baseline:

```text
included successful runs: 24
baseline mean:           12.88M
baseline stddev:          2.06M
baseline CV:             16.0%
central 95%:             9.16M to 17.23M
```

Approximate post-patch detection sensitivity using this variance scale:

```text
future successful runs   detectable upward shift   upper mean threshold
1                        4.41M / 34.2%             17.29M
3                        2.65M / 20.5%             15.53M
6                        1.97M / 15.3%             14.85M
9                        1.69M / 13.1%             14.57M
12                       1.53M / 11.9%             14.41M
24                       1.25M /  9.7%             14.13M
```

This means:

- A single successful run around `17M` can be normal.
- A single successful run above `17.23M` is in the watch zone.
- A single successful run above `18.16M` is a strong single-run anomaly under the pooled fit.
- Failed/stalled runs must not be mixed into successful-run token baselines.
- Failed/stalled runs still indicate benchmark or orchestrator reliability issues and should be investigated independently.
- Repeated post-patch successful runs should be compared by mean shift, and by work-shape bucket when enough samples exist.

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

The current Work Leaf three-feature baseline group contains 30 candidate reports and 24 successful reports used for token-distribution fitting. The 6 failed reports are excluded from successful-run parameter fitting because they are not completed successful trajectories. No successful report is excluded as a token outlier.

The fitted successful-run baseline supports the conclusion that substantial token variance is expected for this workflow. The primary model is:

```text
T_valid ~= Gamma(alpha = 39.171, theta = 328,808)
```

This model gives a central 95% expected range of approximately `9.16M` to `17.23M` `input + output` tokens for successful full-workflow runs. The observed successful-run range, `9.40M` to `17.80M`, is consistent with that fitted distribution, with the maximum run sitting in the watch zone but still below the 99% pooled Gamma quantile of `18.16M`.

The operational conclusion is to treat successful runs inside the fitted interval as ordinary baseline variation, to treat repeated mean shifts as regression evidence, and to investigate failed/stalled runs separately as reliability failures rather than mixing them into the successful-run token baseline.
