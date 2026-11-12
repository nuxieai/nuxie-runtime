# Safe Luau VM: the register window (lane 10 design)

Status: **design, not built.** Written by the coordinator on `origin/main`
`86ed289f1c` (lane 9 landed). It follows `docs/luau-safe-vm-call-path.md`
and changes how the dispatch loop in `crates/nuxie-luau-vm/src/interp.rs`
reaches registers, tables, protected calls and threads.

Lane 9's predictions missed on 16 of 17 rows because they priced the call
steps and took the per-bytecode cost as given. This design prices that
cost first, from a scratch prototype measured on this tree
(`docs/evidence/luau-safe-vm-register-window/prototype.diff`), and gives
ranges only where the prototype or a lane 9 trace backs them. Rows with no
measurement behind them are marked "derived".

No caches (2026-11-12 rule): nothing here is a derived copy of state that
needs invalidation. Section 4 says why for each piece.

## 1. Terms

- **Register**: one of a Lua function's up to 255 local slots. Operands
  `A`, `B`, `C` of an instruction are register numbers, one byte each.
- **Window**: the 256 stack slots starting at the running frame's `base`,
  borrowed as `&mut [Value; 256]`.
- **Register arm**: a dispatch arm that touches only the window, the
  constants, the code and (by field) the heap.
- **General arm**: any other arm. It may call any `&mut Core` method.
- **Floor**: what a bytecode costs before it does its own work: fetch,
  decode, dispatch, register reads and writes.

## 2. Measurements this design starts from

`tools/luau-bench/perf/floor_micro.py` writes 10 one-line loops whose
state is all in locals of the timed function (lane 9's micro benches keep
their state in upvalues, which hid the register cost behind `GETUPVAL`).
200,000 iterations, `LUAU_BENCH_CLOCK=instructions`, Bazel release
fat-LTO, median of 2. Retired instructions per iteration with the empty
loop subtracted. 1-minute load was above 80 throughout; instruction counts
do not move with load, but build-to-build code layout moves a row by up to
about 10.

| Loop body | main | prototype | luaur |
|---|---:|---:|---:|
| empty loop (raw, not subtracted) | 87 | 51 to 53 | 38 |
| `s = s + i` | 64 | 32 to 34 | 33 |
| `s = s + i; t = t + s; u = u + t` | 193 | 96 to 102 | 99 |
| `a, b = b, a` | 160 | 57 to 66 | 51 |
| `if i < 0 then s = i end` | 123 | 49 to 62 | 48 |
| `s = i * 2` | 70 | 35 to 38 | 29 |
| `s = t[1]` | 171 | 52 | 29 |
| `s = t.x` | 38 | 17 to 23 | 20 |
| `t.x = i` (existing key) | 424 | 82 | 41 |
| `s = t[(i % 8) + 1]` | 314 | not converted | 104 |

"Prototype" is two builds: the first converts `MOVE`, `LOADN`, `LOADK`,
`ADD`, `MULK`, `JUMPIFNOTLT` and `FORNLOOP` to the window; the second adds
`GETTABLEN`, `GETTABLEKS` and `SETTABLEKS`. The prototype dispatches twice
on every instruction it does not convert (a first `match` for the
converted arms, then the old one), so its converted rows carry one
dispatch the real build will not, and its unconverted rows are slower than
main. It is not correct code: it raises `MIN_STACK` to 276 to get the
window's slack, which moves the stack-overflow depth.

What it shows:

1. **The borrow checker accepts the shape.** A `&mut [Value; 256]` held in
   a loop local across iterations compiles, provided every path that calls
   a `&mut self` method assigns the local again before the next use. Paths
   that forget are compile errors (E0499/E0502), not stale reads.
2. **Converted arms land at luaur's cost** with no other change: `ADD`
   32 to 34 against 33, compare-and-branch 49 to 62 against 48.
3. **The table store was three lookups.** `Table::set` reads the key to
   pick `__newindex`, looks it up again to size the growth check, and a
   third time to write, with two capacity sums and a budget settle around
   it. One lookup that returns the slot takes `t.x = i` from 424 to 82.

Lane 9's traces (its evidence, section 4) price the rest of today's loop:
dispatch about 20 per instruction, `MOVE` 51 with its dispatch, `GETUPVAL`
47, `FORNLOOP` 62, Lua `CALL` 141, `RETURN` 84.

