# Luau on clients, lane H2: charges live with the native values (2026-11-14)

Branch `claude/luaur-lane-h2-ownership-f75fe7`, based on `levi/luaur-client-stage1f`
(`b495c08c29`).

Owned files:
- in `crates/nuxie-runtime`: `src/mechanical_port/source/viewmodel/`, `src/host_viewmodel/`
  (and its module file), `src/scripting/native_view_model.rs`, and `mechanical_port/source/text/`;
- in `crates/nuxie-scripting/src/vm/`: `view_model.rs`, `lua_font.rs`, `lua_blob.rs`,
  `lua_artboards.rs`, `lua_text.rs`, `lua_paint.rs`, `renderer.rs`, `host_memory_budget.rs`,
  plus the one allowed visibility change in `lua_path.rs`.

## Commits

| Commit | Item | Concern |
| --- | --- | --- |
| `195c41199e` | 1 | A host memory charge is `Send + Sync` |
| `44619da9f9` | 1 | Native values keep a host owner token |
| `91c204b289` | 1 | Measure a named view-model copy before it is made |
| `e5e25dd6d9` | 1 | Host memory charges live with the native values |
| `53240da312` | 4 | Canvas descriptors pass on the call's loan |
| `851a95584b` | 4 | `addToPath` charges and reserves through the path |
| `75b5f7eaef` | 4 | A derived font is charged for its option tables |
| `7e3d57ee79` | 2 | A Text keeps the gradient of each paint it styles with |
| `a5f8bc8272` | 3 | Shaped runs reserve breaks for their own glyphs |
| `99af51bfdb` | 3 | A layout is built within an allowance |
| `3e7dcb36a7` | review | A string property is charged what it keeps |
| `57168b1f88` | review | A derived font keeps only its own charge |
| `60e6c016e4` | review | A copy is measured the way it is made |
| `8004f47681` | H4 | A view model cannot hold what holds it |
| `a9f22f8f81` | review | A rolled-back string keeps its owner |
| `e548904950` | review | A new instance is measured from the schema it is made from |
| `deffd8e74b` | review | A line is admitted for what its copies allocate |
| `2523966787` | review | A copied template string keeps its share of the charge |
| `b44a39dfd3` | review | addToPath appends the artboard's commands as commands |
| `54fd4825b8` | review | A reused text style reserves no paint slot |

The commits were split from one working tree by staging intermediate file
contents. Only the tip of each group was built and tested (item 1's tip, then
the tip after item 4). The intermediate commits were not built one by one.

## Item 1: the ownership design as built

### The token

`nuxie_runtime::HostOwnerToken` is an opaque, cloneable `Arc<dyn Any + Send + Sync>`.
A host hands one to the runtime, and the runtime keeps it in the native value. The
token drops when the value drops. There is no registry, no map, and no sweep.

A charge is now `Send + Sync` (atomic counters), so it can live in a font. A font is
an `Arc<dyn Font>` that the runtime may hand to another thread.

| Native value | Where the token lives | When it drops |
| --- | --- | --- |
| View-model instance (`Data.X.new`, `model:new`, a named copy) | `ViewModelInstance::host_owner` | With the instance, i.e. with the File's arena. See below. |
| List item (`push` / `insert`) | `ViewModelInstanceListItem::host_owner` | With the item, i.e. with the arena |
| A string a script assigned | `ViewModelInstanceString::host_owner` | On the next change of value, by anyone (`property_value_changed`), or with the property |
| Blob | `RuntimeBlobAsset::host_owner` | With the last `Arc<RuntimeBlobAsset>`, wherever data binding spread it |
| Decoded font | `HbFont::host_owner` | With the last font that shares the bytes. Variants and host-outline copies keep it. |
| Derived font (`withOptions`) | `HbFont::variant_owner`, beside the bytes' owner | With that variant only. Its own variants do not keep it. |

### Consequences for each value

Text needs no charge plumbing for fonts. The shaped runs keep the `FontRef`, so
they keep the face's charge. `TextLayout`'s font list (and its charge) is gone.
`glyph.font` hands out the same `FontRef`.

Blob and font property getters and setters no longer carry a charge on the wrapper.
The payload carries it.

### The arena finding

`CoreArena` frees a slot only through `remove`. Instances made by
`create_view_model_instance*` and `copy_view_model_instance`, and list items made by
`insert_sibling`, live until the File drops.

