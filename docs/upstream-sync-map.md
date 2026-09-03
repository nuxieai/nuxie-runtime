# Incremental upstream sync

Maintain the completed C++→Rust port by applying upstream deltas, not by
restarting a whole-runtime migration. [PARITY_WORKFLOW.md](PARITY_WORKFLOW.md)
defines the source-first translation and two separate review passes. This
document replaces the former large-cycle, scored-row, and ratchet workflow.

## Current checkpoint

- LAST_SYNCED_SHA: `e3c5dec2873840d09ee1ea54f78e64e805ca22f7`
- Frozen target: `9d2e7d04d1bd5ee5863c7155d059b1e7b5810148`; newer commits wait.
- The user authorized manual, one-commit-at-a-time work on 2026-08-31.
  The preceding accounted change, upstream's Rive 7.3 layout translation,
  anchor, constraint, scroll virtualization-buffer, and Luau 0.733 update, was ported
  mechanically and reviewed as one source commit. Upstream:
  `2bbd8820878f38398b6ea6e722cb88310588c8e8`. Work:
  [UNIV-1880](https://universe.basis.dev/issue/UNIV-1880).

| Upstream SHA | Applicable translated slices | Work |
| --- | --- | --- |
| `e3c5dec2873840d09ee1ea54f78e64e805ca22f7` | Deferred replay capabilities carried as plain data, recording detached from device ownership, per-frame replay-capability verification, and the directly corresponding device-state test. | [UNIV-2411](https://universe.basis.dev/issue/UNIV-2411) |
| `966499fffe2aadcbcd1fe4388160e4e7d5c0d967` | ORE command and resource wire PODs reordered widest-first with explicit trailing padding, gapless recorded vertex/depth-stencil/bind-group-layout structs, and directly corresponding recording/silver tests. The Emscripten build-only slice is not applicable because the browser renderers target `wasm32-unknown-unknown`. | [UNIV-2411](https://universe.basis.dev/issue/UNIV-2411) |
| `707c4f60f2433b32d34597045b2f43460e6cd8fb` | Layout-stack parent propagation for components and artboard component lists; state-machine key/text input ownership; disposed Lua property reconstruction and view-model-list correction; ORE deferred-session bookkeeping and directly corresponding runtime/renderer tests. | [UNIV-2408](https://universe.basis.dev/issue/UNIV-2408), [UNIV-2409](https://universe.basis.dev/issue/UNIV-2409), [UNIV-2410](https://universe.basis.dev/issue/UNIV-2410), [UNIV-2411](https://universe.basis.dev/issue/UNIV-2411) |
| `1db281b3e82baf850635fd7aa2092920a80b6a2c` | Command-queue view-model clearing and reference confirmations; scripting import-factory routing; IK-over-distance constraint behavior and fixture; shape-paint clipping; triangulation caching and interior-budget control; ORE sampler/anisotropy, deferred shader replay, canvas import, backend shader, and directly corresponding runtime/renderer tests. Android test-host and deployment-harness changes are outside this repository. | [UNIV-2402](https://universe.basis.dev/issue/UNIV-2402), [UNIV-2403](https://universe.basis.dev/issue/UNIV-2403), [UNIV-2404](https://universe.basis.dev/issue/UNIV-2404), [UNIV-2406](https://universe.basis.dev/issue/UNIV-2406), [UNIV-2407](https://universe.basis.dev/issue/UNIV-2407), [UNIV-2907](https://universe.basis.dev/issue/UNIV-2907), [UNIV-2908](https://universe.basis.dev/issue/UNIV-2908) |

- The preceding upstream deferred host/player refactor is a
  source-based **SKIP** for this Rust runtime. Its complete diff only adds or
  changes C++ test/player harness and deferred-host files; it changes no
  runtime/core implementation, schema, fixture, or supported product behavior.
  The renderer additions are test/player-host infrastructure only.
  The existing Rust `DeferredSession`/`DeferredReplayer` surface already covers
  the product-side deferred path, so no speculative shared host abstraction was
  added. Upstream: `11217a528b34966eca3765dc88c4ec0c8417d09c`.
  Work: [UNIV-2906](https://universe.basis.dev/issue/UNIV-2906).
- The preceding applicable port at `2cfa84e8103aeeeff4c2bfee92839ab580521660`
  is complete in [UNIV-1878](https://universe.basis.dev/issue/UNIV-1878).
- Intervening `1de56230e9ea062a2da2e25eee00942eafe3bdb4` and
  `0a8499b87a7d722b982d9c444172cab94d8320f2` need no Rust translation: they
  change upstream coverage and GMS test-host infrastructure, not runtime,
  renderer, format, fixture, or supported product behavior.
- Generated shader provenance remains
  `3ed35ee0ded0d58fb8d380930a156041a4624a2f`: this commit changes no renderer
  source or generated artifact.
- Intervening `8efe18ec7b52a02139844ffe71438c00de13037e` needs no Rust
  translation: Apple products and verification links already explicitly set
  macOS 12. The current upstream oracle source retains its Premake macOS 11
  linker-floor fix. No runtime behavior or future dependency was excluded.
- This source checkpoint is not Adreno/PowerVR hardware qualification. The
  existing MoltenVK C-API content checks fail identically on the previous main
  and this update; follow-up: [UNIV-2875](https://universe.basis.dev/issue/UNIV-2875).
- No new catch-up exclusions have been approved. The established Rust
  adaptations below remain in force. The user authorized merging each reviewed
  manual update as it is completed on 2026-08-31. This is not standing automation
  authorization.
- On 2026-08-31 the user approved the full deferred-rendering redesign in
  `e949498e05483a852c10fbbdad2cd1941c15aebc`: upstream is authoritative,
  including removed APIs. Translate the complete applicable delta, then run
  separate source-equivalence and Rust-integration reviews before validation
  and merge. [UNIV-1678](https://universe.basis.dev/issue/UNIV-1678) records
  the implementation, review, and validation evidence.
  The user also approved migrating the Apple C API caller before merge,
  including API changes where needed: import and replay through an explicit
  deferred session so scripted GPUCanvas follows the new upstream contract.
  Do not restore immediate-context GPUCanvas as a compatibility fallback.

## One upstream commit at a time

1. Fetch upstream metadata and enumerate `LAST_SYNCED_SHA..target` oldest first
   in dependency order. Keep the normal upstream and pinned checkouts untouched;
   use a separate clean candidate worktree at the commit being translated.
2. Inspect the complete diff and surrounding upstream owners, including tests,
   fixtures, definitions, generators, dependencies, and shared backend flags.
   Classify **port**, **skip**, or **mixed** from source, never from the title.
   Check later dependencies before excluding a change. An existing equivalent
   Rust implementation is a skip only after source comparison confirms it.
3. Mechanically translate the applicable delta into the corresponding Rust
   owners. Include the same commit's relevant regression tests, fixtures, schema
   changes, and regenerated assets. Do not retranslate untouched files or add
   compatibility fallbacks. Keep translation separate from review.
4. Perform the source-equivalence adversarial review against that upstream
   commit: defaults, arithmetic, control flow, state, lifetime, and failure paths.
5. Perform a separate Rust-integration adversarial review using the approved
   adaptation boundaries below. Review any semantic corrections from either
   review or validation before considering the change complete.
6. Run affected tests, compilation, and applicable focused differential or
   renderer checks against the matching upstream checkpoint. Do not compare a
   partially updated Rust port against the final catch-up target. Passing corpus
   samples and the structural source-correspondence check are evidence, not
   proof that every changed branch is correct.
7. Commit the reviewed change with a Conventional Commit subject and
   `Upstream-Commit: <full upstream SHA>` trailer. Generally one applicable
   upstream commit becomes one Rust commit; preserve identity when a tightly
   coupled upstream series must land together.

Parallel agents may inspect upcoming commits, translate disjoint owners within
a change, and conduct reviews after translation. Keep one integration writer
and preserve upstream order; do not let parallel work merge dependent changes
out of order. Do not load implement or TDD skills for this workflow.

## Scope decisions

Port fixes and compatible features on supported runtime and renderer surfaces,
even when they lack a corpus signal or appear in an "editor" commit. For mixed
commits, retain applicable shared changes and explain the omitted portion.

Keep Taffy, Rust-native audio/text, luaur scripting, Rust ownership, and both
browser renderers on `wasm32-unknown-unknown`. Do not introduce Yoga, an
Emscripten product, runtime Naga, or legacy WGPU. These boundaries do not excuse
omitting upstream lifecycle or observable behavior around an adapted library.

Ask before adding a substantial new architecture/backend or changing accepted
payload support. Inspect format, bytecode, binding-map, generator, and dependency
changes as producer/consumer contracts; do not update only one side or silently
replace an approved backend. If a relevant change is deferred, keep the last
complete checkpoint or explicitly document the narrower supported scope. Never
describe a deferred behavior as parity with the newer upstream revision.

## Small checkpoint PRs

Several small upstream commits may share a PR, preserving their Git identity.
Run the applicable broader translated tests, Golden/Silver, rendering, lifecycle,
and platform harnesses at that boundary, plus required repository PR checks.
Report exact commands, results, skipped checks, and unavailable hardware honestly.
Do not run the entire platform matrix for every tiny commit or require another
whole-port certification. Record skip reasons and dependencies briefly in the
PR; use Git history and this checkpoint, not new ledgers or certification rows.

Advance LAST_SYNCED_SHA only through a fully accounted prefix. Keep active source,
oracle, and artifact provenance coherent at that checkpoint; preserve fail-closed
identity checks. Rebuild affected reference/generated products as needed.
Current pin locations to inspect together:

- `Makefile`: `RIVE_RUNTIME_REF` and `PERF_EXPECTED_RIVE_RUNTIME_REF`.
- `.github/workflows/{ci,_trusted-macos}.yml` and `.buildkite/pipeline.yml`:
  `RIVE_RUNTIME_REF`.
- `tools/fetch-test-assets.sh`: default revision, not fixed asset provenance.
- `tools/{check-renderer-decoder-provenance,generate-renderer-shaders,renderer-dawn-live-reference-bootstrap}.sh`.
- `tools/golden-runner/runtime-provenance.sh`.
- `tools/silver-corpus/{generate_manifest.py,src/lib.rs}` and `silver-corpus.toml`:
  current inventory reference and new upstream cases; preserve existing result
  classifications unless the translated upstream delta changes their evidence.
- `tools/backend-port/build-{webgpu,webgl2}-source-oracle.sh` and shader
  importers: source revisions and regenerated artifact hashes.

Historical source citations, fixture hashes/revisions, and past oracle evidence
are not current pins. Never blanket-replace old SHAs or relabel stale artifacts.

## Authorization

This guide does not activate or modify scheduled jobs. A scheduled inspection is
read-only unless separately authorized; it must not infer write or merge
permission from this manual update. Follow the repository's PR/merge policy.
