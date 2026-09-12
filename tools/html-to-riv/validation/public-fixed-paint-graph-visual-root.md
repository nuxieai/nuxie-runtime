# Private preserved-paint graph visual review

Root directly inspected all six full-size sheets 00–05 in `output/playwright/public-fixed-paint-graph-r1/folded-capture/public-visual`. They cover 15 representative native/Chrome pairs; the visual receipt establishes 25 exact repeats for all 40 frames. The fixture profile is private source-authored fixed geometry, not public compiler admission. Original and clone resize the same file.

The row/column and reverse-direction examples preserve placement, viewport clipping, visible color regions and ordering. The private overlap case preserves the two alpha-overlap bands and yellow tail. No missing paint or misplaced edge is visible. Diff panels retain faint filled-region differences from Chrome; the claim is passing existing Chrome gates, not exact Chrome pixels. The narrow 19×31 sheets truncate label text but keep the actual image panels complete and unscaled. These observations do not qualify responsive reflow or other fixtures.

Directly reviewed sheet identities:

```json
[
  {
    "path": "/Users/levi/.codex/worktrees/html-css-immutable/tools/html-to-riv/output/playwright/public-fixed-paint-graph-r1/folded-capture/public-visual/sheet-00.png",
    "sha256": "8f9883f09234857e5466f8c384fda8b56f8892f46ae395c8c05e88b3888c4e11"
  },
  {
    "path": "/Users/levi/.codex/worktrees/html-css-immutable/tools/html-to-riv/output/playwright/public-fixed-paint-graph-r1/folded-capture/public-visual/sheet-01.png",
    "sha256": "f25b249e9433dc2bcca1148ebdb71de264d2f28040db6adf74a89f60a96dd762"
  },
  {
    "path": "/Users/levi/.codex/worktrees/html-css-immutable/tools/html-to-riv/output/playwright/public-fixed-paint-graph-r1/folded-capture/public-visual/sheet-02.png",
    "sha256": "e03862bf3d51b140a41677e274383cd329caa05ad09128e6b44d3d0dad33ef2f"
  },
  {
    "path": "/Users/levi/.codex/worktrees/html-css-immutable/tools/html-to-riv/output/playwright/public-fixed-paint-graph-r1/folded-capture/public-visual/sheet-03.png",
    "sha256": "0ccf8083e7664800d3ae906936729969d92da4b206a1ac0613fdbe4712e3cb06"
  },
  {
    "path": "/Users/levi/.codex/worktrees/html-css-immutable/tools/html-to-riv/output/playwright/public-fixed-paint-graph-r1/folded-capture/public-visual/sheet-04.png",
    "sha256": "3d310de3910bd96149db5cc73b4742addd3ea6e30cc8bfb1ac1405f0e9fb034b"
  },
  {
    "path": "/Users/levi/.codex/worktrees/html-css-immutable/tools/html-to-riv/output/playwright/public-fixed-paint-graph-r1/folded-capture/public-visual/sheet-05.png",
    "sha256": "91484debac68d4711e424c303a0f3c7bf058bede89ec39b3a640780bd676f140"
  }
]
```
