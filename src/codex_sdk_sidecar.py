import json
import os
import sys
import threading
import traceback

from openai_codex import CodexConfig
from openai_codex.client import CodexClient
from openai_codex.generated.v2_all import (
    AgentMessageThreadItem,
    ConfigReadResponse,
    ItemCompletedNotification,
    ThreadTokenUsageUpdatedNotification,
    TurnCompletedNotification,
    TurnStartedNotification,
)


REMOVED_CODEX_ENV = (
    "CODEX_THREAD_ID",
    "CODEX_CI",
    "CODEX_MANAGED_BY_NPM",
    "CODEX_MANAGED_PACKAGE_ROOT",
    "WORK_LEAF_CODEX_TRACE",
    "WORK_LEAF_COMMAND_TMPDIR",
    "WORK_LEAF_CONTEXT_BUNDLE_DIR",
    "WORK_LEAF_CODEX_SDK_PYTHON",
)


def write(payload):
    with output_lock:
        sys.stdout.write(json.dumps(payload, separators=(",", ":")) + "\n")
        sys.stdout.flush()


def usage_json(token_usage):
    if token_usage is None:
        return None
    usage = token_usage.last
    return {
        "input_tokens": usage.input_tokens,
        "cached_input_tokens": usage.cached_input_tokens,
        "output_tokens": usage.output_tokens,
        "reasoning_output_tokens": usage.reasoning_output_tokens,
    }


def sandbox_policy(sandbox):
    if sandbox == "workspace-write":
        return {"type": "workspaceWrite"}
    if sandbox == "danger-full-access":
        return {"type": "dangerFullAccess"}
    return {"type": "readOnly"}


def enum_value(value):
    return getattr(value, "value", value)


def root_value(value):
    return getattr(value, "root", value)


def display_value(value):
    value = root_value(value)
    value = enum_value(value)
    return str(value) if value is not None else "unknown"


def thread_start_params(request):
    params = {
        "approvalPolicy": "never",
        "cwd": request["cwd"],
        "sandbox": request.get("sandbox") or "read-only",
    }
    if request.get("model"):
        params["model"] = request["model"]
    return params


def turn_params(request):
    params = {
        "approvalPolicy": "never",
        "cwd": request["cwd"],
        "sandboxPolicy": sandbox_policy(request.get("sandbox") or "read-only"),
    }
    if request.get("model"):
        params["model"] = request["model"]
    return params


def final_response_from_item(item):
    item_root = getattr(item, "root", item)
    if isinstance(item_root, AgentMessageThreadItem):
        return item_root.text
    return None


def make_client():
    client = CodexClient(
        CodexConfig(
            codex_bin=config["codex_bin"],
            cwd=config["cwd"],
            client_name="work_leaf",
            client_title="Work Leaf",
            client_version=config.get("client_version", "0.1.0"),
            experimental_api=True,
        )
    )
    client.start()
    client.initialize()
    return client


def agent_client(agent_id):
    with clients_lock:
        existing = clients.get(agent_id)
    if existing is not None:
        return existing

    created = make_client()
    with clients_lock:
        existing = clients.get(agent_id)
        if existing is None:
            clients[agent_id] = created
            return created

    created.close()
    return existing


def close_clients():
    with clients_lock:
        closing = list(clients.values())
        clients.clear()
    for client in closing:
        try:
            client.close()
        except BaseException:
            pass


