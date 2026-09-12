# Public wrapping r2 effective target audit

Pass within the preserved build-identity scope. The actual public capture probe and renderer hashes match the baseline toolchain and snapped-probe manifests. All 13 baseline manifest files were independently rehashed, as were the snapped node-probe source, linking command and binary, the two runtime library inputs, the three capture tools, and root Cargo files.

Baseline commit `6c7ac16617835b5f581784ff08a9e779bb52faf3`, tree `25ccbb131d88dbd0fdde8f4c660919143b252edf`. The current source guard returns exactly the preserved passing source identity: no index, worktree or nonignored additions outside the compiler module. Root Cargo.toml and Cargo.lock match both the manifest hashes and their baseline Git blobs.

## Recorded build and linking

The baseline Cargo command is `cargo build --locked -p rust-golden-runner -p renderer-replay --features renderer-replay/native-metal --message-format=json`, using root workspace resolution. Recorded overrides are only the local target directory, CARGO_INCREMENTAL=0, and CARGO_PROFILE_DEV_DEBUG=0; RUSTFLAGS and RUSTC_WRAPPER are null. Cargo artifact records report optimization 0, debuginfo 0, debug assertions and overflow checks enabled. No compiler workspace dependency override supplies runtime libraries.

The snapped probe's dependency and native-link argument suffix exactly matches the baseline probe command. Its nuxie_runtime and nuxie_render_api rlib paths exactly match the baseline Cargo artifact records, and their current hashes match the snapped experiment's historical dependency hashes. The current Cargo output renderer also matches the frozen renderer used by capture.

Effective recorded artifact features:

- `nuxie_runtime`: `default`, `js-host-seed`, `tools`.
- `nuxie_render_api`: (none).
- `renderer-replay`: `default`, `native-metal`.
- `taffy`: `alloc`, `block_layout`, `calc`, `content_size`, `default`, `detailed_layout_info`, `flexbox`, `float_layout`, `grid`, `std`, `taffy_tree`.

Taffy is baseline-owned `vendor/taffy-0.12.1-rive-yoga-order` version 0.12.1. Its existing grid feature is a target dependency fact, not HTML/CSS compiler grid support.

The node probe source reads only ordinary RIV bytes and viewport dimensions; it uses ordinary import/update/instance/set_size/draw APIs to record layout and commands. It does not load CSS, the compiler crate, a source map, or a requirements sidecar, or install runtime policies. The old experiment directory contains an adapter and augmenter, but neither occurs in the public capture tool list or this probe build command.

The native search path comes from the baseline Cargo libhydrogen-sys@0.9.4 build-script record (`static=hydrogen`). Current archive `tools/html-to-riv/output/baseline-build/debug/build/libhydrogen-sys-22672b1f884c061a/out/libhydrogen.a` has SHA-256 `9b5603db2640894d2a9d093e4eb05c6658bc1cf1095a30331df7f7a8658176ac`. Neither historical manifest separately hashes this native archive or every transitive rlib, so this archive digest is a current observation, not a historical byte-identity assertion.

The baseline records full rustc -vV; the snapped directory records only its first version line, which matches exactly. This verifies the common recorded release identifier, not an independently recorded LLVM/host identity for the snapped build.

```text
rustc 1.97.1 (8bab26f4f 2026-07-14) (Homebrew)
binary: rustc
commit-hash: 8bab26f4f68e0e26f0bb7960be334d5b520ea452
commit-date: 2026-07-14
host: aarch64-apple-darwin
release: 1.97.1
LLVM version: 22.1.8
```

## Independently checked paths and SHA-256

Paths are relative to `/Users/levi/.codex/worktrees/html-css-immutable`.

