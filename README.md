# Nuxie Runtime

An independent, pure-Rust runtime for Nuxie flows, compatible with the Rive
(`.riv`) file format. This project is not affiliated with or endorsed by Rive
Inc.

The workspace provides file import, artboard instancing, animation and state
machines, data binding, layout and text, scripting, renderer-neutral draw
commands, a public Rust API, and a C ABI for embedded SDK integrations.

## Workspace

- `nuxie`: public Rust API
- `nuxie-renderer`: default pure-Rust renderer with native and browser backends
- `nux-capi`: C SDK surface and `nux_capi.h`
- `nuxie-runtime`: artboard, animation, state-machine, and draw runtime
- `nuxie-binary`: `.riv` importer
- `nuxie-graph`: imported component graph
- `nuxie-render-api`: renderer-neutral traits
- `nuxie-scripting`: optional pure-Rust Luau integration

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

## License

MIT. See [LICENSE](LICENSE) and [THIRD_PARTY_NOTICES.md](THIRD_PARTY_NOTICES.md).
