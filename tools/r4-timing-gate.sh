#!/usr/bin/env bash
# Run the R4 timing acceptance trace with pinned renderer-perf executables.
set -euo pipefail

root="$(cd "$(dirname "$0")/.." && pwd -P)"
original_cwd="$(pwd -P)"
sequence=(A B B A)
dry_run=0
output_dir="${R4_TIMING_GATE_OUT_DIR:-}"
renderer_perf="${R4_TIMING_GATE_RENDERER_PERF:-}"
comparator="${R4_TIMING_GATE_COMPARATOR:-}"
manifest="${R4_TIMING_GATE_MANIFEST:-}"
baseline_runner="${R4_TIMING_GATE_BASELINE_RUNNER:-}"
a_runner="${R4_TIMING_GATE_A_RUNNER:-}"
b_runner="${R4_TIMING_GATE_B_RUNNER:-}"
renderer_perf_max_ratio="${R4_TIMING_GATE_RENDERER_PERF_MAX_RATIO:-2.0}"
capture_max_ratio="${R4_TIMING_GATE_CAPTURE_MAX_RATIO:-1000}"
max_b_over_a="${R4_TIMING_GATE_MAX_B_OVER_A:-1.0}"
max_control_drift="${R4_TIMING_GATE_MAX_CONTROL_DRIFT:-1.05}"
max_repeat_drift="${R4_TIMING_GATE_MAX_REPEAT_DRIFT:-1.05}"
min_idle_percent="${R4_TIMING_GATE_MIN_IDLE_PERCENT:-70}"
max_idle_spread_percent="${R4_TIMING_GATE_MAX_IDLE_SPREAD_PERCENT:-12}"
host_sampler="${R4_TIMING_GATE_HOST_SAMPLER:-}"

usage() {
    cat <<'EOF'
usage: r4-timing-gate.sh [--dry-run] [--output-dir path]

Runs the fixed `renderer-perf` executable A-B-B-A with the pinned runner paths
R4_TIMING_GATE_BASELINE_RUNNER, R4_TIMING_GATE_A_RUNNER, and
R4_TIMING_GATE_B_RUNNER. It validates each rive-renderer-perf-v1 report, then
requires both post-tail B reports to meet R4_TIMING_GATE_RENDERER_PERF_MAX_RATIO
and requires B/A candidate timing plus symmetric C++ control drift to meet
their configured limits. R4_TIMING_GATE_CAPTURE_MAX_RATIO is only a permissive
collection ceiling so a slower pre-tail A report cannot abort the bracket.
Every report, stdout/stderr log, host sample, hash, and comparison is retained
in the output directory.

R4_TIMING_GATE_HOST_SAMPLER may name one executable (without arguments) that
emits a `r4-host-idle-percent=<number>` line or a normal `top` CPU line. Its
raw output is retained. Sampling occurs before and after each leg; paired C++
controls inside every report account for load during the timed work without
running a competing monitor.
EOF
}

is_number() {
    awk -v value="$1" 'BEGIN { exit !(value ~ /^[0-9]+([.][0-9]+)?$/) }'
}

is_positive_number() {
    is_number "$1" && awk -v value="$1" 'BEGIN { exit !(value > 0) }'
}

while [[ $# -gt 0 ]]; do
    case "$1" in
        --dry-run) dry_run=1 ;;
        --output-dir)
            [[ $# -ge 2 ]] || { echo "--output-dir requires a path" >&2; exit 2; }
            output_dir="$2"
            shift
            ;;
        --help|-h)
            usage
            exit 0
            ;;
        *)
            echo "unknown argument: $1" >&2
            usage >&2
            exit 2
            ;;
    esac
    shift
done

