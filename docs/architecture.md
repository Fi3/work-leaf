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
`WORK_LEAF_ORCHESTRATOR_URL` when that environment variable is present; otherwise it starts an
embedded localhost controller on `127.0.0.1:0` and connects to it through
`src/http_controller.rs::HttpControllerClient`. The terminal frontend renders through
`src/terminal_app.rs::RemoteTerminalApp`. The in-process
`src/terminal_app.rs::TerminalApp<B>` remains the local controller adapter used by tests and
embedders that construct a `CommandChat<B>` directly.

The project-root `start` script builds the `work-leaf` binary in release mode unless
`WORK_LEAF_START_SKIP_BUILD=1`, resolves the CLI binary from `WORK_LEAF_START_BIN_DIR` or
`target/release`, and executes it with the remaining arguments. `-d`/`--daemon` starts the packaged
binary in daemon mode, and `-c`/`--cli <http-api-url>` attaches the terminal CLI to an existing HTTP
API endpoint. With `--bench`, the script searches `WORK_LEAF_START_BENCH_RESULTS_DIR` or
`bench-results` for timestamped `*-artifacts` directories that contain executable `bin/work-leaf`
and `bin/work-leaf-orchestrator` files. It lists those saved benchmark binary sets newest first by
artifact name, prompts for a selection, skips the release build, and executes the selected
artifact's `bin/work-leaf`.

The project-root `build-target` script packages the user-facing `work-leaf` binary for the Rust host
target reported by `rustc -vV`. `WORK_LEAF_BUILD_TARGETS` accepts an explicit whitespace-separated
target list for release automation or manually prepared cross-linking environments. Before building
each package, the script asks `rustup` to install the target when `rustup` is available and the target
is not already installed. Each package is written under `dist/work-leaf-<target>` and includes only
the `work-leaf` binary; Unix-like packages also get a `.tar.gz` archive, and Windows packages get a
`.zip` archive when `zip` or PowerShell is available.

The `.github/workflows/release-binaries.yml` workflow builds release packages on native
GitHub-hosted runners: Ubuntu x64 and ARM64 for Linux, macOS Intel and ARM64 for Darwin, and Windows
x64 and ARM64 for MSVC. The workflow installs or verifies the native compiler, SDK, or MSVC
components before installing the Rust target and invoking `build-target` with a single
`WORK_LEAF_BUILD_TARGETS` value.

The project-root `smoke-three-features` script runs the current Work Leaf binaries against a
temporary Git checkout at the three-feature smoke-test base commit. It builds release binaries from
the current checkout unless `WORK_LEAF_SMOKE_SKIP_BUILD=1`, passes those binaries to `start` through
`WORK_LEAF_START_BIN_DIR`, prints the three real-agent `:new` prompts, and removes the temporary
checkout on normal exit, launch failure, or interruption.

The project-root `bench-three-features` script runs the same three-feature scenario through the
localhost HTTP API with the real configured Codex backend. It builds the current release binaries
unless `WORK_LEAF_BENCH_SKIP_BUILD=1`, runs those binaries against a temporary checkout at the smoke
base commit, polls the daemon through `GET /state`, records pass/fail, duration, review and
linearize completion, commit churn, code-quality checks, and efficiency notes under
`bench-results`, enables Codex sidecar tracing in the daemon artifacts, runs Codex through the SDK
sidecar, gives only the linearize agent a `danger-full-access` Codex sandbox through
`WORK_LEAF_CODEX_LINEARIZE_SANDBOX`, and removes the temporary checkout before exit.

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

For non-linearizer project agents, `PromptPolicy` adds a concurrent Work Leaf interpretation and a
per-instruction-file translation before the loaded project instruction files. The original files
remain present and authoritative for repository-specific architecture, APIs, naming, style, safety
rules, and quality bars. The translation adapts only ownership, timing, and tool-access assumptions
that normally assume one agent owns the whole workspace: patch agents prefer focused checks they
touched or introduced, report blockers caused only by another patch agent's owned files once, and
leave cross-agent reconciliation to review or linearization. The translator detects generic
instruction categories such as checks, tests, documentation, commit messages, review rules, and
real-agent verification, then maps each category to patch-agent, review-agent, or linearize-agent
responsibilities. Linearize agents receive the direct-workspace linearizer policy instead.

