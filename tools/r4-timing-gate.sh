#!/usr/bin/env bash
# Run timing-defined R4 acceptance measurements as a load-fenced A-B-B-A trace.
set -euo pipefail

root="$(cd "$(dirname "$0")/.." && pwd)"
sequence=(A B B A)
dry_run=0
output_dir="${R4_TIMING_GATE_OUT_DIR:-}"
min_idle_percent="${R4_TIMING_GATE_MIN_IDLE_PERCENT:-70}"
max_idle_spread_percent="${R4_TIMING_GATE_MAX_IDLE_SPREAD_PERCENT:-12}"
host_sample_command="${R4_TIMING_GATE_HOST_SAMPLE_CMD:-}"

usage() {
    cat <<'EOF'
usage: r4-timing-gate.sh [--dry-run] [--output-dir path]

Requires R4_TIMING_GATE_A_CMD and R4_TIMING_GATE_B_CMD. Each command must
write the existing renderer-perf JSON and Markdown reports to the paths
provided in R4_TIMING_GATE_REPORT_JSON and R4_TIMING_GATE_REPORT_MARKDOWN.

The gate runs A-B-B-A. Before and after every leg it captures host CPU state,
requires each sample to meet R4_TIMING_GATE_MIN_IDLE_PERCENT (default 70), and
requires the overall idle spread to stay within
R4_TIMING_GATE_MAX_IDLE_SPREAD_PERCENT (default 12). Artifacts are retained in
the output directory. R4_TIMING_GATE_HOST_SAMPLE_CMD may override `top`; it
must emit either a normal `top` CPU usage line or
`r4-host-idle-percent=<number>`.
EOF
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

command_a="${R4_TIMING_GATE_A_CMD:-}"
command_b="${R4_TIMING_GATE_B_CMD:-}"
[[ -n "$command_a" ]] || { echo "R4_TIMING_GATE_A_CMD is required" >&2; exit 2; }
[[ -n "$command_b" ]] || { echo "R4_TIMING_GATE_B_CMD is required" >&2; exit 2; }

is_number() {
    awk -v value="$1" 'BEGIN { exit !(value ~ /^[0-9]+([.][0-9]+)?$/) }'
}

is_number "$min_idle_percent" || { echo "R4_TIMING_GATE_MIN_IDLE_PERCENT must be numeric" >&2; exit 2; }
is_number "$max_idle_spread_percent" || { echo "R4_TIMING_GATE_MAX_IDLE_SPREAD_PERCENT must be numeric" >&2; exit 2; }

if [[ "$dry_run" == 1 ]]; then
    printf 'r4-timing-gate dry-run sequence=%s-%s-%s-%s\n' "${sequence[@]}"
    printf 'r4-timing-gate dry-run min_idle_percent=%s max_idle_spread_percent=%s\n' \
        "$min_idle_percent" "$max_idle_spread_percent"
    printf 'r4-timing-gate dry-run A=%s\n' "$command_a"
    printf 'r4-timing-gate dry-run B=%s\n' "$command_b"
    exit 0
fi

if [[ -z "$output_dir" ]]; then
    output_dir="$root/target/r4-timing-gate/$(date -u +%Y%m%dT%H%M%SZ)-$$"
fi
if [[ -e "$output_dir" ]]; then
    echo "refusing to overwrite existing artifact directory: $output_dir" >&2
    exit 2
fi
mkdir -p "$output_dir"

printf 'utc_started=%s\n' "$(date -u +%Y-%m-%dT%H:%M:%SZ)" >"$output_dir/metadata.env"
printf 'git_sha=%s\n' "$(git -C "$root" rev-parse HEAD)" >>"$output_dir/metadata.env"
printf 'min_idle_percent=%s\n' "$min_idle_percent" >>"$output_dir/metadata.env"
printf 'max_idle_spread_percent=%s\n' "$max_idle_spread_percent" >>"$output_dir/metadata.env"
printf 'sequence=A-B-B-A\n' >>"$output_dir/metadata.env"
printf 'label\tphase\tidle_percent\traw_file\n' >"$output_dir/host-idle.tsv"

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
    local phase="$2"
    local raw_file="$output_dir/${label}.host-${phase}.raw"
    if [[ -n "$host_sample_command" ]]; then
        bash -lc "$host_sample_command" >"$raw_file" 2>&1
    else
        /usr/bin/top -l 1 -n 0 >"$raw_file" 2>&1
    fi

    local idle
    idle="$(extract_idle_percent "$raw_file")"
    if ! is_number "$idle"; then
        echo "could not parse host idle percentage from $raw_file" >&2
        exit 1
    fi
    if ! awk -v idle="$idle" -v minimum="$min_idle_percent" 'BEGIN { exit !(idle >= minimum) }'; then
        echo "host idle fence failed for $label $phase: ${idle}% < ${min_idle_percent}%" >&2
        exit 1
    fi
    printf '%s\t%s\t%s\t%s\n' "$label" "$phase" "$idle" "$(basename "$raw_file")" \
        >>"$output_dir/host-idle.tsv"
}

run_leg() {
    local index="$1"
    local variant="$2"
    local command
    case "$variant" in
        A) command="$command_a" ;;
        B) command="$command_b" ;;
        *) echo "unsupported trace variant: $variant" >&2; exit 1 ;;
    esac

    local label
    label="$(printf '%02d-%s' "$index" "$variant")"
    local json="$output_dir/${label}.renderer-perf.json"
    local markdown="$output_dir/${label}.renderer-perf.md"
    printf '%s\n' "$command" >"$output_dir/${label}.command"

    sample_host "$label" before
    set +e
    (
        cd "$root"
        export R4_TIMING_GATE_REPORT_JSON="$json"
        export R4_TIMING_GATE_REPORT_MARKDOWN="$markdown"
        bash -lc "$command"
    ) >"$output_dir/${label}.stdout" 2>"$output_dir/${label}.stderr"
    local status=$?
    set -e
    printf '%s\n' "$status" >"$output_dir/${label}.exit-status"
    if [[ "$status" -ne 0 ]]; then
        echo "measurement command failed for $label; see $output_dir/${label}.stderr" >&2
        exit "$status"
    fi
    [[ -s "$json" ]] || { echo "measurement command did not write $json" >&2; exit 1; }
    [[ -s "$markdown" ]] || { echo "measurement command did not write $markdown" >&2; exit 1; }
    sample_host "$label" after
}

for index in "${!sequence[@]}"; do
    run_leg "$((index + 1))" "${sequence[$index]}"
done

idle_spread="$(awk 'NR == 2 { min = $3; max = $3 } NR > 1 { if ($3 < min) min = $3; if ($3 > max) max = $3 } END { if (NR > 1) printf "%.6f", max - min }' "$output_dir/host-idle.tsv")"
if ! awk -v spread="$idle_spread" -v maximum="$max_idle_spread_percent" 'BEGIN { exit !(spread <= maximum) }'; then
    echo "host idle spread fence failed: ${idle_spread}% > ${max_idle_spread_percent}%" >&2
    exit 1
fi

printf 'utc_finished=%s\n' "$(date -u +%Y-%m-%dT%H:%M:%SZ)" >>"$output_dir/metadata.env"
printf 'idle_spread_percent=%s\n' "$idle_spread" >>"$output_dir/metadata.env"
printf 'status=pass\n' >>"$output_dir/metadata.env"
printf 'r4-timing-gate status=pass artifacts=%s idle_spread_percent=%s\n' \
    "$output_dir" "$idle_spread"
