# Luau on clients: safe and panic-free (stage 1)

Status: designed 2026-11-14, not built. Facts:
`docs/evidence/luaur-client-path-survey-2026-11-14.md` (the survey; cited
below as S and a section number). Tree: `main` at `11fc709367`.

## 1. Goal

Levi, 2026-11-14: the whole luaur port is to be memory-safe and panic-free,
so a script or a file cannot crash a client, and so that the Luau code is
never the reason a client must be built with `panic = "unwind"`.

Stage 1 is the code that is linked into client builds. Stage 2 is the
compiler crates (`luaur-ast`, `luaur-bytecode`, `luaur-compiler`), which run
on build hosts and in tools only (S1.3).

What is linked into the Apple, Android and browser-audio artifacts from the
Luau side (S1.3):

| Crate | `unsafe` sites | Forced panic-lint sites |
|---|---:|---:|
| `nuxie-luau-vm` | 0 | 0 |
| `luaur-common` | 174 | 288 (341 on wasm) |
| `nuxie-scripting` | 19 shipping (7 native, 12 wasm) | 481 Apple, 480 Android, 446 wasm |

## 2. What "done" means

For every shipped variant of `nuxie-scripting` (`__apple`, `__android`,
`__publisher`):

1. `luaur-common` is not linked.
2. Zero `unsafe` outside two named host-boundary modules (section 5), and
   the crate denies `unsafe_code` everywhere else.
3. Zero forced sites for the seven panic lints the VM already holds at zero
   (`unwrap_used`, `expect_used`, `indexing_slicing`, `panic`, `unreachable`,
   `arithmetic_side_effects`, `unwrap_in_result`), ratcheted.
4. The four panic classes the lints do not see are closed by construction,
   each with its own check (section 7): conflicting `RefCell` borrows,
   unbounded recursion on script data, host allocation sized by a script,
   and panics in `Drop`.
5. The crate's tests and a fixed hostile-input corpus pass when built with
   `-Cpanic=abort`, where the two `catch_unwind` sites in the Luau layer
   (S5.2) do nothing. A panic then fails the run instead of becoming a Luau
   error.

Not a goal of stage 1: changing the shipped panic strategy. The rest of the
Apple device graph has about 11,900 forced panic-lint sites outside the Luau
code (S6: `nuxie-runtime` 4,985, `nuxie-schema` 3,138, `nuxie-renderer`
2,435, and others). Stage 1 removes Luau from that list; it does not empty
it.

Speed is parked. No caches. No `unsafe`. Behaviour scripts can observe does
not change except where a crash becomes an error.

## 3. Lane A: crash paths a script can reach today

These are bugs, not lint counts, and no `catch_unwind` contains them
(S3.3, S5.2). They go first, each with a test that fails before the fix.

1. **Deep tables posted to audio processors.** `to_shared` / `from_shared`
   (`vm/lua_audio_processors.rs:656,733`), `push` / `read`
   (`luau_audio_processor_engine/safe.rs:601,678`) and the codec
   (`luau_audio_shared_codec.rs:96,159`) recurse with a cycle check and no
   depth limit. A deeply nested table overflows the native stack.
   Fix: one limit set for a shared value (depth, node count, byte size),
   checked where the value is built from the script table and again where it
   is decoded from bytes. Use the host-command limits as the model
   (`host_commands.rs:18,49`: depth 32 by default, 64 hard cap, node and byte
   budgets) and the same error-code style. Report the numbers chosen.
2. **`ImageMeshInstances:resize(count)`** (`vm/lua_mesh.rs:197`) allocates up
   to 2^32 elements outside the VM's memory ceiling. Fix: a host-side budget
   charged before allocating, like the GPU canvas buffer budgets
   (`gpu_canvas.rs:2616`), and `try_reserve`.
3. **Audio stream sizes** (`vm/lua_audio_processors.rs:340-347`,
   `vm/lua_audio_samples.rs:518,523`): unchecked multiplication of
   descriptor fields before allocating. Fix: checked arithmetic, a stated
   maximum, `try_reserve`.
