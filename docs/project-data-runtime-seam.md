# ProjectData runtime seam

UNIV-1633 moves Nuxie's authored-data converter model, compiler, envelope,
evaluator, and errors into `nuxie-project-data`. The baseline `nuxie-runtime`
retains only the Rive bind-graph integration and a product-neutral external-data
contract. “ProjectData” is the internal crate/type namespace for data authored
into a Nuxie project; it is unrelated to ProjectDO persistence or networking.

## Decision

Adopt the registry/program/state seam rather than exposing bind-graph internals
to a callback adapter. It is deep enough because the baseline owns one coherent
job: translate between Rive runtime values and an opaque external program. The
product crate owns durable identities, JSON, evaluation rules, and product
errors. Neither side needs to know the other's internal graph or model.

The process registry is empty until an authoring/product consumer calls
`nuxie_project_data::install_runtime_adapter`. The Apple distribution's
upper-leaf `nux-apple-product-extension` installs it only through the explicit
`nux_product_file_import_configured` entrypoint before delegating to baseline
configured import. The Android distribution has no equivalent upper-leaf
archive and ships `nux-capi`'s portable configured-import symbols directly, so
the exact `android-vulkan` + `scripting` build installs the adapter at
`nux_file_import_configured`; `nux_file_import_with_assets` reaches the same
seam by delegation. Other `nux-capi` feature combinations do not install it.

## Prototype results

| Concern | Result |
| --- | --- |
| Lifetime | A registry is process-static and installed idempotently by stable id. A decoded program is held by `Arc` in the file's converter cache. Each bind occurrence owns a boxed state created by that program; cloning an occurrence clones its state rather than sharing mutable state. |
| Allocation | Decode happens once per external script asset in `RuntimeDataBindGraphConverterBuildCache`. Runtime-to-product translation allocates only for owned strings, lists, objects, and value paths; scalar values remain copy-only. List materialization is capped at 10,000 items before allocation. |
| Dispatch/performance | Payload classification scans the small installed-registry list, then decode and conversion each use one trait-object dispatch. Repeated references to one asset share one decoded `Arc`; the focused cache test observes one scripting catalog build and one external program decode for two converter references. |
| Errors | An unclaimed payload remains an ordinary script. Once a registry claims a payload, a decode error produces `Unsupported` and never falls through to Luau. Product evaluator errors cross the seam as opaque strings and become a runtime execution failure at the bind boundary. |
| State retention | `RuntimeExternalDataState` defines clone, clear, and active-state behavior. Project-data adapter tests drive stateful interpolation and idempotent registration, while retained-operand runtime tests separately verify live resolver reads and bind dirt propagation. |

## Boundary guarantees

- `nuxie-runtime` and `nuxie` contain no ProjectData types, envelope magic,
  JSON schema, or product converter vocabulary.
- `nuxie-project-data` depends downward on `nuxie-runtime`; the baseline never
  depends upward on the product crate. The sole distribution-root exception is
  an optional `nux-capi` dependency activated only by `android-vulkan`, with
  adapter installation compiled only at the `android-vulkan` + `scripting`
  intersection.
- Pure evaluator tests live with `nuxie-project-data`. Baseline seam tests use a
  deliberately unrelated fake registry and program.
- Project-data tests own adapter registration and payload decode; baseline seam
  tests own output application and retained state through an unrelated fake.
- `serde` and `serde_json` are no longer production dependencies of
  `nuxie-runtime`; they remain dev dependencies for C++ oracle tests.
- `serde_json` is absent from the default `nuxie-binary` library closure. Its
  wire-format inspector opts into the non-default `inspect` feature, while
  differential tests retain a dev-only dependency. The native runtime imports
  directly into source-shaped `File`/`CoreArena` owners; the legacy graph
  projection is not part of the product architecture or dependency closure.
- `make size-report` ratchets the renderer-on, scripting-off SDK dependency
  closure against `nuxie-project-data`, `serde_json`, and `zmij` regressions.
