# Safe, panic-free Luau VM: living plan

Status: **Reoriented 2026-11-11 (Levi) to the goal that started this
work: a memory-safe, panic-free Luau VM that passes the Luau tests. Speed
is a tracked budget, not a gate. Phase 0 landed 2026-10-31. The Phase 1
spikes (2026-11-01, 2026-11-10) proved the safe design: D2 passes, D3 is
confirmed, D1 is decided. Phase 2, the real crate, is next.** Owner: Levi.
Evidence so far: `docs/evidence/luau-safe-vm-bakeoff-2026-10-31.md`,
`docs/evidence/luau-safe-vm-d4-table-layout-2026-10-31.md`,
`docs/luau-safe-vm-seam.md`,
`docs/evidence/luau-safe-vm-phase1-core-spike-2026-11-01.md`,
`docs/evidence/luau-safe-vm-phase1-loop-kernel-2026-11-10.md`,
`docs/evidence/luau-workload-characterization-2026-11-10.md`.
This document is the thing to come back to. The principles and the decision
log are meant to be stable; the phase plan is expected to be rewritten as
decisions resolve. Every change to a decision's status gets a dated line in
its entry, never a silent edit.

## 1. Goal

Replace the vendored `luaur-vm` (a line-by-line C transliteration: 53k lines,
1433 `unsafe` sites, 505 public functions dereferencing raw pointers) with a
Luau VM that is:

1. **Memory-safe**: zero `unsafe` in the VM crate. FFI for tooling is the
   only exception, and it lives outside the VM.
2. **Panic-free**: every guest-reachable failure is a returned error. The
   abort-profile probe (`tools/luaur-rt-test-harness`, panic=abort) stays
   green across the whole error surface, not just the typed boundary.
