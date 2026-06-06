# Work-Leaf Acceptance Checklist

This checklist defines the behavior the repository must satisfy for the initial work-leaf
orchestrator. It combines `Readme.md`, `Spec.md`, and the project chat requirements.

## Launch And Command Scope

`work-leaf` launches from the current project directory and opens the orchestrator workspace.
Workflow commands are entered inside the orchestrator command prompt, not as top-level process
commands.

`new [prompt...]` creates a user agent inside the orchestrator. A created agent appears in the left
control pane, becomes selectable from that pane, and has a right-pane chat surface. Text typed in
that chat is sent to the selected agent, and the agent response is shown in the chat transcript.
Agent creation updates the UI immediately: the left-pane entry and selected chat appear before the
Codex backend finishes launching, and the chat shows a loading indicator while Codex is starting.

`review` starts the review workflow from inside the orchestrator. `linearize` starts the
linearization question workflow from inside the orchestrator. `patch` and `locks` are automatic
agent/orchestrator mechanisms, not user commands.

## Terminal UI

The terminal workspace uses the full terminal viewport. The left pane occupies one fifth of the
viewport while it is visible. `,` hides and shows the left control pane from command mode. The
selected right-side chat or command surface remains visible when the left pane is hidden and expands
to the full terminal width.

The UI behaves like nvim for modal input. `Esc`, `i`, `:`, `,`, and `Ctrl-W h/j/k/l` act
immediately without requiring Enter. `:` opens the bottom command prompt only from command mode.

The visible cursor location must match the logical interaction point. The cursor is not pinned to
the upper-left pane corner. In the left pane it follows the selected command/agent row. In the right
chat pane it follows the chat input. In prompt mode it follows the bottom command prompt text.

Redraws do not clear the full screen on every keypress. The alternate screen is cleared when the UI
starts, then frames update in place so typing does not flash, blink, or drop fast input characters.

The left pane behaves like a navigable control tree. It lists the work-leaf command interface and
all running agents. Command-mode navigation selects entries, opens the selected entry, hides/shows
agents, and lets the user return to any agent chat. Mouse clicks on agent rows select that agent and
open its chat in the right pane. Ready agents are highlighted.

Agent entries show introspection: agent id, feature/work description, readiness, modified files,
conflicting agents, dependencies, and dependents.

The right chat pane shows only the selected agent session. Switching agents changes the visible
conversation to that agent's chat history and does not show command-chat help, global transcripts,
or messages from other agents.

## Agent Communication

The orchestrator can launch Codex-backed user agents and keep enough session state to send later
chat messages to the same Codex thread. Backend failures are shown in the orchestrator transcript
without exiting the app.

Codex launch and resume operations run without blocking terminal input. Codex JSONL status, error,
and agent-message events are streamed into the selected agent chat while the process is still
running, and loading indicators are removed when the operation completes.

The orchestrator can route text between agents where workflows require it. Review sends an original
agent summary to a reviewer agent, sends reviewer findings back to the original agent, and asks the
reviewer to recheck. Linearization sends reviewed commit information and user decisions to a
linearizer agent.

When an agent asks for file text with `@work-leaf read <path...>`, the orchestrator sends available
file snapshots back to that same agent session, reports unavailable paths in the same response, and
continues the follow-up loop so the agent can answer after receiving the file text.

## File Locking

Every agent prompt includes rules that forbid direct file reads and writes. Agents must ask the
orchestrator for file text and must provide unified diff patches for writes.

The orchestrator provides read access through shared read locks and write access through exclusive
write locks. Paths cannot escape the project root.

The command write policy classifies common build, test, format, package, and compiler commands as
write-producing so those commands can be mediated by the orchestrator.

## Patching

Patch requests lock touched files for writing, check the unified diff with git, apply it when it is
clean, stage touched files, and create a provisional metadata commit. The commit records the agent
id, feature, and patch reason.

Patch conflicts keep the worktree clean and send diagnostics back to the original agent so it can
provide a corrected patch.

## Review

The review workflow scans git history, finds the latest metadata commit for every agent id, asks
the original agent for a summary, launches a reviewer agent for that chat id, sends findings back
to the original agent, and asks the reviewer to recheck until no findings remain or the configured
round limit is reached.

## Linearize

The linearize workflow asks the user, for every reviewed chat id, whether the patch should keep a
final commit, integrate into another commit, or be grouped with other chat ids.

The linearizer agent receives reviewed commits, user decisions, groups, and verification commands.
It is instructed to produce a minimal coherent history and iterate until verification passes.

## Testing And Inspection

The repository provides automated tests for every item in this checklist and for the requirements
in `Spec.md`.

The repository provides an inspection tool that lets agents see terminal rendering and drive real
orchestrator behavior with a fake backend. The inspection path must exercise the same command-chat,
agent-launch, and agent-message code as the real UI, not a disconnected fixture-only state machine.
