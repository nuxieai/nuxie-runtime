# Duo twobindrel fixture

`duo-twobindrel.nux` is Duolingo's Duo, rebuilt as a Nuxie Experience: a
question Screen and a lesson Screen joined by a Magic Move that carries Duo,
the Continue button and the progress bar. `live_moves_carried_binds.rs` plays
it through the C API in the iOS SDK's order.

SHA-256: `2fe33bf5c27699777e7e9d5d14909dabd6c2d4c3cd47d6b000c083abf7695357`
(84468 bytes).

It was published at nuxie-dev `2a163d3a84647e8d3ec1c9a09b8d962de8eda339`,
which includes the press target flag (`4581aeaaefa`), so a control's press is
opaque to what is under it. The source folder is the Magic Move handoff's
`references/duolingo-duo/ours-levi-read/take-1/variants/twobindrel/experience`.
It is Duo's folder with two changes:

- Continue's `data-bind:disabled="state.ctaOff"` is removed on both Screens.
  With it, the folder does not open
  ([UNIV-5924](https://universe.basis.dev/issue/UNIV-5924)).
- `.option-text` on the question Screen has `position: relative`, so the
  option words show.

Regenerate in nuxie-dev with

```sh
bash tools/nuxie-editor/scripts/bazel.sh run -p experience-editor-bridge \
  --example publish_folder -- <folder> <out dir>
```

and copy `<out dir>/screen.riv` here.

What the tests rely on:

- Continue sits at (201, 798) and Back at (30, 88) on a 402 by 874 page with
  the iPhone 17 Pro safe area (top 62, bottom 34). The lesson emits
  `continue` and `back` from them.
- The lesson's appear sets `state.pose` to `cheer`. Duo holds his standing
  pose for 100 ms, then cheers. Only the cheering drawing has the raised
  wing's yellow dot (`#ffc800`, a circle 10 units across in a 100-unit
  view box drawn at 108 points). The question's other yellow is a 22 by 24
  icon.
