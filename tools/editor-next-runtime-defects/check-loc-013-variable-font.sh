#!/bin/sh
set -eu

repo_root=$(CDPATH= cd -- "$(dirname "$0")/../.." && pwd)
rive_runtime=${RIVE_RUNTIME_DIR:?set RIVE_RUNTIME_DIR to the pinned rive-runtime checkout}
editor_repo=${EDITOR_NEXT_REPO_DIR:?set EDITOR_NEXT_REPO_DIR to the Editor repository}
expected_rive=d788e8ec6e8b598526607d6a1e8818e8b637b60c
expected_editor=233552c13929b09666a62ddff541eb8620d1882b
helper="$repo_root/tools/editor-next-runtime-defects/loc013"
font="$editor_repo/apps/nuxie-dashboard/tests/visual/style-parity/fonts/InterVariable.ttf"
output="$repo_root/target/editor-next-runtime-defects/loc013"
runtime_libdir="$rive_runtime/out/rive-rust-golden-debug"
provenance="$repo_root/tools/golden-runner/runtime-provenance.sh"

sha256() {
    shasum -a 256 "$1" | awk '{print $1}'
}

require_sha256() {
    actual=$(sha256 "$1")
    if [ "$actual" != "$2" ]; then
        echo "LOC-013 SHA-256 mismatch for $1: expected $2, found $actual" >&2
        exit 1
    fi
}

test "$(git -C "$rive_runtime" rev-parse HEAD)" = "$expected_rive"
git -C "$rive_runtime" diff --quiet --ignore-submodules --
git -C "$rive_runtime" diff --cached --quiet --ignore-submodules --
test -z "$(git -C "$rive_runtime" status --porcelain --untracked-files=no)"
test "$(git -C "$editor_repo" rev-parse HEAD)" = "$expected_editor"
git -C "$editor_repo" diff --quiet --ignore-submodules --
git -C "$editor_repo" diff --cached --quiet --ignore-submodules --
test -z "$(git -C "$editor_repo" status --porcelain --untracked-files=no)"
require_sha256 \
    "$font" \
    4989b125924991b90d05b2d16e0e388c48f7d5bb8b30539bbf9c755278d0ccaf
require_sha256 \
    "$helper/cpp_probe.cpp" \
    8460d1463e627eb22e4861e4f6ab0554664877bb757c90c794bd4dea264b7aa1
test "$(/usr/bin/clang++ --version | sed -n '1p')" = \
    "Apple clang version 21.0.0 (clang-2100.1.1.101)"
require_sha256 \
    /usr/bin/clang++ \
    179301dcb41ea78accc3fa0048a7e6f6710d891945a751a34addd622020c1818

make -C "$repo_root" golden-runner RIVE_RUNTIME_DIR="$rive_runtime" CPP_CONFIG=debug
"$provenance" verify \
    "$rive_runtime" \
    "$runtime_libdir/librive.a" \
    "$runtime_libdir/rive.make" \
    "$runtime_libdir/librive.a.provenance" \
    debug \
    ordinary

test "$(uname -s)" = Darwin
golden_runner="$repo_root/tools/golden-runner/build/macosx/bin/debug/rive_golden_runner"
test -x "$golden_runner"

set -- "$rive_runtime"/dependencies/rive-app_harfbuzz_*/src
test "$#" -eq 1
harfbuzz_include=$1
mkdir -p "$output"

/usr/bin/clang++ \
    -std=c++17 \
    -fno-rtti \
    -Wall \
    -Wextra \
    -Werror \
    -D_RIVE_INTERNAL_ \
    -DWITH_RIVE_TEXT \
    -DWITH_RIVE_LAYOUT \
    -DRIVE_MACOSX \
    -DYOGA_EXPORT= \
    -DDEBUG \
    -isystem "$rive_runtime/include" \
    -isystem "$harfbuzz_include" \
    "$helper/cpp_probe.cpp" \
    -L"$runtime_libdir" \
    -lrive \
    -lrive_harfbuzz \
    -lrive_sheenbidi \
    -lrive_yoga \
    -framework Cocoa \
    -framework CoreFoundation \
    -framework IOKit \
    -framework Security \
    -lbz2 \
    -liconv \
    -llzma \
    -lz \
    -o "$output/loc013-cpp-probe"
require_sha256 \
    "$output/loc013-cpp-probe" \
    22989137e12ccc3faf8148440e925153a277f81b3b88ab5fa92d98dbae2cd12b

