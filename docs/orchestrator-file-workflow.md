# Orchestrator File Workflow

This document defines the work-leaf file-access model, lock semantics, agent nomenclature, and the
normal development path for feature and bug-fix work. The terminology here is the product language
used by developers and agents, even where the Rust modules still expose lower-level names such as
`CommandChat`, `AgentBackend`, or `ReviewCoordinator`.

The core invariant is that agents do not directly own repository writes. By default, agents also
request file text through the orchestrator so reads, writes, stale context updates, patch application,
and review routing share the same coordination model. When the process is launched with
`--no-read-permission`, agents may read repository files directly from the filesystem while writes
still go through orchestrator patch application.

## Agent Nomenclature

### Command Agent

The command agent is the always-available user-facing control surface. The user interacts with it to
start work, create patch agents, ask for orchestration actions, and run high-level commands through
an LLM-mediated interface.

The command surface is represented by `src/cli.rs::CommandChat` and the controller, transport, and
terminal adapters around it:

- `src/cli.rs::CommandChat` owns the backend, file lock table, read tracker, command policy, known
  agents, and review/linearize entry points.
- `src/workspace.rs::WorkLeafController` exposes the UI-neutral control surface used by the terminal
  app.
- `src/http_controller.rs::HttpControllerServer` exposes `WorkLeafController<CodexBackend>` through
  a localhost HTTP API for the `work-leaf-orchestrator` daemon.
- `src/http_controller.rs::HttpControllerClient` drives that API for out-of-process frontends.
- `src/terminal_app.rs::RemoteTerminalApp` adapts terminal input and rendering to the HTTP client
  used by the `work-leaf` CLI.
- `src/terminal_app.rs::TerminalApp` adapts terminal input and rendering to an in-process
  controller for tests and embedders.

The command agent is not a patch author. Its job is to coordinate work and create the correct agent
for the user's request.

### Patch Agent

A patch agent is created by the user, usually through the command agent, to add a feature, fix a bug,
or make another concrete code change. Patch agents request file text through the orchestrator in the
default read-permission mode, or inspect files directly in direct-read mode. Patch agents request
writes by sending unified diff patches to the orchestrator.

A patch agent does not write files directly. In default read-permission mode, it emits:

```text
@work-leaf read <path...>
```

to receive file text. In all read-permission modes, it emits:

```text
@work-leaf patch <reason>
<unified diff>
@work-leaf end
```

to request a repository modification.

The orchestrator applies the patch atomically as one diff, not file by file. If any touched file or
hunk prevents the diff from applying, the whole patch is rejected and no clean hunk is applied as a
partial result.

### Review Agent

A review agent is run by the orchestrator to review a patch agent's work. The review agent reviews
the patch associated with the patch agent and reports either no findings or concrete findings.

The review agent MUST review only behavior introduced or modified by the reviewed patch. It must not
report pre-existing issues, unrelated style preferences, or broad repository problems unless the
reviewed patch makes them worse, depends on them, or claims to fix them.

The intended lifecycle is:

1. A patch agent submits a patch.
2. The orchestrator records the patch as a provisional agent commit.
3. The orchestrator prompts the patch agent to continue when the agent has not emitted
   `@work-leaf done`.
4. The patch agent runs required checks through locked command directives, submits follow-up patches
   when needed, and emits `@work-leaf done` when the patch is ready for review.
5. The orchestrator runs a review agent for that patch.
6. If the review agent emits `@work-leaf` directives, the orchestrator resolves those directives for
   the review agent before treating reviewer output as findings.
7. If there are no findings, the user can mark the review-agent chat as done.
8. If there are findings, the orchestrator sends those findings to the corresponding patch agent.
9. The patch agent keeps patching through the orchestrator until the review agent reports no
   findings.

The current implementation path for this review loop builds patch-agent review targets through
`src/review.rs::GitHistory`, asks the original agent for a summary, launches or resumes the patch
agent's `review-<agent-id>` reviewer, resolves reviewer orchestrator directives such as file reads,
sends findings back to the original agent, and asks the reviewer to recheck until `NO_FINDINGS` or
the round limit. Command-chat and controller review startup keep one reviewer identity per patch
agent and skip latest agent heads that already completed a review pass. Automatic review requires an
unreviewed provisional commit from the patch agent and that agent's `@work-leaf done` directive; the
done directive may arrive in the same turn as the patch or in a later turn from the same agent
session. Review is scoped to all provisional commits from that patch agent since the launch or latest
reviewed baseline. An explicit `review` command is the history-wide review entry point.

