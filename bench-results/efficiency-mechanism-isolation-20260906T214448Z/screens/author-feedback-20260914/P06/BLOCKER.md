# P06 — historical connected-state gate remains blocked

Checked: 2026-09-14T20:42:18+00:00. Local screening stopped promptly after source/state checks;
the 30-minute ceiling is not a target to spend. No model, fork, baseline re-audit
or normal runtime edit occurred. P06 remains BLOCKED in TODO, not DONE or negative.

## Evidence and partial positive result

STATE-AVAILABILITY.json and its exact command identify H003 client43, native
thread 01a04ebc-4c57-7cd2-8454-115efa9c3cd1 and the R07 overlapping window.
The native legacy history survives. The first changed file is exactly recoverable
in memory: base 42,898 bytes / fnv64:25dc74bb7b8daeba plus the recorded diff gives
43,482 bytes / fnv64:fa7dccad062e155b, matching the actual refresh. It would be
wrong to claim that no source content can be recovered.

## Missing common boundary

The original runtime checkout is absent. Saved Git checkpoints are base,
pre-linearize and final, not the selected refresh boundary. The pre-linearize
provisional object is absent in both inspected object stores; the final bundle
advertises only final HEAD 29908fc97bcc6697baeaf41dcd27569dbd355856. This does not
prove that every earlier source is unreconstructable, but it supplies no ready
exact checkpoint at the required multi-agent event.

The saved state.json contains command_transcript and session DTOs. It is not
an orchestrator checkpoint: it omits per-agent FileReadTracker snapshots, pending
command changes, write ownership, locks and backend attachment/queued events.
The native world_state contains model/instruction/permission configuration,
not that WL execution state. src/orchestrator.rs::FileReadTracker is a private
Arc/Mutex map; src/workspace.rs::WorkLeafController::new/snapshot create runtime
state and expose display DTOs, with no matching persisted-state restoration API.
Returning just the reconstructed full file through a new native turn would drop
the WL refresh/interrupt machinery and change the comparison.

Original CLI0.150.1 app-server/work_leaf input also needs faithful continuation;
the existing CLI0.153.4 fresh-exec startup repair does not qualify that bridge.
The previous outer-turn rollback and V11 records establish capability and
developer-reinjection limitations, not an eligible restored H session. No
unsupported fork/rollback of the original thread is performed to probe it.

## Disposition

No connected-state continuation is qualified through the existing permitted
machinery. A new historical multi-agent rehydration/interception framework
exceeds this bounded local check and is not silently built. Leave the joint
effect unresolved and preserve the useful single-file recovery and overlap.
Continue the already-running stronger B/A+B screens. A future continuation
needs an actually qualified state/continuation boundary or a justified alternative
comparison within the approved task; source availability alone is insufficient.