## 3. What this VM does today

`reg!(i)` is `self.get(base.saturating_add(i))`, and `set_reg!` its
mirror. Each one reloads the stack's pointer and length from `self`
(any call may have moved them), adds with saturation, compares against
the length, and on a read selects a default `Nil`. `base` itself is
reloaded from a spill slot. That is about 15 instructions per register
access where the reference has one load or store.

`self.exec.pc = pc` is stored on every instruction.

The loop keeps 14 words live (`self`, `host` ×2, `protos`, `proto`,
`closure`, `base`, `func`, `ftop`, `pc`, `code` ×2, `k` ×2). AArch64 has
10 callee-saved registers, so some are spilled and reloaded around every
arm.

`pcall(f, x)` runs as a native function: a native frame, a `Request`, a
pending entry with a continuation, the callee's frame, then `deliver`,
`step_native` again and `finish_native` to move the results. About 1,500
instructions of protocol around one Lua call (lane 9 evidence).

A thread switch copies the 136-byte `Exec` twice per side (83
instructions each), and `resume`/`yield` run the same request protocol as
`pcall`.

## 4. The design

### 4.1 The window

At the top of the frame loop, and after any general arm:

```rust
let regs: &mut [Value; 256] = self.exec.stack
    .get_mut(base..)
    .and_then(|s| s.first_chunk_mut::<256>())
    .ok_or_else(/* internal error, never a panic */)?;
```

A register arm indexes it with the instruction's byte operand:
`regs[usize::from(a)]`. A `u8` index into a 256-element array is in
bounds by type, so the compiler emits no check and no saturating add. A
read is one 16-byte load, a write one store.

Rules:

- **`regs` is assigned at frame entry and re-assigned after every general
  arm**, once, at the end of the arm (and before any bare `continue`
  inside it). The compiler enforces this: using `regs` after a
  `&mut self` call without re-assigning does not compile.
- **Register `a + 1`, `a + 2`** (the `for` loops, `NAMECALL`) use
  `a.wrapping_add(n)`. The verifier already rejects operands at or above
  `maxstacksize`; wrapping keeps the index a `u8` so no check appears.
- **Slots outside the window** (an open upvalue of this thread, results
  of a variable-count call above `ftop`) are reached through
  `self.exec.stack` in a general arm.

**The invariant: a running Lua frame has 256 slots from its `base`.** The
frame-entry stack compare asks for `base + 256` physical slots where it
asks for `base + maxstacksize + MIN_STACK` today. The *limit* check
(`Limits::max_stack_slots`, "stack overflow") keeps today's expression, so
scripts overflow at the same depth as now and as the reference; the
physical stack may run up to 256 slots past the limit. Each thread's
stack therefore starts at 256 + `MIN_STACK` slots or more. See decision 1
for the memory this costs.

Slots between a frame's `maxstacksize` and 256 belong to no one or to a
dead callee. The verifier keeps verified code from naming them, as it
does today (`reg!` with an out-of-range operand reads the same slots
now). No new exposure.

Why this is not a cache: `regs` is a borrow of the stack, not a copy. It
cannot disagree with the stack, and the compiler will not let it outlive
a change to the stack's allocation.

### 4.2 The loop's locals

The loop keeps eight words: `self`, `host` (two), `regs`, `code` (two),
`pc`, and `closure`. `k` (two more) stays if the build shows it in
registers; otherwise constants are read through `proto`.

- **`base`, `func`, `ftop`, `proto`, `protos`** leave the hot set. A
  general arm that needs one reads it from the top frame
  (`self.exec.frames.last()`), which is where it lives.
- **`self.exec.pc` is written by general arms only**, as their first
  statement (`sync_pc!()`). A register arm cannot raise, call, allocate
  or be observed, so nothing can read `exec.pc` while one runs. An arm
  with a fast path and a slow tail syncs at the top of the tail.
- **The interrupt check** in `FORNLOOP`, `JUMPBACK` and `JUMPX` reads
  `self.interrupt_on` by field beside the window; only when it is set
  does the arm become general.

One `match`, as now. Each arm is written fast path first; a slow tail of
more than a few lines moves to a `#[cold] #[inline(never)]` method on
`Core`, so the loop body stays small and the tail's locals do not widen
the loop's frame.

