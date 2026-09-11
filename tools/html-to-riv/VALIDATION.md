# Immutable-target validation

The runtime identity is pinned in [TARGET.md](TARGET.md). Native evidence from the reverted implementation is invalid for this target. Current public admission is documented separately in [SUPPORT.md](SUPPORT.md); parser/API success is not visual qualification.

## Reproducible checks

Run from repository root unless stated otherwise. Authoring dependencies are resolved only by this module's standalone Cargo.lock/package-lock.json.

```sh
python3 tools/html-to-riv/validation/check-target-runtime.py
cargo test --manifest-path tools/html-to-riv/Cargo.toml --locked
cargo build --manifest-path tools/html-to-riv/Cargo.toml --locked
```

The target check compares pinned source outside this compiler module, including staged/committed and working-tree changes. Run it again after build/feature validation. Do not alter the runtime workspace dependency resolution or features to make a compiler test pass.

For WASM, rustup must have `wasm32-unknown-unknown` installed. On hosts where default rustc is Homebrew's compiler, explicitly select rustup's compiler:

```sh
RUSTC="$(rustup which rustc)" cargo build \
  --manifest-path tools/html-to-riv/Cargo.toml --locked \
  --target wasm32-unknown-unknown --lib
npm ci --prefix tools/html-to-riv
node --test tools/html-to-riv/tests/transport-parity.mjs
node tools/html-to-riv/node_modules/typescript/bin/tsc \
  --noEmit --strict --module nodenext --moduleResolution nodenext \
  --target es2022 --lib es2022,dom tools/html-to-riv/tests/types.mts
python3 tools/html-to-riv/validation/check-target-runtime.py
```

Transport tests default to `tools/html-to-riv/target/debug/html-to-riv` and `tools/html-to-riv/target/wasm32-unknown-unknown/debug/nuxie_html_to_riv.wasm`; override with `HTML_TO_RIV_BIN` and `HTML_TO_RIV_WASM` to test frozen artifacts. They require `public-baseline-cases.json`, `public-color-palettes.json` `public-inheritance-cases.json` `public-css-wide-cases.json` and `public-background-cases.json`; missing fixtures fail rather than skip.

The nine Node tests cover exact Rive bytes/source maps across CLI/WASM, all 15 baseline cases, both palettes and both inheritance scenes and 17 CSS-wide sizing/color scenes and 14 shorthand scenes at three viewports, rejected-style diagnostic parity, strict document contracts, output ownership across failures/reuse, and direct ABI 2 buffer reset checks. The TypeScript test checks success/failure narrowing, source-map types and compile-time rejection of incompatible versions/assets/viewport types. These are transport tests; they do not render pixels.

## Immutable native/browser run

Build a fresh baseline toolchain through the unchanged root workspace, with compiler-owned output paths:

```sh
python3 tools/html-to-riv/validation/build-baseline.py \
  tools/html-to-riv/output/my-baseline-toolchain
```

This records build command/features and dependency artifacts, then links the read-only probe to the exact baseline runtime rlibs. Native receipts must bind the effective dependency graph, command/features, environment, binary hashes and unchanged import path; source identity alone is insufficient. Metal builds require a supported Apple host/toolchain. Use a fresh output directory for each command below:

```sh
node tools/html-to-riv/validation/check-public-baseline.mjs \
  tools/html-to-riv/validation/public-baseline-cases.json \
  tools/html-to-riv/target/debug/html-to-riv \
  tools/html-to-riv/output/my-baseline-toolchain/baseline-probe \
  tools/html-to-riv/output/my-baseline-toolchain/renderer-replay \
  tools/html-to-riv/output/my-public-baseline

node tools/html-to-riv/validation/check-public-baseline.mjs \
  tools/html-to-riv/validation/public-color-palettes.json \
  tools/html-to-riv/target/debug/html-to-riv \
  tools/html-to-riv/output/my-baseline-toolchain/baseline-probe \
  tools/html-to-riv/output/my-baseline-toolchain/renderer-replay \
  tools/html-to-riv/output/my-color-palettes
```

Install the Playwright-managed Chromium binary if absent (`tools/html-to-riv/node_modules/.bin/playwright install chromium`). The driver requires Chrome 153.0.8010.12, device scale 1 and `src/reset.css`. It compiles each fixture once at 390×160, imports **only `.riv`** into the baseline observer, and resizes original and clone through four viewports. Default sequence:240×160→390×200→768×120→240×160; palettes supply explicit taller sizes so every swatch is visible. Source maps are used only afterward to join measured object bounds to DOM IDs. No policy or map is passed into import.

