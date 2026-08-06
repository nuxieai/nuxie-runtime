#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/../.." && pwd)"
RIVE_RUNTIME_DIR="${RIVE_RUNTIME_DIR:?RIVE_RUNTIME_DIR is required}"
RUST_GOLDEN_RUNNER="${RUST_GOLDEN_RUNNER:?RUST_GOLDEN_RUNNER is required}"
PORT="${BROWSER_WASM_PERF_PORT:-8766}"
LIMIT="${WASM_PERF_LIMIT:-5}"
IDS="${WASM_PERF_IDS:-}"
REPEAT="${WASM_PERF_REPEAT:-100}"
RUNS="${WASM_PERF_RUNS:-5}"
WARMUPS="${WASM_PERF_WARMUPS:-1}"
OUTPUT="${WASM_PERF_OUTPUT:-$ROOT/target/wasm-perf.json}"
MARKDOWN="${WASM_PERF_MARKDOWN:-$ROOT/target/wasm-perf.md}"
WORK_DIR="$ROOT/target/browser-wasm-perf"
CONFIG="$WORK_DIR/config.json"
BROWSER_RESULTS="$WORK_DIR/browser-results.json"
SERVER_LOG="$WORK_DIR/server.log"
PLAYWRIGHT_VERSION=1.55.0
PLAYWRIGHT_ROOT="$ROOT/target/browser-tools/playwright"
STABLE_CARGO="${CARGO:-$(rustup which --toolchain stable cargo)}"

if ! command -v npm >/dev/null 2>&1 || ! command -v node >/dev/null 2>&1; then
  echo "wasm perf requires npm and node" >&2
  exit 1
fi

mkdir -p "$WORK_DIR"
PYTHONDONTWRITEBYTECODE=1 python3 "$ROOT/tools/browser-renderer-smoke/wasm_perf.py" audit \
  --repo-root "$ROOT" \
  --cargo "$STABLE_CARGO" \
  --source "$ROOT/tools/browser-renderer-smoke/src/lib.rs"
PYTHONDONTWRITEBYTECODE=1 python3 "$ROOT/tools/browser-renderer-smoke/wasm_perf.py" prepare \
  --repo-root "$ROOT" \
  --rive-runtime-dir "$RIVE_RUNTIME_DIR" \
  --perf-manifest "$ROOT/perf-corpus.toml" \
  --corpus "$ROOT/corpus.toml" \
  --staging-dir "$WORK_DIR/fixtures" \
  --config "$CONFIG" \
  --limit "$LIMIT" \
  --ids "$IDS" \
  --repeat "$REPEAT" \
  --runs "$RUNS" \
  --warmups "$WARMUPS"

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

BROWSER_RENDERER_PRODUCTION_ONLY=1 "$ROOT/tools/browser-renderer-smoke/build.sh"
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
    sed -n '1,200p' "$SERVER_LOG" >&2
    exit 1
  fi
  sleep 0.1
done

NODE_PATH="$PLAYWRIGHT_ROOT/node_modules" \
  node "$ROOT/tools/browser-renderer-smoke/run-wasm-perf.cjs" \
  "http://127.0.0.1:$PORT/tools/browser-renderer-smoke/" \
  "$CONFIG" \
  "$BROWSER_RESULTS"

PYTHONDONTWRITEBYTECODE=1 python3 "$ROOT/tools/browser-renderer-smoke/wasm_perf.py" finalize \
  --config "$CONFIG" \
  --browser-results "$BROWSER_RESULTS" \
  --native-runner "$RUST_GOLDEN_RUNNER" \
  --output "$OUTPUT" \
  --markdown "$MARKDOWN"

echo "wasm perf report: $OUTPUT"
echo "wasm perf summary: $MARKDOWN"
