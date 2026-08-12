#!/usr/bin/env bash
set -euo pipefail

root="$(cd "$(dirname "$0")/.." && pwd)"

PYTHONDONTWRITEBYTECODE=1 python3 \
    "$root/tools/renderer-shader-boundary/check_no_indirect.py" \
    "$root/crates/nuxie-renderer/src"