4. **Panics in `Drop`** (`vm/lua_renderer.rs:127-152`): an `expect` and an
   `assert_eq!` in two `Drop` impls; a second panic while unwinding aborts.
   Lane C replaces this structure; until then the drops must not panic.

For every other allocation in S3.3's table marked "not traced" or "none",
trace it and either show the bound or add one.

## 4. Lane B: `luaur-common` leaves the device graph

The device path uses four small enums, one 43-line `match` and one flag
that is always true (S2.1, S2.4). Everything else in the crate serves the
compiler.

1. `vm/bytecode.rs` (the product bytecode validator) takes its opcode
   numbers, opcode lengths, constant tags and type tags from
   `nuxie-luau-vm`, which already has them (`bytecode/mod.rs:21`). Export
   what is missing from the VM crate as plain data; do not copy tables into
   `nuxie-scripting`. The `u8` to opcode conversion is the VM's checked one,
   not `luaur-common`'s `transmute`.
2. The flag: `FFlag::LuauCallFeedback` is read once (`bytecode.rs:273`) and
   is true on every device. It becomes the constant it is. Delete
   `set_all_flags(true)` from `Lua::new()` (`luau/facade/state.rs:302`) and
   `ScopedAllFlags` from `validate_executable_luau_bytecode`
   (`vm.rs:3050`). This also removes the unsynchronized global write on
   every VM creation (S, surprise 1).
3. `luaur-common` becomes a dependency of the `compiler` feature and of the
   build script only. `luau-host` no longer names it.
4. The compiler must stay in step with the VM's tables. One test under the
   `compiler` feature asserts that the VM's opcode numbers, lengths and tags
   equal `luaur-common`'s, entry by entry.
5. Report whether the product validator and the VM's own loader check the
   same things. If the validator is a strict subset plus the version and
   signature rules, say so; merging them is a decision for the coordinator,
   not part of this lane.

Check: `bazelisk query` shows no `luaur-common__{apple,android,publisher}`
under the three shipped labels; the wasm kernel no longer exports
`luaur-common`'s `malloc`/`free` shims (S2.3) and still builds and passes
its probe.

## 5. Lane C: `unsafe` in shipped `nuxie-scripting`

19 sites (S4.1, S4.2). After this lane the crate has
`#![deny(unsafe_code)]`, with `allow` on exactly the modules named below.

1. **Renderer lifetime erasure** (`vm/lua_renderer.rs:112,184`): a
   `transmute` puts a `&mut dyn Renderer` into a thread-local stack so
   script userdata can reach it. Replace it with a scoped loan the borrow
   checker sees: the renderer is lent for the duration of the script call
   through the facade's existing `lend` mechanism
   (`luau/facade/state.rs:916`), and userdata holds only the token it holds
   today. No thread-local, no `Drop` that asserts stack order. If `lend`
   cannot carry a `&mut dyn Renderer` as it is, report what it lacks before
   changing it.
2. **Verified shader constructors** (`shader_asset.rs:348,417`): `unsafe fn
   from_verified_parts` in `nuxie-render-api` has a safe body; `unsafe` is
   used as a trust marker. Replace the marker with a proof value that only
   the verification step can construct, and make the constructors safe.
3. **Canvas texture wrapping** (`gpu_canvas_ore/canvas.rs:63`,
   `gpu_canvas_ore/mod.rs:211,222`): the trait methods take raw native
   texture pointers. Change the trait to take an owned, typed handle that
   the backend issued, so the call is safe and the pointer is dereferenced
   only inside the backend that owns it. The backend's own `unsafe` is
   outside this stage; say where it ends up.