### Inspection Agent

An inspection agent has read-only access. It can request file text from the orchestrator in the
default read-permission mode, or inspect repository files directly in direct-read mode. It cannot
request writes and cannot submit `@work-leaf patch` directives.

Inspection agents are useful for planning, debugging, architecture review, log inspection, and
answering questions about the repository without creating code changes.

The distinct inspection-agent role is product nomenclature. In default read-permission mode, the
shared read path it depends on is the same orchestrator read protocol implemented in
`src/orchestrator.rs::handle_agent_directives_streaming` and
`src/orchestrator.rs::read_requested_files`.

### System Agents

System agents are run by the orchestrator in the background for internal work such as coordination,
linearization preparation, or other non-user-facing functionality. Users do not need to interact
with system agents directly, but the UI should expose enough introspection to understand what the
orchestrator is doing.

The current source already has system-style internal behavior:

- `src/chat_title.rs::ChatTitleAgent` tracks which chats have been named from their first prompt so
  chat titles are derived locally without a backend launch.
- `src/workspace.rs::WorkLeafController` tracks sessions, loading state, pending events, and
  transcript output.
- `src/ui.rs::AgentListEntry` carries visible agent metadata such as readiness, modified files,
  conflicts, dependencies, and dependents.

## File Access Contract

Agent launch prompts include a file-access policy.
`src/agent.rs::PromptPolicy` injects rules selected by `src/agent.rs::ReadPermission`.

With `ReadPermission::Orchestrator`, prompts tell agents:

- do not read files directly;
- ask the orchestrator for file text;
- do not write files directly;
- provide unified diff patches for requested writes;
- use `@work-leaf locks classify <command>` when the agent is unsure whether a command writes
  project files;
- use `@work-leaf locks run <path> <path...> -- <command>` to run commands while the orchestrator
  holds write locks for paths the command may write;
- keep locked command runs within five minutes unless the user authorizes a longer lock-holding
  command;
- use `@work-leaf done` when no more orchestrator work is required.

With `ReadPermission::DirectFilesystem`, prompts tell agents:

- read repository files directly from the filesystem;
- use read-only inspection commands for repository context instead of `@work-leaf read`;
- do not write files directly;
- provide unified diff patches for requested writes;
- use `@work-leaf locks classify <command>` when the agent is unsure whether a command writes
  project files;
- use `@work-leaf locks run <path> <path...> -- <command>` to run commands while the orchestrator
  holds write locks for paths the command may write;
- keep locked command runs within five minutes unless the user authorizes a longer lock-holding
  command;
- use `@work-leaf done` when no more orchestrator work is required.

The Codex backend applies this policy when launching sessions. Known resumed sessions receive only
the follow-up message as Codex resume stdin, because the Codex thread already contains the
launch-time policy and repository instructions. The source chain is:

1. `src/cli.rs::codex_backend` builds a `src/codex.rs::CodexBackend` with
   `PromptPolicy::for_project_with_read_permission` and resolves the Codex executable from `PATH`
   while skipping Codex's temporary `~/.codex/tmp/arg0` shim when a stable binary is available.
   The selected executable is passed to the Codex Python SDK sidecar through
   `CodexConfig.codex_bin` when SDK mode is active. Its parent directory is prepended to the daemon
   process `PATH` before workers start.
2. `src/codex.rs::CodexBackend` injects the policy into a launch prompt, sends it to the SDK
   sidecar, and records the returned app-server thread id for follow-up turns.
3. Known-session follow-up messages are sent raw to the same SDK/app-server thread. The fallback
   `codex exec resume` path uses policy injection only for resume invocations without in-memory
   session context.
4. Agent replies are processed by `src/cli.rs::CommandChat::process_agent_reply_streaming`.
5. Directive handling enters `src/orchestrator.rs::handle_agent_directives_streaming`.

