# Safe Luau VM: the call path (lane 9 design)

> 2026-11-13: the vendored VM this document cites (`vendor/luaur-rt-0.1.8`,
> `vendor/luaur-vm-0.1.8`) has been deleted
> (`docs/evidence/luau-safe-vm-phase3-deletion-2026-11-13.md`). Paths into
> it name files at commit `e25de6390c`; the safe VM's behavior it describes
> is pinned by the differential snapshots in `tests/luau-vm-diff/`.

Status: **as built** in lane 9 (2026-11-12, branch
`claude/luau-vm-lane9-call-path-build-0d02f0`; evidence
`docs/evidence/luau-safe-vm-phase2-lane9-2026-11-12.md`, which has the
measured row for every prediction in section 6). Built as written except
where marked "as built" below: 4.5 is withdrawn and 4.2's `safeenv` cache
was not built (the no-caches rule, 4.2), the frame is 32 bytes, and a
thread switch swaps once per side. Written by the coordinator on
`origin/main` `64d495673b` (lane 8 landed). It replaces the call-path parts of
`docs/luau-safe-vm-embedding-core.md` section 2.1 (the reserved native frame)
and builds on its `Core`/`HostedVm<H>` split, which stays.

Lane 8's predictions failed because they were made from this VM's own cost
breakdown without reading how the vendored VM runs the same path. Every
section here starts from what the reference does (`vendor/luaur-vm-0.1.8/
src/functions/luau_execute.rs`, cited by line) and from counts measured on
both VMs.

## 1. Measurements this design starts from

`tools/luau-bench/perf/call_path_micro.py` writes 22 one-line loops
(200,000 iterations between two `os.clock` reads). Run on both VMs with
`LUAU_BENCH_CLOCK=instructions`, Bazel release fat-LTO, 2 passes, 1-minute
load 15 to 19 (instruction counts do not move with load). Retired
instructions per iteration, with the empty loop subtracted:

| Loop body | safe VM | luaur | ratio | What luaur executes |
|---|---:|---:|---:|---|
| (empty loop, not subtracted) | 93 | 38 | 2.4 | `FORNLOOP` |
| `s = s + i` (`s` an upvalue) | 262 | 81 | 3.2 | `GETUPVAL`, `ADD`, `SETUPVAL` |
| `f()` | 778 | 146 | 5.3 | `GETUPVAL`, `CALL`, `RETURN` |
| `local y = f(i)` | 866 | 169 | 5.1 | + `MOVE` |
| `local a, b = f(i, i)` | 1,085 | 227 | 4.8 | |
| `local y = o:m(i)` | 1,059 | 215 | 4.9 | `NAMECALL`, `CALL`, `RETURN` |
| `local y = f(i)`, `f` reads an upvalue | 949 | 199 | 4.8 | |
| `local y = f(i)`, `f` is `(...)` | 1,003 | 244 | 4.1 | |
| `math.abs(i)` | 571 | 68 | 8.4 | `FASTCALL1` only |
| `math.floor(i)` | 570 | 68 | 8.4 | `FASTCALL1` only |
| `math.sin(i)` | 623 | 131 | 4.8 | `FASTCALL1` only |
| `type(i)` | 689 | 70 | 9.8 | `FASTCALL1` only |
| `math.min(i, 5)` | 705 | 79 | 8.9 | `FASTCALL2K` only |
| `bit32.band(i, 255)` | 720 | 81 | 8.9 | `FASTCALL2K` only |
| `vector.create(i, i, i)` | 803 | 92 | 8.7 | `FASTCALL3` only |
| `buffer.writef32(b, 0, i)` | 728 | 133 | 5.5 | `LOADN`, `FASTCALL3` |
| `rawget(t, 1)` | 681 | 127 | 5.4 | `LOADN`, `FASTCALL2` |
| `table.insert(t, i)` | 1,052 | 176 | 6.0 | `FASTCALL2` |
| `select('#', i)` | 569 | 238 | 2.4 | a real C call on both |
| `pcall(f, i)` | 2,413 | 742 | 3.3 | |
| `co()` (`coroutine.wrap`, one resume and one yield) | 3,564 | 566 | 6.3 | |
| `tostring(i)` | 13,954 | 622 | 22.4 | number formatting, not the call |

