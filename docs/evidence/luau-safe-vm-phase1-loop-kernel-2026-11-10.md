# Safe Luau VM, Phase 1: the loop-kernel experiment (2026-11-10)

This is the follow-up the Phase 1 core spike proposed
(`docs/evidence/luau-safe-vm-phase1-core-spike-2026-11-01.md`, section 6). It
asks whether a `#![forbid(unsafe_code)]` Luau interpreter loop can reach C++
Luau's per-instruction cost, and if so which loop shape does it. The answer
decides D1 in `docs/luau-safe-vm-plan.md`.

The harness is `spikes/luau-loop-kernel/`. It is throwaway, its own cargo
workspace, and has no Bazel target. The work started on 2026-11-01 and stopped
when the disk filled; it resumed and was measured on 2026-11-10.

**Outcome.**

- **The D1 perf kill fires again.** No loop shape gets near 1.3x C++.
  - The best variant is 1.94x C++ (geomean of interleaved minimums, same
    sitting). It is either `value_aligned` alone or the
    `typed+mask_fetch+value_aligned` combination; they tie at 1.94–1.95x
    across the two runs.
  - The control (the spike's loop) is 1.96–1.99x C++.
  - The vendored VM is 1.24–1.25x C++ in the same sitting.
- **The spike's diagnosis was wrong.** It blamed per-instruction overhead
  in the loop: dispatch, instruction fetch, operand decoding and bounds
  checks. None of the five loop variables moves the five-bench geomean by
  more than 6%, and the best combination moves it by 0.6–2.5%.
- **The cost is in the handler bodies and the out-of-loop paths.** Even in
  a small, separately compiled hot loop whose state stays in machine
  registers, OOP_field_access retires 1.6x C++'s machine instructions per
  bytecode instruction.
  - Lua calls and returns, coroutine switches and native calls all leave
    the loop. On the benches dominated by them (sieve, sha256, tictactoe)
    the spike retires 2 to 2.4x C++'s instructions.
- There is still no costed path down. Section 7 has the proposed D1 status
  line and the next experiment.

## 1. What was built

The harness is a copy of `spikes/luau-core-spike`. It keeps the spike's heap,
tables, `Value`, base library and semantics. Only the interpreter loop
varies, by cargo feature, one variable at a time against the control:

| Variant | What changes against the control |
|---|---|
| control | The spike's loop as committed. It matches on `(insn & 0xff) as u8` over raw `u32` words, fetches with `code.get(pc)`, and indexes a `&mut [Value; 256]` register window with `u8` operands. |
| `typed` (task item 2) | Bytecode is decoded once per prototype into an 8-byte `Instr` enum (`src/typed.rs`), one per code word. The pc, jump offsets and the saved frame pc mean the same as in the raw stream, and an AUX word becomes `Instr::Aux`. The loop matches the enum. KS string keys, DUPCLOSURE closures and DUPTABLE templates are resolved at load, so those handlers do no constant-array lookup. |
| `mask_fetch` (task item 3) | The fetch stream is padded to a power of two. Every fetch, including AUX reads, is `code.get(pc & mask)`. The disassembly confirms the bounds check is gone (`and` + `ldr`, no compare). |
| `value_aligned` (task item 4) | `#[repr(u8)]` `Value`, with every id type `#[repr(align(8))]`. Every payload sits at offset 8, and the size stays 16 bytes. |
| `regs_dyn` (task item 5) | Registers are `&mut [Value]` over `stack[base..]`, bounds-checked on every access, instead of the window. The control already has the window the task describes, so this variant measures what the window buys. |
| `typed+mask_fetch`, `typed+mask_fetch+value_aligned` | Combinations, after the single-variable runs. |
| `typed+mask_fetch+value_aligned+hot_split` | Added after the main run (section 5). The hot opcodes' fast paths run in a small `#[inline(never)]` loop. Anything else falls back, one instruction at a time, to the full loop. |

Not done from the task list:

- **The `split_first` fetch.** The masked fetch already removes the check
  that pattern targets, and the disassembly shows it, so `split_first`
  could not remove anything further.
- **The NaN-boxed `u64` value.** The data below says value layout is
  within noise (section 4), so a NaN-boxed representation is not where
  the factor of 2 is. NaN-boxing also means rewriting every `match` on
  `Value` in the 4,800-line copy.

### 1.1 References

- **vendored**: the vendored luaur VM, driven through `luaur_rt` with the
  code copied from `tools/luau-bench/src/vm.rs`: sandboxed state, the
  script on a fresh thread, `print` captured, `collectgarbage` provided.
  - It is built with cargo at the repository's release profile (fat LTO,
    one codegen unit, unwind), as a measurement-only binary, not the Bazel
    `luau-bench__bin`.
  - Its OOP_field_access minimum is 18.58 ms here. The committed
    `baseline-luaur.json` median is 20.32 ms.
