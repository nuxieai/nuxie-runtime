# Runtime Feather Paint Payload Contract

Date: 2026-06-29

This document continues roadmap `#10` after
`runtime-gradient-paint-payload-contract.md`. It exposes imported `Feather`
scalar state on headless shape-paint commands without implementing feather
rendering, effect path mutation, or inner-path rebuilding.

## Formal Goal

Expose C++-matching feather scalar payloads for shape-owned `Fill` and `Stroke`
commands whose paint has an attached `Feather`.

The slice is complete when:

- `nuxie-graph::ShapePaintNode` carries a structured `FeatherNode` for attached
  `Feather` objects.
- `nuxie-runtime::RuntimeShapePaintCommand` exposes the attached feather's
  `spaceValue`, `strength`, `offsetX`, `offsetY`, and `inner` fields.
- `tools/cpp-probe` emits the same feather payload beside C++ shape-paint draw
  commands.
- C++ probe-backed coverage proves Rust matches C++ for non-default feather
  scalar values.

## Scope Lock

This slice covers only:

- static imported `Feather` scalar fields attached to a `ShapePaint`;
- payload data visible before renderer paint allocation;
- command-stream parity for values C++ can expose without a renderer factory.

It does not implement feather strength application to renderer paint objects,
world/local renderer translations for feather offsets, inner-path rebuilding,
clip-path rendering, effect-path interaction, animated feather dirt propagation,
renderer paint allocation, or GPU work.

## Admission Rule

Before extending this payload, answer:

1. Is the value an imported scalar field on C++ `Feather`?
2. Is the value available immediately from the attached `ShapePaint::feather()`?
3. Can it be compared through the C++ draw-command probe without allocating a
   renderer paint or path object?

If not, defer it to a later feather rendering, inner-path, effect, animation
dirt, or renderer slice.

## Verification

Focused verification:

```sh
make cpp-probe
RIVE_CPP_PROBE=/Users/levi/dev/rive-rust/tools/cpp-probe/build/macosx/bin/debug/rive_cpp_probe \
  cargo test -p nuxie-runtime --test cpp_probe runtime_draw_command_stream_exposes_feather_paint_payloads_like_cpp_probe -- --nocapture
```

Full verification:

```sh
cargo check --workspace
make test
make cpp-compare
```
