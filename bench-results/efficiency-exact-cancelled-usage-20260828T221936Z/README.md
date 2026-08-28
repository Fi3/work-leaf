# Exact Usage For Interrupted Work Leaf Turns

## Abstract

The normal Work Leaf benchmark interrupts Codex after a complete orchestrator directive. The
ChatGPT Codex transport does not report usage for that interrupted response, so the saved Work Leaf
totals are lower bounds rather than exact totals. This study provides a separate API-key transport
that stores each GPT-5.5 response, forwards its live event stream unchanged, and retrieves exact
provider usage after either normal completion or a Work Leaf interruption.

The mechanism works in automated tests and in a real Work Leaf interruption. It does not recover
the old ChatGPT-backed runs. Any new direct-versus-Work-Leaf result collected with it is a distinct
OpenAI API study and must not be merged with the earlier ChatGPT Codex result.

## What Runs

`infrastructure/run_with_exact_usage.py` starts a localhost proxy and places a study-only `codex`
wrapper first on `PATH`. The wrapper selects a custom Responses API provider and pins `gpt-5.5`
with `xhigh` reasoning. The existing benchmark wrappers can add the same pins again, but cannot
select a different model or provider.

`infrastructure/exact_usage_proxy.py` handles the provider requests:

1. It changes only `store` and `background` to `true` on streamed response requests.
2. It forwards the response event stream to Codex without storing prompt or response content.
3. A normal `response.completed` event supplies exact usage directly.
4. When Codex disconnects after Work Leaf interrupts a turn, it closes the upstream stream first,
   requests provider cancellation, and polls the stored response until exact usage appears.
5. It writes one durable final record per started response. The summarizer rejects missing,
   duplicate, malformed, or incomplete response records.

The proxy also forwards Codex's read-only model-list request. It does not store credentials, prompt
text, response text, or model-list contents.

## Fairness Boundary

The next comparison remains fair only under all of these conditions:

- direct Codex and concurrent Work Leaf both run through this same proxy;
- both use the existing frozen three-feature task, base commit, scorer, GPT-5.5 model, and xhigh
  reasoning level;
- direct Codex uses `bench-three-features-sequential` unchanged;
- Work Leaf uses `bench-three-features` unchanged and concurrently;
- normal validation, review, linearization, time limits, and final checks remain unchanged;
- every complete, partial, failed, and interrupted workflow remains in the result set; and
- token totals come from this study's exact provider records, not the incomplete Work Leaf total in
  the existing observer report.

Work Leaf source, the benchmark task, and the quality evaluator are not modified here. The proxy
changes the provider route from ChatGPT Codex to the OpenAI Responses API and enables stored
background responses. Results therefore describe the API route only. They can test whether the
large relative saving exists under complete accounting, but cannot retroactively establish the
exact saving of the old ChatGPT-backed runs.

## Verification

Run the automated checks with:

```sh
python3 -m unittest -v \
  infrastructure/test_exact_usage_proxy.py \
  infrastructure/test_run_with_exact_usage.py
```

The real-agent evidence and exact outcomes are summarized in `RESULT.md`. No full three-feature
benchmark was launched in this infrastructure-validation study.
