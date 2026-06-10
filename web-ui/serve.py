#!/usr/bin/env python3
import argparse
import http.client
import os
from functools import partial
from http.server import SimpleHTTPRequestHandler, ThreadingHTTPServer
from pathlib import Path
from urllib.parse import urlsplit


API_PATHS = {
    "/agent/interrupt",
    "/agent/message",
    "/command",
    "/command-agent",
    "/events/drain",
    "/health",
    "/loading-text",
    "/shutdown",
    "/snapshot",
    "/state",
    "/transcript",
}


class WorkLeafWebHandler(SimpleHTTPRequestHandler):
    api_base = ""

    def end_headers(self):
        self.send_header("Access-Control-Allow-Origin", "*")
        self.send_header("Access-Control-Allow-Headers", "content-type, accept")
        self.send_header("Access-Control-Allow-Methods", "GET, POST, OPTIONS")
        super().end_headers()

    def do_OPTIONS(self):
        self.send_response(204)
        self.end_headers()

    def do_GET(self):
        if self.is_api_request():
            self.proxy_api()
        else:
            super().do_GET()

    def do_POST(self):
        if self.is_api_request():
            self.proxy_api()
        else:
            self.send_error(404)

    def is_api_request(self):
        path = urlsplit(self.path).path
        return path in API_PATHS

    def proxy_api(self):
        target = urlsplit(self.api_base)
        if target.scheme != "http" or not target.hostname:
            self.send_error(502, "invalid Work Leaf API URL")
            return

        body = self.rfile.read(int(self.headers.get("content-length", "0")))
        path = self.path
        if target.path and target.path != "/":
            path = target.path.rstrip("/") + path

        conn = http.client.HTTPConnection(target.hostname, target.port or 80, timeout=30)
        headers = {}
        for name in ("accept", "content-type"):
            value = self.headers.get(name)
            if value:
                headers[name] = value
        try:
            conn.request(self.command, path, body=body, headers=headers)
            response = conn.getresponse()
            payload = response.read()
        except OSError as error:
            self.send_error(502, f"Work Leaf API request failed: {error}")
            return
        finally:
            conn.close()

        self.send_response(response.status, response.reason)
        for name, value in response.getheaders():
            if name.lower() not in {"connection", "keep-alive", "transfer-encoding"}:
                self.send_header(name, value)
        self.end_headers()
        self.wfile.write(payload)


def parse_listen(value):
    host, sep, port = value.rpartition(":")
    if not sep:
        return value, 0
    return host or "127.0.0.1", int(port)


def main():
    parser = argparse.ArgumentParser(description="Serve the Work Leaf web UI.")
    parser.add_argument("--api", default=os.environ.get("WORK_LEAF_ORCHESTRATOR_URL", ""))
    parser.add_argument("--listen", default=os.environ.get("WORK_LEAF_WEB_UI_LISTEN", "127.0.0.1:0"))
    args = parser.parse_args()
    if not args.api:
        parser.error("--api or WORK_LEAF_ORCHESTRATOR_URL is required")

    web_dir = Path(__file__).resolve().parent
    host, port = parse_listen(args.listen)
    handler = partial(WorkLeafWebHandler, directory=str(web_dir))
    WorkLeafWebHandler.api_base = args.api.rstrip("/")
    server = ThreadingHTTPServer((host, port), handler)
    actual_host, actual_port = server.server_address[:2]
    display_host = "127.0.0.1" if actual_host in {"", "0.0.0.0"} else actual_host
    print(f"WORK_LEAF_WEB_UI_URL=http://{display_host}:{actual_port}", flush=True)
    server.serve_forever()


if __name__ == "__main__":
    main()
