# Fair Normal-Workflow Pilot Rerun

This directory contains the second one-pair gate for the Work Leaf efficiency study. It compares:

- one normal concurrent Work Leaf workflow, with all three requests submitted together; and
- one normal direct sequential Codex workflow, with the same requests handled one after another.

The pilot uses the fixed source base, GPT-5.5 with xhigh reasoning, the original three task strings,
normal validation behavior, and the same offline quality scorer for both saved implementations.

Recursive provider verification is disabled for both workflows. The benchmarked agents still run
the tests and validation they consider necessary, but they cannot launch another Codex process from
inside an active Codex turn. This removes unrelated provider smoke sessions from the measurement.

`run-pilot` launches exactly one workflow of each kind, at most two workflows concurrently, and does
not retry either workflow after it reaches the provider. It stops after writing
`PROVISIONAL-RESULT.md` and `result.json`. Larger collection and mechanism allocation remain stopped
until the user reviews this result.

The exact comparison rules are in `FAIRNESS-CONTRACT.md`. The repaired observer's offline check
against the first pilot is recorded under `preflight/`.
