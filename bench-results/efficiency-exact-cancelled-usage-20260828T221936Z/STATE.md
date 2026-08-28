# State

- Goal: obtain exact token usage for normal Work Leaf interruptions without modifying Work Leaf.
- Infrastructure: implemented.
- Automated proxy and launcher tests: passing.
- Real completed-response pilot: passing.
- Real Work Leaf interruption pilot: passing with exact usage.
- `cargo fmt`: passing.
- `cargo clippy --all-targets --all-features -- -D warnings`: passing.
- `cargo test --all-targets --all-features`: passing.
- Full direct-versus-Work-Leaf benchmark: not launched.
- Active benchmark or provider process: none.
- Next action: collect a small independent-group batch containing direct Codex, normal concurrent
  Work Leaf, and the already defined all-three-mechanisms-disabled Work Leaf control through this
  same API route, then compare exact tokens and the frozen three-feature quality scores.
