# Lane C.1 (renderer loan): the renderer is lent, not erased

Date: 2026-11-14. Branch `claude/luaur-lane-c1-renderer-loan-200a01`, on
`a2e2271537` (= `levi/luaur-client-safe` at rebase). Design:
`docs/luaur-client-path-safe.md` section 5 item 1. Sections 1-4 of the first
version of this file (the stop report) are kept below as Appendix A; the
coordinator approved the VM change it described (3.1/3.2) and this is the
build.

**Outcome.** `lua_renderer.rs` has no thread-local, no `transmute`, no
`NonNull`, no `unsafe`, and no `Drop` that checks stack order. Script
renderer userdata holds a token. A Rust call that runs a draw lends a
`RendererScope { token, &mut dyn Renderer }` as a `dyn ScriptLoan` through
the call; host functions that call reaches receive the loan as an argument
and look their token up in it. Error texts are unchanged. One impossible
state is now an impossible type: scopes ended out of order (they are
borrows), so its error text ("scoped renderer stack order") is gone.

Commits (oldest first):

| Commit | What |
|---|---|
| `2c5dc49204` | docs: the stop report (Appendix A) |
| `17e77fed9d` | nuxie-luau-vm: calls from Rust can lend borrowed host data to the host functions they reach |
| `d690ae5b65` | facade: `ScriptLoan`, `call_lent`/`call_with`, lent callbacks |
| `55c3b3bc89` | renderer lent to the draw call; thread-local registry deleted; pcall/xpcall/resume guards, print, font dispatch forward |
| `e708257345` | promise methods, gradient stops, `Canvas:beginFrame` descriptor forward |
| `676101d9ba` | (tools) attributed view-model calls forward |
| this commit | evidence |

## 1. Type shape: associated family on `Host`, proven by a spike

`nuxie-luau-vm` (`hosted.rs`):

```rust
pub trait LoanFamily: 'static {
    type Loan<'l>: ?Sized + 'l;
    fn shorten<'a, 'l: 'a>(loan: &'a mut Self::Loan<'l>) -> &'a mut Self::Loan<'a>;
}
pub enum NoLoan {}                       // Loan<'l> = NoLoan: no loan can exist
pub trait Host: 'static { type Loans: LoanFamily; /* ... */ }
pub type Lent<'a, H> = Option<&'a mut <<H as Host>::Loans as LoanFamily>::Loan<'a>>;
```

**Associated family, not a second generic parameter.** The coordinator
preferred it; I kept it because every driver type is already generic over
`H: Host`, so the loan type follows from `H` with no new parameter on
`HostedVm`, `HostedCtx`, `PlainCtx`, `Hosted` or any driver function, and
the VM's own libraries (whose host functions are written against
`HostedCtx<'_, H>` / `PlainCtx`) keep their signatures. A second parameter
would have had to be threaded through all of those. The cost: each `Host`
impl names its family, one line each (`AudioHost` in
`luau_audio_processor_engine/safe.rs`, the test `Probe`), with `NoLoan` for
both. The embed layer is `Compat<L: LoanFamily = NoLoan>` with
`type Loans = L`, so `embed::Vm`, `HostCtx`, `HostFn`, `UserDataType` keep
their names and default to `NoLoan`; `Vm::new`/`with_limits` stay on
`Vm<NoLoan>` (inference unchanged for existing callers) and
`Vm::<L>::lending(limits)` makes a lending VM. Handle methods that existing
code calls with a turbofish take `&mut Vm<impl LoanFamily>`, so `::<T>`
keeps working.

`shorten` exists because the compiler cannot shrink the lifetime inside a
generic GAT behind `&mut` (invariance); for `dyn ScriptLoan + 'l` the body
is `loan`. It keeps the single lifetime on `HostedCtx<'a, H>`.