Two readings matter for everything below.

**One bytecode instruction costs this VM 45 to 90 instructions** (`FORNLOOP`
93, an upvalue access about 87). luaur pays 27 to 38. That floor is register
access through `&mut self`, the per-instruction `self.pc` store and the
dispatch (perf map section 3.4). It is not this lane's subject, and it bounds
what this lane can reach: a loop body of five bytecodes cannot cost less
than about 250 here whatever a call costs.

**The call protocol is the rest.** `local y = f(i)` is four bytecodes
(`GETUPVAL`, `MOVE`, `CALL`, `RETURN`), about 180 of floor, so `CALL` plus
`RETURN` cost about 690. luaur's whole iteration is 169.

## 2. What the reference does

**`FASTCALL*`** (`luau_execute.rs:3749`, `3896`, and the 2/2K/3 forms). The
compiler emits `FASTCALL` before the instructions that load the function
and before the `CALL`. The handler reads the `CALL` at `pc + skip` for the
result register and count, and if the closure's environment is `safeenv` it
calls `luauF_table[id](L, res, arg0, nresults, args, nparams)`. A
non-negative return is the result count: `pc` skips past the `CALL`. A
negative return, or an unsafe environment, falls through to the ordinary
instructions. There is no frame, no argument window, no result move and no
collector check. The table has 145 entries naming 133 bodies
(`macros/luau_f_table.rs`). A fast body never raises except on allocation
failure; on any input it does not handle it returns -1 and the `CALL`
raises the library function's error.

**`CALL`** (`luau_execute.rs:1169`). Interrupt poll; resolve `__call` if the
callee is not a function; store the caller's `savedpc`; `incr_ci`; write
six `CallInfo` fields (`func`, `base`, `top`, `savedpc`, `flags`,
`nresults`: 40 bytes); one stack check. For a Lua callee: nil the missing
parameters only, set `pc`, `cl`, `base`, `k` in locals, and keep
dispatching. For a C callee: call it, copy `n` results down to `func`, nil
up to `nresults`, pop the `CallInfo`.

**`RETURN`** (`luau_execute.rs:1448`). Interrupt poll; copy results to
`ci->func`, nil up to `nresults`; pop; one flag test
(`LUA_CALLINFO_RETURN`) decides whether to leave the loop; otherwise reload
`cl`, `pc`, `k` from the caller's `CallInfo` and keep dispatching.

**A coroutine switch** changes the `lua_State` pointer the loop runs on.
Nothing is copied.

## 3. What this VM does today

- **Frame** (`vm.rs:117`): 128 bytes, not `Copy`, with drop glue. It holds
  `wait: Wait` (metamethod `Finish`, `Cont` with a `Box<dyn Continuation>`,
  `Outcome`), `restore: Option<Saved>`, `nargs`, two bools.
- **`CALL` of a Lua function** (`interp.rs:753`, `call.rs:431-539`):
  `push_call` (save counters, mode match, `resolve_callable`), two
  `heap.function` lookups, `enter_lua` (depth check, proto lookup,
  `save_caller_pc`, `ensure_stack`, two `fill_nil`, a 128-byte push with a
  reserved-frame branch), then `continue 'frame`, which re-reads the top
  frame, compares the proto table `Arc`, and reloads code and constants.
- **`RETURN`** (`interp.rs:788`): `close_upvals`, pop (128 bytes moved),
  `move_results` (`ensure_stack`, per-slot checked copy, `fill_nil`),
  `deliver` (restore, two flags, `mem::take` of the caller's `Wait` and its
  drop), `continue 'frame`.
- **`CALL` of a plain native** (`call.rs:577`): the frameless path. It still
  checks depth, saves the caller pc, grows the stack, writes six fields of
  the reserved frame, runs the native over an argument window, checks the
  request slot, moves results through `move_results`, runs `safepoint`.
  About 430 instructions before the native's own work.
- **The reserved frame** (`frames.rs`): one 128-byte `Frame` inline in every
  thread's `Frames`, and a branch on `native_active` in `len`, `last`,
  `last_mut`, `get`, `get_mut`, `push`, `pop`, `truncate`. Lua calls pay
  the branches; a coroutine switch copies the frame four times.
