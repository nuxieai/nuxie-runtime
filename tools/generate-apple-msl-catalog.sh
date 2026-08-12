#!/bin/bash
set -euo pipefail

root="$(cd "$(dirname "$0")/.." && pwd)"
cargo run --locked --manifest-path "$root/Cargo.toml" -p apple-msl-catalog -- generate "$root"