- **cpp**: upstream Luau `0.734`, commit
  `3fc82b1071ab387531175869afc4fb528464afa4`, the pin in
  `tests/luau-conformance/ATTRIBUTION.md`.
  - Fetched depth 1, configured with `cmake -DCMAKE_BUILD_TYPE=Release`
    (`-O3 -DNDEBUG`, Apple clang 21) and built as target `Luau.Repl.CLI`.
  - Run as `luau <merged script>.lua` from the script folder.
  - The build lived in the session scratchpad and has been deleted.

### 1.2 Correctness before timing

`tools/loop_kernel.py verify` gates every number below:

- Every VM runs the five merged scripts, and an instrumented copy of each.
  The instrumented copies print a checksum of work the bench does not
  print. The outputs are compared after the timing fields of the report
  lines are blanked.
- All 8 spike variants, the vendored VM and C++ print identical output:
  - sieve: 403,200 primes (24 runs x 100 loops x 168);
  - TableIteration: a sum of 1,212,000,000;
  - OOP_field_access: an accumulator of 20,160,000,000;
  - tictactoe: `Analyzed positions: 44628`;
  - sha256: its own digest `assert`.
- Every variant's `.count` build dispatches exactly the control's
  instruction count on every script:

| Script | Dispatched bytecode instructions |
|---|---:|
| sieve | 385,388,226 |
| tictactoe | 890,398,770 |
| sha256 | 134,062,122 |
| OOP_field_access | 480,001,266 |
| TableIteration | 49,448,274 |

OOP_field_access is 24 runs x 10M iterations x 2 instructions
(`GETTABLEKS`, `FORNLOOP`), plus 1,266 of setup.

All variants run the same bytecode: the vendored compiler at optimization 1,
debug 1, precompiled once by `compile-scripts`. The vendored VM compiles the
same source with the same compiler. The C++ CLI compiles with its own 0.734
compiler at the same levels, so its instruction stream may differ in detail.

## 2. Method and its limits

- **Host:** Apple M5 Max, 18 cores (6 performance, 12 efficiency), rustc
  1.97.1.
- **Load:** the 1-minute load average was 7–33 (median 14) during the main
  run and 9–29 (median 14) during the `hot_split` run. That is much
  quieter than the spike's 46–800, but still not idle.
- **Statistic:** the `tools/luau-bench` one. Per pass, each script's 20
  kept runs are reduced to their median.
- **Passes:** each pass is one whole-suite run of one VM, in a fresh
  process. The passes are interleaved across every VM, and the VM order
  rotates each round. The load average is recorded before and after
  every pass.
  - Main run: 16 rounds over all 9 VMs, 144 passes.
  - `hot_split` run: 16 rounds over 5 VMs, 80 passes.
- **Reported:** the minimum and median of the 16 pass medians, and the
  geomean of the minimums. Every ratio to C++ is a direct same-sitting
  ratio. It equals the ratio to the vendored VM times the vendored VM's
  own same-sitting ratio to C++ (1.24x in the main run).
