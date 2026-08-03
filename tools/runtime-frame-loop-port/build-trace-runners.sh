#!/bin/bash
set -euo pipefail

repo_root="$(cd "$(dirname "$0")/../.." && pwd)"
rive_runtime="${RIVE_RUNTIME_DIR:-/Users/levi/dev/oss/rive-runtime}"
expected_ref="4ac7b32798da0482e441ef09304dc3b480ed3ee5"
actual_ref="$(git -C "$rive_runtime" rev-parse HEAD)"
if [[ "$actual_ref" != "$expected_ref" ]]; then
    echo "frame-loop trace build requires pinned C++ $expected_ref; got $actual_ref" >&2
    exit 2
fi

runtime_out="${RIVE_FRAME_LOOP_CPP_RUNTIME_OUT:-$repo_root/target/runtime-frame-loop-trace/cpp-runtime}"
if [[ "$runtime_out" = /* ]]; then
    runtime_archive="$runtime_out/librive.a"
else
    runtime_archive="$rive_runtime/$runtime_out/librive.a"
fi
runtime_provenance="$runtime_archive.provenance"
trace_provenance="$runtime_archive.frame-loop-trace-provenance"
build_profile_dir="$repo_root/target/runtime-frame-loop-trace/build-profiles"
mkdir -p "$build_profile_dir"
build_profile_pattern="$build_profile_dir/%m-%p.profraw"
cpp_flags="-fprofile-instr-generate=$build_profile_pattern -fcoverage-mapping"
cxx_flags="$cpp_flags -DRIVE_GOLDEN_COVERAGE_TRACE"
link_flags="-fprofile-instr-generate=$build_profile_pattern"
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
    python3 - "$runtime_provenance" "$trace_provenance" <<'PY'
import pathlib
import sys

for value in sys.argv[1:]:
    pathlib.Path(value).unlink(missing_ok=True)
PY
fi

env \
    CFLAGS="$cpp_flags" \
    CXXFLAGS="$cxx_flags" \
    LDFLAGS="$link_flags" \
    LLVM_PROFILE_FILE="$build_profile_pattern" \
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

source_fingerprint_tool="$repo_root/tools/runtime-frame-loop-port/source_fingerprint.py"
trace_evidence="$repo_root/docs/runtime-frame-loop-trace.json"
rust_runner="$repo_root/target/frame-loop-coverage/debug/rust-golden-runner"
rust_trace_provenance="$rust_runner.frame-loop-trace-provenance"
rust_provenance_before="$(
    python3 "$source_fingerprint_tool" \
        --repo-root "$repo_root" \
        --evidence-path "$trace_evidence" \
        --runner-provenance
)"
python3 - "$rust_trace_provenance" <<'PY'
import pathlib
import sys

pathlib.Path(sys.argv[1]).unlink(missing_ok=True)
PY

env \
    CARGO_TARGET_DIR="$repo_root/target/frame-loop-coverage" \
    LLVM_PROFILE_FILE="$build_profile_pattern" \
    RUSTFLAGS="-Cinstrument-coverage" \
    cargo build --quiet --manifest-path "$repo_root/Cargo.toml" \
        -p rust-golden-runner --features coverage-trace

if ! nm "$rust_runner" | grep "__llvm_profile_reset_counters" >/dev/null; then
    echo "trace Rust runner has no LLVM profile reset marker" >&2
    exit 2
fi
rust_provenance_after="$(
    python3 "$source_fingerprint_tool" \
        --repo-root "$repo_root" \
        --evidence-path "$trace_evidence" \
        --runner-provenance
)"
if [[ "$rust_provenance_after" != "$rust_provenance_before" ]]; then
    echo "Rust candidate source changed during trace runner build" >&2
    exit 2
fi
printf '%s\n' "$rust_provenance_after" >"$rust_trace_provenance"

echo "frame-loop trace C++ runner: $cpp_runner"
echo "frame-loop trace Rust runner: $rust_runner"