The old refund on wrapper GC therefore under-counted even without any aliasing. A
loop of `Data.X.new()` that discarded each result grew the arena without bound and
stayed charged at zero. Now each instance or item is charged for the arena's life.
The charge is exactly as long as the native value lives, which is what the brief
asked for. The consequence is that it never comes back while the File lives (see
"What a script observes").

### Named copies

A named copy is measured before it is made. An authored instance names its nested
instances and list items by index, and the copy resolves those indices.
`ScriptViewModel::named_copy_shape` resolves them the same way.

Measuring the uncopied template, as an earlier draft did, under-measured. Its
nested references are unresolved until the copy. The `data_new_is_charged_for_the_named_template`
test caught this. After codex review, the walk also memoizes and limits depth the
way the copy does (see "Review").

### Tests that fail on the old code

- `data_new_is_charged_for_as_long_as_the_instance_lives`: a discard loop is refused.
- `list_items_are_charged_for_as_long_as_they_live`: a push/pop loop is refused.
- `a_string_assigned_to_a_property_is_charged_while_it_holds_it`
- `a_blob_is_charged_while_the_runtime_keeps_it`
- `a_font_assigned_to_a_property_stays_charged`
- `data_new_is_charged_for_the_named_template`: refused before the copy, and nothing charged.

## Item 2: Text and the gradient

Confirmed: `Text:append` keeps the paint's render paint, whose shader is the
gradient's. The gradient's charge lived on the paint wrapper.

A paint's gradient is now a shared slot (`GradientSlot = Rc<RefCell<Option<ScriptedGradient>>>`).
Each new style keeps the slot of its paint. Because the slot is shared, a later
`paint.gradient = …` on the same paint is followed too. The slots vector is charged
at capacity. There is at most one slot per style, and at most 65,536 styles. A
`clear` lets the slots go.

Test: `a_text_keeps_the_gradient_of_a_paint_it_styles_with`. It uses 1,000 stops,
then drops the paint and the gradient and runs GC. The old code refunded the 8 KB.

## Item 3: the Text layout bound

### Choice

Fallible admission inside the layout (`RawText::lay_out_within(allowance)`), not an
input bound. Measured below, the layout's bytes per content byte range from about
67× (one line of ASCII) to about 3,060× (one word per line). The quadratic case
comes from each `OrderedLine` copying every whole `GlyphRun` it touches. No input
multiple would both bound the layout and admit legitimate text.

### What is admitted

Checked before they grow:
- each line's run copies (`OrderedLine::heap_bytes_bound`);
- each style path's growth, render copy included, reserved exactly with `try_reserve`;
- each layout vector push, at the standard library's real growth, including the
  minimum capacity of 8 for one-byte elements.

Checked right after they are built, and freed at once on a refusal:
- the shaped paragraphs;
- the line breaks.

A refused layout is let go (`release_layout`). The script `Text` lays out within
`charge.most()`: what it is charged plus what the budget has free. The error is the
budget's usual one.

### Stated bound

A layout admitted with allowance A holds at most A + the glyph lookup bytes. The
glyph lookup is built after admission and is linear in the content: 4 B per
character plus 4.

On top of that, transient memory is not counted:
- the shaped paragraphs before their check, and the line breaks before theirs;
- one glyph outline at a time.

Shaping is limited by harfrust to max(256 × length, 65,536) glyphs per buffer.

### Measurements

From `layout_bound_tests::a_layout_is_admitted_within_its_allowance`, which runs a
bisection for the least admitted allowance against `RawText::heap_bytes` on each
input:

| Input (Roboto, Arabic: IBM Plex Sans Arabic) | Content B | Laid out B | Least allowance B | / held | Glyph lookup B |
| --- | ---: | ---: | ---: | ---: | ---: |
| 10,000 ASCII, one line | 66,308 | 4,468,304 | 4,428,300 | 0.991 | 40,004 |
| 300 words, one per line | 4,868 | 14,918,380 | 14,916,868 | 1.000 | 3,604 |
| 2,000 one-character runs | 90,724 | 3,966,112 | 3,958,108 | 0.998 | 8,004 |
| 2,000 characters, ellipsis, fixed 40×400 | 8,964 | 1,205,552 | 1,203,388 | 0.998 | 8,004 |
| 5,500 Arabic characters, wrapped at 120 | 33,540 | 46,390,060 | 46,375,048 | 1.000 | 22,004 |

In every case the least allowance equals held − glyph lookup. The admission's count
is exact. Every refusal in the bisection was checked to let the layout go
(heap ≤ content).

### The per-run reserve