- **Instructions retired:** one `/usr/bin/time -l` run per VM per script,
  divided by the control's dispatched count. This is load-independent,
  and it includes process start, bytecode load and, for vendored and C++,
  compilation; all are small next to billions.
- **Cycles:** `/usr/bin/time -l` cycles are **not** usable under this
  load. A single run lands on a performance or an efficiency core, and
  one variant read 25 cycles per instruction on OOP against 7 for its
  neighbours. They are not reported.
- **ns per instruction:** the minimum run time x 24 runs / dispatched
  instructions per script. It slightly understates, because setup
  instructions sit outside the timed region. The C++ figure uses the
  spike's dispatched count, since C++ compiled its own bytecode.

## 3. Results (main run, 16 interleaved passes per VM)

### 3.1 Time, minimum of pass medians (ms)

| VM | sieve | tictactoe | sha256 | OOP_field_access | TableIteration | geomean |
|---|---:|---:|---:|---:|---:|---:|
| control | 153.72 | 89.96 | 19.00 | 33.53 | 5.34 | 34.23 |
| typed | 158.91 | 104.94 | 22.56 | 30.65 | 5.44 | 36.27 |
| mask_fetch | 153.93 | 94.69 | 19.02 | 33.22 | 5.22 | 34.39 |
| value_aligned | 151.14 | 94.83 | 17.93 | 34.11 | 5.23 | 34.07 |
| regs_dyn | 157.71 | 100.06 | 18.74 | 37.72 | 5.30 | 35.83 |
| typed+mask_fetch | 164.32 | 99.76 | 19.85 | 32.53 | 5.31 | 35.48 |
| typed+mask_fetch+value_aligned | 150.51 | 99.18 | 18.20 | 32.50 | 5.17 | 34.03 |
| vendored | 92.60 | 120.44 | 8.55 | 18.58 | 2.71 | 21.69 |
| cpp | 79.19 | 56.53 | 7.13 | 19.64 | 2.60 | 17.47 |

### 3.2 Ratios, by minimum

| VM | sieve | tictactoe | sha256 | OOP | TableIter | **/ C++** | **/ vendored** |
|---|---:|---:|---:|---:|---:|---:|---:|
| control | 1.94 | 1.59 | 2.66 | 1.71 | 2.06 | **1.96** | 1.58 |
| typed | 2.01 | 1.86 | 3.16 | 1.56 | 2.10 | **2.08** | 1.67 |
| mask_fetch | 1.94 | 1.67 | 2.67 | 1.69 | 2.01 | **1.97** | 1.59 |
| value_aligned | 1.91 | 1.68 | 2.51 | 1.74 | 2.02 | **1.95** | 1.57 |
| regs_dyn | 1.99 | 1.77 | 2.63 | 1.92 | 2.04 | **2.05** | 1.65 |
| typed+mask_fetch | 2.08 | 1.76 | 2.78 | 1.66 | 2.05 | **2.03** | 1.64 |
| typed+mask_fetch+value_aligned | 1.90 | 1.75 | 2.55 | 1.65 | 1.99 | **1.95** | 1.57 |
| vendored | 1.17 | 2.13 | 1.20 | 0.95 | 1.04 | **1.24** | 1.00 |

The per-bench columns are ratios to C++. By median, the geomeans to C++ are:

| VM | geomean / C++, median |
|---|---:|
| control | 1.97 |
| typed | 2.10 |
| mask_fetch | 2.01 |
| value_aligned | 2.00 |
| regs_dyn | 2.06 |
| typed+mask_fetch | 2.05 |
| typed+mask_fetch+value_aligned | 1.98 |
| vendored | 1.26 |

### 3.3 Per instruction

