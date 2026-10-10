#!/usr/bin/env bash
set -euo pipefail
runtime_root="$(cd "$(dirname "$0")/../.." && pwd)"
destination="$runtime_root/target/bazel-tools"
if [[ ! -x "$destination/node_modules/.bin/bazelisk" ]]; then
  npm install --prefix "$destination" --no-save --package-lock=false --ignore-scripts --loglevel=error @bazel/bazelisk@1.28.1
fi
