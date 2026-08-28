#!/usr/bin/env python3

import argparse
import hashlib
import json
import os
import shlex
import shutil
import subprocess
import sys
import time
from pathlib import Path

from exact_usage_proxy import summarize_records


MODEL = "gpt-5.5"
REASONING_EFFORT = "xhigh"
PROVIDER_ID = "work_leaf_exact_usage"


def codex_config_args(base_url):
    values = [
        f'model="{MODEL}"',
        f'model_reasoning_effort="{REASONING_EFFORT}"',
        f'model_provider="{PROVIDER_ID}"',
        f'model_providers.{PROVIDER_ID}.name="Work Leaf exact usage proxy"',
        f"model_providers.{PROVIDER_ID}.base_url={json.dumps(base_url)}",
        f'model_providers.{PROVIDER_ID}.env_key="OPENAI_API_KEY"',
        f'model_providers.{PROVIDER_ID}.wire_api="responses"',
        f"model_providers.{PROVIDER_ID}.supports_websockets=false",
    ]
    return [part for value in values for part in ("-c", value)]


def write_codex_wrapper(path, real_codex, base_url):
    path = Path(path)
    path.parent.mkdir(parents=True, exist_ok=True)
    command = [str(Path(real_codex).resolve()), *codex_config_args(base_url), '"$@"']
    rendered = " ".join(shlex.quote(part) if part != '"$@"' else part for part in command)
    temporary = path.with_name(f".{path.name}.{os.getpid()}.tmp")
    temporary.write_text(f"#!/usr/bin/env bash\nexec {rendered}\n")
    temporary.chmod(0o700)
    os.replace(temporary, path)


def _sha256(path):
    digest = hashlib.sha256()
    with Path(path).open("rb") as source:
        while chunk := source.read(1024 * 1024):
            digest.update(chunk)
    return digest.hexdigest()


def _write_json(path, payload):
    path = Path(path)
    temporary = path.with_name(f".{path.name}.{os.getpid()}.tmp")
    temporary.write_text(json.dumps(payload, indent=2, sort_keys=True) + "\n")
    temporary.chmod(0o600)
    os.replace(temporary, path)


def _wait_for_ready(process, ready_path, timeout_seconds=15):
    deadline = time.monotonic() + timeout_seconds
    while time.monotonic() < deadline:
        if ready_path.exists():
            return json.loads(ready_path.read_text())
        status = process.poll()
        if status is not None:
            raise RuntimeError(f"exact-usage proxy exited before becoming ready with status {status}")
        time.sleep(0.05)
    raise RuntimeError("timed out waiting for exact-usage proxy")


def _stop_proxy(process, timeout_seconds):
    if process.poll() is not None:
        return process.returncode, False
    process.terminate()
    try:
        return process.wait(timeout=timeout_seconds), False
    except subprocess.TimeoutExpired:
        process.kill()
        return process.wait(), True