The process starts in `ReadPermission::Orchestrator` by default. The top-level
`--no-read-permission` option selects `ReadPermission::DirectFilesystem`; in that mode the
orchestrator no longer receives normal file-read requests from agents and cannot record those direct
reads as pending file snapshots.

## Lock Table

File locks are implemented by `src/locks.rs::FileLockTable`. The table is an in-memory map from
normalized repository-relative paths to `std::sync::RwLock<()>` values:

```rust
Mutex<BTreeMap<PathBuf, Arc<RwLock<()>>>>
```

The outer `Mutex` protects lookup and creation of per-path locks. Each `RwLock<()>` is the actual
read/write coordination object for one normalized path. The lock stores no file content; the `()`
payload is only a synchronization token.

`FileLockTable` is cloneable. Clones share the same map through `Arc`, so a `CommandChat` clone used
by a background worker and a `GitPatcher` created during directive handling coordinate against the
same lock table.

## Path Normalization

All file lock paths are repository-relative. `src/locks.rs::normalize_relative_path` rejects:

- parent traversal with `..`;
- absolute root paths;
- platform prefixes.

It ignores `.` and keeps normal path components. Multi-path operations sort and deduplicate paths
before retrieving locks. This gives stable lock acquisition order and avoids locking the same path
twice in one operation.

The lock layer is a repository-root boundary, not a full filesystem sandbox. It rejects lexical path
escapes before joining paths to the project root.

## Reads

In default read-permission mode, an agent requests file text with:

```text
@work-leaf read src/lib.rs src/orchestrator.rs
```

A single read directive can name multiple paths. When one agent reply contains consecutive read
directives, `src/orchestrator.rs::handle_agent_directives_streaming` handles them as one grouped
read response to the same agent session.

Repeated reads use the agent's tracked snapshot. If the current digest matches the last snapshot
sent to that agent, the response reports the unchanged digest and does not resend file text. If the
current digest differs, the response sends a unified diff from the agent's last snapshot to the
current file text. The force form is accepted for compatibility:

```text
@work-leaf read --force src/lib.rs
```

For paths that already have a tracked snapshot in the same agent session, the force form still uses
the repeated-read digest/diff response. This keeps large files from being copied into the same chat
session more than once.

The read path is:

1. `src/orchestrator.rs::parse_agent_directives` parses the directive into `AgentDirective::Read`.
2. `src/orchestrator.rs::handle_agent_directives_streaming` groups consecutive read directives and
   calls `read_requested_files` once for the grouped path set.
3. `read_requested_files` normalizes all valid paths.
4. The orchestrator acquires shared read locks for the full valid path set.
5. The orchestrator reads file contents while those read locks are held.
6. The orchestrator sends a `work-leaf file text` response, an unchanged digest, or a repeat-read
   diff to the same agent session.
7. Successful snapshots are recorded in `src/orchestrator.rs::FileReadTracker`.

Read locks are shared. Many agents can read the same file at the same time. A write lock for the same
file waits until existing readers release their read guards.

Read locks are held only while the orchestrator creates the snapshot. The agent receives text and
works from that snapshot. The agent does not keep a long-lived lock lease.

Unavailable paths are reported in the same response under `Unavailable file text`, so an agent can
continue with partial context instead of stalling.

In direct-read mode, the agent reads repository files from the filesystem through the provider's
read-only execution environment. Direct reads do not call `read_requested_files`, do not acquire
`FileLockTable` read locks, and do not create `FileReadTracker` entries. This mode avoids an
orchestrator read round trip at the cost of weaker stale-context tracking for files the agent read
directly.

## Pending Read Tracking

The orchestrator treats successful orchestrator-provided file snapshots as agent context.
`FileReadTracker` stores:

```text
agent id -> path -> last file text snapshot and digest sent to that agent
```

This map is used to detect stale context for orchestrator-mediated reads and to compute compact
refreshes. It also lets repeat reads avoid copying full file text into the same agent thread when
the digest is unchanged or a snapshot-to-current diff is enough. If an agent has read a file through
`@work-leaf read` and another patch changes that file before the reader submits a patch or reports
done, the reader may be about to produce a stale diff. Direct filesystem reads are not present in
this map, so direct-read mode relies on `git apply --check`, conflict diagnostics, and agent rereads
instead of proactive stale-reader updates for those reads.

The tracker updates as follows:

