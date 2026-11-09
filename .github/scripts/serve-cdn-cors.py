#!/usr/bin/env python3
"""Read-only static file server with permissive CORS for CI browsers.

Serves a local CDN mirror (the `cdn-data` artifact) inside the `wasm-test`
job so the browser test suite fetches dictionary and content bytes from the
job itself instead of the production CDN (hermetic wasm suite; see the
`Serve CDN mirror locally` step in .github/workflows/ci.yml).

- `Access-Control-Allow-Origin: *` is REQUIRED: the wasm-bindgen harness
  page runs on a different origin (127.0.0.1:<ephemeral> port), so every
  fetch to this server is cross-origin. The e2e twin of this mechanism
  (the corpus webServer in end2end/playwright.config.ts — its directory
  is resolved by end2end/shots/config.ts) relies on serve's own CORS
  default; this job has no node, hence the stdlib server.
- No `Content-Encoding` header: mirror files are already raw-deflate
  compressed exactly as the production CDN serves them, and the app code
  inflates them itself. A server that transparently decodes (or announces
  an encoding) would break that inflate step.

Usage: python3 serve-cdn-cors.py <root-dir> <port>
"""

import sys
from http.server import SimpleHTTPRequestHandler, ThreadingHTTPServer


class CorsStaticHandler(SimpleHTTPRequestHandler):
    def end_headers(self):
        self.send_header("Access-Control-Allow-Origin", "*")
        super().end_headers()

    def log_message(self, fmt, *args):
        # One line per request to stderr; the job redirects this to a file
        # and dumps it after the test step (hermeticity evidence: which
        # URLs the suite actually consumed, all answered from localhost).
        sys.stderr.write("%s %s\n" % (self.address_string(), fmt % args))
        sys.stderr.flush()


def main() -> None:
    if len(sys.argv) != 3:
        sys.exit("usage: serve-cdn-cors.py <root-dir> <port>")
    root, port = sys.argv[1], int(sys.argv[2])
    server = ThreadingHTTPServer(
        ("127.0.0.1", port),
        lambda *args: CorsStaticHandler(*args, directory=root),
    )
    server.serve_forever()


if __name__ == "__main__":
    main()
