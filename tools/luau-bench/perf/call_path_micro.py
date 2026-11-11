#!/usr/bin/env python3
"""Write call-path micro benches as merged luau-bench scripts.

Usage: call_path_micro.py OUT_DIR
Each bench runs N iterations of one loop body between two os.clock reads.
"""
import importlib.util
import sys
from pathlib import Path

repo = Path(__file__).resolve().parents[3]
out = Path(sys.argv[1])
spec = importlib.util.spec_from_file_location("merge", repo / "tests/luau-bench/merge.py")
merge = importlib.util.module_from_spec(spec)
spec.loader.exec_module(merge)
prelude = merge.support_prelude(repo / "tests/luau-bench/upstream")

N = 200000
# name -> (setup, body)
V = {
    "a_empty": ("", ""),
    "b_add": ("local s = 0", "s = s + i"),
    "c_call0": ("local function f() end", "f()"),
    "d_call1": ("local function f(x) return x end", "local y = f(i)"),
    "e_call2": ("local function f(x, y) return y, x end", "local a, b = f(i, i)"),
    "f_method": ("local o = {}\nfunction o:m(x) return x end", "local y = o:m(i)"),
    "g_abs": ("", "local y = math.abs(i)"),
    "h_sin": ("", "local y = math.sin(i)"),
    "i_writef32": ("local b = buffer.create(16)", "buffer.writef32(b, 0, i)"),
    "j_type": ("", "local y = type(i)"),
    "k_selectn": ("", "local y = select('#', i)"),
    "l_min2": ("", "local y = math.min(i, 5)"),
    "m_band": ("", "local y = bit32.band(i, 255)"),
    "n_pcall": ("local function f(x) return x end", "local ok, y = pcall(f, i)"),
    "o_coro": (
        "local co = coroutine.wrap(function() while true do coroutine.yield() end end)",
        "co()",
    ),
    "p_tostring": ("", "local y = tostring(i)"),
    "q_rawget": ("local t = {1, 2, 3}", "local y = rawget(t, 1)"),
    "r_floor": ("", "local y = math.floor(i)"),
    "s_vector": ("", "local y = vector.create(i, i, i)"),
    "t_tinsert": ("local t = table.create(%d)" % N, "table.insert(t, i)"),
    "u_upcall": ("local k = 1\nlocal function f(x) return x + k end", "local y = f(i)"),
    "v_vararg": ("local function f(...) return ... end", "local y = f(i)"),
}

out.mkdir(parents=True, exist_ok=True)
for name, (setup, body) in V.items():
    src = f"""
local bench = require("bench_support")
{setup}
function test()
    local ts0 = os.clock()
    for i = 1, {N} do
        {body}
    end
    local ts1 = os.clock()
    return ts1 - ts0
end
bench.runCode(test, "{name}")
"""
    (out / f"micro__{name}.lua").write_text(prelude + src)
print(f"wrote {len(V)} scripts, N={N}")