Patch-agent prompts keep documentation and prose-only files out of patch-agent scope. Patch agents
work on code, tests, configuration, and other feature files through orchestrator patches; docs,
README, changelog, markdown, txt, and other prose-only updates are handled during linearization when
the final reviewed behavior requires them. Patch-agent prompts also keep focused validation scoped to
pre-existing checks or checks introduced by that same patch agent, leaving another patch agent's
focused tests and broad integration reconciliation to review or linearization. They also instruct
patch agents to keep the shared worktree buildable by submitting cohesive patch units rather than
known-red intermediate changes. Linearize-agent prompts use a separate direct-workspace policy: the
linearizer reads and writes repository files, runs commands, and rewrites git history directly
rather than using `@work-leaf` read, patch, or lock directives.

`src/agent.rs` also re-exports `AgentBackend`, `AgentStreamEvent`, and `AgentShutdownHandle` from
`src/agent_runtime.rs`, so callers can import all provider-neutral agent interfaces from
`work_leaf::agent`.

## Agent Runtime Interface

`src/agent_runtime.rs` owns the provider-neutral backend contract:

- `AgentBackend::launch` starts an agent session from an `AgentLaunch`.
- `AgentBackend::send` sends a prompt to an existing agent session.
- `AgentBackend::launch_streaming` and `AgentBackend::send_streaming` provide real-time output to a
  sink of `AgentStreamEvent` values. Their default implementations call the non-streaming methods.
- `AgentBackend::launch_streaming_interruptible` and
  `AgentBackend::send_streaming_interruptible` extend the streaming calls with a provider-neutral
  stop detector. Providers that can interrupt an in-flight turn use the detector to stop generation
  after a complete terminal orchestrator directive has streamed; providers that do not override the
  methods keep the ordinary streaming behavior.
- `AgentBackend::shutdown_handle` returns an `AgentShutdownHandle` for terminating active provider
  processes.
- `AgentStreamEvent` carries status text, streamed agent messages, and streamed errors.
- `AgentShutdownHandle::shutdown` terminates registered processes, waits briefly, and then kills
  remaining processes.

The runtime also contains provider-process support that is not part of the public provider API:
registered child process ids are tracked in an internal registry; shutdown first sends terminate and
then kill to remaining registered processes. This behavior is used by provider implementations that
spawn real processes.

New agent providers implement `AgentBackend`; they do not add provider logic to `src/codex.rs`.
Providers that need real-time UI output should override the streaming methods. Providers implemented
inside this crate can use the crate-private process registration helpers for shared shutdown.
Providers outside this crate can implement launch, send, and streaming through the public trait; a
public lifecycle extension is required before external child processes can participate in the shared
`AgentShutdownHandle` registry.

## Codex Provider

`src/codex.rs` contains the Codex-specific implementation of the neutral agent runtime interface:

- `SandboxMode` and `CodexCommandConfig` define Codex runtime settings.
- `CodexBackend` stores Codex session history and implements `AgentBackend`.
- `CodexBackend` starts one embedded Python sidecar from `src/codex_sdk_sidecar.py`. The sidecar
  imports the `openai-codex` Python SDK, starts one Codex app-server through
  `openai_codex.client.CodexClient`, and multiplexes Work Leaf launch, send, command, interrupt, and
  shutdown requests over JSONL.
- `src/cli.rs::codex_backend` resolves the Codex binary from `PATH` while skipping Codex's temporary
  `~/.codex/tmp/arg0` shim when a stable `codex` executable is available later in `PATH`. Work Leaf
  passes that executable to `CodexConfig.codex_bin`, so the SDK drives the same local Codex runtime
  selected from `PATH` instead of silently using the SDK package's pinned binary. `codex_backend`
  prepends the selected executable's parent directory to the daemon process `PATH` before worker
  threads start.
