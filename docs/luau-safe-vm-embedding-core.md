# Safe Luau VM: the embedding core (lane 8 design)

Status: design, stage A of lane 8 (2026-11-12), amended the same day after
the coordinator's review (approved by Levi): builtins move to the frameless
path in this lane, protos are held by index, the audio-block bench comes
first, and `Compat` sits behind a hard module boundary. Section 8 records
the decisions. Not built. Base `origin/main` `4c0e2f1bf8`. Inputs: plan
lane 8, lane 6 and 7 evidence (lane 7 section 10 is the audio brief), the
perf gap map (`docs/evidence/luau-safe-vm-perf-map-2026-11-11.md`,
sections 3.4, 4, 5), the workload characterization (2026-11-10), and the
code cited below by file and line.

## 0. The design in one page

- **`Core`** is the VM: heap, stacks, frames, interpreter, libraries,
  collector, budget, root table, and the prototype table. It holds plain
  data only: no `Rc`, no closure, no `dyn Any`, no host payload. It is
  `Send`, asserted at compile time. Two `Arc`s remain: the prototype table
  (§5.1) and, temporarily, string bytes (§5.1).
- **`HostedVm<H: Host>`** is `Core` plus the host's state `H` plus a
  table of host function pointers (`Vec<HostEntry<H>>`). The core never
  looks inside `H`; it reaches it only through the `Host` trait (interrupt,
  direct-field read) and the function table. `HostedVm<H>` is `Send`
  exactly when `H` is, because fn pointers and `Core` are. It has no
  default type parameter: a default would make the generic layer name
  `Compat` (§5.8).
- **Natives are split by type, not by a list.** A builtin that neither
  calls Lua nor yields is `fn(&mut Core, Args)` and runs frameless (§2.1).
  A builtin that does either is `fn(&mut Resume<'_>, Args)`; only `Resume`
  has the request methods, so a builtin that requests cannot be declared
  plain.
- **Protos by index.** Frames and closures hold a `ProtoId`, never a
  pointer. Protos live in `Core::protos`, a table apart from the heap that
  only `load` writes and that the interpreter borrows while it mutates the
  rest of `Core` (§5.1).
- **A host function is a fn pointer plus a `u32` of data**, addressed by
  `(HostFnId, data)` in the native closure. Two kinds:
  - **plain** `fn(&mut Plain<'_, H>) -> Result<usize, HostError>`: may
    read and push values, allocate, raise; may not call Lua or yield. It is
    called by `CALL` directly on the argument window, with no frame (§2).
  - **full** `fn(&mut HostCtx<'_, H>) -> Result<usize, HostError>` (and an
    optional continuation `fn(&mut HostCtx<'_, H>, CallStatus)`): today's
    host function, with a frame, able to call back and yield. `HostCtx`
    holds `&mut HostedVm<H>`, so it can do everything the host can.
- **Where host code runs.** The interpreter and libraries are compiled
  once, on `Core`. The generic driver (`HostedVm<H>::execute`) calls
  `core.run_frames(&mut dispatch)`, where `dispatch` is `&mut dyn Dispatch`
  over `(&mut H, &host_fns)`, split from `self` by field. Plain functions,
  the interrupt and direct-field reads go through `dispatch` inside the
  loop. A frame running a full host function makes `run_frames` return to
  the driver, which now holds the whole `&mut HostedVm<H>` and calls the
  function with it, then re-enters the loop. The driver also runs `xpcall`
  handlers (§2.2), so every Rust-stack re-entry starts in host code, and no
  `run_frames` is on the Rust stack while host code runs.
- **Userdata** in the core is `(host key: u64, metatable, tag, declared
  size)`. The host owns the payloads and packs whatever it likes in the
  key. The sweep appends each freed userdata's key (and each freed native
  whose function was registered as owning its data) to a freed list that
  the host drains.
- **Roots** are generational indices `Root { vm: u32, index: u32, gen: u32 }`
  into the core's root table, released explicitly. A stale or foreign root
  reads as `None` and its operations return `Error::Host`.
