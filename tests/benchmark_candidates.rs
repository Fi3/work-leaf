#![cfg(unix)]

use std::fs;
use std::io::Write;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::process::{Command, Output, Stdio};
use std::sync::atomic::{AtomicUsize, Ordering};

#[test]
fn artifact_transactions_remove_owned_staging_on_abort_and_commit_failure() {
    let root = test_dir("artifact-transaction-cleanup");
    let results = root.path().join("results");
    fs::create_dir(&results).unwrap();

    let output = bash_harness(
        concat!(
            "set -euo pipefail\n",
            "source \"$1/bench-candidate-common\"\n",
            "results=$2\n",
            "bench_begin_artifact_transaction \"$results\" abort-artifacts abort.md abort.jsonl\n",
            "mkdir -p \"$bench_artifact_transaction_stage_fd_path/nested\"\n",
            "printf 'private bytes\\n' > \"$bench_artifact_transaction_stage_fd_path/nested/value\"\n",
            "bench_abort_artifact_transaction\n",
            "if compgen -G \"$results/.bench-artifact-publish.*\" >/dev/null; then exit 94; fi\n",
            "bench_begin_artifact_transaction \"$results\" failed-artifacts failed.md failed.jsonl\n",
            "stage=$bench_artifact_transaction_stage_fd_path\n",
            "printf 'human report\\n' > \"$stage/.report.md\"\n",
            "printf '{invalid json\\n' > \"$stage/report.json\"\n",
            "if bench_commit_artifact_transaction .report.md; then exit 91; fi\n",
            "[[ \"$bench_artifact_transaction_active\" == 0 ]]\n",
            "if compgen -G \"$results/.bench-artifact-publish.*\" >/dev/null; then exit 95; fi\n",
            "mkdir \"$results/unowned-stage\"\n",
            "printf 'must remain\\n' > \"$results/unowned-stage/sentinel\"\n",
            "exec {owner_fd}<\"$results\"\n",
            "owner_fd_path=$(bench_directory_fd_path \"$owner_fd\")\n",
            "owner_identity=$(stat -Lc '%d:%i' \"$owner_fd_path\")\n",
            "entry_identity=$(stat -c '%d:%i' \"$results/unowned-stage\")\n",
            "if bench_remove_owned_directory \"$owner_fd_path\" \"$owner_identity\" \\\n",
            "  unowned-stage \"$entry_identity\"; then exit 98; fi\n",
            "[[ \"$(cat \"$results/unowned-stage/sentinel\")\" == 'must remain' ]]\n",
        ),
        &[results.as_os_str()],
    );

    assert_success(&output, "artifact transaction cleanup");
}

#[test]
fn artifact_abort_does_not_remove_a_stage_after_its_name_changes() {
    let root = test_dir("artifact-transaction-cleanup-race");
    let results = root.path().join("results");
    fs::create_dir(&results).unwrap();

    let output = bash_harness(
        concat!(
            "set -euo pipefail\n",
            "source \"$1/bench-candidate-common\"\n",
            "results=$2\n",
            "bench_begin_artifact_transaction \"$results\" output-artifacts output.md output.jsonl\n",
            "printf 'preserved bytes\\n' > \"$bench_artifact_transaction_stage_fd_path/sentinel\"\n",
            "python3 - \"$bench_artifact_transaction_root_fd_path\" \\\n",
            "  \"$bench_artifact_transaction_stage_name\" <<'PY'\n",
            "import os\n",
            "import sys\n",
            "root_fd = os.open(sys.argv[1], os.O_RDONLY | os.O_DIRECTORY)\n",
            "try:\n",
            "    os.rename(sys.argv[2], \"renamed-staging\", src_dir_fd=root_fd, dst_dir_fd=root_fd)\n",
            "finally:\n",
            "    os.close(root_fd)\n",
            "PY\n",
            "bench_abort_artifact_transaction\n",
            "[[ \"$(cat \"$results/renamed-staging/sentinel\")\" == 'preserved bytes' ]]\n",
        ),
        &[results.as_os_str()],
    );

    assert_success(&output, "renamed artifact staging safety");
}

#[test]
fn artifact_cleanup_preserves_staging_beyond_its_bounded_depth() {
    let root = test_dir("artifact-transaction-cleanup-depth");
    let results = root.path().join("results");
    fs::create_dir(&results).unwrap();

    let output = bash_harness(
        concat!(
            "set -euo pipefail\n",
            "source \"$1/bench-candidate-common\"\n",
            "results=$2\n",
            "bench_begin_artifact_transaction \"$results\" output-artifacts output.md output.jsonl\n",
            "stage_path=$results/$bench_artifact_transaction_stage_name\n",
            "cursor=$bench_artifact_transaction_stage_fd_path\n",
            "relative=\n",
            "for index in {1..66}; do\n",
            "  mkdir \"$cursor/level-$index\"\n",
            "  cursor=$cursor/level-$index\n",
            "  relative=${relative:+$relative/}level-$index\n",
            "done\n",
            "printf 'preserved deep bytes\\n' > \"$cursor/sentinel\"\n",
            "bench_abort_artifact_transaction\n",
            "[[ \"$(cat \"$stage_path/$relative/sentinel\")\" == 'preserved deep bytes' ]]\n",
        ),
        &[results.as_os_str()],
    );

    assert_success(&output, "bounded artifact staging cleanup depth");
}

#[test]
fn rejected_candidate_publication_removes_only_its_owned_staging() {
    let root = test_dir("candidate-publication-cleanup");
    let artifact = root.path().join("artifact");
    let source = root.path().join("source-candidate");
    let external = root.path().join("external");
    fs::create_dir(&artifact).unwrap();
    fs::create_dir(&source).unwrap();
    fs::create_dir(&external).unwrap();
    fs::write(source.join("payload"), "candidate bytes\n").unwrap();
    fs::write(external.join("sentinel"), "external bytes\n").unwrap();
    let invalid_admission = root.path().join("invalid-admission.json");
    fs::write(&invalid_admission, "{}\n").unwrap();

    let output = bash_harness(
        concat!(
            "set -euo pipefail\n",
            "source \"$1/bench-candidate-common\"\n",
            "if bench_publish_candidate_runtime \"$2\" \"$3\" \"$4\"; then exit 92; fi\n",
            "[[ ! -e \"$2/candidate\" && ! -L \"$2/candidate\" ]]\n",
            "if compgen -G \"$2/.candidate.publish.*\" >/dev/null; then exit 96; fi\n",
            "[[ \"$(cat \"$5/sentinel\")\" == 'external bytes' ]]\n",
        ),
        &[
            artifact.as_os_str(),
            source.as_os_str(),
            invalid_admission.as_os_str(),
            external.as_os_str(),
        ],
    );

    assert_success(&output, "rejected candidate publication cleanup");
}