def run(args):
    if not args.command:
        raise ValueError("a benchmark command is required after --")
    command = args.command[1:] if args.command[0] == "--" else args.command
    if not command:
        raise ValueError("a benchmark command is required after --")
    if not os.environ.get("OPENAI_API_KEY"):
        raise RuntimeError("OPENAI_API_KEY is required but was not exported")

    result_dir = Path(args.result_dir).resolve()
    result_dir.mkdir(parents=True, exist_ok=False)
    runtime_dir = result_dir / "runtime"
    runtime_dir.mkdir(mode=0o700)
    records = result_dir / "responses.jsonl"
    ready_path = runtime_dir / "proxy-ready.json"
    proxy_stdout = (result_dir / "proxy.stdout").open("wb")
    proxy_stderr = (result_dir / "proxy.stderr").open("wb")
    module_dir = Path(__file__).resolve().parent
    real_codex = Path(args.real_codex or shutil.which("codex") or "")
    if not real_codex.is_file() or not os.access(real_codex, os.X_OK):
        raise RuntimeError(f"real Codex executable is not runnable: {real_codex}")

    proxy_command = [
        sys.executable,
        str(module_dir / "exact_usage_proxy.py"),
        "serve",
        "--listen",
        "127.0.0.1:0",
        "--upstream",
        args.upstream,
        "--records",
        str(records),
        "--ready-file",
        str(ready_path),
        "--poll-interval-seconds",
        str(args.poll_interval_seconds),
        "--poll-timeout-seconds",
        str(args.poll_timeout_seconds),
    ]
    proxy = subprocess.Popen(proxy_command, stdout=proxy_stdout, stderr=proxy_stderr)
    command_status = None
    proxy_status = None
    proxy_forced_kill = False
    launcher_error = None
    try:
        ready = _wait_for_ready(proxy, ready_path)
        wrapper = runtime_dir / "bin" / "codex"
        write_codex_wrapper(wrapper, real_codex, ready["base_url"])
        manifest = {
            "schema_version": 1,
            "purpose": "exact provider usage for completed and interrupted benchmark responses",
            "model": MODEL,
            "reasoning_effort": REASONING_EFFORT,
            "provider": PROVIDER_ID,
            "provider_base_url": ready["base_url"],
            "upstream": args.upstream,
            "real_codex": str(real_codex.resolve()),
            "real_codex_sha256": _sha256(real_codex.resolve()),
            "proxy_sha256": _sha256(module_dir / "exact_usage_proxy.py"),
            "wrapper_sha256": _sha256(wrapper),
            "command": command,
            "started_unix_ns": time.time_ns(),
        }
        _write_json(result_dir / "manifest.json", manifest)

        environment = os.environ.copy()
        environment["PATH"] = f"{wrapper.parent}:{environment.get('PATH', '')}"
        environment["WORK_LEAF_BENCH_MODEL"] = MODEL
        environment["WORK_LEAF_BENCH_REASONING_EFFORT"] = REASONING_EFFORT
        environment["WORK_LEAF_DIRECT_BENCH_MODEL"] = MODEL
        environment["WORK_LEAF_DIRECT_BENCH_REASONING_EFFORT"] = REASONING_EFFORT
        command_status = subprocess.run(command, env=environment).returncode
    except (OSError, RuntimeError, ValueError, json.JSONDecodeError) as error:
        launcher_error = str(error)
    finally:
        proxy_status, proxy_forced_kill = _stop_proxy(
            proxy, args.poll_timeout_seconds + 30
        )
        proxy_stdout.close()
        proxy_stderr.close()

    try:
        summary = summarize_records(records)
    except (OSError, ValueError) as error:
        summary = {"complete": False, "error": str(error)}
    _write_json(result_dir / "usage-summary.json", summary)
    launcher_result = {
        "schema_version": 1,
        "command_status": command_status,
        "proxy_status": proxy_status,
        "proxy_forced_kill": proxy_forced_kill,
        "launcher_error": launcher_error,
        "usage_complete": summary.get("complete") is True,
        "finished_unix_ns": time.time_ns(),
    }
    _write_json(result_dir / "launcher-result.json", launcher_result)
    if launcher_error or proxy_status != 0 or proxy_forced_kill or not summary.get("complete"):
        return 90
    return command_status


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("--result-dir", required=True)
    parser.add_argument("--real-codex")
    parser.add_argument("--upstream", default="https://api.openai.com/v1")
    parser.add_argument("--poll-interval-seconds", type=float, default=1.0)
    parser.add_argument("--poll-timeout-seconds", type=float, default=180.0)
    parser.add_argument("command", nargs=argparse.REMAINDER)
    args = parser.parse_args()
    try:
        return run(args)
    except (OSError, RuntimeError, ValueError) as error:
        print(f"error: {error}", file=sys.stderr)
        return 2


if __name__ == "__main__":
    sys.exit(main())
