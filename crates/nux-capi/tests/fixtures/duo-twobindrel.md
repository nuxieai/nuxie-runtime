# Duo twobindrel fixture

`duo-twobindrel.nux` is Duolingo's Duo, rebuilt as a Nuxie Experience: a
question Screen and a lesson Screen joined by a Magic Move that transfers Duo
and the progress bar, both marked transferable, and flies the Continue
button's two copies. `live_moves_carried_binds.rs` plays it through the C API
in the iOS SDK's order.

SHA-256: `f2b3e65202d1e3c47ae05fa99f77cbbfedf9862d5e69301ca322d55e194acc23`
(95426 bytes).

It was published at nuxie-dev `a4f1df8cfdafab515ad45ba216fbf2f3050d24e3`,
whose compiler writes each step unit's `transfers`; a file published before
it says nothing about transfers and the runtime refuses it. The source folder
is the Magic Move handoff's
`references/duolingo-duo/ours-levi-read/take-1/variants/twobindrel/experience`
with `"transferable": true` in `components/duo-mascot/frame.json` and
`components/lesson-progress/frame.json`, as Duo's own folder marks them (a
copy is at `runs/m4-next/unit22/runtime-pr/duo-twobindrel-marked`). The
variant is Duo's folder with two changes:

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