| VM | sieve | tictactoe | sha256 | OOP | TableIter |
|---|---:|---:|---:|---:|---:|
| *ns per dispatched instruction* | | | | | |
| control | 9.57 | 2.42 | 3.40 | 1.68 | 2.59 |
| typed | 9.90 | 2.83 | 4.04 | 1.53 | 2.64 |
| mask_fetch | 9.59 | 2.55 | 3.41 | 1.66 | 2.53 |
| value_aligned | 9.41 | 2.56 | 3.21 | 1.71 | 2.54 |
| regs_dyn | 9.82 | 2.70 | 3.35 | 1.89 | 2.57 |
| typed+mask_fetch | 10.23 | 2.69 | 3.55 | 1.63 | 2.58 |
| typed+mask_fetch+value_aligned | 9.37 | 2.67 | 3.26 | 1.63 | 2.51 |
| hot_split (section 5) | 14.45 | 5.78 | 6.84 | 1.70 | 1.94 |
| vendored | 5.77 | 3.25 | 1.53 | 0.93 | 1.31 |
| **cpp** | **4.93** | **1.52** | **1.28** | **0.98** | **1.26** |
| *machine instructions retired per dispatched instruction* | | | | | |
| control | 157.6 | 59.8 | 96.0 | 68.6 | 72.3 |
| typed | 159.9 | 63.4 | 102.0 | 63.1 | 73.9 |
| mask_fetch | 157.7 | 59.3 | 95.3 | 67.1 | 70.9 |
| value_aligned | 157.2 | 62.6 | 97.7 | 68.6 | 75.3 |
| regs_dyn | 162.7 | 66.9 | 102.8 | 75.1 | 84.2 |
| typed+mask_fetch | 156.7 | 59.2 | 96.4 | 59.1 | 68.9 |
| typed+mask_fetch+value_aligned | 156.1 | 59.3 | 95.8 | 56.7 | 69.1 |
| hot_split (section 5) | 209.3 | 81.7 | 125.5 | 52.6 | 67.3 |
| vendored | 77.8 | 38.6 | 54.4 | 37.7 | 48.9 |
| **cpp** | **65.0** | **30.8** | **40.0** | **32.6** | **45.3** |

The `hot_split` time row comes from its own run (section 5), against that
run's references; its instruction counts come from a separate counters run.

C++ costs 0.98 ns per instruction on OOP_field_access, or 1.96 ns per
iteration, here. The spike's quiet-host figure was 2.4 ns per iteration.
The best spike loop costs 1.53–1.63 ns per instruction there, 1.6x C++.

## 4. What each variable bought

The change is in the geomean of minimums, against the control in the same
run. The machine-instruction column is OOP's instructions per dispatched
bytecode instruction.

| Variable | Five-bench time | OOP time | OOP machine instructions | Reading |
|---|---:|---:|---:|---|
| `typed` | +6.0% | −8.6% | −8% (68.6 → 63.1) | Removes the opcode range check and the operand shifts. OOP gets faster, but tictactoe +17% and sha256 +19% get slower. The disassembly shows LLVM decodes every field of the 8-byte `Instr` before the jump (six loads and extends), and the `execute` function still spills its hot state (below). |
| `mask_fetch` | +0.5% | −0.9% | −2% | The check is gone, but the mask is reloaded from the stack on every dispatch. Net zero. |
| `value_aligned` | −0.5% | +1.7% | 0% | Value layout is within noise. |
| `regs_dyn` (inverse) | +4.7% | +12.5% | +9% | The fixed register window is worth about 5% overall and 12% on OOP. Keep it. |
| `typed+mask_fetch` | +3.7% | −3.0% | −14% | |
| `typed+mask_fetch+value_aligned` | −0.6% (−2.5% in the second run) | −3.1% (−8.6%) | −17% (68.6 → 56.7) | The best combination. Its geomean is within noise of `value_aligned` alone. |

The machine code explains why the loop variables cannot win:

- `execute` is one large function: every opcode's fast path plus the
  slow-path handlers.