- `tools/html-to-riv/output/immutable-baseline-toolchain-r2/baseline-probe`: `7b17829dfdea44709faaff688ad146e659862d66bf284dd9aa4b17696839fe6a`.
- `tools/html-to-riv/output/immutable-baseline-toolchain-r2/baseline-probe.rs`: `91adadd3c6d2408d038eac082825519513a1fd506045397bc62028f3a6efdda8`.
- `tools/html-to-riv/output/immutable-baseline-toolchain-r2/build-baseline.py`: `60fe9339424564e168dc3a4052de102a80fb7c8d2b1bc8eb110ab9e0197997f6`.
- `tools/html-to-riv/output/immutable-baseline-toolchain-r2/build-command.json`: `cf7b99fdf24e494c24361378404c446c04370e4550275ed561fcf2d19eb4c850`.
- `tools/html-to-riv/output/immutable-baseline-toolchain-r2/build-messages.jsonl`: `19be9f2d0e74e460ea4af42461f9c6b296fbdc4a2a9c6251053c7538adfcc0c8`.
- `tools/html-to-riv/output/immutable-baseline-toolchain-r2/build.log`: `9ae36bb22d8f0f810a77044760863f70b81405396a32cce477ec08eecd4e6d59`.
- `tools/html-to-riv/output/immutable-baseline-toolchain-r2/dependency-command.json`: `de83cf4b5404a6920f552c6b74cfb6dca59f779c74aaaf93622fe05334f1297a`.
- `tools/html-to-riv/output/immutable-baseline-toolchain-r2/dependency-features.txt`: `c79a79d77e173ff4427e1bbb0bae61a5567c958d7bca418a7eb6727afd4bdbf2`.
- `tools/html-to-riv/output/immutable-baseline-toolchain-r2/probe-build.log`: `e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855`.
- `tools/html-to-riv/output/immutable-baseline-toolchain-r2/probe-command.json`: `36cdb556ea1d8dbd02e48781f6821540c1438d414c31c9e6f299fcdf24f3780b`.
- `tools/html-to-riv/output/immutable-baseline-toolchain-r2/renderer-replay`: `276a49d810c05de7cfb0f0a4109853d12216da64e5d9d8e131f7400357f8e51f`.
- `tools/html-to-riv/output/immutable-baseline-toolchain-r2/rustc-version.txt`: `eaf195daced7a434718b67881bed97c02440b2b6196dad18db476478c66b1c36`.
- `tools/html-to-riv/output/immutable-baseline-toolchain-r2/source-identity.json`: `516deee91beb6117c5f2734245e5ea452cbfb5aa9a3e5949820464023d31786a`.
- `tools/html-to-riv/output/wrapped-snapped-gate-r1/node-probe`: `2831265baa244ae03a26389fd82daaaa79eb88b77f542f6e8bc617343ca15b53`.
- `tools/html-to-riv/output/wrapped-snapped-gate-r1/node-probe.rs`: `d1ddace846419faf1a01051ad2bc38f0d7699b7caf135bc56a41b86a28a58ffc`.
- `tools/html-to-riv/output/wrapped-snapped-gate-r1/probe-command.json`: `b93122644af9bac3d69cdfb6d9c1cf3d2c6f27149aca506977e5a4ec81a01ac7`.
- `tools/html-to-riv/output/immutable-baseline-toolchain-r2/manifest.json`: `84ca614216439463361dd188b4af1b424fbdad99f30c5f05a4267b0ca25b3d72`.
- `tools/html-to-riv/output/wrapped-snapped-gate-r1/experiment-manifest.json`: `faa62dd5d6268e8b20b213886d589925e7b2e1d6a550b5bf128cf8fdde220611`.
- `tools/html-to-riv/output/playwright/public-wrapping-r2/receipt.json`: `994dee882ad8ab8efab4211c14b41ee0138825332142fbc98ac1f0999ff5b118`.
- `tools/html-to-riv/output/wrapped-snapped-gate-r1/rustc-version.txt`: `4cc234933847a36bed75a45855f5f774dffefa91f46f27571e2be8af23f5adf0`.
- `tools/html-to-riv/output/public-wrapping-product-build-r1/frozen/html-to-riv`: `2cae98f4fbfb31cb9efe98feef9403255bfdb9155d36af509fc03bcdec8e673a`.
- `Cargo.toml`: `6eae874f0f25fdffe224dad17032f269c40a2f414f8a175cf88008088f49fe5e`.
- `Cargo.lock`: `2654da7218ed71d6c84d21157cddea237b278d401003ac77db5bd8376815467e`.
- `tools/html-to-riv/output/baseline-build/debug/deps/libnuxie_runtime-a47d7ecb86bb59ae.rlib`: `221771c957c549b5fa73b5788028a4f2ea7c8b7e6306d5102d940f75b4b429c6`.
- `tools/html-to-riv/output/baseline-build/debug/deps/libnuxie_render_api-1c540122ba14263b.rlib`: `f34089cf9dcc88c44cb1a77aa71cfaaaf5c028ac2724b55e8b1597822a56b8d3`.

## Limits

This read-only audit compares actual files with preserved source/build records; it is not a fresh reproducible build or a cryptographic source-to-binary proof. Historical evidence does not attest every ambient variable, SDK/framework or transitive native input. No rebuild, capture, timing run, compiler edit or runtime edit was performed.

The inherited driver receipt retains stale `passed-private-derived-baseline` and private-constructor scope labels. The actual verified tools.compiler is the frozen public `html-to-riv` CLI in `public-wrapping-product-build-r1`; no adapter is used in this campaign. Its hash is verified here against the capture; product source/build/test qualification remains a separate record.

The capture reports Chrome 153.0.8010.12, rust-metal and effective RasterOrdering. The clockwise-atomic CLI token does not establish forced winding or MSAA. This audit does not certify command-stream grammar: the private observer rejected the public graph shape. Visual equivalence is separately reviewed, and full CSS coverage and performance are outside this identity audit.
