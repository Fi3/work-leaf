# Diagnostic002 postcapture qualification

The sole attempt failed: [TERMINAL.json](../TERMINAL.json) retains exit **101**, reported duration 26.80 seconds, scenario false, and final accepted thread settled. No reviewer or C21 archive was reached. There is no green C21 qualification, replacement outcome, retry authority, or token-effect claim.

The original observer `analyze` and `extract-rollouts` and original source collector each ran once; exact commands, stdout/stderr and exits are retained beside their reports. [Analyze](OBSERVER-ANALYZE-ORIGINAL.json) exits **2** with eight executable marker-scan errors across the two pinned observer proxy binaries. [Extract](OBSERVER-EXTRACT-ORIGINAL.json) exits **0**, matching the one actual thread. The [original collector](REVIEW-EVIDENCE-SOURCE-AUDIT.json), SHA `c0480de9575301e017a71daaf51caf061db061021178d93bfccdea7d35583125`, exits **1** with `terminal run/condition differs`.

That collector error is a publication-contract incompatibility, not a model failure: the preserved002 terminal uses `work-leaf-diagnostic-terminal-v1`, an operator-observed closed time, and no `condition`/`finished_at`, whereas frozen `terminal()` requires the older diagnostic shape or a published workflow row. No legacy-shaped terminal was fabricated and no collector was repeated to bypass it. The qualified frame CLI was not invoked merely to hit the same terminal gate. The independent witness instead uses the unchanged hash-pinned pure frame comparator with exact closed capture/source checks.

## Exact observed chain

[INDEPENDENT-DELIVERY-WITNESS.json](INDEPENDENT-DELIVERY-WITNESS.json), SHA `b1cbc3f0d8ce593d4ea2b84f7aaec5baaa9ad219095a405c63319d67efdce328`, contains the full metadata-only identities and 26 source hashes. Its zero-provider-call field applies to this offline review, not to the diagnostic, which did start. Physical C/S lines refer to `observation/app-server/00000505904194804965-14/client-to-server.raw` and `server-to-client.raw`. Native N lines refer to `/home/user/.codex/sessions/2026/09/07/rollout-2026-09-07T20-30-44-01a07d23-0c8e-75f1-bff2-2f740c5a379f.jsonl`, SHA `878f4420281374f2f3aff1b036c243ff61fe249395914419bc47d8fc3fdd190c`.

| Boundary | Exact witness |
| --- | --- |
| Author launch | C3 → S3 creates thread `01a07d23-0c8e-75f1-bff2-2f740c5a379f`; trace line2 owns agent `user-review-evidence`. |
| Accepted author input | C4 → S6 accepts turn `01a07d23-0cfb-7b80-92d0-1367ae0efa7c`; S11 public user item and N7 native `user.text` match its entire 6,759-byte input. Explicit native turn metadata agrees; no last-context inference is used. |
| Unplanned native action | N11 `exec_command`, call `call_dzI8VKtFVCGEVNkyvB0AykbG`, requests `cat -- ''`, `login=false`, `max_output_tokens=20000`; N13 matches the same call and explicit turn. |
| Native pre-execution failure | N13 reports exit101 and failure to open `/tmp/codex-bwrap-synthetic-mount-targets-1000/lock` because of a read-only filesystem. This is before shell execution, not an actual empty-path file open. No public `commandExecution` item represents this native call/output pair. |
| Wrong-role public reply | N18 = S35, item `msg_0219b09f095e1306016a9f02ec8ff087d2840aedea5245acae`, emits exactly `@work-leaf locks run fixture.rs -- sh ./check-fixture.sh`. The author has emitted the reviewer-step directive instead of its required EDIT. |
| Settled interruption | C5 is forwarded exactly, S39 acknowledges it, and S41 terminates that exact turn as interrupted. |

