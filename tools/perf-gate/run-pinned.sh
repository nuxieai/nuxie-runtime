#!/bin/bash
# Keep both comparator and child runners on one performance-class CPU when the
# host exposes portable affinity controls. macOS does not expose a supported
# per-core affinity API, so Apple Silicon runs retain normal scheduler policy.
set -euo pipefail

if [[ "$(uname -s)" == "Linux" ]] && command -v taskset >/dev/null 2>&1 && command -v lscpu >/dev/null 2>&1; then
    performance_cpu=$(
        lscpu -p=CPU,MAXMHZ 2>/dev/null |
            awk -F, '
                /^#/ { next }
                $1 ~ /^[0-9]+$/ && $2 ~ /^[0-9]+([.][0-9]+)?$/ {
                    if (best == "" || $2 > best) { best = $2; cpu = $1 }
                }
                END { if (cpu != "") print cpu }
            '
    )
    if [[ -n "$performance_cpu" ]]; then
        echo "perf-gate affinity=taskset cpu=$performance_cpu class=highest-max-mhz"
        exec taskset -c "$performance_cpu" "$@"
    fi
fi

echo "perf-gate affinity=unavailable platform=$(uname -s) scheduler=default"
exec "$@"