**Spike** (outside the repo, rustc 1.97.1, a stand-alone model of the
driver: a `'static` stored closure, a ctx with one lifetime, a dyn loan,
`shorten`, a nested reborrow, and `NoLoan`): compiles with no `unsafe` and
prints `["outer", "inner", "outer-after"]` (the outer loan is usable again
after a nested call lent a child loan). Two lessons fed the build: the ctx
needs an `H: Host` bound to name the loan; the loan must be reborrowed into
a local before building the ctx, and a `vm_and_loan()` accessor is needed so
a callback can hold the VM and the loan at once.

**Threading.** The loan is an argument of `execute`, `execute_from`,
`run_host_call`, `invoke_host`, `catch`, `run_error_handler`,
`call_value`(`_inner`), `call_function`, `meta_now`, `tostring_value`; the
`Hosted` dispatch struct and both contexts carry it, `ctx.loan()` /
`vm_and_loan()` hand it out, and nested calls a host function makes through
its context (`call_top`, `pcall_top`, `__index` via `push_field`, base-lib
`pcall`) reborrow it. New entry points: `HostedVm::enter_lent`/`enter_with`,
embed `Function::call_lent`, `Table::get_lent`. Existing entry points pass
`None`. VM tests added: `a_loan_reaches_every_host_function_its_call_reaches`
(call_top, pcall_top, an `__index` through `push_field`, base-lib `pcall`
from the frame loop, a child loan through `call_lent`, and no loan without
lending) and `a_loan_ends_with_its_call`.

VM crate: 0 `unsafe` (unchanged), `[dependencies]` empty (unchanged), no
forced panic-lint site added (`make luau-vm-ratchets`, section 5).

## 2. Facade and renderer

- `facade/loan.rs`: `trait ScriptLoan { fn renderer(&mut self, token: u64) -> Option<&mut dyn Renderer>; }`,
  `ScriptLoans` (the family; `Loan<'a> = dyn ScriptLoan + 'a`),
  `Loan<'a> = Option<&'a mut (dyn ScriptLoan + 'a)>`, `reborrow_loan`
  (`as_deref_mut` does not shorten the dyn lifetime inside an `Option`).
- The facade's VM is `Vm<ScriptLoans>`; `BoxedCallback` takes
  `(&Lua, Loan<'_>, MultiValue)`; the trampoline takes the loan from
  `ctx.vm_and_loan()` before `lend` swaps the VM out. `Function::call_lent`,
  `Function::call_with(loan, args)`, `Table::get_with(loan, key)`,
  `Lua/Scope::create_function_lent`, `UserDataMethods::add_method_lent` /
  `add_meta_method_lent` / `add_function_lent`. Plain registrations keep
  their shape (they ignore the loan).
- `lua_renderer.rs`: `RendererScope` implements `ScriptLoan`; tokens come
  from a static `AtomicU64` (a counter, not a store). A per-userdata
  `Cell<bool>` in-use mark gives "Renderer is already mutably borrowed by
  this callback"; a token the loan does not hold, or no loan, gives
  "Renderer is no longer valid.". `RestoreTarget` (a `Drop` that restores
  the save count of a reborrowed scope; no order check, no panic) is the only
  `Drop` besides the in-use mark's reset.
- Scope openers converted: `call_draw_with_balance`, `lua_transition.rs`
  `call_draw`, `lua_artboards.rs` (`drawModulated`, `draw` and its visitor,
  `VisitedDrawable:draw`), `lua_text.rs` `draw`; renderer methods are all
  `add_method_lent`.

## 3. Reach audit: Rust-to-Lua calls while a renderer scope is open

Before, the renderer was reachable from any Lua running on the thread while
a scope was open. Now it is reachable from Lua run through a call that
carries the loan. Lua-to-Lua calls carry it. I audited every Rust-to-Lua
call reachable during a draw (an agent swept the crate, I checked each
site):

**Forwarded** (the call site has the loan from its callback):