- successful `@work-leaf read` responses store the returned file snapshots for the agent;
- patch conflict responses that include compact file refreshes also refresh the patching agent's
  stored snapshots;
- a successful patch clears the patching agent's pending read entries for the touched files;
- `@work-leaf done` clears all pending read entries for that agent.

After a successful patch, the orchestrator prompts the patch agent to continue when the same
directive turn does not include `@work-leaf done`:

```text
work-leaf patch applied
files: path
Continue from the repository instructions.
Run any required or relevant checks through `@work-leaf locks run <path>... -- <command>` when the command may write files.
Keep locked command runs within five minutes unless the user authorizes a longer lock-holding command.
Provide additional patches if checks fail or more work is needed; emit `@work-leaf done` only when this patch is ready for review.
```

The orchestrator also checks all other agents with pending reads for the touched files. For each
stale reader, it sends:

```text
work-leaf file update
Another agent changed files you previously read before you submitted a patch.
Rebase any pending patch against the compact file refresh below.

work-leaf file refresh
This is a compact refresh, not a patch to submit. It shows changes from the last file text this agent received. Repeated full-text refreshes are intentionally avoided to keep the session compact.

--- path ---
current digest: fnv64:<hash>; bytes:<n>
previous digest: fnv64:<hash>; bytes:<n>
status: changed since this agent's last snapshot
diff --git a/path b/path
--- a/path
+++ b/path
@@ ...
<snapshot-to-current diff>
```

That update is grouped per stale agent and contains a bounded unified diff from the stale snapshot to
the current file. Large refresh diffs are omitted with the current digest and byte count. The point
is to make the next patch more likely to apply while keeping multi-agent stale-context updates small
enough for long-running agent sessions.

## Writes And Atomic Patches

Patch agents request writes with a unified diff. The patch path is:

1. `src/orchestrator.rs::parse_agent_directives` parses `@work-leaf patch <reason>` until
   `@work-leaf end`.
2. `src/orchestrator.rs::handle_agent_directives_streaming` creates a `src/patch.rs::GitPatcher`.
3. `GitPatcher::apply` extracts the unified diff and parses all touched files.
4. All touched files are normalized, sorted, and deduplicated, and the repository root lock is added
   to the lock set.
5. `src/locks.rs::FileLockTable::with_write_locks` acquires exclusive write locks for all touched
   files and the repository root path. The root lock serializes git index operations across
   concurrent patch agents.
6. `GitPatcher::apply_with_locks` runs `git apply --check -` for the entire diff.
7. If the check passes, it runs `git apply -` for the entire diff.
8. Passing patches are staged with `git add -- <files>` and committed as provisional agent commits.

The atomicity rule is strict: `git apply --check` and `git apply` happen for the patch as one unit.
The orchestrator does not apply one file, then another file, then ask the agent to repair the rest.
A patch either applies as a coherent diff or is rejected as a coherent diff.

If `git apply --check` fails, no part of the diff is applied. The orchestrator sends the patch agent:

- the touched file list;
- the git diagnostic;
- a compact file refresh for the touched files when the agent has a prior orchestrator snapshot;
- instructions to rebase the patch against that refresh, with explicit `@work-leaf read` guidance
  when a full file reread is necessary.

Malformed patch bodies that do not contain recognizable unified diff file headers are rejected with a
protocol prompt asking the agent to resend a complete unified diff.

## Command Write Classification

Agents can ask whether a command is write-producing when they are unsure:

```text
@work-leaf locks classify cargo test
```

`src/locks.rs::CommandWritePolicy` uses a conservative heuristic table for common build, test,
format, package, compiler, and language runtime commands. For example, `cargo test` is treated as
write-producing for `target`, and `cargo fmt` is treated as write-producing for `.`. The classifier is
advice for uncertain cases; agents that know a command may write project files can skip
classification and run it directly through `@work-leaf locks run` with the paths they expect the tool
to touch.

Classification is separate from patch application. It tells the agent which paths require
orchestrator mediation. Commands run through an explicit lock directive:

```text
@work-leaf locks run target -- cargo test
```

