"""Tool-free private MCP initialization-order fixture, not a production provider."""
import json
import sys
import time


def reply(request):
    if "id" not in request:
        return None
    response = {"jsonrpc": "2.0", "id": request["id"]}
    method = request.get("method")
    if method == "initialize":
        time.sleep(2)
        response["result"] = {
            "protocolVersion": request["params"]["protocolVersion"], "capabilities": {},
            "serverInfo": {"name": "p01-empty-startup", "version": "1"},
        }
    elif method in {"tools/list", "resources/list", "resources/templates/list", "prompts/list"}:
        key = {"tools/list": "tools", "resources/list": "resources",
               "resources/templates/list": "resourceTemplates", "prompts/list": "prompts"}[method]
        response["result"] = {key: []}
    elif method == "ping":
        response["result"] = {}
    else:
        response["error"] = {"code": -32601, "message": "No such method"}
    return response


if __name__ == "__main__":
    for line in sys.stdin:
        result = reply(json.loads(line))
        if result is not None:
            print(json.dumps(result), flush=True)