`shape_text` reserved a quarter of the whole text for every run's breaks and
joiners. N runs held N × text, as upstream does. Each run now reserves a quarter of
its own glyphs. `many_runs_hold_what_their_glyphs_need` (runtime) and
`many_short_runs_lay_out_in_what_their_glyphs_need` (script) fail on the old code:
3,000 runs held about 18 MB more and were refused by a 12 MiB budget.

### Tests that fail on the old code

- `a_layout_past_the_budget_is_refused_before_it_is_built`: 10,000 words, one per
  line. Under the old code this would build about 4 GB before charging it. I did
  not run it against the old code, which would exhaust the machine. The claim
  follows from the 300-word measurement scaled quadratically.
- The two per-run tests above.

## Item 4: small joins

- **`addToPath`.** Confirmed broken: it checked with a charge it dropped at once,
  then rebuilt the whole path through `with_render_raw_path_mut`, losing the
  reservation. It now measures the added commands, then
  `ScriptedPath::reserve` (now `pub(super)`), then `RawPath::add_path`.
  `check_path_growth` and `scripted_path_bytes` are removed.
  Test: `a_path_added_to_keeps_the_charge_for_what_it_holds`. Many paths, each
  added to once and kept, are refused. The old code kept them all uncharged.
- **Derived fonts.** `HbFont::variant_bytes_bound(coords, features)` covers the
  font object, both hash tables (hashbrown's bucket rule at twice the entries) and
  the shaping features (at least 4). It is checked against the capacities in
  `a_variants_own_bytes_stay_within_the_bound`, which runs 6 rounds of up to 1,024
  entries. The charge is kept by the variant alone.
  Test: `a_derived_font_is_charged_for_its_option_tables`. 1,000 variants with
  1,000 features each are refused under 1 MiB, and a collected variant gives its
  charge back.
- **`context:canvas{…}` / `context:gpuCanvas{…}`.** Confirmed: plain `get`. They
  now use `add_method_lent` and `get_with(reborrow_loan(&mut loan), …)`.
  Test: `a_canvas_descriptor_index_reaches_the_draw_renderer`. A descriptor
  `__index` draws with the draw's renderer.

## Review

`codex review --base levi/luaur-client-stage1f`:

- **Round 1** (on `99af51bfdb`). Three findings, all fixed:
  - P1: each derived font kept its source's whole owner, so a chain of
    `f = f:withOptions(...)` kept every charge, and dropping the last dropped
    the chain recursively. Fixed in `57168b1f88`: a variant keeps the face's
    owner and its own, never its source's variant owner.
    Test: `a_chain_of_variants_keeps_one_charge_each`.
  - P2: a string cut at a NUL was charged its uncut length. Fixed in
    `3e7dcb36a7`: the charge is set to the stored length.
  - P2: the named-copy measure did not memoize or limit depth the way
    `File::copy_view_model_instance` does, so a cyclic or shared authored
    graph could be measured differently from what was copied. Fixed in
    `60e6c016e4`. Test: `copy_shape_tests::a_measured_copy_is_what_the_copy_makes`,
    which uses three upstream fixtures and a self-referential corruption.
- **Round 2** (on `8004f47681`). Two findings, both fixed:
  - P1: a transaction snapshot dropped a script string's owner on rollback.
    Fixed in `a9f22f8f81`. Test: `owner_tests::a_rolled_back_string_keeps_its_owner`.
  - P1: `model:new()` measured the live graph but builds from the schema.
    Fixed in `e548904950`: `schema_shape_of` walks the schema the way
    `File::create_view_model_instance` builds it.
    Test: `model_new_is_charged_for_what_the_schema_makes`.
  - The round-2 tests were not run against the old code. That they fail
    there is reasoned: the old snapshot has no owner to restore, and the old
    measure reads the smaller live graph.
- **Round 3** (on `e548904950`). Two P2 findings, both fixed:
  - The line bound counted the source runs' capacities, and a runs vector
    twice their number. A relayout as large as the layout it replaced could
    therefore be refused at that size, which is a behaviour change at the
    budget's edge. Fixed in `deffd8e74b`: the bound now counts what a clone
    holds (lengths) and the capacity the vector reaches when pushed.
    Test: `a_relayout_fits_where_the_layout_it_replaces_did`. It uses three
    styled spaces in auto-width; the width changes and the relayout is
    allowed exactly what it held. The bisection test still passes, and its
    least allowances cannot fall below held − lookup.
  - Strings copied from a named template were charged in the instance's one
    charge, so replacing one never refunded its share. Fixed in `2523966787`:
    `HostMemoryCharge::split_off` moves each copied string's share onto its
    property's owner, and `ScriptViewModel::own_strings` walks the copy.
    Test: `a_copied_template_string_is_refunded_when_replaced`. It makes every
    authored string 10,000 bytes, copies each named template whose graph holds
    one, clears the copied strings, and checks that each share is refunded.
- **Round 4** (on `2523966787`, the last allowed). Two P2 findings, both
  behaviour changes my commits had introduced, and both fixed:
  - `addToPath` copied the commands into the buffers with
    `RawPath::add_path`, which leaves `contour_is_open` and
    `last_move_index` as they were. After an open contour, a `lineTo` that
    followed the artboard's closed contour drew from the close with no move.
    The old round trip through the render path replayed the commands and
    did not do this. Fixed in `b44a39dfd3`: the commands are appended as
    commands (`append_commands`), with one more verb and point reserved for an
    injected move. Test: `a_command_after_add_to_path_starts_a_new_contour`
    (`fix_rectangle.riv`).
  - `Text:append` with a gradient paint made room for a paint slot even when
    the style was reused. A full slot vector doubled and was charged for a
    slot that was never filled. Fixed in `54fd4825b8`: room is made only when
    the style is new (`has_style`, scanning the existing styles as
    `RawText::append` does). Test: `an_append_reusing_a_style_reserves_no_paint_slot`.
  - The round-3 and round-4 tests were not run against the old code. That
    they fail there is reasoned from the code each fix replaced.
- No fifth round was run. The round-4 fixes have been reviewed only by me.

## H4's hostile-corpus findings (assigned to H2 mid-lane)

From `docs/evidence/luaur-client-path-lane-h4-2026-11-14.md` on
`claude/luaur-lane-h4-abort-gate-5744ba` (`688ce7089b`):

1. **`viewmodel/nested_self_assignment` and `nested_subslot_cycle` (stack
   overflow). Fixed in `8004f47681`.** `ViewModelInstance::rebind_dependents_occurrence`
   recurses up through parents, and `rebind_properties_occurrence` down through
   slots, with no memo. A cycle (`main.chi = main`; `main.chi = c; c.subch = main`)
   never ends. A chain built top-down overflows at about 6,500 levels, which a
   script reaches within budget. The scripted nested-slot setter and the list
   inserts now refuse an instance that would close a cycle or nest the graph
   past 256 (the depth a copy keeps), with the script error
   `a view model cannot hold itself or an instance that holds it, or nest more than 256 deep`.
   The check walks the ancestors of the holder and what the value holds, once
   each (`longest_walk`, a memoized iterative walk, transient). A list item is
   refused before it is made, because the arena would keep a refused one.
   Test: `a_view_model_cannot_hold_what_holds_it`. It covers the self-hold, a
   two-instance cycle, and a 10,000-deep chain. With the guard disabled except
   for the self-hold, it overflows the stack (checked).
   - **Left open: the wrong type is still accepted.** `child.subch = child` puts a
     `child` in a `SubChild` slot. Upstream's `ScriptedPropertyViewModel::setValue`
     → `replaceViewModelByProperty` does not check the type either, so I left it
     (refusing would change behaviour). `replace_view_model_by_name` does check
     the type; the scripted path does not use it.
   - **Left open: diamonds.** The rebind walks have no memo. A graph where
     instances share holders at many levels (each held by two parents) makes the
     walk exponential in depth: a hang, not a crash. Depth is now at most 256,
     but 2^256 paths is still unbounded work. The fix is a visited set in those
     two walks (`viewmodel_instance.rs`). That changes how often dependents
     relink, so it needs the goldens. I did not build it.
2. **`viewmodel/string_property_huge_many_instances`. Fixed by item 1
   (`e5e25dd6d9`) and `3e7dcb36a7`.** Each property charges the string it keeps,
   so six instances holding 16 MiB each are refused at the fifth under 64 MiB.
   Copies made by data-binding fan-out are still uncharged (gap 1 below).
3. **`alloc/font_with_options_hoard`. Fixed by `75b5f7eaef` and `57168b1f88`.**
4. **`audio/shared_budget_vs_host_budget` and `audio/stream_reader_hostile`.
   Not changed, as asked.** See "What a script can hold in total".

## What a script can hold in total

This is read from the code, not measured. It applies to one imported File's VM
with the default `ScriptExecutionLimits` (`nuxie/src/script_import.rs` →
`ScriptVm::new_with_execution_limits`).

| Pool | Limit | Where |
| --- | ---: | --- |
| Main VM Lua heap | 64 MiB | `DEFAULT_SCRIPT_VM_MEMORY_BYTES`, `lua.set_memory_limit` |
| Host memory budget (everything in this lane) | 64 MiB | `DEFAULT_SCRIPT_HOST_MEMORY_BYTES` = the VM limit |
| Audio shared objects and stream readers | 512 MiB | `luau_audio_shared.rs` `DEFAULT_SHARED_BUDGET`, one per main VM (`ProcessorSharedBudget`) |
| Audio processor VMs' Lua heaps | 64 × 64 MiB = 4 GiB | `MAX_LIVE_PROCESSORS` = 64, each with `ProcessorMemoryLimit` = the main VM limit |
| **Total, charged** | **4,736 MiB (about 4.6 GiB)** | |

`ScriptVm::new()` (no explicit limits) uses a 16 MiB Lua heap, also for each
processor. That gives 16 + 64 + 512 + 64 × 16 = 1,616 MiB.

On top of the charged total:
- the uncharged paths in the next section;
- the arena: instances and items charged for the File's life also count above,
  inside the 64 MiB.

Each File has its own VM, so the totals add up per File.

## What a script observes differently

- **Instances and list items.** `Data.X.new` / `model:new` instances and list items
  stay charged after the script drops them, for as long as the File lives (the
  arena keeps them). A loop that makes and discards instances, or pushes and pops,
  is refused once the budget is spent. Before, it was unlimited.
- **Strings.** A string assigned to a property is charged while the property holds
  it, at its stored length (after NUL truncation and UTF-8 repair). A host's change
  refunds it.
- **Blobs and fonts.** A blob or decoded font stays charged while anything in the
  runtime keeps it, including properties it was bound into, and is refunded when
  the last holder lets go. A host clear refunds immediately.
- **Text and faces.** `text:clear()` now refunds a face that only the Text kept. The
  old font list kept it until the Text was collected.
- **Text and gradients.** A Text keeps the gradient charge of each paint it styles
  with, until `clear` or collection.
- **Derived fonts.** Each is charged its own tables: about 700 B with no options,
  plus about 35 B per feature or axis.
- **`addToPath`.** The charge stays on the path.
- **Texts with many runs.** These are charged much less (linear), so some that were
  refused now fit.
- **Layouts too large for the budget.** These fail with the same error, but before
  they are built.
- **Nested slots and list inserts.** Putting an instance in its own nested slot,
  in a slot of something it holds, or nesting the graph past 256 deep now
  raises a script error. These used to crash with a stack overflow.
- **New instances.** A named copy is charged for what it copies. Each
  copied string keeps its own share, which is refunded when the script
  replaces it. `model:new()` is charged for what the schema makes, not for
  the live instance.
- **Rolled-back strings.** A host transaction that overwrites a script's
  string and is then rolled back leaves the string charged.
- **Canvas descriptors.** A canvas descriptor's `__index` can draw with the
  enclosing draw's renderer.

## Every remaining way to hold uncharged host memory

Found while working on this lane (not a full audit):

1. **Data-binding fan-out of strings.** A script-set string is copied into every
   bound target and into change-capture snapshots, uncharged. Its count is
   bounded by the File's bindings, not by the script.
2. **Font caches.** `HbFont::glyph_path_cache` and `color_layer_cache` (one per
   variant) fill as glyphs are drawn, uncharged. They are bounded by the face's
   glyph count times outline size, per variant, and each variant is charged its
   tables only.
3. **Blob bytes.** `RuntimeBlobAsset::bytes_arc()` hands out the bytes alone. A
   consumer that keeps the `Arc<[u8]>` past the blob keeps the bytes after the
   charge is refunded.
4. **The emoji image cache.** `RawText`'s cache of decoded color-glyph images is
   not in `heap_bytes`, and backend images do not report their size.
5. **Deferred recordings.** Renderer deferred recordings hold render paints, and so
   gradient shaders, past the paint (lane G2/H3 territory).
6. **Transient layout memory.** The transient memory named in the Text bound above
   (the shaped paragraphs, the line breaks and one glyph outline). Also the glyph
   lookup, which is linear in the content and is charged at settle, after it is
   built.
7. **`withOptions` input vectors.** The coordinate and feature vectors `withOptions`
   builds from the script's tables are transient, and bounded by those tables, which
   are on the Lua heap.
8. **Per-variant shaping coordinates.** The shaping instance's per-axis coordinates
   are bounded by the face's axis count and not counted in `variant_bytes_bound`.
9. **The quadratic line copies.** These are still built; they are only admitted
   now. A legitimate wrapped text is expensive: 4.9 KB of text costs 14.9 MB. The
   fix is a redesign so that `OrderedRun` shares the paragraph's runs (upstream keeps
   `const GlyphRun*`). That redesign is a follow-up and is outside this lane's bound.