#[test]
fn candidate_publication_rolls_back_a_post_rename_admission_failure() {
    let root = test_dir("candidate-publication-post-rename-cleanup");
    let artifact = root.path().join("artifact");
    let source = root.path().join("source-candidate");
    fs::create_dir(&artifact).unwrap();
    fs::create_dir(&source).unwrap();
    fs::write(source.join("payload"), "candidate bytes\n").unwrap();
    let admission = root.path().join("admission.json");
    fs::write(&admission, "{}\n").unwrap();

    let output = bash_harness(
        concat!(
            "set -euo pipefail\n",
            "source \"$1/bench-candidate-common\"\n",
            "admission_calls=0\n",
            "bench_candidate_paths_are_runnable() {\n",
            "  admission_calls=$((admission_calls + 1))\n",
            "  [[ \"$admission_calls\" == 1 ]]\n",
            "}\n",
            "if bench_publish_candidate_runtime \"$2\" \"$3\" \"$4\"; then exit 89; fi\n",
            "[[ ! -e \"$2/candidate\" && ! -L \"$2/candidate\" ]]\n",
            "if compgen -G \"$2/.candidate.publish.*\" >/dev/null; then exit 90; fi\n",
        ),
        &[
            artifact.as_os_str(),
            source.as_os_str(),
            admission.as_os_str(),
        ],
    );

    assert_success(&output, "post-rename candidate publication rollback");
}

#[test]
fn owned_candidate_publication_never_replaces_an_existing_destination() {
    let root = test_dir("candidate-publication-no-replace");
    let artifact = root.path().join("artifact");
    let staging = artifact.join(".candidate.materialize.ABC123");
    let candidate = artifact.join("candidate");
    fs::create_dir_all(&staging).unwrap();
    fs::create_dir(&candidate).unwrap();
    fs::write(staging.join("sentinel"), "staging bytes\n").unwrap();
    fs::write(candidate.join("sentinel"), "existing bytes\n").unwrap();

    let output = bash_harness(
        concat!(
            "set -euo pipefail\n",
            "source \"$1/bench-candidate-common\"\n",
            "artifact=$2\n",
            "source_name=.candidate.materialize.ABC123\n",
            "exec {owner_fd}<\"$artifact\"\n",
            "owner_fd_path=$(bench_directory_fd_path \"$owner_fd\")\n",
            "owner_identity=$(stat -Lc '%d:%i' \"$owner_fd_path\")\n",
            "source_identity=$(stat -c '%d:%i' \"$owner_fd_path/$source_name\")\n",
            "if bench_publish_owned_directory_noreplace \"$owner_fd_path\" \\\n",
            "  \"$owner_identity\" \"$source_name\" \"$source_identity\" candidate; then exit 88; fi\n",
            "[[ \"$(cat \"$artifact/$source_name/sentinel\")\" == 'staging bytes' ]]\n",
            "[[ \"$(cat \"$artifact/candidate/sentinel\")\" == 'existing bytes' ]]\n",
        ),
        &[artifact.as_os_str()],
    );

    assert_success(&output, "candidate publication no-replace race guard");
}

#[test]
fn publication_cleanup_does_not_follow_a_replaced_staging_name() {
    let root = test_dir("candidate-publication-cleanup-race");
    let artifact = root.path().join("artifact");
    let source = root.path().join("source-candidate");
    let external = root.path().join("external");
    let renamed = artifact.join("renamed-staging");
    let fake_bin = root.path().join("fake-bin");
    fs::create_dir(&artifact).unwrap();
    fs::create_dir(&source).unwrap();
    fs::create_dir(&external).unwrap();
    fs::create_dir(&fake_bin).unwrap();
    fs::write(source.join("payload"), "candidate bytes\n").unwrap();
    fs::write(external.join("sentinel"), "external bytes\n").unwrap();
    let invalid_admission = root.path().join("invalid-admission.json");
    fs::write(&invalid_admission, "{}\n").unwrap();
    let real_find = command_path("find");
    write_executable(
        &fake_bin.join("find"),
        concat!(
            "#!/usr/bin/env bash\n",
            "set -euo pipefail\n",
            "shopt -s nullglob\n",
            "stages=(\"$ARTIFACT\"/.candidate.publish.*)\n",
            "if [[ \"${#stages[@]}\" == 1 && ! -e \"$ATTACK_ONCE\" ]]; then\n",
            "  : > \"$ATTACK_ONCE\"\n",
            "  mv -- \"${stages[0]}\" \"$RENAMED\"\n",
            "  ln -s \"$EXTERNAL\" \"${stages[0]}\"\n",
            "fi\n",
            "exec \"$REAL_FIND\" \"$@\"\n",
        ),
    );

    let output = Command::new("bash")
        .arg("-c")
        .arg(concat!(
            "set -euo pipefail\n",
            "source \"$1/bench-candidate-common\"\n",
            "if bench_publish_candidate_runtime \"$2\" \"$3\" \"$4\"; then exit 97; fi\n",
            "[[ ! -e \"$2/candidate\" && ! -L \"$2/candidate\" ]]\n",
        ))
        .arg("bash")
        .arg(env!("CARGO_MANIFEST_DIR"))
        .arg(&artifact)
        .arg(&source)
        .arg(&invalid_admission)
        .env(
            "PATH",
            format!("{}:{}", fake_bin.display(), std::env::var("PATH").unwrap()),
        )
        .env("REAL_FIND", real_find)
        .env("ARTIFACT", &artifact)
        .env("RENAMED", &renamed)
        .env("EXTERNAL", &external)
        .env("ATTACK_ONCE", root.path().join("attack-once"))
        .output()
        .unwrap();
    assert_success(&output, "candidate staging replacement rejection");
    assert_eq!(
        fs::read(renamed.join("payload")).unwrap(),
        b"candidate bytes\n"
    );
    assert_eq!(
        fs::read(external.join("sentinel")).unwrap(),
        b"external bytes\n"
    );
}

