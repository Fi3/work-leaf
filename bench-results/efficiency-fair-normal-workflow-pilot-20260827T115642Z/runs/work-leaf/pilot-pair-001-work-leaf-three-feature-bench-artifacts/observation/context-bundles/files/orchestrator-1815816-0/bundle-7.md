# Work Leaf Context Bundle

This file contains orchestrator-mediated read output. Use it as read-only context; submit project changes through `@work-leaf edit`.

----- BEGIN FILE docs/architecture.md -----
digest: fnv64:eb3148d3577baccb; bytes:26239

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
- `http_controller` defines the localhost HTTP transport used by the daemon and CLI process
  boundary.
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
- Localhost controller transport: `HttpControllerClient`, `HttpControllerServer`,
  `OrchestratorHttpError`.
- Core workflows: `AgentOrchestrator`, `GitPatcher`, `PatchCoordinator`, `GitHistory`,
  `ReviewCoordinator`, `LinearizePlanner`, `FileLockTable`.
- Terminal UI: `TerminalApp`, `RemoteTerminalApp`, `TerminalUi`, `UiHarness`, `UiAction`, `UiKey`,
  `UiMode`, `UiSurface`, `PaneFocus`, `AgentListEntry`.

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

The package has two binary targets. `src/bin/work-leaf-orchestrator.rs` calls
`work_leaf::run_orchestrator_from_env()`, creates the Codex backend and `CommandChat`, wraps them in
`WorkLeafController<CodexBackend>`, and exposes that controller through
`src/http_controller.rs::HttpControllerServer` on a localhost HTTP address. The daemon prints a
machine-readable `WORK_LEAF_ORCHESTRATOR_URL=http://...` startup line after binding.

`src/bin/work-leaf.rs` calls `work_leaf::run_cli_from_env()`. The CLI connects to
`WORK_LEAF_ORCHESTRATOR_URL` when that environment variable is present; otherwise it starts the
sibling `work-leaf-orchestrator` binary on `127.0.0.1:0`, reads the printed URL, and connects to that
daemon. A CLI-managed daemon receives `WORK_LEAF_PARENT_PID`, does not inherit the terminal file
descriptors, and exits when that parent process is gone. The terminal frontend renders through
`src/terminal_app.rs::RemoteTerminalApp`, which drives `src/http_controller.rs::HttpControllerClient`.
The in-process `src/terminal_app.rs::TerminalApp<B>` remains the local controller adapter used by
tests and embedders that construct a `CommandChat<B>` directly.

The project-root `start` script builds both binary targets in release mode, starts
`work-leaf-orchestrator` on `127.0.0.1:7878` by default, exports the printed URL for `work-leaf`,
and terminates the daemon when the CLI process exits. `WORK_LEAF_START_LISTEN` selects a different
listen address when a caller needs an explicit override; the script does not fall back to another
port when the requested address is unavailable.

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
  `codex exec` and `codex exec resume` calls. Launch invocations include the injected
  `PromptPolicy`; resumed invocations for known sessions pass only the follow-up message because the
  Codex thread already contains the launch-time policy and repository instructions.
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

Review bookkeeping has three scopes. The controller records a launch-time review baseline for each
patch agent, tracks the latest reviewed hash for that patch agent so the same agent head is not
reviewed twice, and asks reviewers to inspect every provisional commit from the active baseline
through the latest patch-agent commit. `CommandChat` also keeps the ordered exact review targets that
completed review during the active instance, and those targets form the linearizer handoff. This lets
one patch-agent session complete more than one reviewed patch without a later hash replacing earlier
reviewed work in the linearizer prompt.

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

`WorkLeafEvent` uses append-oriented transcript events for efficient remote frontends. `AgentAdded`
provides the initial session snapshot, `AgentLineAppended` carries one new session line, and
`CommandTranscriptLine` carries one new command transcript line. `AgentStatusUpdated` carries
session metadata and loading state without re-sending the session transcript. `AgentUpdated` remains
part of the DTO surface for full-session replacement when an integration needs it. Session line
appends and status changes are not paired with full-session replacement events, so remote frontends
can update long transcripts without re-receiving the full transcript text.

New UIs should consume `WorkLeafController` and these DTOs. They should not duplicate worker
spawning, session naming, review lookup, loading bookkeeping, or orchestrator event routing.

## Localhost HTTP Controller

`src/http_controller.rs::HttpControllerServer` is a transport adapter over `WorkLeafController`. It
owns no workflow behavior; each HTTP route delegates to the corresponding controller method or DTO:

- `GET /snapshot` returns `WorkLeafSnapshot`.
- `POST /events/drain` returns pending `WorkLeafEvent` values after polling workers when needed.
- `GET /busy` returns the controller busy state.
- `POST /command` calls `WorkLeafController::execute_command_line`.
- `POST /command-agent` calls `WorkLeafController::send_command_agent_message`.
- `POST /agent/message` calls `WorkLeafController::send_message`.
- `POST /agent/interrupt` calls `WorkLeafController::interrupt_agent`.
- `POST /transcript` calls `WorkLeafController::push_transcript_line`.
- `POST /loading-text` calls `WorkLeafController::loading_text`.
- `POST /shutdown` calls `WorkLeafController::shutdown` and stops the daemon loop.

`src/http_controller.rs::HttpControllerClient` is the matching blocking localhost client. It
serializes and deserializes the same workspace DTOs used by in-process frontends. `AgentId`
deserialization uses `src/agent.rs::AgentId::new`, so HTTP payloads preserve the same identifier
validation as local controller calls.

## Terminal UI

The terminal frontend is an adapter over the UI-neutral controller surface.

`src/terminal_app.rs::TerminalApp<B>` translates raw terminal bytes and modal editing state into
direct `WorkLeafController<B>` calls for in-process use. `src/terminal_app.rs::RemoteTerminalApp`
uses the same terminal state machine with `HttpControllerClient` for the CLI/daemon process split.
Both adapters keep a local render snapshot, apply `WorkLeafEvent` values to that cache and to
`TerminalUi`, and render from the cache rather than fetching a full controller snapshot for every
frame. They own terminal event-loop concerns such as insert mode, prompt mode, `Ctrl-W` navigation,
SGR mouse clicks, SGR mouse wheel scrolling of the right pane, chunked terminal input parsing,
rendering invalidation, and polling background workers. Insert mode sends chat text to the selected
agent session, or to `command-agent` when the Work Leaf command surface is selected. Bracketed-paste
newlines and Shift+Enter are chat prompt line breaks. A plain Enter submits the buffered chat text.
When an agent chat is selected in command mode, `/` focuses the chat, seeds the chat buffer with
`/`, and enters insert mode so `/status`-style input submits through the same selected-agent chat
path.

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

`src/review.rs::GitHistory` reads latest agent commits from repository history, builds cumulative
review targets for a patch agent since a launch or reviewed baseline, and resolves agent metadata
commits by exact hash. `ReviewCoordinator<B>` launches reviewer agents against those review targets
and loops until the reviewer reports no findings or the configured maximum round count is reached.
`CommandChat` resolves reviewer `@work-leaf` directives, such as file reads, before interpreting
reviewer output as findings. `CommandChat` and `WorkLeafController` keep a stable
`review-<agent-id>` reviewer identity for each patch agent and skip latest agent heads that have
already completed review. `AgentCommit`, `ReviewResult`, and `ReviewError` are the public review
workflow types. `WorkLeafController` scopes automatic review after a patch agent reports done to the
patch agent that produced the provisional commit; explicit review commands use the history-wide
review target lookup.

`src/linearize.rs::LinearizePlanner<B>` prepares linearization questions and launches a linearizer
agent with decisions, groups, and required tests. `LinearizeAction`, `LinearizeGroup`,
`LinearizePlan`, `LinearizeQuestion`, `LinearizeHandoff`, and `LinearizeError` are the public
linearization workflow types. `CommandChat` and `WorkLeafController` launch linearization from the
exact commits recorded as reviewed in the current command-chat or controller instance; unrelated
historical agent metadata commits are outside the linearizer scope unless the user explicitly reviews
or adds them in that session. When one patch-agent id completes multiple reviewed commits in one
active instance, each reviewed hash is listed independently for the linearizer.

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

Out-of-process UI support uses `HttpControllerClient` against a running `HttpControllerServer` and
the same snapshot, session, and event DTOs. The HTTP transport remains an adapter over
`WorkLeafController`; new workflow behavior still belongs in the owning workflow or controller
module.

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

The CLI/daemon transport is covered by `tests/http_orchestrator.rs`, which starts the real
`work-leaf-orchestrator` binary and drives it through `HttpControllerClient`. The release launcher is
covered by `tests/start_script.rs`, which runs the root `start` script through a pseudo-terminal with
prebuilt test binaries.

Core workflow changes should test the owning module and the integration path that consumes it. The
existing test suites under `tests/orchestrator_protocol.rs`, `tests/patching.rs`,
`tests/reviews.rs`, `tests/linearize.rs`, and `tests/workspace.rs` provide the current coverage
shape.
----- END FILE docs/architecture.md -----

----- BEGIN FILE src/terminal_app.rs -----
digest: fnv64:61c6b0f256b26a91; bytes:43335

use std::thread;
use std::time::{Duration, Instant};

use rustyline::line_buffer::{ChangeListener, DeleteListener, Direction, LineBuffer};

use crate::agent::{AgentBackend, AgentId};
use crate::cli::{CommandChat, terminal_right_content, ui_action_text};
use crate::http_controller::HttpControllerClient;
use crate::ui::{AgentListEntry, PaneFocus, TerminalUi, UiKey, UiMode};
use crate::workspace::{
    WorkLeafController, WorkLeafEvent, WorkLeafLoading, WorkLeafSession, WorkLeafSnapshot,
};

#[derive(Debug)]
pub struct TerminalApp<B>
where
    B: AgentBackend + Clone + Send + 'static,
{
    inner: TerminalAppCore<LocalTerminalController<B>>,
}

impl<B> TerminalApp<B>
where
    B: AgentBackend + Clone + Send + 'static,
{
    pub fn new(chat: CommandChat<B>, width: u16, height: u16) -> Self {
        Self {
            inner: TerminalAppCore::new(
                LocalTerminalController {
                    controller: WorkLeafController::new(chat),
                },
                width,
                height,
            ),
        }
    }

    pub fn into_chat(mut self) -> CommandChat<B> {
        self.wait_for_idle(Duration::from_secs(5));
        self.inner.controller.controller.into_chat()
    }

    pub fn ui(&self) -> &TerminalUi {
        self.inner.ui()
    }

    pub fn transcript(&self) -> &[String] {
        self.inner.controller.controller.transcript()
    }

    pub fn is_quit(&self) -> bool {
        self.inner.is_quit()
    }

    pub fn is_busy(&mut self) -> bool {
        self.inner.is_busy()
    }

    pub fn needs_render(&self) -> bool {
        self.inner.needs_render()
    }

    pub fn mark_rendered(&mut self) {
        self.inner.mark_rendered();
    }

    pub fn tick(&mut self) {
        self.inner.tick();
    }

    pub fn wait_for_idle(&mut self, timeout: Duration) -> bool {
        self.inner.wait_for_idle(timeout)
    }

    pub fn wait_for_frame_contains(&mut self, needle: &str, timeout: Duration) -> bool {
        self.inner.wait_for_frame_contains(needle, timeout)
    }

    pub fn handle_bytes(&mut self, bytes: &[u8]) -> bool {
        self.inner.handle_bytes(bytes)
    }

    pub fn handle_byte(&mut self, byte: u8) -> bool {
        self.inner.handle_byte(byte)
    }

    pub(crate) fn handle_terminal_bytes(&mut self, bytes: &[u8]) -> bool {
        self.inner.handle_terminal_bytes(bytes)
    }

    pub(crate) fn finish_pending_terminal_input(&mut self) {
        self.inner.finish_pending_terminal_input();
    }

    pub fn render_frame(&self) -> String {
        self.inner.render_frame()
    }

    pub fn poll_worker(&mut self) {
        self.inner.poll_worker();
    }

    #[cfg(test)]
    fn clear_agent_loading(&mut self, agent_id: &AgentId) {
        self.inner.clear_agent_loading(agent_id);
    }

    #[cfg(test)]
    fn set_agent_loading(&mut self, agent_id: &AgentId, loading: Option<LoadingKind>) {
        self.inner.set_agent_loading(agent_id, loading);
    }
}

#[derive(Debug)]
pub struct RemoteTerminalApp {
    inner: TerminalAppCore<HttpControllerClient>,
}

impl RemoteTerminalApp {
    pub fn new(client: HttpControllerClient, width: u16, height: u16) -> Self {
        Self {
            inner: TerminalAppCore::new(client, width, height),
        }
    }

    pub fn ui(&self) -> &TerminalUi {
        self.inner.ui()
    }

    pub fn is_quit(&self) -> bool {
        self.inner.is_quit()
    }

    pub fn is_busy(&mut self) -> bool {
        self.inner.is_busy()
    }

    pub fn needs_render(&self) -> bool {
        self.inner.needs_render()
    }

    pub fn mark_rendered(&mut self) {
        self.inner.mark_rendered();
    }

    pub fn tick(&mut self) {
        self.inner.tick();
    }

    pub fn wait_for_idle(&mut self, timeout: Duration) -> bool {
        self.inner.wait_for_idle(timeout)
    }

    pub fn wait_for_frame_contains(&mut self, needle: &str, timeout: Duration) -> bool {
        self.inner.wait_for_frame_contains(needle, timeout)
    }

    pub fn handle_bytes(&mut self, bytes: &[u8]) -> bool {
        self.inner.handle_bytes(bytes)
    }

    pub fn handle_byte(&mut self, byte: u8) -> bool {
        self.inner.handle_byte(byte)
    }

    pub(crate) fn handle_terminal_bytes(&mut self, bytes: &[u8]) -> bool {
        self.inner.handle_terminal_bytes(bytes)
    }

    pub(crate) fn finish_pending_terminal_input(&mut self) {
        self.inner.finish_pending_terminal_input();
    }

    pub fn render_frame(&self) -> String {
        self.inner.render_frame()
    }

    pub fn poll_worker(&mut self) {
        self.inner.poll_worker();
    }
}

#[derive(Debug)]
struct LocalTerminalController<B>
where
    B: AgentBackend + Clone + Send + 'static,
{
    controller: WorkLeafController<B>,
}

trait TerminalController {
    fn snapshot(&self) -> crate::WorkLeafSnapshot;
    fn drain_events(&mut self) -> Vec<WorkLeafEvent>;
    fn execute_command_line(&mut self, line: &str);
    fn send_command_agent_message(&mut self, message: &str);
    fn send_message(&mut self, agent_id: &AgentId, message: &str);
    fn interrupt_agent(&mut self, agent_id: &AgentId);
    fn push_transcript_line(&mut self, line: String);
    fn is_busy(&mut self) -> bool;
    fn loading_text(&self, loading: WorkLeafLoading) -> String;
    fn shutdown(&mut self);
}

impl<B> TerminalController for LocalTerminalController<B>
where
    B: AgentBackend + Clone + Send + 'static,
{
    fn snapshot(&self) -> crate::WorkLeafSnapshot {
        self.controller.snapshot()
    }

    fn drain_events(&mut self) -> Vec<WorkLeafEvent> {
        self.controller.drain_events()
    }

    fn execute_command_line(&mut self, line: &str) {
        self.controller.execute_command_line(line);
    }

    fn send_command_agent_message(&mut self, message: &str) {
        self.controller.send_command_agent_message(message);
    }

    fn send_message(&mut self, agent_id: &AgentId, message: &str) {
        let _ = self.controller.send_message(agent_id, message);
    }

    fn interrupt_agent(&mut self, agent_id: &AgentId) {
        self.controller.interrupt_agent(agent_id);
    }

    fn push_transcript_line(&mut self, line: String) {
        self.controller.push_transcript_line(line);
    }

    fn is_busy(&mut self) -> bool {
        self.controller.is_busy()
    }

    fn loading_text(&self, loading: WorkLeafLoading) -> String {
        self.controller.loading_text(loading)
    }

    fn shutdown(&mut self) {
        self.controller.shutdown();
    }
}

impl TerminalController for HttpControllerClient {
    fn snapshot(&self) -> crate::WorkLeafSnapshot {
        self.snapshot()
            .unwrap_or_else(|error| crate::WorkLeafSnapshot {
                command_transcript: vec![format!("error: {error}")],
                sessions: Vec::new(),
            })
    }

    fn drain_events(&mut self) -> Vec<WorkLeafEvent> {
        HttpControllerClient::drain_events(self).unwrap_or_default()
    }

    fn execute_command_line(&mut self, line: &str) {
        let _ = HttpControllerClient::execute_command_line(self, line);
    }

    fn send_command_agent_message(&mut self, message: &str) {
        let _ = HttpControllerClient::send_command_agent_message(self, message);
    }

    fn send_message(&mut self, agent_id: &AgentId, message: &str) {
        let _ = HttpControllerClient::send_message(self, agent_id, message);
    }

    fn interrupt_agent(&mut self, agent_id: &AgentId) {
        let _ = HttpControllerClient::interrupt_agent(self, agent_id);
    }

    fn push_transcript_line(&mut self, line: String) {
        let _ = HttpControllerClient::push_transcript_line(self, line);
    }

    fn is_busy(&mut self) -> bool {
        HttpControllerClient::is_busy(self).unwrap_or(false)
    }

    fn loading_text(&self, loading: WorkLeafLoading) -> String {
        HttpControllerClient::loading_text(self, loading)
            .unwrap_or_else(|_| "Waiting for agent".to_string())
    }

    fn shutdown(&mut self) {
        let _ = HttpControllerClient::shutdown(self);
    }
}

#[derive(Debug)]
struct TerminalAppCore<C>
where
    C: TerminalController,
{
    controller: C,
    ui: TerminalUi,
    prompt_buffer: PromptLine,
    prompt_history: Vec<String>,
    prompt_history_index: Option<usize>,
    prompt_history_draft: Option<String>,
    chat_buffer: PromptLine,
    chat_history: Vec<String>,
    chat_history_index: Option<usize>,
    chat_history_draft: Option<String>,
    escape_sequence: Option<PendingEscapeSequence>,
    paste_mode: bool,
    skip_next_paste_lf: bool,
    spinner: usize,
    snapshot: WorkLeafSnapshot,
    loading_text: [(WorkLeafLoading, String); 2],
    dirty: bool,
    quit: bool,
}

