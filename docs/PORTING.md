# PORTING.md — The C++→Rust Idiom Codex

This is the translation manual for porting the Rive C++ runtime (reference at
`/Users/levi/dev/oss/rive-runtime`) into this Rust workspace. It is the
authoritative distillation of the patterns worked out across M0–M8 and recorded
in `docs/v2-status.md`, `docs/v2-log-archive.md`, and `docs/porting-map-v2.md`.

**Who this is for.** (a) Phase R agents about to mechanically translate the
~26k-line C++ renderer algorithm layer, and (b) new contributors. It teaches the
*idioms* — how a C++ construct becomes a Rust one here — so you translate the
same way the existing code did, and so your invalidation, float math, and error
handling stay byte-compatible with the C++ oracle.

**The prime directive.** Correctness is judged by `make golden-compare`: a Rust
run must produce a render-call stream identical to the C++ runtime's. Every rule
below exists to keep that stream exact. When in doubt, do what the C++ source
does at the same site — *including its evaluation order and its guards* — not
what is idiomatic Rust.

**Ground rules that shape every idiom (from `docs/porting-map-v2.md`):**

- Port *code, not behaviors*: one C++ class/file, translated coarsely in one
  sitting, with a comment naming its C++ source. Goldens judge correctness, not
  you; mark uncertain lines `// TODO(golden):` rather than researching each one.
- During Phase R's mechanical renderer translation, `nuxie-schema` and
  `nuxie-binary` are frozen — do not touch them. A Phase S upstream-sync cycle
  may regenerate schema artifacts and update the binary decoder when the
  upstream object model changes; those edits require the sync-map inventory,
  generated-artifact checks, and both normal and forced-scripted goldens.
- Never add skip/cache logic, widen a tolerance, or restructure float math for
  performance unless it mirrors an audited C++ gate. The golden harness only
  samples corpus timelines; invented invalidation breaks the timelines it does
  not sample.

---

## 1. Ownership & Type Mappings

### 1.1 `rcp<T>` / reference counting → arena ownership + dense slots

C++ heap objects are `rcp<Core>` with per-object reference counting. In Rust the
`ArtboardInstance` is the **sole owner** of a flat arena; objects are plain `Vec`
entries addressed by a dense `local_id` index. No `Rc`/`Arc` per object.

```rust
// crates/nuxie-runtime/src/objects.rs:61
pub(crate) struct InstanceObjectArena {
    objects: Vec<Option<InstanceObjectStorage>>,   // indexed by dense local_id
}
```

`ArtboardInstance` (`crates/nuxie-runtime/src/artboard.rs:61`) holds `slots`,
`objects`, and `components` side by side, each a `Vec` indexed by `local_id`.
Lifetime is the arena's lifetime; there is no shared ownership to reason about.

**Rule:** when C++ hands out an `rcp<T>` or stores a `T*`, store the object in
the arena and pass its `local_id` (see 1.2). Cloning an artboard clones the
arena, not a graph of refcounts.

### 1.2 Raw pointer graphs → typed indices (`local_id` / `global_id`)

C++ `Component* parent()`, `std::vector<Component*> children`, and `m_Dependents`
back-references become index fields. There are two id spaces:

- **`local_id: usize`** — dense index within one artboard's arena.
- **`global_id: u32`** — index into the whole file's object table.

```rust
// crates/nuxie-graph/src/lib.rs:325
pub struct ComponentNode {
    pub local_id: usize,
    pub global_id: u32,
    pub parent_local: Option<usize>,     // was: Component* parent()
    pub children: Vec<usize>,            // was: std::vector<Component*>
    pub constraint_locals: Vec<usize>,
    pub dependent_locals: Vec<usize>,    // was: m_Dependents
    ...
}
```

These are plain `usize`/`u32`, **not** distinct newtypes — the `local_`/`global_`
prefix and field position encode which table each index addresses. A null pointer
becomes `Option<usize>`; an empty child list becomes an empty `Vec`. When you
port a C++ traversal, translate `->` chases into arena lookups by the stored
index, and translate `nullptr` checks into `Option` matches (see 3.1).

### 1.3 Virtual dispatch → enum+match *or* trait

Two distinct C++ patterns map to two distinct Rust ones. **Choose by whether the
set of implementations is closed and file-defined, or open and pluggable.**

**Closed set of file-defined variants → `enum` + `match`.** Keyframe
interpolators, converters, constraints, and object-type storage are all closed
sets known at build time.

```rust
// crates/nuxie-runtime/src/animation.rs:16
pub(crate) enum RuntimeInterpolator {
    CubicEase { x1: f32, y1: f32, x2: f32, y2: f32 },
    CubicValue { .. },
    Elastic { amplitude: f32, period: f32, easing_value: u64 },
}
// dispatch at animation.rs:109 — replaces KeyFrameInterpolator::transform() vtable
```

**Open / externally-implemented interface → `trait`.** The renderer backend, the
factory, and the scripting VM are pluggable (FFI backend, host VM), so they stay
traits mirroring the C++ abstract base classes.

```rust
// crates/nuxie-render-api/src/lib.rs:310 — mirrors abstract rive::Renderer
pub trait Renderer {
    fn save(&mut self);
    fn restore(&mut self);
    fn draw_path(&mut self, path: &dyn RenderPath, paint: &dyn RenderPaint);
    ...
}
```

Also `Factory` (`nuxie-render-api/src/lib.rs:338`) and
`ScriptingVm`/`ScriptInstance`/`ScriptHost` (`nuxie-runtime/src/scripting.rs`).

### 1.4 `std::vector` clear-and-refill → retained `Vec` by dense slot

C++ draw loops keep a `std::vector` across frames, `clear()`ing to reuse
capacity. The Rust port does the same, and additionally indexes retained caches
by dense `local_id`, invalidating by graph identity rather than reallocating:

```rust
// crates/nuxie-runtime/src/draw.rs:5912
struct RuntimePathGeometryCommandSlots {
    graph_global_id: Option<u32>,
    by_local: Vec<Option<RuntimeCachedPathGeometryCommands>>,
}
// slot_mut: clear() when graph changes, else resize_with to grow; never per-frame realloc
```

There are ~11 `clear()`-reuse sites in `draw.rs`. Retained render paths follow
the same shape (`runtime_cached_retained_render_path`, `draw.rs:9759`): rebuild
in place when the cache key changes, reuse otherwise. **Do not** allocate a fresh
`Vec`/`RenderPath` per frame — that is the M7 churn the retention campaign
removed.

### 1.5 C++ inheritance → generated `InstanceObjectStorage` + schema-key model

