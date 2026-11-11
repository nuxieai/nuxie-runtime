#!/usr/bin/env python3
"""Group the 87 luau-bench benches by the cost that dominates them and print
each group's nuxie/luaur geomean.

usage: groups.py NUXIE.json LUAUR.json

The assignment is by reading each bench's hot loop and, for the benches
profiled in docs/evidence/luau-safe-vm-perf-map-2026-11-11.md, its profile.
One group per bench (its dominant cost), so groups partition the suite.

Measurement only.
"""
import json
import math
import sys

GROUPS = {
    "builtin calls (FASTCALL-able: math, assert, bit32, select, vector, unpack)": [
        "micro_tests__test_AbsSum_abs", "micro_tests__test_AbsSum_math_abs",
        "micro_tests__test_SqrtSum_sqrt", "micro_tests__test_SqrtSum_math_sqrt",
        "micro_tests__test_SqrtSum_sqrt_getfenv", "micro_tests__test_SqrtSum_sqrt_roundabout",
        "micro_tests__test_Assert", "micro_tests__test_VariadicSelect", "micro_tests__test_vector_lib",
        "micro_tests__test_TableMarshal_select", "micro_tests__test_TableMarshal_table_pack",
        "micro_tests__test_TableMarshal_table_unpack_array", "micro_tests__test_TableMarshal_table_unpack_range",
        "micro_tests__test_TableInsertion_table_insert", "micro_tests__test_TableFind_table_find",
        "tests__sha256", "tests__trig", "tests__base64", "tests__pcmmix", "tests__vector-math",
        "tests__mesh-normal-vector",
    ],
    "Lua calls and returns": [
        "micro_tests__test_Factorial", "micro_tests__test_MethodCalls", "micro_tests__test_OOP_method_call",
        "micro_tests__test_Pcall_call_return", "micro_tests__test_Pcall_pcall_return",
        "micro_tests__test_Pcall_xpcall_return", "micro_tests__test_TableMarshal_varargs", "tests__qsort",
    ],
    "table get/set by constant key (fields, methods, constructors)": [
        "micro_tests__test_OOP_field_access", "micro_tests__test_OOP_field_access_random",
        "micro_tests__test_OOP_method_access", "micro_tests__test_OOP_constructor",
        "micro_tests__test_OOP_virtual_constructor", "tests__mesh-normal-scalar", "tests__chess",
    ],
    "array access and iteration": [
        "micro_tests__test_LargeTableSum_loop_index", "micro_tests__test_LargeTableSum_loop_ipairs",
        "micro_tests__test_LargeTableSum_loop_iter", "micro_tests__test_LargeTableSum_loop_pairs",
        "micro_tests__test_TableIteration", "micro_tests__test_TableInsertion_index_len",
        "micro_tests__test_TableInsertion_index_cached", "micro_tests__test_TableFind_loop_ipairs",
        "micro_tests__test_LargeTableCtor_array", "micro_tests__test_LargeTableCtor_hash",
        "tests__tictactoe", "tests__life", "tests__matrixmult", "tests__voxelgen",
    ],
    "arithmetic and loops": [
        "micro_tests__test_AbsSum_and_or", "micro_tests__test_SqrtSum_exponent",
    ],
    "string building, interning and number formatting": [
        "micro_tests__test_StringInterp/interp: %d format", "micro_tests__test_StringInterp/interp: %f format",
        "micro_tests__test_StringInterp/interp: %* format", "micro_tests__test_StringInterp/interp: %s format",
        "micro_tests__test_StringInterp/interp: concat", "micro_tests__test_StringInterp/interp: interp number",
        "micro_tests__test_StringInterp/interp: interp string", "micro_tests__test_StringInterp/interp: tostring",
        "micro_tests__test_ToNumberString/tonumber", "micro_tests__test_ToNumberString/tostring",
        "micro_tests__test_ToNumberString/tostring-gc",
        "micro_tests__test_string_lib/string: rep (small)", "micro_tests__test_string_lib/string: rep (large)",
        "micro_tests__test_string_lib/string: reverse/upper/lower (unique)",
        "micro_tests__test_string_lib/string: reverse/upper/lower (large)",
        "micro_tests__test_table_concat/table: concat (small)", "micro_tests__test_table_concat/table: concat (big)",
        "micro_tests__test_TableSort/table.sort: 12 strings",
    ],
    "closures and upvalues": ["micro_tests__test_UpvalueCapture"],
    "coroutines": ["tests__sieve", "micro_tests__test_Pcall_pcall_yield"],
    "table library bulk ops (move, insert/remove at index, create, sort)": [
        "micro_tests__test_TableMove_empty_table", "micro_tests__test_TableMove_same_table",
        "micro_tests__test_TableMove_table_create", "micro_tests__test_TableRemoval_table_remove",
        "micro_tests__test_TableInsertion_table_insert_index", "micro_tests__test_TableCreate_nil",
        "micro_tests__test_TableCreate_number", "micro_tests__test_TableCreate_zerofill",
        "micro_tests__test_TableSort/table.sort: 10k numbers", "micro_tests__test_TableSort/table.sort: 100 numbers",
    ],
    "error path (pcall/xpcall of a failing call)": [
        "micro_tests__test_Failure_pcall_a_bar", "micro_tests__test_Failure_pcall_game_Foo",
        "micro_tests__test_Failure_xpcall_a_bar", "micro_tests__test_Failure_xpcall_game_Foo",
    ],
}


def load(path):
    doc = json.load(open(path))
    return {b: min(e["pass_medians_ms"]) for b, e in doc["benches"].items() if e.get("status") == "ok"}


nuxie, luaur = load(sys.argv[1]), load(sys.argv[2])
assigned = [b for members in GROUPS.values() for b in members]
missing = sorted(set(nuxie) - set(assigned))
extra = sorted(set(assigned) - set(nuxie))
dupes = sorted({b for b in assigned if assigned.count(b) > 1})
if missing or extra or dupes:
    sys.exit(f"grouping out of date: unassigned {missing}, unknown {extra}, twice {dupes}")
print("| group | benches | geomean nuxie/luaur | range | share of suite log-gap |")
print("| --- | ---: | ---: | --- | ---: |")
total_log = sum(math.log(nuxie[b] / luaur[b]) for b in nuxie)
for name, members in GROUPS.items():
    ratios = [nuxie[b] / luaur[b] for b in members]
    logs = sum(math.log(r) for r in ratios)
    print(f"| {name} | {len(members)} | {math.exp(logs / len(ratios)):.2f}x | "
          f"{min(ratios):.2f}x to {max(ratios):.2f}x | {logs / total_log * 100:.0f}% |")