- **`FASTCALL*`** (`interp.rs:415`): runs four bodies (`libs/fast.rs`), the
  ones whose result differs from the library function. Every other builtin
  falls through to `CALL`.
- **Thread switch** (`thread.rs:165`): eleven field swaps, twice.
- **Library continuations**: eleven `impl Continuation` types, boxed on
  every `pcall`, `coroutine.resume`, `tostring` with `__tostring`, and so
  on.

## 4. The design

Five changes. Each removes work instead of making it cheaper, and each
deletes code.

### 4.1 A frameless native leaves no record until something looks

Today the reserved frame is written on every frameless call so that an
error or `debug.info` can see the native. The simpler shape: write nothing,
and build the frame when the first observer asks.

- `Core` gets one field, `frameless: u32`: the callee's stack slot plus
  one while a plain native runs from `CALL`, zero otherwise. `CALL` stores
  it before the native and tests it after: two instructions.
- **Materialize on observation.** The frame list becomes private to the
  call protocol (`call.rs`, `interp.rs`, `thread.rs`, `frames.rs`, `gc.rs`).
  Every other reader in the crate (`vm.rs` error positions, `libs/base.rs`
  `getfenv`/`setfenv`, `libs/debug.rs`, `libs/mod.rs` argument-error names,
  `hosted.rs`, `embed/`) reaches frames only through one accessor,
  `Core::frames_observed(&mut self)`, which first turns a pending
  `frameless` into a real pushed native frame (callee slot, closure read
  from that slot, caller `savedpc` set from `self.pc`) and zeroes the field.
  The compiler enforces it: the field is not visible outside the protocol
  modules.
- **After the native returns**, `CALL` tests `frameless`. Still set: clear
  it and take results in place (the hot path). Zero: an observer pushed a
  frame, so pop it through the ordinary native-return path and re-enter
  the frame loop (cold; this is also what reloads cached frame state after
  `getfenv`/`setfenv`).
- **On error**, the catcher's first step is `frames_observed`, so an error
  that never looked at frames (a memory error) still unwinds through a real
  native frame.

Why this keeps every observable: an observer never sees a frameless
native. It sees exactly the frame a framed native has today, so error text
under the Rive `luaL_where` rule, argument-error names, tracebacks,
`debug.info` levels and `getfenv` levels are computed by the same code on
the same frame shape as on `main`. The collector needs no frame: the
native's closure is in the callee slot and its arguments and pushes are
below `self.top`, both already roots (`gc.rs:224`).

Rejected:
- *Classify level-sensitive builtins as framed, by list or by type.* It
  needs a list someone must keep right (`error`, `debug.info`,
  `debug.traceback`, `getfenv`, `setfenv`, every `lib_error` caller).
  Materialize-on-observation needs no list.
- *Decode the native from the `CALL` at the saved pc on the error path
  with no field at all.* An error constructor cannot tell a frameless
  native from an instruction of the Lua frame itself; one `u32` makes that
  explicit.

Deleted: `Frames::native`, `native_active`, `enter_native`, `leave_native`,
`materialize`, and the branch in each of the eight frame views.

### 4.2 Fast bodies for every builtin the reference has one for

A lighter call cannot reach `FASTCALL`: the reference does not call. This
VM runs the same table.

- **A static table** `FAST: [Option<FastEntry>; 256]`, indexed by the
  builtin id the compiler already wrote into the instruction.
  `FastEntry { f: FastFn, allocates: bool }`.
- **One signature**:
  `fn(&mut Core, ra: usize, nresults: i32, args: [Value; 3], nparams: usize) -> Option<usize>`.
  The handler copies up to three arguments out of registers or constants
  (three 16-byte `Copy` values; this is what makes the body free of
  borrows on the stack). A body that takes more (`math.min/max`,
  `bit32.band/bor/bxor/btest`, `select` on varargs) reads the rest from
  `ra + 4 ..` itself, as the reference reads `args`. It writes its results
  to `ra ..` and returns their count; `None` falls through to the `CALL`.
- **The handler**: read the `CALL` at `pc + skip`; if the running
  closure's environment has `safeenv` and the table has an entry, run it.
  `Some(n)`: set `top` for `MULTRET`, set `pc` past the `CALL`; if the
  entry `allocates`, run the safepoint (4.5). No frame, no window, no depth check, no result
  move.