impl<C> TerminalAppCore<C>
where
    C: TerminalController,
{
    fn new(controller: C, width: u16, height: u16) -> Self {
        let snapshot = controller.snapshot();
        let loading_text = [
            (
                WorkLeafLoading::Launching,
                controller.loading_text(WorkLeafLoading::Launching),
            ),
            (
                WorkLeafLoading::WaitingForReply,
                controller.loading_text(WorkLeafLoading::WaitingForReply),
            ),
        ];
        let mut app = Self {
            controller,
            ui: TerminalUi::new(width, height),
            prompt_buffer: PromptLine::new(),
            prompt_history: Vec::new(),
            prompt_history_index: None,
            prompt_history_draft: None,
            chat_buffer: PromptLine::new(),
            chat_history: Vec::new(),
            chat_history_index: None,
            chat_history_draft: None,
            escape_sequence: None,
            paste_mode: false,
            skip_next_paste_lf: false,
            spinner: 0,
            snapshot,
            loading_text,
            dirty: true,
            quit: false,
        };
        let sessions = app.snapshot.sessions.clone();
        for session in sessions {
            app.apply_session_to_ui(&session);
        }
        app
    }

    fn ui(&self) -> &TerminalUi {
        &self.ui
    }

    fn is_quit(&self) -> bool {
        self.quit
    }

    fn is_busy(&mut self) -> bool {
        let busy = self.controller.is_busy();
        self.apply_controller_events();
        busy
    }

    fn needs_render(&self) -> bool {
        self.dirty || self.ui.has_status_notice()
    }

    fn mark_rendered(&mut self) {
        self.dirty = false;
        self.ui.clear_expired_status_notice();
    }

    fn tick(&mut self) {
        let busy = self.controller.is_busy();
        self.apply_controller_events();
        if busy {
            self.spinner = (self.spinner + 1) % SPINNER.len();
            self.dirty = true;
        }
    }

    fn wait_for_idle(&mut self, timeout: Duration) -> bool {
        let start = Instant::now();
        while start.elapsed() < timeout {
            self.apply_controller_events();
            if !self.controller.is_busy() {
                return true;
            }
            thread::sleep(Duration::from_millis(10));
        }
        self.apply_controller_events();
        !self.controller.is_busy()
    }

    fn wait_for_frame_contains(&mut self, needle: &str, timeout: Duration) -> bool {
        let start = Instant::now();
        while start.elapsed() < timeout {
            self.apply_controller_events();
            if self.render_frame().contains(needle) {
                return true;
            }
            thread::sleep(Duration::from_millis(10));
        }
        self.apply_controller_events();
        self.render_frame().contains(needle)
    }

    fn handle_bytes(&mut self, bytes: &[u8]) -> bool {
        if !self.handle_terminal_bytes(bytes) {
            return false;
        }
        self.finish_pending_terminal_input();
        !self.quit
    }

    fn handle_terminal_bytes(&mut self, bytes: &[u8]) -> bool {
        self.apply_controller_events();
        for byte in bytes {
            if !self.handle_byte_without_poll(*byte) {
                self.apply_controller_events();
                return false;
            }
        }
        self.apply_controller_events();
        !self.quit
    }

    pub fn handle_byte(&mut self, byte: u8) -> bool {
        self.apply_controller_events();
        let keep_running = self.handle_byte_without_poll(byte);
        self.apply_controller_events();
        keep_running && !self.quit
    }

    fn finish_pending_terminal_input(&mut self) {
        self.finish_pending_escape_sequence();
        self.apply_controller_events();
    }

    fn handle_byte_without_poll(&mut self, byte: u8) -> bool {
        if self.quit {
            return false;
        }

        if self.continue_escape_sequence(byte) {
            return !self.quit;
        }

        if byte == 27 {
            let defer_escape = self.defer_escape_key();
            self.escape_sequence = Some(PendingEscapeSequence {
                bytes: Vec::new(),
                mode_before: self.ui.mode(),
                escape_dispatched: !defer_escape,
            });
            if !defer_escape {
                self.handle_input(TerminalAppInput::Key(UiKey::Esc));
            }
            return !self.quit;
        }

        let Some(input) = self.input_from_byte(byte) else {
            return true;
        };
        self.handle_input(input);
        !self.quit
    }

    pub fn render_frame(&self) -> String {
        let right_content = self.right_content();
        let right_cursor_column = (self.ui.focus() == PaneFocus::Right
            && self.ui.mode() != UiMode::Prompt)
            .then_some(6 + self.chat_buffer.cursor_char_count());
        self.ui.render_screen_with_cursors(
            &right_content,
            self.prompt_buffer.as_str(),
            self.prompt_buffer.cursor(),
            right_cursor_column,
        )
    }

    pub fn poll_worker(&mut self) {
        self.apply_controller_events();
    }

    fn handle_input(&mut self, input: TerminalAppInput) {
        match input {
            TerminalAppInput::Quit => {
                self.request_quit();
            }
            TerminalAppInput::Interrupt => {
                self.ui.show_ctrl_c_exit_notice();
                if self.ui.focus() == PaneFocus::Right
                    && let Some(agent_id) = self.ui.selected_agent().cloned()
                {
                    self.controller.interrupt_agent(&agent_id);
                    self.apply_controller_events();
                }
                self.dirty = true;
            }
            TerminalAppInput::Backspace if self.ui.mode() == UiMode::Prompt => {
                self.prompt_buffer.backspace();
                self.prompt_history_index = None;
                self.prompt_history_draft = None;
                self.dirty = true;
            }
            TerminalAppInput::Backspace if self.ui.mode() == UiMode::Insert => {
                self.chat_buffer.backspace();
                self.chat_history_index = None;
                self.chat_history_draft = None;
                self.dirty = true;
            }
            TerminalAppInput::Enter if self.ui.mode() == UiMode::Prompt => {
                let line = self.prompt_buffer.trimmed_string();
                self.prompt_buffer.clear();
                self.ui.handle_key(UiKey::Esc);
                if !line.is_empty() {
                    self.prompt_history.push(line.clone());
                    self.prompt_history_index = None;
                    self.prompt_history_draft = None;
                    self.handle_command_line(&line);
                } else {
                    self.prompt_history_index = None;
                    self.prompt_history_draft = None;
                }
                self.dirty = true;
            }
            TerminalAppInput::Enter if self.ui.mode() == UiMode::Insert => {
                self.send_chat_buffer();
            }
            TerminalAppInput::Char('/') if self.should_start_agent_slash_command() => {
                self.start_agent_slash_command();
                self.dirty = true;
            }
            TerminalAppInput::LineBreak if self.ui.mode() == UiMode::Insert => {
                self.chat_buffer.push('\n');
                self.chat_history_index = None;
                self.chat_history_draft = None;
                self.dirty = true;
            }
            TerminalAppInput::PasteStart => {
                self.paste_mode = true;
                self.skip_next_paste_lf = false;
            }
            TerminalAppInput::PasteEnd => {
                self.paste_mode = false;
                self.skip_next_paste_lf = false;
            }
            TerminalAppInput::Char(ch) if self.ui.mode() == UiMode::Prompt => {
                self.prompt_buffer.push(ch);
                self.prompt_history_index = None;
                self.prompt_history_draft = None;
                self.dirty = true;
            }
            TerminalAppInput::Char(ch) if self.ui.mode() == UiMode::Insert => {
                self.chat_buffer.push(ch);
                self.chat_history_index = None;
                self.chat_history_draft = None;
                self.dirty = true;
            }
            TerminalAppInput::Key(UiKey::Left) if self.ui.mode() == UiMode::Prompt => {
                self.prompt_buffer.move_left();
                self.dirty = true;
            }
            TerminalAppInput::Key(UiKey::Right) if self.ui.mode() == UiMode::Prompt => {
                self.prompt_buffer.move_right();
                self.dirty = true;
            }
            TerminalAppInput::Key(UiKey::Up) if self.ui.mode() == UiMode::Prompt => {
                self.recall_prompt_history(-1);
                self.dirty = true;
            }
            TerminalAppInput::Key(UiKey::Down) if self.ui.mode() == UiMode::Prompt => {
                self.recall_prompt_history(1);
                self.dirty = true;
            }
            TerminalAppInput::Key(UiKey::Left) if self.should_route_chat_arrow() => {
                self.chat_buffer.move_left();
                self.dirty = true;
            }
            TerminalAppInput::Key(UiKey::Right) if self.should_route_chat_arrow() => {
                self.chat_buffer.move_right();
                self.dirty = true;
            }
            TerminalAppInput::Key(UiKey::Up) if self.should_route_chat_arrow() => {
                self.recall_chat_history(-1);
                self.dirty = true;
            }
            TerminalAppInput::Key(UiKey::Down) if self.should_route_chat_arrow() => {
                self.recall_chat_history(1);
                self.dirty = true;
            }
            TerminalAppInput::Key(UiKey::Esc) => {
                self.prompt_buffer.clear();
                let actions = self.ui.handle_key(UiKey::Esc);
                self.record_actions(actions);
                self.dirty = true;
            }
            TerminalAppInput::Key(key) => {
                let actions = self.ui.handle_key(key);
                self.record_actions(actions);
                self.dirty = true;
            }
            TerminalAppInput::Char(ch) => {
                let actions = self.ui.handle_key(UiKey::Char(ch));
                self.record_actions(actions);
                self.dirty = true;
            }
            TerminalAppInput::Backspace | TerminalAppInput::Enter | TerminalAppInput::LineBreak => {
            }
        }
    }

    fn handle_command_line(&mut self, line: &str) {
        if is_agent_slash_command(line)
            && let Some(agent_id) = self.ui.selected_agent().cloned()
        {
            self.controller.send_message(&agent_id, line);
            self.apply_controller_events();
            return;
        }
        self.controller.execute_command_line(line);
        self.apply_controller_events();
    }

    fn send_chat_buffer(&mut self) {
        let message = self.chat_buffer.trimmed_string();
        self.chat_buffer.clear();
        self.chat_history_index = None;
        self.chat_history_draft = None;
        if message.is_empty() {
            self.dirty = true;
            return;
        }

        self.chat_history.push(message.clone());
        if let Some(agent_id) = self.ui.selected_agent().cloned() {
            self.controller.send_message(&agent_id, &message);
        } else {
            self.controller.send_command_agent_message(&message);
        }
        self.apply_controller_events();
        self.dirty = true;
    }

    #[cfg(test)]
    fn clear_agent_loading(&mut self, agent_id: &AgentId) {
        self.set_agent_loading(agent_id, None);
    }

    #[cfg(test)]
    fn set_agent_loading(&mut self, agent_id: &AgentId, loading: Option<LoadingKind>) {
        let _ = self.ui.set_agent_ready_state(agent_id, loading.is_none());
    }

    fn record_actions(&mut self, actions: Vec<crate::UiAction>) {
        for action in actions {
            self.controller.push_transcript_line(ui_action_text(action));
        }
        self.apply_controller_events();
    }

    fn apply_controller_events(&mut self) {
        let events = self.controller.drain_events();
        if events.is_empty() {
            return;
        }
        for event in events {
            match event {
                WorkLeafEvent::AgentAdded { session } | WorkLeafEvent::AgentUpdated { session } => {
                    self.upsert_cached_session(session.clone());
                    self.apply_session_to_ui(&session);
                }
                WorkLeafEvent::AgentStatusUpdated {
                    agent_id,
                    kind,
                    title,
                    feature,
                    loading,
                } => {
                    let session =
                        self.upsert_cached_session_status(agent_id, kind, title, feature, loading);
                    self.apply_session_to_ui(&session);
                }
                WorkLeafEvent::AgentLineAppended { agent_id, line } => {
                    self.append_cached_agent_line(&agent_id, line);
                }
                WorkLeafEvent::AgentSelected { agent_id } => {
                    let _ = self.ui.activate_agent_chat(&agent_id);
                }
                WorkLeafEvent::CommandTranscriptLine { line } => {
                    self.snapshot.command_transcript.push(line);
                }
                WorkLeafEvent::QuitRequested => {
                    self.quit = true;
                }
            }
        }
        self.dirty = true;
    }

    fn upsert_cached_session(&mut self, session: WorkLeafSession) {
        if let Some(existing) = self
            .snapshot
            .sessions
            .iter_mut()
            .find(|existing| existing.id == session.id)
        {
            *existing = session;
        } else {
            self.snapshot.sessions.push(session);
            self.snapshot
                .sessions
                .sort_by(|left, right| left.id.as_str().cmp(right.id.as_str()));
        }
    }

    fn upsert_cached_session_status(
        &mut self,
        agent_id: AgentId,
        kind: crate::agent::AgentKind,
        title: String,
        feature: String,
        loading: Option<WorkLeafLoading>,
    ) -> WorkLeafSession {
        if let Some(session) = self
            .snapshot
            .sessions
            .iter_mut()
            .find(|session| session.id == agent_id)
        {
            session.kind = kind;
            session.title = title;
            session.feature = feature;
            session.loading = loading;
            return session.clone();
        }

        let session = WorkLeafSession {
            id: agent_id,
            kind,
            title,
            feature,
            lines: Vec::new(),
            loading,
        };
        self.snapshot.sessions.push(session.clone());
        self.snapshot
            .sessions
            .sort_by(|left, right| left.id.as_str().cmp(right.id.as_str()));
        session
    }

    fn append_cached_agent_line(&mut self, agent_id: &AgentId, line: String) {
        if line.is_empty() {
            return;
        }
        let Some(session) = self
            .snapshot
            .sessions
            .iter_mut()
            .find(|session| &session.id == agent_id)
        else {
            return;
        };
        if !session.lines.iter().any(|existing| existing == &line) {
            session.lines.push(line);
        }
    }

    fn apply_session_to_ui(&mut self, session: &WorkLeafSession) {
        if self
            .ui
            .set_agent_feature(&session.id, session.title.clone())
            .is_err()
        {
            self.ui.add_agent(AgentListEntry::new(
                session.id.clone(),
                session.title.clone(),
            ));
        }
        let _ = self
            .ui
            .set_agent_ready_state(&session.id, session.loading.is_none());
    }

    fn should_start_agent_slash_command(&self) -> bool {
        self.ui.mode() == UiMode::Command && self.ui.selected_agent().is_some()
    }

    fn start_agent_slash_command(&mut self) {
        let Some(agent_id) = self.ui.selected_agent().cloned() else {
            return;
        };
        if self.ui.activate_agent_chat(&agent_id).is_ok() {
            self.chat_buffer.push('/');
            self.chat_history_index = None;
            self.chat_history_draft = None;
        }
    }

    fn should_route_chat_arrow(&self) -> bool {
        self.ui.mode() == UiMode::Insert
            || (self.ui.mode() == UiMode::Command && self.ui.focus() == PaneFocus::Right)
    }

    fn defer_escape_key(&self) -> bool {
        self.ui.mode() == UiMode::Prompt
            || (self.ui.mode() == UiMode::Insert && self.ui.focus() == PaneFocus::Right)
    }

    fn finish_pending_escape_sequence(&mut self) {
        let should_finish = self
            .escape_sequence
            .as_ref()
            .is_some_and(|sequence| sequence.bytes.is_empty());
        if should_finish {
            let sequence = self
                .escape_sequence
                .take()
                .expect("escape sequence is present");
            self.dispatch_pending_escape_if_needed(&sequence);
        }
    }

    fn dispatch_pending_escape_if_needed(&mut self, sequence: &PendingEscapeSequence) {
        if !sequence.escape_dispatched {
            self.handle_input(TerminalAppInput::Key(UiKey::Esc));
        }
    }

    fn input_from_byte(&mut self, byte: u8) -> Option<TerminalAppInput> {
        if self.paste_mode {
            match byte {
                13 => {
                    self.skip_next_paste_lf = true;
                    return Some(TerminalAppInput::LineBreak);
                }
                10 if self.skip_next_paste_lf => {
                    self.skip_next_paste_lf = false;
                    return None;
                }
                10 => return Some(TerminalAppInput::LineBreak),
                _ => {
                    self.skip_next_paste_lf = false;
                }
            }
        }
        TerminalAppInput::from_byte(byte)
    }

    fn recall_prompt_history(&mut self, delta: isize) {
        if self.prompt_history.is_empty() {
            return;
        }

        if self.prompt_history_index.is_none() {
            self.prompt_history_draft = Some(self.prompt_buffer.as_str().to_string());
        }

        let current = self
            .prompt_history_index
            .unwrap_or(self.prompt_history.len()) as isize;
        let next = current + delta;
        if next < 0 {
            self.prompt_history_index = Some(0);
            self.prompt_buffer.replace(&self.prompt_history[0]);
        } else if next >= self.prompt_history.len() as isize {
            self.prompt_history_index = None;
            let draft = self.prompt_history_draft.take().unwrap_or_default();
            self.prompt_buffer.replace(&draft);
        } else {
            let next = next as usize;
            self.prompt_history_index = Some(next);
            self.prompt_buffer.replace(&self.prompt_history[next]);
        }
    }

    fn recall_chat_history(&mut self, delta: isize) {
        if self.chat_history.is_empty() {
            return;
        }

        if self.chat_history_index.is_none() {
            self.chat_history_draft = Some(self.chat_buffer.as_str().to_string());
        }

        let current = self.chat_history_index.unwrap_or(self.chat_history.len()) as isize;
        let next = current + delta;
        if next < 0 {
            self.chat_history_index = Some(0);
            self.chat_buffer.replace(&self.chat_history[0]);
        } else if next >= self.chat_history.len() as isize {
            self.chat_history_index = None;
            let draft = self.chat_history_draft.take().unwrap_or_default();
            self.chat_buffer.replace(&draft);
        } else {
            let next = next as usize;
            self.chat_history_index = Some(next);
            self.chat_buffer.replace(&self.chat_history[next]);
        }
    }

    fn request_quit(&mut self) {
        self.controller.shutdown();
        self.quit = true;
        self.dirty = true;
    }

    fn right_content(&self) -> String {
        if let Some(agent_id) = self.ui.selected_agent() {
            let session = self.snapshot.session(agent_id);
            let mut lines = session
                .map(|session| session.lines.clone())
                .unwrap_or_default();
            if let Some(loading) = session.and_then(|session| session.loading) {
                lines.push(format!(
                    "work-leaf: {} {}",
                    self.cached_loading_text(loading),
                    SPINNER[self.spinner]
                ));
            }
            return terminal_right_content(self.chat_buffer.as_str(), &lines);
        }
        terminal_right_content(self.chat_buffer.as_str(), &self.snapshot.command_transcript)
    }

    fn cached_loading_text(&self, loading: WorkLeafLoading) -> &str {
        self.loading_text
            .iter()
            .find(|(kind, _)| *kind == loading)
            .map(|(_, text)| text.as_str())
            .unwrap_or("Waiting for agent")
    }

    fn continue_escape_sequence(&mut self, byte: u8) -> bool {
        let Some(sequence) = self.escape_sequence.as_mut() else {
            return false;
        };

        if sequence.bytes.is_empty() && byte != b'[' {
            let sequence = self
                .escape_sequence
                .take()
                .expect("escape sequence is present");
            self.dispatch_pending_escape_if_needed(&sequence);
            return false;
        }

        sequence.bytes.push(byte);
        if is_complete_control_sequence(&sequence.bytes) {
            let sequence = self
                .escape_sequence
                .take()
                .expect("escape sequence is present");
            if sequence.escape_dispatched
                && sequence.mode_before == UiMode::Insert
                && self.ui.mode() != UiMode::Insert
            {
                let actions = self.ui.handle_key(UiKey::Char('i'));
                self.record_actions(actions);
            }
            if let Some(input) = parse_control_sequence(&sequence.bytes) {
                self.handle_input(input);
            }
        } else if sequence.bytes.len() > MAX_ESCAPE_SEQUENCE {
            let sequence = self
                .escape_sequence
                .take()
                .expect("escape sequence is present");
            self.dispatch_pending_escape_if_needed(&sequence);
        }

        true
    }
}

#[cfg(test)]
type LoadingKind = WorkLeafLoading;

#[derive(Clone, Debug, Eq, PartialEq)]
struct PendingEscapeSequence {
    bytes: Vec<u8>,
    mode_before: UiMode,
    escape_dispatched: bool,
}

const SPINNER: [&str; 4] = ["|", "/", "-", "\\"];

const MAX_ESCAPE_SEQUENCE: usize = 64;

fn is_complete_control_sequence(sequence: &[u8]) -> bool {
    sequence.len() > 1
        && sequence
            .last()
            .is_some_and(|byte| (0x40..=0x7e).contains(byte))
}

fn parse_control_sequence(sequence: &[u8]) -> Option<TerminalAppInput> {
    match sequence {
        [b'[', b'A'] => Some(TerminalAppInput::Key(UiKey::Up)),
        [b'[', b'B'] => Some(TerminalAppInput::Key(UiKey::Down)),
        [b'[', b'C'] => Some(TerminalAppInput::Key(UiKey::Right)),
        [b'[', b'D'] => Some(TerminalAppInput::Key(UiKey::Left)),
        [b'[', b'2', b'0', b'0', b'~'] => Some(TerminalAppInput::PasteStart),
        [b'[', b'2', b'0', b'1', b'~'] => Some(TerminalAppInput::PasteEnd),
        [b'[', b'1', b'3', b';', b'2', b'u']
        | [b'[', b'1', b'3', b';', b'2', b'~']
        | [b'[', b'2', b'7', b';', b'2', b';', b'1', b'3', b'~'] => {
            Some(TerminalAppInput::LineBreak)
        }
        _ => parse_sgr_mouse_event(sequence).map(TerminalAppInput::Key),
    }
}

