# Phase 2 global source-semantics review

Date: 2026-08-21

Pinned upstream: `4ac7b32798da0482e441ef09304dc3b480ed3ee5`

Result: **RED**

This is the first adversarial pass after the 98-file bulk translation barrier.
Three independent Sol review contexts inspected disjoint manifest partitions
against the pinned upstream source. They made no edits and did not use compiler
results or fixtures to choose work.

## Exact coverage

| Partition | Manifest ordinals | Source→target pairs | Result |
| --- | ---: | ---: | --- |
| ORE, GPU resource, and the two interleaved generic foundations | 1–14 | 34 | red |
| Generic renderer dependencies and implementation | 15–31 | 20 | red |
| Shader/build authority and native Metal owners | 32–35 | 44 | red |
| **Total** | **1–35** | **98** | **red** |

The 35 manifest units are receipt/review/compiler groups. The primary
translation and review denominator remains the 98 individual pinned files.

## Raw finding inventory

The partition reports contain 28 raw findings: 2 P0, 12 P1, 11 P2, and 3 P3.
Correction planning may merge findings that share one source-owned cause, but
may not drop their individual observable behaviors.

### P0

1. `renderer/src/render_context.cpp:132-4013` is represented by a target whose
   executable content is only crate attributes; the implementation remains
   comments in `renderer/src/render_context_cpp.rs:5-18`.
2. `renderer/src/metal/render_context_metal_impl.mm:45-2029` is entirely inert
   comment text in `renderer/src/metal/render_context_metal_impl_mm.rs:27-2058`.

### P1

1. `renderer/src/shaders/Makefile` is stored as provenance, not translated
   build behavior; minification, seven Apple SDK families, metallib linking,
   missing-output recovery, and C-array emission are not executable.
2. `minify.py` passes PLY negative-lookahead expressions to Rust `regex`, whose
   engine rejects look-around; the translated lexer cannot process an input.
3. The background compiler header and implementation define disconnected Rust
   owner types rather than one declaration/definition pair.
4. Native Metal platform/tool branches use nonexistent Cargo features, making
   iOS, tools, canvas, and tracking configurations unreachable or incorrect.
5. Generic buffer-ring wrappers require a hooks argument while all nine helper
   call paths preserve the source one-argument virtual-dispatch form.
6. Shared renderer types are replaced by local empty/uninhabited placeholders,
   including an uninhabited `DitherMode` in `FlushDescriptor`.
7. One upstream tools macro is split across incompatible Rust feature names;
   additional canvas/feather predicates also name nonexistent features.
8. Executable inline `RenderContext` accessors, allocator construction, atlas
   throttling, and barrier publication remain comments or bodyless traits.
9. `Font::shapeText` names untranslated dependency types and cannot form the
   pinned shaping/whitespace-break path.
10. `Texture::nativeHandle` is hard-wired to the base type and loses backend
    virtual dispatch.
11. ORE base `Context`, `RenderPass`, `Buffer`, and `Texture` polymorphic APIs
    are comments rather than a callable Rust dispatch surface.
12. Both `RIVE_OBJC_EXCEPTIONS` recovery branches are omitted, changing native
    exception recovery and `outError` publication.

### P2

1. Native render-target inline texture access and lazy atomic-buffer allocation
   are abstract signatures without source bodies.
2. Background compilation iterates ten feature bits instead of the pinned
   eight and invents macros for high bits.
3. `FlushUniforms` accepts independent dimensions instead of deriving them from
   `flushDesc.renderTarget`, changing consistency and failure order.
4. Sampler-key decoding transmutes arbitrary bytes into a two-variant Rust enum,
   introducing invalid-enum undefined behavior.
5. Texture hash wraparound panics at zero instead of preserving `uint32_t`
   modular behavior.
6. `ClipRectInverseMatrix::default()` becomes the empty zero matrix instead of
   the source identity matrix.
7. Multiple source release-only assertions became unconditional Rust panics.
8. `RenderPassMetal` move operations transfer base state the source leaves
   defaulted or unchanged.
9. BC1/BC3/BC7 mapping rejects supported iOS 16.4+ systems.
10. `TRACK_RIVE_SHADER_ID` is coupled to the unrelated `tools` feature.
11. ORE resource/update/render-pass assertions also became unconditional
    production panics rather than source debug assertions.

### P3

1. ORE descriptor pointer/count and pointer/byte-size pairs were collapsed into
   slices, eliminating valid prefix and explicit-size source states.
2. `drawIndexed` widens before multiplication instead of preserving authored
   32-bit offset wraparound.
3. RSTB encoders reject oversized lengths where the source truncates the wire
   length and still emits all bytes.

## Positive evidence

- All 37 GLSL, vertex, fragment, and Metal shader-source constants were
  independently byte-compared with the pinned checkout and matched exactly.
- The four tiny ORE Metal `.mm` files for sampler, shader module, pipeline, and
  bind group are genuinely empty upstream; their no-op targets are not defects.
- The campaign checker independently accounts for 98 sources, 455 fields,
  82 legacy configuration blocks, 631 exhaustive preprocessor blocks with 842
  branches, 305 include/import occurrences, and 298 dependency edges.

## Phase disposition

Phase 2 is complete but red. No finding is accepted as an intentional
divergence merely because the Rust behavior is safer or more idiomatic. The
complete 98-file tree advances to the independent ownership/lifetime/ABI pass
before owner-bounded corrections begin.

![Dashboard at the Phase 3 ownership-review barrier](../metal-port-images/phase3-ownership-review-summary.png)