- **Lane 6's API is `Vm`, an alias for `HostedVm<Compat>`** declared in
  the embedding layer: `Compat` holds the closures (`Rc<HostFn>`), the
  `Rc<RefCell<T>>` payloads, the app data, the interrupt closure and
  direct-field getters, all keyed by the core's indices; one full
  trampoline dispatches every lane 6 closure. The alias keeps the name `Vm`
  meaning what it means today, so the facade and the tools compile
  unchanged except where they reach internals. `Compat` lives in a private
  module of the layer; nothing in the core can name it (§5.8).
- **The audio engine is `HostedVm<AudioHost>`** with plain host functions
  over its channel index, two full ones (`print`, `require`), and
  `AudioHost: Send`. It moves between threads as one value. No `unsafe`.

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
protocol at all. Builtins and host functions both move in this lane
(decision 1).

**The kinds are types.** `NativeFn` (`Copy`) has four variants, and `CALL`
decides with one match:

| Variant | Signature | Convention |
|---|---|---|
| `Plain(PlainFn)` | `fn(&mut Native<'_>, Args) -> VmResult<usize>` | frameless |
| `Resumable(ResumableFn)` | `fn(&mut Resume<'_>, Args) -> VmResult<usize>` | frame + continuation, as today |
| `HostPlain(HostFnId, u32)` | `fn(&mut Plain<'_, H>) -> Result<usize, HostError>` | frameless |
| `HostFull(HostFnId, u32)` | `fn(&mut HostCtx<'_, H>) -> Result<usize, HostError>` | frame, run at the driver (§0) |

`Native<'_>` is `&mut Core` plus the erased host's interrupt hook (the
`&mut dyn Dispatch` the interpreter holds), and dereferences to `Core`.
`Resume<'_>` is a `Native<'_>` plus the request methods: `request_call`,
`request_yield`, `request_resume` and `call_value` are methods of `Resume`
only, not of `Core` or `Native`. Only `step_native` constructs a `Resume`,
and continuations receive one. A plain builtin has no way to name the
request methods; a builtin that requests does not compile until it is
declared `Resumable`. The classification is checked by the compiler, not
kept by hand, and there is no "plain builtin requested" state to handle at
run time.

Resumable after the change: `pcall`, `xpcall`, `tostring` and `print`
(`__tostring`), `string.format` (`%*` calls `__tostring`; today through
`tostring_value` and `call_function`, `libs/string.rs:1211`), `string.gsub` with a function
replacement, `table.sort` with a comparator, `table.foreach`,
`table.foreachi`, `coroutine.yield`, `resume` and `wrap`'s function, the
userdata dispatchers. Everything else in `libs/` is plain, including
`error`, `assert`, the `math`, `bit32`, `buffer`, `vector`, `utf8` and
`table` functions without callbacks, and `getfenv`, `setfenv` and `debug.*`.

**Frameless `CALL`.** For `Plain` and `HostPlain`, inside the dispatch
loop:

1. checks call depth as if a frame were pushed (the same `frames.len()`
   comparison `push_native` makes today, so the overflow point is
   unchanged) and `ensure_stack(top + MIN_STACK)`;
2. sets the **virtual native record** `{closure, base, nargs}`;
3. calls the function with `Args { base, nargs }`;
4. on success clears the record, moves the top `n` results to `ra`, pads
   to `nresults`, sets `top`, runs `safepoint()` where the frame protocol
   runs it today, and keeps dispatching in place;
5. on error **materializes** the record (pushes the native `Frame` it
   stands for) and returns the `Throw`, so the catcher, `xpcall` handlers
   and `debug.traceback` in the handler see `[C]` at level 0 exactly as now.

No `Frame`, no `Wait`, no `deliver`, no `continue 'frame` on success.

