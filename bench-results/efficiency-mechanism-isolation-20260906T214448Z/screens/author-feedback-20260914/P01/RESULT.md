# P01 input-fidelity check: blocked, not completed

Actual diagnostic: 2026-09-14T18:49:26.609988+00:00 to
2026-09-14T18:49:34.848605+00:00. P01 remains TODO; this is not a completion declaration.

## Verified observations

- The pinned CLI's local skills discovery first omits remote plugin skills and includes
  them after a 1.5-second wait plus reload in the same process, without an installed-list
  request. The installed files and metadata are present. This supports an asynchronous
  startup/discovery boundary, not missing plugin files or the model catalog-size budget.
- The startup-delay diagnostic completes the exact sentinel with no model tool calls,
  one response, one completed task, no compaction, no duplicate usage record and no
  missing tail: 15,662 input + 47 output = **15,709 raw tokens**. The 4,480 cached input
  tokens are included in input; 32 reasoning tokens are included in output.
- All twenty admission source endpoints remain equal. Base instructions match the
  saved reference. CLI 0.153.4, GPT-5.5/xhigh, subscription, read-only and never-approval
  remain in force. Native and public usage agree exactly.
- Developer context still has **7,042 bytes, not the reference's 7,817**. The only
  textual difference remains the remote skill-root alias and two plugin-skill entries.
- The native context is recorded at **18:49:28.839 UTC**; the user item is recorded at
  **18:49:30.857 UTC**, after the startup delay. Waiting at this hook is too late to
  restore that already-recorded context. The two public hook-trust warnings are retained.

## Evidence

- [Actual comparison and source hashes](ACTUAL-INPUT-COMPARISON.json)
- [Single new-response accounting audit](NATIVE-AUDIT.json), reusable by P10 without
  re-auditing the same native observation
- [Admission](input-001/ADMISSION.json), [terminal](input-001/TERMINAL.json),
  [public output](input-001/native.stdout.jsonl), [operator receipt](input-001/OPERATOR-TERMINAL.json)
- [Cold/installed-list discovery](LOADER-READ-TRACE.json) and
  [time-only discriminator](LOADER-TIME-DISCRIMINATOR.json)
- [Other local preview outcomes](LOCAL-PREVIEW-OUTCOMES.json),
  [hook configuration check](HOOK-LOCAL-TRACE.json),
  [fail-first test](DELAY-RED.txt), [six wrapper and six monitor tests](DELAY-AND-MONITOR-GREEN.txt)

## Boundary and next requirement

The startup-delay wrapper is **not qualified for feature benchmarks**. It is retained
as a failed private repair, not deployed to normal WL or substituted into P03/P04.
The required fix must make the installed-skill snapshot ready **before initial context
capture**, without supplying synthetic prompt text, changing native tools or rewriting
the saved reference. A longer wait at this same late hook is not the next experiment.

The P01 actual-diagnostic allowance is consumed. Any further actual verification needs
explicit approval; unused feature slots cannot replace it. P01 and dependent checks stay
unfinished. This does not justify reopening the historical benchmark-error branch, adding
a new task, declaring zero TODO, or reporting the historical saving as disproved.

## Scientific limits

This is input-setup evidence, not evidence explaining the historical ~50% saving. No
B-only, A+B, connected-joint or full-workflow mechanism benchmark ran in this resumption.
The accepted 45.38%–51.62% historical reduction and its **NOT ESTABLISHED** combined
causally explained percentage remain unchanged. No ordinary control, replacement run,
API credits, credential copy, normal WL implementation edit or global config edit occurred.
