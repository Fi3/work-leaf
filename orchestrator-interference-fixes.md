# Orchestrator Interference Fixes

Goal: keep the intentional single-worktree model while preventing patch agents from wasting time on each other's half-finished tests, stale hunks, and duplicate fixes.

## Problems Observed

- Patch agents treated tests introduced by other patch agents as local ownership and edited or re-added them.
- Multiple agents attempted to fix the same compile failure in the same source hunk after one accepted patch already addressed it.
- Large accepted patch bodies remained visible in chat/state, which made agents reason over unrelated concurrent work and made the UI/state payload heavy.
- Patch rejection and rebase prompts did not clearly tell agents that another commit had already fixed the exact hunk.
- Review/linearize is the right integration phase, but patch agents currently do too much cross-agent cleanup before review starts.

## Required Fixes

### 1. Patch Ownership Ledger

Record ownership for every accepted patch commit:

- `agent_id`
- commit hash
- touched files
- changed hunk ranges and stable hunk fingerprints
- added or modified tests when detectable
- patch summary/stat

Use mechanical detection first. If the language or test framework is ambiguous, use a small structured LLM classification step that returns JSON only.

### 2. Generic Test Ownership Policy

Patch-agent instructions should be language-neutral:

- Run checks/tests that existed before the agent's own patch.
- Run checks/tests added by that same agent.
- Do not run another patch agent's focused tests as local validation.
- If a broad existing check now fails in another agent's new test, report it as an integration conflict unless the agent's production code clearly caused the failure.
- Do not edit another agent's tests except for a mechanical compile break directly caused by the agent's own change.

The orchestrator should enforce or warn on this for mediated command execution, regardless of language, build tool, or test runner.

### 3. Test Command Gate

Before executing a patch-agent test/check command, classify it as:

- `own_test`
- `existing_check`
- `broad_integration_check`
- `other_agent_test`
- `ambiguous`

Cheap rules should handle common cases from the ownership ledger. Use a system LLM only for ambiguous commands or unknown languages.

For blocked or risky commands, return a concise message with a safer suggested command or an explanation that the issue belongs to review/linearize.

### 4. Edit Ownership Guard

Before accepting a patch, compare touched files and hunks with the ownership ledger:

- Allow product/source edits needed for the agent's feature.
- Allow edits to tests owned by the same agent.
- Warn or reject edits to another agent's tests unless the patch fixes a mechanical compile break caused by the submitting agent.
- Route duplicated-test cleanup and integration cleanup to review/linearize.

This should be advisory first, then stricter once tests prove the policy works.

### 5. Same-Hunk Collision Handling

When a patch is accepted, mark overlapping pending/rejected hunks as stale.

If another agent submits a patch against the same hunk:

- detect the overlap by hunk fingerprint and file/range
- return a short stale-hunk response
- mention the commit that already changed the hunk
- ask the agent to reread only the affected files
- do not send full neighboring patch bodies

For files or regions with active overlapping changes, consider hunk-level queues so only one agent actively repairs the same region at a time.

### 6. Compact Chat And State Payloads

Do not keep full accepted patch bodies as ordinary visible chat lines.

Store large patches as artifacts and show compact transcript entries:

- title
- commit hash
- file count
- insertion/deletion count
- short summary
- artifact id/path for explicit inspection

The HTTP state and terminal UI should receive compact entries by default. Full patch text should be fetched only on demand.

### 7. Review And Linearize Responsibilities

Patch agents should focus on their feature and their own focused validation.

Review/linearize should own:

- duplicate test cleanup
- cross-agent integration test failures
- docs/plain-text updates
- final broad checks
- final commit organization

Linearize can run broad checks and reconcile tests because it has the integrated view and is expected to touch multiple agents' work.

## Suggested Implementation Order

1. Add ownership metadata to patch history and accepted patch commits.
2. Add tests for ownership recording with fake agents and synthetic patches.
3. Add the generic patch-agent instruction text.
4. Add the test command gate for mediated command execution.
5. Add edit ownership warnings for another agent's tests.
6. Add stale-hunk detection and concise rebase responses.
7. Compact accepted patch transcript/state entries behind artifact lookup.
8. Add bench assertions that report duplicate same-hunk fixes, other-agent test edits, large transcript entries, and time spent before review.

## Bench Acceptance Signals

The three-feature bench should show:

- patch agents do not edit each other's tests except through an explicit allowed integration path
- one accepted same-hunk fix prevents repeated duplicate repair attempts
- patch transcript entries stay compact in HTTP state and terminal chat
- all patch agents reach review without manual intervention
- review starts for every completed patch agent
- linearize includes all reviewed commits
- the final report records duration, token usage, model, backend, read-permission mode, code quality, and remaining inefficiencies
