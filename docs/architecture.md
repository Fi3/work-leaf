# Work Leaf Architecture

This document describes the stable architecture and extension boundaries for Work Leaf. Code changes
should preserve these boundaries unless a human explicitly authorizes an architecture change.

## Public Crate Surface

`src/lib.rs` is the public crate index. It exposes the modules that external callers, tests, UIs, and
agent-provider integrations use:

- `agent` and `agent_runtime` define provider-neutral agent domain types and runtime contracts.
- `workspace` defines the UI-neutral controller and DTOs used by frontends.
- `cli` defines the command-chat API used by the binary and by controller orchestration.
- `codex` defines the Codex provider implementation.
- `orchestrator`, `patch`, `review`, `linearize`, and `locks` define core workflow behavior.
- `terminal_app`, `ui`, and `ui_harness` define the terminal frontend and terminal-specific tests.

Public re-exports in `src/lib.rs` are part of the supported integration surface. The most important
public interfaces are:

- UI integration: `WorkLeafController`, `WorkLeafSnapshot`, `WorkLeafSession`, `WorkLeafEvent`,
  `WorkLeafLoading`.
- Agent-provider integration: `AgentBackend`, `AgentStreamEvent`, `AgentShutdownHandle`,
  `AgentProfile`, `AgentKind`, `AgentLaunch`, `AgentSession`, `AgentId`, `ChatMessage`,
  `MessageRole`, `PromptPolicy`, `ReadPermission`, `AgentError`.
- Command orchestration: `CommandChat`, `CommandChatResult`, `ProcessCommand`, `CliError`.
- Core workflows: `AgentOrchestrator`, `GitPatcher`, `PatchCoordinator`, `GitHistory`,
  `ReviewCoordinator`, `LinearizePlanner`, `FileLockTable`.
- Terminal UI: `TerminalApp`, `TerminalUi`, `UiHarness`, `UiAction`, `UiKey`, `UiMode`,
  `UiSurface`, `PaneFocus`, `AgentListEntry`.

## Layering

The application is organized as four layers.

1. Provider-neutral domain and runtime contracts live in `src/agent.rs` and
   `src/agent_runtime.rs`.
2. Core workflows live in `src/cli.rs`, `src/orchestrator.rs`, `src/patch.rs`, `src/review.rs`,
   `src/linearize.rs`, and `src/locks.rs`.
3. The UI-neutral application controller lives in `src/workspace.rs`.
4. Frontend adapters live in `src/terminal_app.rs`, `src/ui.rs`, and `src/ui_harness.rs`.

The dependency direction is inward. UIs drive `WorkLeafController`; the controller drives
`CommandChat`; `CommandChat` drives the active `AgentBackend` and the workflow coordinators. Core
workflow modules do not depend on terminal rendering, terminal input, or a specific agent provider.

The binary entry point in `src/main.rs` calls `work_leaf::run_cli_from_env()`. CLI setup creates a
`CommandChat<B>` with a backend and passes it to interactive or command-driven flows. The terminal
frontend wraps that same command-chat state in `WorkLeafController<B>` through
`src/terminal_app.rs::TerminalApp`.

## Agent Domain

`src/agent.rs` owns provider-neutral agent data:

- `AgentId` validates stable agent identifiers.
- `AgentKind` identifies the provider kind. `AgentKind::Codex` is the built-in provider, and
  `AgentKind::External(String)` identifies non-Codex providers.
- `AgentProfile` carries the active provider kind, display name, and default feature label.
- `AgentLaunch` describes a new agent session request.
- `AgentSession` stores the agent id, kind, feature, state, messages, and modified files.
- `ChatMessage` and `MessageRole` model conversation history.
- `PromptPolicy` injects project instructions and worktree access rules into agent prompts.
- `ReadPermission` selects whether prompts require orchestrator-mediated reads or allow direct
  filesystem reads while keeping writes mediated by patches.
- `AgentError` is the shared error type for launch, send, and prompt policy failures.

`src/agent.rs` also re-exports `AgentBackend`, `AgentStreamEvent`, and `AgentShutdownHandle` from
`src/agent_runtime.rs`, so callers can import all provider-neutral agent interfaces from
`work_leaf::agent`.

## Agent Runtime Interface

`src/agent_runtime.rs` owns the provider-neutral backend contract:

- `AgentBackend::launch` starts an agent session from an `AgentLaunch`.
- `AgentBackend::send` sends a prompt to an existing agent session.
- `AgentBackend::launch_streaming` and `AgentBackend::send_streaming` provide real-time output to a
  sink of `AgentStreamEvent` values. Their default implementations call the non-streaming methods.
- `AgentBackend::shutdown_handle` returns an `AgentShutdownHandle` for terminating active provider
  processes.
