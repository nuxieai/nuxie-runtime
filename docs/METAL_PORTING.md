# Native Metal maintenance guide

The native Metal port is complete. Use [PARITY_WORKFLOW.md](PARITY_WORKFLOW.md)
for mechanical source translation and the two separate adversarial passes.
This guide preserves Metal-specific invariants, not campaign stages or a
second completion system.

The source authority is Rive at
`4ac7b32798da0482e441ef09304dc3b480ed3ee5`, unless an explicitly reviewed
upstream sync advances that pin. Compare the actual upstream pair with its
current Rust owner; older Rust implementations are not behavioral authority.

## Separate platform responsibilities

The built-in renderer's Metal adapter owns capabilities, native resources,
pipeline selection, encoding, submission, completion, presentation, and
readback. Its upstream authorities include `RenderContextImpl`,
`RenderContextHelperImpl`, and `RenderContextMetalImpl`.

The ORE Metal adapter owns authored GPU buffers, textures, samplers, bindings,
pipelines, passes, and frame state. Its authorities are `rive::ore::Context`
and `rive::ore::ContextMetal`. Keep ORE in its separate optional crate so
scripting-disabled products need not link its shader/reflection machinery.
Where the adapters share device and queue access, preserve the exact native
identity; do not create a second Metal service.

Similar type names do not justify merging the two ownership models. Preserve
each upstream owner's ordering, failure, lifetime, and resource-reuse rules.

## Translation and native ownership

- Keep upstream methods, state, defaults, enum values, and branches visible.
  Preserve capability and fallback decision order, allocation timing, storage
  modes, alignment, buffer growth, frame serials, barriers, render-pass breaks,
  and completion behavior.
- Translate Objective-C ownership deliberately: owning fields retain objects;
  borrowed parameters remain scoped borrows. Completion handlers retain exactly
  the owners upstream keeps alive. State native invariants beside unsafe code.
- Preserve release order, including C++ reverse member destruction where
  observable. A clone of an owner is not a new native resource occurrence.
  Native weak-reference tests must drain the appropriate autorelease pool
  before interpreting release observations.
- Do not reuse buffers, textures, target generations, or command state while
  upstream considers them in flight. Abandonment and failed submission must
  release the same owned graph without leaking a completion reservation.
- Keep source-defined resource cache keys, identity, lazy creation, and failure
  behavior. Equal pixels cannot prove matching allocation, upload, or barrier
  behavior.
- Reapply source-required dynamic encoder state after render-pass breaks;
  replacement encoders do not inherit scissor or binding state automatically.
- Preserve shader specialization and source-defined compatible fallback
  ordering. A source-defined Metal fallback is distinct from an invented
  fallback to another renderer.
- Keep caller-owned surface configuration, drawable acquisition, and scheduling
  at the product boundary. Preserve the upstream command-queue ordering and
  drawable lifetime through completion.
- Match recoverable errors and enforced invariants to the pinned owner.
  Never swallow native errors, substitute CPU storage/rendering, return
  success from an unimplemented body, or recover through legacy WGPU.

A Rust adaptation must name its exact preserved invariant in the implementation
and tests. Do not apply transactional publication, deep copying, cache sharing,
or stronger ownership merely because an earlier implementation did: verify
that the current source and approved boundary require it.

## Authored ORE shader boundary

The Apple authored lane requires authenticated exact target-2 MSL plus the
matching target-10 BindingMap and versioned supplemental reflection. Preserve
entry interfaces, uniform sizes, binding groups and slots, texture/sampler
classes, workgroup limits, and vertex maps.

Each source-required shader lookup creates a fresh physical module and correct
occurrence identity. A Metal validation or execution failure must not be
repaired by selecting target-0 WGSL, sharing a different physical module, or
substituting CPU rendering. Authentication and resource-limit enforcement are
host boundary adaptations, not permission to alter ORE execution order.

Encoded-image decoding to RGBA is a Factory-boundary adaptation; do not use it
to omit the raw Metal owner's source-defined format, block-layout, upload,
mipmap, or adopted-texture behavior.

## Validation and product constraints

[METAL_RENDERER_VALIDATION.md](METAL_RENDERER_VALIDATION.md) lists the retained
manual validation entry points. The primary oracle is pinned C++ native Metal
on the same adapter and inputs. Dawn/WebGPU output is never relabeled Metal.
Native Metal's source-defined raster-order and atomic modes do not imply
support for WebGPU-style MSAA.

The [Apple release contract](nux-capi-apple-release.md) governs current package,
deployment, provenance, and dependency requirements. Final Apple products must
not gain WGPU, runtime Naga, Dawn, hidden WGSL translation, or a CPU fallback
through maintenance work.

An upstream pin update must inspect semantic, lifetime, shader, and capability
changes together and refresh reference provenance. Do not silently advance the
oracle during a parity correction.

The completed campaign's stage logs and ledgers are available in Git history.
Its enduring process lessons remain in
[the Metal postmortem](METAL_RENDERER_PORT_POSTMORTEM.md).
