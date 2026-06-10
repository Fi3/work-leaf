# Stale Snapshot Auto-Rebase

When an agent submits a patch based on an older repository snapshot, the orchestrator does not ask
the agent to resubmit just because `HEAD` moved.

The deterministic rule is based on the files touched by the patch:

1. Work Leaf records the digest of each mediated file snapshot sent to each agent.
2. Accepted patches and locked-command turns do not proactively interrupt other agents with stale
   file updates.
3. When a patch or structured edit fails, Work Leaf reads only the files touched by that submission.
4. If every touched file still matches that agent's latest mediated snapshot, the response tells the
   agent to fix the patch body or context without rebasing for unrelated commits.
5. If a touched file changed, or Work Leaf lacks a prior snapshot for it, the response includes a
   compact refresh for those touched files so the agent can revise against the current file text.

This keeps the safety property that agents do not unknowingly overwrite changed files, while avoiding
token-heavy resubmissions for unrelated concurrent work.

The rule is project-agnostic. It uses file hashes, patch touched-file sets, clean patch application,
and validation results, not repository-specific filenames, commands, or test names.