### 4.3 Which arms are register arms

Converted in stage 1, with the number fast path in the window and the
metamethod tail general: `MOVE`, `LOADNIL`, `LOADB`, `LOADN`, `LOADK`,
`LOADKX`, the arithmetic family and its `K`/`RK` forms, `NOT`, `MINUS`
(number), `LENGTH` (table without `__len` stays general), all `JUMP*`
forms including the `X*K*` constant compares, `FORNPREP` (numbers),
`FORNLOOP`, `GETUPVAL` and `SETUPVAL` for value captures and closed
references (4.5), `FASTCALL1`/`2`/`2K`/`3` argument reads.

Everything else is a general arm in stage 1 and keeps its current body
with `self.get`/`self.set`.

### 4.4 Tables beside the window

Heap methods take `&self.heap` or `&mut self.heap`, a different field
from `self.exec.stack`, so a register arm may call them while it holds
`regs`. (Methods on `Core` may not; that is the line between the two
kinds of arm.)

- **Reads.** `GETTABLEKS`, `GETTABLEN`, `GETTABLE` with a table receiver:
  `self.heap.table(t)` and a raw lookup (`get_str`, the array slot for an
  integer-valued number, `get` otherwise). A non-nil result is written to
  the window. Nil, a non-table receiver, a userdata direct field or a
  vector component take the general tail, which is today's code.
- **Stores to an existing key.** One new operation,
  `Heap::table_slot_mut(id, key) -> Option<&mut Value>`: the write
  barrier, then one lookup that returns the array slot or the node's
  value for a key that is present with a non-nil value, and `None` for a
  read-only table. `SETTABLEKS`, `SETTABLEN`, `SETTABLE` write through it.
  `None` takes the general tail.
  - No `__newindex` question arises: the metamethod is consulted only
    when the raw value is nil.
  - No budget check is skipped: the table's capacity cannot change when
    no key is added.
- **`Table::set` itself becomes one lookup** for every caller (`rawset`,
  the table library, the embedding API): find the slot and write, or fall
  to the new-key path, which alone computes growth and charges the
  budget. Same observable rehash triggers (D4 pins them); the vm-diff
  corpus and the table conformance scripts are the check.

### 4.5 Upvalues