`src/orchestrator.rs::handle_agent_directives_streaming` parses the directive, normalizes and
deduplicates the supplied paths, acquires the corresponding write locks, and runs the command in the
project root. Locked command runs have a five-minute default timeout. When a locked command exceeds
that timeout, the orchestrator terminates it, releases the locks, returns a timed-out command result to
the agent, and requires user authorization before a longer lock-holding command is run. The
orchestrator sends the command status, stdout, stderr, timeout state, and locked paths back to the
same agent as `work-leaf command result`. The command output is agent context; manual feature edits
still use the unified-diff patch flow.

The patch-ownership ledger blocks locked commands that directly target another patch agent's focused
test path. Broad validation commands can still run when their broad lock paths include a directory
that contains another patch agent's focused tests, as long as command classification identifies the
command's write output as separate from those focused tests. This lets agents run integration
validation that writes build or cache output while preserving the rule that another agent's focused
tests are not used as local validation for the current patch.

If a locked command leaves tracked file changes under the requested lock paths, those paths are
tracked as pending command changes for that patch agent. The agent cannot finish with
`@work-leaf done` while pending command changes remain. The orchestrator returns the current tracked
diff and asks the agent to either submit that diff through `@work-leaf patch <reason>` or submit a
reverting patch. Matching already-applied diffs are accepted by the patch flow so formatter, build,
test, or generator output can be saved as a provisional patch commit without reapplying the diff.

The command-lock rule is language- and tool-agnostic. Agents use it for any formatter, build, test,
code generator, package manager, installer, cache-producing tool, or repository-required check that
may write files. The agent chooses the command from repository instructions and project context, and
chooses lock paths from the files, directories, caches, build outputs, dependency folders, or lockfiles
that command may write.

## Review Flow

A provisional patch commit records metadata that review and linearization use:

- agent id;
- feature;
- patch reason;
- context describing files and line counts.

`src/patch.rs::GitPatcher::git_commit` creates this metadata commit. `src/review.rs::GitHistory`
reads git history and parses latest commits per patch agent.

The review flow uses that metadata to connect review findings back to the patch agent that produced
the patch. Automatic review starts only after the patch agent has an applied patch and reports
`@work-leaf done`; patch application alone is not a review-readiness signal. Each patch agent uses a
stable `review-<agent-id>` reviewer identity. The review agent must focus only on the reviewed patch.
Reviewer `@work-leaf` directives are resolved in the reviewer conversation before output is
interpreted as findings. If the reviewer finds issues, the orchestrator sends those findings to the
patch agent. The patch agent then continues through the configured read path and patch protocol. When
the reviewer reports no findings, the patch-agent chat asks whether the feature is done. `yes` marks
that patch-agent chat closed, `no` keeps it open, and a later user message in a closed chat reopens
it before the message is sent to the agent.

## Developer Path

A normal development session in default read-permission mode follows this shape:

1. The user runs `./start` in the project directory, or opens `work-leaf` directly so the CLI starts
   its sibling `work-leaf-orchestrator` daemon.
2. The command agent is available as the control surface.
3. The user asks the command agent to create a patch agent for a feature or bug fix.
4. The patch agent asks for file text with `@work-leaf read`.
5. The orchestrator normalizes paths, takes shared read locks, snapshots file text, records the read
   context, and sends the text back to the patch agent.
6. The patch agent reasons over the snapshot and sends one unified diff through `@work-leaf patch`.
7. The orchestrator parses all touched files, takes exclusive write locks for the touched set and the
   repository root path, checks the entire diff, applies the entire diff, stages, and commits the
   provisional patch.
8. If another agent read any touched file and has not cleared that context, the orchestrator sends
   that agent a proactive `work-leaf file update` with fresh file text.
9. The orchestrator returns a patch-applied continuation prompt when the patch agent has not reported
   done.
10. The patch agent runs required checks through locked command directives, commits or reverts any
    tracked command output through the patch protocol, and reports `@work-leaf done` when the patch is
    ready for review.
11. The orchestrator runs or schedules that patch agent's review agent when the patch agent reports
    `@work-leaf done`; the reviewed scope covers the unreviewed provisional commits from that patch
    agent.
12. The review agent reviews only behavior introduced or modified by the patch.
13. The review agent can request file text through the orchestrator before reporting findings.
14. If the review agent reports findings, the orchestrator sends them to the patch agent and the patch
    agent keeps patching.