The baseline Metal backend is raster ordering. Its historical CLI mode token must not be mistaken for proof that the forced-clockwise or newly added ordinary-Atomics APIs are present. Read `effectiveMode` and bound toolchain evidence in each receipt. Do not transfer corrected Atomics images from the reverted renderer.

## Qualification workflow

1. Record accepted semantics and context-dependent rejections, with an ordinary wire property or proposed file-level composition.
2. Run authoring/transport tests and baseline bytes-only import; metadata must be dispensable.
3. Compile once, resize originals/clones without recompilation, and include changed height/aspect ratio and return-to-start checks.
4. Compare pinned Chrome geometry and actual native PNGs under existing gates in `validation/pixels.mjs`; do not widen tolerances to pass a feature.
5. Inspect complete unique image pairs, including every visible palette cell. Preserve failures. Bind reviews to exact HTML/CSS/reset/assets/viewports, emitted bytes, stream, native/browser PNG and immutable toolchain hashes.
6. Add malformed/resource boundaries, unsupported contexts and relevant composition tests. Mark only the demonstrated profile qualified; alternative encodings require their own evidence.

Current durable public evidence is `validation/public-color-receipt.json`: 136 frames passed geometry/pixel gates (120 baseline and 16 palette frames). All six unique palette pairs were reviewed; 45 unchanged baseline pairs transferred prior review through exact image identity. The subsequent inheritance correction is bound in `validation/public-inheritance-receipt.json`: 24 additional passing/reviewed frames and 8 preserved pre-fix pixel failures. The final compiler checkpoint binds binaries and source hashes in `output/public-compiler-checkpoint-r2/manifest.json`, including exact recompiled bytes/maps for all 20 rendered scenes. The initial scope remains provisional: this does not establish a qualified release or broader CSS admission.

Earlier source audits cover 36 former custom capabilities, 44 renderer/interface/stream files and 62 runtime/vendor paths; these establish mutation scope, not replacement support. The first 56-frame ordinary-layout experiment and its preserved corner failures remain documented in `validation/ordinary-layout-review.md`.

Run `public-inheritance-cases.json` through the same native driver command above to reproduce the inherited currentColor and literal-control scenes.

The CSS-wide corpus uses the same native driver with `public-css-wide-cases.json`. Its 136 passing frames and full visual coverage are bound in `validation/public-css-wide-receipt.json`. New driver runs preserve the exact comparator scripts, reset hash and browser-computed styles automatically.

## Freeze and verify an evidence checkpoint

After building and testing the compiler, freeze the native/WASM binaries and reproduce recorded ordinary files with:

```sh
python3 tools/html-to-riv/validation/freeze-public-compiler.py \
  tools/html-to-riv/target/debug/html-to-riv \
  tools/html-to-riv/target/wasm32-unknown-unknown/debug/nuxie_html_to_riv.wasm \
  tools/html-to-riv/output/my-frozen-checkpoint \
  tools/html-to-riv/output/my-public-baseline/receipt.json
```

Supply additional passed run receipts as trailing arguments. A fresh output directory is required. The command checks actual source requests, maps, Rive files, probe manifests, streams, geometry and PNG hashes; verifies original/clone frame coverage; and compares entire recompiled Rive/map files. It snapshots binaries, current sources and available matching comparison scripts, records commands and rejects mutation during the run. Six negative/success controls run with `python3 tools/html-to-riv/validation/test-freeze-public-compiler.py`.

This verifies exact-file reuse of existing render evidence. It does not rerender or review images and cannot independently prove that a supplied binary was built from its contemporaneous source snapshot. Preserve build commands/results and the immutable toolchain manifest separately. The CSS-wide checkpoint is `output/public-css-wide-checkpoint-r1/manifest.json`.

The current driver also renders every frame against cyan and transparent clears and requires exact native image equality with the white-clear run. The explicit reset requires an opaque white host encoded in the file. `validation/host-background-control.json` preserves the prior missing-paint failure. `validation/public-background-receipt.json` binds a fresh 51-scene/408-frame run, 816 clear controls, all visual review transfers and `output/public-background-checkpoint-r1/manifest.json`. Use `public-background-cases.json` with the same driver command for the new shorthand subset. Comparator sources, fixtures and reset are checked for mutation during each run.