3. **Behavior-compatible** with the engine the product runs today: same
   Luau bytecode in (versions 3 through 6 plus Rive's 7 through 11), same
   results and side-channel streams out, scored by differential tests
   against the vendored VM and by the upstream conformance corpus.
4. **Not slower than what we ship.** Ceiling: the vendored VM's committed
   baseline `tests/luau-bench/baseline-luaur.json` (fat-LTO Bazel release,
   geomean 7.77 ms over 87 benches, 2026-10-31; roughly 1.0x the C++
   numbers from the bake-off, where the non-LTO cargo build had read
   1.12x). Target: at or under that baseline. A safe design that lands at
   ruau's 9.6x is a failed design.
   - **Restated 2026-11-10 (Levi).** The speed goal is **scripted frame
     cost**, not parity on upstream microbenchmarks. Measured on real
     scripted workloads, authored bytecode is 21% of script instructions;
     the Luau-to-host boundary is 41% and host function bodies 37%
     (`docs/evidence/luau-workload-characterization-2026-11-10.md`).
     The metric, in retired instructions on a fat-LTO build: script
     instructions per frame and per event invocation, geomean across a
     fixed corpus, with instructions per host call, per callback entry
     and per bytecode op carried as diagnostics. Target: at or under
     today's engine on every workload, and well under it on
     boundary-heavy ones.
   - **The corpus must include compute-heavy scripts (Levi, 2026-11-10).**
     Today's product scripts are event-driven and barely run per frame,
     but the runtime is meant to host interactive games, shader-style and
     3D rendering scripts, and the procedural audio API. Those are
     bytecode-bound, so interpreter speed still counts. The corpus
     therefore has three parts: the 16 steady scripted fixtures, recorded
     product event traces (`performAction`, `convert`), and a
     compute-heavy set (a game simulation, a software 3D or shader-style
     renderer, an audio processor block) that does not exist yet and must
     be written. The upstream bench suite stays as the interpreter
     regression guard and as the stand-in for the compute-heavy set until
     that set exists.

**Order of the goals (Levi, 2026-11-11).** Goals 1 to 3 are the goal.
Goal 4 is a budget. The plan drifted: goal 4 was written with a hard kill
criterion, the first spike proved goals 1 and 2 and missed goal 4's bar,
and the work that followed chased speed. From here:

- Progress is measured by conformance scripts passed in the new crate,
  with `unsafe` and forced panic lints at zero there.
- Speed is recorded on every PR and may not get worse within the new
  crate without a stated reason, but it does not block a lane, and it
  does not block landing the new VM behind a flag. A safe VM somewhat
  slower than today's is acceptable there. The starting point is the
  spike: about 1.6x the vendored VM on five upstream benches, estimated
  at 10 to 20% on real scripted workloads.
- Speed work (the host boundary D9, the interpreter's known out-of-loop
  costs, the scripted-frame-cost corpus) happens after the VM is correct,
  before or after cutover as Levi decides then.

Non-goals, so they do not creep in: a new scripting language or dialect,
static analysis in the device runtime (UNIV-1655 keeps it editor-side),
JIT, user-defined classes beyond today's rejection behavior, gas metering,
snapshots, or multi-tenant isolation. Deleting features of the current
profile needs explicit approval per cut, as always.

## 2. Principles

These are the part of the plan that should survive every revision.

- **One representation decision, made once, carried everywhere.** Safety
  and panic-freedom are solved together by the data layout: liveness is a
  type-level invariant, not a runtime check. A safety pass that spawns
  `expect("live handle")` sites is a failed pass, not a half-finished one.
- **Prototype, learn, build.** Every load-bearing decision gets a throwaway
  experiment with a kill criterion written before the experiment runs. The
  spike never becomes the product; it produces numbers and a decision entry,
  then is rewritten into the real crate.
- **Grow beside, never convert in place.** The new VM is a new crate behind
  the seam in section 4. The vendored VM stays as the oracle until cutover.
  Stopping at any point costs only the time spent.
- **Tests are acceptance.** A module is done when its ported upstream tests,
  its differential tests against the old VM, and the ratchets are green.
  No review or evidence ceremony beyond that.
- **Perf is measured, never argued.** Every subsystem PR carries its bench
  numbers from the harness, median of five whole-suite passes, with the
  observed host load recorded in the result's provenance. A quiet host is
  never coming (Levi, 2026-10-31), so load is recorded, not waited for;
  the geomean is the enforced number because it held to 0.3% across
  passes under load while single benches swung 10 to 15%.
- **Ratchets only move one way.** Unsafe count, panic-lint count, conformance
  pass set, bench ceiling. A PR that moves one the wrong way needs a dated
  justification in this document.
- **Reversible until cutover.** Feature flag, old VM retained, product
  goldens unchanged. Deletion of the old VM is its own approved step.

## 3. Decision log

Each entry: the decision, the experiment that settles it, the kill
criterion, status. Dated lines append as things are learned.

### D1. Value, stack and call-frame representation (the perf decision)

- Options: tagged enum values with typed index handles; split
  struct-of-arrays stacks; NaN-boxing is out (no unsafe).
- Experiment: the Phase 1 spike. Execute real bytecode from our existing
  compiler for the hot opcode set (move, load constants, get/set table and
  field, arithmetic, compare and jump, call and return, for loops,
  namecall) on five upstream benches: `sieve`, `tictactoe`, `sha256`,
  `OOP_field_access`, `TableIteration`. Compare with `tools` harness
  numbers for C++ and the vendored VM.
- Kill: worse than 1.3x C++ geomean on those five with no identified,
  costed path down. Also killed if the design needs a per-access generation
  or bounds check inside the interpreter loop to be sound.
- Status: open.
- 2026-11-01: core spike
  (`docs/evidence/luau-safe-vm-phase1-core-spike-2026-11-01.md`). The
  safety side passes: no generation or handle check in the loop. Speed
  misses: 1.41x the vendored VM on the five benches (interleaved minimums).
- 2026-11-10: loop-kernel experiment
  (`docs/evidence/luau-safe-vm-phase1-loop-kernel-2026-11-10.md`). Best
  `#![forbid(unsafe_code)]` loop is 1.94x C++ geomean (vendored 1.24x in
  the same sitting). Typed pre-decoded instructions, masked fetch, value
  layout, register window and a register-resident hot loop move the
  geomean by 2.5% at most. The gap is in handler bodies (table get does
  1.6x C++'s machine instructions) and in the call, coroutine and
  native-call paths that leave the loop (1.9 to 2.4x). The original kill
  criterion fires.
- 2026-11-11 (Levi): **decided.** The representation is the spike's:
  tagged 16-byte values, typed index handles into per-kind arenas, a
  fixed register window per frame, chained hash part with the array part
  ported exactly (D4). The speed bar no longer gates it (see "Order of
  the goals"). What the two spikes measured as slow is recorded here as
  later speed work, not as a reason to choose differently: Lua calls and
  returns leaving the loop, coroutine switch by state swap, native calls
  off the fast path, and the table get path.
- 2026-11-10 (Levi), superseded by the line above: **reopened, not killed.** The 1.3x bar judged the
  design on workloads where bytecode is everything; on real scripted
  workloads it is about a fifth. Both spikes also copied C++ Luau's design
  and swapped pointers for checked indices, which can only add cost. D1 is
  now decided after D9, and judged on goal 4's restated metric. The
  interpreter still matters for compute-heavy scripts, so the known
  out-of-loop costs (Lua calls in the loop, coroutine switch by index,
  one-chain table get) remain work to do, not reasons to stop. New kill:
  a design that is slower than today's engine on scripted frame cost for
  the fixed corpus with no costed path down, or that needs a per-access
  liveness check in the loop.

### D2. Heap and handle model (the reentrancy decision)

- Options: (a) branded-lifetime arena, mutation never overlapping
  collection, gc-arena style; (b) generational-index arena with inline
  tri-color marks and ages, ruau style; (c) typed arenas per object kind
  with indices valid for a GC epoch.
- Experiment: the reentrancy sequence. A table write fires a metamethod,
  which calls host Rust, which calls back into the VM, which allocates
  enough to trigger collection, then returns through every layer. Plus a
  suspended coroutine holding a table across that sequence. Must compile
  without `unsafe` and run without a liveness check on the hot path.
- Kill: any option that needs a runtime liveness check in the interpreter
  loop, or an `unsafe` escape, to make the sequence sound.
- Status: open. Leaning (c) with (a)'s invariant; (b) is what ruau has and
  costs it 9x.
- 2026-11-11: **decided: (c) with (a)'s invariant**, as the spike built
  it. This is the answer to "can the safe ideas be done faster than
  ruau": ruau checks a generation on every access (option b, 9.6x C++);
  here liveness holds by structure, with collection only at points where
  no code holds a reference mid-operation, so there is no per-access
  check (about 1.9x C++ on the five benches before any speed work).
- 2026-11-01: **passes** in the core spike. The full reentrancy sequence
  runs under `#![forbid(unsafe_code)]` with no liveness check anywhere,
  including collection inside a host callback fired from `__newindex` and
  a table held only by a suspended coroutine.

### D3. Error and limits model

- Decision: every fallible VM operation returns `Result`; limits (call
  depth 20000, host-call depth 200, host stack slots 8000, meta chain 100,
  bytecode size) are explicit fields on one `Limits` struct, like ruau's.
- Experiment: the abort-profile probe extended to errors raised inside sort
  comparators, `gsub` callbacks, error handlers, `__index` chains at the
  depth limit, and out-of-memory on table resize.
- Kill: none expected; the vendored VM's `luaD_throw` already returns
  `Err`, so this is confirmation, not discovery.
- Status: open.
- 2026-10-31: the seam inventory found that the audio processor engine
  (`vm/luau_audio_processor_engine.rs`) is a second raw VM instance with
  its own budgeted allocator, out-of-memory flag, deadline interrupt and a
  GC step per rendered block, declared `Send` by hand. So the limits model
  must include a memory budget with an allocation-failure path, and the VM
  needs a sendable isolated mode. The product also enforces 100,000
  safepoints per cycle, so interrupt *placement* is part of the contract.
- 2026-11-01: **confirmed** in the core spike for recursion depth, host
  re-entry depth, metatable cycles, errors inside host callbacks, sort
  comparators and `gsub` callbacks, yields across a host frame, and NaN
  and nil keys. Not yet tested: the memory budget with its
  allocation-failure path, and the audio engine's isolated mode.

### D4. Table layout

- Question: must the array/hash split and rehash policy match C++ exactly
  for the length operator and `next` ordering to pass conformance?
- Experiment: run `tables`, `nextvar`-style and `#` assertions from the
  conformance corpus against a prototype table with a different split.
- Kill: not a kill; the answer only changes how much freedom D1 has.
- 2026-10-31: settled (`docs/evidence/luau-safe-vm-d4-table-layout-2026-10-31.md`,
  nine mutated C++ variants against all 305 conformance cases plus a safe
  Rust prototype): port the array part exactly (computesizes/adjustasize
  sizing, `table.create`/`table.clear` capacity, `luaH_getn`'s last-slot
  fast path and Khuong-Morin search; the `aboundary` cache is unobservable
  and free), since 11 conformance sites pin it. The hash part (function,
  collision scheme, capacity) is free for D1, at the cost of 3 hash-order
  sites (basic.luau:249-250, 309-323; tables.luau:797-821) carried as
  assertion-level expected-fails, with `next` tolerating dead keys and
  deletion never moving entries. Array growth must stay geometric. Two
  product sites follow hash order (module load order in
  `vm/native_registration.rs`, header order in `vm/lua_scriptnet.rs`);
  sorting dependency names fixes the first regardless of VM.

### D5. The Rive host surface

- Not an experiment, an inventory (section 4). The product crate reaches
  past the handle API into raw VM symbols and `exec_raw` sites. Each is
  either lifted into the new safe embedding API or re-implemented on it;
  none survives as a raw escape hatch.
- 2026-10-31: inventory and dispositions done in `docs/luau-safe-vm-seam.md`.
  Corrected counts: 22 `exec_raw` sites (20 product), 75 distinct raw
  symbols (73 in product code across 13 files; 24 of them only in the
  audio engine). 36 product hatch rows: 22 lift (collapsing to six API
  additions), 10 reimplement, 4 drop; 6 test-only rows, all drop. Things
  the plan had treated as contracts with no product caller: `call_async`
  and its thread-data patch, non-null thread data, atom reads (all four
  namecall sites compare strings), `gc_collect` (tests only). The handle
  API also leaks into `crates/nuxie` and `crates/nux-capi` via a `pub use`
  and `ScriptVm::lua()`.

### D6. What is taken from ruau, what is rejected

- Take: the arena skeleton (tri-color marks and young/old ages inline per
  slot, write barrier, remembered set) as a reference design; the explicit
  `Limits` struct; the validated bytecode load mode (D7); the conformance
  scope file format (`required` / `permanent-non-goal <reason>`).
- Reject: gas metering on the hot path, the deterministic ambient clock,
  the async driver, the absence of userdata and `newproxy`, merge sort in
  place of introsort, its `Limits` profile (varargs and unpack caps differ
  from upstream, see the bake-off).
- Status: decided 2026-10-31, revisit if D2 picks (b).

### D7. Bytecode trust boundary

- Decision: the device loads bytecode produced by the editor, so the VM
  validates on load (register and constant indices, jump targets, upvalue
  indices, proto bounds) and the interpreter then indexes without checks
  by construction. Upstream C++ trusts the compiler; we do not.
- Experiment: the `luau-fuzz` style corpus from luau-rs plus mutated
  bytecode from our own fixtures; every rejection is an error, never a
  panic.
- Status: decided in principle 2026-10-31; verifier design is part of
  Phase 2.
- 2026-10-31: the seam inventory shows the editor emits bytecode v6 and
  v7 only; v11 (call feedback) comes from this repo's seven build-time
  helper chunks compiled with all flags on, and versions 7 through 13 are
  upstream Luau versions, not Rive extensions. The Rive-specific parts are
  the fastcall builtin IDs and the Mat4 direct-field path, which is VM-side
  dispatch. Recompiling the helper chunks with flags off lets the new VM
  start at v7.

### D8. Error text and chunk names follow the Rive profile

- The vendored VM renders the raw chunk source in error prefixes
  (`=basic.luau:39:` where upstream prints `basic.luau:39:`, and `":: "`
  when there is no Lua frame); `docs/luau-fork.md` rung 9 records this as
  deliberate Rive behavior. Goal 3 is parity with today's engine, so the
  conformance scope files these four scripts (`basic`, `closure`, `pcall`,
  `tmerror`) as permanent non-goals rather than harness limits. Revisit
  only if the product's error-text consumers (seam doc section 3) want the
  upstream form.
- Status: decided 2026-10-31 by the scope file; Levi to confirm.

### D9. The host boundary (speed work, after correctness)

- 2026-11-11 (Levi): no longer first. The new crate implements the
  existing seam (section 4) as it is, so `nuxie-scripting` and the
  differential tests work unchanged. The boundary redesign below is the
  largest known speed lever and is taken up after Phase 2's exit. The
  three fixes to today's bridge (branch `claude/luau-bridge-costs-0bad08`)
  are parked once verified; they are not part of this plan's goal.

- Why first: on real scripted workloads the boundary is the largest cost.
  Scripts make one host call per about 10 bytecode ops (upstream benches:
  under 2 per 1000). A host method costs 2,000 to 5,000 instructions per
  call, about half of it marshalling; entering a callback costs 3,400 to
  7,300. About 70% of callback instructions ran with inputs and outputs
  identical to the previous invocation.
- Options to design and measure: host methods dispatched by the VM from
  static per-type method tables with no lookup crossing; a calling
  convention that reads arguments from registers and writes results back
  without an intermediate collection; vectors and matrices as VM value
  types with VM-side operations; draw and path commands appended to a
  buffer the host consumes after the callback; host properties read from
  columns; host objects that publish change stamps so an unchanged
  callback can be skipped.
- Experiment: a boundary spike on the spike VM, measured with the
  characterization counters on the 16 steady workloads' host-call mix.
  Targets to set before it runs: instructions per host call and per
  callback entry, each against today's figures.
- Kill: a boundary that cannot get the common host call under a few
  hundred instructions without `unsafe`, or that changes script-visible
  behavior.
- Refuted by the same data, so not pursued as speed work: a typed fast
  tier as the primary lever (no script carries type annotations; bytecode
  is at most 21%) and frame-scoped arenas (GC is 0.5%). Sites are almost
  perfectly monomorphic, so type-feedback specialization remains an
  option for the compute-heavy set later.
- Status: open. In the meantime three boundary costs are being fixed in
  today's bridge as separate work (Rust `__index` method lookup on
  `Path`, `Vector.xy` and `Mat2D` constructors as host calls, scoped
  userdata rebuilding metatables per scope).

## 4. The seam

The new VM must stand behind exactly what the product uses today, so the
product does not change at cutover.

1. **Bytecode in.** Luau bytecode from the vendored safe compiler, versions
   3 through 6 plus Rive 7 through 11 (call feedback, integer builtins,
   userdata direct access). Version admission as today.
2. **Handle API out.** The `luaur-rt` surface `nuxie-scripting` calls (by
   frequency, 2026-10-31): `globals`, `create_userdata`, `load`,
   `create_function`, `create_table`, `set_app_data`, `gc_collect`,
   `create_string`, `set_named_registry_value`, `create_buffer`,
   `current_thread`, registry values, `load_bytecode`, `set_timeout_ms`,
   `set_log_sink`, `call_async` and the async coroutine thread-data
   inheritance patch, `state_identity`, module threads.
3. **Raw escape hatches to retire.** 53 distinct `luaur_vm` symbols across
   12 files in `crates/nuxie-scripting/src` (`vm.rs`, `vm/lua_*.rs`,
   `gpu_canvas_ore/*`) and 19 `exec_raw` sites. The heaviest: metatable
   get/set, `gettop`/`settop`, namecall atoms, thread data, userdata
   tagging and direct-field registration, `luau_load`, vector push. Each
   one becomes a line in a mapping table in this section as it is resolved.
4. **Host callbacks.** Interrupt (used for timeouts, fired at a fixed
   safepoint cadence the product counts on), useratom, userthread, debug
   break, and a budgeted allocator with an out-of-memory path (the audio
   engine depends on it; the earlier "not needed" was wrong).

Inventory, per-site dispositions and the proposed API additions:
[docs/luau-safe-vm-seam.md](luau-safe-vm-seam.md) (D5, 2026-10-31).
5. **Observable streams.** Whatever the scripted golden compare and the
   product goldens record. These are the differential oracle at cutover.

## 5. Workflow

The loop, for every piece of work from the spike onward:

1. Pick the next item from the phase list. If it touches a decision that is
   still open, the item is the experiment, not the implementation.
2. Write the kill criterion or the acceptance test first. For a subsystem
   that means: the ported upstream tests for it, and the differential test
   that runs the same bytecode through old and new VM.
3. Implement in a worktree under `~/dev/worktrees/` (never `/tmp`), one
   writer per worktree, Bazel through `runtime.py` and `make`.
4. Run the ratchets (section 7) locally. Run the bench harness on a quiet
   host, median of five, before claiming a perf number.
5. Open one PR per item. The PR body carries the numbers and any ratchet
   movement. `codex review --base origin/main` locally until clean.
6. Append to this document: decision status lines, seam mapping rows,
   phase exit criteria met. If something learned changes a principle,
   that is a conversation, not a PR.

Parallel lanes are fine once the core exists, one subsystem per lane, each
with its own oracle; the battery runs once on the merged result.

## 6. Phases

Timeboxes are honest guesses to be rewritten. Exit criteria are not.

### Phase 0: decisions and ratchets (about one week)

- Turn the bake-off runners into a committed tool: conformance runner over
  the upstream corpus with our own scope file, bench harness with ceilings,
  both runnable against the vendored VM and the new crate.
- Land the ratchet counters as make targets: `unsafe` sites per crate,
  forced clippy panic lints, conformance pass set, bench ceilings.
- Settle D4 (table layout) and write the D5 mapping table skeleton.
- Exit: the four ratchets report today's numbers for the vendored VM on
  `main`, and this document has an entry for every decision above.
- 2026-10-31: landed on the branch (merge to `main` pending). Tools:
  `make luau-conformance` (55 upstream scripts, scope
  `tests/luau-conformance/scope.txt`: 28 required, 3 expected-fail, 24
  permanent-non-goal, exit 0), `make luau-bench` /
  `make luau-bench-check` (87 benches, baseline
  `tests/luau-bench/baseline-luaur.json`), `make ratchet-unsafe`
  (luaur-vm 1431, six luaur crates 3918), `make ratchet-panic-lints`
  (Bazel clippy aspect on the pinned toolchain: luaur-vm 1292, six crates
  3517), `make luau-vm-ratchets` for all of them. D4 and D5 settled, D8
  added. Open from this phase: the bench per-bench ceiling policy
  (section 7).

### Phase 1: the core spike (two to three weeks, throwaway)

- One crate, no library beyond what the five benches need, executing real
  bytecode from our compiler. Settles D1 and D2; confirms D3.
- Exit: D1 and D2 have status lines with numbers. Either both pass their
  kill criteria and Phase 2 starts, or the plan is rewritten here with
  what was learned. The spike code is not merged.
- 2026-11-11: **Phase 1 is closed.** D1, D2 and D3 have their lines. The
  ordering in the next line is superseded: Phase 2 starts now, and the
  boundary spike, the compute-heavy corpus and the interpreter speed work
  move to "Phase 4: speed" below.
- 2026-11-10 (superseded): rewritten here as that exit requires. D2 and D3 are done.
  What remains of Phase 1, in order: (1) the D9 boundary spike; (2) the
  compute-heavy corpus and the scripted-frame-cost harness for goal 4;
  (3) D1 revisited on that metric, with the three known out-of-loop
  interpreter costs addressed. Phase 2 starts when D9 and D1 have passing
  status lines. The spike branches stay unmerged; only their evidence
  docs are in the tree.

### Phase 2: the real crate, subsystem lanes (weeks each)

Order follows heat and dependency; each lane ends in one PR with ported
upstream tests, differential tests against the vendored VM, and ratchets.

1. Values, stack, frames, interpreter loop, bytecode verifier (D7). This
   is the spike rewritten properly. Acceptance (2026-11-11): the first
   conformance scripts pass through the Phase 0 runner, `unsafe` and
   forced panic lints are zero in the crate, and the bench numbers are
   recorded. The perf ceiling is not acceptance.
2. Tables and strings (interning, the layout chosen in D4, `#`, `next`).
3. Collector: incremental mark and sweep, weak tables in k, v, kv and s
   modes, userdata destructors, `collectgarbage` semantics that the GC rung
   tests pin, write barriers that cover every registry setter.
4. Coroutines and error flow: stackless resume, yields across pcall, the
   limit counters, interrupts and `lua_break`, the abort-profile probe.
5. Libraries: base, string with patterns and pack, table with introsort,
   math, bit32, utf8, os subset, buffer, vector, coroutine, debug subset.
6. Embedding API: the handle surface in section 4, userdata with namecall
   dispatch, module threads, app data. (2026-11-11: useratom and
   `call_async` are dropped; the seam inventory found no product caller.)
7. Rive bytecode extensions and the D5 raw-hatch retirement in
   `nuxie-scripting`.
- Exit: the new crate passes our conformance scope file (all 28 required
  scripts, the 3 expected-fails resolved or re-justified) and the
  differential battery, with zero `unsafe` and zero forced panic lints;
  `nuxie-scripting` builds against it with zero raw hatches. Bench and
  scripted-frame-cost numbers are reported, not gating (2026-11-11).

How Phase 2 runs (2026-11-11):

- **The scoreboard is the conformance scope file.** `tools/luau-conformance`
  gains a second VM behind its harness module, and the new crate gets its
  own scope file that starts with every script `expected-fail` and moves
  them to `required` as lanes land. The count of required scripts is the
  progress number.
- **Lane 1 goes alone.** It creates the crate (`#![forbid(unsafe_code)]`,
  the forced panic lints denied at the crate root so a new site fails the
  build), the differential harness that runs the same bytecode through
  both VMs, and the second-VM hookup in the conformance and bench tools.
  It is not a copy of the spike: the spike is read, then rewritten.
- **Lanes 2 to 5 run in parallel** after lane 1 merges, one worktree
  each, each owning its upstream scripts: tables and strings (`tables`,
  `clear`, `move`, `strings`, `strconv`, `stringinterp`), collector
  (`gc`), coroutines and errors (`coroutine`, `pcall`, `errors`,
  `tmerror`, `closure`, `calls`, `vararg`), libraries (`math`, `bitwise`,
  `utf8`, `pm`, `sort`, `tpack`, `buffers`, `vector`, `vector_library`).
- **Lanes 6 and 7 follow**: the embedding API against the seam, then the
  Rive bytecode extensions and the raw-hatch retirement.
- Each lane ends in one PR; the vendored VM stays the product's VM until
  Phase 3.
- 2026-11-11: **lane 1 is written**, on local branch
  `claude/luau-vm-lane1-core-0b14a3` (not merged; evidence
  `docs/evidence/luau-safe-vm-phase2-lane1-2026-11-11.md` on that branch).
  `crates/nuxie-luau-vm` forbids `unsafe` and denies the forced panic
  lints, at zero in both ratchets. The differential corpus (26 cases, 4
  compiler profiles, every admitted opcode) agrees with the vendored VM.
  `scope-nuxie.txt` requires 14 of 55 upstream scripts. Recorded speed:
  4.45x the vendored VM on the 59 benches both ran, under uneven host
  load; not a gate.
- 2026-11-11: lane ownership as lane 1's run found it, replacing the
  lists above where they differ:
  - lane 2 (tables and strings): `tables`, `clear`, `move`, `sort`,
    `calls`, `vararg`, `types`, `strings` (after `newproxy`). `strconv`
    and `stringinterp` already pass. Lane 2 adds `table.create`, `clone`,
    `move`, `find` and `getn` first: they gate 25 of the 84 benches.
  - lane 3 (collector): `gc`, weak tables, and one accounting point per
    allocation site with the memory-budget suite (seed: `limit_tests` in
    `tools/luau-vm-diff`). Eight review rounds found budget bypasses one
    at a time.
  - lane 4 (coroutines and errors): `coroutine`, `cyield`,
    `ndebug_upvalues`, `pcall`, `closure` (after lane 3's weak tables),
    and stack-safe re-entry for every path, not only `pcall`.
  - lane 5 (libraries): `math`, `bitwise`, `utf8`, `tpack`, `buffers`,
    `vector`, `vector_library`, `debug`, `datetime`, `errors`, `events`,
    `iter`, `native_integer_spills`. (Superseded: the Rive profile has no
    `newproxy`, so `errors`, `events`, `iter` and `strings`, and `gc`'s
    `newproxy` blocks, are permanent non-goals, as in `scope.txt`; `vector`
    needs host helpers and passed in lane 6.)
  - harness: `literals` and `pm` need a byte-source compile entry; they
    fail the same way on the vendored VM.
- 2026-11-11: ~~open risk from lane 1~~ **closed by lane 4.** Metamethods,
  `pcall`, sort comparators, `gsub` callbacks and host callbacks re-entered
  the interpreter through Rust recursion. Lane 4 runs all of them except
  host callbacks as VM frames with continuations; host callbacks that call
  back into the VM, `string.format`'s `%*` and `HostCtx::tostring_value`
  still recurse, bounded by `Limits::max_reentry` (16), which fits a
  1 MiB stack (`reentry_reaches_its_limits_on_a_one_mib_stack`).
- 2026-11-11: **lanes 2 to 5 are on `main`**, evidence in
  `docs/evidence/luau-safe-vm-phase2-lane{2,3,4,5}-2026-11-11.md`. Lane 2
  completed the table library and the string library but for `pack` (19
  required). Lane 3 added
  the incremental collector, weak tables and the single accounting point
  (`budget.rs`). Lane 4 added coroutines, error flow and frame-based
  re-entry. Lane 5 added the remaining libraries, integers and buffers
  (31 required). `cyield` waited on lane 6's host continuations, not lane
  4.
- 2026-11-11: **lane 6 is written** on local branch
  `claude/luau-vm-lane6-embedding-f599cf` (not pushed; evidence
  `docs/evidence/luau-safe-vm-phase2-lane6-2026-11-11.md`). The crate
  gains the embedding API: rooted VM-scoped handles, typed conversions,
  host functions with continuations, yields and error values, userdata
  types with per-type and per-tag metatables, direct fields, the
  interrupt, the registry and app data, and a settable memory limit.
  `nuxie-luau-vm` stays at zero `unsafe` and zero forced panic lints.
  The conformance runner installs upstream's host helpers, so `cyield`,
  `userdata`, `udata_direct`, `native_userdata` and `vector` are
  required: **36 of 55**. A shared host surface makes the embedding API
  part of the differential corpus (0 disagreements). Not provided: a
  `Send` VM for the audio lanes (seam item 9); lane 7 keeps those on the
  vendored VM or moves them to one VM per thread.

### Phase 3: cutover

- Feature flag selecting the VM. Scripted golden compare and product
  goldens green on both. Device size and startup measured on the new VM.
- Exit: flag flipped on `main`; old VM deletion proposed as its own PR for
  approval.

### Phase 4: speed (after Phase 2's exit; before or after cutover, Levi's call)

- The host boundary (D9): static method tables, a register-based calling
  convention, vectors and matrices as VM values, command buffers, change
  stamps for skipping unchanged callbacks.
- The interpreter's measured costs: Lua calls and returns inside the loop,
  coroutine switch by index, one-chain table get.
- The scripted-frame-cost corpus (goal 4), including the compute-heavy set
  that has to be written: a game simulation, a shader or 3D style
  renderer, an audio processor block.

## 7. Ratchets

Run on every PR to the new crate, reported in the PR body:

| Ratchet | Direction | Source |
|---|---|---|
| `unsafe` sites per crate | zero in the new VM crate, never up anywhere | `make ratchet-unsafe` against `tests/ratchets/unsafe-baseline.txt` |
| forced clippy panic lints (unwrap, expect, indexing, panic, unreachable, arithmetic side effects, unwrap in result) | down to zero | `make ratchet-panic-lints` (Bazel clippy aspect, pinned toolchain) against `tests/ratchets/panic-lints-baseline.txt` |
| conformance pass set | grows, never shrinks | `make luau-conformance`, `tests/luau-conformance/scope.txt` over the vendored 0.734 corpus |
| bench geomean | vendored VM: at or under `tests/luau-bench/baseline-luaur.json`, ceiling 1.10 enforced. New crate (2026-11-11): reported on every PR against its own previous number, not enforced against the vendored baseline until Phase 4 | `make luau-bench-check`, median of five whole-suite passes, load recorded |
| abort-profile probe | green | `tools/luaur-rt-test-harness`, panic=abort |

Per-bench ceiling policy (open, Levi's call): on this host a 1.10 per-bench
ceiling fails on unchanged code (two benches at 1.11x and 1.15x while the
geomean sat at 1.003x). Options: loosen per-bench to about 1.20, or report
per-bench movement without enforcing it and enforce the geomean only.
Until decided, `make luau-bench-check` can flap on single benches.

## 8. Risks and what would change the plan

- **D1 fails.** (2026-11-11: no longer a live risk in this form. D1 is
  decided and speed is a budget. The risk that remains is that Phase 4
  cannot close the gap to today's engine, in which case cutover is
  Levi's decision on the numbers, not an automatic fallback.) No safe representation gets near the ceiling. Then the plan
  becomes: harden the vendored VM in place (panic lints, verifier,
  containment) and accept `unsafe` with a security review, which is what
  luau-rs does. That is a different plan and this document would say so.
- **D2 forces unsafe.** The reentrancy sequence cannot be made sound without
  an escape hatch. Then the escape hatch is one audited module with a
  documented invariant, and the "zero unsafe" goal becomes "one audited
  module", decided by Levi, not by drift.
- **The seam is bigger than the inventory.** Expect the D5 table to grow as
  lanes hit product behavior the handle API never exposed. Growth is fine;
  new raw hatches are not.
- **Conformance is harness-bound.** Twenty-six of the fifty-five upstream
  scripts need host setup. Our scope file must say which are required with
  which setup, or the pass set is theater.
- **Perf drift under load.** Numbers taken while other lanes build are
  worthless; the bake-off's first pass showed this. Quiet host or no number.

## 9. Open questions

- Does the branded-lifetime style survive the async coroutine bridge
  (`call_async`) that the product relies on? Part of the D2 experiment.
- Which Rive bytecode features are actually emitted by the editor today
  versus merely admitted? Trim D5 to what is emitted.
- Can the editor-side `script-tools` module (UNIV-1655) share the verifier
  so bytecode is validated before it ever ships?
- Do editor-authored scripts inside the `.riv` fixtures depend on hash
  traversal order? Cheap check proposed by the D4 lane: add its remixed-hash
  switch to the vendored VM and run the scripted golden compare once.
- The editor's compile options (optimization level, vector library, which
  Rive fastcall IDs are emitted) live in nuxie-dev and were inferred, not
  observed. Confirm there before trimming D5.
- Two conformance scripts filed as permanent non-goals look reachable
  through the handle API (`tables.luau:684` via `LightUserData`,
  `gc.luau:213` via `set_memory_limit`); promote them when cheap.

## 10. Change history

- 2026-10-31: created after the bake-off; Phase 0 started; D6 and D7
  decided in principle.
- 2026-10-31 (later): Phase 0 landed by four parallel lanes (conformance
  tool, bench and ratchets, D4 experiment, D5 inventory). D4 and D5
  settled, D8 added, D3 and D7 amended from the seam inventory, perf
  ceiling restated on the fat-LTO baseline, quiet-host rule replaced by
  recorded load.
- 2026-11-10: Phase 1 spikes and the workload characterization recorded.
  Goal 4 restated as scripted frame cost with a compute-heavy set in the
  corpus; D1 reopened under that metric; D2 passes; D3 confirmed; D9 (host
  boundary) added and placed first; Phase 1 remainder rewritten.
- 2026-11-11: reoriented to the original goal (Levi). Goals 1 to 3 lead,
  goal 4 becomes a budget; D1 and D2 decided as the spike built them; D9
  moved after correctness; Phase 1 closed; Phase 2 acceptance and lane
  structure rewritten around the conformance scope file; Phase 4 (speed)
  added.
- 2026-11-11: lane 1 outcome recorded under Phase 2; lane ownership
  revised from its scope table; re-entry stack risk recorded for lane 4.