Rive's C++ object model is a deep generated class hierarchy
(`NodeBase : ContainerComponent : ...`) with `virtual coreType()`, per-property
`xChanged()` hooks, and a `CoreRegistry` key→field deserialization switch. The
Rust port **flattens** this. `crates/nuxie-runtime/build.rs` generates (included
at `objects.rs:9`):

- An `enum InstanceObjectStorage` with one variant per object type, replacing the
  vtable/typeKey RTTI. `type_key` dispatch is a generated `match`.
- Per-type `*Object` structs whose properties are `Option<T>` fields keyed by the
  numeric schema property key. Getters/setters are generated `match property_key`
  arms.

The change-notification hook (`xChanged()`) becomes a shared generic helper that
early-returns when the value is unchanged — this is load-bearing (see 3.6):

```rust
// crates/nuxie-runtime/src/objects.rs:51
pub(crate) fn set_optional_field<T: PartialEq>(field: &mut Option<T>, value: T) -> bool {
    if field.as_ref().is_some_and(|current| current == &value) {
        return false;                 // unchanged: no write, no change hook
    }
    *field = Some(value);
    true                              // caller fires the change hook on `true`
}
```

`InstanceObjectArena::set_property_value` (`objects.rs:245`) validates the field
kind against the schema, handles bitmask-passthrough packing, then routes to the
generated per-type setter. Schema metadata (`Property { key, runtime_type,
stores_field, bitmask_passthrough, ... }`) lives in `crates/nuxie-schema`.

---

## 2. The Dirt / Epoch Architecture

This is the subsystem most easily broken by well-meaning Rust idioms, and the one
where a wrong change passes golden tests today and corrupts an unsampled frame
tomorrow. Read this section before touching any invalidation.

### 2.1 `ComponentDirt` bits

A bit-exact port of C++ `include/rive/component_dirt.hpp` — a hand-rolled `u16`
newtype (not `bitflags`), with the *same bit positions and the same aliases*:

```rust
// crates/nuxie-runtime/src/components.rs:18
pub struct ComponentDirt(pub u16);
impl ComponentDirt {
    pub const PATH: Self = Self(1 << 4);
    pub const TEXT_SHAPE: Self = Self(1 << 4);   // aliased on purpose (C++ does this)
    pub const SKIN: Self = Self(1 << 4);         // aliased
    pub const VERTICES: Self = Self(1 << 5);
    pub const WORLD_TRANSFORM: Self = Self(1 << 7);
    pub const PAINT: Self = Self(1 << 9);
    ...
    pub const FILTHY: Self = Self(0xFFFE);
}
```

Preserve the aliases and the `0xFFFE` value exactly — they are part of the
protocol. Dirt is stored per-component (`components.rs:448`, initialized
`FILTHY`) and once at artboard level.

### 2.2 The epoch counters

Dirt drives *intra-frame update ordering*. Epochs are the *cross-frame retained-
cache invalidation keys* — `u64` fields on `ArtboardInstance`
(`artboard.rs:117`), all initialized to `1`, all bumped with `wrapping_add(1)`
(`artboard.rs:1162`).

| Epoch | Bumped by | Read by (what it lets you skip) |
|---|---|---|
| `cache_epoch` | `mark_changed()` — nearly every property write | Paint-prep skip (`draw.rs:829`), `can_skip_prepared_frame` (`draw.rs:5790`), paint-config currency `is_current` (`draw.rs:5691`) |
| `prepared_epoch` | `mark_prepared_changed()`; **cascaded** from layout/path/draw_order/render_opacity bumps | Rebuild of the whole prepared draw-command list (`prepared_artboard_frame`, `draw.rs:6619`) |
| `path_epoch` | `mark_path_changed()` (also bumps prepared) | Shape-paint / path-geometry command caches (`draw.rs:3430`, `7004`) |
| `layout_epoch` | `mark_layout_changed()` (also bumps prepared) | Retained Taffy layout bounds (`draw.rs:6691`, `artboard.rs:927`) |
| `draw_order_epoch` | `mark_draw_order_changed()` (also bumps prepared) | Drawable re-sort (`sorted_drawable_order_frame`, `draw.rs:6658`) |

A sixth, **derived** epoch — `nested_epoch` — is *not* stored: it is an FNV-1a
hash over nested-artboard draw commands and their children's `cache_epoch`,
computed on demand (`runtime_nested_paint_preparation_epoch`, `draw.rs:946`).

`prepared_epoch` is the roll-up "topology changed" counter; `cache_epoch` is the
coarse "anything changed" counter. Every retained sub-cache is a
`if cached.key != key { rebuild }` gate keyed on the matching epoch.

### 2.3 Dirt vs. epoch — when to use which

- Use **dirt** to schedule work *inside* `advance`/`update`: `add_dirt`
  (`artboard.rs:1295`) sets bits and cascades to `dependent_locals`;
  `update_components` clears `COMPONENTS` and walks dependents.
- Use an **epoch** to let `prepare`/`draw` *skip rebuilding a cross-frame cache*.
- The bridge is `add_dirt`: it translates newly-set dirt bits into the correct
  epoch bumps. **This translation is the fence.**

### 2.4 THE FENCE RULES

**A dirt bit does not blindly bump every epoch.** The crux is
`component_dirt_affects_path_epoch` (`artboard.rs:2678`), anchored to a named C++
function:

```rust
// C++ src/shapes/path.cpp::Path::update rebuilds geometry for path/nslicer dirt,
// and only for world-transform dirt when a deformer is present. Plain transform
// animation is applied at draw time and must NOT churn retained path storage.
!(dirt & (PATH | VERTICES | LAYOUT_STYLE | N_SLICER)).is_empty()
```

So `WORLD_TRANSFORM`/`TRANSFORM` dirt bumps only `prepared_epoch`, never
`path_epoch` — animated transforms move the shape without invalidating cached
geometry. A dedicated test enforces it
(`world_transform_dirt_invalidates_prepared_frame_without_rebuilding_paths`,
`artboard.rs:3219`).

The write-side gates are deliberately *narrow whitelists / deny-lists*, each
citing the C++ gate it mirrors:

- `property_affects_effect_path_epoch` (`artboard.rs:2691`) — only
  TrimPath/DashPath/Feather named keys bump `path_epoch`.
- `property_may_affect_prepared_frame` (`artboard.rs:2709`) — a deny-list of
  ~50 animation/state-machine/data-bind/keyframe types that return `false`; plus
  special cases (`NestedArtboard` only on `artboardId`; `SolidColor` bumps unless
  it is the color value). Unknown types default `true`.