#[test]
fn regular_file_publication_cleanup_preserves_a_path_replacement() {
    let root = test_dir("regular-file-publication-cleanup-race");
    let owner = root.path().join("owner");
    let fake_bin = root.path().join("fake-bin");
    let source = root.path().join("source.json");
    let renamed_staging = root.path().join("renamed-owned-staging");
    let attack_marker = root.path().join("attack-ran");
    fs::create_dir(&owner).unwrap();
    fs::create_dir(&fake_bin).unwrap();
    fs::write(&source, "owned staging bytes\n").unwrap();

    let real_cmp = command_path("cmp");
    let real_python = command_path("python3");
    let real_rm = command_path("rm");
    write_executable(
        &fake_bin.join("cmp"),
        concat!(
            "#!/usr/bin/env bash\n",
            "set -euo pipefail\n",
            "if [[ ! -e \"$TARGET_PATH\" ]]; then mkdir -- \"$TARGET_PATH\"; fi\n",
            "exec \"$REAL_CMP\" \"$@\"\n",
        ),
    );
    write_executable(
        &fake_bin.join("python3"),
        concat!(
            "#!/usr/bin/env bash\n",
            "set -euo pipefail\n",
            "if [[ \"${1:-}\" != - ]]; then exec \"$REAL_PYTHON\" \"$@\"; fi\n",
            "program=$(mktemp)\n",
            "trap 'rm -f -- \"$program\"' EXIT\n",
            "cat > \"$program\"\n",
            "if grep -q '[.]bench-file-cleanup[.]' \"$program\" \\\n",
            "  && [[ ! -e \"$ATTACK_MARKER\" ]]; then\n",
            "  stages=(\"$OWNER\"/.inventory.json.publish.*)\n",
            "  [[ \"${#stages[@]}\" == 1 ]]\n",
            "  mv -- \"${stages[0]}\" \"$RENAMED_STAGING\"\n",
            "  printf 'unowned replacement bytes\\n' > \"${stages[0]}\"\n",
            "  : > \"$ATTACK_MARKER\"\n",
            "fi\n",
            "set +e\n",
            "\"$REAL_PYTHON\" \"$program\" \"${@:2}\"\n",
            "status=$?\n",
            "set -e\n",
            "\"$REAL_RM\" -f -- \"$program\"\n",
            "exit \"$status\"\n",
        ),
    );
    write_executable(
        &fake_bin.join("rm"),
        concat!(
            "#!/usr/bin/env bash\n",
            "set -euo pipefail\n",
            "entry=${!#}\n",
            "if [[ \"$entry\" == \"$OWNER\"/.inventory.json.publish.* \\\n",
            "  && ! -e \"$ATTACK_MARKER\" ]]; then\n",
            "  mv -- \"$entry\" \"$RENAMED_STAGING\"\n",
            "  printf 'unowned replacement bytes\\n' > \"$entry\"\n",
            "  : > \"$ATTACK_MARKER\"\n",
            "fi\n",
            "exec \"$REAL_RM\" \"$@\"\n",
        ),
    );

    let output = Command::new("bash")
        .arg("-c")
        .arg(concat!(
            "set -euo pipefail\n",
            "source \"$1/bench-candidate-common\"\n",
            "if bench_publish_regular_file \"$2\" \"$3\" inventory.json; then exit 96; fi\n",
            "[[ -f \"$ATTACK_MARKER\" ]]\n",
            "stage=(\"$2\"/.inventory.json.publish.*)\n",
            "[[ \"${#stage[@]}\" == 1 ]]\n",
            "[[ \"$(cat \"${stage[0]}\")\" == 'unowned replacement bytes' ]]\n",
            "[[ \"$(cat \"$RENAMED_STAGING\")\" == 'owned staging bytes' ]]\n",
            "[[ -d \"$TARGET_PATH\" ]]\n",
        ))
        .arg("bash")
        .arg(env!("CARGO_MANIFEST_DIR"))
        .arg(&owner)
        .arg(&source)
        .env(
            "PATH",
            format!("{}:{}", fake_bin.display(), std::env::var("PATH").unwrap()),
        )
        .env("OWNER", &owner)
        .env("TARGET_PATH", owner.join("inventory.json"))
        .env("RENAMED_STAGING", &renamed_staging)
        .env("ATTACK_MARKER", &attack_marker)
        .env("REAL_CMP", real_cmp)
        .env("REAL_PYTHON", real_python)
        .env("REAL_RM", real_rm)
        .output()
        .unwrap();

    assert_success(&output, "regular-file cleanup pathname replacement safety");
}

#[test]
fn regular_file_cleanup_never_overwrites_a_quarantine_replacement() {
    let root = test_dir("regular-file-cleanup-quarantine-race");
    let owner = root.path().join("owner");
    let fake_bin = root.path().join("fake-bin");
    let staging = owner.join(".inventory.json.publish.ATTACK");
    let attack_marker = root.path().join("attack-ran");
    let trace_runner = root.path().join("trace-python.py");
    fs::create_dir(&owner).unwrap();
    fs::create_dir(&fake_bin).unwrap();
    fs::write(&staging, "owned staging bytes\n").unwrap();
    fs::write(
        &trace_runner,
        concat!(
            "import linecache\n",
            "import os\n",
            "import sys\n",
            "program = sys.argv[1]\n",
            "arguments = sys.argv[2:]\n",
            "attacked = False\n",
            "def trace(frame, event, arg):\n",
            "    global attacked\n",
            "    if event != 'line' or frame.f_code.co_filename != program or attacked:\n",
            "        return trace\n",
            "    line = linecache.getline(program, frame.f_lineno)\n",
            "    if ('os.rename(entry_name, \"entry\"' not in line\n",
            "            and 'rename_no_replace(owner_fd, entry_name' not in line):\n",
            "        return trace\n",
            "    fd = os.open(\"entry\", os.O_WRONLY | os.O_CREAT | os.O_EXCL, 0o600,\n",
            "                 dir_fd=frame.f_locals[\"quarantine_fd\"])\n",
            "    try:\n",
            "        os.write(fd, b\"unowned quarantine bytes\\n\")\n",
            "    finally:\n",
            "        os.close(fd)\n",
            "    open(os.environ[\"ATTACK_MARKER\"], \"wb\").close()\n",
            "    attacked = True\n",
            "    return trace\n",
            "sys.argv = [program, *arguments]\n",
            "sys.settrace(trace)\n",
            "with open(program, \"rb\") as source:\n",
            "    code = compile(source.read(), program, \"exec\")\n",
            "exec(code, {\"__name__\": \"__main__\", \"__file__\": program})\n",
        ),
    )
    .unwrap();
    let real_python = command_path("python3");
    write_executable(
        &fake_bin.join("python3"),
        concat!(
            "#!/usr/bin/env bash\n",
            "set -euo pipefail\n",
            "if [[ \"${1:-}\" != - ]]; then exec \"$REAL_PYTHON\" \"$@\"; fi\n",
            "program=$(mktemp)\n",
            "cat > \"$program\"\n",
            "set +e\n",
            "\"$REAL_PYTHON\" \"$TRACE_RUNNER\" \"$program\" \"${@:2}\"\n",
            "status=$?\n",
            "set -e\n",
            "rm -f -- \"$program\"\n",
            "exit \"$status\"\n",
        ),
    );

    let output = Command::new("bash")
        .arg("-c")
        .arg(concat!(
            "set -euo pipefail\n",
            "source \"$1/bench-candidate-common\"\n",
            "exec {owner_fd}<\"$2\"\n",
            "owner_fd_path=$(bench_directory_fd_path \"$owner_fd\")\n",
            "owner_identity=$(stat -Lc '%d:%i' \"$owner_fd_path\")\n",
            "entry_identity=$(stat -c '%d:%i' \"$3\")\n",
            "if bench_remove_owned_regular_file \"$owner_fd_path\" \"$owner_identity\" \\\n",
            "  .inventory.json.publish.ATTACK \"$entry_identity\"; then exit 97; fi\n",
            "[[ -f \"$ATTACK_MARKER\" ]]\n",
            "[[ \"$(cat \"$3\")\" == 'owned staging bytes' ]]\n",
            "quarantined=(\"$2\"/.bench-file-cleanup.*/entry)\n",
            "[[ \"${#quarantined[@]}\" == 1 ]]\n",
            "[[ \"$(cat \"${quarantined[0]}\")\" == 'unowned quarantine bytes' ]]\n",
        ))
        .arg("bash")
        .arg(env!("CARGO_MANIFEST_DIR"))
        .arg(&owner)
        .arg(&staging)
        .env(
            "PATH",
            format!("{}:{}", fake_bin.display(), std::env::var("PATH").unwrap()),
        )
        .env("REAL_PYTHON", real_python)
        .env("TRACE_RUNNER", &trace_runner)
        .env("ATTACK_MARKER", &attack_marker)
        .output()
        .unwrap();

    assert_success(
        &output,
        "regular-file cleanup quarantine replacement safety",
    );
}

