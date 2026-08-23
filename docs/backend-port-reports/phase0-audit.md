# Vulkan, WebGPU, and WebGL2 phase 0 audit

Date: 2026-08-22

Pinned upstream: `4ac7b32798da0482e441ef09304dc3b480ed3ee5`

Result: **GREEN for audit; preparation remains RED**

## Scope discovered

The generated source-candidate inventory contains 200 unique pinned files:

| Campaign | Candidate files | Current local disposition |
| --- | ---: | --- |
| Vulkan renderer and ORE Vulkan | 40 | No concrete Rust Vulkan backend exists |
| WebGPU renderer, Wagyu browser port, and ORE WGPU | 32 | Existing Rust-WGPU implementation requires full source/ownership correspondence review |
| GL/WebGL2 renderer and ORE GL | 41 | Product implementation retired; reference-only port must not restore it |
| Shared shader and backend build authority | 87 | Existing generated WGSL and Metal-era source translations are evidence, not automatic promotion |

The inventory is generated from the configured source roots, exact extra
sources, all top-level GLSL/vertex/fragment inputs, the complete SPIR-V wrapper
directory, shader generators, backend build files, and Dawn/MoltenVK/SwiftShader
bootstrap scripts. Every row binds the pinned source SHA-256.

The existing Metal campaign remains the shared generic renderer authority.
Preparation must still prove the exact subset and dependency edges consumed by
each new backend rather than assuming every Metal dependency is shared.

## Existing implementation state

### Vulkan

There is no local concrete Vulkan renderer or ORE Vulkan crate. Textual Vulkan
mentions in current Rust sources describe generic constants, source comments,
or diagnostic policy; they are not an implementation.

### WebGPU

`nuxie-renderer` currently defaults to `rust-wgpu`. Twenty-nine Rust source
files contain direct WGPU/WebGPU owners or call sites, and 66 generated WGSL
modules are compiled into the shader catalog. Existing same-runner Dawn
evidence is substantial, but it does not establish complete source-owner,
configuration, generated-input, ownership, or failure correspondence.

The WebGPU campaign is therefore an exhaustive correspondence and correction
port, not permission to declare the existing code complete from its current
tests.

### WebGL2

The live WebGL2/FemtoVG backend, public API, dependencies, and fallback were
deliberately removed. The remaining live check rejects their reintroduction.
This campaign may build a source-shaped reference backend and oracle tooling,
but it must remain outside the shipping crate graph and browser selector.

## Authority decisions

- All campaigns use the same pinned Rive revision as the closed Metal port.
- Pinned C++ Vulkan is Vulkan's primary oracle.
- Pinned C++ Dawn WebGPU is WebGPU's primary oracle.
- Pinned C++ WebGL2 in the same browser/GPU is WebGL2's primary oracle.
- Other Rust or C++ backends are diagnostic only.
- Shipping cutover is outside port completion.
- No common backend HAL will be designed during translation.
- The `implement` and `tdd` skills are actively ignored for the campaign.

## Audit evidence

```text
make backend-port-source-inventory-check
backend source inventory clean: 200 rows
```

## Preparation blockers

All 200 candidates remain deliberately `unclassified`. Preparation stays red
until the following independently derived ledgers are complete:

1. semantic versus nonsemantic source dispositions;
2. complete ownership units and exclusive Rust target sets;
3. shared generic dependency and include/import closure;
4. state-bearing field and lifetime ownership;
5. configuration, extension, feature, and browser/platform branches;
6. generated input, command, tool version, output, and digest authority;
7. oracle corpus, exclusions, adapter identity, and hardware matrix;
8. rooted artifact and forbidden-route contracts.

No translation work is admitted before those blockers close.
