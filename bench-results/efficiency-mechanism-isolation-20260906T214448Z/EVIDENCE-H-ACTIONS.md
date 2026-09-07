# Historical action-path evidence

Scope: the accepted six normal-WL and six Direct workflows in H. The source-bound
[EVIDENCE-H-NATIVE-ACTIONS.json](EVIDENCE-H-NATIVE-ACTIONS.json) retains all 90 native
session identities, role counts, call-ledger digests, 58 exact mediated command headers
and 213 source identities. This is a provider-free exposure census, not a token
retotal, a new observation, a percentage estimate or a causal effect estimate.

## What actually differs

| Role, across six workflows | Direct native exec calls | WL native exec calls | WL additional mediated command handoffs |
| --- | ---: | ---: | ---: |
| Authors | 1,802 | 531 | 58 |
| Reviewers | 1,351 | 417 | 0 |
| Linearizers | 652 | 457 | 0 |
| Title worker | 0 | 0 | 0 |
| Total | 3,805 | 1,405 | 58 |

One native exec call can contain multiple shell commands; several native calls can
belong to one generated response. A mediated handoff is another interface, not an
extra native exec call. These columns must not be summed into a provider-response
count or multiplied by an assumed average token price.

The difference is not confined to source reads. Direct authors generate many more
check invocations and Git inspections; Direct reviewers perform more native
repository/diff inspection. WL's integration phase still performs substantial
native work, so moving work out of an author is not automatically eliminating it.

## C06/C17: repository inspection and host-owned state

Direct authors have 252 native calls whose first shell word is `git`, and its
reviewers have 363. All 18 WL authors and 18 WL reviewers have zero such calls.
WL's 300 leading-`git` calls belong to linearizers. A shell-prefix label is not a
complete shell parser or a measure of Git work inside another program.

The source chain is [PromptPolicy's mediated-access contract](../efficiency-raw-token-pilot-20260906T185328Z/source/src/agent.rs),
[orchestrator snapshot/read delivery](../efficiency-raw-token-pilot-20260906T185328Z/source/src/orchestrator.rs)
and [eager reviewer context](../efficiency-raw-token-pilot-20260906T185328Z/source/src/review.rs).
The H source-equality qualification and exact bundle retrieval examples are in
[EVIDENCE-H-CONTEXT.md](EVIDENCE-H-CONTEXT.md). WL native author/reviewer tools inspect
owned bundle snapshots rather than independently reading the live repository's
status, diffs and individual files. The snapshot/host state and review-evidence
routes overlap; this count cannot apportion their effects.

Both workflows have host-side commit responsibilities. Zero native author Git
calls is therefore not proof that automatic committing alone causes the saving.
The actual missing model-chosen inspection work, selected source delivery and
different review evidence are the relevant unresolved boundaries.

## C10/C13/C14/C15: validation cadence, not simply broad checks

Direct authors have 501 native calls beginning with `cargo`. WL's 58 mediated
command results all belong to authors and contain cargo test commands, including
shell-wrapped, chained and environment-prefixed forms. They retain actual failures
and command-line mistakes. All 45 native WL calls beginning with `cargo` belong to
linearizers; the corresponding Direct linearizer count is 31.

This is not evidence that Direct authors repeatedly run the full all-target gates.
For example, point7's first feature author executes `cargo test --test ui_harness`
11 times, `cargo test --test terminal_ui` nine times,
`cargo test --test terminal_app` nine times, and `cargo fmt` ten times.
Step4 Direct 003's third author executes the same focused terminal-app check eight
times and the same focused workspace check five times. These are repeated focused
validation cycles, not necessarily needless or broad validation.

Exact byte-identical command repeats within an author thread total 462 for Direct
versus two WL native repeats. WL mediated repeats are separate and remain visible
as all 58 command strings in the JSON; its failed checks and rechecks are not
dropped. Different bundle paths can make semantically repeated reads byte-distinct,
so this is not a semantic deduplication measure.

The demonstrated downstream contrast is repeated edit/read/check work. The causal
choice producing it remains to be separated across guidance, test timing, edit
representation, review evidence and context access. The old single ACK cue and
combined work-unit-policy null results remain valid; this census does not undo
them or call any one policy family a proven net saving.

## C25/C29: no exercised asynchronous native polling channel

Every one of the 5,210 native exec calls joins a unique same-thread
`function_call_output` with a completed-process header. None has the asynchronous
process-running header. Across all 90 sessions the only native function names are
`exec_command`, `apply_patch` and `update_plan`: no `write_stdin` or wait tool
appears. Avoiding model-generated asynchronous tool polling is therefore not an
exposed explanation in this H cohort. This does not claim that arbitrary shell
code cannot wait or that locks/interruption have no other effects.

## Reproducibility and limits

Native files are selected only from each original observation's
`rollout-metadata.jsonl`; their bytes must equal its stored source SHA.
The Direct role is bound through the saved `*.thread_id` files, checking that
resumes retain the same role. WL role ownership uses the first native policy
launch's owned Agent-ID footer, with the separate linearizer policy prefix;
the root instruction text contains no earlier such footer. Copied later reviewer
metadata is not treated as a new launch.

For each native `response_item` function call, record physical line, exact call ID,
command UTF-8 size and SHA. Join outputs by that same thread/call ID. Count leading
shell words with `shlex.split`; retain the single command that this lexical parser
cannot parse rather than dropping it. That command is a valid Python heredoc and
its exact native output exits 0 (H003 reviewer `01a04ecc-5fdd-7b90-8f7e-d398bf241844`,
native lines 71–72, call `call_6UnL0ozYDILOdP15XyYIbZXW`). It is not a failed shell
command. Hash the ordered call ledger using sorted-key compact
JSON. Output-byte fields include the native tool wrapper and are not charged
input tokens. Source endpoints are rehashed at completion.

Mediated input extraction requires the actual leading `work-leaf command result`
and command/status header, not a string mentioned inside copied source. Exact
accepted/public/native and archived command-result identity are independently
verified in the H context/lifecycle notes. This JSON does not reproduce private
reasoning or invent response IDs absent from H.

The first exploratory prefix script stopped on that `shlex` parsing limitation;
the complete census retains the successful command explicitly. An exploratory truncation-marker
counter mistakenly included the ordinary `Original token count` header and was
discarded before this artifact; no truncation frequency is used as evidence here.
No runtime, provider, benchmark admission, frozen helper or historical report is
modified by this audit.