#[test]
fn regular_file_cleanup_never_unlinks_a_quarantine_replacement() {
    let root = test_dir("regular-file-cleanup-terminal-quarantine-race");
    let owner = root.path().join("owner");
    let fake_bin = root.path().join("fake-bin");
    let staging = owner.join(".inventory.json.publish.UNLINK");
    let attack_marker = root.path().join("attack-ran");
    let trace_runner = root.path().join("trace-python.py");
    fs::create_dir(&owner).unwrap();
    fs::create_dir(&fake_bin).unwrap();
    fs::write(&staging, "owned staging bytes\n").unwrap();
    fs::write(
        &trace_runner,
        concat!(
            "import linecache\n",
            "import os\n",
            "import sys\n",
            "program = sys.argv[1]\n",
            "arguments = sys.argv[2:]\n",
            "attacked = False\n",
            "def trace(frame, event, arg):\n",
            "    global attacked\n",
            "    if event != 'line' or frame.f_code.co_filename != program or attacked:\n",
            "        return trace\n",
            "    line = linecache.getline(program, frame.f_lineno)\n",
            "    if ('os.unlink(\"entry\"' not in line\n",
            "            and 'terminal_quarantine = ' not in line):\n",
            "        return trace\n",
            "    quarantine_fd = frame.f_locals[\"quarantine_fd\"]\n",
            "    os.rename(\"entry\", \"owned-entry\", src_dir_fd=quarantine_fd,\n",
            "              dst_dir_fd=quarantine_fd)\n",
            "    fd = os.open(\"entry\", os.O_WRONLY | os.O_CREAT | os.O_EXCL, 0o600,\n",
            "                 dir_fd=quarantine_fd)\n",
            "    try:\n",
            "        os.write(fd, b\"unowned quarantine bytes\\n\")\n",
            "    finally:\n",
            "        os.close(fd)\n",
            "    open(os.environ[\"ATTACK_MARKER\"], \"wb\").close()\n",
            "    attacked = True\n",
            "    return trace\n",
            "sys.argv = [program, *arguments]\n",
            "sys.settrace(trace)\n",
            "with open(program, \"rb\") as source:\n",
            "    code = compile(source.read(), program, \"exec\")\n",
            "exec(code, {\"__name__\": \"__main__\", \"__file__\": program})\n",
        ),
    )
    .unwrap();
    let real_python = command_path("python3");
    write_executable(
        &fake_bin.join("python3"),
        concat!(
            "#!/usr/bin/env bash\n",
            "set -euo pipefail\n",
            "if [[ \"${1:-}\" != - ]]; then exec \"$REAL_PYTHON\" \"$@\"; fi\n",
            "program=$(mktemp)\n",
            "cat > \"$program\"\n",
            "set +e\n",
            "\"$REAL_PYTHON\" \"$TRACE_RUNNER\" \"$program\" \"${@:2}\"\n",
            "status=$?\n",
            "set -e\n",
            "rm -f -- \"$program\"\n",
            "exit \"$status\"\n",
        ),
    );

    let output = Command::new("bash")
        .arg("-c")
        .arg(concat!(
            "set -euo pipefail\n",
            "shopt -s nullglob\n",
            "source \"$1/bench-candidate-common\"\n",
            "exec {owner_fd}<\"$2\"\n",
            "owner_fd_path=$(bench_directory_fd_path \"$owner_fd\")\n",
            "owner_identity=$(stat -Lc '%d:%i' \"$owner_fd_path\")\n",
            "entry_identity=$(stat -c '%d:%i' \"$3\")\n",
            "bench_remove_owned_regular_file \"$owner_fd_path\" \"$owner_identity\" \\\n",
            "  .inventory.json.publish.UNLINK \"$entry_identity\" || true\n",
            "[[ -f \"$ATTACK_MARKER\" ]]\n",
            "quarantines=(\"$2\"/.bench-file-cleanup.*)\n",
            "[[ \"${#quarantines[@]}\" == 1 ]]\n",
            "[[ \"$(cat \"${quarantines[0]}/owned-entry\")\" == 'owned staging bytes' ]]\n",
            "[[ \"$(cat \"${quarantines[0]}/entry\")\" == 'unowned quarantine bytes' ]]\n",
        ))
        .arg("bash")
        .arg(env!("CARGO_MANIFEST_DIR"))
        .arg(&owner)
        .arg(&staging)
        .env(
            "PATH",
            format!("{}:{}", fake_bin.display(), std::env::var("PATH").unwrap()),
        )
        .env("REAL_PYTHON", real_python)
        .env("TRACE_RUNNER", &trace_runner)
        .env("ATTACK_MARKER", &attack_marker)
        .output()
        .unwrap();

    assert_success(
        &output,
        "regular-file terminal quarantine replacement safety",
    );

    let implementation = fs::read_to_string("bench-candidate-common").unwrap();
    let helper = implementation
        .split_once("bench_remove_owned_regular_file()")
        .unwrap()
        .1
        .split_once("bench_publish_owned_directory_noreplace()")
        .unwrap()
        .0;
    assert!(!helper.contains("os.unlink("));
    assert!(!helper.contains("os.rmdir("));
}

