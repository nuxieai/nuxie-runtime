#!/bin/sh
# Timing measurements are only meaningful on a quiet machine. Concurrent
# local agent workloads and sibling landing batteries have produced 6-8x inflated
# ratios and false-red gates (observed 2026-08-04: script_create_text_runs
# measured 475x under contention vs 332x quiet). Wait, bounded, for the
# load to drop before benchmarking; if it never does, proceed with a loud
# CONTENTION marker so a red gate is diagnosable as noise, not regression.
set -eu

max_wait=${PERF_GATE_QUIET_MAX_WAIT:-900}
cores=$(sysctl -n hw.ncpu 2>/dev/null || nproc)
# Quiet threshold: 1-minute loadavg below half the cores.
threshold=$((cores / 2))
waited=0
interval=15

while :; do
    load=$(sysctl -n vm.loadavg 2>/dev/null | awk '{print $2}' | cut -d. -f1)
    [ -z "$load" ] && load=$(uptime | awk -F'load averages?: ' '{print $2}' | awk '{print $1}' | cut -d. -f1)
    if [ "$load" -lt "$threshold" ]; then
        [ "$waited" -gt 0 ] && echo "perf-gate quiet after ${waited}s (load=$load threshold=$threshold)"
        exit 0
    fi
    if [ "$waited" -ge "$max_wait" ]; then
        echo "perf-gate CONTENTION: proceeding after ${max_wait}s wait with load=$load >= $threshold — treat ceiling failures below as suspect noise, re-run when quiet" >&2
        exit 0
    fi
    if [ "$waited" -eq 0 ]; then
        echo "perf-gate waiting for quiet machine (load=$load threshold=$threshold, max ${max_wait}s)..."
    fi
    sleep "$interval"
    waited=$((waited + interval))
done
