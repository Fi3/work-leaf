# Scorer Validation

The quality scorer uses only the three behaviors in the original task. It was checked locally
without launching a provider against two fixed revisions:

| Revision | Visual selection | `/status` | Review close/reopen |
| --- | --- | --- | --- |
| Current known implementation (`514b71f`) | pass | pass | pass |
| Benchmark base (`c92a0b7`) | fail | pass | fail |

The first run exposed and corrected one scorer-only wording error: the close/reopen fixture did not
recognize the visible text `feature marked closed`. The fixture accepts that text, `feature closed`,
or removal of the closed agent row. All three current-implementation checks pass after that
correction.

`/status` already works at the benchmark base under the literal original task. At
`c92a0b7`, `src/terminal_app.rs::start_agent_slash_command` opens slash entry for the selected agent,
`TerminalApp::send_chat_buffer` calls the controller's `send_message`, and
`src/workspace.rs::WorkLeafController::send_message` forwards the unchanged text through
`CommandChat::send_to_agent_streaming_with_ids`. The base test
`tests/terminal_app.rs::terminal_app_sends_spawned_codex_slash_command_as_raw_command` confirms that
the Codex backend receives `/status` and its response is displayed.

This means the slash-command row measures an original requested behavior that was already present;
it does not prove that a run added new slash-command code. The benchmark still sends the exact
original request to both workflows. Tightening the requirement or substituting `/fork` would change
the task and is therefore excluded.

The scorer's saved-output materialization is also covered locally. A test reconstructs a candidate
from its Git bundle plus an untracked-file diff and verifies both files. Token scoring reads
`observation/analysis.json -> usage_scopes.total_workflow`, verifies the model and effort strata,
and requires the copied totals in `report.json` to match.

## Pilot Application

Applied to the two saved pilot candidates, the scorer reports Work Leaf at 2/3 and direct Codex at
3/3. Work Leaf consistently fails the completion question: the same saved candidate failed that
fixture in five additional offline runs. Direct Codex's one driver-gate failure appears unrelated to
the three feature fixtures: the exact failed repository test passed ten out of ten additional runs
against the unchanged saved candidate. The original pass/fail records remain preserved.
