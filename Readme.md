I often find my self juggling trough the many codex session opened across even more tmux panes for a
single project. I love my ssds and I'm always short on available space, so many working trees are
kinda scary. The goal of work-leaf is to replicate what instruments like claude-squad offers, but
without using the git work-tree functionality. What I want is an highly opinionated agent orchestrator
for coding. The highly opinionated part is the flow of work: open many agent that change the code
with atomic commits, review with an agent every single commit, patch them, rewrite git history to
have the smallest diff possible and not too many commit, review again agent/human.