4. **Browser audio host** (`vm/luau_audio_web.rs`, 12 sites): the JS import
   block, the buffer hand-off and `kernel_request` are a real boundary.
   Contain them in one module of safe wrappers that take and return slices
   and owned buffers; the host-absent stubs become ordinary safe functions.
   This module is one of the two allowed to contain `unsafe`. The second is
   whatever lane C.3 cannot move out of the crate, if anything; name it or
   report that there is none.

The 32 non-shipping sites (build script, tests, test seams) are the
compiler's C entry point and move with stage 2, except that the build
script and test helpers should call the safe `compile_bytes` path the facade
already uses (`luau/facade/state.rs:1054-1075`) if that removes them at no
cost.

## 6. Lane D: the seven lints to zero

481 sites in 39 files, effectively all device code (S3.1, S3.2). Method, as
in the VM: change the shape so the panic cannot be written, rather than
replacing each `x[i]` with `x.get(i).ok_or(..)?`.

- Fixed-size data is a fixed-size type: a matrix is `[f32; 16]` or named
  fields, read by destructuring; a row or column index is an enum.
- Byte parsing goes through one cursor type with checked readers
  (`bytecode.rs`, `buffer_ext.rs`, `shader_asset.rs`, the audio codecs).
- Sizes and offsets from scripts or files use checked arithmetic and return
  the binding's ordinary script error.
- An internal invariant that "cannot fail" is expressed in the types (an
  enum with no impossible case, a non-empty type) or returns an error.
  No `expect` with a reassuring message.
- Division: 35 candidate integer divisions (S3.2 note); each gets a
  non-zero divisor by type or a checked division.

Four writers in parallel, one worktree each, split by file so they do not
overlap:

| Lane | Files | Sites (Apple) |
|---|---|---:|
| D1 audio | `vm/luau_audio_shared*.rs`, `luau_audio_samples.rs`, `lua_audio_samples.rs`, `lua_audio_processors.rs`, `luau_audio_processor_engine*`, `lua_audio.rs`, wasm audio files | about 160, plus 30 wasm-only |
| D2 values | `vm/lua_mat4*.rs`, `lua_mat2d.rs`, `lua_vec2d.rs`, `lua_text.rs`, `buffer_ext.rs`, `lua_mesh.rs`, `lua_path.rs`, `lua_canvas.rs`, `lua_renderer.rs` | about 165 |
| D3 core | `vm.rs`, `vm/bytecode.rs`, `view_model.rs`, `native_registration.rs`, `host_commands.rs`, `listener_invocation.rs`, the facade, the rest of `vm/` | about 100 |
| D4 GPU | `gpu_canvas.rs`, `gpu_canvas_ore/*`, `shader_asset.rs` | about 60 |

Lane A lands before D1 and D2 start on the same files; lane C.1 lands before
D2 touches `lua_renderer.rs`.

Ratchet: `crates/nuxie-scripting` is added to
`tests/ratchets/panic-lints-baseline.txt` and `unsafe-baseline.txt` at its
current counts in the first commit of this stage, and each lane lowers it.
The panic-lint tool lints the default label only; extend it to lint a named
variant so the wasm-only files (`__publisher`) are counted too.

## 7. Lane E: what the lints do not see

1. **`RefCell` borrows.** 395 shipping `borrow` / `borrow_mut` calls against
   38 `try_borrow` (S3.3). On one thread a borrow panics only if a guard is
   alive while control re-enters code that borrows the same cell, which
   here means: a guard held across a call into the VM (a script call, a
   resume, or any facade operation that can run a metamethod). First step is
   an inventory, not a fix: every guard that is alive across such a call,
   with file and line, and which facade operations can run script code at
   all. I design the fix from that list. Converting 395 sites to
   `try_borrow` is not the design.
2. **Recursion on script data.** Lane A fixes the known cases. The survey
   did not search mutual recursion (facade value conversion, view-model
   walks). Search it; every recursive walk over script-built or file-built
   data has a depth limit or is rewritten as a loop.
3. **Host allocation sized by a script.** Lane A's rule becomes general:
   every host-side allocation whose size a script or a file chooses is
   charged to a budget before it happens. List them all (54 allocation
   sites in shipped code, S3.3) with their bound.