- `AgentStreamEvent` carries status text, streamed agent messages, and streamed errors.
- `AgentShutdownHandle::shutdown` terminates registered processes, waits briefly, and then kills
  remaining processes.

The runtime also contains provider-process support that is not part of the public provider API:
registered child processes are tracked in an internal registry; Unix children run in their own
process group; Linux children receive a parent-death signal; shutdown first sends terminate and then
kill to remaining processes. This behavior is used by provider implementations that spawn real
processes.

New agent providers implement `AgentBackend`; they do not add provider logic to `src/codex.rs`.
Providers that need real-time UI output should override the streaming methods. Providers implemented
inside this crate can use the crate-private process registration helpers for shared shutdown.
Providers outside this crate can implement launch, send, and streaming through the public trait; a
public lifecycle extension is required before external child processes can participate in the shared
`AgentShutdownHandle` registry.

## Codex Provider

`src/codex.rs` contains the Codex-specific implementation of the neutral agent runtime interface:

- `SandboxMode` and `CodexCommandConfig` define Codex CLI invocation settings.
- `CodexInvocation` records the command, arguments, and prompt used for an invocation.
- `CodexBackend` stores Codex session history and implements `AgentBackend`.
- `CodexBackend::build_launch_invocation` and `CodexBackend::build_send_invocation` construct
  `codex exec` and `codex exec resume` calls.
- `CodexBackend::record_launch_reply`, `record_launch_output`, and `session` maintain in-memory
  session state.
- `CodexBackend` parses Codex `--json` event lines from stdout to capture `thread.started`
  identifiers for resume and to convert agent message, error, and status events into
  `AgentStreamEvent` values. The parser accepts standard JSON whitespace around field separators
  while preserving string contents.

`CodexBackend` is a provider implementation, not the owner of the generic agent contract. Callers
that need provider-neutral behavior import `AgentBackend` from `work_leaf::agent` or from the
top-level re-export, not from `work_leaf::codex`.

## Command Chat

`src/cli.rs::CommandChat<B>` is the command orchestration surface shared by the CLI, controller, and
tests. It is generic over `B: AgentBackend`.

`CommandChat` owns:

- project root and prompt policy,
- active `AgentProfile`,
- agent sessions and generated agent ids,
- orchestrator, patch, review, and linearizer coordination,
- transcript output for command-mode and UI consumers,
- backend shutdown through `AgentShutdownHandle`.

The primary public methods are:

- `CommandChat::new` for constructing command chat state with a backend.
- `CommandChat::with_agent_profile` for selecting a non-default provider profile.
- `CommandChat::handle_line` for processing command lines such as `new`, `chat`, `review`, and
  `linearize`.
- `CommandChat::prepare_agent_launch`, `launch_prepared_agent_streaming`, and
  `launch_prepared_agent_streaming_with_ids` for UI-driven launch flows.
- `CommandChat::send_to_agent`, `send_to_agent_streaming`, and
  `send_to_agent_streaming_with_ids` for UI-driven message flows.
- `CommandChat::shutdown_agents` and `shutdown_handle` for lifecycle cleanup.

`CommandChat` uses the active `AgentProfile` when launching user agents, reviewers, and linearizers.
Workflow code must not hard-code `AgentKind::Codex` when the active profile supplies the provider.

## UI-Neutral Controller

`src/workspace.rs::WorkLeafController<B>` is the preferred API for frontends. It owns UI-neutral
application state and hides worker management from frontend adapters.

The controller owns:

- session selection and session snapshots,
- per-session loading state,
- LLM-generated chat titles through hidden `title-<agent-id>` backend launches, with
  `src/chat_title.rs::ChatTitleAgent` tracking first-prompt naming state,
- command transcripts,
- background launch/send/review workers,
- stream routing from `AgentStreamEvent` into the selected session,
- review startup, automatic per-patch-agent review routing, reviewer-session creation, and
  reviewed-commit bookkeeping,
- shutdown propagation to running agents.

When an agent worker finishes, the controller records the agent output and clears that session's
loading state. A user-agent response becomes review-ready only when the orchestrator transcript shows
an applied patch from that agent and the agent emits `@work-leaf done`. Successful patch application
returns a continuation prompt to the patch agent when the agent has not reported done, so the agent can
run repository-required checks through locked command directives, provide follow-up patches, or signal
review readiness. Repository build, test, format, and required-check commands run only through
agent-emitted orchestrator directives that name the command and the write-lock paths the command may
touch. Locked command runs have a five-minute default timeout, after which the command is terminated,
locks are released, and a longer run requires user authorization. `PromptPolicy` injects project
instruction files into agent prompts, and the active backend agent is responsible for choosing and
requesting the repository checks required by those instructions before reporting work done.
Tracked file changes produced by locked commands remain pending for that patch agent until the agent
commits them through the patch protocol or reverts them. Pending command changes block
`@work-leaf done`, and the orchestrator returns the tracked diff so the agent can submit the command
output as a provisional patch when it belongs in the final work.

