# Stale Snapshot Auto-Rebase Note

When an agent submits a patch based on an older repository snapshot, the orchestrator should not
automatically ask the agent to resubmit just because `HEAD` moved.

The efficient path should be deterministic:

1. Record the file hashes visible to the agent when it read each file.
2. When a patch arrives, compare the current hashes for the files the patch edits.
3. If none of the edited files changed since the agent read them, apply the patch on top of the
   current tree and record that it was auto-rebased over unrelated commits.
4. If an edited file changed, try a normal patch application against the current tree.
5. Ask the agent to rebase or revise only when an edited file changed and the patch does not apply
   cleanly, or when validation fails after applying.

This keeps the safety property that agents do not unknowingly overwrite changed files, while avoiding
token-heavy resubmissions for unrelated concurrent work.

The rule should stay project-agnostic. It must use file hashes, patch touched-file sets, clean patch
application, and validation results, not repository-specific filenames, commands, or test names.
