#!/usr/bin/env bash
set -euo pipefail

SOURCE_ROOT="${WASM_PERF_SOURCE_ROOT:-$(cd "$(dirname "$0")/../.." && pwd)}"
SOURCE_COORDINATOR_DIR="$SOURCE_ROOT/tools/browser-renderer-smoke"
SOURCE_WORK_DIR="$SOURCE_ROOT/target/browser-wasm-perf"

if [[ -z "${WASM_PERF_SEALED_COORDINATOR_BUNDLE:-}" ]]; then
  BOOTSTRAP_STATUS="$(git -C "$SOURCE_ROOT" status --porcelain=v1 --untracked-files=no)"
  if [[ -n "$BOOTSTRAP_STATUS" ]]; then
    echo "wasm perf coordinator source checkout is dirty: ${BOOTSTRAP_STATUS%%$'\n'*}" >&2
    exit 1
  fi
  BOOTSTRAP_REPO_SHA="$(git -C "$SOURCE_ROOT" rev-parse HEAD)"
  BOOTSTRAP_REPO_TREE_SHA="$(git -C "$SOURCE_ROOT" rev-parse 'HEAD^{tree}')"
  mkdir -p "$SOURCE_WORK_DIR/coordinators"
  BOOTSTRAP_DIR="$(mktemp -d "$SOURCE_WORK_DIR/coordinators/.bootstrap.XXXXXX")"
  BOOTSTRAP_PYTHON="$BOOTSTRAP_DIR/wasm_perf.py"
  git -C "$SOURCE_ROOT" show \
    "$BOOTSTRAP_REPO_SHA:tools/browser-renderer-smoke/wasm_perf.py" \
    >"$BOOTSTRAP_PYTHON"
  chmod 400 "$BOOTSTRAP_PYTHON"
  COORDINATOR_BUNDLE="$(PYTHONDONTWRITEBYTECODE=1 python3 "$BOOTSTRAP_PYTHON" stage-coordinator \
    --repo-root "$SOURCE_ROOT" \
    --expected-repo-sha "$BOOTSTRAP_REPO_SHA" \
    --expected-repo-tree-sha "$BOOTSTRAP_REPO_TREE_SHA" \
    --output-root "$SOURCE_WORK_DIR/coordinators" \
    --coordinator "run-wasm-perf.sh=tools/browser-renderer-smoke/run-wasm-perf.sh" \
    --coordinator "run-wasm-perf.cjs=tools/browser-renderer-smoke/run-wasm-perf.cjs" \
    --coordinator "wasm_perf.py=tools/browser-renderer-smoke/wasm_perf.py" \
    --coordinator "wasm-perf-driver-lib.cjs=tools/browser-renderer-smoke/wasm-perf-driver-lib.cjs")"
  rm "$BOOTSTRAP_PYTHON"
  rmdir "$BOOTSTRAP_DIR"
  CURRENT_BOOTSTRAP_STATUS="$(git -C "$SOURCE_ROOT" status --porcelain=v1 --untracked-files=no)"
  CURRENT_BOOTSTRAP_REPO_SHA="$(git -C "$SOURCE_ROOT" rev-parse HEAD)"
  CURRENT_BOOTSTRAP_REPO_TREE_SHA="$(git -C "$SOURCE_ROOT" rev-parse 'HEAD^{tree}')"
  if [[ -n "$CURRENT_BOOTSTRAP_STATUS" \
    || "$CURRENT_BOOTSTRAP_REPO_SHA" != "$BOOTSTRAP_REPO_SHA" \
    || "$CURRENT_BOOTSTRAP_REPO_TREE_SHA" != "$BOOTSTRAP_REPO_TREE_SHA" ]]; then
    echo "wasm perf coordinator source changed during Git-blob staging" >&2
    exit 1
  fi
  exec env \
    WASM_PERF_SOURCE_ROOT="$SOURCE_ROOT" \
    WASM_PERF_SEALED_COORDINATOR_BUNDLE="$COORDINATOR_BUNDLE" \
    "$COORDINATOR_BUNDLE/run-wasm-perf.sh" "$@"
fi

ROOT="$SOURCE_ROOT"
COORDINATOR_DIR="$WASM_PERF_SEALED_COORDINATOR_BUNDLE"
EXECUTED_COORDINATOR_DIR="$(cd "$(dirname "$0")" && pwd)"
if [[ "$EXECUTED_COORDINATOR_DIR" != "$COORDINATOR_DIR" ]]; then
  echo "wasm perf must execute the content-addressed coordinator shell" >&2
  exit 1