- In every variant, its hot state is spilled to the stack and reloaded on
  every dispatch: the code base and mask (`ldp x8, x9, [sp, ...]`), the
  register-window base, and some `Value` copies.
- The fetch and dispatch the variants change are a few cheap,
  well-predicted instructions. Even the 17% cut in OOP's instructions buys
  only 3–9% of time: the removed instructions were cheap ones that the
  core retires alongside the dependent load chain.

## 5. Follow-up: `hot_split`, a small loop that stays in registers

The register-pressure finding suggested one shape the task list did not
have:

- A separate `#[inline(never)] fn hot_loop` handles the fast paths of the
  hot opcodes, about 30 of them. They are the loads, moves, table gets
  and sets, add/sub/mul with and without constants, the jumps and
  compares, `FORNLOOP` and the table case of `FORGLOOP`.
- Its live state is only code, mask, pc, registers, heap and constants,
  and the disassembly confirms they stay in machine registers.
- Anything else returns without consuming the instruction. The full loop
  executes that one instruction and calls back in.
- It is built on `typed+mask_fetch+value_aligned`, verifies like the rest,
  and dispatches the same counts.

16 interleaved passes, against the same references:

| VM | sieve | tictactoe | sha256 | OOP | TableIter | **/ C++** |
|---|---:|---:|---:|---:|---:|---:|
| control | 156.60 | 90.17 | 18.70 | 34.84 | 5.33 | **1.99** |
| typed+mask_fetch+value_aligned | 148.31 | 98.78 | 17.97 | 31.85 | 5.16 | **1.94** |
| …+hot_split | 232.04 | 214.32 | 38.22 | 33.95 | 4.00 | **2.78** |
| vendored | 91.66 | 121.02 | 8.51 | 18.59 | 2.73 | **1.25** |
| cpp | 77.89 | 56.28 | 7.12 | 19.71 | 2.54 | 1.00 |

- **TableIteration** gets 23% faster, to 1.57x C++. It is the one bench
  whose whole hot loop is fast paths with no calls.
- **OOP_field_access does not move:** 1.72x C++, 52.6 machine
  instructions per bytecode instruction against C++'s 32.6. With its state
  in registers, the two handlers themselves are still 1.6x C++'s work.
  - Reading the GETTABLEKS path: an arena bounds check, then the `Table`
    header (the nodes pointer and length are two loads through the arena
    element), the masked Fibonacci hash, a bounds-checked node load, a
    `Value` tag-and-id key compare, a `meta` check, and the store.
  - C++ uses a predicted node slot and one pointer chain. The spike's E2
    tried the predicted slot inside the spilled loop and saw no change;
    it was not retried here.
- **The call-heavy benches get much worse** (tictactoe 3.8x, sha256 5.4x),
  because every Lua call, return, native call and fastcall pays an extra
  out-of-line call and return. As built, this is not a candidate.

## 6. Where the factor of two is

From the per-instruction counts, against C++:

| Bench | Spike's best machine instructions / C++'s | Where they go |
|---|---:|---|
| sieve | 2.4x (156 vs 65) | Coroutine resume and yield. Each swaps `ThreadState` and runs a nested `execute`, at 9.4 ns per dispatched instruction against C++'s 4.9. |
| sha256 | 2.4x (96 vs 40) | Native calls that leave the fastcall path (`string.byte` with 4 results, `table.unpack` with 8). Each breaks out of the loop, calls through `call_native`, `place_results` and the safepoint, and re-derives the window. |
| tictactoe | 1.9x (59 vs 31) | Lua calls and returns. `Slow::Call` breaks out of the loop, then `push_lua_frame` (proto `Rc` clone, bounds-checked stack growth, frame push), then the loop re-derives everything. C++ handles a Lua-to-Lua `CALL` inside the loop. |
| OOP_field_access | 1.7x (57 vs 33) | Handler bodies: the table get path (section 5). |
| TableIteration | 1.5x (69 vs 45) | Handler bodies, `FORGLOOP` and `next_from`. `hot_split` recovers part of it. |

