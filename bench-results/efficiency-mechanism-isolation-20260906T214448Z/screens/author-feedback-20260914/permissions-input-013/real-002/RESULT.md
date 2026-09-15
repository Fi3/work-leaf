# Permission inheritance: rejected repair

Checked 2026-09-15 02:11 UTC. The admitted real-002 diagnostic completed at
02:04:43.167567 UTC in 14.285294 seconds. It used 47,780 raw tokens across three
distinct responses and two completed turns. Native/public accounting agrees,
with no incomplete tail; all 213 source pins and repository state match.

Omitting permission overrides from resume does not preserve the fresh turn's
permissions. The fresh context has danger-full-access / disabled / never; the
resumed context has read-only / managed-readonly / never. A second developer
message appears with SHA256
29dad5ed6993c5e717376e0a1c84d54ab60ee593510916ac6cee4295e803315d.
This fails the frozen effective-permission and whole-input qualification gates.
The command's successful exit is not a successful repair.

The existing full-workflow inputs and both failed diagnostic outcomes remain
unchanged. BUG009 stays open. No full benchmark, author replacement or automatic
retry follows this result. A subsequent repair must have a concrete distinction
and separate prospective bound/admission. The next local check considers whether
an explicit configuration-layer permission value, rather than omitted CLI flags,
can align fresh/resumed setup; it is not yet admitted for generation.

Evidence: DIAGNOSTIC-RESULT.json; NATIVE-INPUTS.json;
ONCE-ONLY-AUDIT-COMMAND.json; NATIVE-AUDIT.json; TERMINAL.json;
linearize-plan/REQUEST.json; linearize-accept/REQUEST.json.
The native audit core already ran once; later analysis reuses its saved result.
No causal token effect or historical share is inferred from this setup probe.

