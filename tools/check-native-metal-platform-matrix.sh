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

for target in "${stable_targets[@]}"; do
    echo "checking native Metal platform configuration: $target"
    RUSTFLAGS="$matrix_rustflags" \
        python3 tools/bazel/runtime.py check -q -p nuxie-renderer \
        --target "$target" --no-default-features --features "$features" --lib \
        2>&1 | tee "$evidence_dir/$target.log"
done

printf '%s\n' "${stable_targets[@]}" > "$evidence_dir/targets.txt"
printf 'Bazel std toolchains remain unqualified for: %s\n' "${build_std_targets[*]}" >&2
echo 'Complete the tvOS/visionOS toolchains tracked in https://universe.basis.dev/issue/UNIV-4184 before qualifying all nine Metal configurations.' >&2
exit 2