- `property_affects_layout` (`artboard.rs:1228`) — per-type layout whitelist.

The invariant, stated as a rule:

> **Never bump an epoch that the audited C++ gate for that property would not
> touch. If a sibling setter bumps and yours does not — or vice versa — one of
> them is wrong.**

Generated property setters all route through this fence
(`set_bool_property`/`after_double_property_set`/`set_uint_property`,
`artboard.rs:605–738`): `mark_changed()` for `cache_epoch`, then the *gated*
`mark_prepared_changed_for_property`, `mark_layout_changed_for_property`, and a
whitelisted `mark_path_changed`. `view_model.rs` setters never bump epochs
directly — they resolve a data-bind path and delegate into these artboard
setters, so all invalidation flows through the single fence.

### 2.5 Cautionary case studies — the five drift bugs

The M8 adversarial audit (`docs/v2-status.md` items 21–25) found five bugs. **All
five are the same mistake:** an epoch/invalidation that diverges from the audited
C++ gate — the exact failure this section exists to prevent. Study them; do not
reintroduce their shape. (Check the status log for current fix state before
assuming any is resolved.)

1. **Shallow collapse propagation** (item 21). A commit trusted each component's
   *local* collapsed flag but Rust collapse propagation was not full-subtree on
   two paths, unlike C++ `ContainerComponent::collapse`. Statically-inactive solo
   branches drew on a fresh instance. Goldens missed it because corpus solos are
   SM-driven (full-tree path) or shallow. *Lesson: a fast local check is only
   valid if the propagation feeding it is as deep as C++'s.*
2. **Missing `mark_mutated()` on three view-model setters** (item 22).
   `set_number_by_property_index`, `set_enum_…`, `set_artboard_…` mutated without
   the invalidation every sibling setter fires — the write never reached the
   bound artboard. *Lesson: if every sibling bumps and one does not, it is an
   accidental omission, not an optimization.*
3. **World-space gradients go stale under keyed transform animation** (item 23).
   `set_transform_property_with_key` never called `mark_changed` — the one gap in
   otherwise-complete `cache_epoch` coverage. World-space paints bake
   `shape_world` into shader endpoints; the shader kept pre-move endpoints.
4. **Stateful interpolator bindings never apply at `advance(0)`** (item 24). The
   SM apply phases skipped initialized stateful bindings when `elapsed == 0`, but
   C++ runs `updateDataBinds` unconditionally (only the *time step* is a no-op at
   0). *Lesson: `elapsed == 0` skips the advancer step, not the apply.*
5. **Fill rule replays an import-time snapshot** (item 25). Draw commands baked
   `fill_rule` at graph-build time; a runtime `Fill.fillRule` write bumped every
   epoch correctly and still rendered the load-time rule. C++ reads the live
   property every draw. *Lesson: no epoch bookkeeping can save a value you
   snapshotted at import instead of reading live at draw.*

The recurring pattern — "every sibling bumps this epoch, this one forgot" —
appears three times (bugs 2, 3, and the item-21 asymmetry). When you add a
setter, grep its siblings and match them exactly.

---

## 3. Cross-Language Semantic Traps

Release is `panic = "unwind"` (`Cargo.toml:56`) because luaur implements Luau's
protected-error boundary with `panic_any` / `catch_unwind`; using abort would
turn ordinary authored `pcall` errors into process termination. That exception
does **not** make panics an acceptable runtime control-flow mechanism: a panic
that escapes the protected scripting boundary can still terminate an embedder
at an FFI boundary. The importer accepts many degenerate-but-valid files; the
runtime must not assume more than the importer guaranteed.

### 3.1 Null-tolerant pointer flow → guarded `Option`, never `unwrap`

C++ threads `nullptr` through pointer flows and checks late. Port those to
`Option` and *keep the guard where C++ has it* — never `unwrap()`/`expect()` on
anything reachable from an imported file or a hostile C-ABI value. The M8
semantic sweep (`docs/v2-status.md` item 20) confirmed the codebase is unusually
defensive here: the binary reader is fully `.get()`-based, and every suspect
`len()-N` / modulo site sits behind a faithful C++ guard port. Preserve that when
you add code.

### 3.2 Float comparison → `total_cmp`, never `partial_cmp().unwrap()`

There is **zero** `partial_cmp().unwrap()` in the tree, and sorts use
`f32::total_cmp` for a total, deterministic order across all bit patterns:

```rust
// crates/nuxie-runtime/src/draw.rs:13554
stops.sort_by(f32::total_cmp);          // gradient / nslicer stop sort
```

C++ sorts gradient stops with `operator<` (a partial order; NaN/±0 unspecified).
`total_cmp` trades exact-C++-order-on-degenerate-input for reproducibility. This
is deliberate.

### 3.3 Saturating casts vs C++ UB

`static_cast<int>` of an out-of-range/NaN/inf float is UB in C++; Rust's `as i32`
**saturates** (clamps, NaN→0). The project treats this as a *safer deliberate
divergence*. The only constructible true divergence found:

```rust
// crates/nuxie-runtime/src/animation.rs:801 — PingPong direction
let direction = (seconds / duration) as i32 % 2;   // duration==0 → inf as i32 saturates
```

