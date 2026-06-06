# Terminal UI Requirements

This document is the acceptance checklist for the terminal user interface. The sources are
`Readme.md`, `Spec.md`, and the interactive behavior requested in the project chat.

## Product Scope

`work-leaf` is an agent orchestrator launched from the project directory. The process opens an
interactive terminal workspace where the user can create and talk to Codex-backed user agents,
let agents propose patches through orchestrator-controlled machinery, enter review, and then
linearize the resulting work into a clean git history.

The top-level process starts the orchestrator. Workflow commands such as creating agents,
reviewing, and linearizing belong inside the UI command chat. Patch application and file-lock
classification are automatic agent/orchestrator interactions, not user-facing top-level commands.

## Screen Model

The UI is a full-screen terminal workspace. It must use the whole terminal viewport, not only the
upper-left portion. Rendered frames must keep stable terminal geometry across redraws.
The alternate screen is cleared when the UI opens; normal redraws update the frame in place without
emitting a full-screen clear sequence, so typing does not flash or blink.

The left pane occupies one fifth of the total terminal width when the right pane is visible. The
left pane is the control pane: it lists the work-leaf command interface and all running agents.
Each agent entry shows the agent id, broad feature/work description, readiness state, modified
files, agents touching the same files, dependencies on other agents, and agents depending on it.
Ready agents are visually highlighted.

The right pane shows the selected surface. It can show the work-leaf command interface or the chat
for the selected agent. Pressing `,` from command mode while the left control pane is focused hides
or shows the right pane. Pressing `,` while the right chat pane is focused does not close the active
chat.
An agent chat surface contains only that agent's conversation, loading state, and streamed Codex
events. It does not include command-chat help, global command output, or messages from other agents.

## Modal Input

The UI must feel like nvim. Mode-switching keys act immediately; they must not require Enter.

`Esc` always returns to command mode immediately. `i` enters insert mode immediately when pressed
from command mode. `:` opens the bottom command prompt only from command mode. Typing `:` in insert
mode writes a chat character instead of opening the command prompt.

The cursor stays inside either the left control pane or the right chat pane during normal command
and insert interaction. The cursor enters the bottom command prompt only after `Esc` followed by
`:`.
Insert-mode input is echoed as each byte is handled, including fast typing bursts.

## Pane And Window Navigation

`Ctrl-W` followed by `h` or `k` focuses the left pane from command mode. `Ctrl-W` followed by `l`
or `j` focuses the right pane from command mode when the right pane is visible.

Clicking an agent row in the left pane selects that agent, opens its chat on the right pane, and
places input focus in that chat. Clicking the work-leaf row opens the command surface. Terminal mouse
clicks use SGR mouse reporting while the full-screen UI is active.

Pressing `s` in command mode opens the selected chat in a split of the current pane. Pressing `t`
in command mode opens the selected chat in a new UI window. `gt` moves to the next UI window, and
`gT` moves to the previous UI window. Pressing `f` in command mode while an agent chat is selected
requests a fork of that chat.

## Command Chat

The command prompt belongs inside the full-screen UI. `:new [prompt...]` creates a new user agent
from inside the orchestrator, selects that agent, moves focus to the right chat pane, and enters
insert mode so the user can talk to the new session. When no prompt is provided, the new agent asks
the user what to work on from inside the chat.
The agent entry and chat surface appear immediately. Codex launch runs in the background and the
chat shows a progress indicator plus streamed Codex JSONL status/error/message output until the
session is ready or fails.

`review` and `linearize` are command-chat workflows. `patch` and `locks` are not user commands;
they are triggered automatically when agents need to modify files or interact with the filesystem.
The orchestrator may spawn internal system agents for review, linearization, and coordination with
user agents.

When an agent emits `@work-leaf read <path...>`, the orchestrator reads available project files
through file locks and sends the resulting file text back to that same agent session. Unavailable
paths are reported to the agent in the same response so the session can continue and answer with the
available context instead of stalling at the file request.

## Testability

The repository must provide an automated way to check terminal rendering contracts and a manual
way to launch a realistic UI fixture for visual and interactive inspection. The rendering check
must verify full-width frames, correct cursor placement, modal key behavior, pane navigation, and
the `:new` command behavior that opens an agent chat.

Run the manual fixture with `cargo run --example ui_harness`. The fixture opens a full-screen
terminal workspace with sample agents, raw single-key modal input, pane navigation, prompt mode,
and a local `new [prompt...]` command that creates and selects a fixture agent without contacting
Codex.

## Implementation Anchors

The interactive entrypoint is `src/main.rs::main`, which calls `src/cli.rs::run_cli_from_env`.
Launching with no top-level workflow command reaches `src/cli.rs::run_command_chat`, which chooses
`src/cli.rs::run_terminal_ui` when stdin and stdout are terminals.

Terminal state and rendering live in `src/ui.rs::TerminalUi`. Command-chat behavior lives in
`src/cli.rs::CommandChat`, and Codex-backed agent launching is abstracted by
`src/codex.rs::CodexBackend` through the `src/agent.rs::AgentBackend` trait.
