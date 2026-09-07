# Independent historical action census review

The source-bound H action census has no blocking count, source-join or causal-framing
finding. This review uses the following original cutoff identities; later wording
clarification is separate from this independently reproduced numerical record.

| Reviewed artifact | SHA-256 |
| --- | --- |
| `EVIDENCE-H-ACTIONS.md` | `3f60af932dcbaf23bda61479e4d553f41409a59702005e2149e248f2303ca8c0` |
| `EVIDENCE-H-NATIVE-ACTIONS.json` | `0d41c6f77cde06974f33b6eb399cb89e7a2693db83e5ed99d615a8b79b02a857` |

A provider-free, 180-second-bounded independent replay verified all 213 source
hashes before and after inspection. All twelve observation metadata populations
match the 90 retained native sessions, with no duplicate thread IDs. Direct roles
match the saved implementation/fix/review/linearizer thread-ID files, including
resume consistency. WL roles match the first owned native policy launch and its
Agent-ID footer; earlier native user instructions have no competing footer.

All 90 ordered exec-call ledger hashes reproduce from the physical line, exact
call ID, command UTF-8 size and command SHA using sorted-key compact JSON. Every
one of 5,210 native exec calls joins a unique later same-thread output. Every
actual output wrapper has a completed-process header; none has an asynchronous
process-running header. The only observed native function names are
`exec_command`, `apply_patch` and `update_plan`.

| Role | Direct native exec calls | WL native exec calls |
| --- | ---: | ---: |
| Author | 1,802 | 531 |
| Reviewer | 1,351 | 417 |
| Linearizer | 652 | 457 |
| Title | 0 | 0 |
| Total | 3,805 | 1,405 |

All recorded leading-word/Git-word counts and output-byte fields reproduce.
Byte-identical author-command repeats after the first occurrence reproduce as
462 Direct and 2 WL. The point7 focused-check/format examples reproduce as
11/9/9/10; the step4 Direct 003 third-author named terminal-app/workspace checks
reproduce as 8/5. All 58 mediated command headers reproduce their physical client
line, thread, author role, exact command, status and full delivered-input hash.
These handoffs remain separate from native exec calls and provider responses.

The sole `<unparsed-shell>` label is a `shlex` lexical limitation: exact-normal-003
reviewer thread `01a04ecc-5fdd-7b90-8f7e-d398bf241844`, native call line 71,
`call_6UnL0ozYDILOdP15XyYIbZXW`, starts with `python - <<'PY'` and joins output
line 72 with exit **0**. Its 651-byte command SHA is
`1cd89be6801273d698d6280988ff469335792fd74010c62f3e71e9adb5601392`.
It must not be described as a malformed or failed shell command. The retained
label/count is correct; the prose clarification distinguishes parser limitation
from execution failure.

The evidence explicitly avoids equating command counts with response counts,
pricing output bytes, attributing the historic percentage, or asserting a causal
policy effect from this exposure census. Native-only counts do not eliminate
mediated work, integration work or hidden title responses. No private reasoning,
active candidate costs, provider calls, evidence rewrites or frozen-file changes
were part of this review.
