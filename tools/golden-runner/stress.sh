#!/bin/bash
# Crash-stability stress loop for the pinned C++ golden runner.
#
# The golden gate's authority rests on the oracle process not crashing. The
# pinned upstream (port-manifest.toml upstream_ref) has a known artboard-swap
# use-after-free (register row W3): NestedArtboard::updateArtboard destroys
# the outgoing ArtboardInstance while the shared SemanticManager still holds
# raw pointers into it, so the next drainDiff() reconciles bounds through
# freed memory. The oracle carries a registered fix
# (tools/rive-runtime-patches/librive-0001-nested-artboard-swap-semantic-uaf.patch,
# UNIV-1524); crashes are heap-layout dependent, so an unpatched runner
# resurfaces as a NONDETERMINISTIC gate flake that moves between corpus
# entries. This loop re-runs the historically crashing entries enough times
# that a patch-mechanism regression fails hard and attributably here instead.
#
# usage: stress.sh <runner> <iterations> <entry-id>...
#
# Invocations mirror the golden-compare gate: --side-channel is dropped for
# entries quarantined with a "side-channel-diverges:<row>" feature (as the
# comparator does), but those entries ALSO run a --side-channel variant here
# because that is where the W3 use-after-free historically fired.
#
# For a sanitizer-instrumented runner (the strongest version of this check),
# build with:
#   RIVE_GOLDEN_RUNTIME_OUT="$PWD/target/golden-runner-librive/asan-debug" \
#   RIVE_GOLDEN_RUNNER_NAME=rive_golden_runner_asan \
#   CFLAGS='-fsanitize=address,undefined -fno-omit-frame-pointer' \
#   CXXFLAGS='-fsanitize=address,undefined -fno-omit-frame-pointer' \
#   LDFLAGS='-fsanitize=address,undefined' \
#   tools/golden-runner/build.sh debug
# (`make golden-runner-stress-asan` does exactly this.) The premake gmake
# makefiles fold environment CFLAGS/CXXFLAGS/LDFLAGS into every compile and
# link, and the separate *_OUT/_NAME keep the provenance-bound gate archives
# untouched.
set -euo pipefail

if [[ $# -lt 3 ]]; then
    echo "usage: tools/golden-runner/stress.sh <runner> <iterations> <entry-id>..." >&2
    exit 2
fi

runner="$1"
iterations="$2"
shift 2

script_dir="$(cd "$(dirname "$0")" && pwd)"
repo_root="$(cd "$script_dir/../.." && pwd)"
corpus="$repo_root/corpus.toml"
rive_runtime="${RIVE_RUNTIME_DIR:-/Users/levi/dev/oss/rive-runtime}"

if [[ ! -x "$runner" ]]; then
    echo "stress: runner not executable: $runner" >&2
    exit 2
fi

# id|path|samples_csv|quarantined|artboard|state_machine per requested entry.
entry_rows="$(python3 - "$corpus" "$@" <<'PY'
import sys, tomllib

corpus_path, *ids = sys.argv[1:]
with open(corpus_path, "rb") as handle:
    corpus = tomllib.load(handle)
by_id = {entry["id"]: entry for entry in corpus["file"]}
for entry_id in ids:
    entry = by_id.get(entry_id)
    if entry is None:
        sys.exit(f"stress: unknown corpus entry: {entry_id}")
    quarantined = any(
        feature.startswith("side-channel-diverges:")
        for feature in entry.get("features", [])
    )
    print("|".join([
        entry_id,
        entry["path"],
        ",".join(repr(sample) for sample in entry["samples"]),
        "1" if quarantined else "0",
        entry.get("artboard", ""),
        entry.get("state_machine", ""),
    ]))
PY
)"

stderr_capture="$(mktemp)"
trap 'rm -f "$stderr_capture"' EXIT

total_failures=0
while IFS='|' read -r entry_id path samples quarantined artboard state_machine; do
    variants=("--side-channel")
    if [[ "$quarantined" == "1" ]]; then
        # Gate shape first (comparator drops --side-channel for quarantined
        # entries), then the side-channel variant for crash coverage.
        variants=("" "--side-channel")
    fi
    for variant in "${variants[@]}"; do
        args=(--file "$rive_runtime/$path" --samples "$samples")
        if [[ -n "$artboard" ]]; then
            args+=(--artboard "$artboard")
        fi
        if [[ -n "$state_machine" ]]; then
            args+=(--state-machine "$state_machine")
        fi
        if [[ -n "$variant" ]]; then
            args+=("$variant")
        fi
        failures=0
        first_failure=""
        for ((i = 1; i <= iterations; i++)); do
            status=0
            "$runner" "${args[@]}" >/dev/null 2>"$stderr_capture" || status=$?
            if [[ "$status" -ne 0 ]]; then
                failures=$((failures + 1))
                if [[ -z "$first_failure" ]]; then
                    first_failure="iteration $i exited $status:
$(cat "$stderr_capture")"
                fi
            fi
        done
        total_failures=$((total_failures + failures))
        printf 'stress: %-28s %-16s %d/%d passed\n' \
            "$entry_id" "${variant:-(gate-shape)}" \
            "$((iterations - failures))" "$iterations"
        if [[ -n "$first_failure" ]]; then
            printf '%s\n' "$first_failure" >&2
        fi
    done
done <<<"$entry_rows"

if [[ "$total_failures" -gt 0 ]]; then
    echo "stress: FAILED ($total_failures crashing runs)" >&2
    exit 1
fi
echo "stress: all entries stable"
