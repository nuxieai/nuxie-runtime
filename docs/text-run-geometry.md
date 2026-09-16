# Frame-qualified text geometry

`nux_player_text_run_geometry` reads the geometry of an exact-name root
`TextValueRun` from a successful player step. The caller supplies the player,
the still-live step result, and the authored run name. The result contains no
text, glyph data, or borrowed scene pointers.

The step result carries weak artboard-occurrence ownership. A foreign occurrence,
a failed result, or a changed render revision returns `HANDLE_MISMATCH`. View-model
and renderer invalidations are refreshed before comparing revisions. Normal
creator-thread, reentrancy, handle lifetime, and poisoned-occurrence guards apply.
An absent name returns `NOT_FOUND`; duplicate root names return `INVALID_ARGUMENT`.
Nested artboard occurrences are not searched. Output is written only on success.

Capture geometry while retaining the successful step result, alongside the frame
that will be presented. Publication must accept or discard the pixels and native
overlay using the same returned render revision. The read does not require a
presentation acknowledgement, advance the player, or clear render demand. Copying
geometry does not authorize later publication after an intervening mutation.

The output preserves separate coordinate spaces:

- `world_transform` maps local text-layout coordinates to artboard coordinates,
  including ancestor translation, rotation, scale, and layout.
- `content_transform` also includes the text runtime's internal fitting and
  alignment transform. It maps shaped content coordinates, not the layout box.
- `min_x`, `min_y`, `max_x`, and `max_y` describe the settled local layout box,
  including origin and baseline offsets. Map these through `world_transform`.
  They are not glyph ink bounds.
- When `has_layout_ancestor` is one, the `layout_ancestor_*` fields identify the
  nearest enclosing `LayoutComponent` and its local bounds. The lookup walks
  structural parents, excludes the text itself, and never searches by display
  name. Missing ancestors produce zero fields. This is a generic layout
  relationship, not semantic field authorization. The publisher's compound
  editable-field contract must guarantee that this owner is its field box.
- When `has_first_baseline` is one, `first_baseline` gives the first logical
  line's y coordinate before internal fitting/alignment. Map it through
  `content_transform` to align the native control's measured first baseline.
  It remains the first logical baseline when clipping hides that line;
  empty/unshaped text reports no baseline.

Each matrix uses `[a, b, c, d, tx, ty]`, with `x' = a*x + c*y + tx` and
`y' = b*x + d*y + ty`. Preserve the full affine basis; decomposing an inherited
nonuniform scale and rotation into an axis-aligned rectangle loses information.
Non-finite geometry returns `RUNTIME_ERROR`. The host still owns viewport fitting,
native editing, focus, composition, clipping, and authenticated field admission.

The C-API tests verify inherited affine transforms and stale/foreign/ambiguous
ownership rejection. The Apple product-import test uses the committed SDK font
fixture to distinguish the authored field box from its painted text origin and
checks the shaped first baseline against independent OpenType ascent values. Native content-box/baseline alignment with actual fonts,
public binary qualification, and SDK adoption remain separate acceptance work
under [UNIV-3199](https://universe.basis.dev/issue/UNIV-3199).