- The project-root `start` script provisions `target/work-leaf-codex-sdk-venv`, installs
  `openai-codex` when the environment does not already provide an importable SDK, and exports
  `WORK_LEAF_CODEX_SDK_PYTHON` for the CLI. The `WORK_LEAF_CODEX_SDK_PYTHON` environment variable
  selects an existing Python interpreter for the sidecar.
- `CodexBackend::record_launch_reply` and `session` maintain in-memory session state.
- `CodexBackend` receives app-server notifications from the Python sidecar and records the returned
  thread id, the complete assistant-message transcript for the turn, and per-turn token usage in the
  provider-neutral session state. The complete transcript is used for orchestrator directive parsing
  even when the SDK reports several assistant message items before the final turn-completed response.
- Codex linearizer sessions run with the dedicated linearize sandbox, which defaults to
  `workspace-write` and can be configured with `WORK_LEAF_CODEX_LINEARIZE_SANDBOX` (`read-only`,
  `workspace-write`, or `danger-full-access`). The approval policy remains `never`. Patch agents and
  reviewer agents keep the configured Codex sandbox and continue to use orchestrator-mediated writes.
- `CodexBackend` serializes launch and send operations per `AgentId` across cloned backend handles.
  This keeps a single Codex thread from receiving overlapping turns while allowing different agent
  sessions to work concurrently through the shared SDK/app-server sidecar.
- `CodexBackend` uses the interruptible streaming contract for patch/review agents. When a streamed
  assistant message already contains a complete terminal Work Leaf directive such as a read request,
  edit/patch block, locked command, routed send, or done marker, the backend sends an app-server
  interrupt for that active turn so the orchestrator can process the directive promptly instead of
  waiting for the model to emit duplicate directive blocks.

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
- `CommandChat::handle_line` for processing command lines such as `new`, `chat`, `review`,
  `linearize`, and `force-linearize`.
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
- deterministic chat titles derived locally from the first user prompt, with
  `src/chat_title.rs::ChatTitleAgent` tracking first-prompt naming state and
  `src/chat_title.rs::fallback_chat_title_from_prompt` filtering low-signal prompt wording,
- command transcripts,
- background launch/send/review workers,
- stream routing from `AgentStreamEvent` into the selected session,
- review startup, automatic per-patch-agent review routing, reviewer-session creation, and
  reviewed-commit bookkeeping,
- shutdown propagation to running agents.

