# Single-attempt command recording

`bounded_launch.py` executes exactly the admitted argv with its pinned explicit
environment. It constructs no provider arguments and admits no retry. The actual
C15 argv must contain the separately frozen 300-second timeout/five-second kill
grace; the helper does not infer a deadline or authorize an arbitrary command.

Initial tests fail before the module exists (`718355`). Four synthetic cases then
pass (`9a1c65`). Two additional REDs (`a00a28`, `997e30`) reproduce admission and
environment reread races: a later file read could differ from the bytes checked.
The executing configuration uses exactly the bytes hashed, and all six tests
pass (`6aa64c`, 0.034 seconds). Independent complete-source review and six tests
also pass (`375878`, 0.033 seconds). These are local Python child fixtures, not
provider invocations or real diagnostic attempts.

The automatic cases cover original nonzero exit, stable terminal condition/id/time
fields, rejected reuse, prelaunch source/digest failure, actual spawn failure,
and execution from the exact verified admission/environment bytes. No original
failure is converted to success by a later source checker.

Publication is exclusive create-new and fsynced, not an atomic-rename/crash
transaction. A partial attempt or output prevents a rerun and remains evidence.
`TERMINAL.json` records the supervised command's original return code and UTC
timestamps bracketing its execution, not an invented internal provider-turn time.
Its factor condition is `private-test-first`; `observer_condition: work-leaf`
preserves the workflow-family distinction. Source/admission endpoint failures
are separate fields and never replace the command exit.

This is a trusted, source-frozen operator helper, not a hostile-admission parser,
continuous integrity monitor or private-child closure proof. Input endpoint checks
are linear byte passes. Final executable/root/environment/source freeze and one
create-new admission are root-owned gates; no real call has run here.