def run_turn(request_id, agent_id, thread_id, prompt, request):
    client = agent_client(agent_id)
    final_response = ""
    latest_usage = None
    started = client.turn_start(thread_id, prompt, params=turn_params(request))
    turn_id = started.turn.id
    with state_lock:
        active_turns[agent_id] = (thread_id, turn_id)
    try:
        while True:
            notification = client.next_turn_notification(turn_id)
            payload = notification.payload
            if (
                notification.method == "turn/started"
                and isinstance(payload, TurnStartedNotification)
            ):
                write({"id": request_id, "event": {"type": "status", "text": "Codex is working"}})
                continue
            if (
                notification.method == "item/completed"
                and isinstance(payload, ItemCompletedNotification)
                and payload.turn_id == turn_id
            ):
                response = final_response_from_item(payload.item)
                if response is not None:
                    final_response = response
                    write({"id": request_id, "event": {"type": "message", "text": response}})
                continue
            if (
                notification.method == "thread/tokenUsage/updated"
                and isinstance(payload, ThreadTokenUsageUpdatedNotification)
                and payload.turn_id == turn_id
            ):
                latest_usage = usage_json(payload.token_usage)
                if latest_usage is not None:
                    write({"id": request_id, "event": {"type": "usage", "usage": latest_usage}})
                continue
            if (
                notification.method == "turn/completed"
                and isinstance(payload, TurnCompletedNotification)
                and payload.turn.id == turn_id
            ):
                status = getattr(payload.turn.status, "value", str(payload.turn.status))
                if status == "failed":
                    error = payload.turn.error
                    message = getattr(error, "message", None) or "Codex turn failed"
                    raise RuntimeError(message)
                write(
                    {
                        "id": request_id,
                        "ok": True,
                        "thread_id": thread_id,
                        "reply": final_response,
                        "usage": latest_usage,
                    }
                )
                break
    finally:
        client.unregister_turn_notifications(turn_id)
        with state_lock:
            if active_turns.get(agent_id) == (thread_id, turn_id):
                active_turns.pop(agent_id, None)


def handle_launch(request):
    request_id = request["id"]
    agent_id = request["agent_id"]
    client = agent_client(agent_id)
    started = client.thread_start(thread_start_params(request))
    thread_id = started.thread.id
    with state_lock:
        threads[agent_id] = thread_id
        loaded_threads.add(thread_id)
    write(
        {
            "id": request_id,
            "event": {"type": "status", "text": f"Codex session {thread_id}"},
        }
    )
    run_turn(request_id, agent_id, thread_id, request["prompt"], request)


def resolve_thread(request):
    agent_id = request["agent_id"]
    client = agent_client(agent_id)
    thread_id = request.get("thread_id")
    if not thread_id:
        with state_lock:
            thread_id = threads.get(agent_id)
    if not thread_id:
        raise RuntimeError(f"unknown Codex SDK thread for {agent_id}")
    with state_lock:
        known = thread_id in loaded_threads
    if not known:
        client.thread_resume(thread_id, {"threadId": thread_id, **thread_start_params(request)})
        with state_lock:
            loaded_threads.add(thread_id)
            threads[agent_id] = thread_id
    return thread_id


def handle_send(request):
    request_id = request["id"]
    agent_id = request["agent_id"]
    thread_id = resolve_thread(request)
    run_turn(request_id, agent_id, thread_id, request["prompt"], request)


def read_config(request, client):
    return client.request(
        "config/read",
        {"cwd": request["cwd"], "includeLayers": False},
        response_model=ConfigReadResponse,
    ).config


def thread_status_label(thread):
    status = root_value(thread.status)
    return display_value(getattr(status, "type", status))


def render_status(request, thread_id, client):
    thread = client.thread_read(thread_id, include_turns=False).thread
    try:
        config = read_config(request, client)
    except BaseException:
        config = None
    try:
        account_response = client.account_read()
        account = root_value(account_response.account) if account_response.account is not None else None
        account_text = display_value(getattr(account, "type", None)) if account is not None else "none"
    except BaseException:
        account_text = "unknown"

    model = request.get("model")
    sandbox = request.get("sandbox") or "read-only"
    approval = "never"
    context_window = None
    reasoning = None
    if config is not None:
        model = model or getattr(config, "model", None)
        sandbox = display_value(getattr(config, "sandbox_mode", None)) or sandbox
        approval = display_value(getattr(config, "approval_policy", None)) or approval
        context_window = getattr(config, "model_context_window", None)
        reasoning = getattr(config, "model_reasoning_effort", None)

    lines = [
        "OpenAI Codex SDK status",
        f"Model: {model or 'default'}"
        + (f" (reasoning {display_value(reasoning)})" if reasoning is not None else ""),
        f"Directory: {display_value(thread.cwd)}",
        f"Permissions: sandbox {sandbox}, approval {approval}",
        f"Account: {account_text}",
        f"Session: {thread.id}",
    ]
    if getattr(thread, "forked_from_id", None):
        lines.append(f"Forked from: {thread.forked_from_id}")
    if context_window is not None:
        lines.append(f"Context window: {context_window} tokens")
    lines.append(f"Thread status: {thread_status_label(thread)}")
    return "\n".join(lines)


