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
- `nuxie-render-api`: renderer-neutral traits
- `nuxie-scripting`: optional pure-Rust Luau integration
- `nux-capi`: the sole static-library distribution root, exposing the portable
  C API plus narrow Apple Metal and headless Android Vulkan extensions
- `nuxie-project-data`: authoring/project conversion kept outside baseline
  runtime closures and composed only into product distribution roots

## Development

The compatibility oracle uses a separate checkout of the upstream C++ runtime:

```sh
tools/bazel/install.sh
export RIVE_RUNTIME_DIR=/path/to/rive-runtime
make fixtures
make test
make golden-compare
make scripted-golden-compare
make capi-smoke
```

Rust compilation uses Bazel 9.3.0 and `rules_rust` with Rust 1.94.1. Each crate,
build script, test, and executable is a direct Bazel target. The package frontend
keeps familiar selectors and stages complete artifacts under `target/` for the
existing native and browser consumers:

```sh
tools/bazel/runtime.py build -p nux-capi
tools/bazel/runtime.py test -p nux-capi --lib --test capi
tools/bazel/runtime.py run -p nuxie-codegen -- --help
```

The runtime shares Bazel action, dependency-download, and fetched repository
caches with other Nuxie Bazel workspaces under `~/.cache/nuxie/bazel`. Each Git
worktree retains its own Bazel output base and `target/` products. Set an absolute `NUXIE_BAZEL_CACHE_DIR`
when using the package or distribution frontends to relocate the reusable
caches; leave the output base at its checkout-specific default. Test the cache
override with `python3 -B -m unittest discover -s tools -p 'test_bazel_cache.py'`.

Cargo manifests remain the dependency and feature authority. Run
`python3 tools/bazel/generate.py` after changing them and use `--check` to verify
checked-in targets. `bazel/cargo` contains dependency-resolution stubs;
compilation always uses the original sources. Patched dependency defaults apply
through incoming edges, and their tests and examples stay outside the authored
workspace. Dependency changes require repinning `bazel/cargo/Cargo.Bazel.lock`;
`--reset-lock` seeds its Cargo input from the authored lockfile. Cargo also remains
available for formatting and the mandatory compatibility tests.

The `audio-device`, native Metal replay, and `scriptnet` tool cuts resolve through
`bazel/native-tools-cargo` and `runtime_native_tools_crates`. Their optional
external features stay separate from the SDK shipping dependency pins. Repin
each changed registry independently with `CARGO_BAZEL_REPIN_ONLY` set to its
repository name.

`make golden-compare` compares deterministic render-call streams from the Rust
runtime and the upstream C++ reference. The C++ runtime is a development and CI
dependency only; it is not linked into or shipped with the Nuxie SDK.
The fixture bootstrap pins and verifies the small upstream test-asset set;
those `.riv` binaries are intentionally not stored in this repository.

Nuxie-specific experience, package, authentication, and SDK-session behavior
lives above this repository's shipped runtime. `nux-capi` is the sole static
library distribution root: its portable base composes generic scripting; the
Apple Metal and Android Vulkan extensions share portable image/asset hooks.
Apple owns Metal presentation, while Android returns owned headless Vulkan
frames for an SDK to blit. See [Apple C runtime distribution](docs/nux-capi-apple-release.md).
The iOS SDK consumes the published binary through a pure Swift package layer
and does not compile Rust.

## License

MIT. See [LICENSE](LICENSE) and [THIRD_PARTY_NOTICES.md](THIRD_PARTY_NOTICES.md).