**Every frame walk sees the record.** The frame vector moves into a
`Frames` type in its own module with private fields: the list and the
record. Code outside that module cannot read the list directly (a build
error), only through methods: the interpreter's top-Lua-frame accessor, and
a level view in which an active record is level 0, where the native's
frame is today. Everything that counts levels goes through the view:
runtime-error positions and `luaL_where`, `error` with a level, `getfenv`,
`setfenv`, `debug.info`, `debug.traceback`, `inspect_stack`, the function
name in argument errors (`libs/mod.rs:54`) and `builtin_upval`
(`libs/mod.rs:204`). The callee stays in its stack slot below the
arguments, which roots it for the collector as a frame's `closure` field
does today.

**`libs/fast.rs` stays.** The four bodies there are not duplicates of the
library functions: they are the C++ fastcall bodies where those compute a
different result (`math.modf`'s signed-zero fraction, `math.ldexp` as C
`ldexp`, fused multiply-adds in `vector.magnitude` and `vector.dot`), each
pinned by a vm-diff case that calls the builtin both ways. Deleting them
would change results; the frameless path makes the `CALL` they fall back to
cheap, which was the reason to want them gone.

**One convention or two?** One convention per kind (frameless over the
argument window, or frame plus continuation) and separate signatures for
builtins and host functions: builtins take `&mut Core`, host functions are
typed over `H`. Making builtins generic over `H` to share one signature
would monomorphize the libraries per host; erasing `H` into every builtin
adds a parameter 300 functions ignore.

The pattern matcher is the only builtin that needs the host (the
interrupt fires on every match step, seam 3.1). It calls
`Native::interrupt()`, which reaches `Host::interrupt` through the
dispatch, instead of the `Rc` slot it clones today.

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
8. **Protos by `Arc`, indices later** (stage A's proposal, reversed in
   review). An `Arc<Proto>` on every frame and closure would have put an
   atomic increment and decrement on every Lua call and return to keep a
   pointer the frame does not need: closures already carry a `ProtoId`
   (`heap.rs:217`) and protos are immutable after load. Protos go by index
   in this lane (§5.1). **Strings by index now** stays rejected for this
   lane: removing the shared string bytes is the string-store redesign
   (perf map redesign 6), tied to the `pairs` order decision, and gets its
   own lane.
9. **Building the VM on the audio thread** (rejected by Levi, 2026-11-11)
   and `unsafe impl Send` (forbidden).
10. **Splitting `Core` into `{ protos, state }`** so the interpreter can
    borrow a proto while mutating the state without an `Arc`. Every method
    the interpreter and libraries call would move from `Core` to `state`,
    which is the whole crate; one `Arc` clone per `run_frames` entry gives
    the same borrow (§5.1).
11. **A hand-kept list of plain builtins** (or a `plain: bool` in the
    registration). A builtin wrongly listed plain would request through a
    path that cannot serve it. The kinds are signatures instead (§2.1).
12. **A separate crate for `Compat`.** The hardest boundary, but the lane 6
    layer reaches heap, stack and handle internals that would all have to
    become public API of the core crate. A private module plus a source
    check (§5.8) gives the same build failure for the type and keeps the
    internals crate-private.

## 4. What gets deleted