"$output/loc013-cpp-probe" "$font" >"$output/cpp.json"
CARGO_TARGET_DIR="$output/cargo-target" \
    cargo run \
    --quiet \
    --locked \
    --manifest-path "$helper/Cargo.toml" \
    -- \
    direct "$font" >"$output/rust.json"
python3 "$helper/compare.py" \
    direct "$output/cpp.json" "$output/rust.json" >"$output/direct-result.json"

CARGO_TARGET_DIR="$output/cargo-target" \
    cargo run \
    --quiet \
    --locked \
    --manifest-path "$helper/Cargo.toml" \
    -- \
    scene "$font" "$output/loc013.riv" "$output/rust.stream" \
    2>"$output/scene.log"
(
    cd "$output"
    "$golden_runner" \
        --file loc013.riv \
        --artboard "LOC013 Variable Font" \
        --samples 0 >cpp.stream
)
python3 "$helper/compare.py" \
    stream "$output/cpp.stream" "$output/rust.stream" >"$output/stream-result.json"

require_sha256 \
    "$output/cpp.json" \
    515b8e6748bcb0b65635f3349ecacfa57ae7cbbfc8100c2ce2b01ace43e363cc
require_sha256 \
    "$output/rust.json" \
    61bd2571445566f6c66a381ea7860c0fb8d18136fa81d55857f7abb8689a9c8d
require_sha256 \
    "$output/direct-result.json" \
    aada413fc5dc55a7498fe8d6733525117b4d67a924235d53562c12038a9ad7cd
require_sha256 \
    "$output/loc013.riv" \
    121965b51165b5ed6198189236fc992d5cd1013665c442bad3a42172a43efcf8
require_sha256 \
    "$output/cpp.stream" \
    897a76c374d064037a3a903331cdc3ab86683cebbe943dbc19834da0567f2a94
require_sha256 \
    "$output/rust.stream" \
    36860da1e2efb2610782a18ed290d1cf48acda5f88d965043e68716d4f556a08
require_sha256 \
    "$output/stream-result.json" \
    0a1b18c6520dd2ff30efd1fd5e53c71adef64722eee2c4cd98cef1ad4c2fcf21

make -C "$repo_root" \
    renderer-rust-replay-release \
    renderer-dawn-live-reference-replay \
    renderer-dawn-live-reference-check \
    RIVE_RUNTIME_DIR="$rive_runtime" \
    RENDERER_JOBS="${RENDERER_JOBS:-4}"
cpp_replay="$repo_root/target/renderer-dawn-live-reference/renderer-replay"
rust_replay="$repo_root/target/renderer-golden/release/renderer-replay"
require_sha256 \
    "$cpp_replay" \
    af9be5d64d9757d42afc0eb15852b25bb5ab7ca39e06273fbcd15c581473d63b
require_sha256 \
    "$rust_replay" \
    1259afaa779a2efee71823a9906544613c58a1dd03af19db1a98bd8d4822ef25

mkdir -p "$output/pixels"
"$cpp_replay" \
    --stream "$output/cpp.stream" \
    --output "$output/pixels/cpp.png" \
    --backend ffi-dawn \
    --mode msaa
"$rust_replay" \
    --stream "$output/rust.stream" \
    --output "$output/pixels/rust.png" \
    --backend rust-wgpu \
    --mode msaa
cargo run \
    --quiet \
    --locked \
    --manifest-path "$repo_root/tools/pixel-compare/Cargo.toml" \
    --bin pixel-compare \
    -- \
    "$output/pixels/cpp.png" \
    "$output/pixels/rust.png" \
    --max-channel-delta 0 \
    --max-different-pixels 0 \
    --artifact "$output/pixels/diff.png" >"$output/pixel-result.txt"
require_sha256 \
    "$output/pixels/cpp.png" \
    8e54706fb740e462e58046a9b396cb535e335c454a1c1d06b2a6a814c8662287
require_sha256 \
    "$output/pixels/rust.png" \
    8e54706fb740e462e58046a9b396cb535e335c454a1c1d06b2a6a814c8662287
require_sha256 \
    "$output/pixel-result.txt" \
    d2e2fd8cbb7a1430657fd77c445ce1251f5eb23750f2e720400836c75ed24982

echo "LOC-013 variable-font evidence: C++ and Rust exact through supported WebGPU pixels"