4. **`Drop`.** No `Drop` impl in the crate can panic. List them.
5. **`copy_from_slice`** (12 shipped sites): each has lengths equal by
   construction (fixed-size arrays) or goes through a checked copy.

## 8. Lane F: proof

1. **Abort build.** The `nuxie-scripting` test suite and the hostile corpus
   run under `-Cpanic=abort` as a standing gate. Establish how to run tests
   that way through `tools/bazel/runtime.py` (the abort probe,
   `examples/safe_vm_abort_probe.rs`, already builds so). Tests that assert
   a panic move to the unwind run or are rewritten to assert an error.
2. **Hostile corpus.** A fixed, committed set of inputs, one per entry
   point in S7: truncated and corrupted bytecode (every count and offset
   field at its extremes), each host function called with wrong types,
   extreme sizes, `NaN`, negative and huge indices, deeply nested and cyclic
   tables, re-entrant calls (a callback that calls back into the same
   object), and a script that exhausts its memory limit inside each
   allocating host call. Each case must end in a script error or a clean
   result. It is a fixed set, not an open-ended fuzzer.
3. **The two `catch_unwind` sites** in the Luau layer
   (`facade/state.rs:917`, `luau_audio_processor_engine/safe.rs:380`) stay
   while the shipped binaries are built with unwind, since the rest of the
   binary can still panic through a host callback. They are no longer what
   the Luau layer's safety rests on; the abort gate shows that.

## 9. Order

1. Ratchet entries for `nuxie-scripting` (one small commit).
2. Lane A and lane B, in parallel (different files).
3. Lane C.1 (renderer loan), then lanes D1 to D4 and C.2 to C.4 in
   parallel; lane E.1's inventory at the same time, read-only.
4. Lane E fixes, from my design on the inventory.
5. Lane F gates on the merged result.

Standing gates on every landing: `make luau-vm-ratchets`, `make
luau-vm-diff`, `make luau-conformance`, `nuxie-scripting` tests, `nuxie`
tests, both golden compares, and the shipped sizes reported against the
budgets (Android has 2.3% headroom).

## 10. Stop rules

- If a lint site can only be removed by changing what a script observes
  (a different error message counts), stop and report the site.
- If removing an `unsafe` needs a change outside `nuxie-scripting`,
  `nuxie-render-api` and the trait named in lane C.3, report before making
  it.
- If a fix needs a cache, a pool or a retained copy, report instead.

## 11. Open questions

1. Does anything on a device create `ScriptVm`s on more than one thread
   (S, surprise 1)? Lane B removes the hazard either way; the answer says
   whether it was live.
2. Is the product bytecode validator needed beside the VM loader's own
   validation (lane B.5)?
3. Limits for shared audio values (lane A.1): the numbers.
4. After stage 1, which is next: stage 2 (compiler crates: 1,993 `unsafe`
   sites, 1,766 panic-lint sites, tools and build hosts only), or the rest
   of the device graph (about 11,900 panic-lint sites, which is what
   actually decides the panic strategy)? Levi's call.

## 12. Addendum, 2026-11-14: what lanes A, B, C.1 and E.1 changed in this design

Evidence: `docs/evidence/luaur-client-path-lane-{a,b,c1}-2026-11-14.md`,
`docs/evidence/luaur-client-path-lane-e1-inventory-2026-11-14.md` (E1).

### 12.1 Lane C.1: the renderer loan goes through the VM's embedding API

Section 5 item 1 was wrong: the facade's `lend` lends the VM, not host
data, and every route to a host callback is `'static`. Approved instead
(lane C.1 evidence, section 3): a loan family on the host type
(`LoanFamily` with a generic associated type), an optional loan carried by
the driver and the host-function context, and lent variants of the entry
points that run Lua from Rust. `nuxie-luau-vm` stays at 0 `unsafe`, 0 forced
panic-lint sites, no dependency. The renderer is then reachable only from
Lua run through a call that carries the loan. Rust-to-Lua calls made inside
an open renderer scope are audited and the loan forwarded wherever the call
site has it; any that cannot forward are listed. Both golden compares must
stay exact.