## Edits outside the listed files

- `crates/nuxie-runtime/src/assets/blob_asset.rs` (H1's area): an additive
  `host_owner` field, `with_host_owner` and `host_owner()`. Nothing else in the
  file changed. Item 1 needs it because blobs spread through data binding as
  `Arc<RuntimeBlobAsset>`.
- `crates/nuxie-runtime/src/host_viewmodel.rs`: the module file of the owned
  `host_viewmodel/`, given two `mod` and `pub use` lines.

## Checks

Final tip `54fd4825b8`, run 2026-11-15 00:05–00:13 local, with
`NUXIE_EXECUTION=local` and `RIVE_RUNTIME_DIR` set to the cd5351c3 pin. The
tree was left untouched during the run. Load averages (1/5/15 min) were taken
at the start of each step; other lanes were building at the same time.

| Check | Result | Load at start |
| --- | --- | --- |
| `runtime.py test -p nuxie-scripting` | 38/39 targets. `processor_stream_readers_are_released_when_collected` failed (see below). | 10.06 / 11.84 / 13.13 |
| `runtime.py test -p nuxie-runtime` | 256/256 | 9.35 / 11.44 / 12.91 |
| `runtime.py test -p nuxie --features scripting,test-support` | 41/42. Only the known red `stateful_component_image_bind` failed. | 12.88 / 14.48 / 14.09 |
| `make scripted-golden-compare` | exact 348, diverges 16 (entries 364) | 11.09 / 14.17 / 14.06 |
| `make golden-compare` | exact 356, diverges 8 (entries 364) | 8.54 / 12.99 / 13.62 |
| `tools/ratchets/panic-lints.sh` | PASS: nuxie-scripting at 0 forced sites (and `@wasm` at 0) | 8.62 / 12.40 / 13.37 |
| `tools/ratchets/unsafe-count.sh` | PASS: 2,213 sites over 6 crates, unchanged | 8.54 / 12.25 / 13.31 |

**The audio-processor test.** This lane does not touch
`processor_stream_readers_are_released_when_collected`
(`luau_audio_processor_engine.rs`). It fails with
`audio processor execution budget exceeded`, which is the 42 ms per-block
deadline in a debug build. It failed in three full runs at load 10–20.
- Run alone on this tip: failed once at load 9.8, then passed at load 11.3.
- In the two earlier full suites on this branch, it passed at load 5.6–9.9.

This is a timing flake. I did not run it on the base branch for comparison.

**Earlier gate runs** on intermediate tips gave the same goldens and ratchets.
On `2523966787`, `golden-compare` and `panic-lints` were invalid because I
edited files during the run. The runner's provenance guard refused, and the
lint counted a mid-edit `len() + 1`, which is now `saturating_add`.

**Disk.** Free space was 59–104 GB (df -k, decimal) and never under 60 GB at
the start of a build. `df -g` read 60 GiB once, at 23:30.


## Not verified, and where the brief was wrong

- **Lane G3's "a refused copy is dropped".** This was wrong: the arena keeps it.
  The copy is now measured before it is made.
- **Item 1's "exactly as long as the native value".** For instances and list items
  this is the File's life, so their charge never comes back while the File lives.
  That is correct, but stricter than the per-wrapper refund scripts saw before.
- **Item 3's input bound.** An input bound is not workable, per the 67×–3,060×
  spread above.
- **The 4 GB claim.** The test for it was not run on the old code.
- **Intermediate commits.** Not built one by one; each group's tip was built.
- **Tests run against the old code.** Only the cycle-guard test
  (`8004f47681`) and the item 1–4 tests were. That the review-fix tests from
  rounds 2–4 fail on the old code is reasoned, not run.
- **The audio test** `processor_stream_readers_are_released_when_collected`.
  It is load-sensitive and not in this lane's files. I did not compare it
  against the base branch.
- **Round-4 fixes.** No codex round after them; the brief allowed four.