`GETUPVAL` reads `self.heap.function(closure)` and its `upvals[b]`, both
through `&self.heap`, so it is a register arm for a value capture. For a
reference capture it reads the upvalue arena: a closed upvalue is a
register arm; an open one names an absolute stack slot, so it is general.
`SETUPVAL` mirrors it (the function's write barrier is a heap method).

No layout change to closures in this lane. If the build shows the
function-arena lookup dominating `GETUPVAL`, report it; do not add a
per-frame copy of the upvalue pointer.

### 4.6 Calls and returns in the window

`CALL` of a Lua function (lane 9's inline path) becomes:

1. callee value from `regs[a]`; function and prototype lookups by heap
   field, as today;
2. depth compare; the stack compare of 4.1 (limit, then `new_base + 256`
   physical);
3. missing parameters and the registers above them nil-filled through a
   slice of the stack (isolation, kept from lane 9);
4. caller's `savedpc` written to its frame; the callee's frame pushed;
5. `regs` re-assigned at the new base; `code`, `pc`, `closure` switched.

`RETURN` to a plain Lua caller: pop, move the results down inside the
stack slice, re-assign `regs` at the caller's base, switch `code`, `pc`,
`closure` from the caller's frame.

Frameless natives and fast bodies keep lane 9's signatures (they take
`&mut Core` and absolute slots), so their arms are general: sync, call,
re-assign. Their arguments are read from the window before the call.

### 4.7 A protected call is a frame, not a function that runs

`pcall` gets its own native kind, `NativeFn::Pcall`. `CALL` of it, when
the function being protected (`regs[a + 1]`) is a Lua function, does this
in the loop:

1. push a native frame record for `pcall` at `func`, flagged `CATCH`,
   carrying what `restore_calls` needs (the C-call counters `push_call`
   saves under `CallMode::Counted` today; in the frame's spare field if
   they fit, else one `Pending` entry);
2. enter the Lua callee at `func + 1` with the remaining arguments and a
   variable result count, exactly as 4.6.

`RETURN` from a Lua frame whose caller is a `CATCH` frame: pop both,
write `true` at the `CATCH` frame's `func`, the results after it, adjust
to that frame's `nresults`, restore the counters, continue in the frame
below.

An error: `find_catcher` treats a `CATCH` frame as a catcher alongside
the pending continuations it scans today. Catching truncates to it, pops
it, writes `false` and the error value at its `func`, restores the
counters, continues in the frame below.

Unchanged: the frames list is truthful at every instant (`debug.info`,
tracebacks and the call-depth limit see a `pcall` frame and a Lua frame,
as now), so nothing is materialized for observers. A yield inside the
callee leaves the `CATCH` frame parked with the thread; on resume the
callee's `RETURN` finds it as above. `pcall` of a native, of a value with
`__call`, or reached other than by `CALL` (a metamethod, the host API,
`pcall(pcall, f)`) takes today's path through `step_native`. `xpcall`
stays on today's path: its handler runs before unwinding.

Why this is not a cache: the flag is the fact ("this frame catches"),
stored once where the fact lives.

### 4.8 A thread switch is a pointer swap

`Core::exec` and `Thread::exec` become `Box<Exec>`. `swap_exec` swaps two
pointers. Every `self.exec.field` access gains one pointer load; register
arms do not touch `exec` (4.2), which is why this follows the window and
could not precede it.

Then `coroutine.resume`, the function `coroutine.wrap` returns, and
`coroutine.yield` get native kinds the `CALL` arm recognises, as `pcall`
does:

- **Resume / wrap call**: check the target is suspended; push a native
  frame record flagged `AWAIT` for the resume; move the arguments to the
  target's stack; swap; continue the frame loop on the target.
- **Yield**: move the values to the resumer's stack at its `AWAIT`
  frame's `func` (after `true` for `resume`, bare for `wrap`), pop the
  `AWAIT` frame, swap, continue in the resumer.
- **The coroutine's base frame returns or raises**: the same delivery,
  with `false` and the error for `resume`, a re-raise for `wrap`.

Every other way in (the host API, a metamethod, `pcall(coroutine.yield)`,
a yield across a native continuation) keeps today's request protocol.

This is the thinnest part of the design: it is written from `call.rs`
and the lane 9 trace, not from a prototype, and `thread.rs` has status
and C-call rules (`base_c_calls`, "attempt to yield across a C-call
boundary") that the direct path must reproduce exactly. If it does not
survive the code, stop at the pointer swap and report.

## 5. Out of scope, and why

- **Strings and the table library's bulk operations** (4.9x and 9.9x
  luaur). They are the largest remaining groups and need their own
  designs (the string store lane; `table.*` bodies).
- **Closure layout** (4.5).
- **Fast-body signatures** (4.6). Giving them the window is a follow-up
  if their rows stay out.
- **The host boundary and `Compat`**: the collapse lane.
- **`getfenv`/`setfenv` and imports**: waiting on Levi's decision.

## 6. Predictions

Retired instructions per iteration over the empty loop; the empty loop is
raw. "Now" is main `86ed289f1c`. A range marked **P** is bounded by the
prototype (its measured value, less the second dispatch, plus layout
noise). **T** is derived from a lane 9 trace with the steps this design
removes subtracted. **D** is derived with nothing measured behind it.

Floor rows (`floor_micro.py`), after stage 1 unless marked:

| Loop body | now | range | basis | luaur |
|---|---:|---|---|---:|
| empty loop (raw) | 87 | 38 to 55 | P | 38 |
| `s = s + i` | 64 | 22 to 36 | P | 33 |
| three adds | 193 | 70 to 105 | P | 99 |
| `a, b = b, a` | 160 | 40 to 66 | P | 51 |
| `if i < 0 then s = i end` | 123 | 36 to 62 | P | 48 |
| `s = i * 2` | 70 | 25 to 40 | P | 29 |
| `s = t.x` (stage 2) | 38 | 14 to 25 | P | 20 |
| `s = t[1]` (stage 2) | 171 | 35 to 55 | P | 29 |
| `t.x = i` (stage 2) | 424 | 50 to 85 | P | 41 |
| `s = t[(i % 8) + 1]` (stage 2) | 314 | 110 to 170 | D | 104 |

Call rows (`call_path_micro.py`), after stage 3 unless marked:

| Loop body | now | range | basis | luaur |
|---|---:|---|---|---:|
| `f()` | 335 | 180 to 260 | T | 146 |
| `local y = f(i)` | 415 | 220 to 320 | T | 169 |
| `local a, b = f(i, i)` | 614 | 290 to 430 | T | 227 |
| `o:m(i)` | 597 | 300 to 440 | T | 215 |
| `math.abs(i)` | 127 | 85 to 115 | T | 68 |
| `math.min(i, 5)` | 169 | 115 to 155 | T | 79 |
| `select('#', i)` | 424 | 250 to 360 | T | 238 |
| `pcall(f, i)` (stage 4) | 1,792 | 380 to 600 | D | 743 |
| `co()` (stage 5, swap only) | 2,309 | 1,900 to 2,100 | T | 567 |
| `co()` (stage 5, direct) | 2,309 | 500 to 950 | D | 567 |

How `f()` is derived: lane 9 traced `GETUPVAL` 47 + `CALL` 141 + `RETURN`
84 + three dispatches of 20. Expected here: `GETUPVAL` about 25 (window
write, no spill reloads), `CALL` about 95 (no locals reload, callee read
from the window, no per-instruction `pc` store to undo), `RETURN` about
55, dispatch about 10 each: 205.

Suite groups against luaur (derived from the rows above; the table and
string groups this lane does not touch hold the total up):

| Group | now | range |
|---|---:|---|
| arithmetic and loops | 2.23x | 1.1x to 1.5x |
| array access and iteration | 2.81x | 1.4x to 2.1x |
| table get/set by constant key | 2.90x | 1.3x to 2.0x |
| Lua calls and returns | 2.76x | 1.5x to 2.1x |
| builtin calls | 2.28x | 1.5x to 2.0x |
| error path | 1.89x | 1.0x to 1.6x |
| all 87 | 3.39x | 2.3x to 2.9x |
| audio block (per frame) | 2.0x | 1.3x to 1.7x |

These are report-only. No row is a gate.

**Stop rule**, as in lane 9. A row outside its range is one of: (a) a
designed step built heavier than the text: fix it; (b) a step the design
leaves out: stop and report before going on; (c) cost outside this
design's reach, shown by a trace: record it with the trace.

## 7. Build order

Each stage ends with its rows measured and the gates green: `make
luau-vm-diff`, `make luau-conformance-nuxie`, `make luau-conformance`,
`make luau-vm-ratchets`, `generate.py --check`, crate tests, and
`nuxie-scripting` under both settings.

1. **The window and the loop's locals** (4.1 to 4.3, 4.5): the stack
   invariant with the limit check unchanged, `regs`, `sync_pc!`, the
   register arms, every other arm general. A test that the
   stack-overflow depth and message are unchanged at several limits.
2. **Tables** (4.4): reads beside the window, `table_slot_mut`, and
   `Table::set` as one lookup.
3. **Calls and returns in the window** (4.6).
4. **`pcall` as a frame** (4.7). Tests: error inside, yield inside then
   return and then error, `debug.info` levels and a traceback through
   it, the C-call counter after a caught error, nested `pcall`,
   `pcall` of a native and of a `__call` value (old path).
5. **Threads** (4.8): `Box<Exec>` first, measured alone; then the direct
   resume and yield.

Then `codex review --base origin/main`, at most four rounds, the
evidence doc with every row of section 6, and this doc amended to "as
built".

## 8. Decisions needed

1. **Stack memory per thread.** The window needs 256 slots (4 KB) above
   the running frame's base, so every thread's stack is at least about
   4.4 KB where a new coroutine's is about 1 KB today. The alternative
   is a window sized to the function (`&mut [Value]`), which keeps a
   length compare on every register access: about 2 instructions on
   each of roughly 3 accesses per bytecode, against a floor of 30 to
   40. Recommended: the 256-slot window. It matters only to a script
   that holds many coroutines at once.
2. **Whether stage 5's direct resume and yield are in this lane.**
   Recommended: yes, last, and dropped back to the pointer swap alone if
   the status and C-call rules do not fit the direct path cleanly.