def slash_command_name(prompt):
    stripped = prompt.strip()
    if not stripped.startswith("/") or len(stripped) == 1 or stripped[1].isspace():
        return None, ""
    command, _, rest = stripped[1:].partition(" ")
    return command, rest.strip()


def handle_command(request):
    request_id = request["id"]
    agent_id = request["agent_id"]
    client = agent_client(agent_id)
    thread_id = resolve_thread(request)
    command, args = slash_command_name(request["prompt"])
    if command is None:
        run_turn(request_id, agent_id, thread_id, request["prompt"], request)
        return

    command = command.lower()
    reply_thread_id = thread_id
    if command in ("status", "st"):
        reply = render_status(request, thread_id, client)
    elif command == "fork":
        forked = client.thread_fork(thread_id, thread_start_params(request))
        reply_thread_id = forked.thread.id
        with state_lock:
            threads[agent_id] = reply_thread_id
            loaded_threads.add(reply_thread_id)
        reply = f"Forked Codex thread {thread_id} -> {reply_thread_id}"
    elif command in ("compact", "compress"):
        client.thread_compact(thread_id)
        reply = f"Compaction started for Codex thread {thread_id}."
    elif command in ("rename", "name"):
        if not args:
            reply = "Usage: /rename <name>"
        else:
            client.thread_set_name(thread_id, args)
            reply = f"Renamed Codex thread {thread_id} to {args}."
    elif command == "archive":
        client.thread_archive(thread_id)
        reply = f"Archived Codex thread {thread_id}."
    elif command == "unarchive":
        client.thread_unarchive(thread_id)
        reply = f"Unarchived Codex thread {thread_id}."
    elif command in ("help", "?"):
        reply = (
            "Supported SDK slash commands: /status, /fork, /compact, "
            "/rename <name>, /archive, /unarchive, /help."
        )
    else:
        reply = (
            f"Codex SDK command /{command} is not exposed by the app-server protocol; "
            "no model request was sent."
        )
    write({"id": request_id, "ok": True, "thread_id": reply_thread_id, "reply": reply})


def handle_interrupt(request):
    agent_id = request["agent_id"]
    with state_lock:
        active = active_turns.get(agent_id)
    if active is not None:
        thread_id, turn_id = active
        client = agent_client(agent_id)
        client.turn_interrupt(thread_id, turn_id)
    write({"id": request["id"], "ok": True})


def worker(request):
    try:
        op = request.get("op")
        if op == "launch":
            handle_launch(request)
        elif op == "send":
            handle_send(request)
        elif op == "command":
            handle_command(request)
        elif op == "interrupt":
            handle_interrupt(request)
        else:
            raise RuntimeError(f"unknown sidecar op {op!r}")
    except BaseException as exc:
        write(
            {
                "id": request.get("id"),
                "ok": False,
                "error": f"{type(exc).__name__}: {exc}",
                "trace": traceback.format_exc(limit=8),
            }
        )


for name in REMOVED_CODEX_ENV:
    os.environ.pop(name, None)

config = json.loads(os.environ["WORK_LEAF_CODEX_SDK_CONFIG"])

output_lock = threading.Lock()
state_lock = threading.Lock()
clients_lock = threading.Lock()
clients = {}
threads = {}
loaded_threads = set()
active_turns = {}

write({"id": 0, "ok": True, "ready": True})

for line in sys.stdin:
    if not line.strip():
        continue
    try:
        request = json.loads(line)
    except json.JSONDecodeError as exc:
        write({"id": None, "ok": False, "error": f"invalid JSON request: {exc}"})
        continue
    if request.get("op") == "shutdown":
        write({"id": request.get("id"), "ok": True})
        break
    thread = threading.Thread(target=worker, args=(request,), daemon=True)
    thread.start()

close_clients()