- **Every body is ported from its `luau_f_*`**, including exactly when it
  bails, so the fallback raises the library function's error as on the
  reference. All 133 are ported, the `integer*` ones included (this VM has
  the integer type; whether the product's compiler settings emit those
  ids was not checked); the SSE4.1 variants are the same body here.
- **One implementation where the results agree.** Each fast body lives
  beside its library function in `libs/` and both call one shared
  computation. Where the reference's two bodies differ (`math.modf`,
  `math.ldexp`, `vector.magnitude`, `vector.dot`: `libs/fast.rs` today),
  both stay, as now.
- **`safeenv` is read where it lives** (amended 2026-11-12, by the rule
  below). Each `FASTCALL*` handler reads the running closure's environment
  table's `safeenv` flag directly, as `main` does. Measured in the
  `FASTCALL1` handler (release disassembly, one traced iteration of
  `math.abs(i)`): **20 instructions**: the function-arena lookup for the
  closure 7 (bounds check, `umaddl`, the closure id and the fallback
  pointer reloaded from spill slots), the Lua/native kind select for the
  environment field 4, the environment id 1, the table-arena lookup 6, the
  flag byte and its test 2. The first text cached the flag per frame
  activation against a `Core::deopt` counter (a 5-instruction compare);
  that is a derived copy needing invalidation, so it is not built.

**No caches (Levi, 2026-11-12).** A cache, a derived copy of state that
can go stale and needs invalidation, is a patch over a slow design; this
lane adds none. Where the first text called for one (`safeenv` here, the
safepoint `trigger` in 4.5), the build reads the source of truth, and the
cost of that read is measured and reported rather than shortcut. The
running frame's `base`, `code`, `k` and `closure` in the loop's locals are
not caches: they are the loop's own state, written on every call and
return, and stay.

Rejected:
- *Typed tiers* (`fn(f64) -> f64` for pure unary math, and so on): three
  or four shapes and their handlers to save roughly ten instructions on a
  body. Revisit with the register window, where it would be free.
- *One big `match` on the id inlined into the handlers*: `run_frames`
  already has a 4.4 KB frame and spills its hot locals (perf map 3.4);
  five copies of a 133-arm match make that worse. A table of function
  pointers is the reference's shape and keeps each body with its library.
- *Making the library function the fast path* (perf map 4.1): the bodies
  differ in when they bail and, for four builtins, in results. The
  differential corpus pins both.

Deleted: `libs/fast.rs` (`has_fast_body`, `run`); its four bodies move
beside their library functions as table entries.

### 4.3 A frame is 28 bytes of plain data; calls and returns stay in the loop

```rust
#[derive(Clone, Copy)]
struct Frame {
    func: u32,      // callee slot; results land here
    base: u32,
    top: u32,
    aux: u32,       // Lua: savedpc. Native: nargs.
    closure: FnId,  // u32
    proto: ProtoId, // u32, meaningful when LUA is set
    nresults: i16,
    flags: u8,      // LUA | BOUNDARY | THREAD_BASE | PENDING
}
```

- **Everything a plain Lua-to-Lua call never uses moves to a side stack.**
  `pending: Vec<Pending>` per thread holds, for the frames that need it,
  what `Frame.wait` and `Frame.restore` hold today: a metamethod `Finish`,
  a native's continuation and its saved C-call counters, a yield state. A
  frame with an entry has `PENDING` set; popping such a frame pops its
  entry. Entries are pushed by `push_meta`, by a native's request, and by
  a yield: never by `CALL` of a Lua function from Lua.
- **Library continuations become one enum**, `LibCont`, with small states
  inline and the three large ones (`gsub`, `sort`, `format`) boxed. `pcall`
  and `coroutine.resume` stop allocating. The `Continuation` trait and its
  eleven `Box<dyn>` impls go; `resume` becomes a `match`.
- **`CALL` of a Lua function, in the handler**: callee tag; one function
  lookup; one proto lookup (parameter count, stack size, vararg flag,
  code, constants); depth compare; one stack-size compare; nil the frame's
  registers above the arguments with one slice `fill`; store the caller's
  `savedpc`; push 28 bytes; set the running frame's locals (`base`, `top`,
  `code`, `k`, `closure`; `safeenv` is not among them, 4.2) and dispatch
  the callee's first instruction. No `push_call`, no second lookup, no outer-loop re-entry.
  `__call`, varargs and stack growth are one cold call each.