#[test]
fn candidate_admission_uses_structure_provenance_and_digests_not_ui_text() {
    for path in [
        "bench-candidate-common",
        "bench-three-features",
        "bench-three-features-direct-common",
        "materialize-bench-candidate",
    ] {
        let implementation = fs::read_to_string(path).unwrap();
        assert!(implementation.contains("bounded-startup-quit"));
        assert!(!implementation.contains("Command chat:"));
    }
    let admission = fs::read_to_string("bench-candidate-common").unwrap();
    assert!(admission.contains("work-leaf-benchmark-candidate-smoke-v1"));
    assert!(admission.contains("scripted-command-chat"));
    for path in [
        "bench-three-features",
        "bench-three-features-direct-common",
        "materialize-bench-candidate",
    ] {
        assert!(
            !fs::read_to_string(path)
                .unwrap()
                .contains("scripted-command-chat")
        );
    }

    let root = test_dir("candidate-admission-contract");
    let artifact = root.path().join("artifact");
    fs::create_dir_all(artifact.join("candidate/bin")).unwrap();

    let output = bash_harness(
        concat!(
            "set -euo pipefail\n",
            "source \"$1/bench-candidate-common\"\n",
            "artifact=$2\n",
            "candidate=$artifact/candidate\n",
            "cat > \"$candidate/bin/work-leaf\" <<'BIN'\n",
            "#!/usr/bin/env bash\n",
            "IFS= read -r command\n",
            "[[ \"$command\" == quit ]]\n",
            "printf 'application startup completed\\n'\n",
            "BIN\n",
            "chmod 0755 \"$candidate/bin/work-leaf\"\n",
            "printf '[\"work-leaf\"]\\n' > \"$candidate/SOURCE_TARGETS.json\"\n",
            "printf 'application startup completed\\n' > \"$candidate/SMOKE.stdout\"\n",
            ": > \"$candidate/SMOKE.stderr\"\n",
            "binary_digest=$(bench_sha256_file \"$candidate/bin/work-leaf\")\n",
            "target_digest=$(bench_sha256_file \"$candidate/SOURCE_TARGETS.json\")\n",
            "stdout_digest=$(bench_sha256_file \"$candidate/SMOKE.stdout\")\n",
            "stderr_digest=$(bench_sha256_file \"$candidate/SMOKE.stderr\")\n",
            "stdin_digest=$(printf 'quit\\n' | sha256sum | awk '{print $1}')\n",
            "printf '%s  bin/work-leaf\\n' \"$binary_digest\" > \"$candidate/SHA256SUMS\"\n",
            "jq -n --arg schema \"$bench_candidate_schema\" \\\n",
            "  --arg smoke_schema \"$bench_candidate_smoke_schema\" \\\n",
            "  --arg binary_digest \"$binary_digest\" --arg target_digest \"$target_digest\" \\\n",
            "  --arg stdout_digest \"$stdout_digest\" --arg stderr_digest \"$stderr_digest\" \\\n",
            "  --arg stdin_digest \"$stdin_digest\" '{\n",
            "    schema:$schema,\n",
            "    source_commit:\"0123456789abcdef0123456789abcdef01234567\",\n",
            "    entrypoint:\"candidate/bin/work-leaf\",\n",
            "    executables:[{path:\"candidate/bin/work-leaf\",sha256:$binary_digest}],\n",
            "    targets:{path:\"candidate/SOURCE_TARGETS.json\",sha256:$target_digest},\n",
            "    smoke:{schema:$smoke_schema,gate:\"bounded-startup-quit\",\n",
            "      command:[\"candidate/bin/work-leaf\"],stdin_sha256:$stdin_digest,\n",
            "      timeout_seconds:20,exit_code:0,\n",
            "      stdout:{path:\"candidate/SMOKE.stdout\",sha256:$stdout_digest},\n",
            "      stderr:{path:\"candidate/SMOKE.stderr\",sha256:$stderr_digest}}\n",
            "  }' > \"$candidate/PROVENANCE\"\n",
            "candidate_json=$(jq -c . \"$candidate/PROVENANCE\")\n",
            "jq -n --argjson candidate \"$candidate_json\" '{\n",
            "  run_id:\"admission-fixture\",result:\"pass\",commits_after_base:1,\n",
            "  changed_files:1,candidate:$candidate\n",
            "}' > \"$artifact/report.json\"\n",
            "bench_candidate_is_runnable \"$artifact\"\n",
            "printf 'tampered\\n' >> \"$candidate/bin/work-leaf\"\n",
            "if bench_candidate_is_runnable \"$artifact\"; then exit 93; fi\n",
        ),
        &[artifact.as_os_str()],
    );

    assert_success(&output, "candidate structural admission");
}

#[test]
fn materializer_verifies_evidence_and_cleans_failed_build_staging() {
    let root = test_dir("materializer-failure-cleanup");
    let fixture = HistoricalFixture::new(root.path());
    let fake_cargo = root.path().join("fake-cargo");
    write_fake_cargo(&fake_cargo, false);

    let output = fixture.run_materializer(&fake_cargo);
    assert!(
        !output.status.success(),
        "failing build unexpectedly materialized"
    );
    assert!(!fixture.artifact.join("candidate").exists());
    assert_no_private_materialization_dirs(&fixture.artifact);

    let tampered_root = test_dir("materializer-evidence-rejection");
    let tampered = HistoricalFixture::new(tampered_root.path());
    let saved_patch = fs::read_to_string(&tampered.patch).unwrap();
    let changed_patch = saved_patch.replacen("candidate feature", "tampered candidate", 1);
    assert_ne!(changed_patch, saved_patch);
    fs::write(&tampered.patch, changed_patch).unwrap();
    let cargo_marker = tampered_root.path().join("cargo-was-run");
    write_executable(
        &cargo_marker,
        "#!/usr/bin/env bash\nprintf invoked > \"$CARGO_MARKER\"\nexit 99\n",
    );
    let output = tampered
        .command(&cargo_marker)
        .env("CARGO_MARKER", tampered_root.path().join("marker"))
        .output()
        .unwrap();
    assert!(
        !output.status.success(),
        "tampered evidence unexpectedly verified"
    );
    assert!(!tampered_root.path().join("marker").exists());
    assert!(!tampered.artifact.join("candidate").exists());
    assert_no_private_materialization_dirs(&tampered.artifact);
}

#[test]
fn materializer_accepts_a_bounded_startup_quit_without_fixed_ui_output() {
    let root = test_dir("materializer-structural-smoke");
    let fixture = HistoricalFixture::new(root.path());
    let fake_cargo = root.path().join("fake-cargo");
    write_fake_cargo(&fake_cargo, true);

    let output = fixture.run_materializer(&fake_cargo);
    assert_success(&output, "historical materialization");
    let candidate = fixture.artifact.join("candidate");
    let provenance: serde_json::Value =
        serde_json::from_slice(&fs::read(candidate.join("PROVENANCE")).unwrap()).unwrap();
    assert_eq!(
        provenance["smoke"]["schema"],
        "work-leaf-benchmark-candidate-smoke-v2"
    );
    assert_eq!(provenance["smoke"]["gate"], "bounded-startup-quit");
    let stdout = fs::read_to_string(candidate.join("SMOKE.stdout")).unwrap();
    assert_eq!(stdout, "application startup completed\n");
    assert!(!stdout.contains("Command chat:"));
    assert_no_private_materialization_dirs(&fixture.artifact);
}

