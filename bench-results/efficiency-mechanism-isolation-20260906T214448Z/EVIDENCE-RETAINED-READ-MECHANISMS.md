# Retained large-read delivery and retrieval evidence

Scope: steps 1–3 of [the retained-read plan](PLAN-RETAINED-READ-EVIDENCE.md), within
the broad candidate investigation. This is a complete source-only census of the
six actual R workflows, in run-ID order. No provider runs, accounting functions,
whole-workflow totals, percentages, primary-shaped report or randomization test
belong to this evidence. W remains a separate historical reference.

## Population and preserved qualifications

[The phase source receipt](phases/untracked-reads-01/postcapture/PHASE-SOURCE-REVIEW.json)
verifies all 114 original frozen files, the original manifest
`706c0b13d62376b3ea10c4604bb63957c271a1a7224d39b27db20fa24e2da263`,
all twelve ordered schedule/result identities and all six exact exit receipts.
001 succeeded; 002/003/004 timed out; 005/006 have successful workflow exit codes
but retain their original infrastructure/measurement classifications. Unlaunched
007/010/012 controls remain canceled; 008/009/011 treatments remain held. Their
original `not_launched_after_signal` rows have no fabricated usage or replacements.

All six original observer reports have `capture_complete:false`. Their interrupted
usage-gap counts are respectively 2/3/10/12/4/0. The additional missing controller
rows are user-2 in 005 and user-1/user-2 in 006. The phase retains both behavioral
and unexplained configuration drift. The paused monitor interval, original final
`FileNotFoundError` trust classification and separate 005 hold-time prefix proof
remain distinct. Source hashes do not establish continuous configuration integrity.

## Complete delivery census

Each run directory under [postcapture](phases/untracked-reads-01/postcapture) contains
`SOURCE-DELIVERY-ORIGINAL.json` and `RETRIEVAL-SOURCE-REVIEW.json`. The first preserves
the complete unchanged frozen `prompt_inventory` result, physical source locators,
every accepted input/native item identity, capture provenance and original flags.
The second retains every issued-path call candidate, exact call/output identities,
archive witnesses, and every read disposition. No favorable examples replace the census.

| R run | Condition | Captured/native threads | Accepted inputs | Read handoffs / eligible | Native path candidates | Selected-content calls | Other actual candidate outcomes |
| --- | --- | ---: | ---: | ---: | ---: | ---: | --- |
| 001 | control | 8 | 56 | 15 / 14 | 169 | 166 | 3 parser failures |
| 002 | control | 6 | 33 | 10 / 9 | 124 | 123 | 1 empty search |
| 003 | inline | 6 | 38 | 13 / 13 | 0 | 0 | No explicit archived-path candidate |
| 004 | control | 6 | 28 | 11 / 11 | 131 | 130 | 1 out-of-range empty selection |
| 005 | inline | 8 | 64 | 22 / 16 | 0 | 0 | No explicit archived-path candidate |
| 006 | inline | 8 | 49 | 15 / 13 | 0 | 0 | No explicit archived-path candidate |

The frozen delivery checker is `available`, with `errors:[]`, for every run; no
correction or trace conversion was needed. All original/forwarded turn-start bytes,
typed replies, selected prompts and reverse coverage match. Policy/ACK/command
handoffs are unchanged (respectively 8/7/7, 6/6/6, 6/5/5, 6/3/3, 8/8/9, 8/8/8).
Every thread's last accepted turn has an actual terminal notification; terminal
includes interrupted and failed, not just completed.

All captured thread starts match the complete native metadata roster, including
title and linearizer sessions. Membership does not depend on usage: the full scan
finds no usage-less native thread in this particular population. All 268 accepted
inputs have a distinct native `user.text` item with the exact explicit thread/turn
and full input text; all native `user.text` items are consumed once. Host instruction
items remain separate. Native session metadata is CLI 0.153.4, and every recorded
turn context is gpt-5.5/xhigh in its captured cwd. Primary raw metadata and the
1000-ms forward grace are verified from invocation start/end and exact rewrite
provenance. Original rollout audits have no missing or unobserved-cwd threads;
their saved verdict is separate from the independent identity checks.

## What the representation actually does

All 76 eligible bundle archives match the original source formatter applied to
the exact held snapshot bytes in the inline candidate. FNV labels are checked as
FNV; archived and candidate bytes also have independent SHA-256 identities.
The frozen validator proves equality outside the owned component. In 006 trace
line 11, only untracked `src/cli.rs` changes representation; six unchanged tracked
snapshots retain their ordinary suffix representation. Ineligible reads stay visible.

All 42 eligible inline handoffs deliver their complete untracked bodies in the
exact native user input. The 34 baseline eligible handoffs instead deliver a bundle
manifest. Thirty-three have subsequent exact source-content retrieval witnesses;
one fails at the provider before any assistant/tool output (below). No candidate
produces a whole bundle as one exact native output. This does **not** establish
that no full file or full bundle was eventually reconstructed across many outputs.

