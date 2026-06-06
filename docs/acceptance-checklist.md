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

`review` starts the review workflow from inside the orchestrator. `linearize` starts the
linearization question workflow from inside the orchestrator. `patch` and `locks` are automatic
agent/orchestrator mechanisms, not user commands.

## Terminal UI

The terminal workspace uses the full terminal viewport. The left pane occupies one fifth of the
viewport while the right pane is visible. The right pane can be hidden and shown from command mode
with `,`.

The UI behaves like nvim for modal input. `Esc`, `i`, `:`, `,`, and `Ctrl-W h/j/k/l` act
immediately without requiring Enter. `:` opens the bottom command prompt only from command mode.

The visible cursor location must match the logical interaction point. The cursor is not pinned to
the upper-left pane corner. In the left pane it follows the selected command/agent row. In the right
chat pane it follows the chat input. In prompt mode it follows the bottom command prompt text.

The left pane behaves like a navigable control tree. It lists the work-leaf command interface and
all running agents. Command-mode navigation selects entries, opens the selected entry, hides/shows
agents, and lets the user return to any agent chat. Ready agents are highlighted.

Agent entries show introspection: agent id, feature/work description, readiness, modified files,
conflicting agents, dependencies, and dependents.

## Agent Communication

The orchestrator can launch Codex-backed user agents and keep enough session state to send later
chat messages to the same Codex thread. Backend failures are shown in the orchestrator transcript
without exiting the app.

The orchestrator can route text between agents where workflows require it. Review sends an original
agent summary to a reviewer agent, sends reviewer findings back to the original agent, and asks the
reviewer to recheck. Linearization sends reviewed commit information and user decisions to a
linearizer agent.

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