#[test]
fn admission_accepts_exact_historical_smoke_metadata_without_reading_ui_text() {
    let root = test_dir("historical-smoke-admission");
    let fixture = HistoricalFixture::new(root.path());
    let fake_cargo = root.path().join("fake-cargo");
    write_fake_cargo(&fake_cargo, true);

    let output = fixture.run_materializer(&fake_cargo);
    assert_success(&output, "historical fixture materialization");
    let candidate = fixture.artifact.join("candidate");
    let provenance_path = candidate.join("PROVENANCE");
    let admission_path = candidate.join("ADMISSION.json");
    let mut provenance: serde_json::Value =
        serde_json::from_slice(&fs::read(&provenance_path).unwrap()).unwrap();
    provenance["smoke"]["schema"] = serde_json::json!("work-leaf-benchmark-candidate-smoke-v1");
    provenance["smoke"]["gate"] = serde_json::json!("scripted-command-chat");
    fs::write(
        &provenance_path,
        serde_json::to_vec_pretty(&provenance).unwrap(),
    )
    .unwrap();
    let mut admission: serde_json::Value =
        serde_json::from_slice(&fs::read(&admission_path).unwrap()).unwrap();
    admission["candidate"] = provenance.clone();
    admission["materialization"] = provenance["materialization"].clone();
    fs::write(
        &admission_path,
        serde_json::to_vec_pretty(&admission).unwrap(),
    )
    .unwrap();

    let admitted = bash_harness(
        concat!(
            "set -euo pipefail\n",
            "source \"$1/bench-candidate-common\"\n",
            "bench_candidate_is_runnable \"$2\"\n",
        ),
        &[fixture.artifact.as_os_str()],
    );
    assert_success(&admitted, "exact historical smoke admission");

    let mut replay = Command::new(Path::new(env!("CARGO_MANIFEST_DIR")).join("start"))
        .arg("--bench")
        .env("WORK_LEAF_START_BENCH_RESULTS_DIR", &fixture.results)
        .env("WORK_LEAF_START_SKIP_BUILD", "1")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    replay
        .stdin
        .take()
        .unwrap()
        .write_all(b"1\nquit\n")
        .unwrap();
    let replay = replay.wait_with_output().unwrap();
    assert_success(&replay, "exact historical candidate replay");
    assert!(String::from_utf8_lossy(&replay.stdout).contains("application startup completed"));

    provenance["smoke"]["gate"] = serde_json::json!("unknown-historical-gate");
    fs::write(
        &provenance_path,
        serde_json::to_vec_pretty(&provenance).unwrap(),
    )
    .unwrap();
    admission["candidate"] = provenance;
    fs::write(
        &admission_path,
        serde_json::to_vec_pretty(&admission).unwrap(),
    )
    .unwrap();
    let rejected = bash_harness(
        concat!(
            "set -euo pipefail\n",
            "source \"$1/bench-candidate-common\"\n",
            "if bench_candidate_is_runnable \"$2\"; then exit 93; fi\n",
        ),
        &[fixture.artifact.as_os_str()],
    );
    assert_success(&rejected, "unknown historical smoke rejection");
}

#[test]
fn legacy_admission_migration_publishes_the_saved_report_without_rebuilding() {
    let root = test_dir("legacy-admission-success");
    let fixture = HistoricalFixture::new(root.path());
    let fake_cargo = root.path().join("fake-cargo");
    write_fake_cargo(&fake_cargo, true);
    prepare_legacy_materialized_candidate(&fixture, &fake_cargo);

    let output = fixture.run_materializer(&fake_cargo);
    assert_success(&output, "legacy admission migration");
    assert!(
        String::from_utf8_lossy(&output.stdout)
            .contains("migrated historical candidate admission without rebuilding")
    );
    assert_eq!(
        fs::read(fixture.artifact.join("candidate/ADMISSION.json")).unwrap(),
        fs::read(fixture.artifact.join("report.json")).unwrap()
    );
}

