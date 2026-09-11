# HTML/CSS to ordinary Rive

An initial standalone compiler is available through Rust, CLI and WASM/JavaScript. It emits ordinary `.riv` bytes for the unchanged runtime pinned in [TARGET.md](TARGET.md). The currently admitted profile is deliberately small: nested box elements, single-line flex directions and order, width/height with inheritance and initial/unset sizing, selectors/cascade, solid colors and solid-color background shorthand. Native visual qualification remains provisional; this is not a qualified release or a general web-page importer.

PR #628 was reverted by [PR #629](https://github.com/nuxieai/nuxie-runtime/pull/629) because it depended on runtime and renderer extensions. This replacement has its own Cargo workspace and authoring dependencies. It requires no CSS runtime policies, custom renderer methods, requirements sidecar or editor integration. The historical implementation at `20248ee6835a7bb071dbf83a364606f5e58aeef9` is reference material; its native qualification does not transfer.

## Compile a document

From the repository root:

```sh
cargo build --manifest-path tools/html-to-riv/Cargo.toml --locked
```

Save an input JSON file:

```json
{
  "html": "<div id=\"box\"></div>",
  "css": "#box { width: 100%; height: 80px; background-color: rebeccapurple; }",
  "width": 390,
  "height": 160
}
```

```sh
tools/html-to-riv/target/debug/html-to-riv input.json output.riv
```

Success writes `output.riv` and `output.map.json`. The map is authoring metadata containing `{id,path,object_id}` entries; the runtime loads the Rive file without it. No `.requirements.json` is produced. Input accepts exactly `html`, `css`, `width` and `height`; assets and unknown fields are rejected. Compilation diagnostics are JSON arrays on stderr with a nonzero exit status. Rejected compilation writes neither output file. Successful output writes are not transactional: an I/O failure can leave a partial `.riv`/map pair.

Rust exposes `compile(&CompileInput) -> Result<CompileOutput, Vec<Diagnostic>>`. `CompileOutput` contains `riv: Vec<u8>` and `source_map: Vec<SourceNode>`. Diagnostics contain `code`, `source` and `message` strings.

## WASM and JavaScript

Build the library for `wasm32-unknown-unknown` using the rustup compiler with that target installed; see [VALIDATION.md](VALIDATION.md). Import the module directly:

```js
import {createCompiler, LANGUAGE_VERSION} from './tools/html-to-riv/js/index.mjs';

const compiler = await createCompiler(wasmBytes);
const result = compiler.compile({
  languageVersion: LANGUAGE_VERSION,
  html: '<div id="box"></div>',
  css: '#box { width: 100%; height: 80px; background-color: rebeccapurple; }',
  width: 390,
  height: 160,
});
if (result.ok) {
  // result.riv is an owned Uint8Array; result.sourceMap is authoring metadata.
} else {
  console.error(result.diagnostics);
}
```

The language is `nuxie-html-immutable-v1`, and the private WASM bridge is ABI version2. Old language/ABI instances are incompatible. Output byte buffers remain valid across subsequent compile calls. Type declarations are in `js/index.d.mts`; the package is currently private, with no published npm artifact claimed.

The explicit authoring reset in `src/reset.css` is exported to Rust as `BROWSER_RESET_CSS` and used by browser references. The reset defaults to column flex boxes; authored directions can override it. Browser default styles are not emulated. Read [SUPPORT.md](SUPPORT.md) for admission and limitations, [VALIDATION.md](VALIDATION.md) for exact checks, and [BACKLOG.md](BACKLOG.md) for retained future work.

The fixed white page background is encoded as ordinary Artboard paint. Rendered designs do not depend on the application clearing its canvas to white.
