# Safe Luau VM: the embedding core (lane 8 design)

Status: design, stage A of lane 8 (2026-11-12). Not built. Base
`origin/main` `c311d439d9`. Inputs: plan lane 8, lane 6 and 7 evidence
(lane 7 section 10 is the audio brief), the perf gap map
(`claude/luau-vm-perf-map`, sections 3.4, 4, 5), the workload
characterization (2026-11-10), and the code cited below by file and line.

## 0. The design in one page

- **`Core`** is the VM: heap, stacks, frames, interpreter, libraries,
  collector, budget, root table. It holds plain data only: no `Rc`, no
  closure, no `dyn Any`, no host payload. It is `Send`, asserted at compile
  time.
- **`Vm<H: Host = Compat>`** is `Core` plus the host's state `H` plus a
  table of host function pointers (`Vec<HostEntry<H>>`). The core never
  looks inside `H`; it reaches it only through the `Host` trait (interrupt,
  direct-field read) and the function table. `Vm<H>` is `Send` exactly
  when `H` is, because fn pointers and `Core` are.
- **A host function is a fn pointer plus a `u32` of data**, addressed by
  `(HostFnId, data)` in the native closure. Two kinds:
  - **plain** `fn(&mut Plain<'_, H>) -> Result<usize, HostError>`: may
    read and push values, allocate, raise; may not call Lua or yield. It is
    called by `CALL` directly on the argument window, with no frame (§2).
  - **full** `fn(&mut HostCtx<'_, H>) -> Result<usize, HostError>` (and an
    optional continuation `fn(&mut HostCtx<'_, H>, CallStatus)`): today's
    host function, with a frame, able to call back and yield. `HostCtx`
    holds `&mut Vm<H>`, so it can do everything the host can.
- **Where host code runs.** The interpreter and libraries are compiled
  once, on `Core`. The generic driver (`Vm<H>::execute`) calls
  `core.run_frames(&mut dispatch)`, where `dispatch` is `&mut dyn Dispatch`
  over `(&mut H, &host_fns)`, split from `self` by field. Plain functions,
  the interrupt and direct-field reads go through `dispatch` inside the
  loop. A frame running a full host function makes `run_frames` return to
  the driver, which now holds the whole `&mut Vm<H>` and calls the function
  with it, then re-enters the loop. The driver also runs `xpcall` handlers
  (§2.2), so every Rust-stack re-entry starts in host code.
- **Userdata** in the core is `(host key: u64, metatable, tag, declared
  size)`. The host owns the payloads and packs whatever it likes in the
  key. The sweep appends each freed userdata's key (and each freed native
  whose function was registered as owning its data) to a freed list that
  the host drains.
- **Roots** are generational indices `Root { vm: u32, index: u32, gen: u32 }`
  into the core's root table, released explicitly. A stale or foreign root
  reads as `None` and its operations return `Error::Host`.
- **Lane 6's API is `Vm<Compat>`**: `Compat` holds the closures
  (`Rc<HostFn>`), the `Rc<RefCell<T>>` payloads, the app data, the
  interrupt closure and direct-field getters, all keyed by the core's
  indices; one full trampoline dispatches every lane 6 closure. The default
  type parameter keeps the name `Vm` meaning what it means today
  (`HashMap::new` is the precedent), so the facade and the tools compile
  unchanged except where they reach internals.
- **The audio engine is `Vm<AudioHost>`** with plain host functions over
  its channel index, two full ones (`print`, `require`), and `AudioHost:
  Send`. It moves between threads as one value. No `unsafe`.

The sketch in the brief had `H` passed on every entry
(`vm.call(&mut host, ..)`). The code says otherwise (§3, item 5): a host
callback in lane 6 must be able to produce one `&mut Vm` (the facade's
`lend` swaps it, `crates/nuxie-scripting/src/luau/facade/state.rs:122`,
and `HostCtx: DerefMut<Target = Vm>` is how every tool callback reaches the
API). With `H` passed beside the VM, a callback would hold two borrows,
`&mut Core` and `&mut H`, that cannot be recombined into one place. Owning
`H` keeps one place and changes nothing else: the core still never sees
inside it.

