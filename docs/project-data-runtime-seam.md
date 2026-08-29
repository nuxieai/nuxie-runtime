# ProjectData runtime seam

Nuxie's authored-data converter model, compiler, envelope, evaluator, and
errors live in the runtime-independent `nuxie-project-data` crate.
“ProjectData” is the internal crate/type namespace for data authored into a
Nuxie project; it is unrelated to ProjectDO persistence or networking.

## Decision

Use the translated `ScriptingVm` boundary. `nuxie-runtime` exposes an opaque
`ScriptProgramAdapter` hook, while the upper-leaf
`nuxie-project-data-scripting` crate is the only module that knows both the
runtime script interface and ProjectData's program model. It recognizes an
authenticated `NUXPCV1` payload, retains it as a `RuntimeScriptProgram`, and
creates a stateful `ScriptInstance`. Returning `None` for an unrelated payload
delegates to the ordinary Luau VM.

There is no process registry and no global installation API. A trusted product
host passes the adapter as an explicit import capability, so the adapter and
renderer Factory have the same File-scoped ownership. The Apple product
extension supplies it through renderer-first
`nux_product_file_import_configured`. Android supplies it only through
`nux_file_import_android_vulkan_with_trusted_wgsl`. Ordinary portable, Metal,
and Android Vulkan imports install neither ProjectData execution nor authored
native-shader authority.

## Implementation results

| Concern | Result |
| --- | --- |
| Lifetime | The import capability owns an `Arc<dyn ScriptProgramAdapter>`. A decoded backend program is retained by `RuntimeScriptProgram`; each scripted occurrence owns its own `ProjectDataConverterState`. No mutable program state is process-global. |
| Allocation | Payload decode occurs during exact script-asset registration. Runtime-to-product translation allocates only when crossing the owned-value boundary; scalar converter values remain direct. |
| Dispatch/performance | One optional adapter dispatch occurs during registration and one backend dispatch occurs for converter execution. Unrelated bytecode goes directly to the configured Luau backend. |
| Errors | An unrelated payload remains an ordinary script. A claimed but invalid `NUXPCV1` payload returns a script-registration error and never falls through to Luau. Evaluator errors cross the seam as `ScriptError`. |
| State retention | The `ScriptInstance` owns interpolation time, inputs, and `ProjectDataConverterState`, matching the translated runtime's per-occurrence script lifecycle. |

## Boundary guarantees

- `nuxie-runtime` and `nuxie` contain no ProjectData types, envelope magic,
  JSON schema, or product converter vocabulary.
- `nuxie-project-data` has no runtime dependency. The upper-leaf
  `nuxie-project-data-scripting` adapter depends on both `nuxie` and
  `nuxie-project-data`; dependencies never point back down from the runtime.
- Pure evaluator tests live with `nuxie-project-data`. Adapter registration,
  payload classification, scalar conversion, and state retention tests live
  with `nuxie-project-data-scripting`.
- Product imports pass the adapter explicitly. There is no registry, hidden
  fallback, or factory-free configured-import path.
- `serde` and `serde_json` are not production dependencies of
  `nuxie-runtime`; JSON remains owned by the ProjectData leaf.
- `serde_json` is absent from the default `nuxie-binary` library closure. Its
  wire-format inspector opts into the non-default `inspect` feature, while
  differential tests retain a dev-only dependency. Native runtime import goes
  directly into source-shaped `File`/`CoreArena` owners.
- `make size-report` can continue to ratchet renderer-only closures against
  accidental ProjectData/JSON dependencies; only the product adapter leaf may
  introduce those crates.
