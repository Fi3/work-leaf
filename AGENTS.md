# DEMAND Repo Guide for Agents

This file exists to make agents productive quickly AND to prevent low-signal, ungrounded output.
When in doubt, inspect the repo and cite concrete file paths + symbols.

# Agent Behavior Contract (IMPORTANT)

## Grounding policy (no hand-waving)
When you describe behavior, you must point to:
- file paths (relative), and ideally symbols (types/functions/modules), and
- the chain of calls or configuration that makes the behavior true.

If uncertain, do more repo inspection until certain. Only leave TODOs if the repo truly lacks
information, and then state exactly what you searched and where.

## Large deliverables (especially documentation)
For any request that produces a large artifact (multi-thousand-line docs, specs, runbooks):
1) Work iteratively: write in chunks to the file; do NOT attempt a single huge response.
2) Use objective gates and do not stop until they pass.
3) Prefer writing to disk and only reporting results in chat.

## Documentation writing rule
When updating documentation anywhere in the repo, including any `README.md` and anything under `./docs`,
agents must describe the system in its current resulting state, not the fact that it was changed.

Do not write documentation in change-log style. In particular, do not use wording such as:
- `now the app does ...`
- `now the lib does ...`
- `this changes ...`
- `this adds ...`
- `it was updated to ...`

Documentation must explain the component as it is, how it works, what it requires, and how it behaves,
as if the reader is seeing the repository in that state for the first time.

The description must be system-oriented, not patch-oriented. 
The place to describe what changed and why is the commit message or PR description, not the documentation itself.
Only exepction is migrations.md file.

## Commit message rules
Every commit message must clearly say what was done and why it was done. When adding new features it
must describe which is the underling logic. Never include things that can be seen with a git diff,
like which file or function are changes unless they are necessary tho explain why something have
been done or the underling logic.

The first line must be written in imperative form and must read like the commit itself is performing
the action.

The first line must start with exactly one of these verbs:
- `ADD`: new feature or addition of something new
- `FIX`: bug fix
- `UPDATE`: improvement to something already present and backward compatible
- `UPGRADE`: improvement or change that is not backward compatible
- `DELETE`: delete something

Do not use other leading verbs.

The first line must be specific and concise, and it must describe both the change and the reason
when possible.

If more detail is needed, add a body after the first line explaining the rationale, constraints,
or important implementation notes, but keep the first line strong enough to stand on its own.

## Bug FIX rules
When asked to fix a bug, always write a test that reproduces the bug, verify that the test fails, and then write the fix.

## New feature rules
When asked to one or more feature, always write a test that test the feature (unit or integration), verify that the test fails, and then write the feature.

## Review rules
Any patch that increases algorithmic complexity to O(n²) or worse must be flagged.

