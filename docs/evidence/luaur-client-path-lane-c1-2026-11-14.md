# Lane C.1 (renderer loan): stopped at the `lend` question

Date: 2026-11-14. Branch `claude/luaur-lane-c1-renderer-loan-200a01`, based
on `9eb238d58f` (= `levi/luaur-client-safe`). Design:
`docs/luaur-client-path-safe.md` section 5 item 1.

**Outcome: no code changed.** The design assumed the facade's `lend` could
lend a `&mut dyn Renderer` for a script call. It cannot, and nothing else in
the facade can either: every route from a Rust call site to a host callback
passes through `'static` storage. Removing the lifetime erasure safely needs
a change to `nuxie-luau-vm`'s embedding API (described below). The brief
says to describe such a change and wait, so this lane stops here.

## 1. What `lend` is

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

## 2. Can it carry a `&mut dyn Renderer + 'a` as it is? No.

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

## 3. The smallest safe change

### 3.1 In `nuxie-luau-vm` (embedding API)

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

### 3.2 In the facade (`src/luau/facade/`), after that

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

### 3.3 One behaviour the change does not preserve on its own

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

## 4. Alternatives considered and not taken

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

## 5. Counts (unchanged, nothing was edited)

`crates/nuxie-scripting/src/vm/lua_renderer.rs` at `9eb238d58f`:
`unsafe` blocks: 2 (`:122` transmute, `:220` `as_mut`). Forced panic-lint
sites: 5 per the survey's table (`docs/evidence/luaur-client-path-survey-2026-11-14.md`
row `src/vm/lua_renderer.rs | 5 | 0 | 2 | 0 | 0 | 0 | 3 | 0`); I did not
re-run `tools/ratchets/panic-lints.sh`, since nothing changed.

## 6. Not verified

- No build or test was run (no code changed). The type-level claims in
  section 2 are from reading the signatures cited, not from a compile
  experiment.
- The size estimate in 3.1 is not measured.
- Which Rust-to-Lua calls happen inside an open renderer scope (3.3) was not
  audited.

## 7. Where the design was wrong

Section 5 item 1 says the renderer is lent "through the facade's existing
`lend` mechanism". `lend` lends the VM, not host data, and has no payload; no
facade mechanism can carry a non-`'static` borrow to a callback. Item 1
needs the VM API change in 3.1 first, and the reach question in 3.3 decided.
Load average when this was written: 8.80 7.99 10.16 (no measurements taken).