canonical_new_directory() {
    local path="$1"
    [[ "$path" = /* ]] || path="$original_cwd/$path"
    [[ ! -e "$path" ]] || { echo "refusing to overwrite existing artifact directory: $path" >&2; return 1; }
    local parent
    parent="$(dirname "$path")"
    mkdir -p "$parent"
    parent="$(cd -P "$parent" && pwd)"
    printf '%s/%s' "$parent" "$(basename "$path")"
}

if [[ -z "$output_dir" ]]; then
    output_dir="$root/target/r4-timing-gate/$(date -u +%Y%m%dT%H%M%SZ)-$$"
fi
output_dir="$(canonical_new_directory "$output_dir")" || exit 2

if [[ "$dry_run" == 1 ]]; then
    printf 'r4-timing-gate dry-run sequence=%s-%s-%s-%s\n' "${sequence[@]}"
    printf 'r4-timing-gate dry-run output_dir=%s\n' "$output_dir"
    printf 'r4-timing-gate dry-run baseline=%s A=%s B=%s\n' "$baseline_runner" "$a_runner" "$b_runner"
    exit 0
fi

mkdir "$output_dir"
metadata="$output_dir/metadata.env"
phase="initialization"
failure_reason=""
gate_status="running"
comparison_path="$output_dir/comparison.json"

finalize_metadata() {
    local exit_status=$?
    trap - EXIT
    if [[ "$exit_status" -eq 0 && "$gate_status" == "pass" ]]; then
        printf 'status=pass\n' >>"$metadata"
    else
        [[ -n "$failure_reason" ]] || failure_reason="exit status $exit_status"
        printf 'status=fail\n' >>"$metadata"
        printf 'failure_phase=%q\n' "$phase" >>"$metadata"
        printf 'failure_reason=%q\n' "$failure_reason" >>"$metadata"
    fi
    printf 'utc_finished=%s\n' "$(date -u +%Y-%m-%dT%H:%M:%SZ)" >>"$metadata"
    printf 'artifact_dir=%q\n' "$output_dir" >>"$metadata"
    printf 'comparison_path=%q\n' "$comparison_path" >>"$metadata"
    exit "$exit_status"
}
trap finalize_metadata EXIT

printf 'utc_started=%s\n' "$(date -u +%Y-%m-%dT%H:%M:%SZ)" >"$metadata"
printf 'git_sha=%s\n' "$(git -C "$root" rev-parse HEAD)" >>"$metadata"
printf 'sequence=A-B-B-A\n' >>"$metadata"
printf 'renderer_perf_max_ratio=%s\n' "$renderer_perf_max_ratio" >>"$metadata"
printf 'capture_max_ratio=%s\n' "$capture_max_ratio" >>"$metadata"
printf 'max_b_over_a=%s\n' "$max_b_over_a" >>"$metadata"
printf 'max_control_drift=%s\n' "$max_control_drift" >>"$metadata"
printf 'max_repeat_drift=%s\n' "$max_repeat_drift" >>"$metadata"
printf 'min_idle_percent=%s\n' "$min_idle_percent" >>"$metadata"
printf 'max_idle_spread_percent=%s\n' "$max_idle_spread_percent" >>"$metadata"
printf 'label\tphase\tidle_percent\traw_file\n' >"$output_dir/host-idle.tsv"

fail() {
    phase="$1"
    failure_reason="$2"
    echo "r4-timing-gate $phase: $failure_reason" >&2
    exit "${3:-1}"
}

phase="validate-configuration"
for setting in \
    R4_TIMING_GATE_RENDERER_PERF="$renderer_perf" \
    R4_TIMING_GATE_COMPARATOR="$comparator" \
    R4_TIMING_GATE_MANIFEST="$manifest" \
    R4_TIMING_GATE_BASELINE_RUNNER="$baseline_runner" \
    R4_TIMING_GATE_A_RUNNER="$a_runner" \
    R4_TIMING_GATE_B_RUNNER="$b_runner"; do
    [[ -n "${setting#*=}" ]] || fail "validate-configuration" "${setting%%=*} is required" 2
done
for numeric in \
    R4_TIMING_GATE_RENDERER_PERF_MAX_RATIO="$renderer_perf_max_ratio" \
    R4_TIMING_GATE_CAPTURE_MAX_RATIO="$capture_max_ratio" \
    R4_TIMING_GATE_MAX_B_OVER_A="$max_b_over_a" \
    R4_TIMING_GATE_MAX_CONTROL_DRIFT="$max_control_drift" \
    R4_TIMING_GATE_MAX_REPEAT_DRIFT="$max_repeat_drift" \
    R4_TIMING_GATE_MIN_IDLE_PERCENT="$min_idle_percent" \
    R4_TIMING_GATE_MAX_IDLE_SPREAD_PERCENT="$max_idle_spread_percent"; do
    is_positive_number "${numeric#*=}" \
        || fail "validate-configuration" "${numeric%%=*} must be a positive number" 2
done

canonical_executable() {
    local path="$1"
    [[ -f "$path" && -x "$path" && ! -L "$path" ]] || return 1
    local parent
    parent="$(cd -P "$(dirname "$path")" && pwd)"
    printf '%s/%s' "$parent" "$(basename "$path")"
}

canonical_file() {
    local path="$1"
    [[ -f "$path" && ! -L "$path" ]] || return 1
    local parent
    parent="$(cd -P "$(dirname "$path")" && pwd)"
    printf '%s/%s' "$parent" "$(basename "$path")"
}

hash_file() {
    shasum -a 256 "$1" | awk '{print $1}'
}

pin_executable() {
    local name="$1"
    local configured_path="$2"
    local pinned_path
    pinned_path="$(canonical_executable "$configured_path")" || return 1
    local hash
    hash="$(hash_file "$pinned_path")" || return 1
    printf '%s\t%s\t%s\n' "$name" "$pinned_path" "$hash" >>"$output_dir/runner-pins.tsv"
    printf '%s' "$pinned_path"
}

phase="pin-runners"
printf 'name\tpath\tsha256\n' >"$output_dir/runner-pins.tsv"
renderer_perf="$(pin_executable renderer-perf "$renderer_perf")" || fail "pin-runners" "renderer-perf must be an executable regular file"
comparator="$(pin_executable comparator "$comparator")" || fail "pin-runners" "comparator must be an executable regular file"
baseline_runner="$(pin_executable baseline "$baseline_runner")" || fail "pin-runners" "baseline runner must be an executable regular file"
a_runner="$(pin_executable A "$a_runner")" || fail "pin-runners" "A runner must be an executable regular file"
b_runner="$(pin_executable B "$b_runner")" || fail "pin-runners" "B runner must be an executable regular file"
manifest="$(canonical_file "$manifest")" || fail "pin-runners" "manifest must be a regular file: $manifest"

baseline_hash="$(hash_file "$baseline_runner")"
a_hash="$(hash_file "$a_runner")"
b_hash="$(hash_file "$b_runner")"
if [[ "$a_runner" == "$b_runner" || "$a_hash" == "$b_hash" ]]; then
    fail "pin-runners" "A and B runners must be distinct immutable artifacts"
fi
printf 'manifest=%q\n' "$manifest" >>"$metadata"
printf 'renderer_perf=%q\n' "$renderer_perf" >>"$metadata"
printf 'comparator=%q\n' "$comparator" >>"$metadata"
printf 'baseline_runner=%q\n' "$baseline_runner" >>"$metadata"
printf 'a_runner=%q\n' "$a_runner" >>"$metadata"
printf 'b_runner=%q\n' "$b_runner" >>"$metadata"

if [[ -n "$host_sampler" ]]; then
    host_sampler="$(canonical_executable "$host_sampler")" || fail "pin-host-sampler" "host sampler must be an executable regular file: $host_sampler"
    printf 'host_sampler=%q\n' "$host_sampler" >>"$metadata"
fi

assert_runner_hashes() {
    local current_baseline current_a current_b
    current_baseline="$(hash_file "$baseline_runner")" || fail "verify-runner-hashes" "could not hash baseline runner"
    current_a="$(hash_file "$a_runner")" || fail "verify-runner-hashes" "could not hash A runner"
    current_b="$(hash_file "$b_runner")" || fail "verify-runner-hashes" "could not hash B runner"
    printf 'baseline\t%s\nA\t%s\nB\t%s\n' "$current_baseline" "$current_a" "$current_b" >>"$output_dir/runner-hashes-after.tsv"
    [[ "$current_baseline" == "$baseline_hash" ]] || fail "verify-runner-hashes" "baseline runner changed during the gate"
    [[ "$current_a" == "$a_hash" ]] || fail "verify-runner-hashes" "A runner changed during the gate"
    [[ "$current_b" == "$b_hash" ]] || fail "verify-runner-hashes" "B runner changed during the gate"
}

extract_idle_percent() {
    local raw_file="$1"
    local idle
    idle="$(sed -nE 's/.*r4-host-idle-percent=([0-9]+([.][0-9]+)?).*/\1/p' "$raw_file" | head -n 1)"
    if [[ -z "$idle" ]]; then
        idle="$(sed -nE 's/.* ([0-9]+([.][0-9]+)?)% idle.*/\1/p' "$raw_file" | head -n 1)"
    fi
    printf '%s' "$idle"
}

sample_host() {
    local label="$1"
    local sample_phase="$2"
    local raw_file="$output_dir/${label}.host-${sample_phase}.raw"
    phase="sample-host"
    if [[ -n "$host_sampler" ]]; then
        "$host_sampler" >"$raw_file" 2>&1 || fail "sample-host" "host sampler failed for $label $sample_phase"
    else
        /usr/bin/top -l 1 -n 0 >"$raw_file" 2>&1 || fail "sample-host" "top failed for $label $sample_phase"
    fi
    local idle
    idle="$(extract_idle_percent "$raw_file")"
    is_number "$idle" || fail "sample-host" "could not parse host idle percentage from $raw_file"
    awk -v idle="$idle" -v minimum="$min_idle_percent" 'BEGIN { exit !(idle >= minimum) }' \
        || fail "sample-host" "idle fence failed for $label $sample_phase: ${idle}% < ${min_idle_percent}%"
    printf '%s\t%s\t%s\t%s\n' "$label" "$sample_phase" "$idle" "$(basename "$raw_file")" >>"$output_dir/host-idle.tsv"
}

run_leg() {
    local index="$1"
    local variant="$2"
    local candidate_runner
    case "$variant" in
        A) candidate_runner="$a_runner" ;;
        B) candidate_runner="$b_runner" ;;
        *) fail "run-leg" "unsupported trace variant: $variant" ;;
    esac
    local label json markdown status
    label="$(printf '%02d-%s' "$index" "$variant")"
    json="$output_dir/${label}.renderer-perf.json"
    markdown="$output_dir/${label}.renderer-perf.md"

    sample_host "$label" before
    phase="run-$label"
    set +e
    (
        cd "$root"
        "$renderer_perf" --manifest "$manifest" --baseline-runner "$baseline_runner" \
            --candidate-runner "$candidate_runner" --max-ratio "$capture_max_ratio" \
            --json "$json" --markdown "$markdown"
    ) >"$output_dir/${label}.stdout" 2>"$output_dir/${label}.stderr"
    status=$?
    set -e
    printf '%s\n' "$status" >"$output_dir/${label}.exit-status"
    [[ "$status" -eq 0 ]] || fail "run-$label" "renderer-perf failed; see $output_dir/${label}.stderr" "$status"
    [[ -s "$json" ]] || fail "run-$label" "renderer-perf did not write $json"
    [[ -s "$markdown" ]] || fail "run-$label" "renderer-perf did not write $markdown"
    sample_host "$label" after
    phase="verify-runner-hashes"
    assert_runner_hashes
}

