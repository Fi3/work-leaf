# Prospective automatic-refresh runner

`runner_automatic_refresh.py` is a private v7 adapter over the exact unchanged
`runner_work_units.py` supervisor. It accepts exactly three
`automatic-changed-refresh-full` workflows, one block and one wave. Its plan declares
`mixed_waves: false`, the sole condition count of three, and only randomized launch order;
saved baselines are reused, not newly allocated. Controls, replacements, extra fields,
changed order, changed phase kind and expanded or reduced populations are rejected.

The adapter compiles the verified engine bytes in a private module with the engine's real
`__file__`. It retains the existing `prepare`, subscription environment, manifest verification,
supervisor timing, launch, outcome and trust paths. Only its private condition/schema/plan and
dependency verification hooks differ. Both adapter and engine must be frozen in the actual
evidence tree; the manifest's runner identity remains the actual engine, not a relabeled wrapper.
Inherited API authentication and Work Leaf overrides are stripped by the unchanged engine,
while the existing subscription home is retained. No runtime or public API is modified.

Test-first evidence: root recorded the missing-module RED `9cd339`; the implementer independently
reproduced all three missing-module errors with `python3 -B -m unittest
test_runner_automatic_refresh -v` (`a9752e`, exit 1) before creating the adapter. The same tests
then passed (`987b13`, exit 0; three tests). The final combined new and legacy adapter check was:

```text
python3 -B -m unittest test_runner_automatic_refresh test_runner_candidates -v
```

Receipt `99fc8a`: six passed, zero failures, 0.075 seconds. All plan construction was in memory;
no CLI plan/prepare/run, actual schedule, allocation shuffle, admission, provider, extraction,
accounting or benchmark execution occurred. The retained root-authored test file and both legacy
runner sources have identical before/after hashes.

| Artifact | Bytes | SHA-256 |
| --- | ---: | --- |
| `runner_automatic_refresh.py` | 6031 | `eeba247844dfa073754b01060d50de7898728d422aab27ff69af42d97b8b15fe` |
| `test_runner_automatic_refresh.py` | 3580 | `dce066f671109504cab32cb7359e9f8d2e804934b28bf00ba530cedae96bad69` |
| Existing `runner_candidates.py` | — | `ecac7753fb535c59974deedf6a04f0ce11b6e05a19f8c4112416ab7e29b7cb03` |
| Frozen engine `runner_work_units.py` | — | `2f019c297a2e436c54241ad0643d4f105db6ec8bc782b1cac456f3acde0bc98e` |

No new quadratic path is introduced: plan cardinality is fixed at three, indexed frozen-file
membership and engine source hashing are retained. The pre-existing supervisor's O(W × E)
frozen-input checks remain, with W fixed at one wave. Independent source review is still required.
This provider-free runner qualification does not establish real-agent C08 delivery or admit the
three-workflow screen; those remain separate root-owned source, protocol and authority gates.