From `nuxie-luau-vm` (lane 6's layer and the core):

| Deleted | Where today | Replaced by |
|---|---|---|
| `NativeFn::Host(Rc<HostFn>)`, `HostK(Rc<HostFn>, Rc<HostContFn>)`, `BuiltinWith(_, Rc<[Value]>)` and the clone in `step_native` | `native.rs:49`, `call.rs:523` | `Copy` `NativeFn` with `(HostFnId, data)`; upvalues in `NativeClosure` |
| The frame protocol for every builtin that does not request (frame push, `Wait`, `deliver`, `continue 'frame`) | `call.rs:508-583` | the frameless `CALL` path (§2.1) |
| `Rc<Proto>` in `LuaClosure`, `Frame`, the heap's proto arena, and the clones on every call and frame switch | `heap.rs:216,417`, `interp.rs:290,565,603`, `call.rs` `enter_lua` | `ProtoId`; `Core::protos` (§5.1) |
| `request_*` and `call_value` as `Core` methods | `call.rs:483`, `vm.rs:679` | methods of `Resume` (§2.1); the driver |
| direct reads of the frame vector outside the frame module (63 sites) | `vm.rs`, `call.rs`, `interp.rs`, `thread.rs`, `gc.rs`, `libs/` | `Frames` methods and the level view (§2.1) |
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

What is added: `Core`/`HostedVm<H>` split, `Host`/`Dispatch`, `Roots`,
the freed list, `Frames`, the proto table, the frameless path, `Native` and
`Resume`, `Compat`, `AudioHost`. The additions are mostly the old code
moved behind a type boundary; the net line count of `nuxie-luau-vm` should drop (lane 6's `Rc` plumbing), and the
engine's safe processor should be shorter than `RawProcessor`.

## 5. Safety and panic-freedom

### 5.1 Making `Core` `Send`

`Core` holds `Rc` in four places that are not the embedding layer
(`grep` of the crate): `Rc<Proto>` (heap arena, closures, frames,
`heap.rs:216,417`, `interp.rs:565`), interned string bytes `Rc<[u8]>`
shared between the arena and the interner key and cloned by `str_rc` so
string builtins can hold bytes across allocation (`heap.rs:259,423,743`),
native names `Rc<str>`, and builtin upvalues `Rc<[Value]>`. Names become
`Box<str>`, upvalues `Box<[Value]>`. `Box<dyn Continuation>` becomes
`Box<dyn Continuation + Send>`; every implementation holds plain data once
`HostCont` holds ids. The budget's `Cell<bool>` is `Send`. A
`const _: () = assert_send::<Core>()` and one for `HostedVm<AudioHost>`
make it a build failure to regress; nothing asserts it for `Vm`
(`HostedVm<Compat>`), which is `!Send` and need not be.

**Protos by index.** `LuaClosure` and `Frame` hold a `ProtoId` (closures
already carry one, `heap.rs:217`; children are already ids,
`bytecode/mod.rs:205`). Protos live in `Core::protos: Arc<ProtoTable>`, a
field apart from the heap:

- *Written only by `load`.* `load` runs from host code, and host code never
  runs while `run_frames` is on the Rust stack (§0), so `load` takes
  `Arc::get_mut`; if that ever fails it returns an internal error rather
  than cloning the table.
- *Borrowed by the interpreter.* `run_frames` clones the `Arc` once at
  entry (one atomic increment per entry, not per call or frame switch) and
  borrows `&Proto` from its clone while it mutates the rest of `Core`; a
  frame switch re-reads by index, a bounds-checked `Vec` access. The
  collector and the error and debug paths read `self.protos` directly,
  which the borrow checker allows because it is a separate field.
- *Collected as today.* Protos stay collectable objects with today's
  marking (a closure marks its proto, a proto its constants and children).
  Their colors move out of the immutable table into a vector in the heap.
  The sweep credits a dead proto's bytes to the budget at once, so
  `collectgarbage("count")` and the memory limit see what they see today,
  and appends its id to `dead_protos`. The slots are emptied at the next
  driver point where `Arc::get_mut` succeeds (every top-level entry and
  exit, and `load`); a slot is reused only after it is emptied.
  `accounting_drift` treats pending dead protos as already credited. A
  reachable closure cannot hold a dead id, because the collector frees a
  proto only when nothing marks it; lookups still use `get` and raise an
  internal error on a miss.

**Strings stay `Arc` in this lane, temporarily.** Interned string bytes
become `Arc<[u8]>`: the smallest change that makes `Core` `Send` without
touching the string store. It is temporary. Removing the sharing (the
arena, the interner key and `str_rc` all holding the same bytes) is the
string-store redesign, perf map redesign 6, which is tied to the `pairs`
order decision and gets its own lane. Until then string builtins that
hold bytes across allocation pay an atomic increment where they paid a
plain one.

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
`&mut HostedVm<H>`. Its host state is `ctx.host_mut()`, a borrow of
`ctx`; calling back (`ctx.call`, `ctx.call_yieldable`) needs `&mut ctx`, so
the borrow checker forbids holding host state across a re-entry. No `RefCell` is
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
sees today (`tools/luau-vm-diff/src/host_surface.rs:56`). Accepted
(decision 3).

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

### 5.8 The `Compat` boundary

`Compat` and everything only it uses (lane 6's closures and
continuations, payload store, app data, interrupt closure, direct-field
getters, handle release queue) live in `embed::compat`, a private module
of the embedding layer `embed`. Rust privacy makes naming `Compat` from any
module outside `embed` a build error (E0603, private module): the
interpreter, heap, call protocol, collector and libraries cannot reach it.

The layer exports one alias, `pub type Vm = HostedVm<compat::Compat>`, the
name the facade and the tools use. Privacy cannot stop crate code from
naming a public alias, so a unit test in `nuxie-luau-vm` (`boundary`)
reads every source file outside `src/embed/` and fails if one mentions
the `embed` module or the identifiers `Vm` or `Compat`. Layering is
one-way: `embed` uses the core, the core never uses `embed`; the test is
that rule in a dozen lines, and it runs with the crate's unit tests in
every gate.

## 6. Predictions

No targets. These are what the measurement in stage B should show; where
one fails, the design gets revisited, not tuned.

**The audio-block bench, before** (decision 2; `tests/luau-bench/nuxie/`,
32 blocks of 1024 stereo frames per run; base `4c0e2f1bf8`, Bazel
fat-LTO release, 1-minute load 11.8 on a shared 16-core laptop, so wall
times are inflated and the instruction counts are the measurement):

| Bench | luaur M instr | safe VM M instr | ratio | luaur ms | safe VM ms |
|---|---:|---:|---:|---:|---:|
| `AudioBlock_sine` | 17.38 | 107.09 | 6.16x | 0.52 | 5.07 |
| `AudioBlock_shared_controls` | 18.23 | 107.76 | 5.91x | 0.54 | 5.11 |

Per frame that is 530 instructions on luaur and 3,270 on the safe VM. A
frame makes three builtin calls (`math.sin`, two `buffer.writef32`), so the
gap is about 910 instructions per builtin call, the perf map's measured
native-protocol cost (+830 to +860, section 2.2) plus the arithmetic the
safe VM's interpreter adds around it. The 16 host calls per block of the
shared-controls variant are not visible.

After lane 8:

- **`AudioBlock_*`:** the frameless path replaces the native protocol on
  all three builtin calls. Prediction: 1,100 to 1,400 instructions per
  frame, **2 to 2.6x luaur** (from 6.2x), the same for both variants.
- **`make luau-vm-perf-report` (87 benches), the 21 builtin-call benches**
  (5.2x group, perf map 4.1) fall the most, toward the perf map's inferred
  1.5 to 2x of luaur. The exceptions are benches whose builtin calls are
  `string.format`, `tostring` or `print`, which stay resumable (§2.1) and
  do not move.
- **The call-heavy benches** (`Factorial`, `MethodCalls`, `OOP_*`,
  `UpvalueCapture`): protos by index remove an `Rc` increment and decrement on
  every call and every frame switch (`call.rs` `enter_lua`,
  `interp.rs:290`); a few percent fewer instructions, within 2 to 5%.
  Everything else: no movement beyond the measured spread. The
  full-host-call driver exit costs nothing on benches, which make no host
  calls.
- **Host surface (`tools/luau-vm-diff` host cases, conformance host
  helpers):** plain host calls drop from roughly a builtin call's cost
  (about a thousand instructions) to one or two hundred; full calls get
  slightly cheaper (no `NativeFn` clone) and slightly dearer (driver exit),
  net about flat.
- **Product scripted frame cost (facade):** about flat. The facade's
  per-call `Vec`s, handles and `lend` swaps stay; lane 8 removes none of
  them. Handle release becomes O(1) instead of a scan, which is visible
  only in collection-heavy frames.
- **Audio, vendored engine to safe engine:** slower per block by the
  residual of the first item, about 2 to 2.6x the vendored engine's VM
  instructions, rather than the 6x a move without frameless builtins
  would have cost. A 1024-frame block is 21.3 ms of real time at 48 kHz;
  the bench's block is about 16 microseconds on luaur.

## 7. Commits (each green on its own)

0. `tools/luau-bench`: the audio-block benches (decision 2), run on both
   VMs (§6), then `make luau-vm-perf-report` for the before table. No VM
   code before this.
1. `nuxie-luau-vm`: protos by index (`ProtoId` on frames and closures,
   `Core::protos`, deferred slot reuse); interned strings `Arc`; native
   names and builtin upvalues boxed; `NativeFn` `Copy`, no clone in
   `step_native`. Behavior-neutral; vm-diff and conformance.
2. `nuxie-luau-vm`: `Frames` with private fields and the level view; every
   frame walk through it. Behavior-neutral.
3. `nuxie-luau-vm`: no core recursion with a host callee: `catch` split
   around the `xpcall` handler; `%*` and the dispatcher error as
   continuations; `Resume` holds the request methods and `call_value`.
   vm-diff cases for a host `__tostring` under `%*` and a host `xpcall`
   handler.
4. `nuxie-luau-vm`: `Core` / `HostedVm<H>` split, `Host` and `Dispatch`,
   full host calls at the driver; `embed::compat::Compat` holds today's
   closures and continuations; `Vm` is the alias; the `boundary` test.
   Lane 6 API and facade unchanged.
5. `nuxie-luau-vm`: userdata payloads move to the host (`key: u64`), the
   freed list, `Compat`'s payload store and drain points; type metatables
   and app data into `Compat`.
6. `nuxie-luau-vm`: interrupt and direct fields as `Host` hooks; the
   matcher through `Native::interrupt`. Deletes the interrupt slot.
7. `nuxie-luau-vm`: core `Roots`; lane 6 handles over them with the release
   queue; `error_object` as a root. Deletes the `Weak` scan. Compile-time
   `Send` assertion for `Core`. Core unit tests: stale root, foreign root,
   freed list drained exactly once, re-entry, budget for declared payload
   sizes.
8. `nuxie-luau-vm`: the frameless `CALL` path with the virtual record and
   materialize-on-error, for `Plain` builtins and `HostPlain`; every
   non-requesting builtin becomes `Plain` (a few library families per
   commit, each with its differential cases); buffer slices; `HostError`
   from a core value; allocation-free `call` on the stack.
9. `nuxie-scripting`: the audio processor on `HostedVm<AudioHost>` under
   `nuxie-vm` (the raw processor moves to a `cfg(not(nuxie-vm))` module
   unchanged; `Owner`, `Shared`, `ProcessorHandle` shared); `Send`
   assertion for the processor; the 15 engine tests under both settings;
   the browser kernel builds for wasm32.
10. Docs: evidence, plan, seam rows R33 to R36.

Gates per the brief run on the final tip. If protos by index turns out to
need a wider frame redesign than commit 1 describes, the lane stops and
reports before doing it.

## 8. Decisions (coordinator review, approved by Levi, 2026-11-12)

1. **Builtins move in this lane.** Every builtin that neither yields nor
   calls Lua runs frameless, classified by signature (§2.1). The four
   bodies in `libs/fast.rs` turned out not to be duplicates and stay
   (§2.1).
2. **Audio-block bench first**, before any code, on both VMs, report-only
   (§6, commit 0).
3. **Destructor timing accepted:** userdata `Drop` runs at `Compat`'s next
   drain point, not in the sweep (§5.6).
4. **Protos by index now**, not `Arc` (§3 item 8, §5.1). Strings stay `Arc`
   in this lane; that is temporary and belongs to the string-store
   redesign.
5. **`H` owned by `HostedVm<H>`** accepted (§0, §3 item 5).
6. **`Compat` behind a hard module boundary** (§5.8).

The product will then have two stacked compatibility layers: the lane 7
facade (`luaur-rt`-shaped handles over lane 6's API) on `Compat` (lane 6's
API over the core). Collapsing them, and moving hot bindings onto plain
host functions, is the next lane, not this one; the lane 8 evidence records
what this lane learns about the cost of the stack.