## 1. Data and frequency

Product numbers are from the workload characterization
(`docs/evidence/luau-workload-characterization-2026-11-10.md`, measured on
the vendored VM, 16 steady workloads). Audio numbers are read from the
engine (`crates/nuxie-scripting/src/vm/luau_audio_processor_engine.rs`).

| Operation | Data it touches today (safe VM + facade) | Product frequency | Audio frequency |
|---|---|---|---|
| Host call (facade callback) | `step_native` (`call.rs:508`): frame push (128 B), `Wait` take/drop, `NativeFn` clone (`Rc` inc/dec). Facade trampoline (`state.rs:886`): `Weak` upgrade, args to a `Vec` with one `Rc<Marker>` handle per object arg, a second `Vec` of facade values, `lend` (two `mem::swap`s of the whole `Vm`, at least 1.8 KB each: `self.pc` is at offset `0x730`, perf map 3.4), results back through another `Vec`. | One per ~10 bytecode ops; 98.5 per 1k dispatched instructions; 1.36 M per frame in `script_layout_test`. Marshalling 1.3 to 2.4 k instructions per call, entry 3.4 to 7.3 k per callback (vendored). | A few per block: `atomicLoad`, `receive`, `readInto`, `publish`. |
| Userdata method call | `NAMECALL`: metatable lookup, `__index` or `__namecall`, then a host call as above. | 46% of authored field accesses are on userdata, one type per site. | None (no userdata). |
| Field read on userdata | `__index` builtin dispatcher (`embed.rs:957`) requesting a getter call (a second frame), or the direct-field path (`embed.rs:549`: `HashMap` lookup by key bytes, `Rc<dyn Fn>` call, `RefCell` borrow). | Most of the 46% above; Mat4's `m11`..`m44` use direct fields. | `pcm.frame`, `blob.size`: per block at most. |
| Handle create / release | `HandleTable::insert` allocates an `Rc<Marker>`; release is a refcount drop; dead slots are found by scanning `Weak`s at every collection and when the table doubles (`handle.rs:86`). | One per object argument and result of every host call. | None per block: four registry refs for the VM's lifetime. |
| Userdata create / free | `Userdata::new` allocates `Rc<RefCell<T>>`; the sweep drops it and `T::drop` runs inside the collector. | 7.6 scoped userdata per frame. | None. |
| Interrupt check | `Option` test; when set, `Rc<RefCell<Box<dyn FnMut>>>` borrow and call (`vm.rs:62`), also on every pattern-match step through a cloned `Rc` (`libs/string.rs:314`). | Installed always (`nuxie-scripting/src/vm.rs:2247`): every `CALL`, `RETURN`, back edge. | Every back edge of the per-sample loop; reads `Instant::now()` each time. |
| Builtin call (for contrast) | Same frame protocol as a host call; `FASTCALL` falls through for all but 4 builtins (`libs/fast.rs`), +830 to +860 instructions over luaur per call (perf map 2.2). | Vector and math builtins. | **Per sample and channel**: `buffer.writef32` (every processor in the engine's tests, e.g. `:1720`). |

Two things follow and shape the rest:

1. The audio block loop is bound by **builtin** calls, not host calls. A
   1024-frame stereo block is 2048 `buffer.writef32` calls, each paying the
   full native protocol, where luaur runs a fast builtin.
2. The product's host-call cost on the safe VM is mostly the **facade**
   (vectors, handles, `lend`), which exists because the bindings are written
   against `luaur-rt`'s self-sufficient handles. The core does not remove
   it; only bindings written against plain host functions would (D9).

## 2. Work removed, not made cheaper

### 2.1 The native calling convention

Today every native runs as a frame stepped through `Wait` / `Request` /
`Continuation` (`call.rs:508-583`), because some natives yield or call Lua.
The design splits natives by what they can do and gives the common kind no
protocol at all.

**Plain natives (no yield, no call into Lua).** `CALL` sees a plain native
and, inside the dispatch loop:

1. checks call depth as if a frame were pushed (`frames.len() + 2`, same
   overflow point as today) and `ensure_stack(top + MIN_STACK)`;
2. writes a 16-byte **virtual native record** `{closure, base, nargs}` in a
   `Core` field (for `luaL_argerror` names and `builtin_upval`, which today
   read `frames.last()`, `libs/mod.rs:204`);
3. calls the function with `Args { base, nargs }`;
4. clears the record; if a `Request` was set, that is an internal error
   (a plain native cannot request);
5. moves the top `n` results to `ra`, pads to `nresults`, sets `top`, runs
   `safepoint()` and keeps dispatching in place.

No `Frame`, no `Wait`, no `deliver`, no `continue 'frame`. Errors raised by
a plain native **materialize** the frame on the cold path (push a native
`Frame` from the record before returning the `Throw`), so `xpcall`
handlers, `debug.traceback` and `inspect_stack` see `[C]` at level 0 exactly
as now. Nothing else can observe the missing frame: a plain native cannot
run Lua, yield or reach the host's interrupt closure with VM access, and the
callee stays in its stack slot below the arguments, which roots it as a
frame's `closure` field does today.

**Resumable natives** (builtins that request: `pcall`, `xpcall`,
`tostring`, `print`, `table.sort`, `string.gsub`, `coroutine.*`, the
userdata dispatchers; and full host functions) keep today's frame protocol
unchanged.

**One convention or two?** One convention (frameless over the argument
window for plain, frame plus continuation for resumable) and two
signatures: builtins stay `fn(&mut Core, Args) -> VmResult<usize>`, host
functions are typed over `H`. Making builtins generic over `H` to share one
signature would monomorphize the libraries per host; erasing `H` into every
builtin adds a parameter 300 functions ignore. The kind is a field of the
native closure (`NativeFn::{Builtin, BuiltinK, Plain, Full}`, `Copy`), so
`CALL` decides with one match.

The pattern matcher is the only builtin that needs the host (the
interrupt fires on every match step, seam 3.1). It gets the erased host
through a `BuiltinHost` kind used by `find`, `match`, `gmatch` and `gsub`,
instead of the `Rc` slot it clones today.

**Whether builtins move in this lane is open question 1.** The mechanism
is the same code path; moving a builtin is a one-word classification plus
its differential cases. Plain host functions alone do not change any
product path and change only a handful of audio calls per block.

### 2.2 Other work that disappears

- **`NativeFn` clone per native call** (`call.rs:523`, an `Rc` inc/dec for
  host functions): `NativeFn` becomes `Copy` (fn pointer or
  `(HostFnId, data)`); builtin upvalues move from `Rc<[Value]>` in the
  `NativeFn` to a `Box<[Value]>` in the `NativeClosure`.
- **Closure capture as the way to bind data**: the audio engine binds 15
  methods per channel by upvalue (`:1194`), lane 6 by capturing closures.
  Both become the `data: u32` of the native.
- **The `Weak` scan of the handle table** at every collection
  (`handle.rs:130-153`): core roots are released explicitly; lane 6's
  drop-released handles push their index to a release queue that `Compat`
  drains at its own entry and exit points.
- **`T::drop` inside the sweep**: the sweep only appends keys.
- **Core-side recursion with a host callee.** `call_value` is reached from
  core code in two places whose callee can be a host function: the `xpcall`
  handler (`call.rs:743`, called from `catch`) and `tostring_value` from
  `string.format`'s `%*` and the field dispatcher's error text
  (`libs/string.rs:1211`, `embed.rs:996`). With host dispatch at the
  driver (§0) these move: `catch` is split so the driver runs the handler
  between "find the catcher" and "unwind"; `%*` and the dispatcher error
  become continuations, as `gsub` already is. After that only host code
  re-enters the interpreter on the Rust stack, and `Limits::max_reentry`
  bounds exactly that.
- **The audio engine's counting allocator** (`allocate`, `:723`) and its
  `bytes`/`oom`/`memory_limit` fields: the core budget is the one
  accounting point (`set_memory_limit`, `memory_exhausted`).
- **The audio engine's raw stack scraping** (`diagnostic`, `:778`) and its
  registry refs: errors come back as `Error`, refs become `Root`s.

## 3. Alternatives rejected

1. **A `Send` bound on today's API** (`HostFn: Send`, payload `T: Send`,
   `Arc` for every shared cell). The product's callbacks capture `Lua`
   (`Rc<Shared>`) and `Rc<Cell<..>>` state (`vm.rs:2241-2255`); they would
   all have to become `Arc`/`Mutex`, paying atomic refcounts on every
   handle in a VM that never leaves its thread. It also keeps closures in
   the VM, which is what made it `!Send`.
