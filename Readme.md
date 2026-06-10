I often find my self juggling trough the many codex session opened across even more tmux panes for a
single project. I love my ssds and I'm always short on available space, so many working trees are
kinda scary. The goal of work-leaf is to replicate what instruments like claude-squad offers, but
without using the git work-tree functionality. What I want is an highly opinionated agent orchestrator
for coding. The highly opinionated part is the flow of work: open many agent that change the code
with atomic commits, review with an agent every single commit, patch them, rewrite git history to
have the smallest diff possible and not too many commit, review again agent/human.

## Running

`./start` builds the `work-leaf` and `work-leaf-orchestrator` binaries in release mode, starts the
orchestrator daemon on `127.0.0.1:7878`, and renders the terminal CLI. When the CLI exits, the script
stops the daemon process. Set `WORK_LEAF_START_LISTEN` to choose a different listen address; the
script fails when the requested address is unavailable.

`./smoke-three-features` builds the current release binaries, creates a temporary checkout at the
three-feature smoke-test base commit, and runs `./start` from that temporary checkout. The script
prints the three `:new` commands used by the real-agent smoke and removes the temporary checkout
when Work Leaf exits, fails, or is interrupted. Set `WORK_LEAF_SMOKE_BASE` to choose a different
base commit, or pass daemon options after `--`.

`./bench-three-features` runs the same three-feature scenario through the localhost HTTP API with
the real configured Codex backend. It uses the default mediated-read workflow, records pass/fail,
duration, review and linearize completion, commit churn, code-quality checks, and observed
inefficiencies under `bench-results`, and enables Codex child-process trace output in the saved
daemon artifacts. It always removes its temporary checkout.

`./bench-three-features-sequential` runs a direct-Codex baseline for the same three requests without
Work Leaf orchestration. It implements and reviews one request at a time in a single temporary
checkout, commits after each patch/fix turn, runs a final Codex linearizer, records the same
duration, model, token, review, linearize, code-quality, patch, and binary artifacts, and removes
its temporary checkout.

`./bench-three-features-worktree` runs the direct-Codex worktree baseline. It creates one temporary
Git worktree per request, runs the patch/review loops in parallel, then asks a final Codex
linearizer to merge the reviewed branches into a minimal final history in the integration checkout.
It records the same benchmark data and removes the temporary checkout and worktrees.

`work-leaf-orchestrator` owns the controller, agent backend, locks, review routing, and patch
workflow. It prints `WORK_LEAF_ORCHESTRATOR_URL=http://...` after binding its localhost HTTP API.
`work-leaf` connects to that URL through `WORK_LEAF_ORCHESTRATOR_URL`; when the variable is absent,
the CLI starts the sibling daemon on an ephemeral localhost port and connects to it.
