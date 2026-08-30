#!/usr/bin/env python3

import argparse
import hashlib
import os
from pathlib import Path


WORK_LEAF_SOURCE_SHA256 = "d2487780c63c14021904b8a3c882d54fe231c5846f4a7f57fe955f50201f5644"
DIRECT_SOURCE_SHA256 = "489289165601e00a545f76ab0631d5e0f48d644b928376488677e1875651e386"

REPO_ROOT_LINE = 'repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"'
DIAGNOSTIC_REPO_ROOT_LINE = (
    'repo_root="${WORK_LEAF_DIAGNOSTIC_SOURCE_REPO:?'
    'WORK_LEAF_DIAGNOSTIC_SOURCE_REPO is required}"'
)

WORK_LEAF_START = '''start_benchmark_features() {
  post_feature_command 1 || fail_bench "failed to post first patch-agent command"
  post_feature_command 2 || fail_bench "failed to post second patch-agent command"
  post_feature_command 3 || fail_bench "failed to post third patch-agent command"
  launched_features=3
  observer_timeline feature-start "features=1,2,3" || fail_bench "failed to record observer timeline"
  log "started 3 concurrent patch agents"
}
'''

SEQUENTIAL_WORK_LEAF_START = '''start_benchmark_features() {
  post_feature_command 1 || fail_bench "failed to post first patch-agent command"
  launched_features=1
  observer_timeline feature-start "feature=1 schedule=sequential-diagnostic" \\
    || fail_bench "failed to record observer timeline"
  log "started sequential diagnostic patch agent 1"
}

launch_next_sequential_feature() {
  local next_feature
  next_feature=$((launched_features + 1))
  (( next_feature <= 3 )) || return 1
  post_feature_command "$next_feature" \\
    || fail_bench "failed to post sequential diagnostic patch-agent command $next_feature"
  launched_features="$next_feature"
  feature_stage_deadline=$((now_active + timeout_secs))
  observer_timeline feature-start \\
    "feature=$next_feature schedule=sequential-diagnostic" \\
    || fail_bench "failed to record observer timeline"
  log "started sequential diagnostic patch agent $next_feature"
}
'''

LINEARIZE_CONDITION = '''  if [[ "$linearize_started" == "0" && "$busy" == "false" && "$user_count" == "3" && "$terminal_users" == "$user_count" && "$ready_users" -ge "$patch_agents_with_commits" ]]; then
'''

SEQUENTIAL_ADVANCE = '''  if [[ "$linearize_started" == "0" && "$busy" == "false" \\
    && "$user_count" == "$launched_features" \\
    && "$terminal_users" == "$user_count" \\
    && "$ready_users" -ge "$patch_agents_with_commits" \\
    && "$patch_agents_with_commits" == "$launched_features" \\
    && "$launched_features" -lt 3 ]]; then
    launch_next_sequential_feature
    last_progress_active_seconds="$now_active"
    sleep 5
    continue
  fi

'''

DIRECT_LINEARIZE_PLAN = '''linearize_plan_prompt_sequential() {
  cat <<EOF
You are a normal direct linearizer for a sequential three-feature benchmark.

The repository contains reviewed provisional commits for three distinct requests since $base_commit.
Inspect the history and resulting diff, then propose the final linearization plan. Do not modify
files, rewrite history, create commits, or run the repository check suite in this planning turn.

Final history contract:
- Produce exactly three final commits after the plan is accepted, one final commit for each request.
- Fold each request's implementation, tests, review fixes, validation fixes, and required
  documentation into that request's final commit.
- Do not create separate support, test-hygiene, review-fix, validation-fix, or documentation-only
  commits.
- Keep the rewritten history rooted at $base_commit.
- Preserve all reviewed behavior and follow the repository's commit-message rules.

Documentation and plain-text updates were deferred by implementation agents. Identify any required
updates in the plan so the accepted linearization turn can include them.

End with the proposed three-commit plan and wait for acceptance.
EOF
}
'''