2. **A second `Arc`/`Mutex` flavour** (a `SendVm`, or the VM generic over a
   pointer family). Two copies of the embedding layer, or the whole crate
   monomorphized twice; a `Mutex` around the interrupt on the realtime
   thread; nothing deleted.
3. **`&mut dyn Any` as host state.** Every host function would downcast on
   every call and could fail at run time with a type error the compiler
   could have caught. The design erases `H` at exactly one boundary,
   `&mut dyn Dispatch` passed into the interpreter, whose implementation is
   typed code generated per `H`; there is no failing downcast anywhere.
4. **The whole interpreter generic over `H`.** Simplest to write, but the
   interpreter and libraries (most of the crate's 19 k lines) would be
   compiled once for `Compat` and once for `AudioHost` into the same
   `nuxie-scripting` binary, which matters for the wasm and device size
   Phase 3 measures. Only the driver and glue are generic here.
5. **`H` passed on every entry** (the brief's sketch). Rejected for the
   reason in §0: two borrows a callback cannot recombine into the one
   `&mut Vm` that lane 6's `HostCtx` deref and the facade's `lend` need.
6. **Handles released on drop, in the core.** Drop-release needs the
   handle to reach the VM's table without the VM: `Rc`/`Weak` (not `Send`)
   or a channel (an allocation and atomics per handle). The core uses
   explicit release, which the audio engine needs (four roots for the VM's
   lifetime) and which is free. Drop-release stays where it is wanted, in
   `Compat`, built on explicit release plus a release queue.
7. **Scoped release** (roots tied to a guard borrowing the VM). A guard
   that borrows `&mut Vm` blocks every other use of the VM while it lives;
   one that does not borrow needs shared ownership again. Explicit release
   plus generation checks is simpler and makes misuse an error, not a leak
   or a wrong object.
8. **Protos and strings by index now** (perf map redesigns 2 and 6). These
   delete the `Rc`s outright instead of turning them into `Arc`s (§5.1),
   but they are interpreter and string-store redesigns with their own
   gates. This lane takes the smallest step that makes `Core` `Send`.
9. **Building the VM on the audio thread** (rejected by Levi, 2026-11-11)
   and `unsafe impl Send` (forbidden).

## 4. What gets deleted

From `nuxie-luau-vm` (lane 6's layer and the core):

| Deleted | Where today | Replaced by |
|---|---|---|
| `NativeFn::Host(Rc<HostFn>)`, `HostK(Rc<HostFn>, Rc<HostContFn>)`, `BuiltinWith(_, Rc<[Value]>)` and the clone in `step_native` | `native.rs:49`, `call.rs:523` | `Copy` `NativeFn` with `(HostFnId, data)`; upvalues in `NativeClosure` |
| `InterruptSlot` (`Rc<RefCell<Box<InterruptFn>>>`), `fire_interrupt`, the matcher's `Rc` clone | `vm.rs:56-67`, `libs/string.rs:181,596` | `Host::interrupt(&mut self)` and a `Core` flag that skips it when unset |
| `Vm::app_data: HashMap<TypeId, Rc<dyn Any>>` | `vm.rs:219` | a field of `Compat` (lane 6 API unchanged) |
| `Vm::direct_fields: Vec<Option<Rc<DirectFields>>>` | `vm.rs:226` | per-tag bit in `Core`, `Host::direct_field(tag, key, bytes)`; getters in `Compat` |
| `Vm::type_metas: HashMap<TypeId, TableId>` | `vm.rs:221` | `Compat` (core keeps per-tag metatables only) |
| `Userdata.payload: Option<Rc<dyn Any>>`, `type_id`, `type_name` | `userdata.rs:47` | `key: u64`; type checks in `Compat` |
| `HandleTable`'s `Weak` owners, `reclaim`, `reclaim_at`, `live()` scan | `handle.rs:52-160` | core `Roots` (generational, explicit) + `Compat` release queue |
| `HostCont { k: Option<Rc<HostContFn>> }` | `embed.rs:1023` | `HostCont { k: Option<HostFnId>, .. }` (plain data, `Send`) |
| `call_host` / `run_host` taking closures | `call.rs:589-623` | table dispatch at the driver |
| `error_object: Option<handle::Value>` | `vm.rs:232` | `Option<Root>` |
| core `call_value` from `catch`, `string.format`, the dispatcher | `call.rs:743`, `libs/string.rs:1211`, `embed.rs:996` | driver-level handler; continuations |

From the audio engine (under `nuxie-vm`; the default keeps `RawProcessor`
untouched, since old-VM code is not deleted in this lane): the 6 `luaur_vm`
imports, `RawProcessor`'s raw state pointer and `Drop` with `lua_close`,
`unsafe impl Send`, `allocate`, `context()`, `diagnostic`, the `c_int`
registry refs (`object`, `process`, `output`, `stop_error`, `budget_error`,
`loaded`, `channel_refs`), `channel_objects` keyed by `lua_topointer`
(keyed by table root identity instead), and all 78 `unsafe` occurrences in
the code that builds under `nuxie-vm`.

What is added: `Core`/`Vm<H>` split, `Host`/`Dispatch`, `Roots`, the freed
list, the plain kind with its frameless path, `Compat`, `AudioHost`. The
additions are mostly the old code moved behind a type boundary; the net
line count of `nuxie-luau-vm` should drop (lane 6's `Rc` plumbing), and the
engine's safe processor should be shorter than `RawProcessor`.

## 5. Safety and panic-freedom

### 5.1 Making `Core` `Send`

`Core` holds `Rc` in four places that are not the embedding layer
(`grep` of the crate): `Rc<Proto>` (heap arena, closures, frames,
`heap.rs:216,417`, `interp.rs:565`), interned string bytes `Rc<[u8]>`
shared between the arena and the interner key and cloned by `str_rc` so
string builtins can hold bytes across allocation (`heap.rs:259,423,743`),
native names `Rc<str>`, and builtin upvalues `Rc<[Value]>`. Names become
`Box<str>`, upvalues `Box<[Value]>`. Protos and string bytes become `Arc`:
they are immutable and shared, and `Arc` is the honest type until
redesigns 2 and 6 remove the sharing. `Box<dyn Continuation>` becomes
`Box<dyn Continuation + Send>`; every implementation holds plain data once
`HostCont` holds ids. The budget's `Cell<bool>` is `Send`. A
`const _: () = assert_send::<Core>()` and one for `Vm<AudioHost>` make it
a build failure to regress; nothing asserts it for `Vm<Compat>`, which is
`!Send` and need not be.

### 5.2 Indices

- **Stale root** (released, slot reused): the generation differs, `get`
  returns `None`, operations return `Error::Host("a stale root")`.
- **Foreign root** (another VM's): roots carry the VM's id (today's
  `next_vm_id`, `handle.rs:35`); a mismatch is `Error::Host`, never a
  wrong object.
- **Host function id out of range** (a `HostFnId` from another VM): checked
  at closure creation, so the interpreter's lookup cannot miss; it still
  uses `get` and raises an internal error if it does.
- **Host store out of step with the VM** (a host that frees a payload
  before the VM freed the userdata): the core never dereferences a host key,
  so the worst case is the host's own lookup failing (`Compat` keys carry a
  generation and the payload's `TypeId`; a mismatch is the same
  `UserDataTypeMismatch` error as today). A key appears on the freed list
  exactly once: the sweep pushes it when it frees the arena slot, and the
  slot's key is cleared in the same step.

### 5.3 Re-entry and host state

A full host function gets `&mut HostCtx<'_, H>`, which holds the only
`&mut Vm<H>`. Its host state is `ctx.host_mut()`, a borrow of `ctx`; calling
back (`ctx.call`, `ctx.call_yieldable`) needs `&mut ctx`, so the borrow
checker forbids holding host state across a re-entry. No `RefCell` is
needed for `H`'s own fields. `Compat` keeps closures in `Rc` and clones the
running one out before calling it, as today, so a re-entrant call that
replaces closures cannot free the running one. A plain function's context
has no call, yield or `inspect_stack` method: it cannot re-enter by type.
The interrupt hook gets `&mut H` only, never the VM (as today, `vm.rs:58`).

### 5.4 A host function that fails midway

The `Throw` propagates to the catcher, which unwinds the stack and frames
to its own level (unchanged); for a plain function the materialized frame
(§2.1) is unwound with the rest. Host-state changes made before the error
are not rolled back, as today and as in C++. Full host functions keep
today's rooting of received errors (`host_errors`). `HostError` gains a
variant holding a core value (a pre-interned, rooted string), so the audio
interrupt raises "audio processor execution budget exceeded" without
allocating when the budget is exhausted (lane 7 gap 3). A failure that
`pcall` must not swallow stays a host flag (`AudioHost::failure`), as the
engine does today; the region-lock test pins it.

### 5.5 Memory budget for host payloads

Unchanged in substance: the core charges `USERDATA_BYTES` plus the declared
size before creating the userdata and credits it in the sweep (budget rule
1). The host's payload memory beyond the declared size is the host's, as
today (`create_userdata_with_size`). The payload is dropped at the host's
next drain rather than inside the sweep, so it outlives its credit by at
most one host entry. The freed list and root table are host bookkeeping,
not charged, like today's handle table (`handle.rs:24`).

### 5.6 Destructor timing (observable change)

`Compat` drains the freed list at the start and end of every trampoline
call and every top-level API operation, so every point where host code can
observe a destructor's effect comes after the drain: a Lua chunk that runs
`collectgarbage()` and then calls a host function sees the drop count it
sees today (`tools/luau-vm-diff/src/host_surface.rs:56`). Open question 3.

### 5.7 The audio thread

What the current engine guarantees, read from the code: the block's host
side (`process_block`, `:953`) does not allocate; the script's allocations
go to the system allocator through the counting allocator, bounded by
`memory_limit` with a sticky failure; host functions allocate when they
copy messages (`read` builds `BTreeMap`s, `receiveInto` a `Vec` of keys,
`:1422`, `:1553`); every block has a deadline checked by the interrupt;
any failure renders silence afterwards. There is no "no allocation after
warm-up" guarantee for script code. The new engine keeps exactly these:
`vm.call` on the core leaves results on the stack (no result `Vec`), the
output buffer is zeroed and read through `Core::buffer_bytes{,_mut}` slices
(lane 7 gap 2), and the stack and frame vectors keep their capacity after
the first block, so the host side of the block stays allocation-free; the
collector's per-block step frees arena slots without returning capacity.

## 6. Predictions

No targets. These are what the measurement in stage B should show; where
one fails, the design gets revisited, not tuned.

- **`make luau-vm-perf-report` (87 benches), core changes only:** no
  movement beyond the measured spread. Instructions per `Arc` clone/drop are
  about those of `Rc` (one `ldadd` vs load/add/store), so the counter will
  not see the atomics; **wall time** on call-heavy benches (`Factorial`,
  `MethodCalls`, `OOP_*`) may rise by a few percent from the proto `Arc` on
  every frame switch. The full-host-call driver exit costs nothing on
  benches, which make no host calls.
- **If builtins move to the plain convention (open question 1):** the 21
  builtin-call benches (5.2x group, perf map 4.1) fall the most, toward the
  perf map's inferred 1.5 to 2x of luaur; nothing else moves.
- **Host surface (`tools/luau-vm-diff` host cases, conformance host
  helpers):** plain host calls drop from roughly a builtin call's cost
  (about a thousand instructions) to one or two hundred; full calls get
  slightly cheaper (no `NativeFn` clone) and slightly dearer (driver exit),
  net about flat.
- **Product scripted frame cost (facade):** about flat. The facade's
  per-call `Vec`s, handles and `lend` swaps stay; lane 8 removes none of
  them. Handle release becomes O(1) instead of a scan, which is visible
  only in collection-heavy frames.
- **Audio, vendored engine to safe engine:** **slower**, by roughly the
  safe VM's builtin-call gap, because the block is per-sample
  `buffer.writef32` calls that luaur runs as fast builtins and the safe VM
  runs through the full native protocol (§1). Without the builtin change, a
  processor near its realtime deadline on the vendored engine can miss it
  on the safe one. There is no audio bench to measure this; open question 2.

## 7. Commits (each green on its own)

1. `nuxie-luau-vm`: protos and interned strings shared by `Arc`; native
   names and builtin upvalues boxed; `NativeFn` `Copy`, no clone in
   `step_native`. Behavior-neutral; vm-diff and conformance.
2. `nuxie-luau-vm`: no core recursion with a host callee: `catch` split
   around the `xpcall` handler; `%*` and the dispatcher error as
   continuations. vm-diff cases for a host `__tostring` under `%*` and a host
   `xpcall` handler.
3. `nuxie-luau-vm`: `Core` / `Vm<H = Compat>` split, `Host` and `Dispatch`,
   full host calls at the driver; `Compat` holds today's closures and
   continuations; host natives are `(HostFnId, data)`. Lane 6 API and
   facade unchanged.
4. `nuxie-luau-vm`: userdata payloads move to the host (`key: u64`), the
   freed list, `Compat`'s payload store and drain points; type metatables
   and app data into `Compat`.
5. `nuxie-luau-vm`: interrupt and direct fields as `Host` hooks; the matcher
   through `BuiltinHost`. Deletes the interrupt slot.
6. `nuxie-luau-vm`: core `Roots`; lane 6 handles over them with the release
   queue; `error_object` as a root. Deletes the `Weak` scan. Compile-time
   `Send` assertion for `Core`. Core unit tests: stale root, foreign root,
   freed list drained exactly once, re-entry, budget for declared payload
   sizes.
7. `nuxie-luau-vm`: the plain kind and the frameless `CALL` path with the
   virtual record and materialize-on-error; buffer slices; `HostError`
   from a core value; allocation-free `call` on the stack.
8. (Open question 1) builtins classified plain, a few families per commit,
   each with its differential cases.
9. `nuxie-scripting`: the audio processor on `Vm<AudioHost>` under
   `nuxie-vm` (the raw processor moves to a `cfg(not(nuxie-vm))` module
   unchanged; `Owner`, `Shared`, `ProcessorHandle` shared); `Send`
   assertion for the processor; the 15 engine tests under both settings;
   the browser kernel builds for wasm32.
10. Docs: evidence, plan, seam rows R33 to R36.

Gates per the brief run on the final tip.

## 8. Open questions for the coordinator

1. **Builtins in this lane?** The frameless path is built either way; moving
   builtins onto it is what changes audio (§1, §6) and the 21 builtin
   benches. Recommendation: yes, at least the builtins the vendored
   compiler emits `FASTCALL` for, which also deletes `libs/fast.rs`'s
   duplicate bodies. It overlaps perf map redesign 1, so it is also the
   Phase 4 lane's first item; doing it here keeps the audio move from
   regressing realtime processors.
2. **An audio-block bench.** Add one to `tools/luau-bench` (a stereo
   processor writing a 1024-frame block with `buffer.writef32`, plus a
   variant reading shared atomics), run on both VMs, so §6's audio
   prediction is measured. Phase 4 already lists "an audio processor block"
   for the compute-heavy set.
3. **Destructor timing.** Userdata `Drop` moves from the sweep to `Compat`'s
   next drain point (§5.6). Every host-observable point is after a drain;
   accept?
4. **`Arc` now, indices later.** Accept the `Arc` proto clone on frame
   switches (§5.1, §6) until redesign 2 removes it, or do redesign 2's
   "protos by index" in this lane?
5. **`H` owned by `Vm<H>`** instead of passed per entry (§0, §3 item 5): a
   deliberate departure from the sketch.
