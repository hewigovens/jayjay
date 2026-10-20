"""Answers the GitHub REST calls a stacked-PR submit makes, so a take can show the success state without reaching GitHub."""
import json
import re
import sys
import threading
from http.server import BaseHTTPRequestHandler, ThreadingHTTPServer

REPO = "jayjay-demo/flightdeck"
pulls = {}
stacks = {}


def pull(number):
    return {"number": number, "html_url": f"https://github.com/{REPO}/pull/{number}", "state": "open"}


class Handler(BaseHTTPRequestHandler):
    def reply(self, body, status=200):
        data = json.dumps(body).encode()
        self.send_response(status)
        self.send_header("Content-Type", "application/json")
        self.send_header("Content-Length", str(len(data)))
        self.end_headers()
        self.wfile.write(data)

    def body(self):
        length = int(self.headers.get("Content-Length") or 0)
        return json.loads(self.rfile.read(length) or b"{}")

    def do_GET(self):
        path = self.path.split("?")[0]
        if path == "/user":
            return self.reply({"login": "jayjay-demo"})
        if path == f"/repos/{REPO}/pulls" or path == f"/repos/{REPO}/stacks":
            return self.reply([])
        if match := re.fullmatch(rf"/repos/{REPO}/pulls/(\d+)", path):
            return self.reply(pull(int(match[1])))
        self.reply({"message": "Not Found"}, 404)

    def do_POST(self):
        path, body = self.path, self.body()
        if path == f"/repos/{REPO}/pulls":
            number = 41 + len(pulls)
            pulls[number] = body
            return self.reply(pull(number), 201)
        if path == f"/repos/{REPO}/stacks":
            stacks[1] = body.get("pull_requests", [])
            return self.reply({"number": 1, "pull_requests": [{"number": n} for n in stacks[1]]}, 201)
        if match := re.fullmatch(rf"/repos/{REPO}/stacks/(\d+)/add", path):
            stack = stacks.setdefault(int(match[1]), [])
            stack += body.get("pull_requests", [])
            return self.reply({"number": int(match[1]), "pull_requests": [{"number": n} for n in stack]})
        if re.fullmatch(rf"/repos/{REPO}/issues/\d+/comments", path):
            return self.reply({"id": 1}, 201)
        self.reply({"message": "Not Found"}, 404)

    def do_PATCH(self):
        self.body()
        if match := re.fullmatch(rf"/repos/{REPO}/pulls/(\d+)", self.path):
            return self.reply(pull(int(match[1])))
        self.reply({"message": "Not Found"}, 404)


server = ThreadingHTTPServer(("127.0.0.1", int(sys.argv[1])), Handler)
threading.Timer(600, server.shutdown).start()
server.serve_forever()