- **`RETURN`, in the handler**: open-upvalue check (one compare when none
  are open); pop 28 bytes; copy results to `func ..` and nil up to
  `nresults` as two slice operations with no stack growth (the
  destination is inside the caller's frame, which the verifier bounds);
  then one test: callee `flags == LUA` and caller without `PENDING`.
  True: reload the caller's locals and dispatch. False: the existing
  `deliver` logic, cold.
- **`CALL` of a plain native** (not fast, or the fallback): depth compare,
  stack compare, set `top`, store `frameless`, call, test `frameless`,
  move `n` results down in place, nil-fill, restore `top`, safepoint
  trigger. No request-slot check: a plain native has no way to request
  (lane 8 made that a type property), so the check and its error go.
- **Slots are `u32`.** `Limits::max_stack_slots` is capped at `u32::MAX`
  when the VM is built.

As built: the frame is **32 bytes**, not 28 (`nresults` stayed `i32`;
`Copy`, no drop glue and `<= 32` are asserted at compile time). Before an
inline call or return the handler also checks that callee and caller
share the prototype list (`Arc::ptr_eq`, lane 8's prototypes by index), 4
to 5 instructions each way.

**Kept on purpose: the callee's registers are nil on entry.** The
reference nils only missing parameters and lets stale values sit above
them. Here a stale slot can hold the id of an object the incremental
sweep has since freed and reused, and verified-but-hostile bytecode could
read it and reach another script's object in the same VM. The verifier
does not prove definition before use. One `fill` over at most 255 slots is
the price of that guarantee; proving it at load instead is a verifier
change for a later lane.

Rejected:
- *Keep one `Frame` type and shrink `Wait`.* The drop glue and the take on
  every return remain; the side stack removes them from the path
  entirely.
- *Thread state addressed by index* (perf map 4.7): every register access
  would go through the thread arena until the register window exists.

Deleted: `Frame.wait`, `restore`, `nargs`, `boundary`, `thread_base` as
fields; `Wait`'s place in the frame; the `Continuation` trait;
`move_results`' stack growth; `Flow::Inline`; `push_call` and `deliver`
from the Lua-to-Lua path (they remain for natives, metamethods and the
host).

### 4.4 A thread switch is one swap

The eleven swapped fields become one `Exec` struct held inline in `Core`
(`self.exec.stack`, same access cost as today's `self.stack`). A switch
is `mem::swap(&mut self.exec, &mut thread.exec)` once per side: about 130
bytes with small frames, against eleven swaps plus two 128-byte reserved
frames today.

As built: `Exec` is 136 bytes (17 words); a resume or a yield swaps it out
and the target's in, so a `co()` round trip is four swaps of 83
instructions each with the thread lookup. Section 6's `co()` row says "one
swap each"; this paragraph's "once per side" is what was built.

### 4.5 The safepoint: withdrawn (2026-11-12)

The first text gave `Gc` a `trigger: usize`, the smaller of the step
threshold and the memory-pressure point, recomputed where either changes,
so the safepoint's fast path would be one compare. It was built (stage 2)
and reverted: the trigger is a derived copy of two inputs that every
threshold write and `set_memory_limit` must keep current, which the
no-caches rule (4.2) excludes. The other one-number shape, folding the
pressure point into the threshold itself, changes the collector's pacing
and what `collectgarbage("step")` reports to save about ten instructions;
rejected.

As built: `main`'s shape. `safepoint()` reads the bytes in use, evaluates
the pressure rule directly (a limit is set, the collector is running,
`used > live + (limit - live) / 2`), then compares the step threshold.
`needs_gc()`, which fast bodies that make a string test before
allocating, is the same rule without the work. Measured fast path (release
disassembly at the plain-native call site): **8 instructions with no
memory limit** (three loads, the limit-or-stopped test, the threshold
compare), **16 with a limit** (the pressure point is two more loads, a
saturating subtract, a halve-and-add and a compare), against about 4 for
the one compare. Fast bodies still run it only when their entry
`allocates`.

### 4.6 The host as the loop sees it: unchanged, with one experiment

Lane 8 measured 3% to 5% on calls and thread switches at the commit that
introduced `&mut dyn Dispatch`, the wider exit type and the driver loop
together. It did not attribute the cost to the `dyn`. This design changes
the exits (4.3), so the attribution would be stale anyway.

Decision: keep `&mut dyn Dispatch` and one compiled interpreter. A
generic `run_frames<D>` compiles the largest function in the crate once
per host type, and nothing shows that the `dyn` is what costs.

Experiment (stage 5 below, one scratch commit, not landed): make
`run_frames` and `call_frameless` generic over `D: Dispatch` and take the
micro counts and binary size both ways. The result goes in the evidence
doc and decides nothing by itself; if generic moves call benches by more
than the 3% codegen noise floor, the choice comes back to the
coordinator.

As built: scratch commit `ba330ef24b` (branch
`scratch/luau-vm-lane9-generic-host`). Generic made the call benches 4%
to 5% dearer (the empty loop itself 82 to 90) and host calls within 1%;
binary size flat with one host type. Over the floor, in the wrong
direction: the evidence's section 6 has the numbers, and `dyn` stays.

## 5. Out of scope, and why

- **The host-boundary type.** Lane 8's review found five entry points
  that reached no safepoint or drain. The structural fix is one guard
  type through which all host-side allocation goes and whose `Drop` runs
  the safepoint and the host's drain. It belongs to the next lane, which
  rewrites the host API anyway (collapsing the lane 7 facade and
  `Compat`); building it now means building it twice. The five sites stay
  patched as they are on `main`.
- **The register window, `self.pc` per instruction, dispatch.** The 45 to
  90 instruction floor. This design keeps the running frame's state in
  the loop's locals, which is what the window will need.
- **Tables, strings, `pairs` order.** `table.insert` and `tostring` get
  fast bodies here but stay slow inside (store accounting, number
  formatting).

## 6. Predictions

Per iteration over the empty loop, same micro benches as section 1. Each
range is built from the floor measured in section 1 plus a step count read
from the design; step counts come from source, not disassembly, so each
range is wide. A result outside its range means the reasoning is wrong:
find which step, write it down, and bring it back before changing code.

| Loop body | now | luaur | predicted | Reasoning |
|---|---:|---:|---:|---|
| `math.abs(i)`, `math.floor(i)` | 571 | 68 | 65 to 100 | one bytecode: dispatch, read the `CALL` word, the `safeenv` read, three-value copy, pointer call, tag test, write. Amended 2026-11-12: every fast-call row rises by the measured direct `safeenv` read (20, 4.2); the first range (45 to 80) priced a counter compare |
| `type(i)` | 689 | 70 | 70 to 110 | same, plus a type-name lookup (+20 for `safeenv`, was 50 to 90) |
| `math.min(i, 5)`, `bit32.band(i, 255)` | 705, 720 | 79, 81 | 75 to 120 | `FASTCALL2K` (+20, was 55 to 100) |
| `math.sin(i)` | 623 | 131 | 120 to 170 | the `abs` path plus libm (63 on luaur) (+20, was 100 to 150) |
| `vector.create(i, i, i)` | 803 | 92 | 90 to 140 | `FASTCALL3` (+20, was 70 to 120) |
| `buffer.writef32(b, 0, i)` | 728 | 133 | 130 to 190 | `LOADN` at the floor, then buffer lookup and bounds (+20, was 110 to 170) |
| `rawget(t, 1)` | 681 | 127 | 130 to 210 | `LOADN`, then a raw table read (+20, was 110 to 190) |
| `table.insert(t, i)` | 1,052 | 176 | 274 to 474 | the call goes; the store's accounting stays (+20 for `safeenv`; +4 for the safepoint as built, 4.5, the entry `allocates`; was 250 to 450) |
| `select('#', i)` | 569 | 238 | 324 to 424 | not fast on either VM: `GETIMPORT`, `LOADK`, `MOVE` at the floor (198), the plain-native protocol as designed (about 90) and `select`'s body (about 70). Amended in lane 9: the first range (150 to 250) priced the call alone and left out the three bytecodes before it. Amended 2026-11-12: +4 for the safepoint as built (4.5; 8 against the one compare's 4, no memory limit in the bench) |
| `f()` | 778 | 146 | 200 to 300 | `GETUPVAL` at the floor (87) plus about 100 for `CALL` and 75 for `RETURN` |
| `local y = f(i)` | 866 | 169 | 250 to 360 | plus `MOVE` and one result |
| `local a, b = f(i, i)` | 1,085 | 227 | 330 to 460 | |
| `o:m(i)` | 1,059 | 215 | 400 to 560 | `NAMECALL`'s table lookup is unchanged |
| `pcall(f, i)` | 2,413 | 742 | 800 to 1,200 | two frames and one pending entry, no allocation, no `Wait` drop |
| `co()` | 3,564 | 566 | 900 to 1,500 | two switches at one swap each, no boxed continuation, small frames; lowest confidence |
| `s = s + i`, `tostring(i)` | 262, 13,954 | 81, 622 | unchanged | not call cost |

Derived:

- **Audio block**: about 11 bytecodes a frame at the floor (about 600),
  `FORNLOOP` (93), and three fast bodies (about 130, plus 60 for three
  direct `safeenv` reads, amended 2026-11-12): **810 to 1,010
  instructions a frame**, against 2,428 now and about 540 on luaur.
- **Plain host call**: the plain-native path plus the `dyn` call, entry
  lookup and context: **134 to 224** (amended 2026-11-12: +4 for the
  safepoint as built, 4.5; +12 under a memory limit), against 539 now.
  This is the number the next lane's bindings will be written against.
- **Suite groups** (lane 8 evidence 6.1). Builtin calls 4.33x toward 2.5x
  to 3.2x: `AbsSum_math_abs` is `GETUPVAL`, `FASTCALL1`, `ADD`, `SETUPVAL`
  and the loop, about 435 against luaur's 138, and the fast call is 80 of
  it (60 before the direct `safeenv` read, 4.2). Lua calls 3.87x toward
  2.2x to 2.8x. Coroutines 4.35x toward 2.5x. What remains in each is the per-bytecode floor.

## 7. Build order

One session, in order. Each stage is its own commit series, ends with
`make luau-vm-diff` at zero disagreements and the micro table re-taken,
and is compared with its rows in section 6 before the next begins.

1. **Instrument.** Run `call_path_micro.py` on `main`; record the table.
2. **No reserved frame** (4.1), results in place for plain natives, the
   safepoint trigger (4.5; built, then reverted when 4.5 was withdrawn on
   2026-11-12). Checks `select('#')` and every level-sensitive
   observable.
3. **The fast table** (4.2). Checks the builtin rows and the audio block.
4. **Small frames, the pending stack, `LibCont`, `CALL`/`RETURN` in the
   handler** (4.3). Checks the call and `pcall` rows.
5. **One-swap thread switch** (4.4). Checks `co()`.
6. **The generic-host experiment** (4.6), scratch only.

Differential cases to add before the code they cover:

- For 4.1: `error` with levels 0 to 3 from inside and around a builtin;
  an argument error's function name; `debug.info(1 .. 3, "sln")` and
  `debug.traceback()` called as the builtin and from an `xpcall` handler
  after a builtin raised; `getfenv(1)` and `setfenv(1, t)`; a memory
  error raised inside a plain builtin under `Limits`; each with the
  builtin called directly, through `pcall`, and as a metamethod.
- For 4.2, per ported body: the handled types; each wrong type and each
  missing argument (the fallback's error text); zero, one and `MULTRET`
  results (`f(math.abs(x))`); the same call after `getfenv(0)` turned
  `safeenv` off and after `math.abs = other`; and for the variadic ones,
  more than three arguments.
- For 4.3: a return to a caller waiting on a metamethod, on a native
  continuation, at a host boundary, and at a thread base; `__call`;
  varargs with fewer and more arguments than parameters; stack growth
  inside a call; call depth at the limit and one below it.

## 8. Decisions needed

1. **Scope.** The host-boundary type moves to the next lane (section 5).
2. **All 133 fast bodies are ported**, the `integer*` ones included.
3. **The host stays `dyn`**, with the generic experiment recorded but not
   landed (4.6).
4. **The callee's registers stay nil on entry** (4.3), a deliberate
   difference from the reference for isolation inside one VM.