fi
PYTHON_COORDINATOR="$COORDINATOR_DIR/wasm_perf.py"
NODE_COORDINATOR="$COORDINATOR_DIR/run-wasm-perf.cjs"
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
SEAL="$WORK_DIR/seal.json"
BROWSER_RESULTS="$WORK_DIR/browser-results.json"
SERVER_LOG="$WORK_DIR/server.log"
WASM_ARTIFACT="$ROOT/tools/browser-renderer-smoke/pkg/browser_renderer_smoke_bg.wasm"
WASM_BINDGEN_JS="$ROOT/tools/browser-renderer-smoke/pkg/browser_renderer_smoke.js"
WASM_PERF_DRIVER_JS="$COORDINATOR_DIR/wasm-perf-driver-lib.cjs"
WASM_PERF_HTML="$ROOT/tools/browser-renderer-smoke/wasm-perf.html"
GENERATED_PKG="$ROOT/tools/browser-renderer-smoke/pkg"
PLAYWRIGHT_VERSION=1.55.0
PLAYWRIGHT_ROOT="$ROOT/target/browser-tools/playwright"
STABLE_CARGO="${CARGO:-$(rustup which --toolchain stable cargo)}"

if ! command -v npm >/dev/null 2>&1 || ! command -v node >/dev/null 2>&1; then
  echo "wasm perf requires npm and node" >&2
  exit 1
fi

mkdir -p "$WORK_DIR"
PYTHONDONTWRITEBYTECODE=1 python3 "$PYTHON_COORDINATOR" audit \
  --repo-root "$ROOT" \
  --cargo "$STABLE_CARGO" \
  --source "$ROOT/tools/browser-renderer-smoke/src/lib.rs"
PYTHONDONTWRITEBYTECODE=1 python3 "$PYTHON_COORDINATOR" prepare \
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
  --warmups "$WARMUPS" \
  --allowed-output "$WORK_DIR" \
  --allowed-output "$OUTPUT" \
  --allowed-output "$MARKDOWN" \
  --allowed-output "$GENERATED_PKG"

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
RUN_SEAL_SHA256="$(PYTHONDONTWRITEBYTECODE=1 python3 "$PYTHON_COORDINATOR" seal \
  --config "$CONFIG" \
  --seal "$SEAL" \
  --repo-root "$ROOT" \
  --rive-runtime-dir "$RIVE_RUNTIME_DIR" \
  --native-runner "$RUST_GOLDEN_RUNNER" \
  --wasm-artifact "$WASM_ARTIFACT" \
  --wasm-bindgen-js "$WASM_BINDGEN_JS" \
  --wasm-perf-driver-js "$WASM_PERF_DRIVER_JS" \
  --wasm-perf-html "$WASM_PERF_HTML" \
  --wasm-perf-node "$NODE_COORDINATOR" \
  --wasm-perf-python "$PYTHON_COORDINATOR" \
  --wasm-perf-shell "$COORDINATOR_DIR/run-wasm-perf.sh" \
  --allowed-output "$WORK_DIR" \
  --allowed-output "$OUTPUT" \
  --allowed-output "$MARKDOWN" \
  --allowed-output "$GENERATED_PKG")"
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
  node "$NODE_COORDINATOR" \
  "http://127.0.0.1:$PORT/tools/browser-renderer-smoke/" \
  "$CONFIG" \
  "$BROWSER_RESULTS" \
  "$SEAL" \
  "$RUN_SEAL_SHA256"

PYTHONDONTWRITEBYTECODE=1 python3 "$PYTHON_COORDINATOR" finalize \
  --config "$CONFIG" \
  --seal "$SEAL" \
  --expected-seal-sha256 "$RUN_SEAL_SHA256" \
  --browser-results "$BROWSER_RESULTS" \
  --native-runner "$RUST_GOLDEN_RUNNER" \
  --repo-root "$ROOT" \
  --rive-runtime-dir "$RIVE_RUNTIME_DIR" \
  --allowed-output "$WORK_DIR" \
  --allowed-output "$OUTPUT" \
  --allowed-output "$MARKDOWN" \
  --allowed-output "$GENERATED_PKG" \
  --output "$OUTPUT" \
  --markdown "$MARKDOWN"

echo "wasm perf report: $OUTPUT"
echo "wasm perf summary: $MARKDOWN"