fn parse_sgr_mouse_event(sequence: &[u8]) -> Option<UiKey> {
    let final_byte = *sequence.last()?;
    if !sequence.starts_with(b"[<") || !matches!(final_byte, b'M' | b'm') {
        return None;
    }

    let body = std::str::from_utf8(&sequence[2..sequence.len() - 1]).ok()?;
    let mut parts = body.split(';');
    let button = parts.next()?.parse::<u16>().ok()?;
    let column = parts.next()?.parse::<u16>().ok()?;
    let row = parts.next()?.parse::<u16>().ok()?;
    if parts.next().is_some() {
        return None;
    }

    let button_kind = button & !0b0001_1100_u16;
    match (button_kind, final_byte) {
        (64, b'M') => Some(UiKey::MouseScrollUp { column, row }),
        (65, b'M') => Some(UiKey::MouseScrollDown { column, row }),
        (_, b'M' | b'm') if button_kind < 64 && button & 0b11 == 0 => {
            Some(UiKey::MouseClick { column, row })
        }
        _ => None,
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum TerminalAppInput {
    Key(UiKey),
    Char(char),
    Enter,
    Backspace,
    LineBreak,
    PasteStart,
    PasteEnd,
    Interrupt,
    Quit,
}

impl TerminalAppInput {
    fn from_byte(byte: u8) -> Option<Self> {
        match byte {
            3 => Some(Self::Interrupt),
            4 => Some(Self::Quit),
            13 | 10 => Some(Self::Enter),
            27 => Some(Self::Key(UiKey::Esc)),
            23 => Some(Self::Key(UiKey::CtrlW)),
            8 | 127 => Some(Self::Backspace),
            byte if byte.is_ascii_graphic() || byte == b' ' => Some(Self::Char(byte as char)),
            _ => None,
        }
    }
}

fn is_agent_slash_command(message: &str) -> bool {
    let mut chars = message.chars();
    matches!((chars.next(), chars.next()), (Some('/'), Some(next)) if !next.is_whitespace())
}

#[derive(Debug)]
struct PromptLine {
    buffer: LineBuffer,
}

impl PromptLine {
    const CAPACITY: usize = 64 * 1024;

    fn new() -> Self {
        Self {
            buffer: LineBuffer::with_capacity(Self::CAPACITY),
        }
    }

    fn as_str(&self) -> &str {
        self.buffer.as_str()
    }

    fn cursor(&self) -> usize {
        self.buffer.pos()
    }

    fn cursor_char_count(&self) -> usize {
        self.as_str()[..self.cursor()].chars().count()
    }

    fn trimmed_string(&self) -> String {
        self.as_str().trim().to_string()
    }

    fn push(&mut self, ch: char) {
        let mut listener = NoopLineListener;
        let _ = self.buffer.insert(ch, 1, &mut listener);
    }

    fn move_left(&mut self) {
        self.buffer.move_backward(1);
    }

    fn move_right(&mut self) {
        self.buffer.move_forward(1);
    }

    fn backspace(&mut self) {
        let mut listener = NoopLineListener;
        self.buffer.backspace(1, &mut listener);
    }

    fn clear(&mut self) {
        let mut listener = NoopLineListener;
        let len = self.buffer.as_str().len();
        self.buffer.replace(0..len, "", &mut listener);
    }

    fn replace(&mut self, text: &str) {
        let mut listener = NoopLineListener;
        let len = self.buffer.as_str().len();
        self.buffer.replace(0..len, text, &mut listener);
        self.buffer.move_end();
    }
}

#[derive(Debug)]
struct NoopLineListener;

impl DeleteListener for NoopLineListener {
    fn delete(&mut self, _idx: usize, _string: &str, _dir: Direction) {}
}

impl ChangeListener for NoopLineListener {
    fn insert_char(&mut self, _idx: usize, _c: char) {}

    fn insert_str(&mut self, _idx: usize, _string: &str) {}

    fn replace(&mut self, _idx: usize, _old: &str, _new: &str) {}
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;
    use std::sync::{
        Arc,
        atomic::{AtomicUsize, Ordering},
    };

    use super::*;
    use crate::agent::{
        AgentError, AgentKind, AgentLaunch, AgentSession, ChatMessage, MessageRole,
    };

    #[derive(Clone, Debug)]
    struct NoopBackend;

    impl AgentBackend for NoopBackend {
        fn launch(&mut self, request: AgentLaunch) -> Result<AgentSession, AgentError> {
            Ok(AgentSession::new(request))
        }

        fn send(&mut self, _agent_id: &AgentId, prompt: &str) -> Result<ChatMessage, AgentError> {
            Ok(ChatMessage::new(MessageRole::Agent, prompt))
        }
    }

    #[derive(Debug)]
    struct CountingController {
        snapshot: crate::WorkLeafSnapshot,
        snapshot_calls: Arc<AtomicUsize>,
        drain_calls: Arc<AtomicUsize>,
    }

    impl CountingController {
        fn new(
            snapshot: crate::WorkLeafSnapshot,
            snapshot_calls: Arc<AtomicUsize>,
            drain_calls: Arc<AtomicUsize>,
        ) -> Self {
            Self {
                snapshot,
                snapshot_calls,
                drain_calls,
            }
        }
    }

    impl TerminalController for CountingController {
        fn snapshot(&self) -> crate::WorkLeafSnapshot {
            self.snapshot_calls.fetch_add(1, Ordering::Relaxed);
            self.snapshot.clone()
        }

        fn drain_events(&mut self) -> Vec<WorkLeafEvent> {
            self.drain_calls.fetch_add(1, Ordering::Relaxed);
            Vec::new()
        }

        fn execute_command_line(&mut self, _line: &str) {}

        fn send_command_agent_message(&mut self, _message: &str) {}

        fn send_message(&mut self, _agent_id: &AgentId, _message: &str) {}

        fn interrupt_agent(&mut self, _agent_id: &AgentId) {}

        fn push_transcript_line(&mut self, _line: String) {}

        fn is_busy(&mut self) -> bool {
            false
        }

        fn loading_text(&self, _loading: WorkLeafLoading) -> String {
            "Waiting for Codex".to_string()
        }

        fn shutdown(&mut self) {}
    }

    #[test]
    fn pasted_command_prompt_input_polls_controller_once_per_chunk() {
        let snapshot_calls = Arc::new(AtomicUsize::new(0));
        let drain_calls = Arc::new(AtomicUsize::new(0));
        let controller = CountingController::new(
            crate::WorkLeafSnapshot {
                command_transcript: Vec::new(),
                sessions: Vec::new(),
            },
            Arc::clone(&snapshot_calls),
            Arc::clone(&drain_calls),
        );
        let mut app = TerminalAppCore::new(controller, 80, 24);
        app.handle_bytes(b":");
        drain_calls.store(0, Ordering::Relaxed);

        let paste = "a".repeat(4096);
        assert!(app.handle_bytes(paste.as_bytes()));

        assert_eq!(app.prompt_buffer.as_str(), paste);
        assert!(
            drain_calls.load(Ordering::Relaxed) <= 3,
            "large input chunks should not drain events once per byte"
        );
    }

    #[test]
    fn rendering_uses_cached_snapshot_instead_of_refetching_full_transcripts() {
        let snapshot_calls = Arc::new(AtomicUsize::new(0));
        let drain_calls = Arc::new(AtomicUsize::new(0));
        let agent_id = AgentId::new("user-1").expect("test agent id is valid");
        let controller = CountingController::new(
            crate::WorkLeafSnapshot {
                command_transcript: vec!["help".to_string()],
                sessions: vec![WorkLeafSession {
                    id: agent_id,
                    kind: AgentKind::Codex,
                    title: "feature".to_string(),
                    feature: "feature".to_string(),
                    lines: vec!["large transcript line".repeat(256)],
                    loading: None,
                }],
            },
            Arc::clone(&snapshot_calls),
            Arc::clone(&drain_calls),
        );
        let app = TerminalAppCore::new(controller, 80, 24);
        snapshot_calls.store(0, Ordering::Relaxed);

        assert!(app.render_frame().contains("help"));
        assert!(app.render_frame().contains("help"));

        assert_eq!(
            snapshot_calls.load(Ordering::Relaxed),
            0,
            "rendering and scrolling should use the local snapshot cache"
        );
    }

    #[test]
    fn clearing_agent_loading_marks_chat_ready_in_left_pane() {
        let chat = CommandChat::new(PathBuf::from("."), NoopBackend);
        let mut app = TerminalApp::new(chat, 80, 24);
        let agent_id = AgentId::new("user-1").expect("test agent id is valid");

        app.inner
            .ui
            .add_agent(AgentListEntry::new(agent_id.clone(), "feature"));
        app.inner
            .ui
            .activate_agent_chat(&agent_id)
            .expect("test agent is registered");
        app.set_agent_loading(&agent_id, Some(LoadingKind::WaitingForReply));

        assert!(!app.render_frame().contains('\u{7}'));
        assert!(!app.ui().render_left_pane().contains("READY"));

        app.clear_agent_loading(&agent_id);

        assert!(app.render_frame().starts_with('\u{7}'));
        assert!(!app.render_frame().contains('\u{7}'));
        assert!(
            app.ui()
                .render_left_pane()
                .contains("\u{1b}[7m>feature user-1  working: feature  READY\u{1b}[0m")
        );
    }

    #[test]
    fn command_surface_insert_mode_renders_chat_buffer() {
        let chat = CommandChat::new(PathBuf::from("."), NoopBackend);
        let mut app = TerminalApp::new(chat, 80, 24);

        app.handle_bytes(b"itype in command agent");

        assert!(app.render_frame().contains("type in command agent"));
    }

    #[test]
    fn command_surface_chat_uses_command_agent_to_spawn_codex_agent() {
        let root =
            std::env::temp_dir().join(format!("work-leaf-command-surface-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&root);
        std::fs::create_dir_all(&root).unwrap();
        let chat = CommandChat::new(root, NoopBackend);
        let mut app = TerminalApp::new(chat, 80, 24);

        assert!(app.ui().selected_agent().is_none());

        app.handle_bytes(b"ispawn a new patch agent that uses codex\n");

        assert!(app.wait_for_idle(Duration::from_secs(1)));
        let agent_id = AgentId::new("user-1").expect("test agent id is valid");
        assert_eq!(app.ui().selected_agent(), Some(&agent_id));
        assert!(app.transcript().iter().any(|line| line
            == "command-agent: launching Codex user agent for patch agent that uses codex"));
        assert!(
            app.transcript()
                .iter()
                .any(|line| line == "work-leaf> new patch agent that uses codex")
        );
    }
}
----- END FILE src/terminal_app.rs -----

----- BEGIN FILE src/ui.rs -----
digest: fnv64:9ee9f02c5bc43f7d; bytes:43323

use std::{
    cell::Cell,
    path::PathBuf,
    time::{Duration, Instant},
};

use crate::agent::AgentId;
use tui::{
    Terminal,
    backend::TestBackend,
    buffer::Buffer,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Modifier, Style},
    text::{Span, Spans},
    widgets::{Block, Borders, List, ListItem, Paragraph, Wrap},
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum UiMode {
    Command,
    Insert,
    Prompt,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PaneFocus {
    Left,
    Right,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum UiSurface {
    WorkLeafCommand,
    AgentChat,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum UiKey {
    Char(char),
    Esc,
    CtrlW,
    Up,
    Down,
    Left,
    Right,
    MouseClick { column: u16, row: u16 },
    MouseScrollUp { column: u16, row: u16 },
    MouseScrollDown { column: u16, row: u16 },
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum UiAction {
    OpenChatSamePane(AgentId),
    OpenChatNewWindow(AgentId),
    ForkAgent(AgentId),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TerminalLayout {
    pub left_width: u16,
    pub right_width: u16,
    pub height: u16,
    pub right_surface: Option<UiSurface>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AgentListEntry {
    pub id: AgentId,
    pub feature: String,
    pub ready: bool,
    pub hidden: bool,
    pub modified_files: Vec<PathBuf>,
    pub conflicting_agents: Vec<AgentId>,
    pub depends_on: Vec<AgentId>,
    pub depended_on_by: Vec<AgentId>,
}

impl AgentListEntry {
    pub fn new(id: AgentId, feature: impl Into<String>) -> Self {
        Self {
            id,
            feature: feature.into(),
            ready: false,
            hidden: false,
            modified_files: Vec::new(),
            conflicting_agents: Vec::new(),
            depends_on: Vec::new(),
            depended_on_by: Vec::new(),
        }
    }

    pub fn with_ready(mut self, ready: bool) -> Self {
        self.ready = ready;
        self
    }

    pub fn with_hidden(mut self, hidden: bool) -> Self {
        self.hidden = hidden;
        self
    }

    pub fn with_modified_file(mut self, path: impl Into<PathBuf>) -> Self {
        self.modified_files.push(path.into());
        self
    }

    pub fn with_conflicting_agent(mut self, agent_id: AgentId) -> Self {
        self.conflicting_agents.push(agent_id);
        self
    }

    pub fn with_dependency(mut self, agent_id: AgentId) -> Self {
        self.depends_on.push(agent_id);
        self
    }

    pub fn with_dependent(mut self, agent_id: AgentId) -> Self {
        self.depended_on_by.push(agent_id);
        self
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum PendingKey {
    CtrlW,
    G,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct StatusNotice {
    message: String,
    expires_at: Instant,
}

const STATUS_NOTICE_SECONDS: u64 = 5;
const COMMAND_MODE_TYPING_NOTICE_THRESHOLD: usize = 5;
const COMMAND_MODE_TYPING_NOTICE: &str = "command mode: press i for insert mode before typing";
const CTRL_C_EXIT_NOTICE: &str = "to exit, press Esc then :q then Enter";

#[derive(Clone, Debug, Eq, PartialEq)]
enum LeftPaneClickTarget {
    Command,
    Agent(AgentId),
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct UiWindow {
    surface: UiSurface,
    agent_id: Option<AgentId>,
}

impl UiWindow {
    fn command() -> Self {
        Self {
            surface: UiSurface::WorkLeafCommand,
            agent_id: None,
        }
    }

    fn chat(agent_id: AgentId) -> Self {
        Self {
            surface: UiSurface::AgentChat,
            agent_id: Some(agent_id),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct PromptView {
    line: String,
    cursor_column: u16,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TerminalUi {
    width: u16,
    height: u16,
    mode: UiMode,
    focus: PaneFocus,
    left_visible: bool,
    agents: Vec<AgentListEntry>,
    selected_agent: Option<AgentId>,
    control_selected: usize,
    split_chats: Vec<AgentId>,
    windows: Vec<UiWindow>,
    active_window: usize,
    right_scroll_rows: usize,
    pending: Option<PendingKey>,
    pending_bell: Cell<bool>,
    status_notice: Option<StatusNotice>,
    command_mode_typing_count: usize,
    command_mode_typing_controls_only: bool,
}

impl TerminalUi {
    pub fn new(width: u16, height: u16) -> Self {
        Self {
            width,
            height,
            mode: UiMode::Command,
            focus: PaneFocus::Left,
            left_visible: true,
            agents: Vec::new(),
            selected_agent: None,
            control_selected: 0,
            split_chats: Vec::new(),
            windows: vec![UiWindow::command()],
            active_window: 0,
            right_scroll_rows: 0,
            pending: None,
            pending_bell: Cell::new(false),
            status_notice: None,
            command_mode_typing_count: 0,
            command_mode_typing_controls_only: true,
        }
    }

    pub fn layout(&self) -> TerminalLayout {
        let left_width = if self.left_visible { self.width / 5 } else { 0 };
        let right_width = self.width.saturating_sub(left_width);
        TerminalLayout {
            left_width,
            right_width,
            height: self.height,
            right_surface: Some(self.windows[self.active_window].surface),
        }
    }

    pub fn mode(&self) -> UiMode {
        self.mode
    }

    pub fn focus(&self) -> PaneFocus {
        self.focus
    }

    pub(crate) fn show_ctrl_c_exit_notice(&mut self) {
        self.show_status_notice(
            CTRL_C_EXIT_NOTICE,
            Duration::from_secs(STATUS_NOTICE_SECONDS),
        );
    }

    pub(crate) fn has_status_notice(&self) -> bool {
        self.status_notice.is_some()
    }

    pub(crate) fn clear_expired_status_notice(&mut self) {
        if self.status_notice_expired() {
            self.status_notice = None;
        }
    }

    pub fn window_count(&self) -> usize {
        self.windows.len()
    }

    pub fn active_window(&self) -> usize {
        self.active_window
    }

    pub fn selected_agent(&self) -> Option<&AgentId> {
        self.selected_agent.as_ref()
    }

    pub fn control_selected_row(&self) -> usize {
        self.control_selected
    }

    pub fn add_agent(&mut self, agent: AgentListEntry) {
        self.agents.push(agent);
    }

    pub(crate) fn set_agent_feature(
        &mut self,
        agent_id: &AgentId,
        feature: impl Into<String>,
    ) -> Result<(), String> {
        let Some(agent) = self.agents.iter_mut().find(|agent| &agent.id == agent_id) else {
            return Err(format!("unknown agent `{agent_id}`"));
        };
        agent.feature = feature.into();
        Ok(())
    }

    pub(crate) fn set_agent_ready_state(
        &mut self,
        agent_id: &AgentId,
        ready: bool,
    ) -> Result<(), String> {
        let Some(agent) = self.agents.iter_mut().find(|agent| &agent.id == agent_id) else {
            return Err(format!("unknown agent `{agent_id}`"));
        };
        if ready && !agent.ready {
            self.pending_bell.set(true);
        }
        agent.ready = ready;
        Ok(())
    }

    pub fn select_agent(&mut self, agent_id: &AgentId) -> Result<(), String> {
        if self.agents.iter().any(|agent| &agent.id == agent_id) {
            self.selected_agent = Some(agent_id.clone());
            self.windows[self.active_window] = UiWindow::chat(agent_id.clone());
            self.control_selected = self
                .visible_agent_indices()
                .iter()
                .position(|index| self.agents[*index].id == *agent_id)
                .map(|position| position + 1)
                .unwrap_or(self.control_selected);
            self.reset_right_scroll();
            Ok(())
        } else {
            Err(format!("unknown agent `{agent_id}`"))
        }
    }

    pub fn activate_agent_chat(&mut self, agent_id: &AgentId) -> Result<(), String> {
        self.select_agent(agent_id)?;
        self.focus = PaneFocus::Right;
        self.mode = UiMode::Insert;
        Ok(())
    }

    pub fn select_command_interface(&mut self) {
        self.selected_agent = None;
        self.windows[self.active_window] = UiWindow::command();
        self.control_selected = 0;
        self.reset_right_scroll();
    }

    pub fn handle_key(&mut self, key: UiKey) -> Vec<UiAction> {
        let command_mode_text_key = self.command_mode_text_key_control_status(key);
        let actions = self.handle_key_inner(key);
        self.update_command_mode_typing_notice(command_mode_text_key);
        actions
    }

    fn handle_key_inner(&mut self, key: UiKey) -> Vec<UiAction> {
        match key {
            UiKey::MouseClick { column, row } => {
                self.pending = None;
                return self.handle_mouse_click(column, row);
            }
            UiKey::MouseScrollUp { column, row } => {
                self.pending = None;
                self.handle_mouse_scroll(column, row, true);
                return Vec::new();
            }
            UiKey::MouseScrollDown { column, row } => {
                self.pending = None;
                self.handle_mouse_scroll(column, row, false);
                return Vec::new();
            }
            _ => {}
        }

        if let Some(pending) = self.pending.take() {
            return self.handle_pending_key(pending, key);
        }

        match key {
            UiKey::Esc => {
                self.mode = UiMode::Command;
                Vec::new()
            }
            UiKey::CtrlW if self.mode == UiMode::Command => {
                self.pending = Some(PendingKey::CtrlW);
                Vec::new()
            }
            UiKey::Char('g') if self.mode == UiMode::Command => {
                self.pending = Some(PendingKey::G);
                Vec::new()
            }
            UiKey::Char('i') if self.mode == UiMode::Command => {
                self.mode = UiMode::Insert;
                Vec::new()
            }
            UiKey::Char(':') if self.mode == UiMode::Command => {
                self.mode = UiMode::Prompt;
                Vec::new()
            }
            UiKey::Char(',') if self.mode == UiMode::Command => {
                self.left_visible = !self.left_visible;
                self.focus = if self.left_visible {
                    PaneFocus::Left
                } else {
                    PaneFocus::Right
                };
                Vec::new()
            }
            UiKey::Char('j') if self.mode == UiMode::Command && self.focus == PaneFocus::Left => {
                self.move_control_selection(1);
                self.select_control_row_surface();
                Vec::new()
            }
            UiKey::Down if self.mode == UiMode::Command && self.focus == PaneFocus::Left => {
                self.move_control_selection(1);
                self.select_control_row_surface();
                Vec::new()
            }
            UiKey::Right if self.mode == UiMode::Command && self.focus == PaneFocus::Left => {
                self.move_control_selection(1);
                self.select_control_row_surface();
                Vec::new()
            }
            UiKey::Char('k') if self.mode == UiMode::Command && self.focus == PaneFocus::Left => {
                self.move_control_selection(-1);
                self.select_control_row_surface();
                Vec::new()
            }
            UiKey::Up if self.mode == UiMode::Command && self.focus == PaneFocus::Left => {
                self.move_control_selection(-1);
                self.select_control_row_surface();
                Vec::new()
            }
            UiKey::Left if self.mode == UiMode::Command && self.focus == PaneFocus::Left => {
                self.move_control_selection(-1);
                self.select_control_row_surface();
                Vec::new()
            }
            UiKey::Char('l') if self.mode == UiMode::Command && self.focus == PaneFocus::Left => {
                self.open_control_selection();
                Vec::new()
            }
            UiKey::Char('x') if self.mode == UiMode::Command && self.focus == PaneFocus::Left => {
                self.hide_control_selection();
                Vec::new()
            }
            UiKey::Char('s') if self.mode == UiMode::Command => self.open_selected_same_pane(),
            UiKey::Char('t') if self.mode == UiMode::Command => self.open_selected_new_window(),
            UiKey::Char('f') if self.mode == UiMode::Command => self.fork_selected_agent(),
            _ => Vec::new(),
        }
    }

    pub fn render_left_pane(&self) -> String {
        let mut rendered = String::new();
        if self.control_selected == 0 {
            rendered.push_str("> work-leaf  command\n");
        } else {
            rendered.push_str("  work-leaf  command\n");
        }
        for (visible_position, agent_index) in self.visible_agent_indices().iter().enumerate() {
            let agent = &self.agents[*agent_index];
            let mut row = String::new();
            row.push(if self.control_selected == visible_position + 1 {
                '>'
            } else {
                ' '
            });
            let (primary, secondary) = agent_list_labels(agent);
            row.push_str(primary);
            row.push(' ');
            row.push_str(secondary);
            row.push_str("  working: ");
            row.push_str(&agent.feature);
            if agent.ready {
                row.push_str("  READY");
                rendered.push_str("\u{1b}[7m");
                rendered.push_str(&row);
                rendered.push_str("\u{1b}[0m");
            } else {
                rendered.push_str(&row);
            }
            rendered.push('\n');
            if !agent.modified_files.is_empty() {
                rendered.push_str("    ");
                rendered.push_str("files: ");
                rendered.push_str(
                    &agent
                        .modified_files
                        .iter()
                        .map(|path| path.display().to_string())
                        .collect::<Vec<_>>()
                        .join(", "),
                );
                rendered.push('\n');
            }
            self.render_agent_links("conflicts", &agent.conflicting_agents, &mut rendered);
            self.render_agent_links("depends-on", &agent.depends_on, &mut rendered);
            self.render_agent_links("depended-on-by", &agent.depended_on_by, &mut rendered);
        }
        rendered
    }

    pub fn render_screen(&self, right_content: &str) -> String {
        self.render_screen_with_prompt(right_content, "")
    }

    pub fn render_screen_with_prompt(&self, right_content: &str, prompt: &str) -> String {
        let visible_right_content = self.visible_right_content(right_content);
        let prompt_cursor = prompt.len();
        let buffer = self.render_tui_buffer(&visible_right_content, prompt, prompt_cursor);
        let mut rendered = String::new();
        rendered.push_str(self.bell_prefix());
        rendered.push_str("\u{1b}[H");
        rendered.push_str(&buffer_to_string(&buffer));
        rendered.push_str(&self.cursor_sequence(&visible_right_content, prompt));
        rendered
    }

    pub fn render_screen_with_cursors(
        &self,
        right_content: &str,
        prompt: &str,
        prompt_cursor: usize,
        right_cursor_column: Option<usize>,
    ) -> String {
        let visible_right_content = self.visible_right_content(right_content);
        let buffer = self.render_tui_buffer(&visible_right_content, prompt, prompt_cursor);
        let mut rendered = String::new();
        rendered.push_str(self.bell_prefix());
        rendered.push_str("\u{1b}[H");
        rendered.push_str(&buffer_to_string(&buffer));
        rendered.push_str(&self.cursor_sequence_with_cursors(
            &visible_right_content,
            prompt,
            prompt_cursor,
            right_cursor_column,
        ));
        rendered
    }

    pub fn scroll_right_pane_up(&mut self) {
        self.right_scroll_rows = self.right_scroll_rows.saturating_add(3);
    }

    pub fn scroll_right_pane_down(&mut self) {
        self.right_scroll_rows = self.right_scroll_rows.saturating_sub(3);
    }

    pub fn reset_right_scroll(&mut self) {
        self.right_scroll_rows = 0;
    }

    fn visible_right_content(&self, right_content: &str) -> String {
        let (inner_width, inner_height) = self.right_inner_size();
        visible_content(
            right_content,
            inner_width,
            inner_height,
            self.right_scroll_rows,
        )
    }

    fn render_tui_buffer(&self, right_content: &str, prompt: &str, prompt_cursor: usize) -> Buffer {
        let backend = TestBackend::new(self.width, self.height);
        let mut terminal = Terminal::new(backend).expect("test backend is valid");
        terminal
            .draw(|frame| {
                let area = frame.size();
                let body_height = area.height.saturating_sub(1);
                let body = Rect::new(area.x, area.y, area.width, body_height);
                let bottom = Rect::new(area.x, body_height, area.width, 1);
                let layout = self.layout();
                let panes = if layout.left_width > 0 {
                    Layout::default()
                        .direction(Direction::Horizontal)
                        .constraints([
                            Constraint::Length(layout.left_width),
                            Constraint::Length(layout.right_width),
                        ])
                        .split(body)
                } else {
                    vec![body]
                };

                if layout.left_width > 0 {
                    frame.render_widget(self.left_widget(), panes[0]);
                    frame.render_widget(self.right_widget(right_content), panes[1]);
                } else {
                    frame.render_widget(self.right_widget(right_content), panes[0]);
                }
                frame.render_widget(
                    Paragraph::new(self.bottom_line(prompt, prompt_cursor)),
                    bottom,
                );
            })
            .expect("test backend draw succeeds");
        terminal.backend().buffer().clone()
    }

    fn left_widget(&self) -> List<'static> {
        let inner_width = usize::from(self.layout().left_width.saturating_sub(2).max(1));
        let mut items = vec![ListItem::new(if self.control_selected == 0 {
            Spans::from(vec![Span::raw("> work-leaf  command")])
        } else {
            Spans::from(vec![Span::raw("  work-leaf  command")])
        })];
        for (visible_position, agent_index) in self.visible_agent_indices().iter().enumerate() {
            let agent = &self.agents[*agent_index];
            let selected = self.control_selected == visible_position + 1;
            let item = ListItem::new(Spans::from(vec![Span::raw(compact_agent_row(
                agent,
                selected,
                inner_width,
            ))]));
            let item = if agent.ready {
                item.style(Style::default().add_modifier(Modifier::REVERSED))
            } else {
                item
            };
            items.push(item);
            if !agent.modified_files.is_empty() {
                items.push(ListItem::new(Spans::from(vec![Span::raw(format!(
                    "    files: {}",
                    agent
                        .modified_files
                        .iter()
                        .map(|path| path.display().to_string())
                        .collect::<Vec<_>>()
                        .join(", ")
                ))])));
            }
            for (label, agents) in [
                ("conflicts", &agent.conflicting_agents),
                ("depends-on", &agent.depends_on),
                ("depended-on-by", &agent.depended_on_by),
            ] {
                if !agents.is_empty() {
                    items.push(ListItem::new(Spans::from(vec![Span::raw(format!(
                        "    {label}: {}",
                        agents
                            .iter()
                            .map(AgentId::as_str)
                            .collect::<Vec<_>>()
                            .join(", ")
                    ))])));
                }
            }
        }
        List::new(items).block(Block::default().title("work-leaf").borders(Borders::ALL))
    }

    fn right_widget(&self, right_content: &str) -> Paragraph<'static> {
        let title = match self.windows[self.active_window].surface {
            UiSurface::WorkLeafCommand => "command",
            UiSurface::AgentChat => "chat",
        };
        Paragraph::new(right_content.to_string())
            .block(Block::default().title(title).borders(Borders::ALL))
            .wrap(Wrap { trim: false })
    }

    fn bottom_line(&self, prompt: &str, prompt_cursor: usize) -> String {
        if self.mode == UiMode::Prompt {
            self.prompt_view(prompt, prompt_cursor).line
        } else {
            self.render_status_line()
        }
    }

    fn cursor_sequence(&self, right_content: &str, prompt: &str) -> String {
        self.cursor_sequence_with_cursors(right_content, prompt, prompt.len(), None)
    }

    fn cursor_sequence_with_cursors(
        &self,
        right_content: &str,
        prompt: &str,
        prompt_cursor: usize,
        right_cursor_column: Option<usize>,
    ) -> String {
        let (row, column) = if self.mode == UiMode::Prompt {
            (
                self.height,
                self.prompt_view(prompt, prompt_cursor).cursor_column,
            )
        } else {
            match self.focus {
                PaneFocus::Left => (self.control_cursor_row(), 2),
                PaneFocus::Right => {
                    self.right_cursor_position_with_cursor(right_content, right_cursor_column)
                }
            }
        };
        let row = row.clamp(1, self.height.max(1));
        let column = column.clamp(1, self.width.max(1));
        format!("\u{1b}[{row};{column}H")
    }

    fn prompt_view(&self, prompt: &str, prompt_cursor: usize) -> PromptView {
        let width = usize::from(self.width.max(1));
        let input_width = width.saturating_sub(1);
        let max_cursor_offset = width.saturating_sub(2);
        let cursor_chars = cursor_char_count(prompt, prompt_cursor);
        let start = cursor_chars.saturating_sub(max_cursor_offset);
        let visible_prompt = prompt
            .chars()
            .skip(start)
            .take(input_width)
            .collect::<String>();
        let cursor_offset = cursor_chars.saturating_sub(start).min(max_cursor_offset);
        let cursor_column = if self.width <= 1 {
            1
        } else {
            cursor_offset.saturating_add(2).min(width) as u16
        };
        PromptView {
            line: format!(":{visible_prompt}"),
            cursor_column,
        }
    }

    fn bell_prefix(&self) -> &'static str {
        if self.pending_bell.replace(false) {
            "\u{7}"
        } else {
            ""
        }
    }

    fn handle_pending_key(&mut self, pending: PendingKey, key: UiKey) -> Vec<UiAction> {
        match (pending, key) {
            (PendingKey::CtrlW, UiKey::Char('h')) if self.mode == UiMode::Command => {
                if self.left_visible {
                    self.focus = PaneFocus::Left;
                }
                Vec::new()
            }
            (PendingKey::CtrlW, UiKey::Char('k')) if self.mode == UiMode::Command => {
                if self.left_visible {
                    self.focus = PaneFocus::Left;
                }
                Vec::new()
            }
            (PendingKey::CtrlW, UiKey::Char('l')) if self.mode == UiMode::Command => {
                self.focus = PaneFocus::Right;
                self.mode = UiMode::Command;
                Vec::new()
            }
            (PendingKey::CtrlW, UiKey::Char('j')) if self.mode == UiMode::Command => {
                self.focus = PaneFocus::Right;
                self.mode = UiMode::Command;
                Vec::new()
            }
            (PendingKey::G, UiKey::Char('t')) if self.mode == UiMode::Command => {
                self.next_window();
                Vec::new()
            }
            (PendingKey::G, UiKey::Char('T')) if self.mode == UiMode::Command => {
                self.previous_window();
                Vec::new()
            }
            _ => Vec::new(),
        }
    }

    fn command_mode_text_key_control_status(&self, key: UiKey) -> Option<bool> {
        if self.mode != UiMode::Command || self.pending.is_some() {
            return None;
        }
        let UiKey::Char(ch) = key else {
            return None;
        };
        (ch.is_ascii_alphanumeric() || ch == ' ').then(|| self.is_command_control_char(ch))
    }

    fn is_command_control_char(&self, ch: char) -> bool {
        matches!(ch, 'i' | ':' | ',' | 's' | 't' | 'f' | 'g')
            || (self.focus == PaneFocus::Left && matches!(ch, 'j' | 'k' | 'l' | 'x'))
    }

    fn update_command_mode_typing_notice(&mut self, command_mode_text_key_control: Option<bool>) {
        if let Some(command_mode_text_key_control) = command_mode_text_key_control
            && self.mode == UiMode::Command
            && self.pending.is_none()
        {
            self.command_mode_typing_count = self.command_mode_typing_count.saturating_add(1);
            self.command_mode_typing_controls_only &= command_mode_text_key_control;
            if self.command_mode_typing_count >= COMMAND_MODE_TYPING_NOTICE_THRESHOLD {
                if !self.command_mode_typing_controls_only {
                    self.show_status_notice(
                        COMMAND_MODE_TYPING_NOTICE,
                        Duration::from_secs(STATUS_NOTICE_SECONDS),
                    );
                }
                self.command_mode_typing_count = 0;
                self.command_mode_typing_controls_only = true;
            }
        } else {
            self.command_mode_typing_count = 0;
            self.command_mode_typing_controls_only = true;
        }
    }

    fn show_status_notice(&mut self, message: impl Into<String>, duration: Duration) {
        self.status_notice = Some(StatusNotice {
            message: message.into(),
            expires_at: Instant::now() + duration,
        });
    }

    fn active_status_notice(&self) -> Option<&str> {
        self.status_notice
            .as_ref()
            .filter(|notice| Instant::now() < notice.expires_at)
            .map(|notice| notice.message.as_str())
    }

    fn status_notice_expired(&self) -> bool {
        self.status_notice
            .as_ref()
            .is_some_and(|notice| Instant::now() >= notice.expires_at)
    }

    fn open_selected_same_pane(&mut self) -> Vec<UiAction> {
        let Some(agent_id) = self.action_agent_id() else {
            return Vec::new();
        };
        self.split_chats.push(agent_id.clone());
        vec![UiAction::OpenChatSamePane(agent_id)]
    }

    fn open_selected_new_window(&mut self) -> Vec<UiAction> {
        let Some(agent_id) = self.action_agent_id() else {
            return Vec::new();
        };
        self.windows.push(UiWindow::chat(agent_id.clone()));
        self.active_window = self.windows.len() - 1;
        vec![UiAction::OpenChatNewWindow(agent_id)]
    }

    fn fork_selected_agent(&self) -> Vec<UiAction> {
        self.action_agent_id()
            .map(UiAction::ForkAgent)
            .into_iter()
            .collect()
    }

    fn open_control_selection(&mut self) {
        if self.control_selected == 0 {
            self.select_command_interface();
            self.focus = PaneFocus::Right;
            return;
        }
        if let Some(agent_id) = self.control_selected_agent_id() {
            let _ = self.select_agent(&agent_id);
            self.focus = PaneFocus::Right;
        }
    }

    fn select_control_row_surface(&mut self) {
        if self.control_selected == 0 {
            self.select_command_interface();
            self.focus = PaneFocus::Left;
            return;
        }
        if let Some(agent_id) = self.control_selected_agent_id() {
            let _ = self.select_agent(&agent_id);
            self.focus = PaneFocus::Left;
        }
    }

    fn handle_mouse_click(&mut self, column: u16, row: u16) -> Vec<UiAction> {
        if column == 0 || row == 0 {
            return Vec::new();
        }

        let left_width = self.layout().left_width;

        if self.left_visible && column <= left_width {
            self.mode = UiMode::Command;
            self.focus = PaneFocus::Left;
            let Some(target) = self.left_pane_click_target(row) else {
                return Vec::new();
            };
            match target {
                LeftPaneClickTarget::Command => {
                    self.select_command_interface();
                    self.focus = PaneFocus::Right;
                }
                LeftPaneClickTarget::Agent(agent_id) => {
                    let _ = self.activate_agent_chat(&agent_id);
                }
            }
        } else {
            self.focus = PaneFocus::Right;
            self.mode = if self.selected_agent.is_some()
                && self.windows[self.active_window].surface == UiSurface::AgentChat
            {
                UiMode::Insert
            } else {
                UiMode::Command
            };
        }

        Vec::new()
    }

    fn handle_mouse_scroll(&mut self, column: u16, row: u16, up: bool) {
        if column == 0 || row == 0 || row >= self.height {
            return;
        }

        let left_width = self.layout().left_width;
        if self.left_visible && column <= left_width {
            return;
        }

        if up {
            self.scroll_right_pane_up();
        } else {
            self.scroll_right_pane_down();
        }
    }

    fn hide_control_selection(&mut self) {
        if self.control_selected == 0 {
            return;
        }
        let Some(agent_index) = self.control_selected_agent_index() else {
            return;
        };
        let hidden_agent = self.agents[agent_index].id.clone();
        self.agents[agent_index].hidden = true;
        let hidden_was_selected = self
            .selected_agent
            .as_ref()
            .is_some_and(|selected| selected == &hidden_agent);
        self.clamp_control_selection();
        if hidden_was_selected {
            self.select_control_row_surface();
        }
    }

    fn move_control_selection(&mut self, delta: isize) {
        let max_row = self.visible_agent_indices().len();
        let current = self.control_selected as isize;
        let next = (current + delta).clamp(0, max_row as isize);
        self.control_selected = next as usize;
    }

    fn clamp_control_selection(&mut self) {
        let max_row = self.visible_agent_indices().len();
        if self.control_selected > max_row {
            self.control_selected = max_row;
        }
    }

    fn visible_agent_indices(&self) -> Vec<usize> {
        self.agents
            .iter()
            .enumerate()
            .filter_map(|(index, agent)| (!agent.hidden).then_some(index))
            .collect()
    }

    fn control_selected_agent_index(&self) -> Option<usize> {
        if self.control_selected == 0 {
            return None;
        }
        self.visible_agent_indices()
            .get(self.control_selected - 1)
            .copied()
    }

    fn control_selected_agent_id(&self) -> Option<AgentId> {
        self.control_selected_agent_index()
            .map(|index| self.agents[index].id.clone())
    }

    fn left_pane_click_target(&mut self, row: u16) -> Option<LeftPaneClickTarget> {
        let list_row = usize::from(row.saturating_sub(2));
        if row < 2 {
            return None;
        }
        if list_row == 0 {
            self.control_selected = 0;
            return Some(LeftPaneClickTarget::Command);
        }

        let mut current_row = 1;
        for (visible_position, agent_index) in self.visible_agent_indices().iter().enumerate() {
            let agent = &self.agents[*agent_index];
            if list_row == current_row {
                self.control_selected = visible_position + 1;
                return Some(LeftPaneClickTarget::Agent(agent.id.clone()));
            }
            current_row += 1;

            if !agent.modified_files.is_empty() {
                if list_row == current_row {
                    self.control_selected = visible_position + 1;
                    return Some(LeftPaneClickTarget::Agent(agent.id.clone()));
                }
                current_row += 1;
            }

            for linked_agents in [
                &agent.conflicting_agents,
                &agent.depends_on,
                &agent.depended_on_by,
            ] {
                if !linked_agents.is_empty() {
                    if list_row == current_row {
                        self.control_selected = visible_position + 1;
                        return linked_agents
                            .first()
                            .cloned()
                            .map(LeftPaneClickTarget::Agent);
                    }
                    current_row += 1;
                }
            }
        }

        None
    }

    fn action_agent_id(&self) -> Option<AgentId> {
        self.control_selected_agent_id()
            .or_else(|| self.selected_agent.clone())
    }

    fn control_cursor_row(&self) -> u16 {
        (self.control_selected + 2).min(usize::from(u16::MAX)) as u16
    }

    fn right_cursor_position_with_cursor(
        &self,
        right_content: &str,
        cursor_column: Option<usize>,
    ) -> (u16, u16) {
        let layout = self.layout();
        let inner_width = layout.right_width.saturating_sub(2).max(1);
        let lines = right_content.lines().collect::<Vec<_>>();
        let Some(line) = lines.last().copied() else {
            return (2, layout.left_width.saturating_add(2));
        };
        if !line.starts_with("chat> ") {
            return (2, layout.left_width.saturating_add(2));
        }
        let previous_rows = lines[..lines.len() - 1]
            .iter()
            .map(|line| visual_rows(line, inner_width))
            .sum::<u16>();
        let line_chars = line.chars().count();
        let line_len = cursor_column
            .unwrap_or(line_chars)
            .min(line_chars)
            .min(usize::from(u16::MAX)) as u16;
        let row = 2_u16
            .saturating_add(previous_rows)
            .saturating_add(line_len / inner_width);
        let column = layout
            .left_width
            .saturating_add(2)
            .saturating_add(line_len % inner_width);
        (row, column)
    }
    fn right_inner_size(&self) -> (u16, u16) {
        let layout = self.layout();
        let inner_width = layout.right_width.saturating_sub(2).max(1);
        let body_height = self.height.saturating_sub(1);
        let inner_height = body_height.saturating_sub(2).max(1);
        (inner_width, inner_height)
    }

    fn next_window(&mut self) {
        if !self.windows.is_empty() {
            self.active_window = (self.active_window + 1) % self.windows.len();
        }
    }

    fn previous_window(&mut self) {
        if !self.windows.is_empty() {
            self.active_window = if self.active_window == 0 {
                self.windows.len() - 1
            } else {
                self.active_window - 1
            };
        }
    }

    fn render_agent_links(&self, label: &str, agents: &[AgentId], rendered: &mut String) {
        if agents.is_empty() {
            return;
        }
        rendered.push_str("    ");
        rendered.push_str(label);
        rendered.push_str(": ");
        for (index, agent_id) in agents.iter().enumerate() {
            if index > 0 {
                rendered.push_str(", ");
            }
            rendered.push_str(agent_id.as_str());
        }
        rendered.push('\n');
    }

    fn render_status_line(&self) -> String {
        if let Some(notice) = self.active_status_notice() {
            return notice.to_string();
        }

        format!(
            "mode={} focus={} window={}/{}",
            self.mode.as_str(),
            self.focus.as_str(),
            self.active_window + 1,
            self.windows.len()
        )
    }
}

impl UiMode {
    fn as_str(&self) -> &'static str {
        match self {
            Self::Command => "command",
            Self::Insert => "insert",
            Self::Prompt => "prompt",
        }
    }
}

impl PaneFocus {
    fn as_str(&self) -> &'static str {
        match self {
            Self::Left => "left",
            Self::Right => "right",
        }
    }
}

fn agent_list_labels(agent: &AgentListEntry) -> (&str, &str) {
    if agent.id.as_str().starts_with("review-") {
        (agent.id.as_str(), &agent.feature)
    } else {
        (&agent.feature, agent.id.as_str())
    }
}

fn compact_agent_row(agent: &AgentListEntry, selected: bool, width: usize) -> String {
    let prefix = if selected { ">" } else { " " };
    let status = if agent.ready { " READY" } else { "" };
    let id = agent.id.as_str();
    let width = width.max(1);

    let row = if id.starts_with("review-") {
        compact_fixed_first(prefix, id, &agent.feature, status, width)
    } else {
        compact_fixed_last(prefix, &agent.feature, id, status, width)
    };
    truncate_to_width(&row, width)
}

fn compact_fixed_first(
    prefix: &str,
    fixed: &str,
    flexible: &str,
    status: &str,
    width: usize,
) -> String {
    let fixed_width = prefix.chars().count() + fixed.chars().count() + status.chars().count();
    if fixed_width >= width {
        return format!("{prefix}{fixed}{status}");
    }

    let flexible_width = width.saturating_sub(fixed_width + 1);
    format!(
        "{prefix}{fixed} {}{status}",
        truncate_to_width(flexible, flexible_width)
    )
}

fn compact_fixed_last(
    prefix: &str,
    flexible: &str,
    fixed: &str,
    status: &str,
    width: usize,
) -> String {
    let fixed_width = prefix.chars().count() + 1 + fixed.chars().count() + status.chars().count();
    if fixed_width >= width {
        return format!("{prefix}{fixed}{status}");
    }

    let flexible_width = width.saturating_sub(fixed_width);
    format!(
        "{prefix}{} {fixed}{status}",
        truncate_to_width(flexible, flexible_width)
    )
}

fn truncate_to_width(text: &str, width: usize) -> String {
    text.chars().take(width).collect()
}

fn buffer_to_string(buffer: &Buffer) -> String {
    const ANSI_REVERSE_VIDEO: &str = "\u{1b}[7m";
    const ANSI_RESET: &str = "\u{1b}[0m";
    let mut output = String::new();
    for y in 0..buffer.area.height {
        let mut reversed = false;
        for x in 0..buffer.area.width {
            let cell = buffer.get(x, y);
            let cell_reversed = cell.modifier.contains(Modifier::REVERSED);
            if cell_reversed != reversed {
                output.push_str(if cell_reversed {
                    ANSI_REVERSE_VIDEO
                } else {
                    ANSI_RESET
                });
                reversed = cell_reversed;
            }
            output.push_str(&cell.symbol);
        }
        if reversed {
            output.push_str(ANSI_RESET);
        }
        if y + 1 < buffer.area.height {
            output.push_str("\r\n");
        }
    }
    output
}

fn visual_rows(line: &str, width: u16) -> u16 {
    visual_row_count(line, width).min(usize::from(u16::MAX)) as u16
}

fn visual_row_count(line: &str, width: u16) -> usize {
    let width = usize::from(width.max(1));
    let len = line.chars().count().min(usize::from(u16::MAX));
    (len / width).saturating_add(1)
}

fn cursor_char_count(text: &str, cursor: usize) -> usize {
    text.char_indices()
        .take_while(|(index, _)| *index < cursor)
        .count()
}

fn visible_content(content: &str, width: u16, height: u16, scroll_rows: usize) -> String {
    let height = usize::from(height);
    let Some((history, prompt)) = split_chat_prompt(content) else {
        return tail_visible_content(content, width, height, scroll_rows);
    };

    let prompt_rows = visual_row_count(prompt, width);
    let history_height = height.saturating_sub(prompt_rows).max(1);
    let visible_history = tail_visible_content(history, width, history_height, scroll_rows);
    if visible_history.is_empty() {
        prompt.to_string()
    } else {
        format!("{visible_history}\n{prompt}")
    }
}

fn split_chat_prompt(content: &str) -> Option<(&str, &str)> {
    let (history, prompt) = content.rsplit_once('\n')?;
    prompt.starts_with("chat> ").then_some((history, prompt))
}

fn tail_visible_content(content: &str, width: u16, height: usize, scroll_rows: usize) -> String {
    if content.is_empty() || height == 0 {
        return String::new();
    }

    let lines = content.lines().collect::<Vec<_>>();
    let rows_to_skip = scroll_rows.min(
        lines
            .iter()
            .map(|line| visual_row_count(line, width))
            .sum::<usize>()
            .saturating_sub(height),
    );
    let mut visible = Vec::new();
    let mut skipped_rows = 0_usize;
    let mut used_rows = 0_usize;
    for line in lines.iter().rev() {
        let rows = visual_row_count(line, width);
        if skipped_rows.saturating_add(rows) <= rows_to_skip {
            skipped_rows = skipped_rows.saturating_add(rows);
            continue;
        }
        if visible.is_empty() || used_rows.saturating_add(rows) <= height {
            visible.push(*line);
            used_rows = used_rows.saturating_add(rows);
        } else {
            break;
        }
    }
    visible.reverse();
    visible.join("\n")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn setting_agent_ready_queues_one_bell_for_next_render() {
        let mut ui = TerminalUi::new(80, 24);
        let agent_id = AgentId::new("user-1").expect("test agent id is valid");
        ui.add_agent(AgentListEntry::new(agent_id.clone(), "parser"));

        ui.set_agent_ready_state(&agent_id, true)
            .expect("test agent is registered");

        assert!(ui.render_screen("reply").starts_with('\u{7}'));
        assert!(!ui.render_screen("reply").contains('\u{7}'));
    }

    #[test]
    fn ready_agent_row_is_reversed_across_the_tui_left_pane() {
        let mut ui = TerminalUi::new(100, 24);
        let agent_id = AgentId::new("user-1").expect("test agent id is valid");
        ui.add_agent(AgentListEntry::new(agent_id, "parser").with_ready(true));

        let buffer = ui.render_tui_buffer("reply", "", 0);
        let left_width = ui.layout().left_width;

        for column in 1..left_width.saturating_sub(1) {
            assert!(
                buffer.get(column, 2).modifier.contains(Modifier::REVERSED),
                "column {column} on the ready agent row should be reversed"
            );
        }
    }
}
----- END FILE src/ui.rs -----

----- BEGIN FILE src/ui_harness.rs -----
digest: fnv64:7403655dc0614ae0; bytes:21942

use crate::{
    AgentId, AgentListEntry, PaneFocus, TerminalUi, UiKey, UiMode,
    chat_title::{ChatTitleAgent, fallback_chat_title_from_prompt},
};

#[derive(Debug)]
pub struct UiHarness {
    ui: TerminalUi,
    prompt_buffer: String,
    prompt_cursor: usize,
    prompt_history: Vec<String>,
    prompt_history_index: Option<usize>,
    prompt_history_draft: Option<String>,
    chat_buffer: String,
    chat_cursor: usize,
    chat_history: Vec<String>,
    chat_history_index: Option<usize>,
    chat_history_draft: Option<String>,
    escape_sequence: Option<PendingEscapeSequence>,
    transcript: Vec<String>,
    chat_title_agent: ChatTitleAgent,
    next_agent: usize,
    quit: bool,
}

impl UiHarness {
    pub fn new(width: u16, height: u16) -> Self {
        let parser = AgentId::new("user-1").expect("fixture agent id is valid");
        let tests = AgentId::new("user-2").expect("fixture agent id is valid");
        let mut chat_title_agent = ChatTitleAgent::new();
        chat_title_agent.mark_named(&parser);
        chat_title_agent.mark_named(&tests);

        Self {
            ui: fixture_ui(width, height, parser, tests),
            prompt_buffer: String::new(),
            prompt_cursor: 0,
            prompt_history: Vec::new(),
            prompt_history_index: None,
            prompt_history_draft: None,
            chat_buffer: String::new(),
            chat_cursor: 0,
            chat_history: Vec::new(),
            chat_history_index: None,
            chat_history_draft: None,
            escape_sequence: None,
            transcript: vec![
                "UI harness".to_string(),
                "Esc command, i insert, : prompt, Ctrl-W h/j/k/l focus, , toggle right, q quit"
                    .to_string(),
            ],
            chat_title_agent,
            next_agent: 3,
            quit: false,
        }
    }

    pub fn ui(&self) -> &TerminalUi {
        &self.ui
    }

    pub fn transcript(&self) -> &[String] {
        &self.transcript
    }

    pub fn mark_agent_ready(&mut self, agent_id: &str) -> Result<(), String> {
        let agent_id = AgentId::new(agent_id).map_err(|error| error.to_string())?;
        self.ui.set_agent_ready_state(&agent_id, true)
    }

    pub fn is_quit(&self) -> bool {
        self.quit
    }

    pub fn handle_bytes(&mut self, bytes: &[u8]) -> bool {
        let mut index = 0;
        while index < bytes.len() {
            if let Some((key, len)) = parse_key_sequence(&bytes[index..]) {
                self.handle_input(HarnessInput::Key(key));
                index += len;
            } else if !self.handle_byte(bytes[index]) {
                return false;
            } else {
                index += 1;
            }
        }
        self.finish_pending_escape_sequence();
        !self.quit
    }

    pub fn handle_byte(&mut self, byte: u8) -> bool {
        if self.quit {
            return false;
        }

        if self.continue_escape_sequence(byte) {
            return !self.quit;
        }

        if byte == 27 {
            let defer_escape = self.defer_escape_key();
            self.escape_sequence = Some(PendingEscapeSequence {
                bytes: vec![27],
                mode_before: self.ui.mode(),
                escape_dispatched: !defer_escape,
            });
            if !defer_escape {
                self.handle_input(HarnessInput::Key(UiKey::Esc));
            }
            return !self.quit;
        }

        let Some(input) = HarnessInput::from_byte(byte) else {
            return true;
        };
        self.handle_input(input);
        !self.quit
    }

    pub fn render_frame(&self) -> String {
        self.ui.render_screen_with_cursors(
            &self.right_content(),
            &self.prompt_buffer,
            self.prompt_cursor,
            Some(self.chat_cursor_column()),
        )
    }

    fn handle_input(&mut self, input: HarnessInput) {
        match input {
            HarnessInput::Quit => self.quit = true,
            HarnessInput::Interrupt => {
                self.ui.show_ctrl_c_exit_notice();
                if self.ui.focus() == PaneFocus::Right
                    && let Some(agent_id) = self.ui.selected_agent()
                {
                    self.transcript
                        .push(format!("work-leaf: sent Ctrl-C to {agent_id}"));
                }
            }
            HarnessInput::Backspace if self.ui.mode() == UiMode::Prompt => {
                self.backspace_prompt_char();
            }
            HarnessInput::Backspace if self.ui.mode() == UiMode::Insert => {
                self.backspace_chat_char();
            }
            HarnessInput::Enter if self.ui.mode() == UiMode::Prompt => {
                let line = self.prompt_buffer.trim().to_string();
                self.prompt_buffer.clear();
                self.prompt_cursor = 0;
                self.prompt_history_index = None;
                self.prompt_history_draft = None;
                self.ui.handle_key(UiKey::Esc);
                if !line.is_empty() {
                    self.prompt_history.push(line.clone());
                    self.transcript.push(format!("work-leaf> {line}"));
                    self.execute_prompt(&line);
                }
            }
            HarnessInput::Enter if self.ui.mode() == UiMode::Insert => {
                let message = self.chat_buffer.trim().to_string();
                self.chat_buffer.clear();
                self.chat_cursor = 0;
                self.chat_history_index = None;
                self.chat_history_draft = None;
                if !message.is_empty() {
                    self.chat_history.push(message.clone());
                    let target_agent = self.ui.selected_agent().cloned();
                    if let Some(agent_id) = target_agent.as_ref() {
                        self.name_chat_from_first_prompt(agent_id, &message);
                    }
                    let target = target_agent
                        .as_ref()
                        .map(AgentId::as_str)
                        .unwrap_or("work-leaf");
                    self.transcript.push(format!("{target}> {message}"));
                    self.transcript
                        .push("fixture reply: message recorded".to_string());
                }
            }
            HarnessInput::Char('/') if self.should_start_agent_slash_command() => {
                self.start_agent_slash_command();
            }
            HarnessInput::Char(ch) if self.ui.mode() == UiMode::Prompt => {
                self.insert_prompt_char(ch);
            }
            HarnessInput::Char(ch) if self.ui.mode() == UiMode::Insert => {
                self.insert_chat_char(ch);
            }
            HarnessInput::Key(UiKey::Left) if self.ui.mode() == UiMode::Prompt => {
                self.move_prompt_cursor_left();
            }
            HarnessInput::Key(UiKey::Right) if self.ui.mode() == UiMode::Prompt => {
                self.move_prompt_cursor_right();
            }
            HarnessInput::Key(UiKey::Up) if self.ui.mode() == UiMode::Prompt => {
                self.recall_prompt_history(-1);
            }
            HarnessInput::Key(UiKey::Down) if self.ui.mode() == UiMode::Prompt => {
                self.recall_prompt_history(1);
            }
            HarnessInput::Key(UiKey::Left) if self.should_route_chat_arrow() => {
                self.move_chat_cursor_left();
            }
            HarnessInput::Key(UiKey::Right) if self.should_route_chat_arrow() => {
                self.move_chat_cursor_right();
            }
            HarnessInput::Key(UiKey::Up) if self.should_route_chat_arrow() => {
                self.recall_chat_history(-1);
            }
            HarnessInput::Key(UiKey::Down) if self.should_route_chat_arrow() => {
                self.recall_chat_history(1);
            }
            HarnessInput::Key(UiKey::Esc) => {
                self.prompt_buffer.clear();
                self.prompt_cursor = 0;
                self.prompt_history_index = None;
                self.prompt_history_draft = None;
                let actions = self.ui.handle_key(UiKey::Esc);
                self.record_actions(actions);
            }
            HarnessInput::Key(key) => {
                let actions = self.ui.handle_key(key);
                self.record_actions(actions);
            }
            HarnessInput::Char(ch) => {
                let actions = self.ui.handle_key(UiKey::Char(ch));
                self.record_actions(actions);
            }
            HarnessInput::Backspace | HarnessInput::Enter => {}
        }
    }

    fn execute_prompt(&mut self, line: &str) {
        if is_agent_slash_command(line)
            && let Some(agent_id) = self.ui.selected_agent().cloned()
        {
            self.name_chat_from_first_prompt(&agent_id, line);
            self.transcript.push(format!("{agent_id}> {line}"));
            self.transcript
                .push("fixture reply: message recorded".to_string());
            return;
        }

        if matches!(line, "quit" | "exit" | "q") {
            self.quit = true;
            return;
        }

        let new_prompt = if line == "new" {
            Some("interactive task discovery")
        } else {
            line.strip_prefix("new ")
        };

        if let Some(prompt) = new_prompt {
            let agent_id = AgentId::new(format!("user-{}", self.next_agent))
                .expect("generated fixture id is valid");
            self.next_agent += 1;
            self.ui
                .add_agent(AgentListEntry::new(agent_id.clone(), "harness-agent"));
            self.ui
                .activate_agent_chat(&agent_id)
                .expect("generated fixture agent is registered");
            self.transcript
                .push(format!("agent {agent_id} launched for: {prompt}"));
            return;
        }

        match line {
            "help" | "?" => {
                self.transcript
                    .push("commands: new [prompt...], review, linearize, quit".into());
            }
            "review" => self.transcript.push("fixture review: no findings".into()),
            "linearize" => self
                .transcript
                .push("fixture linearize: keep user-1, keep user-2".into()),
            other => self
                .transcript
                .push(format!("unknown fixture command: {other}")),
        }
    }

    fn name_chat_from_first_prompt(&mut self, agent_id: &AgentId, prompt: &str) {
        if !self.chat_title_agent.reserve_first_prompt_title(agent_id) {
            return;
        }
        let title = fallback_chat_title_from_prompt(prompt);
        let _ = self.ui.set_agent_feature(agent_id, title);
    }

    fn insert_prompt_char(&mut self, ch: char) {
        self.prompt_buffer.insert(self.prompt_cursor, ch);
        self.prompt_cursor += ch.len_utf8();
        self.prompt_history_index = None;
        self.prompt_history_draft = None;
    }
    fn backspace_prompt_char(&mut self) {
        let Some((previous, _)) = self.prompt_buffer[..self.prompt_cursor]
            .char_indices()
            .next_back()
        else {
            return;
        };
        self.prompt_buffer.drain(previous..self.prompt_cursor);
        self.prompt_cursor = previous;
        self.prompt_history_index = None;
        self.prompt_history_draft = None;
    }
    fn move_prompt_cursor_left(&mut self) {
        if let Some((previous, _)) = self.prompt_buffer[..self.prompt_cursor]
            .char_indices()
            .next_back()
        {
            self.prompt_cursor = previous;
        }
    }
    fn move_prompt_cursor_right(&mut self) {
        if self.prompt_cursor >= self.prompt_buffer.len() {
            return;
        }
        let next = self.prompt_buffer[self.prompt_cursor..]
            .chars()
            .next()
            .map(|ch| self.prompt_cursor + ch.len_utf8())
            .unwrap_or(self.prompt_buffer.len());
        self.prompt_cursor = next;
    }
    fn recall_prompt_history(&mut self, delta: isize) {
        if self.prompt_history.is_empty() {
            return;
        }
        if self.prompt_history_index.is_none() {
            self.prompt_history_draft = Some(self.prompt_buffer.clone());
        }
        let current = self
            .prompt_history_index
            .unwrap_or(self.prompt_history.len()) as isize;
        let next = current + delta;
        if next < 0 {
            self.prompt_history_index = Some(0);
            self.prompt_buffer = self.prompt_history[0].clone();
        } else if next >= self.prompt_history.len() as isize {
            self.prompt_history_index = None;
            self.prompt_buffer = self.prompt_history_draft.take().unwrap_or_default();
        } else {
            let next = next as usize;
            self.prompt_history_index = Some(next);
            self.prompt_buffer = self.prompt_history[next].clone();
        }
        self.prompt_cursor = self.prompt_buffer.len();
    }
    fn insert_chat_char(&mut self, ch: char) {
        self.chat_buffer.insert(self.chat_cursor, ch);
        self.chat_cursor += ch.len_utf8();
        self.chat_history_index = None;
        self.chat_history_draft = None;
    }

    fn backspace_chat_char(&mut self) {
        let Some((previous, _)) = self.chat_buffer[..self.chat_cursor]
            .char_indices()
            .next_back()
        else {
            return;
        };
        self.chat_buffer.drain(previous..self.chat_cursor);
        self.chat_cursor = previous;
        self.chat_history_index = None;
        self.chat_history_draft = None;
    }

    fn move_chat_cursor_left(&mut self) {
        if let Some((previous, _)) = self.chat_buffer[..self.chat_cursor]
            .char_indices()
            .next_back()
        {
            self.chat_cursor = previous;
        }
    }

    fn move_chat_cursor_right(&mut self) {
        if self.chat_cursor >= self.chat_buffer.len() {
            return;
        }
        let next = self.chat_buffer[self.chat_cursor..]
            .chars()
            .next()
            .map(|ch| self.chat_cursor + ch.len_utf8())
            .unwrap_or(self.chat_buffer.len());
        self.chat_cursor = next;
    }

    fn recall_chat_history(&mut self, delta: isize) {
        if self.chat_history.is_empty() {
            return;
        }

        if self.chat_history_index.is_none() {
            self.chat_history_draft = Some(self.chat_buffer.clone());
        }

        let current = self.chat_history_index.unwrap_or(self.chat_history.len()) as isize;
        let next = current + delta;
        if next < 0 {
            self.chat_history_index = Some(0);
            self.chat_buffer = self.chat_history[0].clone();
        } else if next >= self.chat_history.len() as isize {
            self.chat_history_index = None;
            self.chat_buffer = self.chat_history_draft.take().unwrap_or_default();
        } else {
            let next = next as usize;
            self.chat_history_index = Some(next);
            self.chat_buffer = self.chat_history[next].clone();
        }
        self.chat_cursor = self.chat_buffer.len();
    }

    fn chat_cursor_column(&self) -> usize {
        CHAT_PROMPT.chars().count() + cursor_char_count(&self.chat_buffer, self.chat_cursor)
    }
    fn record_actions(&mut self, actions: Vec<crate::UiAction>) {
        self.transcript
            .extend(actions.into_iter().map(|action| format!("{action:?}")));
    }

    fn should_start_agent_slash_command(&self) -> bool {
        self.ui.mode() == UiMode::Command && self.ui.selected_agent().is_some()
    }

    fn start_agent_slash_command(&mut self) {
        let Some(agent_id) = self.ui.selected_agent().cloned() else {
            return;
        };
        if self.ui.activate_agent_chat(&agent_id).is_ok() {
            self.insert_chat_char('/');
        }
    }

    fn should_route_chat_arrow(&self) -> bool {
        self.ui.mode() == UiMode::Insert
            || (self.ui.mode() == UiMode::Command && self.ui.focus() == PaneFocus::Right)
    }

    fn defer_escape_key(&self) -> bool {
        self.ui.mode() == UiMode::Prompt
            || (self.ui.mode() == UiMode::Insert && self.ui.focus() == PaneFocus::Right)
    }

    fn finish_pending_escape_sequence(&mut self) {
        let should_finish = self
            .escape_sequence
            .as_ref()
            .is_some_and(|sequence| sequence.bytes.len() == 1);
        if should_finish {
            let sequence = self
                .escape_sequence
                .take()
                .expect("escape sequence is present");
            self.dispatch_pending_escape_if_needed(&sequence);
        }
    }

    fn dispatch_pending_escape_if_needed(&mut self, sequence: &PendingEscapeSequence) {
        if !sequence.escape_dispatched {
            self.handle_input(HarnessInput::Key(UiKey::Esc));
        }
    }

    fn continue_escape_sequence(&mut self, byte: u8) -> bool {
        let Some(sequence) = self.escape_sequence.as_mut() else {
            return false;
        };

        if sequence.bytes.len() == 1 && byte != b'[' {
            let sequence = self
                .escape_sequence
                .take()
                .expect("escape sequence is present");
            self.dispatch_pending_escape_if_needed(&sequence);
            return false;
        }

        sequence.bytes.push(byte);
        if let Some((key, len)) = parse_key_sequence(&sequence.bytes) {
            if len == sequence.bytes.len() {
                let sequence = self
                    .escape_sequence
                    .take()
                    .expect("escape sequence is present");
                if sequence.escape_dispatched
                    && sequence.mode_before == UiMode::Insert
                    && self.ui.mode() != UiMode::Insert
                {
                    let actions = self.ui.handle_key(UiKey::Char('i'));
                    self.record_actions(actions);
                }
                self.handle_input(HarnessInput::Key(key));
            }
        } else if sequence.bytes.len() > MAX_ESCAPE_SEQUENCE {
            let sequence = self
                .escape_sequence
                .take()
                .expect("escape sequence is present");
            self.dispatch_pending_escape_if_needed(&sequence);
        }

        true
    }

    fn right_content(&self) -> String {
        let mut content = self.transcript.join("\n");
        if !content.is_empty() {
            content.push('\n');
        }
        content.push_str(CHAT_PROMPT);
        content.push_str(&self.chat_buffer);
        content
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct PendingEscapeSequence {
    bytes: Vec<u8>,
    mode_before: UiMode,
    escape_dispatched: bool,
}

const CHAT_PROMPT: &str = "chat> ";
const MAX_ESCAPE_SEQUENCE: usize = 64;

fn parse_key_sequence(bytes: &[u8]) -> Option<(UiKey, usize)> {
    match bytes {
        [27, b'[', b'A', ..] => Some((UiKey::Up, 3)),
        [27, b'[', b'B', ..] => Some((UiKey::Down, 3)),
        [27, b'[', b'C', ..] => Some((UiKey::Right, 3)),
        [27, b'[', b'D', ..] => Some((UiKey::Left, 3)),
        _ => parse_sgr_mouse_sequence(bytes),
    }
}

fn parse_sgr_mouse_sequence(bytes: &[u8]) -> Option<(UiKey, usize)> {
    if !bytes.starts_with(b"\x1b[<") {
        return None;
    }

    let final_index = bytes.iter().position(|byte| matches!(byte, b'M' | b'm'))?;
    let final_byte = bytes[final_index];
    let body = std::str::from_utf8(&bytes[3..final_index]).ok()?;
    let mut parts = body.split(';');
    let button = parts.next()?.parse::<u16>().ok()?;
    let column = parts.next()?.parse::<u16>().ok()?;
    let row = parts.next()?.parse::<u16>().ok()?;
    if parts.next().is_some() {
        return None;
    }

    let button_kind = button & !0b0001_1100_u16;
    let key = match (button_kind, final_byte) {
        (64, b'M') => UiKey::MouseScrollUp { column, row },
        (65, b'M') => UiKey::MouseScrollDown { column, row },
        (_, b'M' | b'm') if button_kind < 64 && button & 0b11 == 0 => {
            UiKey::MouseClick { column, row }
        }
        _ => return None,
    };
    Some((key, final_index + 1))
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum HarnessInput {
    Key(UiKey),
    Char(char),
    Enter,
    Backspace,
    Interrupt,
    Quit,
}

impl HarnessInput {
    fn from_byte(byte: u8) -> Option<Self> {
        match byte {
            3 => Some(Self::Interrupt),
            4 => Some(Self::Quit),
            13 | 10 => Some(Self::Enter),
            27 => Some(Self::Key(UiKey::Esc)),
            23 => Some(Self::Key(UiKey::CtrlW)),
            8 | 127 => Some(Self::Backspace),
            byte if byte.is_ascii_graphic() || byte == b' ' => Some(Self::Char(byte as char)),
            _ => None,
        }
    }
}

fn cursor_char_count(text: &str, cursor: usize) -> usize {
    text.char_indices()
        .take_while(|(index, _)| *index < cursor)
        .count()
}

fn is_agent_slash_command(message: &str) -> bool {
    let mut chars = message.chars();
    matches!((chars.next(), chars.next()), (Some('/'), Some(next)) if !next.is_whitespace())
}

fn fixture_ui(width: u16, height: u16, parser: AgentId, tests: AgentId) -> TerminalUi {
    let mut ui = TerminalUi::new(width, height);
    ui.add_agent(
        AgentListEntry::new(parser.clone(), "parser")
            .with_ready(true)
            .with_modified_file("src/parser.rs")
            .with_conflicting_agent(tests.clone())
            .with_dependent(tests.clone()),
    );
    ui.add_agent(
        AgentListEntry::new(tests.clone(), "tests")
            .with_modified_file("tests/parser.rs")
            .with_dependency(parser.clone()),
    );
    ui.select_agent(&parser)
        .expect("fixture parser agent is registered");
    ui
}
----- END FILE src/ui_harness.rs -----

----- BEGIN FILE src/workspace.rs -----
digest: fnv64:001c1f08f5fa26bd; bytes:40263

use std::collections::{BTreeMap, BTreeSet};
use std::sync::mpsc::{self, Receiver, Sender};
use std::thread::{self, JoinHandle};
use std::time::{Duration, Instant};

use serde::{Deserialize, Serialize};

use crate::agent::{
    AgentBackend, AgentId, AgentKind, AgentLaunch, AgentShutdownHandle, AgentStreamEvent,
};
use crate::chat_title::{ChatTitleAgent, fallback_chat_title_from_prompt};
use crate::cli::{
    CliError, CommandChat, CommandChatResult, command_chat_error_text, command_result_text,
    render_command_chat_help,
};
use crate::review::{AgentCommit, GitHistory, ReviewResult};

#[derive(Debug)]
pub struct WorkLeafController<B>
where
    B: AgentBackend + Clone + Send + 'static,
{
    chat: Option<CommandChat<B>>,
    shutdown: AgentShutdownHandle,
    shutdown_on_drop: bool,
    workers: Vec<Worker>,
    command_transcript: Vec<String>,
    sessions: BTreeMap<AgentId, WorkLeafSession>,
    title_agent: ChatTitleAgent,
    pending_events: Vec<WorkLeafEvent>,
    reviewers: BTreeSet<AgentId>,
    review_commits_in_progress: BTreeMap<AgentId, String>,
    reviewed_agent_commits: BTreeMap<AgentId, String>,
    agent_review_baselines: BTreeMap<AgentId, String>,
}

impl<B> WorkLeafController<B>
where
    B: AgentBackend + Clone + Send + 'static,
{
    pub fn new(chat: CommandChat<B>) -> Self {
        let shutdown = chat.shutdown_handle();
        Self {
            chat: Some(chat),
            shutdown,
            shutdown_on_drop: true,
            workers: Vec::new(),
            command_transcript: vec![render_command_chat_help()],
            sessions: BTreeMap::new(),
            title_agent: ChatTitleAgent::new(),
            pending_events: Vec::new(),
            reviewers: BTreeSet::new(),
            review_commits_in_progress: BTreeMap::new(),
            reviewed_agent_commits: BTreeMap::new(),
            agent_review_baselines: BTreeMap::new(),
        }
    }

    pub fn into_chat(mut self) -> CommandChat<B> {
        self.wait_for_idle(Duration::from_secs(5));
        self.shutdown_on_drop = false;
        self.chat
            .take()
            .expect("work-leaf controller command chat is present")
    }

    pub fn transcript(&self) -> &[String] {
        &self.command_transcript
    }

    pub fn push_transcript_line(&mut self, line: impl Into<String>) {
        self.push_command_line(line.into());
    }

    pub fn snapshot(&self) -> WorkLeafSnapshot {
        WorkLeafSnapshot {
            command_transcript: self.command_transcript.clone(),
            sessions: self.sessions.values().cloned().collect(),
        }
    }

    pub fn drain_events(&mut self) -> Vec<WorkLeafEvent> {
        if self.pending_events.is_empty() {
            self.poll_worker();
        }
        self.pending_events.drain(..).collect()
    }

    pub fn is_busy(&mut self) -> bool {
        self.poll_worker();
        !self.workers.is_empty()
    }

    pub fn wait_for_idle(&mut self, timeout: Duration) -> bool {
        let start = Instant::now();
        while start.elapsed() < timeout {
            self.poll_worker();
            if self.workers.is_empty() {
                return true;
            }
            thread::sleep(Duration::from_millis(10));
        }
        self.poll_worker();
        self.workers.is_empty()
    }

    pub fn wait_for_session_line(
        &mut self,
        agent_id: &AgentId,
        needle: &str,
        timeout: Duration,
    ) -> bool {
        let start = Instant::now();
        while start.elapsed() < timeout {
            self.poll_worker();
            if self.session_contains(agent_id, needle) {
                return true;
            }
            thread::sleep(Duration::from_millis(10));
        }
        self.poll_worker();
        self.session_contains(agent_id, needle)
    }

    pub fn execute_command_line(&mut self, line: &str) {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            return;
        }
        self.push_command_line(format!("work-leaf> {trimmed}"));
        let parts = split_command_line(trimmed);
        let Some(command) = parts.first().map(String::as_str) else {
            return;
        };

        match command {
            "quit" | "exit" | "q" => self.request_quit(),
            "new" => {
                let prompt = parts[1..].join(" ");
                if let Err(error) = self.create_agent(prompt) {
                    self.push_command_line(command_chat_error_text(&error));
                }
            }
            "review" => {
                if let Err(error) = self.start_review() {
                    self.push_command_line(command_chat_error_text(&error));
                }
            }
            "linearize" => {
                if let Err(error) = self.start_linearize() {
                    self.push_command_line(command_chat_error_text(&error));
                }
            }
            _ => self.start_command_worker(trimmed.to_string()),
        }
    }

    pub fn send_command_agent_message(&mut self, message: &str) {
        let message = message.trim();
        if message.is_empty() {
            return;
        }

        self.push_command_line(format!("user: {message}"));
        let display_name = self.agent_display_name();
        if literal_command_line(message).is_none()
            && let Some(request) = command_agent_new_request(message)
        {
            self.push_command_line(format!(
                "command-agent: {}",
                command_agent_launch_reply(&display_name, &request)
            ));
            let command_line = command_agent_new_command_line(&request.prompt);
            for _ in 0..request.count {
                self.execute_command_line(&command_line);
            }
            return;
        }

        match command_agent_response(message, &display_name) {
            CommandAgentResponse::Execute {
                command_line,
                reply,
            } => {
                self.push_command_line(format!("command-agent: {reply}"));
                self.execute_command_line(&command_line);
            }
            CommandAgentResponse::Reply(reply) => {
                self.push_command_line(format!("command-agent: {reply}"));
            }
        }
    }

    pub fn interrupt_agent(&mut self, agent_id: &AgentId) {
        let display_name = self.agent_display_name();
        let result = self
            .chat
            .as_mut()
            .expect("work-leaf controller command chat is present")
            .interrupt_agent(agent_id);
        match result {
            Ok(()) => self.append_agent_line(
                agent_id,
                format!("work-leaf: sent Ctrl-C to {display_name}"),
            ),
            Err(error) => self.append_agent_line(agent_id, command_chat_error_text(&error)),
        }
    }

    pub fn create_agent(&mut self, prompt: impl Into<String>) -> Result<AgentId, CliError> {
        let prompt = prompt.into();
        let args = split_command_line(&prompt);
        let title_pending = args.is_empty();
        let launch = {
            let chat = self
                .chat
                .as_mut()
                .expect("work-leaf controller command chat is present");
            chat.prepare_agent_launch(&args)?
        };
        let agent_id = launch.id.clone();
        self.remember_agent_review_baseline(&agent_id);
        let title_prompt = self.reserve_launch_title_prompt(&launch, title_pending);
        let title = launch.feature.clone();
        self.register_agent_feature(agent_id.clone(), title.clone());
        self.add_session(WorkLeafSession {
            id: agent_id.clone(),
            kind: launch.kind.clone(),
            title,
            feature: launch.feature.clone(),
            lines: Vec::new(),
            loading: Some(WorkLeafLoading::Launching),
        });
        self.pending_events.push(WorkLeafEvent::AgentSelected {
            agent_id: agent_id.clone(),
        });
        if let Some(first_prompt) = title_prompt {
            self.start_title_worker(agent_id.clone(), first_prompt);
        }
        self.start_launch_worker(launch);
        Ok(agent_id)
    }

    pub fn send_message(&mut self, agent_id: &AgentId, message: &str) -> Result<(), CliError> {
        let message = message.trim();
        if message.is_empty() {
            return Ok(());
        }
        if self
            .sessions
            .get(agent_id)
            .and_then(|session| session.loading)
            .is_some()
        {
            self.append_agent_line(
                agent_id,
                format!("work-leaf: {} is still working", self.agent_display_name()),
            );
            return Ok(());
        }

        let title_prompt = self.reserve_first_chat_title_prompt(agent_id, message);
        self.set_session_loading(agent_id, Some(WorkLeafLoading::WaitingForReply));
        self.append_agent_line(agent_id, format!("user: {message}"));
        let agent_id = agent_id.clone();
        let message = message.to_string();
        if let Some(first_prompt) = title_prompt {
            self.start_title_worker(agent_id.clone(), first_prompt);
        }
        self.start_worker(move |mut chat, sender| {
            let stream_sender = sender.clone();
            let display_name = chat.agent_profile().display_name.clone();
            let mut stream = move |event_agent_id: &AgentId, event| {
                let _ = stream_sender.send(WorkerEvent::Stream {
                    agent_id: event_agent_id.clone(),
                    text: stream_event_text(event, &display_name),
                });
            };
            match chat.send_to_agent_streaming_with_ids(&agent_id, &message, &mut stream) {
                Ok(result) => {
                    let _ = sender.send(WorkerEvent::Complete {
                        agent_id: Some(agent_id),
                        result,
                    });
                }
                Err(error) => {
                    let _ = sender.send(WorkerEvent::Error {
                        agent_id: Some(agent_id),
                        message: command_chat_error_text(&error),
                    });
                }
            }
        });
        Ok(())
    }

    pub fn start_review(&mut self) -> Result<Vec<AgentId>, CliError> {
        self.start_review_for_agent(None)
    }

    fn start_review_for_patch_agent(
        &mut self,
        agent_id: &AgentId,
    ) -> Result<Vec<AgentId>, CliError> {
        self.start_review_for_agent(Some(agent_id))
    }

    fn start_review_for_agent(
        &mut self,
        target_agent_id: Option<&AgentId>,
    ) -> Result<Vec<AgentId>, CliError> {
        let (project_dir, agent_profile) = {
            let chat = self
                .chat
                .as_ref()
                .expect("work-leaf controller command chat is present");
            (
                chat.project_dir().to_path_buf(),
                chat.agent_profile().clone(),
            )
        };
        let empty_baselines = BTreeMap::new();
        let agent_baselines = if target_agent_id.is_some() {
            &self.agent_review_baselines
        } else {
            &empty_baselines
        };
        let commits = GitHistory::new(project_dir)
            .latest_agent_review_commits(&self.reviewed_agent_commits, agent_baselines)?;
        if commits.is_empty() {
            self.push_command_line("no agent commits found".to_string());
            return Ok(Vec::new());
        }

        let mut reviewer_ids = Vec::new();
        for commit in commits {
            if target_agent_id.is_some_and(|agent_id| &commit.agent_id != agent_id) {
                continue;
            }
            if self
                .reviewed_agent_commits
                .get(&commit.agent_id)
                .is_some_and(|hash| hash == &commit.hash)
                || self
                    .review_commits_in_progress
                    .get(&commit.agent_id)
                    .is_some_and(|hash| hash == &commit.hash)
            {
                continue;
            }
            let reviewer_id = AgentId::new(format!("review-{}", commit.agent_id.as_str()))
                .map_err(CliError::Agent)?;
            let reviewer_busy = self
                .sessions
                .get(&reviewer_id)
                .and_then(|session| session.loading)
                .is_some();
            if reviewer_busy {
                continue;
            }
            let session_exists = self.sessions.contains_key(&reviewer_id);
            let reuse_reviewer = self.reviewers.contains(&reviewer_id);
            if session_exists {
                self.set_session_loading(&reviewer_id, Some(WorkLeafLoading::WaitingForReply));
            } else {
                self.add_session(WorkLeafSession {
                    id: reviewer_id.clone(),
                    kind: agent_profile.kind.clone(),
                    title: format!("review {}", commit.feature),
                    feature: format!("review {}", commit.feature),
                    lines: Vec::new(),
                    loading: Some(WorkLeafLoading::WaitingForReply),
                });
            }
            if reviewer_ids.is_empty() {
                self.pending_events.push(WorkLeafEvent::AgentSelected {
                    agent_id: reviewer_id.clone(),
                });
            }
            self.review_commits_in_progress
                .insert(commit.agent_id.clone(), commit.hash.clone());
            self.start_review_worker(commit, reviewer_id.clone(), reuse_reviewer);
            reviewer_ids.push(reviewer_id);
        }
        Ok(reviewer_ids)
    }

    pub fn start_linearize(&mut self) -> Result<Option<AgentId>, CliError> {
        let launch = {
            let chat = self
                .chat
                .as_mut()
                .expect("work-leaf controller command chat is present");
            chat.prepare_linearize_launch()?
        };
        let Some(launch) = launch else {
            self.push_command_line("no reviewed agent commits found".to_string());
            return Ok(None);
        };

        let agent_id = launch.id.clone();
        let title = launch.feature.clone();
        self.register_agent_feature(agent_id.clone(), title.clone());
        self.add_session(WorkLeafSession {
            id: agent_id.clone(),
            kind: launch.kind.clone(),
            title: title.clone(),
            feature: title,
            lines: Vec::new(),
            loading: Some(WorkLeafLoading::Launching),
        });
        self.pending_events.push(WorkLeafEvent::AgentSelected {
            agent_id: agent_id.clone(),
        });
        self.start_launch_worker(launch);
        Ok(Some(agent_id))
    }

    pub fn loading_text(&self, loading: WorkLeafLoading) -> String {
        match loading {
            WorkLeafLoading::Launching => {
                format!("Starting {} session", self.agent_display_name())
            }
            WorkLeafLoading::WaitingForReply => {
                format!("Waiting for {}", self.agent_display_name())
            }
        }
    }

    pub fn shutdown(&mut self) {
        self.shutdown.shutdown();
    }

    fn poll_worker(&mut self) {
        let mut events = Vec::new();
        for worker in &self.workers {
            while let Ok(event) = worker.receiver.try_recv() {
                events.push(event);
            }
        }
        for event in events {
            self.apply_worker_event(event);
        }

        let mut index = 0;
        while index < self.workers.len() {
            if self.workers[index].handle.is_finished() {
                let worker = self.workers.swap_remove(index);
                while let Ok(event) = worker.receiver.try_recv() {
                    self.apply_worker_event(event);
                }
                worker
                    .handle
                    .join()
                    .expect("work-leaf worker did not panic");
            } else {
                index += 1;
            }
        }
    }

    fn session_contains(&self, agent_id: &AgentId, needle: &str) -> bool {
        self.sessions
            .get(agent_id)
            .is_some_and(|session| session.lines.iter().any(|line| line.contains(needle)))
    }

    fn reserve_launch_title_prompt(
        &mut self,
        launch: &AgentLaunch,
        title_pending: bool,
    ) -> Option<String> {
        if title_pending {
            None
        } else {
            self.title_agent.mark_named(&launch.id);
            Some(launch.prompt.clone())
        }
    }

    fn register_agent_feature(&mut self, agent_id: AgentId, feature: String) {
        if let Some(chat) = self.chat.as_mut() {
            chat.register_agent_feature(agent_id, feature);
        }
    }

    fn reserve_first_chat_title_prompt(
        &mut self,
        agent_id: &AgentId,
        prompt: &str,
    ) -> Option<String> {
        if !agent_id.as_str().starts_with("user-") {
            return None;
        }
        if !self.title_agent.reserve_first_prompt_title(agent_id) {
            return None;
        }
        Some(prompt.to_string())
    }

    fn remember_agent_review_baseline(&mut self, agent_id: &AgentId) {
        if !agent_id.as_str().starts_with("user-")
            || self.agent_review_baselines.contains_key(agent_id)
        {
            return;
        }
        let Some(root) = self
            .chat
            .as_ref()
            .map(|chat| chat.project_dir().to_path_buf())
        else {
            return;
        };
        if let Ok(Some(hash)) = GitHistory::new(root).head_hash() {
            self.agent_review_baselines.insert(agent_id.clone(), hash);
        }
    }

    fn add_session(&mut self, session: WorkLeafSession) {
        self.sessions.insert(session.id.clone(), session.clone());
        self.pending_events
            .push(WorkLeafEvent::AgentAdded { session });
    }

    fn start_launch_worker(&mut self, launch: AgentLaunch) {
        let agent_id = launch.id.clone();
        self.set_session_loading(&agent_id, Some(WorkLeafLoading::Launching));
        self.start_worker(move |mut chat, sender| {
            let stream_sender = sender.clone();
            let display_name = chat.agent_profile().display_name.clone();
            let mut stream = move |event_agent_id: &AgentId, event| {
                let _ = stream_sender.send(WorkerEvent::Stream {
                    agent_id: event_agent_id.clone(),
                    text: stream_event_text(event, &display_name),
                });
            };
            match chat.launch_prepared_agent_streaming_with_ids(launch, &mut stream) {
                Ok(result) => {
                    let _ = sender.send(WorkerEvent::Complete {
                        agent_id: Some(agent_id),
                        result,
                    });
                }
                Err(error) => {
                    let _ = sender.send(WorkerEvent::Error {
                        agent_id: Some(agent_id),
                        message: command_chat_error_text(&error),
                    });
                }
            }
        });
    }

    fn start_title_worker(&mut self, agent_id: AgentId, first_prompt: String) {
        self.start_worker(move |mut chat, sender| {
            let title = chat
                .generate_chat_title(&agent_id, &first_prompt)
                .unwrap_or_else(|_| fallback_chat_title_from_prompt(&first_prompt));
            let _ = sender.send(WorkerEvent::TitleGenerated { agent_id, title });
        });
    }

    fn start_review_worker(
        &mut self,
        commit: crate::review::AgentCommit,
        reviewer_id: AgentId,
        reuse_reviewer: bool,
    ) {
        self.start_worker(move |mut chat, sender| {
            let stream_sender = sender.clone();
            let display_name = chat.agent_profile().display_name.clone();
            let reviewed_agent_id = commit.agent_id.clone();
            let mut stream = move |event_agent_id: &AgentId, event| {
                let _ = stream_sender.send(WorkerEvent::Stream {
                    agent_id: event_agent_id.clone(),
                    text: stream_event_text(event, &display_name),
                });
            };
            match chat.review_commit_streaming_with_ids(
                commit,
                reviewer_id.clone(),
                reuse_reviewer,
                &mut stream,
            ) {
                Ok(result) => {
                    let _ = sender.send(WorkerEvent::Complete {
                        agent_id: Some(reviewer_id),
                        result: CommandChatResult::ReviewComplete(vec![result]),
                    });
                }
                Err(error) => {
                    let _ = sender.send(WorkerEvent::ReviewError {
                        reviewer_id,
                        reviewed_agent_id,
                        message: error.to_string(),
                    });
                }
            }
        });
    }

    fn start_command_worker(&mut self, line: String) {
        self.start_worker(move |mut chat, sender| match chat.handle_line(&line) {
            Ok(result) => {
                let _ = sender.send(WorkerEvent::Complete {
                    agent_id: None,
                    result,
                });
            }
            Err(error) => {
                let _ = sender.send(WorkerEvent::Error {
                    agent_id: None,
                    message: command_chat_error_text(&error),
                });
            }
        });
    }

    fn start_worker<F>(&mut self, operation: F)
    where
        F: FnOnce(CommandChat<B>, Sender<WorkerEvent>) + Send + 'static,
    {
        let Some(chat) = self.chat.as_ref().cloned() else {
            return;
        };
        let (sender, receiver) = mpsc::channel();
        let handle = thread::spawn(move || operation(chat, sender));
        self.workers.push(Worker { receiver, handle });
    }

    fn apply_worker_event(&mut self, event: WorkerEvent) {
        match event {
            WorkerEvent::Stream { agent_id, text } => {
                self.append_agent_line(&agent_id, text);
            }
            WorkerEvent::TitleGenerated { agent_id, title } => {
                self.apply_agent_title(&agent_id, title);
            }
            WorkerEvent::Complete { agent_id, result } => {
                if let Some(agent_id) = agent_id {
                    let start_review = should_start_review(&agent_id, &result);
                    self.set_session_loading(&agent_id, None);
                    self.apply_agent_result(&agent_id, &result);
                    if start_review && let Err(error) = self.start_review_for_patch_agent(&agent_id)
                    {
                        self.push_command_line(command_chat_error_text(&error));
                    }
                } else {
                    self.push_command_line(command_result_text(&result));
                    if matches!(result, CommandChatResult::Quit) {
                        self.request_quit();
                    }
                }
            }
            WorkerEvent::Error { agent_id, message } => {
                if let Some(agent_id) = agent_id {
                    self.set_session_loading(&agent_id, None);
                    self.append_agent_line(&agent_id, message);
                } else {
                    self.push_command_line(message);
                }
            }
            WorkerEvent::ReviewError {
                reviewer_id,
                reviewed_agent_id,
                message,
            } => {
                self.review_commits_in_progress.remove(&reviewed_agent_id);
                self.set_session_loading(&reviewer_id, None);
                self.append_agent_line(&reviewer_id, message);
            }
        }
    }

    fn apply_agent_result(&mut self, agent_id: &AgentId, result: &CommandChatResult) {
        match result {
            CommandChatResult::AgentLaunched { reply, .. }
            | CommandChatResult::AgentMessage { reply, .. } => {
                if !reply.is_empty() {
                    self.append_agent_line(agent_id, reply.clone());
                }
            }
            CommandChatResult::ReviewComplete(results) => {
                let text = command_result_text(result);
                self.push_command_line(text.clone());
                for review in results {
                    self.record_review_result(review);
                    self.append_agent_line(&review.commit.agent_id, format!("review: {text}"));
                }
            }
            other => {
                self.push_command_line(command_result_text(other));
            }
        }
    }

    fn apply_agent_title(&mut self, agent_id: &AgentId, title: String) {
        if let Some(session) = self.sessions.get_mut(agent_id) {
            session.title = title.clone();
            session.feature = title.clone();
            self.pending_events.push(WorkLeafEvent::AgentStatusUpdated {
                agent_id: session.id.clone(),
                kind: session.kind.clone(),
                title: session.title.clone(),
                feature: session.feature.clone(),
                loading: session.loading,
            });
        }
        self.register_agent_feature(agent_id.clone(), title);
    }

    fn append_agent_line(&mut self, agent_id: &AgentId, line: String) {
        if line.is_empty() {
            return;
        }
        let fallback_kind = self.agent_kind();
        if !self.sessions.contains_key(agent_id) {
            let session = WorkLeafSession::unknown(agent_id.clone(), fallback_kind);
            self.sessions.insert(agent_id.clone(), session.clone());
            self.pending_events
                .push(WorkLeafEvent::AgentAdded { session });
        }
        let session = self
            .sessions
            .get_mut(agent_id)
            .expect("session was inserted before appending a line");
        if session.lines.iter().any(|existing| existing == &line) {
            return;
        }
        session.lines.push(line.clone());
        self.pending_events.push(WorkLeafEvent::AgentLineAppended {
            agent_id: agent_id.clone(),
            line,
        });
    }

    fn record_review_result(&mut self, review: &ReviewResult) {
        self.review_commits_in_progress.remove(&review.agent_id);
        let latest_commit = self
            .latest_agent_review_commit(&review.agent_id)
            .unwrap_or_else(|| review.commit.clone());
        self.reviewed_agent_commits
            .insert(review.agent_id.clone(), latest_commit.hash.clone());
        self.agent_review_baselines
            .insert(review.agent_id.clone(), latest_commit.hash.clone());
        if let Some(chat) = self.chat.as_mut() {
            chat.mark_reviewed_agent_commit(latest_commit);
        }
        self.reviewers.insert(review.reviewer_id.clone());
    }

    fn latest_agent_review_commit(&self, agent_id: &AgentId) -> Option<AgentCommit> {
        let root = self
            .chat
            .as_ref()
            .map(|chat| chat.project_dir().to_path_buf())?;
        let boundary = self
            .reviewed_agent_commits
            .get(agent_id)
            .or_else(|| self.agent_review_baselines.get(agent_id))
            .map(String::as_str);
        GitHistory::new(root)
            .agent_review_commit(agent_id, boundary)
            .ok()?
    }

    fn set_session_loading(&mut self, agent_id: &AgentId, loading: Option<WorkLeafLoading>) {
        let fallback_kind = self.agent_kind();
        if !self.sessions.contains_key(agent_id) {
            let session = WorkLeafSession::unknown(agent_id.clone(), fallback_kind);
            self.sessions.insert(agent_id.clone(), session.clone());
            self.pending_events
                .push(WorkLeafEvent::AgentAdded { session });
        }
        let session = self
            .sessions
            .get_mut(agent_id)
            .expect("session was inserted before updating loading");
        session.loading = loading;
        self.pending_events.push(WorkLeafEvent::AgentStatusUpdated {
            agent_id: session.id.clone(),
            kind: session.kind.clone(),
            title: session.title.clone(),
            feature: session.feature.clone(),
            loading: session.loading,
        });
    }

    fn push_command_line(&mut self, line: String) {
        if line.is_empty() {
            return;
        }
        self.command_transcript.push(line.clone());
        self.pending_events
            .push(WorkLeafEvent::CommandTranscriptLine { line });
    }

    fn request_quit(&mut self) {
        self.shutdown.shutdown();
        self.pending_events.push(WorkLeafEvent::QuitRequested);
    }

    fn agent_display_name(&self) -> String {
        self.chat
            .as_ref()
            .map(|chat| chat.agent_profile().display_name.clone())
            .unwrap_or_else(|| "agent".to_string())
    }

    fn agent_kind(&self) -> AgentKind {
        self.chat
            .as_ref()
            .map(|chat| chat.agent_profile().kind.clone())
            .unwrap_or_else(|| AgentKind::External("agent".to_string()))
    }
}

impl<B> Drop for WorkLeafController<B>
where
    B: AgentBackend + Clone + Send + 'static,
{
    fn drop(&mut self) {
        if self.shutdown_on_drop {
            self.shutdown.shutdown();
        }
    }
}

fn should_start_review(agent_id: &AgentId, result: &CommandChatResult) -> bool {
    agent_id.as_str().starts_with("user-")
        && match result {
            CommandChatResult::AgentLaunched { reply, .. }
            | CommandChatResult::AgentMessage { reply, .. } => {
                contains_applied_patch(agent_id, reply) && contains_done_directive(reply)
            }
            _ => false,
        }
}

fn contains_applied_patch(agent_id: &AgentId, text: &str) -> bool {
    let prefix = format!("applied patch from {agent_id}:");
    text.lines()
        .any(|line| line.trim_start().starts_with(&prefix))
}

fn contains_done_directive(text: &str) -> bool {
    text.lines()
        .any(|line| line.trim_start() == "@work-leaf done")
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct WorkLeafSnapshot {
    pub command_transcript: Vec<String>,
    pub sessions: Vec<WorkLeafSession>,
}

impl WorkLeafSnapshot {
    pub fn session(&self, agent_id: &AgentId) -> Option<&WorkLeafSession> {
        self.sessions.iter().find(|session| &session.id == agent_id)
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct WorkLeafSession {
    pub id: AgentId,
    pub kind: AgentKind,
    pub title: String,
    pub feature: String,
    pub lines: Vec<String>,
    pub loading: Option<WorkLeafLoading>,
}

impl WorkLeafSession {
    fn unknown(agent_id: AgentId, kind: AgentKind) -> Self {
        Self {
            id: agent_id,
            kind,
            title: "agent".to_string(),
            feature: "agent".to_string(),
            lines: Vec::new(),
            loading: None,
        }
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub enum WorkLeafLoading {
    Launching,
    WaitingForReply,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub enum WorkLeafEvent {
    AgentAdded {
        session: WorkLeafSession,
    },
    AgentUpdated {
        session: WorkLeafSession,
    },
    AgentStatusUpdated {
        agent_id: AgentId,
        kind: AgentKind,
        title: String,
        feature: String,
        loading: Option<WorkLeafLoading>,
    },
    AgentLineAppended {
        agent_id: AgentId,
        line: String,
    },
    AgentSelected {
        agent_id: AgentId,
    },
    CommandTranscriptLine {
        line: String,
    },
    QuitRequested,
}

#[derive(Debug)]
struct Worker {
    receiver: Receiver<WorkerEvent>,
    handle: JoinHandle<()>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
enum WorkerEvent {
    Stream {
        agent_id: AgentId,
        text: String,
    },
    TitleGenerated {
        agent_id: AgentId,
        title: String,
    },
    Complete {
        agent_id: Option<AgentId>,
        result: CommandChatResult,
    },
    Error {
        agent_id: Option<AgentId>,
        message: String,
    },
    ReviewError {
        reviewer_id: AgentId,
        reviewed_agent_id: AgentId,
        message: String,
    },
}

fn stream_event_text(event: AgentStreamEvent, agent_display_name: &str) -> String {
    let label = agent_display_name.to_ascii_lowercase();
    match event {
        AgentStreamEvent::Status(text) => format!("{label}: {text}"),
        AgentStreamEvent::AgentMessage(text) => text,
        AgentStreamEvent::Error(text) => format!("{label} error: {text}"),
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
enum CommandAgentResponse {
    Execute { command_line: String, reply: String },
    Reply(String),
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct CommandAgentNewRequest {
    count: usize,
    prompt: String,
}

fn command_agent_response(message: &str, agent_display_name: &str) -> CommandAgentResponse {
    if let Some(command_line) = literal_command_line(message) {
        return CommandAgentResponse::Execute {
            reply: format!("running `{command_line}`"),
            command_line,
        };
    }

    let lower = message.to_ascii_lowercase();
    if asks_for_new_agent(&lower) {
        let prompt = command_agent_new_prompt(message);
        let command_line = if prompt.is_empty() {
            "new".to_string()
        } else {
            format!("new {prompt}")
        };
        let reply = if prompt.is_empty() {
            format!("launching {agent_display_name} user agent")
        } else {
            format!("launching {agent_display_name} user agent for {prompt}")
        };
        return CommandAgentResponse::Execute {
            command_line,
            reply,
        };
    }

    for (needle, command_line) in [
        ("linearize", "linearize"),
        ("linearise", "linearize"),
        ("review", "review"),
        ("help", "help"),
        ("quit", "quit"),
        ("exit", "quit"),
    ] {
        if lower.contains(needle) {
            return CommandAgentResponse::Execute {
                command_line: command_line.to_string(),
                reply: format!("running `{command_line}`"),
            };
        }
    }

    CommandAgentResponse::Reply(
        "I can run help, new [prompt...], review, linearize, or quit.".to_string(),
    )
}

fn literal_command_line(message: &str) -> Option<String> {
    let command = split_command_line(message).into_iter().next()?;
    matches!(
        command.as_str(),
        "help" | "?" | "new" | "review" | "linearize" | "quit" | "exit" | "q"
    )
    .then(|| message.to_string())
}

fn asks_for_new_agent(lower: &str) -> bool {
    lower.contains("agent")
        && ["new", "spawn", "create", "start", "launch"]
            .iter()
            .any(|verb| lower.contains(verb))
}

fn command_agent_new_request(message: &str) -> Option<CommandAgentNewRequest> {
    let lower = message.to_ascii_lowercase();
    if !asks_for_agent_launch_request(&lower) {
        return None;
    }

    let prompt = command_agent_launch_prompt(message);
    let count = agent_launch_count(&prompt).unwrap_or(1);
    let prompt = strip_agent_launch_count_and_noun(&prompt);
    Some(CommandAgentNewRequest {
        count,
        prompt: normalize_common_agent_typos(&prompt),
    })
}

fn asks_for_agent_launch_request(lower: &str) -> bool {
    lower.contains("agent")
        && ["new", "spawn", "create", "start", "launch", "open", "make"]
            .iter()
            .any(|verb| lower.contains(verb))
}

fn command_agent_launch_prompt(message: &str) -> String {
    let trimmed = strip_polite_prefix(message.trim());
    [
        "open a new ",
        "open new ",
        "open an ",
        "open a ",
        "open ",
        "spawn a new ",
        "spawn new ",
        "spawn an ",
        "spawn a ",
        "spawn ",
        "create a new ",
        "create new ",
        "create an ",
        "create a ",
        "create ",
        "start a new ",
        "start new ",
        "start an ",
        "start a ",
        "start ",
        "launch a new ",
        "launch new ",
        "launch an ",
        "launch a ",
        "launch ",
        "make a new ",
        "make new ",
        "make an ",
        "make a ",
        "make ",
        "new an ",
        "new a ",
        "new ",
    ]
    .iter()
    .find_map(|prefix| strip_ascii_prefix_case_insensitive(trimmed, prefix))
    .unwrap_or(trimmed)
    .to_string()
}

fn command_agent_new_command_line(prompt: &str) -> String {
    if prompt.is_empty() {
        "new".to_string()
    } else {
        format!("new {prompt}")
    }
}

fn command_agent_launch_reply(
    agent_display_name: &str,
    request: &CommandAgentNewRequest,
) -> String {
    let count_prefix = if request.count > 1 {
        format!("{} ", request.count)
    } else {
        String::new()
    };
    let agent_label = if request.count == 1 {
        "user agent"
    } else {
        "user agents"
    };

    if request.prompt.is_empty() {
        format!("launching {count_prefix}{agent_display_name} {agent_label}")
    } else {
        format!(
            "launching {count_prefix}{agent_display_name} {agent_label} for {}",
            request.prompt
        )
    }
}

fn agent_launch_count(text: &str) -> Option<usize> {
    text.split_whitespace().next().and_then(agent_count_word)
}

fn strip_agent_launch_count_and_noun(prompt: &str) -> String {
    let words = prompt.split_whitespace().collect::<Vec<_>>();
    let mut start = 0;
    let mut end = words.len();
    if words
        .first()
        .and_then(|word| agent_count_word(word))
        .is_some()
    {
        start = 1;
    }
    if words.last().is_some_and(|word| is_agent_noun(word)) {
        end -= 1;
    }
    words[start..end].join(" ")
}

fn agent_count_word(word: &str) -> Option<usize> {
    let clean = clean_agent_word(word);
    if let Ok(count) = clean.parse::<usize>() {
        return (count > 0).then_some(count);
    }

    match clean.as_str() {
        "a" | "an" | "one" => Some(1),
        "two" => Some(2),
        "three" => Some(3),
        "four" => Some(4),
        "five" => Some(5),
        "six" => Some(6),
        "seven" => Some(7),
        "eight" => Some(8),
        "nine" => Some(9),
        "ten" => Some(10),
        _ => None,
    }
}

fn is_agent_noun(word: &str) -> bool {
    matches!(clean_agent_word(word).as_str(), "agent" | "agents")
}

fn normalize_common_agent_typos(text: &str) -> String {
    text.split_whitespace()
        .map(|word| {
            if clean_agent_word(word) == "pacth" {
                "patch"
            } else {
                word
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

fn clean_agent_word(word: &str) -> String {
    word.trim_matches(|ch: char| !ch.is_ascii_alphanumeric())
        .to_ascii_lowercase()
}

fn command_agent_new_prompt(message: &str) -> String {
    let trimmed = strip_polite_prefix(message.trim());
    [
        "spawn a new ",
        "spawn new ",
        "create a new ",
        "create new ",
        "start a new ",
        "start new ",
        "launch a new ",
        "launch new ",
        "make a new ",
        "make new ",
        "new ",
    ]
    .iter()
    .find_map(|prefix| strip_ascii_prefix_case_insensitive(trimmed, prefix))
    .unwrap_or(trimmed)
    .to_string()
}

fn strip_polite_prefix(message: &str) -> &str {
    ["please ", "can you ", "could you ", "would you "]
        .iter()
        .find_map(|prefix| strip_ascii_prefix_case_insensitive(message, prefix))
        .unwrap_or(message)
}

fn strip_ascii_prefix_case_insensitive<'a>(message: &'a str, prefix: &str) -> Option<&'a str> {
    message
        .to_ascii_lowercase()
        .starts_with(prefix)
        .then(|| message[prefix.len()..].trim())
}

fn split_command_line(line: &str) -> Vec<String> {
    line.split_whitespace().map(str::to_string).collect()
}
----- END FILE src/workspace.rs -----

----- BEGIN FILE tests/terminal_app.rs -----
digest: fnv64:0cce6f91224ff9c9; bytes:43279

use std::collections::VecDeque;
use std::fs;
use std::path::PathBuf;
use std::process::Command;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};

use work_leaf::{
    AgentBackend, AgentError, AgentId, AgentLaunch, AgentSession, ChatMessage, CodexBackend,
    CodexCommandConfig, CommandChat, MessageRole, PaneFocus, PromptPolicy, TerminalApp, UiMode,
};

#[test]
fn terminal_app_new_and_chat_message_use_real_command_chat_backend() {
    let backend = FakeBackend::new(["launch reply", "follow reply"]);
    let chat = CommandChat::new(PathBuf::from("/repo"), backend);
    let mut app = TerminalApp::new(chat, 100, 24);

    app.handle_bytes(b":new implement parser\n");
    app.wait_for_idle(Duration::from_secs(1));

    assert_eq!(
        app.ui().selected_agent().map(AgentId::as_str),
        Some("user-1")
    );
    assert_eq!(app.ui().focus(), PaneFocus::Right);
    assert_eq!(app.ui().mode(), UiMode::Insert);
    assert!(app.render_frame().contains("user-1"));
    assert!(app.render_frame().contains("launch reply"));

    app.handle_bytes(b"please continue\n");
    app.wait_for_idle(Duration::from_secs(1));

    assert!(app.render_frame().contains("user: please continue"));
    assert!(app.render_frame().contains("follow reply"));
    let backend = app.into_chat().into_backend();
    let launches = backend.launches();
    let sends = backend.sends();
    assert_eq!(launches.len(), 1);
    assert_eq!(sends.len(), 1);
    assert_eq!(sends[0].0.as_str(), "user-1");
    assert_eq!(sends[0].1, "please continue");
}

#[test]
fn terminal_app_slash_command_from_chat_view_sends_agent_command() {
    let backend = FakeBackend::new(["launch reply", "status reply"]);
    let chat = CommandChat::new(PathBuf::from("/repo"), backend);
    let mut app = TerminalApp::new(chat, 100, 24);

    app.handle_bytes(b":new status command\n");
    app.wait_for_idle(Duration::from_secs(1));
    app.handle_bytes(b"\x1b/status\n");
    app.wait_for_idle(Duration::from_secs(1));

    assert_eq!(app.ui().mode(), UiMode::Insert);
    assert_eq!(app.ui().focus(), PaneFocus::Right);
    assert!(app.render_frame().contains("user: /status"));
    assert!(app.render_frame().contains("status reply"));

    let backend = app.into_chat().into_backend();
    let sends = backend.sends();
    assert_eq!(sends.len(), 1);
    assert_eq!(sends[0].0.as_str(), "user-1");
    assert_eq!(sends[0].1, "/status");
}

#[test]
fn terminal_app_prompt_slash_command_routes_to_selected_backend_agent() {
    let backend = FakeBackend::new(["launch reply", "status reply"]);
    let chat = CommandChat::new(PathBuf::from("/repo"), backend);
    let mut app = TerminalApp::new(chat, 100, 24);

    app.handle_bytes(b":new status command\n");
    app.wait_for_idle(Duration::from_secs(1));
    app.handle_bytes(b"\x1b:/status\n");
    app.wait_for_idle(Duration::from_secs(1));

    assert_eq!(app.ui().mode(), UiMode::Command);
    assert_eq!(app.ui().focus(), PaneFocus::Right);
    let frame = app.render_frame();
    assert!(frame.contains("user: /status"));
    assert!(frame.contains("status reply"));

    let backend = app.into_chat().into_backend();
    let sends = backend.sends();
    assert_eq!(sends.len(), 1);
    assert_eq!(sends[0].0.as_str(), "user-1");
    assert_eq!(sends[0].1, "/status");
}

#[test]
fn terminal_app_sends_spawned_codex_slash_command_as_raw_command() {
    let root = temp_dir("terminal-app-codex-slash-command");
    let fake_bin = root.join("bin");
    fs::create_dir_all(&fake_bin).unwrap();
    let codex = fake_bin.join("codex");
    fs::write(
        &codex,
        "\
#!/bin/sh
seen_resume=0
for arg in \"$@\"; do
  if [ \"$arg\" = \"resume\" ]; then
    seen_resume=1
  fi
done
input=$(cat)
if [ \"$seen_resume\" = \"1\" ]; then
  if [ \"$input\" = \"/status\" ]; then
    printf '%s\\n' '{\"type\":\"item.completed\",\"item\":{\"id\":\"status\",\"type\":\"agent_message\",\"text\":\"status command output\"}}'
  else
    printf '%s\\n' '{\"type\":\"item.completed\",\"item\":{\"id\":\"wrapped\",\"type\":\"agent_message\",\"text\":\"slash command was wrapped as prompt\"}}'
  fi
else
  printf '%s\\n' '{\"type\":\"thread.started\",\"thread_id\":\"thread-slash-command\"}'
  printf '%s\\n' '{\"type\":\"item.completed\",\"item\":{\"id\":\"launch\",\"type\":\"agent_message\",\"text\":\"launch reply from fake codex\"}}'
fi
",
    )
    .unwrap();
    make_executable(&codex);
    let backend = CodexBackend::new(
        CodexCommandConfig::new(root.clone()).with_binary(&codex),
        PromptPolicy::for_restricted_agents(),
    );
    let chat = CommandChat::new(root, backend);
    let mut app = TerminalApp::new(chat, 100, 24);

    app.handle_bytes(b":new status command\n");
    assert!(app.wait_for_idle(Duration::from_secs(1)));

    app.handle_bytes(b"\x1b/status\n");
    assert!(app.wait_for_idle(Duration::from_secs(1)));

    let frame = app.render_frame();
    assert!(frame.contains("user: /status"));
    assert!(frame.contains("status command output"));
    assert!(
        !frame.contains("slash command was wrapped as prompt"),
        "{frame}"
    );
}

#[test]
fn terminal_app_does_not_run_project_required_checks_outside_agent() {
    let root = temp_dir("terminal-app-no-required-check-run");
    fs::write(
        root.join("AGENTS.md"),
        "## Required Checks\n- `./check.sh`\n",
    )
    .unwrap();
    fs::write(
        root.join("check.sh"),
        "#!/bin/sh\necho compile failed\nexit 1\n",
    )
    .unwrap();
    let backend = FakeBackend::new(["launch reply"]);
    let chat = CommandChat::new(root, backend);
    let mut app = TerminalApp::new(chat, 100, 24);

    app.handle_bytes(b":new break compile\n");
    assert!(app.wait_for_idle(Duration::from_secs(1)));

    let frame = app.render_frame();
    assert!(frame.contains("launch reply"));
    assert!(frame.contains("READY"));
    assert!(!frame.contains("required check failed"));
    assert!(!frame.contains("compile failed"));
}

#[test]
fn terminal_app_command_agent_opens_multiple_patch_agents_from_chat_request() {
    let backend = FakeBackend::new([
        "launch reply",
        "launch reply",
        "launch reply",
        "launch reply",
    ]);
    let chat = CommandChat::new(PathBuf::from("/repo"), backend);
    let mut app = TerminalApp::new(chat, 100, 24);

    app.handle_bytes(b"iopen 4 pacth agents\n");
    assert!(app.wait_for_idle(Duration::from_secs(1)));

    assert_eq!(
        app.ui().selected_agent().map(AgentId::as_str),
        Some("user-4")
    );
    let frame = app.render_frame();
    assert!(frame.contains("user-4"));
    assert!(frame.contains("launch reply"));

    let backend = app.into_chat().into_backend();
    let launches = backend.launches();
    assert_eq!(launches.len(), 4);
    assert!(launches.iter().all(|launch| launch.prompt == "patch"));
}

#[test]
fn terminal_app_ctrl_c_never_quits_and_only_right_focus_interrupts_agent() {
    let backend = InterruptRecordingBackend::default();
    let chat = CommandChat::new(PathBuf::from("/repo"), backend.clone());
    let mut app = TerminalApp::new(chat, 100, 24);

    app.handle_bytes(b":new interruptible task\n");
    assert!(app.is_busy());
    assert_eq!(app.ui().focus(), PaneFocus::Right);

    assert!(app.handle_byte(3));
    assert!(!app.is_quit());
    assert_eq!(backend.interrupts(), vec![AgentId::new("user-1").unwrap()]);
    assert!(
        app.render_frame()
            .contains("work-leaf: sent Ctrl-C to Codex")
    );

    app.handle_bytes(&[27, 23, b'h']);
    assert_eq!(app.ui().focus(), PaneFocus::Left);
    assert!(app.handle_byte(3));
    assert!(!app.is_quit());
    assert_eq!(backend.interrupts(), vec![AgentId::new("user-1").unwrap()]);

    assert!(app.wait_for_idle(Duration::from_secs(1)));
}

#[test]
fn terminal_app_quits_only_from_prompt_q() {
    let backend = FakeBackend::from_replies(VecDeque::new());
    let chat = CommandChat::new(PathBuf::from("/repo"), backend);
    let mut app = TerminalApp::new(chat, 100, 24);

    assert!(app.handle_byte(3));
    assert!(!app.is_quit());
    assert!(app.handle_byte(b'q'));
    assert!(!app.is_quit());

    assert!(!app.handle_bytes(b":q\n"));
    assert!(app.is_quit());
}

#[test]
fn terminal_app_command_mode_typing_shows_insert_mode_notice() {
    let backend = FakeBackend::from_replies(VecDeque::new());
    let chat = CommandChat::new(PathBuf::from("/repo"), backend);
    let mut app = TerminalApp::new(chat, 100, 24);

    app.handle_bytes(b"hello");

    assert_eq!(app.ui().mode(), UiMode::Command);
    assert!(
        app.render_frame()
            .contains("command mode: press i for insert mode before typing")
    );
}

#[test]
fn terminal_app_ctrl_c_shows_quit_notice() {
    let backend = FakeBackend::from_replies(VecDeque::new());
    let chat = CommandChat::new(PathBuf::from("/repo"), backend);
    let mut app = TerminalApp::new(chat, 100, 24);

    assert!(app.handle_byte(3));
    assert!(!app.is_quit());
    assert!(
        app.render_frame()
            .contains("to exit, press Esc then :q then Enter")
    );
}

#[derive(Clone, Debug, Default)]
struct InterruptRecordingBackend {
    state: Arc<Mutex<InterruptRecordingState>>,
}

#[derive(Debug, Default)]
struct InterruptRecordingState {
    interrupts: Vec<AgentId>,
}

impl InterruptRecordingBackend {
    fn interrupts(&self) -> Vec<AgentId> {
        self.state.lock().unwrap().interrupts.clone()
    }
}

impl AgentBackend for InterruptRecordingBackend {
    fn launch(&mut self, request: AgentLaunch) -> Result<AgentSession, AgentError> {
        if request.id.as_str().starts_with("title-") {
            let mut session = AgentSession::new(request);
            session.push_message(MessageRole::Agent, "interruptible-task");
            return Ok(session);
        }

        thread::sleep(Duration::from_millis(250));
        let mut session = AgentSession::new(request);
        session.push_message(MessageRole::Agent, "finished without interrupt");
        Ok(session)
    }

    fn send(&mut self, _agent_id: &AgentId, _prompt: &str) -> Result<ChatMessage, AgentError> {
        Ok(ChatMessage::new(MessageRole::Agent, "follow-up"))
    }

    fn interrupt(&mut self, agent_id: &AgentId) -> Result<(), AgentError> {
        self.state.lock().unwrap().interrupts.push(agent_id.clone());
        Ok(())
    }
}

#[test]
fn terminal_app_keeps_visible_cursor_on_chat_input() {
    let backend = FakeBackend::new(["launch reply"]);
    let chat = CommandChat::new(PathBuf::from("/repo"), backend);
    let mut app = TerminalApp::new(chat, 100, 24);

    app.handle_bytes(b":new implement parser\nhello");

    assert_eq!(app.ui().focus(), PaneFocus::Right);
    assert!(app.render_frame().ends_with("\u{1b}[3;33H"));
}

#[test]
fn terminal_app_command_prompt_arrows_keep_visible_cursor_at_edit_position() {
    let backend = FakeBackend::from_replies(VecDeque::new());
    let chat = CommandChat::new(PathBuf::from("/repo"), backend);
    let mut app = TerminalApp::new(chat, 20, 10);

    app.handle_bytes(b":abcdefghijklmnopqrstuvwxyz0123\x1b[D\x1b[D\x1b[D\x1b[D\x1b[DX");

    assert_eq!(app.ui().mode(), UiMode::Prompt);
    let frame = app.render_frame();
    assert!(frame.contains(":ijklmnopqrstuvwxyXz"));
    assert!(frame.ends_with("\u{1b}[10;20H"));
}

#[test]
fn terminal_app_prompt_history_down_restores_in_progress_command() {
    let backend = FakeBackend::from_replies(VecDeque::new());
    let chat = CommandChat::new(PathBuf::from("/repo"), backend);
    let mut app = TerminalApp::new(chat, 80, 24);

    app.handle_bytes(b":help\n");
    app.handle_bytes(b":draft command\x1b[A");

    assert_eq!(app.ui().mode(), UiMode::Prompt);
    assert!(app.render_frame().contains(":help"));

    app.handle_bytes(b"\x1b[B");

    assert_eq!(app.ui().mode(), UiMode::Prompt);
    assert!(app.render_frame().contains(":draft command"));
}

#[test]
fn terminal_app_chat_arrows_keep_visible_cursor_at_edit_position() {
    let backend = FakeBackend::new(["launch reply"]);
    let chat = CommandChat::new(PathBuf::from("/repo"), backend);
    let mut app = TerminalApp::new(chat, 80, 24);

    app.handle_bytes(b":new cursor render\n");
    app.wait_for_idle(Duration::from_secs(1));
    app.handle_bytes(b"abcdefghijklmnopqrstuvwxyz0123\x1b[D\x1b[D\x1b[D\x1b[D\x1b[DX");

    let frame = app.render_frame();
    assert!(frame.contains("chat> abcdefghijklmnopqrstuvwxyXz0123"));
    assert!(frame.ends_with("\u{1b}[3;50H"));
}

#[test]
fn terminal_app_chat_history_down_restores_in_progress_message() {
    let backend = FakeBackend::new(["launch reply", "first reply", "draft reply"]);
    let chat = CommandChat::new(PathBuf::from("/repo"), backend);
    let mut app = TerminalApp::new(chat, 100, 24);

    app.handle_bytes(b":new chat history\n");
    app.wait_for_idle(Duration::from_secs(1));
    app.handle_bytes(b"first\n");
    app.wait_for_idle(Duration::from_secs(1));
    app.handle_bytes(b"draft message\x1b[A");
    assert!(app.render_frame().contains("chat> first"));

    app.handle_bytes(b"\x1b[B\n");
    app.wait_for_idle(Duration::from_secs(1));

    let backend = app.into_chat().into_backend();
    let sends = backend.sends();
    assert_eq!(
        sends
            .iter()
            .filter(|(_, prompt)| prompt == "draft message")
            .count(),
        1
    );
}

#[test]
fn terminal_app_bytewise_arrow_prefix_keeps_focused_chat_in_insert_mode() {
    let backend = FakeBackend::new(["launch reply", "follow reply"]);
    let chat = CommandChat::new(PathBuf::from("/repo"), backend);
    let mut app = TerminalApp::new(chat, 100, 24);

    app.handle_bytes(b":new arrow editing\n");
    app.wait_for_idle(Duration::from_secs(1));
    app.handle_bytes(b"ab");
    app.handle_byte(27);

    assert_eq!(app.ui().focus(), PaneFocus::Right);
    assert_eq!(app.ui().mode(), UiMode::Insert);
    assert!(app.render_frame().contains("mode=insert focus=right"));

    app.handle_byte(b'[');
    assert_eq!(app.ui().mode(), UiMode::Insert);

    app.handle_byte(b'D');
    app.handle_byte(b'Z');
    app.handle_byte(b'\n');
    app.wait_for_idle(Duration::from_secs(1));

    assert!(app.render_frame().contains("user: aZb"));
}

#[test]
fn terminal_app_chat_focus_arrows_edit_buffer_while_command_mode_is_active() {
    let backend = FakeBackend::new(["launch reply", "follow reply"]);
    let chat = CommandChat::new(PathBuf::from("/repo"), backend);
    let mut app = TerminalApp::new(chat, 100, 24);

    app.handle_bytes(b":new command arrow editing\n");
    app.wait_for_idle(Duration::from_secs(1));
    app.handle_bytes(b"ab\x1b");

    assert_eq!(app.ui().focus(), PaneFocus::Right);
    assert_eq!(app.ui().mode(), UiMode::Command);

    app.handle_bytes(b"\x1b[DiZ\n");
    app.wait_for_idle(Duration::from_secs(1));

    assert!(app.render_frame().contains("user: aZb"));
}

#[test]
fn terminal_app_chat_focus_arrows_recall_history_while_command_mode_is_active() {
    let backend = FakeBackend::new([
        "launch reply",
        "first reply",
        "second reply",
        "repeat reply",
    ]);
    let chat = CommandChat::new(PathBuf::from("/repo"), backend);
    let mut app = TerminalApp::new(chat, 100, 24);

    app.handle_bytes(b":new command history\n");
    app.wait_for_idle(Duration::from_secs(1));
    app.handle_bytes(b"first\n");
    app.wait_for_idle(Duration::from_secs(1));
    app.handle_bytes(b"second\n");
    app.wait_for_idle(Duration::from_secs(1));
    app.handle_bytes(b"\x1b");

    assert_eq!(app.ui().focus(), PaneFocus::Right);
    assert_eq!(app.ui().mode(), UiMode::Command);

    app.handle_bytes(b"\x1b[Ai\n");
    app.wait_for_idle(Duration::from_secs(1));

    let backend = app.into_chat().into_backend();
    let second_sends = backend
        .sends()
        .iter()
        .filter(|(_, prompt)| prompt == "second")
        .count();
    assert_eq!(second_sends, 2);
}

#[test]
fn terminal_app_new_and_chat_work_through_spawned_codex_backend() {
    let root = temp_dir("terminal-app-codex-backend");
    let fake_bin = root.join("bin");
    fs::create_dir_all(&fake_bin).unwrap();
    let codex = fake_bin.join("codex");
    fs::write(
        &codex,
        "\
#!/bin/sh
seen_exec=0
seen_resume=0
for arg in \"$@\"; do
  if [ \"$arg\" = \"exec\" ]; then
    seen_exec=1
  fi
  if [ \"$arg\" = \"resume\" ]; then
    seen_resume=1
  fi
  if [ \"$seen_exec\" = \"1\" ] && [ \"$arg\" = \"--ask-for-approval\" ]; then
    echo \"--ask-for-approval must be passed before exec\" >&2
    exit 42
  fi
done

if [ \"$seen_exec\" != \"1\" ]; then
  echo \"missing exec subcommand\" >&2
  exit 43
fi

if [ \"$seen_resume\" = \"1\" ]; then
  printf '%s\\n' '{\"type\":\"item.completed\",\"item\":{\"id\":\"item-2\",\"type\":\"agent_message\",\"text\":\"resume reply from fake codex\"}}'
else
  printf '%s\\n' '{\"type\":\"thread.started\",\"thread_id\":\"thread-user-1\"}'
  printf '%s\\n' '{\"type\":\"item.completed\",\"item\":{\"id\":\"item-1\",\"type\":\"agent_message\",\"text\":\"launch reply from fake codex\"}}'
fi
",
    )
    .unwrap();
    make_executable(&codex);
    let backend = CodexBackend::new(
        CodexCommandConfig::new(root.clone()).with_binary(&codex),
        PromptPolicy::for_restricted_agents(),
    );
    let chat = CommandChat::new(root, backend);
    let mut app = TerminalApp::new(chat, 100, 24);

    app.handle_bytes(b":new spawned process\n");
    app.wait_for_idle(Duration::from_secs(1));

    assert_eq!(
        app.ui().selected_agent().map(AgentId::as_str),
        Some("user-1")
    );
    assert!(app.render_frame().contains("user-1"));
    assert!(app.render_frame().contains("launch reply from fake codex"));

    app.handle_bytes(b"continue\n");
    app.wait_for_idle(Duration::from_secs(1));

    assert!(app.render_frame().contains("user: continue"));
    assert!(app.render_frame().contains("resume reply from fake codex"));
}

#[test]
fn terminal_app_does_not_clear_screen_on_each_render_or_drop_fast_input() {
    let backend = FakeBackend::new(["launch reply"]);
    let chat = CommandChat::new(PathBuf::from("/repo"), backend);
    let mut app = TerminalApp::new(chat, 100, 24);

    app.handle_bytes(b":new fast input\nabcdef");
    app.wait_for_idle(Duration::from_secs(1));

    let frame = app.render_frame();
    assert!(!frame.contains("\u{1b}[2J"));
    assert!(frame.contains("chat> abcdef"));
}

#[test]
fn terminal_app_keeps_chat_prompt_visible_after_large_agent_output() {
    let long_reply = (0..48)
        .map(|index| format!("agent-output-line-{index:02}"))
        .collect::<Vec<_>>()
        .join("\n");
    let backend = FakeBackend::from_replies(VecDeque::from([long_reply]));
    let chat = CommandChat::new(PathBuf::from("/repo"), backend);
    let mut app = TerminalApp::new(chat, 80, 12);

    app.handle_bytes(b":new large output\n");
    app.wait_for_idle(Duration::from_secs(1));

    let frame = app.render_frame();
    assert!(frame.contains("agent-output-line-47"));
    assert!(frame.contains("chat> "));

    app.handle_bytes(b"next question");
    assert!(app.render_frame().contains("chat> next question"));
}

#[test]
fn terminal_app_mouse_wheel_scrolls_chat_history() {
    let long_reply = (0..48)
        .map(|index| format!("agent-output-line-{index:02}"))
        .collect::<Vec<_>>()
        .join("\n");
    let backend = FakeBackend::from_replies(VecDeque::from([long_reply]));
    let chat = CommandChat::new(PathBuf::from("/repo"), backend);
    let mut app = TerminalApp::new(chat, 80, 12);

    app.handle_bytes(b":new large scroll\n");
    app.wait_for_idle(Duration::from_secs(1));

    let bottom_frame = app.render_frame();
    assert!(bottom_frame.contains("agent-output-line-47"));
    assert!(!bottom_frame.contains("agent-output-line-00"));
    assert!(bottom_frame.contains("chat> "));

    for _ in 0..16 {
        app.handle_bytes(b"\x1b[<64;20;3M");
    }

    let scrolled_frame = app.render_frame();
    assert!(scrolled_frame.contains("agent-output-line-00"));
    assert!(scrolled_frame.contains("chat> "));

    for _ in 0..16 {
        app.handle_bytes(b"\x1b[<65;20;3M");
    }

    let bottom_again = app.render_frame();
    assert!(bottom_again.contains("agent-output-line-47"));
    assert!(!bottom_again.contains("agent-output-line-00"));
    assert!(bottom_again.contains("chat> "));
}

#[test]
fn terminal_app_new_adds_agent_immediately_while_backend_is_loading() {
    let backend = SlowBackend;
    let chat = CommandChat::new(PathBuf::from("/repo"), backend);
    let mut app = TerminalApp::new(chat, 100, 24);
    let start = Instant::now();

    app.handle_bytes(b":new slow launch\n");

    assert!(start.elapsed() < Duration::from_millis(100));
    assert_eq!(
        app.ui().selected_agent().map(AgentId::as_str),
        Some("user-1")
    );
    assert!(app.render_frame().contains("user-1"));
    assert!(app.render_frame().contains("Starting Codex session"));

    app.wait_for_idle(Duration::from_secs(2));
    assert!(app.render_frame().contains("slow launch reply"));
}

#[test]
fn terminal_app_chat_pane_shows_only_the_selected_agent_session() {
    let backend = FakeBackend::new([
        "first launch",
        "second launch",
        "second reply",
        "first reply",
    ]);
    let chat = CommandChat::new(PathBuf::from("/repo"), backend);
    let mut app = TerminalApp::new(chat, 100, 24);

    app.handle_bytes(b":new first\n");
    app.wait_for_idle(Duration::from_secs(1));
    app.handle_bytes(b"\x1b:new second\n");
    app.wait_for_idle(Duration::from_secs(1));
    app.handle_bytes(b"message for second\n");
    app.wait_for_idle(Duration::from_secs(1));

    assert!(app.render_frame().contains("message for second"));
    app.handle_bytes(&[27, 23, b'h', b'k', b'l']);

    let frame = app.render_frame();
    assert!(frame.contains("first launch"));
    assert!(!frame.contains("message for second"));
    assert!(!frame.contains("second reply"));

    app.handle_bytes(b"imessage for first\n");
    app.wait_for_idle(Duration::from_secs(1));
    let frame = app.render_frame();
    assert!(frame.contains("message for first"));
    assert!(frame.contains("first reply"));
    assert!(!frame.contains("message for second"));
}

#[test]
fn terminal_app_left_pane_keyboard_selection_switches_the_visible_chat() {
    let backend = FakeBackend::new(["first launch", "second launch"]);
    let chat = CommandChat::new(PathBuf::from("/repo"), backend);
    let mut app = TerminalApp::new(chat, 100, 24);

    app.handle_bytes(b":new first\n");
    app.wait_for_idle(Duration::from_secs(1));
    app.handle_bytes(b"\x1b:new second\n");
    app.wait_for_idle(Duration::from_secs(1));

    assert_eq!(
        app.ui().selected_agent().map(AgentId::as_str),
        Some("user-2")
    );

    app.handle_bytes(&[27, 23, b'h', b'k']);

    assert_eq!(
        app.ui().selected_agent().map(AgentId::as_str),
        Some("user-1")
    );
    assert!(app.render_frame().contains("first launch"));
    assert!(!app.render_frame().contains("second launch"));

    app.handle_bytes(b"j");

    assert_eq!(
        app.ui().selected_agent().map(AgentId::as_str),
        Some("user-2")
    );
    assert!(app.render_frame().contains("second launch"));
    assert!(!app.render_frame().contains("first launch"));
}

#[test]
fn terminal_app_comma_collapses_left_pane_and_keeps_selected_chat_visible() {
    let backend = FakeBackend::new(["launch reply"]);
    let chat = CommandChat::new(PathBuf::from("/repo"), backend);
    let mut app = TerminalApp::new(chat, 100, 24);

    app.handle_bytes(b":new keep chat visible\n");
    app.wait_for_idle(Duration::from_secs(1));

    app.handle_bytes(b"\x1b,");

    let layout = app.ui().layout();
    assert_eq!(layout.left_width, 0);
    assert_eq!(layout.right_width, 100);
    assert!(app.render_frame().contains("launch reply"));
    assert!(app.render_frame().contains("chat> "));

    app.handle_bytes(b",");

    let layout = app.ui().layout();
    assert_eq!(layout.left_width, 20);
    assert_eq!(layout.right_width, 80);
    assert!(app.render_frame().contains("user-1"));
    assert!(app.render_frame().contains("launch reply"));
}

#[test]
fn terminal_app_sgr_mouse_click_on_left_agent_row_selects_that_chat() {
    let backend = FakeBackend::new(["first launch", "second launch"]);
    let chat = CommandChat::new(PathBuf::from("/repo"), backend);
    let mut app = TerminalApp::new(chat, 100, 24);

    app.handle_bytes(b":new first\n");
    app.wait_for_idle(Duration::from_secs(1));
    app.handle_bytes(b"\x1b:new second\n");
    app.wait_for_idle(Duration::from_secs(1));

    assert_eq!(
        app.ui().selected_agent().map(AgentId::as_str),
        Some("user-2")
    );

    app.handle_bytes(b"\x1b[<0;4;3M");

    assert_eq!(
        app.ui().selected_agent().map(AgentId::as_str),
        Some("user-1")
    );
    assert!(app.render_frame().contains("first launch"));
    assert!(!app.render_frame().contains("second launch"));
}

#[test]
fn terminal_app_sgr_mouse_release_on_left_agent_row_selects_that_chat() {
    let backend = FakeBackend::new(["first launch", "second launch"]);
    let chat = CommandChat::new(PathBuf::from("/repo"), backend);
    let mut app = TerminalApp::new(chat, 100, 24);

    app.handle_bytes(b":new first\n");
    app.wait_for_idle(Duration::from_secs(1));
    app.handle_bytes(b"\x1b:new second\n");
    app.wait_for_idle(Duration::from_secs(1));

    app.handle_bytes(b"\x1b[<0;4;3m");

    assert_eq!(
        app.ui().selected_agent().map(AgentId::as_str),
        Some("user-1")
    );
    assert!(app.render_frame().contains("first launch"));
    assert!(!app.render_frame().contains("second launch"));
}

#[test]
fn terminal_app_streams_spawned_codex_output_before_process_finishes() {
    let root = temp_dir("terminal-app-codex-streaming");
    let fake_bin = root.join("bin");
    fs::create_dir_all(&fake_bin).unwrap();
    let codex = fake_bin.join("codex");
    fs::write(
        &codex,
        "\
#!/bin/sh
printf '%s\\n' '{\"type\":\"thread.started\",\"thread_id\":\"thread-stream\"}'
printf '%s\\n' '{\"type\":\"error\",\"message\":\"streamed progress before final\"}'
sleep 1
printf '%s\\n' '{\"type\":\"item.completed\",\"item\":{\"id\":\"item-1\",\"type\":\"agent_message\",\"text\":\"streamed final reply\"}}'
",
    )
    .unwrap();
    make_executable(&codex);
    let backend = CodexBackend::new(
        CodexCommandConfig::new(root.clone()).with_binary(&codex),
        PromptPolicy::for_restricted_agents(),
    );
    let chat = CommandChat::new(root, backend);
    let mut app = TerminalApp::new(chat, 100, 24);

    app.handle_bytes(b":new stream output\n");

    assert!(app.wait_for_frame_contains("streamed progress before final", Duration::from_secs(1)));
    assert!(app.is_busy());
    app.wait_for_idle(Duration::from_secs(2));
    assert!(app.render_frame().contains("streamed final reply"));
}

#[test]
fn terminal_app_spawned_codex_processes_directive_message_before_later_prose_message() {
    let root = temp_dir("terminal-app-codex-directive-before-prose");
    fs::create_dir_all(root.join("src")).unwrap();
    fs::write(root.join("src/ui.rs"), "pub fn ui() {}\n").unwrap();
    fs::write(root.join("src/ui_harness.rs"), "pub fn harness() {}\n").unwrap();
    let fake_bin = root.join("bin");
    fs::create_dir_all(&fake_bin).unwrap();
    let codex = fake_bin.join("codex");
    fs::write(
        &codex,
        "\
#!/bin/sh
seen_resume=0
for arg in \"$@\"; do
  if [ \"$arg\" = \"resume\" ]; then
    seen_resume=1
  fi
done
input=$(cat)
if [ \"$seen_resume\" = \"1\" ]; then
  case \"$input\" in
    *\"work-leaf file text\"*)
      printf '%s\\n' '{\"type\":\"item.completed\",\"item\":{\"id\":\"item-follow\",\"type\":\"agent_message\",\"text\":\"I can patch after receiving src/ui.rs and src/ui_harness.rs.\"}}'
      ;;
    *)
      printf '%s\\n' '{\"type\":\"item.completed\",\"item\":{\"id\":\"item-bad\",\"type\":\"agent_message\",\"text\":\"missing orchestrator file text\"}}'
      ;;
  esac
else
  printf '%s\\n' '{\"type\":\"thread.started\",\"thread_id\":\"thread-directive-prose\"}'
  printf '%s\\n' '{\"type\":\"item.completed\",\"item\":{\"id\":\"item-directive\",\"type\":\"agent_message\",\"text\":\"@work-leaf read src/ui.rs src/ui_harness.rs\"}}'
  printf '%s\\n' '{\"type\":\"item.completed\",\"item\":{\"id\":\"item-prose\",\"type\":\"agent_message\",\"text\":\"I have requested the relevant UI and harness files from the orchestrator.\"}}'
fi
",
    )
    .unwrap();
    make_executable(&codex);
    let backend = CodexBackend::new(
        CodexCommandConfig::new(root.clone()).with_binary(&codex),
        PromptPolicy::for_restricted_agents(),
    );
    let chat = CommandChat::new(root, backend);
    let mut app = TerminalApp::new(chat, 100, 24);

    app.handle_bytes(b":new patch arrow keys\n");
    app.wait_for_idle(Duration::from_secs(1));

    let frame = app.render_frame();
    assert!(frame.contains("I have requested the relevant UI and harness files"));
    assert!(frame.contains("sent file text to user-1: src/ui.rs, src/ui_harness.rs"));
    assert!(frame.contains("I can patch after receiving src/ui.rs and src/ui_harness.rs"));
}

#[test]
fn terminal_app_answers_agent_file_requests_even_when_one_requested_path_is_missing() {
    let root = temp_dir("terminal-app-missing-agent-read");
    fs::write(root.join("Readme.md"), "work-leaf readme\n").unwrap();
    fs::write(root.join("Cargo.toml"), "[package]\nname = \"work-leaf\"\n").unwrap();
    let backend = FakeBackend::new([
        "@work-leaf read README.md Cargo.toml",
        "I can answer after receiving the available file text",
    ]);
    let chat = CommandChat::new(root, backend);
    let mut app = TerminalApp::new(chat, 100, 24);

    app.handle_bytes(b":new explain repo\n");
    app.wait_for_idle(Duration::from_secs(1));

    let frame = app.render_frame();
    assert!(frame.contains("reported unavailable file text to user-1"));
    assert!(frame.contains("sent file text to user-1: Cargo.toml"));
    assert!(frame.contains("I can answer after receiving the available file text"));

    let backend = app.into_chat().into_backend();
    let sends = backend.sends();
    assert_eq!(sends.len(), 1);
    assert!(sends[0].1.contains("Cargo.toml"));
    assert!(sends[0].1.contains("name = \"work-leaf\""));
    assert!(sends[0].1.contains("Unavailable file text"));
    assert!(sends[0].1.contains("README.md"));
}

#[test]
fn terminal_app_streams_automatic_agent_follow_up_output() {
    let root = temp_dir("terminal-app-streaming-agent-read");
    fs::create_dir_all(root.join("src")).unwrap();
    fs::write(root.join("src/lib.rs"), "pub fn value() -> u8 { 1 }\n").unwrap();
    let chat = CommandChat::new(root, StreamingDirectiveBackend);
    let mut app = TerminalApp::new(chat, 100, 24);

    app.handle_bytes(b":new inspect source\n");

    assert!(app.wait_for_frame_contains(
        "streamed answer from directive follow-up",
        Duration::from_secs(1)
    ));
    assert!(app.is_busy());
    app.wait_for_idle(Duration::from_secs(2));
    assert!(
        app.render_frame()
            .contains("final answer from directive follow-up")
    );
}

#[test]
fn terminal_app_sends_to_one_chat_while_another_chat_is_waiting_for_codex() {
    let backend = ConcurrentChatBackend::default();
    let chat = CommandChat::new(PathBuf::from("/repo"), backend);
    let mut app = TerminalApp::new(chat, 100, 24);

    app.handle_bytes(b":new first parser task\n");
    app.wait_for_idle(Duration::from_secs(1));
    app.handle_bytes(b"\x1b:new second docs task\n");
    app.wait_for_idle(Duration::from_secs(1));

    app.handle_bytes(b"slow question\n");
    assert!(app.is_busy());

    app.handle_bytes(&[27, 23, b'h', b'k', b'l']);
    app.handle_bytes(b"iquick question\n");

    assert!(app.wait_for_frame_contains("quick reply for user-1", Duration::from_millis(150)));
    assert!(
        !app.render_frame()
            .contains("work-leaf: Codex is still working")
    );
}

#[test]
fn terminal_app_review_adds_reviewer_chat_and_streams_output_immediately() {
    let root = git_repo("terminal-app-review-streams");
    fs::write(
        root.join("README.md"),
        "Agent-ID: user-1\nFeature: parser\nReason: parse configs\nContext: parser work",
    )
    .unwrap();
    Command::new("git")
        .current_dir(&root)
        .args(["add", "README.md"])
        .status()
        .unwrap();
    Command::new("git")
        .current_dir(&root)
        .args([
            "commit",
            "-m",
            "UPDATE apply parser patch from user-1",
            "-m",
            "Agent-ID: user-1\nFeature: parser\nReason: parse configs\nContext: parser work",
        ])
        .status()
        .unwrap();

    let chat = CommandChat::new(root, ReviewStreamingBackend);
    let mut app = TerminalApp::new(chat, 100, 24);

    app.handle_bytes(b":review\n");

    assert!(app.render_frame().contains("review-user-1"));
    assert!(app.wait_for_frame_contains(
        "reviewer streamed before finishing",
        Duration::from_millis(150)
    ));
}

#[test]
fn terminal_app_names_user_session_from_first_prompt_immediately() {
    let backend = FakeBackend::new(["launch reply"]);
    let chat = CommandChat::new(PathBuf::from("/repo"), backend);
    let mut app = TerminalApp::new(chat, 100, 24);

    app.handle_bytes(b":new implement parser combinator for config files\n");

    assert!(app.wait_for_idle(Duration::from_secs(1)));
    assert!(app.ui().render_left_pane().contains("parser-combinator"));
    let frame = app.render_frame();
    assert!(!frame.contains("working: user-agent"));
}

#[test]
fn terminal_app_names_interactive_chat_from_first_inserted_prompt_immediately() {
    let backend = FakeBackend::new(["launch reply", "follow reply"]);
    let chat = CommandChat::new(PathBuf::from("/repo"), backend);
    let mut app = TerminalApp::new(chat, 100, 24);

    app.handle_bytes(b":new\n");
    app.wait_for_idle(Duration::from_secs(1));
    let initial_left_pane = app.ui().render_left_pane();
    assert!(initial_left_pane.contains("user-1"));
    assert!(!initial_left_pane.contains("oauth redirect handler"));

    app.handle_bytes(b"please fix the OAuth redirect handler\n");

    assert!(app.wait_for_idle(Duration::from_secs(1)));
    let named_left_pane = app.ui().render_left_pane();
    assert!(named_left_pane.contains(">oauth-redirect-handler"));
    assert!(!named_left_pane.contains("oauth redirect handler"));
    assert!(!named_left_pane.contains("working: user-agent"));

    app.wait_for_idle(Duration::from_secs(1));
    let backend = app.into_chat().into_backend();
    assert_eq!(
        backend.sends()[0].1,
        "please fix the OAuth redirect handler"
    );
}

#[derive(Clone, Debug)]
struct FakeBackend {
    state: Arc<Mutex<FakeBackendState>>,
}

#[derive(Debug)]
struct FakeBackendState {
    replies: VecDeque<String>,
    launches: Vec<AgentLaunch>,
    sends: Vec<(AgentId, String)>,
}

#[derive(Clone, Debug)]
struct SlowBackend;

#[derive(Clone, Debug)]
struct StreamingDirectiveBackend;

#[derive(Clone, Debug, Default)]
struct ConcurrentChatBackend {
    state: Arc<Mutex<ConcurrentChatState>>,
}

#[derive(Debug, Default)]
struct ConcurrentChatState {
    launches: usize,
}

#[derive(Clone, Debug)]
struct ReviewStreamingBackend;

fn temp_dir(name: &str) -> PathBuf {
    let root = std::env::temp_dir().join(format!("work-leaf-{name}-{}", std::process::id()));
    let _ = fs::remove_dir_all(&root);
    fs::create_dir_all(&root).unwrap();
    root
}

#[cfg(unix)]
fn make_executable(path: &std::path::Path) {
    use std::os::unix::fs::PermissionsExt;

    let mut permissions = fs::metadata(path).unwrap().permissions();
    permissions.set_mode(0o755);
    fs::set_permissions(path, permissions).unwrap();
}

#[cfg(not(unix))]
fn make_executable(_path: &std::path::Path) {}

fn fake_title_from_title_prompt(prompt: &str) -> String {
    let first_prompt = prompt
        .rsplit_once("First prompt:\n")
        .map(|(_, first_prompt)| first_prompt)
        .unwrap_or(prompt);
    if first_prompt.contains("parser combinator") {
        "parser-combinator".to_string()
    } else if first_prompt.contains("OAuth redirect handler") {
        "oauth-redirect-handler".to_string()
    } else if first_prompt.contains("patch") {
        "patch".to_string()
    } else if first_prompt.contains("cursor render") {
        "cursor-render".to_string()
    } else if first_prompt.contains("chat history") {
        "chat-history".to_string()
    } else {
        "chat-title".to_string()
    }
}

impl FakeBackend {
    fn new<const N: usize>(replies: [&str; N]) -> Self {
        Self::from_replies(replies.into_iter().map(String::from).collect())
    }

    fn from_replies(replies: VecDeque<String>) -> Self {
        Self {
            state: Arc::new(Mutex::new(FakeBackendState {
                replies,
                launches: Vec::new(),
                sends: Vec::new(),
            })),
        }
    }

    fn launches(&self) -> Vec<AgentLaunch> {
        self.state.lock().unwrap().launches.clone()
    }

    fn sends(&self) -> Vec<(AgentId, String)> {
        self.state.lock().unwrap().sends.clone()
    }
}

impl AgentBackend for FakeBackend {
    fn launch(&mut self, request: AgentLaunch) -> Result<AgentSession, AgentError> {
        if request.id.as_str().starts_with("title-") {
            let mut session = AgentSession::new(request);
            let title = fake_title_from_title_prompt(&session.messages[0].text);
            session.push_message(MessageRole::Agent, title);
            return Ok(session);
        }

        let mut state = self.state.lock().unwrap();
        state.launches.push(request.clone());
        let mut session = AgentSession::new(request);
        session.push_message(
            MessageRole::Agent,
            state.replies.pop_front().expect("missing fake reply"),
        );
        Ok(session)
    }

    fn send(&mut self, agent_id: &AgentId, prompt: &str) -> Result<ChatMessage, AgentError> {
        let mut state = self.state.lock().unwrap();
        state.sends.push((agent_id.clone(), prompt.to_string()));
        Ok(ChatMessage::new(
            MessageRole::Agent,
            state.replies.pop_front().expect("missing fake reply"),
        ))
    }
}

impl AgentBackend for SlowBackend {
    fn launch(&mut self, request: AgentLaunch) -> Result<AgentSession, AgentError> {
        if request.id.as_str().starts_with("title-") {
            let mut session = AgentSession::new(request);
            let title = fake_title_from_title_prompt(&session.messages[0].text);
            session.push_message(MessageRole::Agent, title);
            return Ok(session);
        }

        thread::sleep(Duration::from_millis(250));
        let mut session = AgentSession::new(request);
        session.push_message(MessageRole::Agent, "slow launch reply");
        Ok(session)
    }

    fn send(&mut self, _agent_id: &AgentId, _prompt: &str) -> Result<ChatMessage, AgentError> {
        thread::sleep(Duration::from_millis(250));
        Ok(ChatMessage::new(MessageRole::Agent, "slow send reply"))
    }
}

impl AgentBackend for StreamingDirectiveBackend {
    fn launch(&mut self, request: AgentLaunch) -> Result<AgentSession, AgentError> {
        if request.id.as_str().starts_with("title-") {
            let mut session = AgentSession::new(request);
            let title = fake_title_from_title_prompt(&session.messages[0].text);
            session.push_message(MessageRole::Agent, title);
            return Ok(session);
        }

        let mut session = AgentSession::new(request);
        session.push_message(MessageRole::Agent, "@work-leaf read src/lib.rs");
        Ok(session)
    }

    fn send(&mut self, _agent_id: &AgentId, _prompt: &str) -> Result<ChatMessage, AgentError> {
        Ok(ChatMessage::new(
            MessageRole::Agent,
            "final answer from directive follow-up",
        ))
    }

    fn send_streaming(
        &mut self,
        _agent_id: &AgentId,
        _prompt: &str,
        sink: &mut dyn FnMut(work_leaf::AgentStreamEvent),
    ) -> Result<ChatMessage, AgentError> {
        sink(work_leaf::AgentStreamEvent::AgentMessage(
            "streamed answer from directive follow-up".to_string(),
        ));
        thread::sleep(Duration::from_millis(250));
        Ok(ChatMessage::new(
            MessageRole::Agent,
            "final answer from directive follow-up",
        ))
    }
}

impl AgentBackend for ConcurrentChatBackend {
    fn launch(&mut self, request: AgentLaunch) -> Result<AgentSession, AgentError> {
        if request.id.as_str().starts_with("title-") {
            let mut session = AgentSession::new(request);
            let title = fake_title_from_title_prompt(&session.messages[0].text);
            session.push_message(MessageRole::Agent, title);
            return Ok(session);
        }

        self.state.lock().unwrap().launches += 1;
        let mut session = AgentSession::new(request);
        session.push_message(MessageRole::Agent, "ready");
        Ok(session)
    }

    fn send(&mut self, agent_id: &AgentId, _prompt: &str) -> Result<ChatMessage, AgentError> {
        if agent_id.as_str() == "user-2" {
            thread::sleep(Duration::from_millis(350));
            return Ok(ChatMessage::new(
                MessageRole::Agent,
                "slow reply for user-2",
            ));
        }
        Ok(ChatMessage::new(
            MessageRole::Agent,
            "quick reply for user-1",
        ))
    }
}

impl AgentBackend for ReviewStreamingBackend {
    fn launch(&mut self, request: AgentLaunch) -> Result<AgentSession, AgentError> {
        if request.id.as_str().starts_with("title-") {
            let mut session = AgentSession::new(request);
            let title = fake_title_from_title_prompt(&session.messages[0].text);
            session.push_message(MessageRole::Agent, title);
            return Ok(session);
        }

        let mut session = AgentSession::new(request);
        session.push_message(MessageRole::Agent, "NO_FINDINGS");
        Ok(session)
    }

    fn send(&mut self, _agent_id: &AgentId, _prompt: &str) -> Result<ChatMessage, AgentError> {
        Ok(ChatMessage::new(MessageRole::Agent, "summary"))
    }

    fn launch_streaming(
        &mut self,
        request: AgentLaunch,
        sink: &mut dyn FnMut(work_leaf::AgentStreamEvent),
    ) -> Result<AgentSession, AgentError> {
        sink(work_leaf::AgentStreamEvent::AgentMessage(
            "reviewer streamed before finishing".to_string(),
        ));
        thread::sleep(Duration::from_millis(350));
        self.launch(request)
    }
}

fn git_repo(name: &str) -> PathBuf {
    let root = temp_dir(name);
    Command::new("git")
        .current_dir(&root)
        .args(["init", "-q"])
        .status()
        .unwrap();
    Command::new("git")
        .current_dir(&root)
        .args(["config", "user.email", "test@example.com"])
        .status()
        .unwrap();
    Command::new("git")
        .current_dir(&root)
        .args(["config", "user.name", "Test User"])
        .status()
        .unwrap();
    root
}
----- END FILE tests/terminal_app.rs -----

----- BEGIN FILE tests/ui_harness.rs -----
digest: fnv64:4b9c7837fd221a86; bytes:16550

use work_leaf::{PaneFocus, UiHarness, UiMode};

#[test]
fn scripted_harness_renders_full_width_crlf_frame() {
    let harness = UiHarness::new(80, 24);
    let rendered = harness.render_frame();
    let lines = rendered.split("\r\n").collect::<Vec<_>>();

    assert!(rendered.starts_with("\u{1b}[H"));
    assert!(!rendered.contains("\u{1b}[2J"));
    assert!(rendered.contains("UI harness"));
    assert!(rendered.contains("user-1"));
    assert_eq!(lines.len(), 24);
    assert!(
        lines
            .iter()
            .all(|line| strip_ansi(line).chars().count() == 80)
    );
}

#[test]
fn scripted_harness_rings_and_highlights_ready_chat() {
    let mut harness = UiHarness::new(100, 24);

    assert!(!harness.render_frame().starts_with('\u{7}'));

    harness
        .mark_agent_ready("user-2")
        .expect("fixture user-2 agent is registered");

    let ready_frame = harness.render_frame();
    assert!(ready_frame.starts_with('\u{7}'));
    assert!(ready_frame.contains("\u{1b}[7m test user-2 READY\u{1b}[0m"));
    assert!(!harness.render_frame().starts_with('\u{7}'));
}

#[test]
fn scripted_harness_switches_modes_without_enter() {
    let mut harness = UiHarness::new(80, 24);

    harness.handle_byte(b'i');
    assert_eq!(harness.ui().mode(), UiMode::Insert);

    harness.handle_byte(27);
    assert_eq!(harness.ui().mode(), UiMode::Command);

    harness.handle_byte(b':');
    assert_eq!(harness.ui().mode(), UiMode::Prompt);
    assert!(harness.render_frame().ends_with("\u{1b}[24;2H"));

    harness.handle_byte(b'n');
    assert_eq!(harness.ui().mode(), UiMode::Prompt);
    assert!(harness.render_frame().ends_with("\u{1b}[24;3H"));
}

#[test]
fn scripted_harness_prompt_arrow_keys_move_visible_cursor() {
    let mut harness = UiHarness::new(80, 24);
    harness.handle_bytes(b":ab\x1b[D");
    assert_eq!(harness.ui().mode(), UiMode::Prompt);
    assert!(harness.render_frame().ends_with("\u{1b}[24;3H"));
    harness.handle_bytes(b"\x1b[C");
    assert_eq!(harness.ui().mode(), UiMode::Prompt);
    assert!(harness.render_frame().ends_with("\u{1b}[24;4H"));
}

#[test]
fn scripted_harness_long_prompt_arrows_keep_rendered_cursor_at_edit_position() {
    let mut harness = UiHarness::new(20, 10);

    harness.handle_bytes(b":abcdefghijklmnopqrstuvwxyz0123\x1b[D\x1b[D\x1b[D\x1b[D\x1b[DX");

    let frame = harness.render_frame();
    assert!(frame.contains(":ijklmnopqrstuvwxyXz"));
    assert!(frame.ends_with("\u{1b}[10;20H"));

    harness.handle_bytes(b"\n");

    assert!(
        harness
            .transcript()
            .iter()
            .any(|line| line == "unknown fixture command: abcdefghijklmnopqrstuvwxyXz0123")
    );
}
#[test]
fn scripted_harness_prompt_arrow_keys_recall_prompt_history() {
    let mut harness = UiHarness::new(80, 24);
    harness.handle_bytes(b":review\n:linearize\n:\x1b[A\x1b[A\x1b[B\n");
    assert_eq!(harness.ui().mode(), UiMode::Command);
    assert_eq!(
        harness
            .transcript()
            .iter()
            .filter(|line| line.as_str() == "work-leaf> linearize")
            .count(),
        2
    );
}

#[test]
fn scripted_harness_prompt_history_down_restores_in_progress_prompt() {
    let mut harness = UiHarness::new(80, 24);

    harness.handle_bytes(b":review\n:draft command\x1b[A\x1b[B\n");

    assert!(
        harness
            .transcript()
            .iter()
            .any(|line| line == "unknown fixture command: draft command")
    );
}

#[test]
fn scripted_harness_bytewise_prompt_arrow_keys_edit_without_leaving_prompt() {
    let mut harness = UiHarness::new(80, 24);
    harness.handle_byte(b':');
    harness.handle_byte(b'a');
    harness.handle_byte(b'b');
    harness.handle_byte(27);
    assert_eq!(harness.ui().mode(), UiMode::Prompt);
    harness.handle_byte(b'[');
    assert_eq!(harness.ui().mode(), UiMode::Prompt);
    harness.handle_byte(b'D');
    harness.handle_byte(b'Z');
    harness.handle_byte(b'\n');
    assert_eq!(harness.ui().mode(), UiMode::Command);
    assert!(
        harness
            .transcript()
            .iter()
            .any(|line| line == "unknown fixture command: aZb")
    );
}
#[test]
fn scripted_harness_drives_ctrl_w_navigation_and_left_toggle() {
    let mut harness = UiHarness::new(80, 24);

    harness.handle_bytes(&[23, b'l']);
    assert_eq!(harness.ui().focus(), PaneFocus::Right);
    assert!(harness.render_frame().ends_with("\u{1b}[5;24H"));

    harness.handle_bytes(&[23, b'h']);
    assert_eq!(harness.ui().focus(), PaneFocus::Left);
    assert!(harness.render_frame().ends_with("\u{1b}[3;2H"));

    harness.handle_byte(b',');
    assert_eq!(harness.ui().layout().left_width, 0);
    assert_eq!(harness.ui().layout().right_width, 80);
    assert!(harness.ui().layout().right_surface.is_some());
    assert_eq!(harness.ui().focus(), PaneFocus::Right);

    harness.handle_byte(b',');
    assert_eq!(harness.ui().layout().left_width, 16);
    assert_eq!(harness.ui().focus(), PaneFocus::Left);
    harness.handle_bytes(&[23, b'j']);
    assert_eq!(harness.ui().focus(), PaneFocus::Right);

    harness.handle_bytes(&[23, b'k']);
    assert_eq!(harness.ui().focus(), PaneFocus::Left);
}

#[test]
fn scripted_harness_ctrl_c_never_quits_and_only_right_focus_interrupts_agent() {
    let mut harness = UiHarness::new(80, 24);

    assert_eq!(harness.ui().focus(), PaneFocus::Left);
    assert!(harness.handle_byte(3));
    assert!(!harness.is_quit());
    assert!(
        !harness
            .transcript()
            .iter()
            .any(|line| line.contains("sent Ctrl-C"))
    );

    harness.handle_bytes(&[23, b'l']);
    assert_eq!(harness.ui().focus(), PaneFocus::Right);
    assert!(harness.handle_byte(3));
    assert!(!harness.is_quit());
    assert!(
        harness
            .transcript()
            .iter()
            .any(|line| line == "work-leaf: sent Ctrl-C to user-1")
    );
}

#[test]
fn scripted_harness_command_mode_typing_shows_insert_mode_notice() {
    let mut harness = UiHarness::new(80, 24);

    harness.handle_bytes(b"hello");

    assert_eq!(harness.ui().mode(), UiMode::Command);
    assert!(
        harness
            .render_frame()
            .contains("command mode: press i for insert mode before typing")
    );
}

#[test]
fn scripted_harness_ctrl_c_shows_quit_notice() {
    let mut harness = UiHarness::new(80, 24);

    assert!(harness.handle_byte(3));
    assert!(!harness.is_quit());
    assert!(
        harness
            .render_frame()
            .contains("to exit, press Esc then :q then Enter")
    );
}

#[test]
fn scripted_harness_structural_command_keys_do_not_show_typing_notice() {
    let mut harness = UiHarness::new(80, 24);

    harness.handle_bytes(&[23, b'l']);

    assert_eq!(harness.ui().focus(), PaneFocus::Right);
    assert!(!harness.render_frame().contains("command mode: press i"));
}

#[test]
fn scripted_harness_left_pane_navigation_does_not_show_typing_notice() {
    let mut harness = UiHarness::new(80, 24);

    harness.handle_bytes(b"jjjjj");

    assert_eq!(harness.ui().focus(), PaneFocus::Left);
    assert_eq!(
        harness.ui().selected_agent().map(|id| id.as_str()),
        Some("user-2")
    );
    assert!(!harness.render_frame().contains("command mode: press i"));
}

#[test]
fn scripted_harness_quits_only_through_colon_q() {
    let mut harness = UiHarness::new(80, 24);

    assert!(harness.handle_byte(b'q'));
    assert!(!harness.is_quit());

    assert!(!harness.handle_bytes(b":q\n"));
    assert!(harness.is_quit());
}

#[test]
fn scripted_harness_new_commands_select_new_agent_chat() {
    let mut harness = UiHarness::new(80, 24);

    harness.handle_bytes(b":new ui automation\n");

    assert_eq!(
        harness.ui().selected_agent().map(|id| id.as_str()),
        Some("user-3")
    );
    assert_eq!(harness.ui().focus(), PaneFocus::Right);
    assert_eq!(harness.ui().mode(), UiMode::Insert);
    assert!(
        harness
            .transcript()
            .iter()
            .any(|line| line.contains("agent user-3 launched for: ui automation"))
    );
    assert!(harness.render_frame().contains("user-3"));

    harness.handle_bytes(b"\x1b:new\n");

    assert_eq!(
        harness.ui().selected_agent().map(|id| id.as_str()),
        Some("user-4")
    );
    assert_eq!(harness.ui().focus(), PaneFocus::Right);
    assert_eq!(harness.ui().mode(), UiMode::Insert);
}

#[test]
fn scripted_harness_names_new_chat_from_first_inserted_prompt() {
    let mut harness = UiHarness::new(80, 24);

    harness.handle_bytes(b":new\n");
    assert!(
        harness
            .ui()
            .render_left_pane()
            .contains(">harness-agent user-3  working: harness-agent")
    );

    harness.handle_bytes(b"please fix the OAuth redirect handler\n");

    let named_left_pane = harness.ui().render_left_pane();
    assert!(named_left_pane.contains(
        ">please-fix-the-oauth-redirect-handler user-3  working: please-fix-the-oauth-redirect-handler"
    ));
    assert!(!named_left_pane.contains("harness-agent user-3"));

    harness.handle_bytes(b"add cookie coverage\n");

    let unchanged_left_pane = harness.ui().render_left_pane();
    assert!(unchanged_left_pane.contains(
        ">please-fix-the-oauth-redirect-handler user-3  working: please-fix-the-oauth-redirect-handler"
    ));
    assert!(!unchanged_left_pane.contains("add-cookie-coverage user-3"));
}

#[test]
fn scripted_harness_insert_mode_records_chat_text_and_literal_colons() {
    let mut harness = UiHarness::new(80, 24);

    harness.handle_bytes(b"ihello:world\n");

    assert_eq!(harness.ui().mode(), UiMode::Insert);
    assert!(
        harness
            .transcript()
            .iter()
            .any(|line| line == "user-1> hello:world")
    );
    assert!(
        harness
            .transcript()
            .iter()
            .any(|line| line == "fixture reply: message recorded")
    );
}

#[test]
fn scripted_harness_slash_command_starts_agent_chat_command_from_chat_view() {
    let mut harness = UiHarness::new(80, 24);

    assert_eq!(harness.ui().mode(), UiMode::Command);
    assert_eq!(
        harness.ui().selected_agent().map(|id| id.as_str()),
        Some("user-1")
    );

    harness.handle_bytes(b"/status\n");

    assert_eq!(harness.ui().mode(), UiMode::Insert);
    assert_eq!(harness.ui().focus(), PaneFocus::Right);
    assert!(
        harness
            .transcript()
            .iter()
            .any(|line| line == "user-1> /status")
    );
}

#[test]
fn scripted_harness_prompt_slash_command_routes_to_selected_agent() {
    let mut harness = UiHarness::new(80, 24);

    harness.handle_bytes(b":/status\n");

    assert_eq!(harness.ui().mode(), UiMode::Command);
    assert_eq!(
        harness.ui().selected_agent().map(|id| id.as_str()),
        Some("user-1")
    );
    assert!(
        harness
            .transcript()
            .iter()
            .any(|line| line == "user-1> /status")
    );
    assert!(
        harness
            .transcript()
            .iter()
            .any(|line| line == "fixture reply: message recorded")
    );
}

#[test]
fn scripted_harness_mouse_wheel_scrolls_chat_history() {
    let mut harness = UiHarness::new(80, 10);

    harness.handle_bytes(&[23, b'l']);
    harness.handle_byte(b'i');
    for index in 0..12 {
        harness.handle_bytes(format!("message-{index:02}\n").as_bytes());
    }

    let bottom_frame = harness.render_frame();
    assert!(!bottom_frame.contains("UI harness"));
    assert!(bottom_frame.contains("message-11"));

    for _ in 0..8 {
        harness.handle_bytes(b"\x1b[<64;20;3M");
    }

    let scrolled_frame = harness.render_frame();
    assert!(scrolled_frame.contains("UI harness"));
    assert!(scrolled_frame.contains("chat> "));

    for _ in 0..8 {
        harness.handle_bytes(b"\x1b[<65;20;3M");
    }

    let bottom_again = harness.render_frame();
    assert!(!bottom_again.contains("UI harness"));
    assert!(bottom_again.contains("message-11"));
}

#[test]
fn scripted_harness_arrow_keys_edit_focused_chat_without_switching_to_command() {
    let mut harness = UiHarness::new(80, 24);

    harness.handle_bytes(&[23, b'l']);
    harness.handle_bytes(b"iab\x1b[DZ\n");

    assert_eq!(harness.ui().mode(), UiMode::Insert);
    assert!(
        harness
            .transcript()
            .iter()
            .any(|line| line == "user-1> aZb")
    );
}

#[test]
fn scripted_harness_insert_arrow_keys_move_visible_chat_cursor() {
    let mut harness = UiHarness::new(80, 24);
    harness.handle_bytes(&[23, b'l']);
    harness.handle_bytes(b"iab\x1b[D");
    assert_eq!(harness.ui().mode(), UiMode::Insert);
    assert!(harness.render_frame().ends_with("\u{1b}[5;25H"));
    harness.handle_bytes(b"\x1b[C");
    assert_eq!(harness.ui().mode(), UiMode::Insert);
    assert!(harness.render_frame().ends_with("\u{1b}[5;26H"));
}
#[test]
fn scripted_harness_arrow_keys_recall_chat_history() {
    let mut harness = UiHarness::new(80, 24);

    harness.handle_bytes(&[23, b'l']);
    harness.handle_bytes(b"ifirst\nsecond\n\x1b[A\x1b[A\x1b[B\n");

    assert_eq!(harness.ui().mode(), UiMode::Insert);
    assert_eq!(
        harness
            .transcript()
            .iter()
            .filter(|line| line.as_str() == "user-1> second")
            .count(),
        2
    );
}

#[test]
fn scripted_harness_chat_history_down_restores_in_progress_message() {
    let mut harness = UiHarness::new(80, 24);

    harness.handle_bytes(b"ifirst\nsecond draft\x1b[A\x1b[B\n");

    assert!(
        harness
            .transcript()
            .iter()
            .any(|line| line == "user-1> second draft")
    );
}

#[test]
fn scripted_harness_bytewise_arrow_keys_edit_focused_chat_without_switching_to_command() {
    let mut harness = UiHarness::new(80, 24);

    harness.handle_bytes(&[23, b'l']);
    harness.handle_byte(b'i');
    harness.handle_byte(b'a');
    harness.handle_byte(b'b');
    harness.handle_byte(27);
    harness.handle_byte(b'[');
    harness.handle_byte(b'D');
    harness.handle_byte(b'Z');
    harness.handle_byte(b'\n');

    assert_eq!(harness.ui().mode(), UiMode::Insert);
    assert!(
        harness
            .transcript()
            .iter()
            .any(|line| line == "user-1> aZb")
    );
}

#[test]
fn scripted_harness_bytewise_arrow_prefix_keeps_focused_chat_in_insert_mode() {
    let mut harness = UiHarness::new(80, 24);

    harness.handle_bytes(&[23, b'l']);
    harness.handle_byte(b'i');
    harness.handle_byte(b'a');
    harness.handle_byte(27);

    assert_eq!(harness.ui().focus(), PaneFocus::Right);
    assert_eq!(harness.ui().mode(), UiMode::Insert);
    assert!(harness.render_frame().contains("mode=insert focus=right"));

    harness.handle_byte(b'[');
    assert_eq!(harness.ui().mode(), UiMode::Insert);

    harness.handle_byte(b'D');
    harness.handle_byte(b'Z');
    harness.handle_byte(b'\n');

    assert!(harness.transcript().iter().any(|line| line == "user-1> Za"));
}

#[test]
fn scripted_harness_arrow_keys_move_left_pane_selection_like_j_k() {
    let mut harness = UiHarness::new(80, 24);

    assert_eq!(harness.ui().focus(), PaneFocus::Left);
    assert_eq!(
        harness.ui().selected_agent().map(|id| id.as_str()),
        Some("user-1")
    );

    harness.handle_bytes(b"\x1b[B");
    assert_eq!(
        harness.ui().selected_agent().map(|id| id.as_str()),
        Some("user-2")
    );

    harness.handle_bytes(b"\x1b[A");
    assert_eq!(
        harness.ui().selected_agent().map(|id| id.as_str()),
        Some("user-1")
    );
}

#[test]
fn scripted_harness_left_right_arrows_move_left_pane_selection_like_j_k() {
    let mut harness = UiHarness::new(80, 24);

    assert_eq!(harness.ui().focus(), PaneFocus::Left);
    assert_eq!(
        harness.ui().selected_agent().map(|id| id.as_str()),
        Some("user-1")
    );

    harness.handle_bytes(b"\x1b[C");
    assert_eq!(
        harness.ui().selected_agent().map(|id| id.as_str()),
        Some("user-2")
    );

    harness.handle_bytes(b"\x1b[D");
    assert_eq!(
        harness.ui().selected_agent().map(|id| id.as_str()),
        Some("user-1")
    );
}

fn strip_ansi(input: &str) -> String {
    let mut output = String::new();
    let mut chars = input.chars();
    while let Some(ch) = chars.next() {
        if ch == '\u{1b}' {
            for next in chars.by_ref() {
                if next.is_ascii_alphabetic() {
                    break;
                }
            }
        } else {
            output.push(ch);
        }
    }
    output
}
----- END FILE tests/ui_harness.rs -----
