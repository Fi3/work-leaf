#!/usr/bin/env python3
"""Bounded, explicitly selected official-API diagnostics; no benchmark adapter."""

import argparse
import json
import os
import re
import time
import urllib.error
import urllib.request
from pathlib import Path
from urllib.parse import urlsplit


def safe_label(value, secret, pattern):
    if not isinstance(value, str):
        return None
    if secret and secret in value:
        return "redacted"
    return value if re.fullmatch(pattern, value) else "unrecognized"


def safe_error(error, secret):
    return {
        key: safe_label(error.get(key), secret, r"[a-z_]{1,80}")
        for key in ("code", "type")
    }


def safe_response(response, secret):
    usage = response.get("usage")
    result = {
        "response_id": safe_label(response.get("id"), secret, r"resp_[A-Za-z0-9_-]+"),
        "model": safe_label(response.get("model"), secret, r"gpt-5\.5(?:-[0-9-]+)?"),
        "status": safe_label(response.get("status"), secret, r"[a-z_]{1,30}"),
        "usage": None,
        "error": safe_error(response["error"], secret)
        if isinstance(response.get("error"), dict)
        else None,
    }
    if isinstance(usage, dict):
        values = {
            "input_tokens": usage.get("input_tokens"),
            "cached_input_tokens": (usage.get("input_tokens_details") or {}).get("cached_tokens"),
            "output_tokens": usage.get("output_tokens"),
            "reasoning_output_tokens": (usage.get("output_tokens_details") or {}).get("reasoning_tokens"),
            "total_tokens": usage.get("total_tokens"),
        }
        result["usage"] = {
            key: value if type(value) is int and value >= 0 else None
            for key, value in values.items()
        }
    return result


def cancel_usage_gate(in_progress, output_seen, response):
    usage = response.get("usage")
    return bool(
        in_progress
        and output_seen
        and response.get("status") == "cancelled"
        and isinstance(usage, dict)
        and all(type(usage.get(key)) is int and usage[key] >= 0 for key in
                ("input_tokens", "cached_input_tokens", "output_tokens", "total_tokens"))
        and usage["input_tokens"] > 0
        and usage["output_tokens"] > 0
        and usage["cached_input_tokens"] <= usage["input_tokens"]
        and usage["total_tokens"] == usage["input_tokens"] + usage["output_tokens"]
        and (usage.get("reasoning_output_tokens") is None or (
            type(usage["reasoning_output_tokens"]) is int
            and 0 <= usage["reasoning_output_tokens"] <= usage["output_tokens"]
        ))
    )


class NoRedirect(urllib.request.HTTPRedirectHandler):
    def redirect_request(self, req, fp, code, msg, headers, newurl):
        return None


