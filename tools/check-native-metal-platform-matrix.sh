#!/usr/bin/env bash
set -euo pipefail

repo_root="$(cd "$(dirname "$0")/.." && pwd)"
cd "$repo_root"

features="native-ore-metal-experimental,rive-decoders"
evidence_dir="$repo_root/target/native-metal-platform-matrix"
mkdir -p "$evidence_dir"

# Keep the workspace warning-clean without turning the vendored wgpu-core
# `expect(unused)` compatibility annotation into a port failure.
matrix_rustflags="${RUSTFLAGS:+$RUSTFLAGS }-Dwarnings -Aunfulfilled-lint-expectations"

stable_rustc="$(rustup which --toolchain stable rustc)"
nightly_rustc="$(rustup which --toolchain nightly rustc)"

stable_targets=(
    aarch64-apple-darwin
    x86_64-apple-darwin
    aarch64-apple-ios
    aarch64-apple-ios-sim
    x86_64-apple-ios
)

build_std_targets=(
    aarch64-apple-tvos
    aarch64-apple-tvos-sim
    aarch64-apple-visionos
    aarch64-apple-visionos-sim
)

installed_stable_targets="$(rustup target list --toolchain stable --installed)"
for target in "${stable_targets[@]}"; do
    if ! grep -Fqx "$target" <<<"$installed_stable_targets"; then
        echo "missing stable Rust target: $target" >&2
        exit 2
    fi
    echo "checking native Metal platform configuration: $target"
    RUSTC="$stable_rustc" RUSTFLAGS="$matrix_rustflags" \
        rustup run stable cargo check -q -p nuxie-renderer \
        --target "$target" --no-default-features --features "$features" --lib \
        2>&1 | tee "$evidence_dir/$target.log"
done

nightly_sysroot="$(rustup run nightly rustc --print sysroot)"
if [[ ! -d "$nightly_sysroot/lib/rustlib/src/rust/library" ]]; then
    echo "nightly rust-src is required for tvOS and visionOS build-std checks" >&2
    exit 2
fi

for target in "${build_std_targets[@]}"; do
    echo "checking native Metal platform configuration with build-std: $target"
    RUSTC="$nightly_rustc" RUSTFLAGS="$matrix_rustflags" \
        rustup run nightly cargo check -q -Z build-std=std,panic_abort \
        -p nuxie-renderer --target "$target" --no-default-features \
        --features "$features" --lib \
        2>&1 | tee "$evidence_dir/$target.log"
done

printf '%s\n' "${stable_targets[@]}" "${build_std_targets[@]}" \
    > "$evidence_dir/targets.txt"
echo "native Metal platform matrix passed: 9/9 checked configurations"