When an agent worker finishes, the controller records the agent output and clears that session's
loading state. Launch requests are queued by command handling and started from the controller polling
path. Rapidly created launches wait until the active launch emits its first backend stream event or
finishes, which prevents multiple backend child-process startups from piling up while still allowing
the agents to run concurrently after startup. The first backend stream event changes that session's
loading state from launch startup to waiting for a reply, so frontends can distinguish provider
startup from an active agent turn without relying on provider-specific status text. Routed
orchestrator follow-up turns that stream output into a different session mark that target session as
waiting for a reply until the worker that owns the follow-up finishes, and the completed follow-up
reply is appended to the target session transcript. A user-agent session becomes review-ready when
that agent has an unreviewed
provisional commit in git history and the agent emits `@work-leaf done`; the patch commit and the
done directive may come from different turns in the same session. Successful patch application
returns a continuation prompt to the patch agent when the agent has not reported done, so the agent
can run repository-required checks through locked command directives, provide follow-up patches, or
signal review readiness. Repository build, test, format, and required-check commands run only
through agent-emitted orchestrator directives that name the command and the write-lock paths the
command may touch. Locked command requests that use common shell constructs to force a successful
status, such as `|| true`, trailing `; true`, `set +e`, or `set +o errexit`, are
rejected before execution so validation failures remain visible. Locked command runs have a
five-minute default timeout, after which the command is terminated, locks are released, and a longer
run requires user authorization. The command-result prompt sent back to the agent includes status,
timeout state, locked paths, and compacted stdout/stderr when output is large; controller command-run
events retain the captured command output for integrations that need it. `PromptPolicy` injects
project instruction files into agent prompts, and the active backend agent is responsible for
choosing and requesting the repository checks required by those instructions before reporting work
done.
When a mediated file-read response would be large, the orchestrator writes the exact file text to a
temporary context bundle in a per-orchestrator system-temp directory and sends the agent a compact
manifest with the bundle path, file names, digests, and byte counts. The bundle directory is removed
when the owning orchestrator state is dropped. Agents in orchestrator-read mode may read only those
orchestrator-provided bundle paths directly; repository file reads remain mediated by
`@work-leaf read`, and manual repository writes use the structured `@work-leaf edit` protocol. The
legacy `@work-leaf patch` protocol remains available for complete valid unified diffs and for
tracked command diffs. The orchestrator tracks
per-agent file snapshots with digests. A repeated read for unchanged text returns only the matching
digest; a repeated read for changed text returns a diff from that agent's last mediated snapshot
instead of re-sending full file text. The `@work-leaf read --force <path>` form is accepted for
compatibility, but once an agent has a tracked snapshot for a path the repeated-read response still
uses the digest/diff path so large files are not repeatedly copied into the same agent session.
Tracked file changes produced by locked commands are captured as per-file diffs while the command
locks are still held, then restored out of the shared checkout. Those captured command diffs remain
pending for that patch agent until the agent submits them through the patch protocol or emits
`@work-leaf command discard <reason>`. Pending command output blocks `@work-leaf done`, and the
orchestrator returns the captured diff so the agent can submit the command output as a provisional
patch when it belongs in the final work.
Accepted patch commits are recorded in a patch-ownership ledger for coordination inside the shared
worktree. The ledger tracks test-like paths by generic project conventions such as test/spec
directories, test/spec file stems, and test/spec extensions. Patch-agent command directives that lock
another patch agent's focused test path are blocked before the command starts, and the agent receives
compact guidance to run pre-existing checks or checks introduced by its own patch instead. Broad
validation commands may lock broad directories that contain another patch agent's focused tests when
`CommandWritePolicy` classifies the command's write output as disjoint from those tests, such as a
build or test runner that writes only cache or build output. Broad integration failures that involve
another patch agent's focused tests are handled during review or linearization unless the submitting
agent's own source change clearly caused the failure.
Already-applied or stale duplicate patches receive a compact already-applied response instead of a
file refresh, so the agent does not rebase and resend a diff already represented in the repository.

Review bookkeeping has three scopes. The controller records a launch-time review baseline for each
patch agent, tracks the latest reviewed hash for that patch agent so the same agent head is not
reviewed twice, and asks reviewers to inspect every provisional commit from the active baseline
through the latest patch-agent commit. `CommandChat` also keeps the ordered exact review targets that
completed review during the active instance, including their cumulative review scope text. The
linearizer handoff compacts multiple reviewed hashes from the same patch agent into one final patch
target while preserving the individual hashes, reasons, and context inside that target. This keeps
the final history shaped as one commit per accepted patch-agent feature unless the user explicitly
accepts a different grouping.
When review resolves with no findings, the controller marks the patch-agent session as needing a
user completion decision and appends a yes/no question to that session. `yes` closes the feature. A
bare `no` keeps the feature open. A `no` followed by punctuation and follow-up text, such as
`no, handle the remaining case`, keeps the feature open and sends the patch agent a structured
follow-up prompt that asks for the requested fixes through the patch flow and another
`@work-leaf done` before the controller starts a new review pass. After the new review resolves with
no findings, the controller selects the patch-agent chat and asks the completion question again.
Messages that are not accepted yes/no answers keep the completion question active and are not sent to
the backend. A later message in a closed chat clears the closed state before sending the message to
the agent backend. The normal `linearize` command requires every reviewed patch-agent chat in the
current instance to be closed before launching the linearizer. `force-linearize` launches the same
linearizer handoff without that closed-chat gate for automation and direct command-chat flows that
intentionally bypass the completion decision.

