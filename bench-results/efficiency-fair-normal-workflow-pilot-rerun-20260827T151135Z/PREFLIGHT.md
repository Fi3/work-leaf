# Preflight Evidence

No three-feature benchmark workflow was launched while preparing this evidence. One bounded,
read-only real Codex call verified the run-local wrapper before the paid pair.

## Direct Resume Regression

The observer regression reproduces one direct Codex conversation split across an initial
`codex exec` invocation and a later `codex exec resume` invocation. The first reports 100 input
tokens and the second reports 250. The accepted total is 350, not the larger individual value of
250.

The rollout fixture contains two task epochs whose counters restart, matching the structure observed
in the first paid pilot. The provider capture and rollout sum must agree before the observer marks a
measurement complete.

## First-Pilot Offline Reanalysis

The repaired observer was run against all raw direct evidence from
`../efficiency-fair-normal-workflow-pilot-20260827T115642Z`:

- 42 captured Codex CLI invocations;
- 15 provider conversations;
- 15 of 15 conversations matched to saved Codex rollout files;
- zero rollout or accounting errors;
- corrected raw total: 35,947,089 tokens; and
- corrected uncached total: 1,353,041 tokens.

The machine-readable outputs are `preflight/prior-direct-reanalysis.json` and
`preflight/prior-direct-rollout-audit.json`.

The original temporary observer executable no longer existed. For this offline check only, the copied
observer configuration points to the locally built repaired observer and its SHA-256. The original
profile wrapper was reconstructed at its recorded path with its exact recorded SHA-256
`d5cbb8ee9971e0a77415b5ff5902f9a321da113f49eb7628c962c85f2ef78e3b`. Raw provider streams,
process records, rollout files, candidate code, task text, and quality results were not changed.

## Provider Isolation

Automated tests verify that both product benchmark launchers install and restore the same temporary
provider-isolation policy. A separate wrapper test verifies that an allowed top-level Codex process
receives GPT-5.5/xhigh while an inherited child invocation exits with status 86, does not reach the
real Codex executable, and leaves an attempt record.

The original task hash test remains green, so the isolation policy does not alter any feature request.

## Real Provider Smoke

The generated wrapper launched the real configured Codex CLI with GPT-5.5 and xhigh reasoning in a
read-only checkout. The request asked for the exact reply `WORK_LEAF_REAL_AGENT_OK`; that exact reply
was returned, and the recursive-attempt log remained empty. The matching Codex rollout records
`gpt-5.5` and `xhigh` for the task.

This smoke validated provider launch and model selection only. It did not ask Codex to start a
write-capable tool, so it could not expose the nested workspace-write sandbox failure later observed
in the paid direct workflow. The replacement gate requires a bounded workspace-write smoke.

The captured CLI stream is `preflight/real-agent-smoke.jsonl`. Its hashes, Codex version, rollout
identity, model profile, and result are recorded in `preflight/real-agent-smoke.json`.
