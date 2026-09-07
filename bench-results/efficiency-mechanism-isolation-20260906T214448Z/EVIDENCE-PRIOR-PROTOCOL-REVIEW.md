# Independent prior-protocol evidence review

Verdict: no finding in the reviewed source, retained outcomes or sampled interpretation.
The reviewed [protocol/interruption note](EVIDENCE-PRIOR-PROTOCOL-AND-INTERRUPTION.md)
has SHA-256 `54bd7755382d0976a2dd4306c50fbb49d001f4259dcfb6860875de47c3d1dbe1`.
This is a provider-free historical review, not a rerun of accounting, a new comparison,
an acceptance of individual causal shares or permission for further controls.

## Retained population

The [continued score manifest](../efficiency-causal-validation-20260829T210343Z/continued-response-score-manifest.json)
and [main score manifest](../efficiency-mechanism-attribution-20260830T081131Z/score-manifest.json)
contain exactly the following nine identities. Every launcher exit is zero and every
saved workflow report says `pass`. The independently saved scorer details agree with
the evidence rows; no quality-selected exclusion is present.

| Retained run | Saved feature outcomes | Additional retained qualification |
| --- | --- | --- |
| continued-response-001 | visual/status pass; completion fail | two resumed-output/usage decisions |
| continued-response-002 | all three pass | two resumed-output/usage decisions; one timeout |
| continued-response-003 | status pass; visual/completion fail | four resumed-output/usage decisions |
| compact-direct-001 | all three pass | three-target, seven-commit plan |
| compact-direct-002 | all three pass | three-target, four-commit plan |
| compact-direct-003 | all three pass | three-target, seven-commit plan |
| sequential-work-leaf-combined-001 | visual/status pass; completion fail | sequential/direct-read/continued-response package |
| sequential-work-leaf-combined-002 | all three pass | same package |
| sequential-work-leaf-combined-003 | all three pass | same package |

Authorities: [continued evidence](../efficiency-causal-validation-20260829T210343Z/continued-response-evidence.json),
[continued quality](../efficiency-causal-validation-20260829T210343Z/continued-response-quality.json),
[main evidence](../efficiency-mechanism-attribution-20260830T081131Z/evidence.json) and
[main quality](../efficiency-mechanism-attribution-20260830T081131Z/quality.json).
All nine saved analysis hashes, all six main report hashes, and all 69 referenced native
source hashes match. Native session metadata identifies CLI 0.150.1 throughout these
nine runs. No modern exact response-ID ledger is inferred from that older format.

## Response completion is not natural turn completion

- At observer commit `8eb26a4bf77b7fcbffbb60682f81069cd28880a9`,
  `bench-observer/src/lib.rs::ProviderUsageGrace::observe_server_value` (721) marks
  post-directive usage; `wait_before_interrupt` (747) distinguishes usage, resumed
  output, turn completion and timeout. The `exact_usage_seen` flag alone does not
  validate an advancing counter. `forward_app_server_client_frame` (1084) still writes
  the original interrupt after its bounded wait; the serial client pump preserves
  request order while the separate stdout pump (1120) continues forwarding output.
- At normal-WL commit `5b1d1ef9590850faed26052f909ddff7ff8f127d`,
  `src/codex.rs:276–290` requests interruption, unregisters the turn and returns on a
  detector-accepted complete public message. The note correctly distinguishes later
  provider-history text from another interpreted WL directive.
- The normal-H sample S93→S95 in `exact-normal-001`, resolved through
  [H evidence](../efficiency-exact-normal-work-leaf-20260829T181318Z/evidence.json)
  `accounting.capture_bound_audits[0]`, matches its pinned raw SHA and exact thread/turn.
  G1 records 40 ms within the 1000-ms cap. Its nonzero four-field `last` usage is
  contained in the advancing cumulative delta. The statement that these already
  measured tokens cannot subsequently be erased by the interrupt is supported.
- All eight continuation-table rows were independently checked, not merely sampled:
  exact G thread/turn, C interrupt/RPC, S complete directive→extra public item→usage,
  and N assistant item ID/explicit passthrough turn/full-text equality agree. Each
  associated usage payload has nonzero `last` contained componentwise in the advancing
  input/cached-input/output/reasoning delta. These are public continuation episodes;
  the check does not price each extra item or infer a distinct response ID for it.
- The long-wait example `continued-response-001` G22/C56/S39816/S39897/S39898/N14
  really waits 79891 ms and matches native item
  `msg_03a045ed7e7e6714016a936969eb8487d29326640c10690202`.
  The eight exact native paths/IDs remain in the reviewed note's table and evidence
  inventory. All three original client streams equal their forwarded streams.