All 424 candidate calls have exact native call-ID/output and public
`commandExecution` thread/turn/item joins, exact command equality and matching
exit status. The leading programs are sed, rg, grep, nl and awk. For 310 closed
numeric-sed commands, the complete command-produced output is reconstructed from
archived line ranges; 309 also match the native delivered body exactly. One
numeric compound selection has native display truncation. The remaining successful
queries carry exact archived-line witnesses, not a claim that their entire output
has been semantically reproduced. Six native displays are truncated (4/1/1 in
001/002/004); the untruncated public command output and delivered native display
are retained separately. Null public output for the two empty-result cases is not
silently converted to an empty string.

For multi-bundle commands, a witness must identify that exact archive: either an
explicit output path and line, or a fully reconstructed numeric-sed segment. A
common line from archive A cannot establish content delivery from archive B.
Path substrings alone are retained as candidates, never operating-system file-open
counts. Absence of explicit path references is not absence of indirect retrieval.

## Concrete chains and exceptions

[The public context receipt](phases/untracked-reads-01/postcapture/PUBLIC-CONTEXT-EXCEPTIONS.json)
records all raw error events and each first eligible read's subsequent same-turn
public messages, plus the inherited reviewer-path case.

- 001: trace line 5 → client line 10 → native user item
  `msg_01a07add-c09d-7c20-a884-0d436f4f3afc` at native line 22. The 744-byte manifest
  points to the 147,925-byte archive. Native call `call_EGJt6v4dPWKaifNka96c5RIU`
  at line 26 executes `sed -n '1,220p'`; output line 29 and raw public line 185
  deliver the exact 13,099-byte selected range. Later same-turn native selections
  precede another actual `@work-leaf read` at raw line 614. These are retrieval
  operations inside one outer provider turn, not 7 new outer launches or inferred
  completed-response counts.
- 003: trace line 5 → client line 10 → native line 22 delivers 201,802 bytes of
  selected full text. Its next completed public message, raw line 285, requests
  further project files. The initial requested file sets differ from 001, so their
  byte counts are not a matched counterfactual comparison.
- 001 reviewer-user-2: five calls at native lines 190/195/200/207/214 use bundles
  7/1 inherited in the exact reviewer launch input, native line 7, rather than in
  that reviewer's own read handoff. The pure census correctly retains
  `not_issued_to_this_thread`; this separate exact launch-input witness explains
  the source without relabeling the original read inventory.
- 001's three failures are isolated rg commands: one unrecognized option and two
  unmatched shell backtick quotations. Their exact error-only output is retained.
  The classification rule does not erase earlier output from a compound command.
- 002 raw line 4297 and 003 raw line 2167 report `serverOverloaded` for user-1;
  these original failures are not attributed to the representation factor.
- 004 bundle-5: accepted native input line 102 is followed by raw line 1253
  `serverOverloaded`, then line 1254 failed turn. No later assistant or native tool
  output exists in that thread. Its absent retrieval is an actual failed-response
  case, **not** evidence that the agent intentionally avoided reading the bundle.
- 005 retains four `responseStreamDisconnected` retry notices, raw lines
  3783/5899/15138/20914. Those visible retries and the controller/usage gaps are not
  resolved by the otherwise exact input-delivery proof.

The demonstrated mechanism is the source-delivery route: immediate complete
untracked text versus a compact manifest followed by model-selected native
retrieval commands and selected returned text. This establishes the actual
representation and action path, including its retrieval overhead. It does not
establish net token savings, a counterfactual action count, attribution of later
work, or a share of the accepted historical WL reduction.

## Reproduction and qualification

The new stdout-only operational scripts are
[replay_source_only.py](phases/untracked-reads-01/postcapture/replay_source_only.py)
and [replay_retrieval_only.py](phases/untracked-reads-01/postcapture/replay_retrieval_only.py).
Their SHA-256 identities are `2a23819818a4e87109147f7f79db934fbf65342ee7c0a576ff6693b880a2fcea`
and `4adcc05dfed501fcaa192711fe76d7629c5bfe9ea43a4e08341bf425a4bba83d`.
Run either with `python -B <script> --run N` for N=1…6; the first without `--run`
replays the original phase-source population. No command from a captured agent
is executed. Existing frozen helper bytes, actual `__file__` paths and sources are
revalidated at both endpoints; native usage records and private reasoning bodies
are omitted from this pass before the frozen native inventory function is called.
The frozen pure delivery/native helpers are respectively `30a58a93…` and `2ad81ed5…`.
Neither `derive_run`, `replay_launched_sources`, accounting nor a primary CLI runs.

[Three provider-free scope checks](phases/untracked-reads-01/postcapture/test_retrieval_scope.py)
pass. The multi-bundle propagation and compound-failure defects reproduced actual
assertion RED before their fixes; the extra shared-line guard was also introduced
under a failing test. Initial operational qualification retained two harmless
failed assumptions: invocation-index end metadata is a subset of the fuller end
receipt, and archived bundles have their own formatter rather than the inline
header. Original frozen-checker results were never edited. No real-agent workflow
is affected by these offline evidence scripts; no additional provider verification
or runtime build is needed for this source-only work.
