# R03 resource-publication failure

2026-09-14 13:53 UTC. The operator's resource loop omitted creation of its
R03/resources directory before invoking the frozen host's create-new save_json.
That helper does not create parents. The first sample therefore raised
FileNotFoundError for resources/sample-0001.json. The exception branch signalled
the exact R03 supervisor with SIGINT; its subsequent attempt to save the error
under the same absent directory also raised FileNotFoundError.

Original tool receipt bd0fba retains both tracebacks. The original benchmark
TERMINAL.json records exit 1 after 1.068679s, 13:53:02.460471–13:53:03.529143 UTC.
host/invocation-0001/exit.json retains the native child cancellation. The created
native thread is 01a0a031-552b-7f53-a428-30aa5b98d2d3. Source endpoints match;
there are no accepted edits, source remains the original base, and no complete
model turn or recorded response usage exists at the terminal resource check.
An in-flight/unreported charge is unknown, not established zero.

This is an operator setup error, not a B-factor result or provider failure.
No replacement, repaired restart, different arm or extra observation is admitted.
The missing directory is not created to conceal the original failure. R09 retains
this outcome and its accounting tail; R08 cannot treat it as a negative B effect.