COMPACT_DIRECT_LINEARIZE_PLAN = '''compact_linearize_target_rows() {
  local feature_index request rows
  for feature_index in 1 2 3; do
    request="$(feature_prompt "$feature_index")"
    rows="$(git -C "$checkout_dir" log --reverse \\
      --fixed-strings --grep="direct-agent baseline feature $feature_index" \\
      --format='  - Commit: %H%n    Subject: %s' "$base_commit"..HEAD)"
    [[ -n "$rows" ]] || return 1
    printf 'Feature target %s:\n  Request: %s\n%s\n' "$feature_index" "$request" "$rows"
  done
}

linearize_plan_prompt_sequential() {
  local exact_targets
  exact_targets="$(compact_linearize_target_rows)" \\
    || fail_bench "could not construct compact exact linearization targets"
  cat <<EOF
You are a normal direct linearizer for a sequential three-feature benchmark.

Exact reviewed provisional targets:
Reviewed stack base: $base_commit
$exact_targets

The list above is the complete reviewed target set. Use those exact commits and feature groupings
instead of reconstructing the target set from open-ended history exploration. Inspect the resulting
diff as needed, then propose the final linearization plan. Do not modify files, rewrite history,
create commits, or run the repository check suite in this planning turn.

Final history contract:
- Produce exactly three final commits after the plan is accepted, one final commit for each request.
- Fold each request's implementation, tests, review fixes, validation fixes, and required
  documentation into that request's final commit.
- Do not create separate support, test-hygiene, review-fix, validation-fix, or documentation-only
  commits.
- Keep the rewritten history rooted at $base_commit.
- Preserve all reviewed behavior and follow the repository's commit-message rules.

Documentation and plain-text updates were deferred by implementation agents. Identify any required
updates in the plan so the accepted linearization turn can include them.

End with the proposed three-commit plan and wait for acceptance.
EOF
}
'''


def digest(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def replace_once(text: str, old: str, new: str, description: str) -> str:
    count = text.count(old)
    if count != 1:
        raise ValueError(f"expected one {description}, found {count}")
    return text.replace(old, new, 1)


def generate(mode: str, source: Path) -> str:
    if mode == "sequential-work-leaf":
        expected_sha256 = WORK_LEAF_SOURCE_SHA256
    elif mode == "compact-direct-linearizer":
        expected_sha256 = DIRECT_SOURCE_SHA256
    else:
        raise ValueError(f"unknown diagnostic mode: {mode}")

    actual_sha256 = digest(source)
    if actual_sha256 != expected_sha256:
        raise ValueError(
            f"source SHA-256 differs for {mode}: expected {expected_sha256}, got {actual_sha256}"
        )

    text = source.read_text(encoding="utf-8")
    text = replace_once(text, REPO_ROOT_LINE, DIAGNOSTIC_REPO_ROOT_LINE, "repo root line")

    if mode == "sequential-work-leaf":
        text = replace_once(
            text,
            'readonly feature_schedule="concurrent"',
            'readonly feature_schedule="sequential-diagnostic"',
            "feature schedule",
        )
        text = replace_once(
            text,
            WORK_LEAF_START,
            SEQUENTIAL_WORK_LEAF_START,
            "feature launch function",
        )
        text = replace_once(
            text,
            LINEARIZE_CONDITION,
            SEQUENTIAL_ADVANCE + LINEARIZE_CONDITION,
            "linearize admission condition",
        )
    else:
        text = replace_once(
            text,
            DIRECT_LINEARIZE_PLAN,
            COMPACT_DIRECT_LINEARIZE_PLAN,
            "direct linearize planning function",
        )

    return text


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument(
        "mode", choices=("sequential-work-leaf", "compact-direct-linearizer")
    )
    parser.add_argument("source", type=Path)
    parser.add_argument("output", type=Path)
    arguments = parser.parse_args()

    output = generate(arguments.mode, arguments.source)
    arguments.output.write_text(output, encoding="utf-8")
    os.chmod(arguments.output, 0o700)
    print(arguments.output)


if __name__ == "__main__":
    main()
