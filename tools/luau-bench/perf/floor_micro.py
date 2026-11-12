#!/usr/bin/env python3
"""Write per-bytecode floor micro benches as merged luau-bench scripts.

Usage: floor_micro.py OUT_DIR
Each bench runs N iterations of one loop body, with all state in locals of
the timed function (docs/luau-safe-vm-register-window.md, section 2).
"""
import importlib.util, sys
from pathlib import Path
repo, out = Path(__file__).resolve().parents[3], Path(sys.argv[1])
spec = importlib.util.spec_from_file_location("merge", repo / "tests/luau-bench/merge.py")
merge = importlib.util.module_from_spec(spec); spec.loader.exec_module(merge)
prelude = merge.support_prelude(repo / "tests/luau-bench/upstream")
N = 200000
V = {
    "a_empty": ("", ""),
    "b_addlocal": ("local s = 0", "s = s + i"),
    "c_add3": ("local s, t, u = 0, 0, 0", "s = s + i; t = t + s; u = u + t"),
    "d_move": ("local a, b = 1, 2", "a, b = b, a"),
    "e_iflt": ("local s = 0", "if i < 0 then s = i end"),
    "f_mulk": ("local s = 0", "s = i * 2"),
    "g_tget": ("local t = {1, 2, 3}\n    local s = 0", "s = t[1]"),
    "h_tks": ("local t = {x = 1}\n    local s = 0", "s = t.x"),
    "i_tset": ("local t = {x = 1}", "t.x = i"),
    "j_tidx": ("local t = table.create(8, 0)\n    local s = 0", "s = t[(i % 8) + 1]"),
}
out.mkdir(parents=True, exist_ok=True)
for name, (setup, body) in V.items():
    src = f"""
local bench = require("bench_support")
function test()
    {setup}
    local ts0 = os.clock()
    for i = 1, {N} do
        {body}
    end
    local ts1 = os.clock()
    return ts1 - ts0
end
bench.runCode(test, "{name}")
"""
    (out / f"floor__{name}.lua").write_text(prelude + src)
print(len(V))
