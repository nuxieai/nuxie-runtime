# ProjectData runtime seam

UNIV-1633 moves the ProjectDO converter model, compiler, envelope, evaluator,
and errors into `nuxie-project-data`. The baseline `nuxie-runtime` retains only
the Rive bind-graph integration and a product-neutral external-data contract.

## Decision

Adopt the registry/program/state seam rather than exposing bind-graph internals
to a callback adapter. It is deep enough because the baseline owns one coherent
job: translate between Rive runtime values and an opaque external program. The
product crate owns durable identities, JSON, evaluation rules, and product
errors. Neither side needs to know the other's internal graph or model.

The process registry is empty until a product entry point calls
`nuxie_project_data::install_runtime_adapter`. `nuxie-product` does this before
every file-import or locally-authored-runtime adoption path.

## Prototype results

| Concern | Result |
| --- | --- |
| Lifetime | A registry is process-static and installed idempotently by stable id. A decoded program is held by `Arc` in the file's converter cache. Each bind occurrence owns a boxed state created by that program; cloning an occurrence clones its state rather than sharing mutable state. |
| Allocation | Decode happens once per external script asset in `RuntimeDataBindGraphConverterBuildCache`. Runtime-to-product translation allocates only for owned strings, lists, objects, and value paths; scalar values remain copy-only. List materialization is capped at 10,000 items before allocation. |
| Dispatch/performance | Payload classification scans the small installed-registry list, then decode and conversion each use one trait-object dispatch. Repeated references to one asset share one decoded `Arc`; the focused cache test observes one scripting catalog build and one external program decode for two converter references. |
| Errors | An unclaimed payload remains an ordinary script. Once a registry claims a payload, a decode error produces `Unsupported` and never falls through to Luau. Product evaluator errors cross the seam as opaque strings and become a runtime execution failure at the bind boundary. |
| State retention | `RuntimeExternalDataState` defines clone, clear, and active-state behavior. The adapter test drives a stateful interpolation, clones the in-flight state, verifies identical midpoint output, and verifies clearing deactivates it. A retained-operand runtime test verifies live resolver reads and bind dirt propagation. |

## Boundary guarantees

- `nuxie-runtime` and `nuxie` contain no ProjectData types, envelope magic,
  JSON schema, or product converter vocabulary.
- `nuxie-project-data` depends downward on `nuxie-runtime`; the baseline never
  depends upward on the product crate.
- Pure evaluator tests live with `nuxie-project-data`. Baseline seam tests use a
  deliberately unrelated fake registry and program.
- `serde` and `serde_json` are no longer production dependencies of
  `nuxie-runtime`; they remain dev dependencies for C++ oracle tests.