Agent dependency options are validated before dependent work is registered. A dependency target from
`--depends-on <agent-id>` must name an existing, different session. When the dependency is still
open, the controller records `WorkLeafSession.depends_on` and `depended_on_by`, exposes
`WorkLeafLoading::WaitingForDependency`, and stores the pending launch or patch-promotion send until
the dependency is closed. When the dependency is already closed, the controller proceeds immediately
and records a visible dependency-release line in the dependent transcript.

When linearization starts, the controller interrupts all visible non-linearizer sessions, clears their
loading state, cancels pending dependent launches and patch-promotion sends, detaches dependency
links for those cancelled waits, leaves chat transcripts visible, and ignores late worker events from
stopped sessions. This keeps stale patch or review workers from appending findings, releasing
dependent work, or starting new reviews after the linearizer has taken ownership of the reviewed
work.

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
  When the target session is already waiting for a backend reply, the controller appends the user's
  text immediately and queues the backend send until that session is available. Messages that start
  with `/` followed by a non-empty command token are routed to the selected backend instead of being
  interpreted as Work Leaf commands. The slash command output visible in the chat is the selected
  backend's response to the raw slash-prefixed message.
- `start_review` to create or resume reviewer sessions for explicit history-wide review and stream
  reviewer output.
- `is_busy`, `wait_for_idle`, and `wait_for_session_line` for tests and event loops.
- `shutdown` to terminate active backend processes.

The controller exposes renderable state through:

- `WorkLeafSnapshot`, which contains the command transcript and sessions.
- `WorkLeafSession`, which contains agent id, kind, feature/title, transcript lines, loading state,
  completion state, and optional provider token usage.
- `WorkLeafEvent`, which reports session creation, session updates, streamed lines, selection
  changes, token-usage updates, transcript lines, and quit requests.
- `WorkLeafLoading`, which distinguishes launch, waiting-for-reply, and waiting-for-dependency
  states.

`WorkLeafEvent` uses append-oriented transcript events for efficient remote frontends. `AgentAdded`
provides the initial session snapshot, `AgentLineAppended` carries one new session line, and
`CommandTranscriptLine` carries one new command transcript line. `AgentStatusUpdated` carries
session metadata and loading state without re-sending the session transcript. `AgentUpdated` remains
part of the DTO surface for full-session replacement when an integration needs it. Session line
appends and status changes are not paired with full-session replacement events, so remote frontends
can update long transcripts without re-receiving the full transcript text. When an agent turn is
processed through orchestrator directive follow-ups, the controller keeps the final agent-visible
reply but does not append aggregate `orchestrator:` and `agent follow-up from ...` transcript blocks
as one chat line; those blocks duplicate streamed lines and command/file events that frontends
already receive incrementally.

New UIs should consume `WorkLeafController` and these DTOs. They should not duplicate worker
spawning, session naming, review lookup, loading bookkeeping, or orchestrator event routing.

## Localhost HTTP Controller

`src/http_controller.rs::HttpControllerServer` is a transport adapter over `WorkLeafController`. It
owns no workflow behavior; each HTTP route delegates to the corresponding controller method or DTO:

- `GET /snapshot` returns `WorkLeafSnapshot`.
- `GET /state` polls workers once and returns `WorkLeafControllerState`, containing both `busy` and
  `snapshot`, for polling clients that need consistent state with one HTTP request.
- `POST /events/drain` returns pending `WorkLeafEvent` values after polling workers when needed.
- `GET /busy` returns the controller busy state.
- `POST /command` calls `WorkLeafController::execute_command_line`.
- `POST /command-agent` calls `WorkLeafController::send_command_agent_message`.
- `POST /agent/message` calls `WorkLeafController::send_message`.
- `POST /agent/interrupt` calls `WorkLeafController::interrupt_agent`; when the backend accepts the
  interrupt, the selected session loading state is cleared immediately so frontends stop presenting
  the chat as actively waiting.
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
frame. They own terminal event-loop concerns such as insert mode, prompt mode, visual text
selection, `Ctrl-W` navigation, SGR mouse clicks, SGR mouse wheel scrolling of the right pane,
chunked terminal input parsing, rendering invalidation, and polling background workers. Insert mode
sends chat text to the selected agent session, or to `command-agent` when the Work Leaf command
surface is selected. Bracketed-paste newlines and Shift+Enter are chat prompt line breaks. A plain
Enter submits the buffered chat text.
When an agent chat is selected in command mode, `/` focuses the chat, seeds the chat buffer with
`/`, and enters insert mode so `/status`-style input submits through the same selected-agent chat
path. Selected-agent chat messages whose first token is a slash command are routed to the selected
backend rather than the Work Leaf command parser. Slash-prefixed colon-prompt input also routes to
the selected agent chat when an agent is selected.

