#!/bin/bash
set -euo pipefail

repo_root="$(cd "$(dirname "$0")/../.." && pwd)"
rive_runtime="${RIVE_RUNTIME_DIR:-/Users/levi/dev/oss/rive-runtime}"
expected_ref="d788e8ec6e8b598526607d6a1e8818e8b637b60c"
actual_ref="$(git -C "$rive_runtime" rev-parse HEAD)"
if [[ "$actual_ref" != "$expected_ref" ]]; then
    echo "frame-loop trace build requires pinned C++ $expected_ref; got $actual_ref" >&2
    exit 2
fi

runtime_out="out/rive-frame-loop-coverage-debug"
runtime_archive="$rive_runtime/$runtime_out/librive.a"
runtime_provenance="$runtime_archive.provenance"
trace_provenance="$runtime_archive.frame-loop-trace-provenance"
cpp_flags="-fprofile-instr-generate -fcoverage-mapping"
cxx_flags="$cpp_flags -DRIVE_GOLDEN_COVERAGE_TRACE"
link_flags="-fprofile-instr-generate"
expected_trace_provenance="$(
    printf '%s\n' \
        "schema=nuxie-frame-loop-trace-build-v1" \
        "runtime_revision=$expected_ref" \
        "config=debug" \
        "runtime_out=$runtime_out" \
        "cflags=$cpp_flags" \
        "cxxflags=$cxx_flags" \
        "ldflags=$link_flags"
)"

if [[ ! -f "$trace_provenance" ]] ||
    [[ "$(cat "$trace_provenance")" != "$expected_trace_provenance" ]]; then
    # This archive is isolated from ordinary golden builds. Removing only its
    # provenance stamp makes the existing helper rebuild with the trace flags.
    rm -f "$runtime_provenance" "$trace_provenance"
fi

env \
    CFLAGS="$cpp_flags" \
    CXXFLAGS="$cxx_flags" \
    LDFLAGS="$link_flags" \
    RIVE_GOLDEN_RUNTIME_OUT="$runtime_out" \
    RIVE_GOLDEN_RUNNER_NAME="rive_golden_runner_coverage" \
    RIVE_RUNTIME_DIR="$rive_runtime" \
    "$repo_root/tools/golden-runner/build.sh" debug

cpp_runner="$repo_root/tools/golden-runner/build/macosx/bin/debug/rive_golden_runner_coverage"
if ! nm "$runtime_archive" | grep "___llvm_profile_runtime_user" >/dev/null; then
    echo "trace librive archive has no LLVM profile runtime marker" >&2
    exit 2
fi
if ! nm "$cpp_runner" | grep "g_frameLoopAllocations" >/dev/null; then
    echo "trace C++ runner has no allocation-counter marker" >&2
    exit 2
fi
printf '%s\n' "$expected_trace_provenance" >"$trace_provenance"

env \
    CARGO_TARGET_DIR="$repo_root/target/frame-loop-coverage" \
    RUSTFLAGS="-Cinstrument-coverage" \
    cargo build --quiet --manifest-path "$repo_root/Cargo.toml" \
        -p rust-golden-runner --features coverage-trace

rust_runner="$repo_root/target/frame-loop-coverage/debug/rust-golden-runner"
if ! nm "$rust_runner" | grep "__llvm_profile_reset_counters" >/dev/null; then
    echo "trace Rust runner has no LLVM profile reset marker" >&2
    exit 2
fi

echo "frame-loop trace C++ runner: $cpp_runner"
echo "frame-loop trace Rust runner: $rust_runner"
