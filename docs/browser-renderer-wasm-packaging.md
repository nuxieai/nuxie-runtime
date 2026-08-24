# Browser renderer Wasm packaging

Decision date: 2026-08-24

Both shipping browser renderer products use `wasm32-unknown-unknown`. WebGPU
and WebGL2 remain separate artifacts and are selected explicitly by the
editor; neither renderer automatically falls back to the other. Emscripten is
not part of either product.

## Reproducible size measurement

The historical baseline is revision
`522b551fc70ad096dfd87653978065c5b64cce9e`. In a disposable worktree at that
revision it was built with:

```sh
CARGO="$(rustup which --toolchain 1.94.1 cargo)" \
RUSTC="$(rustup which --toolchain 1.94.1 rustc)" \
RIVE_RUNTIME_DIR=/absolute/path/to/pinned/rive-runtime \
  tools/webgpu-renderer-emscripten/build.sh
```

Its measured files were
`tools/webgpu-renderer-emscripten/pkg/renderer_replay.wasm`,
`tools/webgpu-renderer-emscripten/pkg/renderer-replay.js`, and
`tools/webgpu-renderer-emscripten/replay.html`. That build was the deleted
Emscripten product, not the frozen C++ source oracle.

The current scripts pin Rust 1.94.1 themselves and are run with:

```sh
tools/webgpu-renderer-replay/build.sh
tools/webgl2-renderer-replay/build.sh
```

The builds use Rust 1.94.1 and `wasm-bindgen-cli` 0.2.126. Raw sizes use
`wc -c < file`; compressed sizes use `gzip -9 -c file | wc -c`.

| Product | Included files | Raw bytes | gzip bytes |
| --- | --- | ---: | ---: |
| Historical WebGPU core (`wasm32-unknown-emscripten`) | replay Wasm + generated JS | 2,223,220 | 756,464 |
| Current WebGPU core (`wasm32-unknown-unknown`) | replay Wasm + wasm-bindgen JS | 1,963,207 | 661,028 |
| Historical WebGPU complete browser root | core + replay HTML | 2,228,603 | 758,239 |
| Current WebGPU complete browser root | core + direct WebGPU host + HTML | 2,005,952 | 671,386 |
| Current WebGL2 complete browser root (`wasm32-unknown-unknown`) | replay Wasm + wasm-bindgen JS + HTML | 1,800,650 | 674,479 |

The like-for-like WebGPU core is 260,013 raw bytes (11.70%) and 95,436 gzip
bytes (12.62%) smaller. The complete browser root, including the platform host
and HTML in both cases, is 222,651 raw bytes (9.99%) and 86,853 gzip bytes
(11.45%) smaller.

Current WebGPU files are
`tools/webgpu-renderer-replay/pkg/webgpu_renderer_replay_bg.wasm`,
`tools/webgpu-renderer-replay/pkg/webgpu_renderer_replay.js`,
`tools/webgpu-renderer-replay/webgpu-host.js`, and
`tools/webgpu-renderer-replay/index.html`. Current WebGL2 files are the
equivalent Wasm and JavaScript files under
`tools/webgl2-renderer-replay/pkg` plus
`tools/webgl2-renderer-replay/index.html`.

## WebGPU dependency audit

The exact Rust translation imports the pinned Dawn C ABI. The removed product
used Emscripten and Emdawnwebgpu to supply that ABI, generate JavaScript glue,
obtain the canvas surface, deliver adapter/device callbacks, wait for submitted
work, and map readback buffers.

The current Wasm import inventory is reproducible with:

```sh
wasm-objdump -x \
  tools/webgpu-renderer-replay/pkg/webgpu_renderer_replay_bg.wasm \
  | sed -n '/Import\[/,/Function\[/p'
```

It contains 85 function imports: 75 direct `env.wgpu*` imports for the pinned
Dawn ABI and 10 wasm-bindgen, js-sys, and web-time imports. The build script
retains the exported function table with `--keep-lld-exports`, and
`inject_webgpu_imports.py` deterministically connects the 75 raw imports to
`createWebGpuImports`.

The current platform seam is
`tools/webgpu-renderer-replay/webgpu-host.js`:

- it uses `navigator.gpu.requestAdapter`, `GPUAdapter.requestDevice`,
  `GPUCanvasContext`, `GPUBufferUsage`, and `GPUMapMode`;
- the browser adapter and device are obtained before entering the synchronous
  Rust factory, so the pinned callback ABI can be delivered synchronously;
- the pinned Dawn descriptors, ownership calls, and handles map directly to
  browser WebGPU, including freeing host-allocated capability and adapter-info
  members through the pinned ABI;
- surface capabilities preserve the upstream RGBA-first order;
- frame capture is appended to the renderer queue submission and mapped after
  completion, preserving the source texture bytes and alpha channel;
- canvas/device release is deferred until submitted work and capture complete;
- the replay page uses `document`, canvas, `fetch`, `TextEncoder`,
  `TextDecoder`, `CompressionStream`, `Blob`, `Response`, and `crypto.subtle`
  for broker I/O, PNG encoding, and artifact hashing.

No renderer behavior, shader compilation, or fallback path moved into the
host. The candidate product needs neither Emscripten, Emdawnwebgpu, Asyncify,
nor browser threads. Rust receives adapter/device callbacks synchronously; the
only asynchronous work is JavaScript queue completion and buffer mapping after
Rust submits the frame. The frozen C++ source oracle still uses its upstream
Emscripten integration solely to prove parity; it is not a runtime artifact.

## Parity evidence

The durable macOS Chrome full-corpus result and current artifact identities are
recorded in
`docs/evidence/browser-webgpu-wasm32-unknown-2026-08-24.json`. The tests ran the
frozen C++ source and the `wasm32-unknown-unknown` candidate in the same browser
on the same adapter, with no renderer fallback. The final host hash passed all
1,469 frozen cases with zero divergences or gated rows.

## Loading strategy

WebGPU loads `webgpu_renderer_replay_bg.wasm`, wasm-bindgen's small module,
and the direct WebGPU host. WebGL2 loads its existing
`webgl2_renderer_replay_bg.wasm` module. They remain separate because their
platform APIs, generated imports, and lifecycle contracts differ even though
their Rust compilation target is now identical.

Naga remains editor-owned. It is not linked into either runtime renderer
bundle and may be delivered independently when editor shader compilation
requires it.