None of these is the loop's dispatch, fetch, operand decoding or register
access. Those were the five variables tested, and the best combination
improves the geomean by 0.6–2.5%.

## 7. Verdict and proposed status line

**D1's performance kill criterion fires.** The best safe loop is 1.94x C++
on the five benches (geomean of interleaved minimums, same sitting; 1.98x by
median), against a limit of 1.3x. The vendored VM is 1.24x C++ in the same
sitting, so the safe design is 1.57x the vendored VM. No loop shape tested
passes, so none is proposed as the D1 representation.

Proposed status line, for the coordinator to copy into the plan:

- **D1**, 2026-11-10: loop-kernel experiment
  (`docs/evidence/luau-safe-vm-phase1-loop-kernel-2026-11-10.md`).
  - **Perf kill criterion fires.** The best `#![forbid(unsafe_code)]`
    loop is 1.94x C++ geomean on the five benches (interleaved minimums,
    16 passes, same sitting; vendored is 1.24x C++).
  - Five loop variables improve the geomean by 2.5% at most: typed
    pre-decoded instructions, masked fetch, 8-aligned `Value` payloads,
    register window versus checked slice, and a register-resident hot
    loop.
  - The gap is in handler bodies (table get path 1.6x C++'s machine
    instructions) and in the out-of-loop call, coroutine and native-call
    paths (1.9–2.4x).
  - Those paths have no costed fix yet.
  - Status: **killed as specified** (no identified, costed path down).
    Section 8's fallback (harden the vendored VM in place) applies,
    unless the out-of-loop experiment below finds a costed path.

**What I would try next, and why.** One more bounded experiment, about two
days, before the plan switches to its section 8 fallback. Its targets are the
three places the instructions actually go, each measured by instructions
retired (load-independent) against C++:

1. **Lua-to-Lua `CALL` and `RETURN` inside the loop.** Push the frame and
   re-derive the window in the loop, without breaking out to `&mut self`,
   as C++ does. The borrow structure allows it when the stack is
   preallocated to the call-depth limit, so a call never reallocates.
   - Target: tictactoe from 59 toward 31 instructions per bytecode
     instruction.
2. **Coroutine switch by index, not by swapping `ThreadState`.** Keep each
   thread's stack in the thread arena and run the resumed thread in the
   same `execute`, changing only which thread is current.
   - Target: sieve from 156 toward 65.
3. **The table get path as one chain.** Put the table header inline in
   its arena slot and the nodes in one slab, and retry C++'s predicted
   node slot inside a register-resident loop.
   - Target: OOP from 57 toward 33.

Each is costable before it is built, from these counts. If the three
together cannot project the geomean under 1.3x C++, the safe VM does not
meet the bar, and the decision between a slower safe VM and hardening the
vendored one belongs to the plan, not to this document.

## 8. Not verified

- **Quiet host.** The load was 7–33, not idle. All ratios are same-sitting
  interleaved minimums; the absolute times would be lower on an idle host.
- **Cycle counts.** Unusable under this load (section 2), so IPC figures
  are not reported.
- **The vendored VM through Bazel.** It was built with cargo at the same
  release profile, not through `make luau-bench`. Its numbers match the
  committed baseline's scale.
- **NaN-boxing and `split_first`.** Not built (section 1).
- **C++ bytecode equivalence.** C++ ran bytecode from its own 0.734
  compiler, not the vendored compiler's. Its outputs matched; its
  instruction stream was not compared. The per-instruction C++ figures
  divide by the spike's dispatch counts.
- **`typed`'s tictactoe regression.** It reproduced across the passes
  (median 1.89x C++ against the control's 1.61x), but was not investigated.
- **Raw data.** The per-pass JSON files (schema 1, load per pass) were
  copied to the session-group notes folder, not committed. The harness
  regenerates them with `tools/loop_kernel.py bench`.
