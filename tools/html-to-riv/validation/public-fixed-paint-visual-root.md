# Direct fixed paint primitive: visual review

Root directly inspected all six unscaled Chrome/native/diff sheets at original resolution, covering 18 representative frame pairs. The capture records 48 passing pixel gates and 96 alternate-clear comparisons; 30 exact repeated image pairs cover return-to-initial and cloned instances. This is a primitive experiment with directly authored rectangle descriptors, not public HTML/CSS folding or a proof of source-derived geometry.

No visible edge-placement, clipping, missing-paint or paint-order mismatch was observed. Reversing the two alpha rectangles changes the overlap color in both renderings as expected. The full-envelope rectangle covers each resized artboard, while transparent and empty/inverted rectangles add no visible paint. Half-integer centers preserve the intended integer edges. Faint filled-region differences remain visible in the diff panels; passing gates do not mean identical RGBA bytes.

Sheets inspected (each contains frames 0, 1 and 2):

- `tools/html-to-riv/output/playwright/public-fixed-paint-primitive-r1/visual/signed-clipped.png` — SHA256 `ef3b398bdda23d24dc0f54ebecdb87414ce140a79ae6867da1788fa2497d7038`.
- `tools/html-to-riv/output/playwright/public-fixed-paint-primitive-r1/visual/half-centers.png` — SHA256 `36f7edcf5cc46807320778cd2e81b7577852e2e7d236f90fbbd331c82ce436e4`.
- `tools/html-to-riv/output/playwright/public-fixed-paint-primitive-r1/visual/alpha-a-then-b.png` — SHA256 `f1b63f0f3704e3c315ee58556583144fea592220bbecf5369d0b2b22d67df9ef`.
- `tools/html-to-riv/output/playwright/public-fixed-paint-primitive-r1/visual/alpha-b-then-a.png` — SHA256 `ba5e5c29ddeba6edee99f466858305057176dd25f48b349ee12d8607322f8eef`.
- `tools/html-to-riv/output/playwright/public-fixed-paint-primitive-r1/visual/full-envelope.png` — SHA256 `0228eca4890ad563b81cfa1fa5c770ae5d16dc6ec2b375a60b2aa6b570f7e200`.
- `tools/html-to-riv/output/playwright/public-fixed-paint-primitive-r1/visual/invisible-and-empty.png` — SHA256 `06c580ddcbb481b258dfc8711acb13b5f5f56835bde2806eb8e8a87840e31631`.

This review does not independently verify native command grammar, internal GPU state, final constrained layout coordinates, or whole-scene folded equivalence. Those are separate checks.