### 12.2 Lane E, from the inventory

`RefCell`: no guard in the crate is alive across an operation that runs
Luau in a way that can conflict today (E1 section 2). So there is no borrow
redesign. What remains is mechanical and goes to the owner of each file:
end the borrow before creating a VM object (E1 2(a) rows 2 to 14), and
replace `set_app_data` followed by `expect` with an install that returns an
error (E1 section 3). E1 2(a) row 1 is in `nuxie-runtime` (a property
listener runs while the value's slot is mutably borrowed): reproduce it,
then report; the fix is outside this crate.

**E2a, recursion and hangs** (files: `luau/facade/error.rs`,
`vm/native_registration.rs`, `vm/buffer_ext.rs`, the budget reset in
`vm.rs`):

1. `Error::CallbackError` never wraps another `CallbackError`. Wrapping an
   error that is already a callback error keeps the existing root cause and
   the existing traceback. `Display` prints only the cause today, so the
   text is unchanged; depth is at most one, so `Display`, `is_structured`
   and `Drop` need no recursion. E1's script (4.1) is the test.
2. `sort` becomes an iterative topological sort with an explicit stack. It
   also stops reporting cycles that do not exist (E1 4.2: 9,560 of 76,871
   random acyclic graphs); that is a bug fix scripts can observe, and the
   minimal input in E1 is the test. A real cycle keeps today's error text.
3. Nested host entries do not reset the execution budget: a nesting depth,
   and only the outermost entry resets (E1 section 3). A test with a script
   that re-enters in a loop must hit the cap.
4. `buffer.stridedcopy` and `buffer.convert` with a zero stride or a huge
   count: bounded work, checked against the interrupt budget, no
   whole-buffer copy per iteration.

**E2b, one host memory budget per VM** (new module in `vm/`, applied in
`lua_path.rs`, `lua_mesh.rs`, `lua_text.rs`, `lua_rive_base.rs`,
`lua_image_decode.rs`, `lua_font.rs`, `renderer.rs`, `lua_data_value.rs`,
and the listener lists in `view_model.rs`):

The VM's memory ceiling covers the VM's heap only. Host-side containers a
script can grow (E1 section 5, 26 rows) have no bound, and one of them
doubles per call (`Path:add(self)`: 14 GB in 40 calls). The shape: one
`HostMemoryBudget` per script VM, in bytes, owned by the VM's host state
and reachable from every binding. A binding charges before it grows and the
payload refunds when it drops; growth uses `try_reserve`. The mesh budget
lane A added becomes a charge against it, not a second budget. Counts that
are not bytes (listeners per property, processors per VM, image decodes in
flight) are small fixed caps beside it. `decodeImage` uses the bounded
decoder. `print` charges or truncates its line. Default: the same number as
the VM's own default ceiling (`DEFAULT_SCRIPT_VM_MEMORY_BYTES`, 64 MiB),
settable through `ScriptExecutionLimits` like the VM ceiling. Exceeding it
is an ordinary script error, one text for all bindings. No fixture may
change result; report the largest charge any fixture reaches.

**Audio follow-ups** (to the audio lane, after D1): iterative drop for
snapshot and ring hold chains (E1 4.4, reproduced abort), `HostStream::
restart` on an oversized record (hang), the processor VM's `pcmReader`
charged and freed, a cap on processors per VM and on the init queue, holds
charged at their size.

**GPU follow-ups** (to the GPU lane, after D4): the ORE path charges a
per-VM GPU byte budget at create and refunds on drop, canvas and texture
dimension caps as the direct path has, borrows of the ORE context ended
before `create_userdata` (E1 2(a) rows 2 to 8).
