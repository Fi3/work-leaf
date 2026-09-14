# P01 startup-delay input qualification

This is the single P01 read-only actual-input diagnostic, not a feature screen or a
token-saving observation. It uses the existing ChatGPT subscription through the unchanged
native CLI 0.153.4, GPT-5.5/xhigh, read-only sandbox and no-approval policy. API-key and
alternate endpoint variables are removed by both the admission environment and frozen provider.

## Concrete repair under qualification

The saved local loader trace has no remote plugin skills at 0.2305 seconds, and has the
installed skills after a 1.5-second wait and reload at 1.7682 seconds, without an intervening
installed-list request. A separately listed installed-plugin call also precedes successful
discovery. This supports asynchronous initialization, not a context-size or missing-file cause.
Config-preview enablement and private local-marketplace routing did not restore the catalog.

The benchmark-private wrapper configures one synchronous SessionStart hook matching only
`startup`: `/usr/bin/sleep 2`, timeout five seconds. The command emits no output or context.
No hook is added to native resume invocations, preserving the reference's resume behavior.
The invocation-local hook-trust bypass is limited by the admission's source pins and the
checked absence of other active hook definitions. No permission/sandbox bypass is used.
The model-facing task, base instructions, model, effort, native tool availability, actual
file contents, provider and normal host protocol remain unchanged. No synthetic skill text,
fake tool receipt, replacement reference, provider binary edit or global config edit is permitted.

Official hook output and startup semantics are described by the
[OpenAI Hooks documentation](https://learn.chatgpt.com/docs/hooks). The pinned native diagnostic,
not that documentation or a debug preview, decides whether the repair actually works.

## Observation and budget

Exactly one fresh diagnostic, P01-input-001. The fixed prompt asks only for the sentinel
WORK_LEAF_INPUT_FIDELITY_OK, with no tools or repository changes. The existing R13 checkout
is reused read-only at its unchanged reference commit; artifacts have a separate directory.
The frozen host enforces a 60-second timeout and retains partial output/accounting tails.
Independent monitoring checks the owned public and native totals every five seconds and
stops at 100,000 recorded raw tokens; these two representations are not added together.
This is a recorded-usage tripwire, not a hard bound on unreported in-flight usage.
There is no retry, replacement, extra diagnostic, generated feature work or ordinary control.
All local preparation remains inside P01's existing deadline, 2026-09-14T19:04:17+00:00.

## Pass and stop gates

The wrapper's six fail-first local tests require explicit benchmark opt-in, recursive-launch
rejection, unchanged incoming arguments, startup-only scope, native resume preservation and
zero hook stdout/stderr. The six existing monitoring failure/cancellation tests remain required.
These are setup checks only; they are not a positive mechanism result.

Actual qualification requires the saved first developer message (7,817 UTF-8 bytes) and base
instruction hashes to match exactly, the pinned CLI/model/effort/approval/sandbox to match,
one completed sentinel-only response with complete distinct-response usage, no tools or file
changes, and all source endpoint hashes unchanged. Any extra hook context, retained missing
catalog, untrusted-hook rejection, time/usage stop or source drift fails qualification.
Failure preserves the observation and leaves P01 blocked/TODO; dependent feature checks stay
unrun/TODO. Success permits P02's exact comparison qualification before P03/P04 admission.