def run(operation, response_id=None):
    secret = os.environ.get("OPENAI_API_KEY", "")
    base = os.environ.get("OPENAI_BASE_URL") or os.environ.get("OPENAI_API_BASE") or "https://api.openai.com/v1"
    parsed = urlsplit(base)
    result = {
        "schema_version": 1,
        "operation": operation,
        "endpoint_hostname": parsed.hostname,
        "model": "gpt-5.5",
        "api_key_present": bool(secret),
        "model_requests_attempted": 0,
    }
    if parsed.scheme != "https" or parsed.hostname != "api.openai.com" or any(
        (parsed.username, parsed.password, parsed.query, parsed.fragment)
    ):
        result["local_status"] = "configured_route_not_official_public_api"
        return result
    if not secret:
        result["local_status"] = "missing_api_key"
        return result
    opener = urllib.request.build_opener(NoRedirect)
    headers = {"Authorization": "Bearer " + secret, "Content-Type": "application/json"}

    def request(method, suffix, body=None):
        req = urllib.request.Request(
            base.rstrip("/") + suffix,
            data=None if body is None else json.dumps(body).encode(),
            headers=headers,
            method=method,
        )
        return opener.open(req, timeout=15)

    def failure(error):
        saved = {"local_error_type": type(error).__name__}
        if isinstance(error, urllib.error.HTTPError):
            saved["http_status"] = error.code
            try:
                detail = json.load(error).get("error", {})
            except Exception:
                detail = {}
            saved["error"] = safe_error(detail, secret)
        return saved

    started = time.monotonic()
    if operation in ("model", "retrieve"):
        suffix = "/models/gpt-5.5" if operation == "model" else "/responses/" + response_id
        try:
            with request("GET", suffix) as response:
                result["http_status"] = response.status
                payload = json.load(response)
                result["response"] = safe_response(payload, secret) if operation == "retrieve" else {
                    "model_id": "gpt-5.5" if payload.get("id") == "gpt-5.5" else None
                }
        except Exception as error:
            result["request_error"] = failure(error)
        return result

    body = {
        "model": "gpt-5.5", "reasoning": {"effort": "xhigh"},
        "input": "List the integers from 1 to 1000, one per line. Do not use tools.",
        "max_output_tokens": 2048, "background": True, "stream": True,
    }
    result.update({
        "request": body, "model_requests_attempted": 1,
        "in_progress_seen": False, "output_text_delta_seen": False,
        "events_seen": [], "post_cancel_observations": [],
        "cancel_usage_gate_passed": False,
    })
    stream = None
    try:
        stream = request("POST", "/responses", body)
        result["create_http_status"] = stream.status
        while time.monotonic() - started < 45:
            line = stream.readline()
            if not line:
                break
            if not line.startswith(b"data: "):
                continue
            try:
                event = json.loads(line[6:])
            except Exception:
                continue
            kind = event.get("type")
            if kind in {"response.created", "response.in_progress", "response.completed",
                        "response.failed", "response.incomplete", "response.output_text.delta"}:
                if kind not in result["events_seen"]:
                    result["events_seen"].append(kind)
            response = event.get("response") or {}
            if isinstance(response.get("id"), str) and re.fullmatch(r"resp_[A-Za-z0-9_-]+", response["id"]):
                response_id = response["id"]
                result["response_id"] = response_id
            if kind == "response.in_progress":
                result["in_progress_seen"] = True
            if kind == "response.output_text.delta":
                result["output_text_delta_seen"] = True
                result["cancel_trigger"] = "first_output_text_delta"
                break
            if kind in {"response.completed", "response.failed", "response.incomplete"}:
                result["terminal_before_cancel"] = safe_response(response, secret)
                break
    except Exception as error:
        result["stream_error"] = failure(error)
    finally:
        if response_id:
            result.setdefault("cancel_trigger", "cleanup_after_stream_end_or_timeout")
            try:
                with request("POST", "/responses/" + response_id + "/cancel", {}) as response:
                    result["cancel_http_status"] = response.status
                    result["cancel_result"] = safe_response(json.load(response), secret)
            except Exception as error:
                result["cancel_error"] = failure(error)
            if stream is not None:
                stream.close()
                stream = None
            for delay in (0, 2, 5):
                time.sleep(delay)
                try:
                    with request("GET", "/responses/" + response_id) as response:
                        observation = safe_response(json.load(response), secret)
                        observation.update({"http_status": response.status, "elapsed_seconds": round(time.monotonic() - started, 3)})
                        result["post_cancel_observations"].append(observation)
                        if cancel_usage_gate(result["in_progress_seen"], result["output_text_delta_seen"], observation):
                            result["cancel_usage_gate_passed"] = True
                            break
                except Exception as error:
                    result["retrieve_error"] = failure(error)
                    break
        if stream is not None:
            stream.close()
    result["elapsed_seconds"] = round(time.monotonic() - started, 3)
    return result


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("operation", choices=("model", "cancel", "retrieve"))
    parser.add_argument("--response-id")
    parser.add_argument("--output", type=Path, required=True)
    args = parser.parse_args()
    if args.output.exists():
        parser.error("output already exists; choose a distinct evidence file")
    if args.operation == "retrieve" and not re.fullmatch(r"resp_[A-Za-z0-9_-]+", args.response_id or ""):
        parser.error("retrieve requires a response ID")
    result = run(args.operation, args.response_id)
    with args.output.open("x", encoding="utf-8") as output:
        json.dump(result, output, indent=2, sort_keys=True)
        output.write("\n")
    print(json.dumps(result, sort_keys=True))


if __name__ == "__main__":
    main()