The terminal app renders the left pane as command, patch lifecycle, review lifecycle, read, and
linearize sections. Patch chats are grouped as closed, new, ready, and working rows. Review chats are
grouped as reviewing or reviewed based on the controller-recorded review outcome order. Keyboard
selection and mouse targets follow the grouped section order while preserving each visible agent's
chat selection behavior. Pressing Enter while the left pane is focused opens the selected chat and
enters insert mode; pressing comma from a right-pane command-mode chat focuses the visible left pane.
The terminal app maps a patch or read session to a left-pane `READY` marker when the controller
exposes no loading state for that session. Review sessions do not show the ready marker after review
output completes. Sessions waiting for a completion answer show `DONE?` in the row title, and closed
sessions show `CLOSED` without the ready marker. `TerminalUi` queues one terminal bell when a chat
transitions into a ready-highlighted state and renders ready rows in reverse video so they remain
highlighted until the chat becomes busy or closed.

`src/ui.rs::TerminalUi` owns terminal-specific presentation state:

- `UiMode`, `PaneFocus`, `UiSurface`, `UiKey`, and `UiAction` model terminal interactions.
- `AgentListEntry` is the terminal left-pane representation of an agent row.
- `TerminalLayout` computes pane geometry.
- `TerminalUi` renders left/right panes, prompts, cursor placement, command-interface selection,
  visual selections, and terminal navigation actions. The right pane keeps the chat prompt visible
  while scroll offsets reveal earlier transcript rows. Command-mode `v`, `V`, and `Ctrl-V` start
  character, line, and block selection in the focused pane; `y` and `Y` yank selected text through
  the terminal OSC 52 clipboard sequence. Agent-chat panes support message folds: command mode
  `zM` folds all message bodies, `zR` unfolds them, and visual-mode `zc`, `zo`, and `za` close,
  open, or toggle the message under the visual cursor. Fold state belongs to the active terminal
  window, so split chat windows can show different fold states for the same transcript.

`src/ui_harness.rs::UiHarness` is the test harness for terminal behavior. It exercises the same
`TerminalUi` frame path used by the interactive example. UI tests should drive
`UiHarness::handle_byte` or `UiHarness::handle_bytes` rather than duplicating terminal input logic.

A web UI, desktop UI, or non-terminal integration should not depend on `TerminalApp` or
`TerminalUi`; it should depend on `WorkLeafController` and the DTOs in `src/workspace.rs`.

## Core Workflow Modules

`src/orchestrator.rs::AgentOrchestrator<B>` parses and executes `@work-leaf` directives emitted by
agents. It uses `FileLockTable` for file reads and command write locks, `CommandWritePolicy` for
command classification, `PatchCoordinator` for patch requests, a patch-ownership ledger for
shared-worktree test coordination, and the active `AgentBackend` for routed follow-up messages. Its
public output is `OrchestratorEvent`.

`src/locks.rs::FileLockTable` owns root-scoped path normalization and read/write locking.
`FileSnapshot` carries file read results. `CommandWritePolicy` and `CommandWriteIntent` provide
heuristic read-only/write-intent classification for commands when an agent is unsure. Agent-requested
command runs execute in the project root while `FileLockTable` holds write locks for the normalized
lock paths supplied by the agent. File paths are normalized relative to the project root and cannot
escape that root.

