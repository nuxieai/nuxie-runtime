# Vulkan, WebGPU, and WebGL2 phase 0 audit

Date: 2026-08-22

Pinned upstream: `4ac7b32798da0482e441ef09304dc3b480ed3ee5`

Result: **GREEN for audit; preparation remains RED**

## Scope discovered

The generated source-candidate inventory contains 200 unique pinned files:

| Campaign | Candidate files | Current local disposition |
| --- | ---: | --- |
| Vulkan renderer and ORE Vulkan | 40 | No concrete Rust Vulkan backend exists |
| WebGPU renderer, Wagyu browser port, and ORE WGPU | 32 | Build a new exact port; retain the legacy Rust-WGPU implementation only until all three ports close |
| GL/WebGL2 renderer and ORE GL | 41 | Build an exact concrete port and add explicit web-editor selection |
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

The WebGPU campaign is a new exhaustive source-owner translation. The existing
backend is diagnostic evidence, not a translation target or a source of
authority. It is deleted during product cutover after all three ports close.

### WebGL2

The live WebGL2/FemtoVG backend, public API, dependencies, and fallback were
previously removed in favor of WebGPU. That describes the starting state only.
This campaign restores WebGL2 by exact source-owner translation and adds an
explicit WebGPU/WebGL2 choice to the web editor. It does not infer an automatic
failure fallback policy.

## Authority decisions

- All campaigns use the same pinned Rive revision as the closed Metal port.
- Pinned C++ Vulkan is Vulkan's primary oracle.
- Pinned C++ Dawn WebGPU is WebGPU's primary oracle.
- Pinned C++ WebGL2 in the same browser/GPU is WebGL2's primary oracle.
- Other Rust or C++ backends are diagnostic only.
- Product cutover follows frozen closeout: root the new WebGPU and WebGL2
  backends, add explicit editor selection, and delete the legacy Rust-WGPU
  implementation and its exclusive dependencies.
- No common backend HAL will be designed during translation.
- The `implement` and `tdd` skills are actively ignored for the campaign.

## Audit evidence

```text
make backend-port-source-inventory-check
backend source inventory clean: 200 rows
```

## Preparation blockers

At the audit checkpoint all 200 candidates were deliberately unclassified.
The subsequent ownership ledger now classifies every row; preparation stays
red until the remaining independently derived ledgers are complete:

1. semantic versus nonsemantic source dispositions;
2. complete ownership units and exclusive Rust target sets;
3. shared generic dependency and include/import closure;
4. state-bearing field and lifetime ownership;
5. configuration, extension, feature, and browser/platform branches;
6. generated input, command, tool version, output, and digest authority;
7. oracle corpus, exclusions, adapter identity, and hardware matrix;
8. rooted artifact and forbidden-route contracts.

No translation work is admitted before those blockers close.