| Site | Why it matters | Covered by |
|---|---|---|
| `pcall`, `xpcall` (and its handler), `coroutine.resume` guards, `resource_limits.rs` | these are host guards that call the originals from Rust, so a `pcall` inside `draw` crossed Rust (this refutes my stop report's "Lua-to-Lua calls keep it") | new test `a_draw_reaches_its_renderer_through_guarded_calls_print_and_gradient_stops`; `scope_probe.riv` tests |
| `print` -> `tostring` (`lua_rive_base.rs`) | `__tostring` | same test |
| font `__index`/`__namecall`/`__newindex` dispatchers (`lua_font.rs`) | `Text:draw`, `upstream_scripting_text` depend on it | existing text tests |
| promise `andThen`/`catch`/`finally`/`cancel`/`onCancel`/`getStatus` (`lua_promise.rs`) | the engine runs handlers synchronously when a promise is settled | (not separately tested) |
| gradient stops `stop.get` (`renderer.rs`) | `__index` on a stop table | new test above |
| `Canvas:beginFrame` `descriptor.get("clearColor")` (`lua_canvas.rs`) | `__index` on the descriptor | (not separately tested) |
| (tools) `attributedCall` (`view_model.rs`) | the editor's attribution wrapper runs view-model methods and the Lua `__newindex` write from Rust | (tools build + tests) |

**Not forwarded**, each with a script that observes the narrower reach
(it drew before; it now errors "Renderer is no longer valid."):

1. **View-model listeners** (`nuxie-runtime` `flush_property_change_callbacks`
   -> stored `Box<dyn Fn()>` callbacks). The listener closures are owned by
   `nuxie-runtime`, which does not know the scripting loan type; forwarding
   needs a `nuxie-runtime` callback signature change.
   ```lua
   draw = function(self, renderer)
       self.vm.count:addListener(function() renderer:save(); renderer:restore() end)
       self.vm.count.value = self.vm.count.value + 1   -- listener runs from Rust
   end
   ```
2. **GPU canvas descriptor reads** (about 22 `Table::get` on descriptors in
   `gpu_canvas.rs` / `gpu_canvas_ore/shader.rs`): lane D4's files on its own
   branch; forwarding is the same one-line change per site as `beginFrame`
   once D4 lands.
   The observing script has the `beginFrame` shape: a descriptor table
   with an `__index` that draws, passed to a GPU canvas create call during
   a draw.
3. **Other instances re-entered from a draw** (artboard `advance`, pointer and
   data calls on a nested artboard run that artboard's scripts through
   `nuxie-runtime`, which calls back into scripting with no loan): needs a
   `nuxie-runtime` API change.
   ```lua
   draw = function(self, renderer)
       _G.outer = renderer
       self.child:advance(0)       -- child's advance(): _G.outer:save()
   end
   ```
4. **The `draw` field lookup** (`table.get_field("draw")` in
   `call_draw_with_balance`) runs before this scope opens; inside an outer
   draw (a nested artboard) an `__index` on the inner instance's table
   could reach the outer renderer before, and cannot now.
   ```lua
   return setmetatable({}, {__index = function(_, k) _G.outer:save(); _G.outer:restore() end})
   ```

All four need a script that stashes a renderer in a global or upvalue and
uses it from a callback the host runs; no fixture, golden or upstream-ported
test depends on any of them (every renderer and golden check below is
green). No thread-local or ambient store closes them, per the brief.

## 4. Tests changed

Every renderer test passes unchanged except two that tested the deleted
registry's mechanism, which no longer exists:

- `renderer_scopes_ended_out_of_order_report_an_error_instead_of_panicking`
  dropped an outer scope guard before an inner one and checked the
  "scoped renderer stack order" error. Scopes are borrows now; that order
  does not compile. Replaced by
  `a_renderer_userdata_reaches_only_its_own_scope_while_that_scope_is_lent`
  (own token: ok; another scope's loan or no loan: "no longer valid"; after
  `end`: error).
- `nested_renderer_scope_reborrows_without_unlocking_its_parent` drove
  `ScopedRendererAccess`/`with_active_renderer` directly. Rewritten on
  `RendererScope`/`with_renderer_mut`; same assertions: the parent is not
  reachable while the nested scope is, the nested token dies with its
  scope, and the stream is `save save restore restore`.

Mechanical changes elsewhere: test seams and tests that opened a scope call
`call_lent`/`end(Some(&mut scope))` (`MeshInstancesTest::render`,
`lua_artboards.rs` tests).

## 5. Checks

Load averages are the 1/5/15-minute values at the start of each run
(another session's builds share the host; none of these checks is
timing-bound).

| Check | Result | Load at start |
|---|---|---|
| `runtime.py test -p nuxie-scripting` | 39/39 targets pass | 24.3 26.5 26.3 |
| same, `--features tools` | 39/39 pass | 8.0 10.2 13.7 |
| `runtime.py test -p nuxie --features scripting,test-support` | 41/42; the one red is `upstream_data_binding_images_direct` / `stateful_component_image_bind`, known red on main | 10.5 11.3 13.7 |
| `runtime.py test -p nuxie-luau-vm` | pass; `clippy -p nuxie-luau-vm --all-targets` clean | |
| `make luau-vm-diff` | 409 runs, 0 disagreements | 15.3 22.9 26.1 |
| `make luau-conformance` | 37/37 required OK, PASS | 19.4 23.5 25.9 |
| `make scripted-golden-compare` | 348 exact / 16 diverge, exit 0 | 10.0 16.8 22.4 |
| `make golden-compare` | 356 exact / 8 diverge, exit 0 | 11.3 16.4 21.1 |
| `make luau-vm-ratchets` | PASS; unsafe "counts went down"; panic lints `nuxie-luau-vm` 0, `nuxie-scripting` 473 (wasm publisher 438), "counts went down" | 10.0 16.8 22.4 |

**`lua_renderer.rs`, before (`a2e2271537`) and after**, measured with
`tools/ratchets/panic-lints.sh --baseline <scratch copy> crates/nuxie-scripting`
(through a wrapper that splits the diagnostics by file): `unsafe` 2 -> 0;
forced panic-lint sites 3 -> 0 (three `arithmetic_side_effects`, at base
lines 274, 339, 351: the `save_count` `+ 1`/`- 1`, now saturating). The
survey's table and my stop report said 5; the measured number at base is 3. Crate total 476 -> 473. `tests/ratchets/*` not edited.

**`make luau-bench`** (`LUAU_BENCH_CLOCK=instructions`, so "ms" are millions
of retired instructions; report only): before at `2c5dc49204` (base plus
docs, 0 dirty files, load 22.4 26.6 21.2), after at `676101d9ba` (0 dirty,
load 11.0 16.0 20.1). 89 benches, geomean after/before 0.9966, range
0.976-1.001; no row moves more than 3%. The driver now passes one more
`Option<&mut _>` through its call path; no measurable cost.


## 6. Not verified

- Promise forwarding and `beginFrame` forwarding have no dedicated test
  (both are the same `call_with`/`get_with` path the new test covers).
- The four not-forwarded sites in section 3 were not run as scripts; the
  examples are written from the code paths.

## Appendix A: the stop report (sections 1-4 of the first version)

### A.1. What `lend` is

`lend` (`crates/nuxie-scripting/src/luau/facade/state.rs:120`) does not lend
host data. It lends **the VM itself**: a host callback runs with the VM held
by the interpreter through `&mut HostCtx`; `lend` swaps the VM's contents
with a spare `vm::Vm` (`std::mem::swap`), parks the real VM in
`Shared::slot: RefCell<Option<Box<vm::Vm>>>` for the callback's duration, so
facade handles (`Table`, `Function`, ...) that reach the VM through the
`Rc<Shared>` slot work inside the callback, then swaps it back. Its signature
is `fn lend<R>(shared: &Shared, vm: &mut vm::Vm, f: impl FnOnce() -> R)`:
there is no payload parameter, and the only thing it stores is an owned
`Box<vm::Vm>`.

### A.2. Can it carry a `&mut dyn Renderer + 'a` as it is? No.

What it lacks: a channel for a non-`'static` borrow between the code that
starts a Lua call and the host callbacks that call reaches. Every candidate
channel is `'static` by type:

| Channel | Where | Bound |
|---|---|---|
| The lend slot | `Shared::slot` (`state.rs:22`) | `Box<vm::Vm>`, owned |
| App data | `Shared::app_data` (`state.rs:29`), `Compat::app_data` (`nuxie-luau-vm/src/embed/compat.rs:113`) | `Box<dyn Any>` / `Rc<dyn Any>`: `Any` is `'static` |
| Host callbacks | `BoxedCallback = Box<dyn Fn(&Lua, MultiValue) -> Result<MultiValue>>` (`state.rs:153`); `vm.create_host_function(name, impl Fn(&mut HostCtx<'_>) -> ... + 'static)` (`embed/mod.rs:373`) | `'static` closures; they receive only `&Lua` and the arguments |
| Scoped functions | `Scope::create_function` (`state.rs:~960`) | `F: 'static` (no lifetime erasure, unlike mlua's scope) |
| VM host state | `trait Host: 'static` (`nuxie-luau-vm/src/hosted.rs:35`); the embed VM is `HostedVm<Compat>` with `Compat` defined in the VM crate | `'static`; `HostedCtx<'a, H>`'s lifetime covers only the borrow of the VM, not host data |
| The `Lua` a callback gets | `Lua::borrowed(shared)` in `trampoline` (`state.rs:~909`) | an `Rc<Shared>`, `'static` |

So a `&'a mut dyn Renderer` cannot be stored anywhere a callback can read it
without erasing `'a`, which is what `lua_renderer.rs` does today with
`transmute` (now at `lua_renderer.rs:122`; the survey's `:112`) and
`NonNull::as_mut` (`:220`; the survey's `:184`). This is a type-level fact,
not a missing method: no facade-only change can provide it, because the
borrow has to cross the interpreter (`Function::call` -> `vm.call_raw` ->
frame loop -> host function), and the interpreter's types know nothing
non-`'static` from the host.

### A.3. The smallest safe change

#### A.3.1 In `nuxie-luau-vm` (embedding API)

Thread an optional **loan** through the driver, typed by the host, with no
`unsafe` and no dependency:

1. A loan family on the host side:
   ```rust
   pub trait LoanFamily: 'static {
       type Loan<'l>: ?Sized + 'l;
       /// Shortens the loan's lifetime to the borrow (for a generic GAT the
       /// compiler cannot do this coercion itself; for `dyn Trait + 'l` the
       /// body is just `loan`).
       fn shorten<'a, 'l: 'a>(loan: &'a mut Self::Loan<'l>) -> &'a mut Self::Loan<'a>;
   }
   ```
   `Compat` becomes `Compat<L: LoanFamily = NoLoan>` (so `embed::Vm` and
   `HostCtx` keep their names and defaults), or `Host` gains the associated
   family. Shortening keeps the existing single lifetime on `HostedCtx<'a, H>`
   and `PlainCtx<'a, H>`, so the signature of every host function in the VM's
   own libraries stays as it is.
2. `Hosted<'a, H>` (the frame loop's dispatch, `hosted.rs:~72`),
   `HostedCtx` and `PlainCtx` gain `loan: Option<&'a mut L::Loan<'a>>`, with
   `ctx.loan() -> Option<&mut L::Loan<'_>>`. Nested calls a host function
   makes through its context (`call_top`, `pcall_top`, `call_yieldable`,
   `pcall_yieldable`) reborrow the same loan.
3. Lent variants of the entry points that run Lua from Rust:
   `Function::call_lent(&self, vm, loan, args)` (`embed/mod.rs:1614`), and
   `resume_lent` (`:1831`, `run.rs:307`) if a draw may resume a coroutine from
   Rust; `HostedVm::enter`/`call_value` take the loan as an argument
   internally. The existing entry points pass `None`.

What `nuxie-luau-vm` keeps: 0 `unsafe`, no new dependency, no forced
panic-lint site (nothing here indexes or unwraps). Rough size: the driver
structs and 4-7 entry points; I estimate 200-400 lines, not measured.

Coroutines: a Lua coroutine resumed by Lua (`coroutine.resume`) runs inside
the same driver call and keeps the loan; one resumed from Rust only if Rust
uses `resume_lent`. A coroutine that yields while holding a renderer and is
resumed after the draw finds no loan: "Renderer is no longer valid.", which
is today's text for that case.

#### A.3.2 In the facade (`src/luau/facade/`), after that

1. `Function::call_lent(&self, renderer: &mut dyn Renderer, args)` (or a
   facade loan type that names the renderer) calling the VM's
   `call_lent` under `Shared::with_vm`.
2. `trampoline` reads `ctx.loan()` and hands it to the callback:
   `BoxedCallback` gains a loan argument, and `UserDataMethods` gets methods
   that receive it (renderer methods are the only users; the other
   registrations keep their shape through an adapter).
3. The loan carries a small stack of `(token, borrowed)` for the scopes it
   was reborrowed through, so a userdata whose token belongs to an enclosing
   scope gets today's "Renderer is already mutably borrowed by this callback"
   and an unknown token gets "Renderer is no longer valid.".

Then `lua_renderer.rs` loses `ACTIVE_RENDERERS`, `NEXT_RENDERER_TOKEN`,
`BROKEN_RENDERER_SCOPE`, `RENDERER_STACK_POISONED`, `ScopedRendererAccess`,
`ActiveRendererBorrow` and both `Drop` impls; `with_reborrowed_renderer`
(the artboard draw visitor, `lua_artboards.rs:282`) becomes a
`visitor.call_lent(renderer, ...)`. The scope openers that change:
`lua_renderer.rs:58` (`call_draw_with_balance`), `lua_transition.rs:151`,
`lua_artboards.rs:282,595,723,986`, and the `upstream-test-seams` /
test callers of `create_call_scoped_userdata` (`lua_renderer.rs:700,755,761`).

#### A.3.3 One behaviour the change does not preserve on its own

Today the renderer is reachable from **any** Lua code that runs on the
thread while the scope is open, because the stack is a thread-local. With a
loan it is reachable only from Lua code run through a call that carries the
loan. Lua-to-Lua calls inside the draw keep it. A Rust-to-Lua call made from
inside a draw callback through a plain facade handle (`table.get` hitting an
`__index` metamethod, a view-model listener fired synchronously from a setter
called during the draw, `tostring` via `__tostring`) runs without the loan;
a script that stashed the renderer in an upvalue or global and uses it there
would get "Renderer is no longer valid." instead of drawing. The fix is to
forward the loan at each such call site; I did not audit which exist (the
visitor and nested `call_draw` are the two I found, and both already take the
renderer explicitly). This needs a decision: accept the narrower reach (only
pathological scripts see it), or audit and forward.

### A.4. Alternatives considered and not taken

- **Record, then replay**: the draw call records renderer commands into an
  owned list and replays them into the `&mut dyn Renderer` after the call.
  No VM change, but it is a retained copy of draw data (paths, paints, images
  held past the script's own handles), it needs a recording `Renderer` for
  nested artboard draws (`draw_with_visitor`), and it changes when
  `restore` runs on error. Ruled out by the no-retained-copies rule.
- **Move the renderer into the slot**: impossible from `&mut dyn Renderer`
  (unsized, no replacement value) without changing the embedder API to hand
  over an owned renderer.
- **Facade-only threading**: the borrow must cross the interpreter (section
  2); no facade change reaches the callback.

