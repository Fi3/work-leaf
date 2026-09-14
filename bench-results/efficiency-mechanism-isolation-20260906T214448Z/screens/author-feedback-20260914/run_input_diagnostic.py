"""One read-only R13 input delivery diagnostic using the unchanged bounded host."""
import argparse
from pathlib import Path
import os
import sys

sys.path.insert(0, str(Path(__file__).resolve().parents[1] / "author-inverse-20260914"))
import author_inverse


def main():
    parser = argparse.ArgumentParser()
    for name in ("arm", "repo", "artifact-dir", "prompt-file", "codex-bin"):
        parser.add_argument("--" + name, required=True)
    args = parser.parse_args()
    if args.arm != "R13" or os.environ.get("WORK_LEAF_BENCH_INVERSE") != "1":
        parser.error("only the explicitly opted-in R13 diagnostic is supported")
    host = author_inverse.load_host("identity")
    root = Path(args.artifact_dir).resolve()
    argv = [args.codex_bin, "--cd", args.repo, "--sandbox", "read-only",
            "--ask-for-approval", "never", "--model", "gpt-5.5", "exec",
            "--color", "never", "--json", "-o", str(root / "native.final.txt"), "-"]
    result = host.execute_child(argv, Path(args.repo), Path(args.prompt_file),
        root / "native.stdout.jsonl", root / "native.stderr.txt", 60, os.environ.copy())
    host.save_json(root / "native.exit.json", result)
    return result["exit_code"] if result["exit_code"] >= 0 else 1


if __name__ == "__main__":
    raise SystemExit(main())