15. If the review agent reports no findings, the user can mark the review chat as done.
16. Reviewed work from the current command-chat or controller instance can then be linearized into the
    final history.

In direct-read mode, steps 4 and 5 are replaced by direct filesystem inspection from the agent. The
write, review, and linearization steps remain orchestrator-controlled.

## Example Session

The example below shows the default read-permission interaction, not the raw terminal rendering.

The user asks the command agent:

```text
Create a patch agent to add JSON config parsing.
```

The command agent creates a patch agent:

```text
patch agent user-1: JSON config parsing
```

The patch agent requests context:

```text
@work-leaf read src/config.rs src/main.rs Cargo.toml
```

The orchestrator responds to the patch agent:

```text
work-leaf file text

--- src/config.rs ---
<current config source>

--- src/main.rs ---
<current main source>

--- Cargo.toml ---
<current manifest>
```

The patch agent submits one coherent patch:

```text
@work-leaf patch add JSON config parsing
diff --git a/src/config.rs b/src/config.rs
--- a/src/config.rs
+++ b/src/config.rs
@@ ...
...
diff --git a/Cargo.toml b/Cargo.toml
--- a/Cargo.toml
+++ b/Cargo.toml
@@ ...
...
@work-leaf end
```

The orchestrator applies the whole patch under write locks. If it succeeds, the transcript contains
an event like:

```text
applied patch from user-1: add JSON config parsing; commit=<hash>; files=Cargo.toml, src/config.rs
```

If another patch agent previously read `Cargo.toml`, the orchestrator sends that other agent:

```text
work-leaf file update
Another agent changed files you previously read before you submitted a patch.
Rebase any pending patch against the compact file refresh below.

work-leaf file refresh

--- Cargo.toml ---
diff --git a/Cargo.toml b/Cargo.toml
--- a/Cargo.toml
+++ b/Cargo.toml
@@ ...
<snapshot-to-current manifest diff>
```

The orchestrator runs a review agent:

```text
review-user-1: review JSON config parsing patch
```

If the review agent finds a regression introduced by the patch, the orchestrator sends the findings
to `user-1`. The patch agent reads any needed files again and submits a corrective patch through the
same atomic patch path.

If the review agent reports:

```text
NO_FINDINGS
```

the user can mark the review chat as done and proceed toward linearization.

## Source Anchors

The important source symbols for this workflow are:

- `src/agent.rs::PromptPolicy`: injects file-access rules into agent prompts.
- `src/agent.rs::ReadPermission`: selects orchestrator-mediated or direct filesystem read prompts.
- `src/codex.rs::CodexBackend`: launches Codex sessions with injected policy and sends known-session
  follow-ups as raw resume stdin.
- `src/cli.rs::CommandChat`: owns the command surface, backend, file locks, read tracker, and
  directive loop.
- `src/http_controller.rs::HttpControllerServer`: exposes the workspace controller as localhost HTTP
  routes owned by the daemon process.
- `src/http_controller.rs::HttpControllerClient`: sends CLI controller requests to the daemon and
  decodes the same snapshots and events used by local frontends.
- `src/orchestrator.rs::parse_agent_directives`: parses `@work-leaf` protocol directives.
- `src/orchestrator.rs::handle_agent_directives_streaming`: handles reads, patches, command
  classification, sends, done, stale updates, and follow-up routing.
- `src/orchestrator.rs::FileReadTracker`: tracks which agents have outstanding file snapshots.
- `src/orchestrator.rs::read_requested_files`: snapshots file text under read locks.
- `src/locks.rs::FileLockTable`: owns per-path read/write locks and root-safe path normalization.
- `src/locks.rs::CommandWritePolicy`: classifies commands that write project files.
- `src/patch.rs::GitPatcher`: applies whole unified diffs under write locks and creates provisional
  metadata commits.
- `src/review.rs::ReviewCoordinator`: runs reviewer conversations over agent patch commits.
- `src/review.rs::GitHistory`: finds latest agent commits from git history and builds cumulative
  review targets since launch or latest reviewed baselines.
- `src/workspace.rs::WorkLeafController`: exposes UI-neutral orchestration state and events.
- `src/terminal_app.rs::RemoteTerminalApp`: renders the CLI terminal UI through the HTTP controller
  client.