#[test]
fn legacy_admission_migration_never_replaces_a_destination_that_appears() {
    let root = test_dir("legacy-admission-destination-race");
    let fixture = HistoricalFixture::new(root.path());
    let fake_cargo = root.path().join("fake-cargo");
    let fake_bin = root.path().join("fake-bin");
    let attack_marker = root.path().join("attack-ran");
    let staging_record = root.path().join("staging-path");
    let renamed_staging = root.path().join("renamed-owned-staging");
    fs::create_dir(&fake_bin).unwrap();
    write_fake_cargo(&fake_cargo, true);
    prepare_legacy_materialized_candidate(&fixture, &fake_cargo);
    write_legacy_admission_attack_wrappers(&fake_bin);

    let admission = fixture.artifact.join("candidate/ADMISSION.json");
    let output = fixture
        .command(&fake_cargo)
        .env(
            "PATH",
            format!("{}:{}", fake_bin.display(), std::env::var("PATH").unwrap()),
        )
        .env("REAL_MV", command_path("mv"))
        .env("REAL_PYTHON", command_path("python3"))
        .env("ATTACK_KIND", "destination")
        .env("ATTACK_MARKER", &attack_marker)
        .env("ADMISSION_PATH", &admission)
        .env("CANDIDATE_DIR", fixture.artifact.join("candidate"))
        .env("STAGING_RECORD", &staging_record)
        .env("RENAMED_STAGING", &renamed_staging)
        .output()
        .unwrap();

    assert!(
        !output.status.success(),
        "legacy migration replaced a destination that appeared\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(attack_marker.exists(), "destination race hook did not run");
    assert_eq!(fs::read(&admission).unwrap(), b"unowned admission bytes\n");
}

#[test]
fn legacy_admission_migration_never_unlinks_a_staging_replacement() {
    let root = test_dir("legacy-admission-staging-race");
    let fixture = HistoricalFixture::new(root.path());
    let fake_cargo = root.path().join("fake-cargo");
    let fake_bin = root.path().join("fake-bin");
    let attack_marker = root.path().join("attack-ran");
    let staging_record = root.path().join("staging-path");
    let renamed_staging = root.path().join("renamed-owned-staging");
    fs::create_dir(&fake_bin).unwrap();
    write_fake_cargo(&fake_cargo, true);
    prepare_legacy_materialized_candidate(&fixture, &fake_cargo);
    write_legacy_admission_attack_wrappers(&fake_bin);

    let admission = fixture.artifact.join("candidate/ADMISSION.json");
    let output = fixture
        .command(&fake_cargo)
        .env(
            "PATH",
            format!("{}:{}", fake_bin.display(), std::env::var("PATH").unwrap()),
        )
        .env("REAL_MV", command_path("mv"))
        .env("REAL_PYTHON", command_path("python3"))
        .env("ATTACK_KIND", "staging")
        .env("ATTACK_MARKER", &attack_marker)
        .env("ADMISSION_PATH", &admission)
        .env("CANDIDATE_DIR", fixture.artifact.join("candidate"))
        .env("STAGING_RECORD", &staging_record)
        .env("RENAMED_STAGING", &renamed_staging)
        .output()
        .unwrap();

    assert!(
        !output.status.success(),
        "legacy migration accepted a replaced staging path\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(attack_marker.exists(), "staging race hook did not run");
    let staging_path = PathBuf::from(fs::read_to_string(&staging_record).unwrap());
    assert_eq!(
        fs::read(staging_path).unwrap(),
        b"unowned staging replacement\n"
    );
    assert!(
        renamed_staging.is_file(),
        "owned staging inode was not preserved"
    );
    assert!(!admission.exists());
}

#[test]
fn materializer_rolls_back_a_post_rename_admission_failure() {
    let root = test_dir("materializer-post-rename-cleanup");
    let fixture = HistoricalFixture::new(root.path());
    let fake_cargo = root.path().join("fake-cargo");
    let fake_bin = root.path().join("fake-bin");
    let tamper_marker = root.path().join("tampered-after-publication");
    fs::create_dir(&fake_bin).unwrap();
    write_fake_cargo(&fake_cargo, true);
    let real_find = command_path("find");
    write_executable(
        &fake_bin.join("find"),
        concat!(
            "#!/usr/bin/env bash\n",
            "set -euo pipefail\n",
            "if [[ -d \"$MATERIALIZED_ARTIFACT/candidate\" && ! -e \"$TAMPER_MARKER\" ]]; then\n",
            "  : > \"$TAMPER_MARKER\"\n",
            "  printf 'tampered after publication\\n' >> \"$MATERIALIZED_ARTIFACT/candidate/SOURCE_TARGETS.json\"\n",
            "fi\n",
            "exec \"$REAL_FIND\" \"$@\"\n",
        ),
    );

    let output = fixture
        .command(&fake_cargo)
        .env(
            "PATH",
            format!("{}:{}", fake_bin.display(), std::env::var("PATH").unwrap()),
        )
        .env("REAL_FIND", real_find)
        .env("MATERIALIZED_ARTIFACT", &fixture.artifact)
        .env("TAMPER_MARKER", &tamper_marker)
        .output()
        .unwrap();
    assert!(
        !output.status.success(),
        "tampered publication passed admission"
    );
    assert!(tamper_marker.exists(), "post-publication hook did not run");
    assert!(!fixture.artifact.join("candidate").exists());
    assert_no_private_materialization_dirs(&fixture.artifact);
}

struct TestDir(PathBuf);

impl TestDir {
    fn path(&self) -> &Path {
        &self.0
    }
}

impl Drop for TestDir {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

fn test_dir(name: &str) -> TestDir {
    static NEXT: AtomicUsize = AtomicUsize::new(0);
    let path = std::env::temp_dir().join(format!(
        "work-leaf-benchmark-candidates-{name}-{}-{}",
        std::process::id(),
        NEXT.fetch_add(1, Ordering::Relaxed)
    ));
    let _ = fs::remove_dir_all(&path);
    fs::create_dir_all(&path).unwrap();
    TestDir(path)
}

fn bash_harness(script: &str, args: &[&std::ffi::OsStr]) -> Output {
    let mut command = Command::new("bash");
    command
        .arg("-c")
        .arg(script)
        .arg("bash")
        .arg(env!("CARGO_MANIFEST_DIR"));
    command.args(args).output().unwrap()
}

fn assert_success(output: &Output, context: &str) {
    assert!(
        output.status.success(),
        "{context} failed\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}

fn write_executable(path: &Path, contents: &str) {
    fs::write(path, contents).unwrap();
    let mut permissions = fs::metadata(path).unwrap().permissions();
    permissions.set_mode(0o755);
    fs::set_permissions(path, permissions).unwrap();
}

fn command_path(command: &str) -> String {
    let output = Command::new("bash")
        .args(["-lc", &format!("command -v {command}")])
        .output()
        .unwrap();
    assert_success(&output, "command lookup");
    String::from_utf8(output.stdout).unwrap().trim().to_owned()
}

fn prepare_legacy_materialized_candidate(fixture: &HistoricalFixture, fake_cargo: &Path) {
    let output = fixture.run_materializer(fake_cargo);
    assert_success(&output, "legacy migration source fixture");
    let admission = fixture.artifact.join("candidate/ADMISSION.json");
    fs::copy(&admission, fixture.artifact.join("report.json")).unwrap();
    fs::remove_file(admission).unwrap();
}

fn write_legacy_admission_attack_wrappers(fake_bin: &Path) {
    write_executable(
        &fake_bin.join("mv"),
        concat!(
            "#!/usr/bin/env bash\n",
            "set -euo pipefail\n",
            "target=${!#}\n",
            "if [[ \"$target\" == \"$ADMISSION_PATH\" && ! -e \"$ATTACK_MARKER\" ]]; then\n",
            "  case \"$ATTACK_KIND\" in\n",
            "    destination)\n",
            "      printf 'unowned admission bytes\\n' > \"$ADMISSION_PATH\"\n",
            "      : > \"$ATTACK_MARKER\"\n",
            "      ;;\n",
            "    staging)\n",
            "      source_path=${@: -2:1}\n",
            "      printf '%s' \"$source_path\" > \"$STAGING_RECORD\"\n",
            "      \"$REAL_MV\" -T -- \"$source_path\" \"$RENAMED_STAGING\"\n",
            "      printf 'unowned staging replacement\\n' > \"$source_path\"\n",
            "      : > \"$ATTACK_MARKER\"\n",
            "      exit 73\n",
            "      ;;\n",
            "  esac\n",
            "fi\n",
            "exec \"$REAL_MV\" \"$@\"\n",
        ),
    );
    write_executable(
        &fake_bin.join("python3"),
        concat!(
            "#!/usr/bin/env bash\n",
            "set -euo pipefail\n",
            "if [[ \"${1:-}\" != - ]]; then exec \"$REAL_PYTHON\" \"$@\"; fi\n",
            "program=$(mktemp)\n",
            "trap 'rm -f -- \"$program\"' EXIT\n",
            "cat > \"$program\"\n",
            "if grep -q 'legacy admission destination appeared during publication' \"$program\" \\\n",
            "  && [[ ! -e \"$ATTACK_MARKER\" ]]; then\n",
            "  case \"$ATTACK_KIND\" in\n",
            "    destination)\n",
            "      printf 'unowned admission bytes\\n' > \"$ADMISSION_PATH\"\n",
            "      ;;\n",
            "    staging)\n",
            "      stages=(\"$CANDIDATE_DIR\"/.ADMISSION.json.publish.*)\n",
            "      [[ \"${#stages[@]}\" == 1 ]]\n",
            "      printf '%s' \"${stages[0]}\" > \"$STAGING_RECORD\"\n",
            "      \"$REAL_MV\" -T -- \"${stages[0]}\" \"$RENAMED_STAGING\"\n",
            "      printf 'unowned staging replacement\\n' > \"${stages[0]}\"\n",
            "      ;;\n",
            "  esac\n",
            "  : > \"$ATTACK_MARKER\"\n",
            "fi\n",
            "set +e\n",
            "\"$REAL_PYTHON\" \"$program\" \"${@:2}\"\n",
            "status=$?\n",
            "set -e\n",
            "exit \"$status\"\n",
        ),
    );
}

fn git(path: &Path, args: &[&str]) -> Output {
    let output = Command::new("git")
        .args(args)
        .current_dir(path)
        .output()
        .unwrap();
    assert_success(&output, &format!("git {}", args.join(" ")));
    output
}

fn git_text(path: &Path, args: &[&str]) -> String {
    String::from_utf8(git(path, args).stdout)
        .unwrap()
        .trim()
        .to_owned()
}

struct HistoricalFixture {
    source: PathBuf,
    results: PathBuf,
    artifact: PathBuf,
    patch: PathBuf,
    final_commit: String,
}

impl HistoricalFixture {
    fn new(root: &Path) -> Self {
        let source = root.join("source");
        fs::create_dir(&source).unwrap();
        git(&source, &["init", "--quiet"]);
        git(&source, &["config", "user.name", "Candidate Test"]);
        git(&source, &["config", "user.email", "candidate-test@invalid"]);
        fs::write(
            source.join("Cargo.toml"),
            "[package]\nname = \"work-leaf\"\nversion = \"0.0.0\"\n",
        )
        .unwrap();
        fs::write(source.join("Cargo.lock"), "# fixture lock\n").unwrap();
        git(&source, &["add", "Cargo.toml", "Cargo.lock"]);
        git(&source, &["commit", "--quiet", "-m", "base"]);
        let base_commit = git_text(&source, &["rev-parse", "HEAD"]);
        fs::write(source.join("feature.txt"), "candidate feature\n").unwrap();
        git(&source, &["add", "feature.txt"]);
        git(&source, &["commit", "--quiet", "-m", "candidate"]);
        let final_commit = git_text(&source, &["rev-parse", "HEAD"]);

        let results = root.join("results");
        let artifact = results.join("fixture-artifacts");
        let pass = artifact.join("patches/pass");
        let patches = pass.join("format-patch");
        fs::create_dir_all(&patches).unwrap();
        let bundle = pass.join("commits.bundle");
        let range = format!("{base_commit}..HEAD");
        git(
            &source,
            &["bundle", "create", bundle.to_str().unwrap(), &range],
        );
        let formatted = Command::new("git")
            .args([
                "format-patch",
                "--no-signature",
                "-o",
                patches.to_str().unwrap(),
                &range,
            ])
            .current_dir(&source)
            .output()
            .unwrap();
        assert_success(&formatted, "git format-patch");
        let patch = PathBuf::from(
            String::from_utf8(formatted.stdout)
                .unwrap()
                .lines()
                .next()
                .unwrap(),
        );
        fs::write(artifact.join("final-status.txt"), "").unwrap();
        fs::write(pass.join("git-status.txt"), "").unwrap();
        fs::write(
            artifact.join("final-log.txt"),
            git(&source, &["log", "--oneline", "--max-count=30"]).stdout,
        )
        .unwrap();
        fs::write(
            pass.join("git-log.txt"),
            git(&source, &["log", "--oneline", &range]).stdout,
        )
        .unwrap();

        let report_document = results.join("fixture.md");
        fs::write(&report_document, "# Fixture\n").unwrap();
        let record = serde_json::json!({
            "run_id": "historical-fixture",
            "result": "pass",
            "worktree_source_commit": base_commit,
            "worktree_source_dirty": "no",
            "base_commit": base_commit,
            "review_completed": "yes",
            "linearize_completed": "yes",
            "commits_after_base": 1,
            "changed_files": 1,
            "changed_lines_added": 1,
            "changed_lines_deleted": 0,
            "changed_lines_total": 1,
            "code_quality": "passed required checks",
            "patch_artifacts": pass,
            "report": report_document,
        });
        fs::write(
            results.join("fixture.jsonl"),
            format!("{}\n", serde_json::to_string(&record).unwrap()),
        )
        .unwrap();

        Self {
            source,
            results,
            artifact,
            patch,
            final_commit,
        }
    }

    fn command(&self, cargo: &Path) -> Command {
        let mut command = Command::new("bash");
        command
            .arg(Path::new(env!("CARGO_MANIFEST_DIR")).join("materialize-bench-candidate"))
            .args([
                "--results-dir",
                self.results.to_str().unwrap(),
                "--source-repo",
                self.source.to_str().unwrap(),
                "--artifact",
                self.artifact.to_str().unwrap(),
                "--max-builds",
                "1",
            ])
            .env("WORK_LEAF_BENCH_MATERIALIZE_CARGO", cargo)
            .env("EXPECTED_FINAL_COMMIT", &self.final_commit);
        command
    }

    fn run_materializer(&self, cargo: &Path) -> Output {
        self.command(cargo).output().unwrap()
    }
}

fn write_fake_cargo(path: &Path, build_succeeds: bool) {
    let behavior = if build_succeeds {
        concat!(
            "mkdir -p \"$CARGO_TARGET_DIR/release\"\n",
            "cat > \"$CARGO_TARGET_DIR/release/work-leaf\" <<'BIN'\n",
            "#!/usr/bin/env bash\n",
            "set -euo pipefail\n",
            "IFS= read -r command\n",
            "[[ \"$command\" == quit ]]\n",
            "printf 'application startup completed\\n'\n",
            "BIN\n",
            "chmod 0755 \"$CARGO_TARGET_DIR/release/work-leaf\"\n",
        )
    } else {
        "exit 23\n"
    };
    write_executable(
        path,
        &format!(
            concat!(
                "#!/usr/bin/env bash\n",
                "set -euo pipefail\n",
                "if [[ \"$*\" == 'metadata --no-deps --format-version 1 --locked' ]]; then\n",
                "  printf '{{\"packages\":[{{\"manifest_path\":\"%s/Cargo.toml\",\"targets\":[{{\"name\":\"work-leaf\",\"kind\":[\"bin\"]}}]}}]}}\\n' \"$PWD\"\n",
                "  exit 0\n",
                "fi\n",
                "[[ \"$*\" == 'build --release --locked --bins' ]]\n",
                "[[ \"$(git rev-parse HEAD)\" == \"$EXPECTED_FINAL_COMMIT\" ]]\n",
                "{behavior}",
            ),
            behavior = behavior
        ),
    );
}

fn assert_no_private_materialization_dirs(artifact: &Path) {
    let leftovers = fs::read_dir(artifact)
        .unwrap()
        .map(|entry| entry.unwrap().file_name())
        .filter(|name| {
            name.to_string_lossy()
                .starts_with(".candidate.materialize.")
        })
        .collect::<Vec<_>>();
    assert!(
        leftovers.is_empty(),
        "private staging remained: {leftovers:?}"
    );
}
