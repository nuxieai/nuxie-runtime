#!/bin/bash
# Build scripted C++ and Rust golden runners with LLVM coverage instrumentation
# so a frame loop can report exact per-function call counts on both sides
# (RIVE_GOLDEN_COVERAGE_FRAME_ONLY=1 writes only the frame loop's counters).
# Counting happens before inlining, so the counts are source-level calls: they
# show whether Rust does more operations than C++, or the same operations at
# a higher cost each. See tools/perf-gate/callcounts.py for the comparison.
#
# Both builds are isolated under target/callcount/ and never replace the
# ordinary release runners the timing gates use.
set -euo pipefail

repo_root="$(cd "$(dirname "$0")/../.." && pwd)"
rive_runtime="${RIVE_RUNTIME_DIR:-/Users/levi/dev/oss/rive-runtime}"
out="$repo_root/target/callcount"
mkdir -p "$out"

# Build-time tool runs (premake, build scripts) would otherwise drop profraw
# files into the working directories.
export LLVM_PROFILE_FILE=/dev/null

instrument="-fprofile-instr-generate -fcoverage-mapping"
env \
    CFLAGS="$instrument" \
    CXXFLAGS="$instrument -DRIVE_GOLDEN_COVERAGE_TRACE" \
    LDFLAGS="-fprofile-instr-generate" \
    RIVE_GOLDEN_WITH_SCRIPTING=1 \
    RIVE_GOLDEN_SCRIPTING_OUT="$out/cpp-librive-scripted-release" \
    RIVE_GOLDEN_RUNNER_NAME=rive_golden_runner_scripted_callcount \
    RIVE_RUNTIME_DIR="$rive_runtime" \
    "$repo_root/tools/golden-runner/build.sh" release

cpp_runner="$repo_root/tools/golden-runner/build/macosx/bin/release/rive_golden_runner_scripted_callcount"
if ! nm "$cpp_runner" | grep "___llvm_profile_reset_counters" >/dev/null; then
    echo "C++ call-count runner has no LLVM profile runtime" >&2
    exit 2
fi

env \
    CARGO_TARGET_DIR="$out/rust" \
    RUSTFLAGS="-Cinstrument-coverage" \
    python3 "$repo_root/tools/bazel/runtime.py" build --release --quiet --manifest-path "$repo_root/Cargo.toml" \
        -p rust-golden-runner --features scripting,coverage-trace

rust_runner="$out/rust/release/rust-golden-runner"
if ! nm "$rust_runner" | grep "__llvm_profile_reset_counters" >/dev/null; then
    echo "Rust call-count runner has no LLVM profile runtime" >&2
    exit 2
fi

echo "call-count C++ runner: $cpp_runner"
echo "call-count Rust runner: $rust_runner"