Recorded policy (`docs/v2-status.md` item 20 #10): document the float→int
saturating-cast policy and add NaN/inf fixtures so the divergence surfaces
deliberately. When you port a float→int cast, know that Rust will not reproduce
C++ UB — usually a feature, occasionally a divergence to guard (e.g. add a
`duration == 0.0` guard). Byte-clamp sites round-then-saturate on purpose:
`(255.0 * opacity.clamp(0.0, 1.0)).round() as u8` (`draw.rs:11938`).

### 3.4 Integer overflow policy

Two profiles, one landed and one planned:

- **Production (landed):** `[profile.release] panic = "unwind"`
  (`Cargo.toml:56`) for luaur's protected scripting errors, with
  `overflow-checks` at its default (off) → wrapping in release. Intentional
  wrap for hashing/epochs uses explicit `wrapping_add`; non-scripting runtime
  code must still avoid reachable panics, and FFI entry points must not allow
  unwinds to escape.
- **Tests/fuzz (planned, not yet a committed profile):** a hardened profile with
  `overflow-checks = true` (`docs/v2-status.md` item 20 #7). **TODO:** confirm
  whether this profile has landed before relying on it.

At the parser/hostile-input boundary, use explicit `checked_*` and reject on
overflow (e.g. `bytecode.rs`, `nuxie-binary/src/lib.rs:12689`,
`draw.rs:13136` `point_count.checked_mul(2)`). Do not let a mutated file's length
field wrap into a small allocation.

### 3.5 `debug_assert` must not carry side effects

`debug_assert*` operands are pure reads only — the assert is a documentation /
early-signal guard, never load-bearing logic (it vanishes in release):

```rust
// crates/nuxie-runtime/src/draw.rs:7383
debug_assert_eq!(target.len(), bytes.len());   // pure .len() reads
target.copy_from_slice(bytes);                 // would panic anyway on mismatch
```

*(The rule is expressed structurally, not as a prose sentence; every
`debug_assert*` in the runtime reads pure values only. Keep it that way.)*

### 3.6 The equality early-out generated-setter pattern (why steady frames matter)

Every generated/bindable setter returns `bool` (changed?) and early-returns when
the value is unchanged:

```rust
// crates/nuxie-runtime/src/state_machine/bindables.rs:237
pub(crate) fn set_value(&mut self, value: f32) -> bool {
    if self.value == value { return false; }
    self.value = value;
    true
}
```

This mirrors C++'s generated `if (m_Field == value) return; ...; fieldChanged();`.
The `== value` guard is **load-bearing**: a data bind that re-writes the same
number every frame must produce *no* dirt and *no* recompute, or a static/steady
frame churns caches and (worse) can bump an epoch it shouldn't. When you add a
setter, thread the `-> bool` "did it actually change" signal to the caller
instead of dirtying unconditionally.

---

## 4. Float-Exactness Lore

The golden comparator uses a tight numeric epsilon (`1.3e-4`), so geometry math
must match C++ *rounding*, not just C++ *value*. Two runtimes that compute the
"same" transform two different ways diverge by ulps that fail the diff.

### 4.1 No reassociation, no fast-math

Never reorder a float expression, distribute a multiply, or enable fast-math for
performance. Translate the C++ arithmetic *in the C++ grouping*. The grouping is
part of the contract:

```rust
// crates/nuxie-runtime/src/components.rs:390 — Mat2D::mapPoints
// The grouping matters for cancellation-heavy local path composition.
if b == 0.0 && c == 0.0 { (a.mul_add(x, e), d.mul_add(y, f)) }
else { (a.mul_add(x, c.mul_add(y, e)), d.mul_add(y, b.mul_add(x, f))) }
```

### 4.2 Fused `scaleAndAdd` and the FMA asymmetry

C++ writes `a + b*scale` as separate multiply-then-add, but **clang's default
`-ffp-contract=on` fuses it into a single FMA in the geometry pipeline** (Rust
never contracts automatically). To match C++'s rounding you must reach for
`f32::mul_add` *at exactly the sites clang would fuse* — and *not* elsewhere:

```rust
// crates/nuxie-runtime/src/draw.rs:13840 — geometry path: FUSE (matches contracted C++)
// Mirrors C++ Vec2D::scaleAndAdd after compiler contraction; rounded midpoint
// pruning can depend on the one-ulp split this preserves.
(vector.0.mul_add(scale, point.0), vector.1.mul_add(scale, point.1))
```

```rust
// crates/nuxie-scripting/src/vm.rs:347 — script VM: do NOT fuse
Ok(LuaVector::new(a.x() + b.x() * scale, ...))   // C++ Lua binding runs through
                                                 // the interpreter, uncontracted
```

**The asymmetry:** fuse in the geometry pipeline, do not fuse in the script VM.
The C++ Lua binding (`src/lua/math/lua_vec2d.cpp`) evaluates through the
interpreter with no geometry-path contraction, so the Rust Luau binding must use
plain `+`/`*`.

`Mat2D::invert` and `Mat2D::multiply` have an important contraction asymmetry.
Clang contracts the determinant and inverse translation cross-product
subtractions. Matrix multiplication contracts each two-term dot product, but
adds the left-hand translation separately. Rust therefore uses `mul_add` for
the inverse cross products and the first two terms of each multiply column,
then a separate `+ a[4]`/`+ a[5]` for translation. Local path composition uses
that same `multiply`; transform-shape heuristics can mask a missing contraction
in `invert`, but do not reflect any C++ branch. `transform_point` uses plain
`*`/`+` while `map_point` fuses, matching their distinct C++ call sites.

**Perf caveat (`docs/v2-log-archive.md` item 18):** closing the FMA gap globally
(bulk `mul_add`) changes float results and can flip exact files. Treat any new
`mul_add` as a per-site change requiring golden re-verification — never a bulk
pass.

### 4.3 Contour canonicalization vs HarfBuzz conventions

C++ `src/text/font_hb.cpp` records static `glyf` contours at the font's authored
start points. Skrifa's HarfBuzz-style path conversion *rotates* those start
points. To preserve C++ ordering, the port picks the conversion style by whether
the font is variable:

```rust
// crates/nuxie-runtime/src/text.rs:1098
let path_style = if style_font.axes().is_empty() {
    PathStyle::FreeType     // non-variable: preserve authored contour starts (C++ order)
} else {
    PathStyle::HarfBuzz     // variable: HarfBuzz path required
};
```

Residual outline float drift is covered by `tolerant` verification, but **glyph
contour *ordering* stays strict even under tolerant mode**
(`docs/v2-status.md:3535`) — ordering is ported behavior, not delegated-engine
drift. Zero-size glyph contours collapse to move/close pairs to match C++
`RawPath` (`text.rs:3851`).

---

## 5. Library Substitution Table

C++ delegates several subsystems to vendored C/C++ libraries; the Rust port uses
Rust-native equivalents. **The engine-swap decision rule:** *spec-defined
behavior may swap engines; implementation-defined behavior may not* — where a
faithful, upstream-conformance-verified port (HarfRust, luaur) counts as the
*same* engine. Each swappable subsystem sits behind a trait so the engine can be
changed without touching runtime code.

Only **3 of 6** planned substitutions are wired as dependencies today. Bidi,
image decoders, and audio remain paper decisions.

| Subsystem | Crate | Present? | Version | Trait seam |
|---|---|---|---|---|
| Layout | `taffy` | **Yes** | 0.12.1 | `RuntimeLayoutEngine` (`draw.rs:4024`) |
| Text shaping | `harfrust` + `skrifa`/`read-fonts` | **Yes** | harfrust 0.12, skrifa 0.44 | none (concrete in `text.rs`) |
| Bidi | `unicode-bidi` | No | — | — |
| Image decode | `image`/`png`/`zune-jpeg`/`image-webp` | No (headers only) | — | `Factory::decode_image` (render-api:365) |
| Audio | `cpal`/`rodio`/`kira` | No (schema enums only) | — | — |
| Scripting | `luaur-rt` (+ common/vm) | **Yes** (feat `luau`, default) | =0.1.8 | `ScriptingVm`/`ScriptInstance`/`ScriptHost` in `nuxie-runtime/src/scripting.rs` |

**Per-library gotchas actually discovered:**

- **Taffy (layout).** No `yoga` dep exists; Yoga-via-FFI is only the *untriggered
  fallback*. Files author against Yoga's React-Native-era quirks, so edge-case
  layouts may diverge — those verify in `tolerant` mode. Real convention
  mismatches captured in `draw.rs`: Yoga stores left/top parent-local and
  composes through parent world transforms, so the artboard origin is applied
  *exactly once* (`draw.rs:2082`); fill maps to stored units, not Yoga auto
  (`draw.rs:2677`); non-explicit position edges are Yoga-undefined → Taffy `auto`,
  not zero (`draw.rs:5327`); `TaffyTree::disable_rounding()` mirrors Yoga
  point-scale. **Fence:** do not pin Taffy against Yoga behavior-by-behavior —
  that is V1's mistake and a tripwire. Extend the hand-rolled flex fallback no
  further; the next unsupported layout feature triggers full Taffy integration.
- **Skrifa (font outlines).** Its HarfBuzz conversion rotates contour start
  points — see 4.3. Missing Arabic fallback shaper for malformed fonts is a
  documented backlog gap (act only if a corpus file hits it). Legacy-kern is
  disabled for advances (`text.rs:916`) and custom line metrics reproduce
  HarfBuzz ascent/descent (`text.rs:927`).
- **Image decoding.** Only *encoded-header dimension parsing* is implemented in
  Rust (`nuxie-render-api/src/lib.rs:1327`); real pixel decode is delegated to C++
  over FFI. Golden streams carry `decodeImage id=… width=… height=…` with **no
  payload hashes** — cross-runtime image comparison uses decoded dimensions +
  tolerant pixel sampling (PNG is lossless → exact; JPEG is not bit-identical
  across decoders → tolerant).
- **luaur (scripting).** Pinned exactly `=0.1.8` (validated against upstream Luau
  commit 8f33df9); it is a faithful port, so scripted files target `exact`, not
  tolerant, and the C++ golden runner (built *with* scripting) is a third oracle
  for any drift. **Bytecode validation requirement (landed):**
  `ScriptVm::load_bytecode` (`vm.rs:154`) runs `validate_luau_bytecode` — a full
  structural bounds-check — *before* the unsafe `luau_load`, because the pinned
  luaur deserializer does unbounded pointer reads; a hostile `.riv` bytecode
  payload must be preflighted (this closes audit item 23's UNSOUND finding).
  **Sandbox order (landed):** install all Rive globals first, *then*
  `sandbox(true)` — Luau's `GETIMPORT` resolves globals at load time, so globals
  must exist before any bytecode loads (`vm.rs:112`). The seam traits are owned by
  `nuxie-runtime` and implemented by the optional `nuxie-scripting` crate (inverted
  so the runtime never depends on the VM). `mlua`+`luau` remains the untriggered
  fallback behind the same seam. **TODO:** the "iOS no-JIT" property is implicit
  in the pure-Rust interpreter choice; there is no explicit in-code note for it.

---

## 6. Verification Method

### 6.1 Golden-stream discipline

Correctness is one number: **`exact-segments`** from `make golden-compare` — the
sum of verified (file × sample) segments across `exact` corpus entries. The C++
runtime is the oracle; a Rust run must emit an identical render-call stream
(`save`/`restore`/`transform`/`clipPath`/`drawPath`/`drawImage`/… with full
verbs, points, and paint state). A slice is "done" when the file it targets is
byte/epsilon-identical, not when a behavior is "pinned."

`corpus.toml` carries a per-entry `verification` mode, as strict as the file's
content allows:

- **`exact`** (default) — byte/epsilon-identical. All fully-ported subsystems,
  plus scripting (same VM).
- **`tolerant(ε)`** — positions/pixels within ε. Only files exercising a swapped
  engine: Taffy layout, HarfRust-shaped text numeric drift, or lossy image
  decode. A file with no layout/text/images may not hide behind `tolerant`.
- **`structural`** — same call sequence/counts, values within tolerance; last
  resort, requires a Decision-log entry.

Rive-owned behavior (text layout, wrapping, draw suppression, call order, glyph
contour ordering) is *ported*, so it stays `exact` — it is not delegated-engine
drift.

There is a separate scripted lane: the C++ golden runner is built *with*
scripting (`make scripted-golden-runner` / `make scripted-golden-compare`) so
scripted files get real reference streams, and luaur-shaped drift surfaces as an
attributable stream diff against real Luau.

### 6.2 The escalation ladder (divergence protocol)

When a golden diff fails, localize before you theorize — **budget half a day per
divergence:**

1. First divergent render call (the harness reports it with context).
2. Binary-search the timeline.
3. Disable subtrees/objects to isolate the component.
4. Read the two implementations side by side at that site.
5. Only if still stuck after ~half a day: write **one** targeted `cpp-probe` pin
   for that behavior — then fix it or file it in the backlog with findings and
   take the next task. Never let one divergence consume a session.

The long tail is *discovered*, not enumerated: an unconsidered behavior either
shows up as a diff on a real file (fix it) or never manifests (ignore it).
Differential fuzzing (corpus files at randomized times/inputs/mutations through
both runtimes) finds the weird interleavings automatically.

### 6.3 The tripwire / fence culture

The failure mode is *you* — V1 spent 94% of its map pinning data-binding edge
cases while nothing rendered. Stop and return to the milestone queue if any fire
(`.claude/commands/goal.md`): three commits on one C++ behavior family with no
corpus file changing status; writing a doc that enumerates C++ cases or a test
for behavior no corpus file exercises; a commit message that cannot name a
milestone tag; extending the frozen contract suite; or `exact-segments`
unmoved in ~10 commits. Perfectionism about one behavior is scope failure, not
rigor — shipped-and-diffed beats proven-in-isolation.

The full command reference (session loop, porting method, perf rules, thread
protocol) is `.claude/commands/goal.md` — this section summarizes it, it does not
replace it.

---

## Appendix: Quick Reference for a Phase R Translator

- Owning a new object → arena `Vec` + `local_id`, never `Rc`. (§1.1–1.2)
- Pointer field → `Option<usize>`/`Vec<usize>` index; `nullptr` → `Option`. (§1.2, §3.1)
- Closed variant set → `enum`+`match`; pluggable interface → `trait`. (§1.3)
- Per-frame vector → retained `Vec`, `clear()`+reuse, never realloc. (§1.4)
- New property → generated setter through the fence; grep siblings, match their
  epoch bumps exactly. (§1.5, §2.4, §2.5)
- Invalidation → mirror the named C++ gate; bump only what that gate touches. (§2.4)
- Float math → C++ grouping, no reassociation; `mul_add` only where clang fuses,
  and not in the script VM. (§4)
- Sort floats with `total_cmp`; never `partial_cmp().unwrap()`; never `unwrap`
  on file/ABI-reachable values (release aborts). (§3.1–3.2)
- Verify by golden stream, `exact` unless a swapped engine forces `tolerant`;
  localize divergences in half a day. (§6)

---

## §8 Architecture-Fidelity Rules (added for #B-6, 2026-07-21)

Behavioral gates cannot see design drift: the pre-rebuild data-binding
layer passed every golden and probe gate for months while being built on
polling and copies instead of C++'s retained identity and dependents. These
rules exist so a reviewer can cite the violated rule behind every finding.

- **AF-1 Retained identity.** Where C++ holds a pointer/`rcp` to a shared
  mutable object, Rust retains a shared handle (`Rc<RefCell<..>>` or arena
  id) to ONE object. Copying the value into a record and re-syncing later
  is a violation, regardless of test results.
- **AF-1 retained-cell corollary.** A C++ `ViewModelInstanceValue` is itself
  the retained, reference-counted property cell and dependency source; a
  `DataBind` keeps that same cell as `m_Source` rather than copying its payload
  (`include/rive/viewmodel/viewmodel_instance_value.hpp:34-71,88-97`,
  `src/viewmodel/viewmodel_instance_value.cpp:93-105`,
  `src/data_bind/data_bind.cpp:210-240`). The faithful Rust idiom is
  `RuntimeViewModelCell` shared identity plus
  `RuntimeOwnedViewModelHandle` graph identity
  (`crates/nuxie-runtime/src/view_model_cell.rs:400-450,482-530`,
  `crates/nuxie-runtime/src/view_model.rs:1781-1817,10390-10441`); source
  resolution returns the cell alongside any projected value
  (`crates/nuxie-runtime/src/artboard_data_bind.rs:1110-1150`). A copied scalar
  may be an application payload, but it must never replace the retained source
  cell used by binds, listeners, converters, or nested ViewModel links.
- **AF-2 Push, never reconstruct.** Where C++ registers a dependent or
  property observer, Rust registers a dependent (weak dirt-sink). Polling,
  generation counters, epoch comparisons, observed-value diffing, and
  rescan loops that exist to DISCOVER a change C++ would have been TOLD
  about are violations.
- **AF-2 weak-dirt-sink corollary.** C++ stores dependent pointers, registers
  them explicitly, cascades dirt synchronously, and unregisters them when the
  source is cleared (`include/rive/dependency_helper.hpp:19-60`,
  `src/viewmodel/viewmodel_instance_value.cpp:93-105`,
  `src/data_bind/data_bind.cpp:210-240`). Rust maps that lifetime asymmetry to a
  consumer-owned `RuntimeCellDirtSink`; the cell retains only its downgraded
  `RuntimeCellDependent`, and dead weak entries are discarded during dirt
  delivery (`crates/nuxie-runtime/src/view_model_cell.rs:108-220,482-530`). The
  source must not strongly own the bind/listener/converter, and a mutation must
  push dirt through this sink rather than wait for a generation poll or rescan.
- **AF-3 Poll only where C++ polls.** The inverse also binds: where C++
  itself indexes by time/id with no observer (keyed animation binary
  search, `resolve(objectId())`), a Rust loop/lookup is correct.
  Introducing an observer C++ doesn't have is also drift.
- **AF-4 One dirt model.** Direction/ordering state follows C++'s dirt
  bits + origin flags. Proliferating per-phase boolean latches to encode
  what C++ encodes in one bitset is a violation of representation even
  when the phase ORDER is correct.
- **AF-5 Import-time devirtualization is legitimate.** A field computed
  once at build (type discriminant, subscription flag, precomputed target
  kind) that C++ derives per frame via virtual dispatch or registry lookup
  is an accepted idiom — PROVIDED it is never mutated during the
  advance/update/bind cycle. The mutation-timing gate is the test.
- **AF-6 Deep copy is explicit.** `Clone` preserves C++ copy semantics
  (deep, with `copyViewModelInstance`-style dedupe preserving internal
  sharing topology). Sharing is only ever introduced through an explicit
  handle type.
- **AF-7 Own-by-value for unique ownership.** C++ `unique_ptr` vectors map
  to Rust `Vec<T>` by value. Reaching for `Rc` where C++ has unique
  ownership is drift in the other direction.
- **AF-8 No invented lifecycles.** Bind/unbind/teardown happen at the C++
  call sites. Adding refresh/rebind passes at points where C++ has none
  (facade-wide dirty bits, rebind-before-every-advance) is a violation
  even if idempotent.
- **AF-8 nested-container corollary.** A C++ object that is itself a
  `DataBindContainer` owns the DataBinds targeting it. In particular, a bind
  whose target is a `DataConverter` is subordinate to that converter; it is
  not an occurrence in the enclosing artboard's authored DataBind list
  (`data_bind.cpp:94-100`, `data_bind_container.cpp:86-112`). Rust may
  devirtualize the target kind, but must keep subordinate binds on their own
  source-path queue rather than assigning them an outer occurrence index.
- **AF-9 DataContext lookup is local-then-parent and occupant-typed.** C++
  tests each locally retained instance in order, accepts an instance only when
  its actual `viewModelId` matches the authored path, and recurses to the parent
  only after every local attempt fails
  (`src/data_bind/data_context.cpp:265-297,397-418`; instance lookup follows the
  same rule at `src/data_bind/data_context.cpp:335-363,464-483`). Therefore a
  same-model local instance that lacks an intermediate or final property does
  **not** shadow a complete parent instance: resolution continues. Global slot
  keys control placement/replacement only; the occupant's actual ViewModel
  identity controls lookup (`include/rive/data_bind/data_context.hpp:37-61,
  105-112`). The faithful Rust path is `RuntimeOwnedDataContext::resolve` plus
  `resolved_property_path`/`resolved_property_path_with_manifest`
  (`crates/nuxie-runtime/src/artboard_data_bind.rs:807-964,1035-1079`), with the
  occupant identity check in
  `RuntimeOwnedViewModelContextPathStorage::from_context_source_path`
  (`crates/nuxie-runtime/src/artboard_data_bind.rs:2561-2572`). Candidate-vector
  precedence, slot-key path rewriting, or stopping at a partial same-model
  match is architecture drift even when a sampled value happens to agree.

---

## §9 Renderer-Feed Translation Rules (RD-1)

These rules bind the Phase RD move from scene-level prepared replay to the
pinned C++ runtime's live traversal. They apply to the lane map in
`docs/rd1-renderer-feed-map.md`; renderer pixels, ordinary goldens, and scripted
goldens referee every merge.

- **RF-1 Traverse retained object topology live.** C++ constructs and relinks
  `Artboard::m_FirstDrawable`/`Drawable::{prev,next}` under draw-order dirt, then
  walks that linked object graph on every `drawInternal`
  (`src/artboard.cpp:429-840,1159-1169,1652-1698`,
  `include/rive/drawable.hpp:27-31,75-78`). Rust stores equivalent object ids or
  handles and follows them per frame. A prepared frame, sorted replay array, or
  retained scene command stream is not an equivalent implementation.
- **RF-2 Retain resources at the C++ owner.** An embedded or uniquely owned C++
  resource stays with its corresponding Rust object: `Shape` owns its
  `PathComposer`, paths own their live geometry, paints own render paint, and
  assets own render images. Do not move those resources into an artboard- or
  facade-level cache. Conversely, non-owning C++ pointers become ids/handles,
  not cloned resources (`include/rive/shapes/shape.hpp:18-26`).
- **RF-3 Preserve live virtual-dispatch boundaries.** Closed C++ drawable
  subclasses may become enum dispatch, but `willDraw`, `draw`, `isHidden`,
  `isClipStart`, `isClipEnd`, and `emptyClipCount` must be evaluated on the live
  object at the same traversal site and in the same order. Import-time type
  devirtualization may choose the match arm; it may not snapshot a mutable
  return value (`include/rive/drawable.hpp:42-68`).
- **RF-4 Port lazy clipping as one ordered state machine.** Preserve the exact
  `emptyClips` update, `willDraw` guard, clip-start push, adjacent clip-end pop,
  pending-clip flush, and drawable draw order from
  `Artboard::drawInternal` (`src/artboard.cpp:1652-1698`). Do not precompute clip
  commands, split the logic across phases, or emit save/clip/restore for a clip
  pair that C++ elides because no drawable occurs between it.
- **RF-5 Read mutable render state at draw time.** Hidden/collapsed state,
  opacity, visibility, fill/path selection, world transform, clip counts, and
  other mutable properties are live reads. Epoch-correct invalidation cannot
  make an import-time or prepared-frame snapshot faithful
  (`src/drawable.cpp:41`, `src/shapes/shape.cpp:123-147,309`).
- **RF-6 Keep save/restore ownership and predicates local.** Artboard save is
  exactly `clip() || m_FrameOrigin`, with one balanced restore. Shape paint save
  intent is exactly `m_needsSaveOperation || m_ShapePaints.size() > 1` and is
  passed to each visible paint draw (`src/artboard.cpp:1620-1640,1699-1702`,
  `src/shapes/shape.cpp:123-147`). Do not infer a broader scene-wide save policy.
- **RF-7 Preserve transform side, multiplication order, and path space.** A
  local `Shape` path composes `transform * worldTransform`; a world path uses
  `transform` directly. Raw-path null transforms and hit-test transforms follow
  their separate C++ branches (`src/shapes/shape.cpp:90-121,171-218`). Reusing
  one normalized transform formula across those sites is drift.
- **RF-8 Translate every skip/continue guard in source order.** Examples include
  hidden/collapsed drawables, zero render opacity, invisible paint, null
  `pickPath`, translucent hit-test paint, and collapsed child paths. Combining
  predicates is allowed only when it preserves short-circuit evaluation and
  every intervening side effect (`src/shapes/shape.cpp:123-147,161-218,309-317`).
- **RF-9 Separate retained object state from traversal scratch.** C++'s
  `m_PathComposer`, `m_Paths`, and shape-paint resources cross frames; the local
  `pendingClipOperations` vector and `emptyClips` counter in `drawInternal` do
  not. Rust may reuse allocation privately, but must not turn per-call scratch
  into a scene representation or give it independent invalidation/ownership.
- **RF-10 Preserve construction-only work as construction-only.** Dependency
  wiring, deformer discovery, path-list membership, and the current non-animated
  blend-mode propagation happen at the corresponding C++ add/build hooks
  (`src/shapes/shape.cpp:20-25,264-307`). Do not refresh them every frame; if an
  upstream property becomes animated, port its new C++ dirt/update site rather
  than inventing a scan.
- **RF-11 Preserve frame-entry side effects.** `Artboard::draw` increments the
  frame id and draws canvases before `drawInternal`; `drawInternal` clears
  `m_didChange` before the opacity early return (`src/artboard.cpp:1606-1618`).
  A facade or renderer adapter must not reorder or omit those effects when it
  switches from prepared replay to live traversal.
- **RF-12 Cut over only when the reachable family is complete.** No scene-cache
  deletion occurs until every drawable kind reachable from the live artboard
  walk has a faithful dispatch arm and the 1,468-row pixel corpus is exact.
  RD-C1/RD-C2 must first remove the temporary command-materialization seam and
  report the second measured checkpoint; RD-C7 owns scene-cache deletion.

### RD-1b2 dual-translation stress-test decisions

On 2026-07-22, two independent disposable translations of
`src/drawable.cpp`, `src/shapes/shape.cpp`, and
`src/artboard.cpp:1606-1698` were compared: one followed this rulebook
strictly; one approached the same slice as a senior Rust engineer without the
rulebook. Neither translation was retained as implementation. Every
disagreement is resolved below and is binding on RD-C1/RD-C2.

- **RF-13 A cached sorted vector is not the live-list port.** The senior
  translation treated `Arc<Vec<SortedDrawableNode>>` keyed by
  `draw_order_epoch` as equivalent to C++'s list. The strict translation kept
  `firstDrawable` plus `prev`/`next` ids on retained drawable objects, including
  synthetic clip/layout proxies. RD uses the strict form: imported graph order
  may seed construction, but the runtime representation being traversed is the
  relinked object list. Its direction is literal: start at `m_FirstDrawable`
  and advance through `prev` (`src/artboard.cpp:1652-1654`); do not infer a
  different direction from the field names.
- **RF-14 Backend-specific storage does not change logical ownership.** The
  senior translation left `RenderPath`/`RenderPaint` in centralized
  `RuntimeRenderPathCache`/`RuntimeRenderPaintCache`; the strict translation
  placed them with the owning PathComposer/ShapePaintPath/ShapePaint. C++
  ownership wins. Rust may use a backend-specific dense sidecar when trait
  objects cannot live in cloneable CPU state, but each slot must be one-to-one
  with, created for, invalidated by, cloned/recreated with, and dropped with its
  owning runtime object. A key-addressed artboard/facade cache with an
  independent lifetime or scene epoch is still a scene cache and is deleted.
- **RF-15 `m_FrameOrigin` is live artboard state, not a draw option.** The
  senior translation proposed deriving it from the current
  `apply_origin_transform` argument; the strict translation retained the C++
  field. Port the field and its setter. It defaults true, is copied with an
  artboard instance, and nested/scripted/component-list callers set it false
  (`include/rive/artboard.hpp:105,554,611-617`,
  `src/nested_artboard.cpp:120`, `src/scripted/scripted_object.cpp:52`,
  `src/artboard_component_list.cpp:1481`). A facade argument may set that live
  state at the corresponding C++ call site; it may not become a second source
  of truth.
- **RF-16 Validate structural cycles once; do not invent per-frame guards.**
  The senior translation added visited sets to parent and proxy walks; the
  strict translation preserved the C++ loops. If Rust can import a parent or
  proxy cycle that the C++ construction contract rejects, close that importer
  or build-time validation gap. Do not allocate a visited set or silently
  return a different result on every draw/hit test unless the pinned C++ site
  has that guard.
- **RF-17 Port object-local invalidation representation exactly.** The senior
  translation replaced `DrawableFlag::WorldBoundsClean` and
  `Shape::m_WorldLength == -1` with epoch-keyed cache entries. The strict
  translation kept the flag and sentinel. Keep the C++ representation and
  invalidation call sites (`include/rive/shapes/shape.hpp:21-24,52-72`,
  `src/shapes/shape.cpp:55-88`). Do not introduce an epoch merely because the
  old scene replay already has one.
- **RF-18 Do not clone topology vectors to satisfy the borrow checker.** One
  disposable translation cloned path/paint id vectors before live loops; the
  other used borrowed topology. C++ iterates retained vectors in place. Rust
  must split graph/object/resource borrows, use stable ids, or use index-based
  loops so drawing does not copy `path_locals`, `paint_locals`, clipping ids, or
  drawable order each frame.
- **RF-19 Preserve renderer stack balance across Rust errors.** C++ draw
  methods in this slice are infallible; a direct translation therefore has no
  exit between `save` and `restore`. If a Rust engine substitution or lazy
  factory call remains fallible, complete it before emitting renderer side
  effects where possible. Otherwise use a scope guard/finally-shaped helper
  that restores on every `Result` path. A `?` that can leave a saved or clipped
  renderer state active is never a faithful port.
- **RF-20 Preserve C++ integer narrowing and later mutation.**
  `Drawable::onAddedDirty` casts the stored `uint32_t` through
  `BlendMode : unsigned char` before validation
  (`include/rive/generated/drawable_base.hpp:35-52`,
  `include/rive/shapes/paint/blend_mode.hpp:5`, `src/drawable.cpp:17-39`). Rust
  must validate the narrowed `u8`. It may retain a typed value only if every
  later generated/property write updates that value; otherwise narrow the live
  property at each C++ read site. Import-time normalization that snapshots a
  mutable property, or later matching the original `u32`, disagrees with C++.
  Import validation may reject early too, but it does not replace the
  `onAddedDirty` hook after `Super::onAddedDirty`; that order is observable to
  programmatically constructed objects.
- **RF-21 Trailing pending clip starts are discarded, not asserted away.** The
  senior translation allowed `debug_assert!(pending.is_empty())` if validation
  promised balance; the strict translation followed C++ and simply dropped
  the local vector at return. The pinned traversal states no balance invariant,
  and deliberately elides clip ranges with no later drawable. Do not add an
  assertion or import rejection solely because pending starts remain after the
  walk (`src/artboard.cpp:1660-1698`).
- **RF-22 Private helper devirtualization is allowed only with exact state.**
  The strict translation mirrored `ComputeBoundsCommandPath`; the senior
  translation proposed operating directly on a temporary `RawPath`. Either
  representation is acceptable because the helper is closed and local, but it
  must preserve the expansion sentinel, per-path rewind, collapsed-only skip,
  command order, and `pathTransform * suppliedTransform` multiplication
  (`src/shapes/shape.cpp:315-392`). It must not reuse a generic filtered-path
  iterator: length, emptiness, HiFi hit testing, paint hit testing, and bounds
  intentionally apply different hidden/collapsed filters.
- **RF-23 Mutable flags never belong in immutable topology.** The senior
  translation's proposed topology record included `authored_hidden`; the strict
  translation read `drawableFlags()` live. Hidden, opaque, blend mode, save
  need, and any other property with a generated setter live in instance/object
  state. Immutable topology may keep only identity and type discrimination.
  Import-time devirtualization does not authorize copying a mutable payload.
- **RF-24 Ordered memberships live on their C++ owner.** The senior translation
  left shape path/paint membership in immutable `ArtboardGraph`; the strict
  translation put the ordered non-owning ids on the retained `RuntimeShape`.
  Use the strict form. Graph nodes may seed `Shape::m_Paths`,
  `ShapePaintContainer` membership, `Drawable::m_ClippingShapes`, and proxy
  objects during construction, but the live object's vectors are the runtime
  source of truth. Append-only construction data may be frozen after build; it
  is still owned and traversed through that object, not rediscovered by a graph
  scan (`src/shapes/shape.cpp:20-25`, `src/drawable.cpp:36-39`).
- **RF-25 Preserve nullable/self/other pointer flow.** The strict translation
  made a proxy target required; the senior translation exposed
  `Option<local_id>`. Construction may make a specific proxy target non-null,
  but the generic `Drawable::hittableComponent` contract still has three
  outcomes: self and null both fall through to base component hit testing;
  another drawable delegates with both booleans unchanged
  (`src/drawable.cpp:58-77`). Do not collapse null into a panic, and do not make
  it skip the base call.
- **RF-26 Scratch allocation reuse has no semantic owner.** One translation
  allocated `pendingClipOperations` per call; the other cleared a vector stored
  in the old render-path cache. Reusing the allocation is allowed, but move it
  to a draw-context/artboard scratch slot that is cleared at entry and carries
  no cache key, epoch, or cross-frame content. The scene cache does not survive
  merely because it owns a useful `Vec` allocation.
