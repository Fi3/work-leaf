# Subscription-only tool-handoff feasibility check

## Scope

This isolated diagnostic uses the installed Codex CLI, the existing ChatGPT subscription login,
`gpt-5.5`, and `xhigh` reasoning. It runs no Work Leaf benchmark and changes no product runtime,
public interface, global configuration, or credentials. The model works in an empty scratch
directory under a read-only sandbox. Its only requested action is one synthetic dynamic tool call.
The local tool returns a fixed string; it executes no command and reads or modifies no file.
The diagnostic disables apps, multi-agent tools, image generation, shell, browser, computer tools,
hooks, and plugins through supported CLI feature flags. These diagnostic-only restrictions do not
describe either benchmark condition.

Public API-key authentication, separate API billing, provider substitutions, and copied credentials
are outside scope. The child environment excludes API keys and endpoint overrides, and
`account/read` must report account type `chatgpt` before any model turn. The CLI receives
`forced_login_method="chatgpt"`. Only authentication type is retained, never account email or tokens.

## Frozen experiment

One fresh thread receives one user turn. Register a directly visible function tool named
`handoff_probe` with a required string argument `request`. Ask the model to invoke it once with
`{"request":"ping"}`, then return exactly the tool's text without invoking other tools.
The tool returns `WORK_LEAF_SUBSCRIPTION_HANDOFF_OK`.

The client continuously consumes stdout through a dedicated reader, including while a server tool
request is outstanding. On `item/tool/call`, withhold the result for at most 10 seconds while
looking for a matching `rawResponse/completed` containing valid exact usage. Send the result when
that usage is observed, or at the 10-second bound if it is absent. The latter is a negative outcome
for pre-result accounting, but permits observation of whether accounting and completion follow
the result. A matching exact response already observed before the callback also counts. Matching
requires the callback's `callId` in a preceding raw function-call item of that completed response,
not merely an earlier response from the same turn.

Allow 45 seconds to obtain the tool request and 45 seconds after returning the tool result for
the turn to finish. Initialization and other client requests have 15-second bounds. The complete
diagnostic has a 150-second internal bound and a 180-second shell timeout with a further 25-second
forced-termination bound; cleanup has bounded
interrupt and process termination waits. No automatic retry, replacement, or second model turn is
part of this check. Cleanup interruption is recorded as such and cannot count as normal completion.

## Evidence and acceptance

Retain monotonic event timings, thread/turn/response/call identities, resolved model and effort,
validity of the synthetic arguments, expected-output equality, exact response usage, cumulative
thread usage, terminal state, and cleanup outcome. Do not retain raw reasoning, prompts loaded by
Codex, arbitrary tool arguments, account details, or credential-bearing error messages.

Pre-result accounting requires a matching exact completion for the fresh active turn before the
client sends the tool result. An item-completed or turn-completed notification alone does not prove
provider response accounting. The deliberately single-call case does not prove parallel-call
ordering or map arbitrary tool calls to upstream responses.

Full single-handoff feasibility additionally requires exactly one valid tool request, successful
tool completion, the expected final answer, a normally completed turn with no cleanup interruption,
and exactly two non-null, positive-usage upstream completions, one on each side of the tool result.
Unsolicited assistant messages before the result, extra responses, duplicate completion IDs, or
other active tool types reject this deliberately narrow case. Unique response usage must equal
the final thread totals for input, cached input, output, reasoning output, and total tokens. Cached
input cannot exceed input; reasoning is part of output; total equals input plus output.

Success establishes only this synthetic handoff under this configuration. It does not establish
architecture readiness, general reliability, concurrent handoffs, recovery of old runs, or exact
accounting for genuine cancellations/timeouts. Product integration requires a separately approved
design. The current first-batch gate in `PROTOCOL.md` remains unchanged.

## Protocol references

The [official app-server documentation](https://learn.chatgpt.com/docs/app-server#dynamic-tool-calls-experimental)
describes experimental tool registration and callback/result messages, without guaranteeing usage
ordering. The installed CLI's experimental JSON schema supplies the concrete field shapes. Its
`RawResponseCompletedNotification` describes exact upstream-response usage and allows null usage.
