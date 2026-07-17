#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/../.." && pwd)"
PORT="${BROWSER_RENDERER_SMOKE_PORT:-8765}"
SERVER_LOG="$ROOT/target/browser-renderer-smoke-server.log"
PLAYWRIGHT_VERSION=1.55.0
PLAYWRIGHT_ROOT="$ROOT/target/browser-tools/playwright"

if ! command -v npm >/dev/null 2>&1 || ! command -v node >/dev/null 2>&1; then
  echo "browser renderer smoke requires npm and node" >&2
  exit 1
fi

installed_version="$(
  node -p "try { require('$PLAYWRIGHT_ROOT/node_modules/playwright/package.json').version } catch (_) { '' }"
)"
if [[ "$installed_version" != "$PLAYWRIGHT_VERSION" ]]; then
  mkdir -p "$PLAYWRIGHT_ROOT"
  PLAYWRIGHT_SKIP_BROWSER_DOWNLOAD=1 npm install \
    --silent \
    --no-save \
    --package-lock=false \
    --prefix "$PLAYWRIGHT_ROOT" \
    "playwright@$PLAYWRIGHT_VERSION"
fi

cleanup() {
  if [[ -n "${SERVER_PID:-}" ]]; then
    kill "$SERVER_PID" >/dev/null 2>&1 || true
    wait "$SERVER_PID" >/dev/null 2>&1 || true
  fi
}
trap cleanup EXIT

"$ROOT/tools/browser-renderer-smoke/build.sh"
python3 -m http.server "$PORT" \
  --bind 127.0.0.1 \
  --directory "$ROOT" \
  >"$SERVER_LOG" 2>&1 &
SERVER_PID=$!

for _ in $(seq 1 50); do
  if curl --fail --silent "http://127.0.0.1:$PORT/" >/dev/null; then
    break
  fi
  if ! kill -0 "$SERVER_PID" 2>/dev/null; then
    cat "$SERVER_LOG" >&2
    exit 1
  fi
  sleep 0.1
done

NODE_PATH="$PLAYWRIGHT_ROOT/node_modules" \
  node "$ROOT/tools/browser-renderer-smoke/run-browser.cjs" \
  "http://127.0.0.1:$PORT/tools/browser-renderer-smoke/"
