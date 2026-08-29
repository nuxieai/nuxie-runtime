# Docs index

The portable C++→Rust port of the Rive runtime is **complete**. Platform
renderer implementations continue as explicit campaigns. This repo now serves
two ongoing workflows, and every document here supports one of them:

1. **Upstream sync** — triage incoming `rive-app/rive-runtime` changes, port
   approved ones to Rust, advance the reference pin with green ratchets.
2. **Nuxie platform work** — features, APIs, and optimizations the upstream
   runtime does not have, while keeping the ported surface accurate and every
   divergence from upstream intentional and recorded.

## The forward set

| Document | What it is |
|---|---|
| `PARITY_WORKFLOW.md` | Reusable upstream-parity execution loop: evidence, work partitioning, model roles, validation, promotion, and progress reporting for Metal, WebGPU, Vulkan, and main-runtime work. |
| `PORTING.md` | The C++→Rust translation manual (idioms, float exactness, invalidation fences, adaptation ceilings). Read before porting any upstream change. |
| `METAL_PORTING.md` | Native Metal renderer-platform and ORE mechanical-port guide: source correspondence, lifetime rules, oracle hierarchy, validation ladder, and product decisions. |
| `METAL_RENDERER_PORT_POSTMORTEM.md` | Process postmortem for the 39-hour native Metal port: false start, whole-owner reset, review findings, closeout, and follow-up actions. |
| `NEXT_RENDERER_PORTS_PLAN.md` | Ordered source-owner campaign for exact Vulkan, WebGPU, and WebGL2 ports, followed by web-editor selection and legacy Rust-WGPU retirement. |
| `backend-port-reports/preparation-status.md` | Live preparation barrier: frozen denominators, exact product intent, and blockers that keep translation closed. |
| `ore-port-lifetimes.tsv` | Field-level C++→Rust ownership ledger for the file-oriented ORE/Metal translation queue. |
| `upstream-sync-map.md` | The recurring Upstream Sync cycle: triage, approval gate, porting order, pin advance. Cycle outputs land in `sync/`. |
| `upstream-test-findings.md` | Open coverage gaps and production divergences found while porting the upstream unit-test suite; each is anchored by an `#[ignore]`d Rust test. |
| `command-queue-test-ledger.md` | Complete CommandQueue/CommandServer upstream-test correspondence, including the S4-45 blob disposition. |
| `pure-runtime-boundary.md` | Ratified ownership and dependency-direction contract separating the ported runtime baseline from product, editor, browser, and Apple layers. |
| `player-scheduling-contract.md` | Product-neutral dirty/settled/render-demand evidence, occurrence-scoped presentation acknowledgement, and optional monotonic wake semantics. |
| `project-data-runtime-seam.md` | Decision and evidence for the product-owned ProjectDO evaluator and the neutral external-data adapter retained by the baseline bind graph. |
| `nux-capi-apple-release.md` | The slim Apple binary contract: sole nux-capi root, dual packages, provenance, size deltas, and immutable publication. |
| `side-channel-format.md` | Wire format of the golden-stream runtime side channel (settled bool, hit results, events, semantics). Implemented by both golden runners. |
| `SIZE.md` | The blocking 9 MiB SDK size budget and its measurement method. |
| `renderer-parity-workflow.md` | Acceptance contract for renderer performance parity (1.0x threshold, fixed report matrix). |
| `renderer-exactness-map.md` | Contract-exact vs byte-exact renderer corpus metrics and the same-runner gate. |
| `renderer-fuzz-replay.md` | The dual-renderer negative-input gate. |
| `audio-core-parity.md` | The audio parity boundary (headless engine exact; no device sink). |
| `browser-renderer-wasm-packaging.md` | Current browser packaging decision: separate exact WebGPU and WebGL2 products, both on `wasm32-unknown-unknown`, with explicit editor selection and no automatic fallback. |
| `webgpu-only-browser-cut.md` | Historical WebGPU-only product cut, superseded by the exact WebGL2 port and explicit editor renderer selection. |
| `luau-fork.md` | State of the in-house `luaur` Luau engine fork and its carried patches. |
| `watch-cpp-nest-semantic-uaf.md` | Live upstream heap-use-after-free (register row W3) + candidate upstream patch. |
The runtime source port follows `runtime-bun-style-source-port-plan.md` and
does not preserve the superseded runtime parity or test-correspondence ledger
systems. Renderer campaigns keep their own machine-checked manifests,
including `metal-port-manifest.toml` and `metal-port-ownership.toml`. Corpus,
golden, silver, performance, and fuzz manifests remain live validation inputs.

`evidence/` holds dated measurement records that back still-open register rows
or live budgets. `sync/` holds one triage report per completed sync cycle.

## Glossary of legacy identifiers

The port was executed in phases with codenames. Superseded phase documents are
recoverable from git history; identifiers that remain in source comments or
historical evidence are citations, not live workflow state.
Do not renumber or reuse them.

- **V2** — the second (successful) porting attempt; "the original port." V1's
  map and V2's working logs are in git history (`docs/porting-map.md`,
  `docs/porting-map-v2.md`, `docs/v2-status.md`, `docs/v2-log-archive.md`).
  `corpus.toml` is still described in places as "the V2 ratchet."
- **M0–M8** — the original port's milestone sequence.
- **Phase R / R0–R5** — the renderer port and its sub-phases. Its audit trail
  is in git history (`docs/renderer-status.md`, `docs/renderer-port-map.md`,
  `docs/renderer-r3-*`, `docs/renderer-r4-*`).
- **RD-1 / RF-nn, FL-A…FL-E / FLR-nn / FL-Gnn, B6-NNNN** — historical
  runtime-port and structural-audit identifiers. Their receipts are available
  from git history; current translation rules live in `PORTING.md`.
- **S\<cycle\>-\<n\>** (e.g. S4-45) — Upstream Sync cycle row ids, defined in
  `upstream-sync-map.md`. Stable across workflow renames.
- **V/F/A/C/D/H/W rows** — historical gap-register row families.
- **P1/P2/P3, W\<n\>, VFIX, OR-n, LT-1, RB-n** — completed work-item codenames;
  they appear only inside historical evidence/note strings and git history.
- **silver** — upstream's own serialized-render test facility
  (`SerializingFactory`, `.sriv`); not a migration term.
- **golden gate** — the golden-stream comparison gates (`make golden-compare`,
  `make scripted-golden-compare`); descriptive, not a codename.

Ledger `evidence`/`note`/`audit_record` strings may cite work-item documents
(e.g. `LT1SC-report.md`) that were deleted in the cleanup; they remain valid
citations into git history.
