# Nuxie Runtime

An independent, pure-Rust interactive graphics runtime compatible with the Rive
(`.riv`) file format. This project is not affiliated with or endorsed by Rive
Inc.

The workspace provides file import, artboard instancing, animation and state
machines, data binding, layout and text, scripting, renderer-neutral draw
commands, a public Rust API, and a C ABI for embedded SDK integrations.

## Workspace

- `nuxie`: public Rust API
- `nuxie-renderer`: default pure-Rust renderer with native and browser backends
- `nuxie-runtime`: artboard, animation, state-machine, and draw runtime
- `nuxie-binary`: `.riv` importer
- `nuxie-graph`: imported component graph
- `nuxie-render-api`: renderer-neutral traits
- `nuxie-scripting`: optional pure-Rust Luau integration
- `nux-capi`: the sole static-library distribution root, exposing the portable
  C API plus narrow Apple Metal, image-decoding, and asset hooks
- `nuxie-project-data`: authoring/project conversion kept outside the shipped
  runtime dependency closure

## Development

The compatibility oracle uses a separate checkout of the upstream C++ runtime:

```sh
export RIVE_RUNTIME_DIR=/path/to/rive-runtime
make fixtures
cargo test --workspace
make golden-compare
make scripted-golden-compare
make capi-smoke
```

`make golden-compare` compares deterministic render-call streams from the Rust
runtime and the upstream C++ reference. The C++ runtime is a development and CI
dependency only; it is not linked into or shipped with the Nuxie SDK.
The fixture bootstrap pins and verifies the small upstream test-asset set;
those `.riv` binaries are intentionally not stored in this repository.

Nuxie-specific experience, package, authentication, and SDK-session behavior
lives above this repository's shipped runtime. `nux-capi` is the sole static
library distribution root: its portable base composes generic scripting, and
its Apple extension adds renderer-owned Metal presentation and image/asset
hooks. See [Apple C runtime distribution](docs/nux-capi-apple-release.md). The
iOS SDK consumes the published binary through a pure Swift package layer and
does not compile Rust.

## License

MIT. See [LICENSE](LICENSE) and [THIRD_PARTY_NOTICES.md](THIRD_PARTY_NOTICES.md).