The only trace rows are activation and the author policy delivery. Original/forwarded policy bytes agree and contain the declared FEATURE exactly once. The pure frozen comparator verifies all five client frames; only initialize C1 and thread/start C3 contain the permitted journalled metadata additions. C4 input and C5 interruption bytes remain identical. Captured/public/native accepted-input and thread censuses are complete, including the separate non-`user.text` native environment/abort messages retained as metadata rather than falsely counted as provider input requests.

Native N1 and N6 plus accepted server settings establish CLI0.153.4, `gpt-5.5`, `xhigh`, the owned project cwd, approval never and read-only sandbox. The one project inventory boundary matches its journal/baseline, its entries digest recomputes exactly, and it finishes before `child.json`'s provider start. It records absent project `.codex` layers and no inventory errors. This is a recorded boundary, not continuous filesystem immutability.

The native/public mismatch matters: absence of a public tool item did **not** prove absence of native tool activity. No reasoning body or token total is exported. All156 admission endpoints and the global configuration SHA `21bcb4825009b9561fa200a806ccc7d68993cbd4e718342b257a594dfaf31908` match at the postcapture check.

## What did not happen

There is one accepted author turn, no edit group, no patch ACK, no reviewer thread, no issued archive, no captured locked command, no command-result handoff and no clean-review verdict. The command was a generated directive, not an executed locked check: immutable harness `BoundedBackend::check_reply` rejects it on call1 before `CommandChat` processes an accepted reply. The fixture remains `VALUE = 0`, Git is clean, and the only commit is the initial fixture commit `766cdd974d681197ef0046cc1da78b163a2ea24e`.

The delivered FEATURE includes the reviewer instructions in the author task label, as prospectively disclosed. The author actually attempted their native read and emitted their CHECK. This demonstrates conflicting role instructions in the diagnostic input/action chain; it does not establish the model's hidden reason. The separate native sandbox pre-execution failure also remains a qualification condition.

[NATIVE-PREEXECUTION-METADATA.json](NATIVE-PREEXECUTION-METADATA.json) records read-only postclosure `stat`: the registry directory and empty lock belong to uid/gid1000, with modes0755/0644. Those parent-environment permissions do not show what was writable inside the exited child's mount namespace. No lock write, permission/config repair, doctor invocation, native retry or provider call was performed.

## Bounded next-fixture design only

The smallest role-safe qualification is **a deterministic, explicitly labeled author fixture plus one real Codex reviewer**, not another two-real-agent instruction experiment. A private test `AgentBackend` can return a fixed author EDIT and then clearly labeled scripted non-Git evidence/DONE through ordinary `CommandChat::launch_prepared_agent_streaming`. Normal GitPatcher acceptance and the normal review call remain real host operations; the backend's author `AgentSession` must retain that exact scripted history. `handle_line("review")` then routes only the reviewer launch/send to the real Codex backend. Existing immutable `tests/bench_review_evidence.rs::Backend`, its session hook, and `review_evidence_child` demonstrate the provider-free session/source seam; the real smoke's bounded backend demonstrates the reviewer routing/acceptance checks.

Reviewer-only qualification wording can occupy the fixture's feature because no model consumes the scripted author launch. The unchanged C21 hook still archives the complete held commit/log/recorded-chat context before actual reviewer policy injection. Nothing is rewritten after the adapter. The real reviewer must issue the exact full archive read, then the focused locked CHECK; only its actual successful result permits `NO_FINDINGS`. Two scripted author replies and two real reviewer turns would be declared separately, with one real thread and an explicit finite provider-call/watchdog limit. Scripted source evidence must never be described as generated by a real author or as full real-agent end-to-end verification.

That design removes the observed role-confusion path but does **not** qualify the current native-access environment:002's pre-shell registry error could still block the real reviewer. Any next admission must explicitly retain that remaining condition and have separate user authority; broader permissions, environment repairs, another provider attempt or a benchmark are not authorized by this suggestion. No implementation or generation follows from this note.
