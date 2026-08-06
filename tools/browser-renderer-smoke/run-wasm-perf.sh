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
COORDINATOR_BUNDLE_NAME="$(basename "$COORDINATOR_DIR")"
if [[ "$EXECUTED_COORDINATOR_DIR" != "$COORDINATOR_DIR" \
  || ! "$COORDINATOR_BUNDLE_NAME" =~ ^[0-9a-f]{64}$ \
  || ! -f "$COORDINATOR_DIR/manifest.json" ]]; then
  echo "invalid content-addressed coordinator bundle" >&2
  exit 1
fi
if ! python3 - "$COORDINATOR_DIR" "$ROOT" "$0" <<'PY'
import hashlib
import json
import subprocess
import sys
from pathlib import Path

bundle = Path(sys.argv[1]).resolve()
repo = Path(sys.argv[2]).resolve()
executed_shell = Path(sys.argv[3]).resolve()
manifest_bytes = (bundle / "manifest.json").read_bytes()
if hashlib.sha256(manifest_bytes).hexdigest() != bundle.name:
    raise SystemExit("coordinator manifest digest differs from bundle path")
manifest = json.loads(manifest_bytes)
if manifest.get("schema") != "nuxie-wasm-perf-coordinator-bundle-v1":
    raise SystemExit("invalid coordinator manifest schema")
expected_names = {
    "run-wasm-perf.sh",
    "run-wasm-perf.cjs",
    "wasm_perf.py",
    "wasm-perf-driver-lib.cjs",
}
files = manifest.get("files")
source = manifest.get("source")
if not isinstance(files, dict) or set(files) != expected_names:
    raise SystemExit("coordinator manifest member set differs")
if not isinstance(source, dict) or set(source) != {"repo_sha", "repo_tree_sha"}:
    raise SystemExit("coordinator manifest source identity is invalid")
if {path.name for path in bundle.iterdir()} != expected_names | {"manifest.json"}:
    raise SystemExit("coordinator bundle member set differs")
for name, expected in files.items():
    if not isinstance(expected, dict) or set(expected) != {"bytes", "sha256"}:
        raise SystemExit(f"invalid coordinator member identity: {name}")
    contents = (bundle / name).read_bytes()
    current = {
        "bytes": len(contents),
        "sha256": hashlib.sha256(contents).hexdigest(),
    }
    if current != expected:
        raise SystemExit(f"coordinator member identity mismatch: {name}")
if executed_shell != bundle / "run-wasm-perf.sh":
    raise SystemExit("executed shell differs from coordinator bundle")
status = subprocess.run(
    ["git", "-C", str(repo), "status", "--porcelain=v1", "--untracked-files=no"],
    text=True,
    capture_output=True,
    check=True,
).stdout
current_source = {
    "repo_sha": subprocess.run(
        ["git", "-C", str(repo), "rev-parse", "HEAD"],
        text=True,
        capture_output=True,
        check=True,
    ).stdout.strip(),
    "repo_tree_sha": subprocess.run(
        ["git", "-C", str(repo), "rev-parse", "HEAD^{tree}"],
        text=True,
        capture_output=True,
        check=True,
    ).stdout.strip(),
}
if status or current_source != source:
    raise SystemExit("coordinator bundle source identity differs from clean checkout")
PY
then
  echo "invalid content-addressed coordinator bundle" >&2
  exit 1
fi
PYTHON_COORDINATOR="$COORDINATOR_DIR/wasm_perf.py"
NODE_COORDINATOR="$COORDINATOR_DIR/run-wasm-perf.cjs"
exec 9<"$PYTHON_COORDINATOR"
PYTHON_COORDINATOR_FD_PATH="/dev/fd/9"
PYTHON_COORDINATOR_LOADER='import os; os.lseek(9, 0, os.SEEK_SET); source = b"".join(iter(lambda: os.read(9, 1048576), b"")); exec(compile(source, "<sealed-wasm-perf-coordinator>", "exec"), {"__name__": "__main__", "__file__": "<sealed-wasm-perf-coordinator>"})'
run_python_coordinator() {
  PYTHONDONTWRITEBYTECODE=1 python3 -c "$PYTHON_COORDINATOR_LOADER" "$@"
}
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
run_python_coordinator audit \
  --repo-root "$ROOT" \
  --cargo "$STABLE_CARGO" \
  --source "$ROOT/tools/browser-renderer-smoke/src/lib.rs" \
  --shell-source "$COORDINATOR_DIR/run-wasm-perf.sh"
run_python_coordinator prepare \
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
RUN_SEAL_SHA256="$(run_python_coordinator seal \
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

run_python_coordinator finalize \
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