`src/patch.rs::GitPatcher` applies structured exact-block edits and complete unified diffs under
write locks, then creates metadata commits for accepted patches. Structured edits match old blocks
against current UTF-8 file text, reject missing or ambiguous matches before writing, write the
resulting files, and let Git compute the final diff from the working tree. Unified diffs are
validated through `git apply --recount --check` before application. Patch application locks the
touched files and the repository root path in `FileLockTable`; the root lock serializes git index
operations such as `git add` and `git commit` while agents can still reason and produce patches
concurrently. The unified-diff path also accepts a matching already-applied diff when the same change
is already present, and it applies captured locked-command diffs through the normal patch path so
command output can be saved as the agent's provisional patch. `PatchCoordinator<B>` connects patch
conflicts and malformed patch diagnostics back to the active agent backend. `PatchRequest`,
`PatchOutcome`, and `PatchError` are the public patch workflow types.

`src/review.rs::GitHistory` reads latest agent commits from repository history, builds cumulative
review targets for a patch agent since a launch or reviewed baseline, and resolves agent metadata
commits by exact hash. `ReviewCoordinator<B>` launches reviewer agents against those review targets
and loops until the reviewer reports no findings or the configured maximum round count is reached.
Reviewer launch prompts include source context rendered from Work Leaf commit metadata, git commit
logs, and any recorded backend session chat history for the patch agent. The review flow does not ask
the patch agent for a separate summary before launching the reviewer. `CommandChat` resolves
reviewer `@work-leaf` directives, such as file reads, before interpreting reviewer output as
findings. `CommandChat` and `WorkLeafController` keep a stable `review-<agent-id>` reviewer identity
for each patch agent and skip latest agent heads that have already completed review. `AgentCommit`,
`ReviewResult`, and `ReviewError` are the public review workflow types. `WorkLeafController` scopes
automatic review after a patch agent reports done to the patch agent that produced the provisional
commit; explicit review commands use the history-wide review target lookup. Reviewer prompts treat
documentation and prose-only updates as linearizer responsibility, so missing docs, README,
changelog, markdown, txt, or other plain-text updates are not reported as patch-agent findings. If a
reviewer reports a non-code finding such as missing real-agent verification, the patch agent can
resolve it by replying with the exact evidence or blocker instead of submitting another code patch;
the reviewer recheck prompt includes that reply so the reviewer evaluates the evidence directly on
the next pass.

`src/linearize.rs::LinearizePlanner<B>` prepares linearization questions and launches a linearizer
agent with decisions, groups, and required tests. `LinearizeAction`, `LinearizeGroup`,
`LinearizePlan`, `LinearizeQuestion`, `LinearizeHandoff`, and `LinearizeError` are the public
linearization workflow types. `CommandChat` and `WorkLeafController` launch linearization from the
exact commits recorded as reviewed in the current command-chat or controller instance; unrelated
historical agent metadata commits are outside the linearizer scope unless the user explicitly reviews
or adds them in that session. When one patch-agent id completes multiple reviewed commits in one
active instance, the linearizer receives one final target for that patch agent and the target context
lists every reviewed hash that must be preserved. The linearizer owns documentation and plain-text
updates deferred by patch agents, uses direct workspace access instead of orchestrator mediation, and
rewrites provisional work-leaf commits into final commits after the user accepts its proposed plan.
The rewritten stack stays rooted at the parent or common base of the reviewed commits unless the user
explicitly requests retargeting to another branch tip; branch names such as `main` or `master` are
context for discovering that base, not automatic rewrite targets.
The resulting history contains exactly one final commit per linearize target unless the accepted
plan explicitly groups targets together; support, test-hygiene, review-fix, validation-fix, and
documentation-only work is folded into the relevant feature commit rather than kept as extra final
commits.

`src/instructions.rs` is crate-private. It loads project instruction files used by `PromptPolicy`
for agent launch prompts.

`src/chat_title.rs` is crate-private. It derives lowercase hyphenated chat titles from first prompts,
filters low-signal prompt wording around the salient task words, caps titles at 40 characters, and
tracks which sessions have already been named.

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