- The ninth activation remains a timeout: run 002 G36, read S58405, C87 string RPC
  `86`, thread `01a04fda-9234-7f53-8347-77150b91774f`, turn
  `01a04fda-928c-7b03-8a3f-18975aeefe4f`. No usage event exists for that turn;
  the original interrupt is retained. Later workflow reconciliation is not evidence
  that this 120000-ms activation completed.

Direct raw-frame enumeration independently confirms the following distinction:

| Sequential-WL suffix | Actual interrupt requests | Actual interrupted terminal notifications | Saved accounting field | Ordinary / resumed usage-release decisions |
| --- | ---: | ---: | ---: | --- |
| 001 | 38 | 33 | 0 | 31 / 7 |
| 002 | 27 | 23 | 0 | 15 / 12 |
| 003 | 40 | 35 | 0 | 14 / 26 |

Source: each main evidence `groups.S.run_rows[*].analysis` parent directory's sole
`app-server` capture, original client/server and `provider-usage-grace.jsonl` streams.
`analyze_app_server` at the pinned observer's lines 5341–5358 counts interrupted turns
without accepted usage under its historical rule. Thus its zero field cannot support
“no interrupts” or “natural turn completion.” The reviewed note preserves this limit.

## Package activation and interpretation

The hash-pinned [driver generator](../efficiency-mechanism-attribution-20260830T081131Z/generate_diagnostic_driver.py)
requires exact predecessor bytes. Its sequential start/advance sections (28/54) and
compact-plan section (93), together with the [launcher](../efficiency-mechanism-attribution-20260830T081131Z/run-attribution-control)
lines 206–213, substantiate the coupled schedule/direct-read/120000-ms wait policy.
Normal structured writes, lock-mediated commands and review are not removed.

All three actual sequential logs match their saved hashes. Feature-start physical lines
are 9/73/108, 9/62/105 and 9/84/115; preceding later-start state lines are respectively
72/107, 61/104 and 83/114 and report idle prior agents/review with `NeedsDecision`.
All three captures have eight first-input threads, seven containing the exact historical
direct-read policy sentence from `src/agent.rs:245`; this is actual activation, not a label.
All three complete compact-plan prompts match the unique native user input at N7,
including the quoted target counts and exact item IDs in the reviewed note.

The saved reports provide the quoted group directions; no token total, interval,
permutation test or share was recomputed here. The note appropriately rejects importing
the older “proven contribution”/telescoping-share language as individual identification.
The main comparison preserves a connected orchestration package and residual permission,
auxiliary-context and policy differences. It does not separately identify C10–C24,
prove an uncached saving, establish equal-quality populations or authorize replacements.
The timing contrast includes its longer timeout policy and downstream effects, not
just the bytes of eight extra public messages.

## Source pins and review boundary

| Reviewed authority | SHA-256 |
| --- | --- |
| Continued evidence | `9549ccb59dab003b4d091d997462264168a792886abb3e5cac9dd79664ff20f8` |
| Continued quality | `3ac496646b6c09a2d3052573d6913eb71d2c5eca52131b5ff8b2b6ca4d52a6ab` |
| Continued launcher | `42bfeedb886163908a63c618f7661f260f2c5edaf09c81d5b54cb2cdfab16f45` |
| Main evidence | `aeb9a2d569d2552e01de13dd9a8679dfd5a3bb0dffc7cf8b3c9345b7ae579c22` |
| Main final report | `491eb2f6c4c81e44ede0f6f161ec0b10b16fe79d62451674d710c1fba900e53d` |
| Main generator | `eab5a8dd441b6815c2aa645b7d26b317b3c11678d190d0048b786bd3d90198c8` |
| Main launcher | `3c3bbd1a7af4594b99c1357eb2513cccadaa0ec33b25d743c437b65e570c39cc` |
| Observer Git blob at 8eb26a4 | `6281adfec446552d7c08951a12dd654cb54a270013b0f844720e688e34532a11` |
| Codex Git blob at 5b1d1ef | `ab4e418ca769505f12d80d1b457afe99bed3d4e8d98b3378fbb536eef2b71a2d` |
| Agent-policy Git blob at 5b1d1ef | `dd4464f94d887b30ad603f331743d18e69936d0f898c74383595c34f43d4cb74` |

The continuation raw/grace SHA triplets match all nine values in the reviewed note;
the reviewed document's endpoint hash is unchanged. This review does not repeat its
entire 110-file provenance census or all 252/162/105 ordinary usage-payload checks:
the independent payload scope is the normal-H example and every listed resumed case,
plus the full timeout and transport-interruption checks described above.
No private reasoning bodies, active candidate costs or parked read-factor costs are
examined. No code, helper, frozen report, admission or disposition ledger is modified.
No agent-facing workflow is affected, so no real-provider verification is required or run.
