# Published text precision regression

Copied byte-for-byte from the Nuxie iOS canonical `fixtures/runtime/font-metrics-binding` fixture at commit `25c569b1608ab1b5122572c67354412d0372cefd`.

`provenance.json` retains the production publisher revision, wasm identity, source snapshot identity, RIV SHA-256 and external-font SHA-256. The regression imports the actual static Paywall artboard and font and compares owned offscreen frames across Metal shader compilation modes. It does not depend on an SDK checkout or a local SDK runtime override.

This fixture reproduced 15 changed text-edge pixels between specialized and precompiled Metal shaders before matching vertex coverage precision. Keep exact pixel equality; do not replace the published text with a simpler filled shape that does not reproduce the defect.
