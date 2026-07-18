#!/usr/bin/env bash
set -euo pipefail

repo_dir=$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)
scratch_dir=$(mktemp -d "${TMPDIR:-/tmp}/nuxie-wgpu-consumer.XXXXXX")
trap 'rm -rf -- "$scratch_dir"' EXIT

cargo new --quiet --lib "$scratch_dir/consumer"
cargo add --quiet \
  --manifest-path "$scratch_dir/consumer/Cargo.toml" \
  --path "$repo_dir/crates/nuxie-renderer" \
  nuxie-renderer
cargo add --quiet \
  --manifest-path "$scratch_dir/consumer/Cargo.toml" \
  --path "$repo_dir/crates/nuxie" \
  nuxie
cargo add --quiet \
  --manifest-path "$scratch_dir/consumer/Cargo.toml" \
  --path "$repo_dir/vendor/wgpu-30.0.0" \
  --no-default-features \
  --features angle,vulkan-portability,webgl \
  wgpu

metadata_path="$scratch_dir/metadata.json"
cargo metadata --format-version 1 \
  --manifest-path "$scratch_dir/consumer/Cargo.toml" >"$metadata_path"

python3 - "$metadata_path" "$repo_dir" <<'PY'
import json
import pathlib
import sys

metadata = json.loads(pathlib.Path(sys.argv[1]).read_text())
repo = pathlib.Path(sys.argv[2]).resolve()
required = {
    "wgpu": repo / "vendor/wgpu-30.0.0/Cargo.toml",
    "wgpu-core": repo / "vendor/wgpu-core-30.0.0/Cargo.toml",
    "wgpu-hal": repo / "vendor/wgpu-hal-30.0.0/Cargo.toml",
    "wgpu-core-deps-apple": repo / "vendor/wgpu-core-deps-apple-30.0.0/Cargo.toml",
    "wgpu-core-deps-emscripten": repo / "vendor/wgpu-core-deps-emscripten-30.0.0/Cargo.toml",
    "wgpu-core-deps-wasm": repo / "vendor/wgpu-core-deps-wasm-30.0.0/Cargo.toml",
    "wgpu-core-deps-windows-linux-android": repo / "vendor/wgpu-core-deps-windows-linux-android-30.0.0/Cargo.toml",
}
def is_guarded(name):
    return name in {"wgpu", "wgpu-core", "wgpu-hal"} or name.startswith(
        "wgpu-core-deps-"
    )

for package in metadata["packages"]:
    if is_guarded(package["name"]) and package["source"] is not None:
        raise SystemExit(
            f"{package['name']} unexpectedly resolved from {package['source']}"
        )

for name, expected_manifest in required.items():
    matches = [package for package in metadata["packages"] if package["name"] == name]
    if len(matches) != 1:
        raise SystemExit(f"expected one {name} package, found {len(matches)}")
    actual_manifest = pathlib.Path(matches[0]["manifest_path"]).resolve()
    if actual_manifest != expected_manifest:
        raise SystemExit(
            f"{name} manifest mismatch: expected {expected_manifest}, got {actual_manifest}"
        )
PY

cargo remove --quiet \
  --manifest-path "$scratch_dir/consumer/Cargo.toml" \
  wgpu

CARGO_TARGET_DIR="$repo_dir/target" cargo check --quiet --locked \
  --manifest-path "$scratch_dir/consumer/Cargo.toml"

echo "renderer-wgpu-consumer-check path-stack=pass compile=pass"
