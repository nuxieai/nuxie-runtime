# Duo twobindrel fixture

`duo-twobindrel.nux` is Duolingo's Duo, rebuilt in Nuxie: a
question Screen and a lesson Screen joined by a Magic Move that transfers Duo
and the progress bar, both marked transferable, and flies the Continue
button's two copies. `live_moves_carried_binds.rs` plays it through the C API
in the iOS SDK's order.

SHA-256: `bd638be8ae67fe001705edc8588126d86323dc038d4224413e47913a3b762361`
(75244 bytes).

It was published at nuxie-dev `4859f66572da4ad8954549d65b9f195d0730ca0e` with
this repository at `42f21c6fe437a4a8f863f247928ebf478f97a0c1`. That compiler
flies elements in place: it writes each step unit's `transfers`, flight records
and no slot records. The source folder
is the `twobindrel` variant under the Magic Move handoff's
`references/duolingo-duo/ours-levi-read/take-1/variants/`, with `"transferable": true` in `components/duo-mascot/frame.json` and
`components/lesson-progress/frame.json`, as Duo's own folder marks them (a
copy is at `runs/m4-next/unit22/runtime-pr/duo-twobindrel-marked`). The
variant is Duo's folder with two changes:

- Continue's `data-bind:disabled="state.ctaOff"` is removed on both Screens.
  With it, the folder does not open
  ([UNIV-5924](https://universe.basis.dev/issue/UNIV-5924)).
- `.option-text` on the question Screen has `position: relative`, so the
  option words show.

Regenerate in nuxie-dev by running the editor bridge crate's
`publish_folder` example through `tools/nuxie-editor/scripts/bazel.sh` with
the folder and an output directory, and copy `screen.riv` from that output
here.

What the tests rely on:

- Continue sits at (201, 798) and Back at (30, 88) on a 402 by 874 page with
  the iPhone 17 Pro safe area (top 62, bottom 34). The lesson emits
  `continue` and `back` from them.
- The lesson's appear sets `state.pose` to `cheer`. Duo holds his standing
  pose for 100 ms, then cheers. Only the cheering drawing has the raised
  wing's yellow dot (`#ffc800`, a circle 10 units across in a 100-unit
  view box drawn at 108 points). The question's other yellow is a 22 by 24
  icon.