for index in "${!sequence[@]}"; do
    run_leg "$((index + 1))" "${sequence[$index]}"
done

phase="validate-comparison"
if ! "$comparator" --a-first "$output_dir/01-A.renderer-perf.json" \
    --b-first "$output_dir/02-B.renderer-perf.json" \
    --b-second "$output_dir/03-B.renderer-perf.json" \
    --a-second "$output_dir/04-A.renderer-perf.json" \
    --max-renderer-ratio "$renderer_perf_max_ratio" \
    --max-b-over-a "$max_b_over_a" --max-control-drift "$max_control_drift" \
    --max-repeat-drift "$max_repeat_drift" \
    --output "$comparison_path" >"$output_dir/comparator.stdout" 2>"$output_dir/comparator.stderr" \
; then
    comparison_reason="$(tr '\n' ' ' <"$output_dir/comparator.stderr")"
    fail "validate-comparison" "${comparison_reason:-comparison failed; see $output_dir/comparator.stderr}"
fi

idle_spread="$(awk 'NR == 2 { min = $3; max = $3 } NR > 1 { if ($3 < min) min = $3; if ($3 > max) max = $3 } END { if (NR > 1) printf "%.6f", max - min }' "$output_dir/host-idle.tsv")"
awk -v spread="$idle_spread" -v maximum="$max_idle_spread_percent" 'BEGIN { exit !(spread <= maximum) }' \
    || fail "validate-host-load" "idle spread fence failed: ${idle_spread}% > ${max_idle_spread_percent}%"
printf 'idle_spread_percent=%s\n' "$idle_spread" >>"$metadata"

gate_status="pass"
printf 'r4-timing-gate status=pass artifacts=%s idle_spread_percent=%s\n' "$output_dir" "$idle_spread"