The command transcript is also the conversation history for the persistent `command-agent`. That
system agent interprets chat sent to the Work Leaf command surface. It recognizes literal command
lines and common natural-language requests for help, review, linearization, quitting, and launching
one or more user agents. Multi-agent launch requests dispatch `new [prompt...]` once per requested
agent through the same controller paths used by command-mode input.

Frontend code should use these methods:

- `WorkLeafController::new` to wrap a `CommandChat<B>`.
- `snapshot` to read renderable state.
- `drain_events` to consume UI-neutral events.
- `execute_command_line` to run command-mode input.
- `create_agent` to reserve, select, and launch an agent session from a prompt.
- `send_command_agent_message` to route chat from the Work Leaf command surface to `command-agent`.
- `send_message` to send a prompt to one session while other sessions may still be busy.
- `start_review` to create or resume reviewer sessions for explicit history-wide review and stream
  reviewer output.
- `is_busy`, `wait_for_idle`, and `wait_for_session_line` for tests and event loops.
- `shutdown` to terminate active backend processes.

The controller exposes renderable state through:

- `WorkLeafSnapshot`, which contains the command transcript and sessions.
- `WorkLeafSession`, which contains agent id, kind, feature/title, transcript lines, and loading
  state.
- `WorkLeafEvent`, which reports session creation, session updates, streamed lines, selection
  changes, transcript lines, and quit requests.
- `WorkLeafLoading`, which distinguishes launch and waiting-for-reply states.

New UIs should consume `WorkLeafController` and these DTOs. They should not duplicate worker
spawning, session naming, review lookup, loading bookkeeping, or orchestrator event routing.

## Terminal UI

The terminal frontend is an adapter over the UI-neutral controller.

`src/terminal_app.rs::TerminalApp<B>` translates raw terminal bytes and modal editing state into
controller commands, applies `WorkLeafEvent` values to `TerminalUi`, and renders controller
snapshots. It owns terminal event-loop concerns such as insert mode, prompt mode, `Ctrl-W`
navigation, SGR mouse clicks, SGR mouse wheel scrolling of the right pane, bytewise input parsing,
rendering invalidation, and polling background workers. Insert mode sends chat text to the selected
agent session, or to `command-agent` when the Work Leaf command
surface is selected. Bracketed-paste newlines and Shift+Enter are chat prompt line breaks. A plain
Enter submits the buffered chat text.

The terminal app maps a session to a left-pane `READY` marker when the controller exposes no loading
state for that session. `TerminalUi` queues one terminal bell when a chat transitions into the ready
state and renders ready rows in reverse video so they remain highlighted until the chat becomes busy
again.

`src/ui.rs::TerminalUi` owns terminal-specific presentation state:

- `UiMode`, `PaneFocus`, `UiSurface`, `UiKey`, and `UiAction` model terminal interactions.
- `AgentListEntry` is the terminal left-pane representation of an agent row.
- `TerminalLayout` computes pane geometry.
- `TerminalUi` renders left/right panes, prompts, cursor placement, command-interface selection, and
  terminal navigation actions. The right pane keeps the chat prompt visible while scroll offsets
  reveal earlier transcript rows.

`src/ui_harness.rs::UiHarness` is the test harness for terminal behavior. It exercises the same
`TerminalUi` frame path used by the interactive example. UI tests should drive
`UiHarness::handle_byte` or `UiHarness::handle_bytes` rather than duplicating terminal input logic.

A web UI, desktop UI, or non-terminal integration should not depend on `TerminalApp` or
`TerminalUi`; it should depend on `WorkLeafController` and the DTOs in `src/workspace.rs`.

## Core Workflow Modules

`src/orchestrator.rs::AgentOrchestrator<B>` parses and executes `@work-leaf` directives emitted by
agents. It uses `FileLockTable` for file reads and command write locks, `CommandWritePolicy` for
command classification, `PatchCoordinator` for patch requests, and the active `AgentBackend` for
routed follow-up messages. Its public output is `OrchestratorEvent`.

`src/locks.rs::FileLockTable` owns root-scoped path normalization and read/write locking.
`FileSnapshot` carries file read results. `CommandWritePolicy` and `CommandWriteIntent` provide
heuristic read-only/write-intent classification for commands when an agent is unsure. Agent-requested
command runs execute in the project root while `FileLockTable` holds write locks for the normalized
lock paths supplied by the agent. File paths are normalized relative to the project root and cannot
escape that root.

`src/patch.rs::GitPatcher` validates and applies unified diffs under write locks and creates
metadata commits for accepted patches. It also accepts a matching already-applied diff when a locked
command has produced the tracked working-tree change, so the command output can be saved as the
agent's provisional patch. `PatchCoordinator<B>` connects patch conflicts and malformed patch
diagnostics back to the active agent backend. `PatchRequest`, `PatchOutcome`, and `PatchError` are
the public patch workflow types.

`src/review.rs::GitHistory` reads latest agent commits from repository history.
`ReviewCoordinator<B>` launches reviewer agents against those commits and loops until the reviewer
reports no findings or the configured maximum round count is reached. `CommandChat` resolves
reviewer `@work-leaf` directives, such as file reads, before interpreting reviewer output as
findings. `CommandChat` and `WorkLeafController` keep a stable `review-<agent-id>` reviewer identity
for each patch agent and skip latest agent commits that have already completed review. `AgentCommit`,
`ReviewResult`, and `ReviewError` are the public review workflow types.
`WorkLeafController` scopes automatic review after a patch agent reports done to the patch agent that
produced the provisional commit; explicit review commands use the history-wide latest-commit lookup.

`src/linearize.rs::LinearizePlanner<B>` prepares linearization questions and launches a linearizer
agent with decisions, groups, and required tests. `LinearizeAction`, `LinearizeGroup`,
`LinearizePlan`, `LinearizeQuestion`, `LinearizeHandoff`, and `LinearizeError` are the public
linearization workflow types. `CommandChat` and `WorkLeafController` launch linearization from the
commits recorded as reviewed in the current command-chat or controller instance; unrelated historical
agent metadata commits are outside the linearizer scope unless the user explicitly reviews or adds
them in that session.

`src/instructions.rs` is crate-private. It loads project instruction files used by `PromptPolicy`
for agent launch prompts.

`src/chat_title.rs` is crate-private. It builds the prompt used for hidden chat-title backend
launches, sanitizes title replies to lowercase hyphenated names capped at 80 characters, provides a
first-prompt fallback, and tracks which sessions have already requested a generated title.

## Extension Rules

New UI support follows this path:

1. Construct a `CommandChat<B>` with the desired backend.
2. Wrap it in `WorkLeafController<B>`.
3. Render from `WorkLeafSnapshot` and `WorkLeafSession`.
4. Drive user actions through controller methods.
5. Consume `WorkLeafEvent` values from `drain_events`.

New agent-provider support follows this path:

1. Define an `AgentProfile` with `AgentKind::External`.
2. Implement `AgentBackend` for the provider.
3. Override streaming methods when the provider can emit real-time output.
4. Return an `AgentShutdownHandle` when the provider owns child processes.
5. Pass the profile through `CommandChat::with_agent_profile`.
6. Use `WorkLeafController` or `CommandChat` without modifying terminal UI code.

New core workflow behavior belongs in the workflow module that owns the behavior. UI adapters should
only translate user input into controller calls and render controller snapshots. Agent providers
should only implement launch, send, streaming, and shutdown behavior.

## API and Architecture Change Policy

A breaking public API change requires human authorization before implementation. Public API includes
top-level re-exports in `src/lib.rs`, public items in public modules, and the documented integration
surfaces for UIs, agent providers, command orchestration, and core workflows. In Rust, removing or
renaming public items, changing public method signatures, adding required trait methods, changing
public enum matching behavior, changing public struct construction behavior, or changing documented
semantics can be breaking.

A non-breaking public API extension does not require human authorization, but this document must
describe the resulting public surface whenever the extension affects UI integration, agent-provider
integration, command orchestration, or core workflow integration.

An architecture change requires human authorization before implementation when the requested work can
only be completed by changing documented ownership, dependency direction, extension boundaries, or
integration paths. After authorization, this document must describe the resulting architecture in the
same patch as the code change.

When compatibility is unclear, treat the change as breaking until the caller confirms otherwise.

## Validation Expectations

Provider-interface changes should have tests that prove an external provider can implement
`AgentBackend` without depending on Codex-specific code. `tests/agent_provider_interface.rs` covers
that contract.

Controller and UI behavior should use `WorkLeafController`, `TerminalApp`, and `UiHarness` tests
instead of duplicating internal terminal or worker logic. Terminal UI behavior is covered through
`tests/ui_harness.rs`, `tests/terminal_ui.rs`, and `tests/terminal_app.rs`.

Core workflow changes should test the owning module and the integration path that consumes it. The
existing test suites under `tests/orchestrator_protocol.rs`, `tests/patching.rs`,
`tests/reviews.rs`, `tests/linearize.rs`, and `tests/workspace.rs` provide the current coverage
shape.
