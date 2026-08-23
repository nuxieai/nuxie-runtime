# Backend global source-review evidence

This document is the receipt contract enforced by
`tools/backend-port/check_source_review.py`. The checker derives authority from
the tracked campaign manifest, source-review plan, frozen source-owner and SCC
order ledgers, pinned upstream tree, all translation receipts, physical pinned
external dependencies, retained generated outputs, and the source-review
support inventory. Receipts attest to that authority; they do not redefine it.

The source pass is independent, read-only, and SCC-atomic. It actively ignores
the `implement` and `tdd` skills. Reviewers do not select work from compiler
diagnostics, tests, fixtures, or feature behavior, and they do not correct a
finding before the later global ownership/lifetime/ABI pass finishes.

The plan identity is exact:

```toml
review_kind = "global-source-semantics"
review_mode = "independent-read-only-scc-waves"
receipt_directory = "docs/backend-port-source-reviews"
severity_order = ["P0", "P1", "P2", "P3"]
finding_id_rule = "SR-C<component number>-<two-digit nonzero ordinal>"
```

The product boundary is also frozen: Vulkan, WebGPU, and WebGL2 are exact
ported renderers; WebGPU and WebGL2 become explicit editor choices with no
automatic fallback; and legacy Rust-WGPU remains until each of the three ports
independently passes frozen closeout, after which it is deleted. Source review
does not jump ahead to editor wiring, browser execution, closeout, or deletion.

## Exact evidence set

The complete global evidence set is exactly 117 tracked files under
`docs/backend-port-source-reviews`, all immediate canonical
`*.source-review.toml` receipts. A nested or differently named file is extra
evidence and fails the global set check.

- 115 canonical SCC component receipts, one for every exact `component_id` in
  the frozen SCC ledger;
- one `support.source-review.toml`, covering all 52 support-inventory files;
- one `overlays.source-review.toml`, containing all nine cross-seam overlays in
  their fixed order.

The 115 component receipts cover the union of all 135 ownership units and all
200 pinned source owners. They bind 188 translated source-to-target pairs and
independently revalidate 12 nontranslated source dispositions. Four components
are exclusion-only. A nontranslated source still receives full source evidence
but receives no target record.

The frozen source denominator is 55,916 logical lines and 2,277,054 bytes:

| Class | Sources | Logical lines | Bytes |
| --- | ---: | ---: | ---: |
| Translated | 188 | 55,125 | 2,246,991 |
| Nontranslated | 12 | 791 | 30,063 |
| Total | 200 | 55,916 | 2,277,054 |

A logical-line count is `len(file_bytes.splitlines())`. This counts a nonempty
unterminated final line, including the three pinned upstream files without a
trailing newline.

The coverage array is exact and ordered:

```toml
coverage = [
  "owned-source-lines",
  "translated-target-lines",
  "declarations",
  "conditionals",
  "include-owners",
  "source-semantics",
  "pinned-build-exclusions",
]
```

The overlay receipt uses the same array with one final item:

```toml
coverage = [
  "owned-source-lines",
  "translated-target-lines",
  "declarations",
  "conditionals",
  "include-owners",
  "source-semantics",
  "pinned-build-exclusions",
  "cross-backend-overlays",
]
```

## Admission replay

Every checker mode performs the complete authority replay before it reads any
source-review receipt. It verifies all of the following:

1. The campaign manifest has its exact declared top-level key set, and the
   campaign and plan are tracked. The active queue is `source-review` or a
   later queue, `source_review_status` is `active` or `complete`, all three
   backend translations and every shared-source translation are complete, the
   ignored-skill and product-cutover contracts are exact, and the upstream
   checkout is exactly
   `4ac7b32798da0482e441ef09304dc3b480ed3ee5`.
   A queue later than `source-review` requires the status to be `complete`.
2. The plan workspace base
   `5967e72706cf63702ecbc84a982e4172b8d6245f` is a full ancestor revision. Its
   seven waves, denominators, rules, nine overlays, and changed-byte closure
   must rederive exactly. Every pre-existing campaign authority file, all
   translation receipts, translated targets, source snapshots, dependency
   files and trees, and all 52 support artifacts must remain byte-for-byte
   clean against that revision. Updating a target and its receipt together
   cannot redefine review authority after launch.
   The post-base plan, schema, support-inventory, receipt-directory, and manifest
   path identities are literal checker constants, and the complete plan, schema,
   and support-inventory bytes are bound by launch SHA-256 constants.
3. The ownership and SCC-order ledgers rederive 200 unique source owners, 135
   units, and 115 components. Every pinned source path and SHA-256 is replayed
   against the current upstream bytes, and no SCC spans waves.
4. The dependency, configuration, and generated ledgers rederive 900 semantic
   dependency rows, 5,386 semantic configuration rows, 520 generated artifact
   rows, and 438 semantic generated-owner edges. All 35 distinct
   `pinned-source-external` paths are replayed against 271,592 exact upstream
   bytes and 7,957 logical lines. All 514 retained generated outputs are replayed
   against 9,459,764 exact upstream bytes and 114,637 logical lines; the six
   `ephemeral-final-header-retained` intermediates must be absent and each must
   name a retained final header.
5. Every tracked translation receipt is replayed. The resulting closure must
   be exactly 188 translations and 188 unique source snapshots. Each receipt
   must bind the frozen owner, campaign, unit, complete-source-owner kind,
   source hash/line/byte count, exclusive current target path and hash, source
   snapshot path and hash, and exact dependency-unit set. Each source snapshot
   is tracked and byte-identical to its pinned source.
6. Translation dependency artifacts rederive exactly 17 tracked files plus four
   tracked directory trees containing 259 unique files. A tree hash is SHA-256
   over each recursively sorted tree-root-relative member path, a NUL byte, its
   complete bytes, and another NUL byte. File and tree hashes are replayed in
   every mode.
7. The support inventory rederives exactly 52 unique, tracked, non-target files
   totaling 97,253 logical lines. Every path, hash, line count, role, overlay,
   source authority, and `review-full-source-semantics` disposition is exact.
8. The nine overlay authorities are rederived from their complete typed
   authority-key sets: components, support files, raw and pair dependency edges,
   generated rows, retained generated-output hashes, generic seams, physical
   pinned-external hashes, dependency-file hashes, dependency-tree hashes, VMA
   call probes, build/configuration predicates, and excluded-source
   classifications.
9. All 813 `ACDMRT` paths changed from campaign admission revision
   `aa25e76acdbe0ad0f4099f8a360937bd74d856f9` through review workspace base
   `5967e72706cf63702ecbc84a982e4172b8d6245f` are classified once, in the
   following first-match order:

| Changed-byte category | Paths |
| --- | ---: |
| `translated_target` | 186 |
| `source_snapshot` | 186 |
| `dependency_tree_member` | 193 |
| `dependency_file` | 5 |
| `source_review_support` | 47 |
| `campaign_documentation` | 192 |
| `campaign_tooling` | 3 |
| `ownership_only_evidence` | 1 |
| `explicit_deletion` | 0 |
| Total | 813 |

The changed-byte counts are a Git-diff closure, not receipt denominators. For
example, all 188 translations, all 259 dependency-tree members, and all 52
support files are replayed even though only 186 targets, 193 tree members, and
47 support files changed inside this particular admission-to-base interval.
No changed path may remain unclassified.

The three explicit `campaign_tooling` paths are `Makefile`,
`tools/backend-port/check_translation.py`, and
`tools/backend-port/import_source_owner_snapshots.py`. The sole
`ownership_only_evidence` path is
`tools/backend-port/extract_field_inventory.py`, and the explicit-deletion set
is empty. All other categories are rederived from the replayed authorities and
the Git diff rather than supplied by a receipt.

## Field and lifecycle boundary

Field-layout and lifecycle inventories belong to the next global
ownership/lifetime/ABI review, not this source-semantics receipt set. The
campaign's 1,946 field rows and 2,431 lifecycle rows are therefore deliberately
absent from component, support, and overlay receipt membership. The field
extractor is classified as the one `ownership_only_evidence` path in the
813-path closure; that classification closes the Git diff but does not admit
the extractor or its output as source-review support evidence.

Similarly, a field or lifecycle document can be classified as campaign
documentation without becoming source-review semantic evidence. Source
reviewers still inspect source declarations and executable behavior required by
the coverage contract, but they do not make field-layout, ownership, retain/
release, destruction-order, or ABI closeout claims in this pass. Findings from
those authorities are issued by the later ownership review.

## Wave and structural-admission order

One receipt covers the complete source and target union of one SCC component.
The 135 ownership units are never split into separate receipts, including a
multi-unit SCC. Component receipts are admitted by the frozen `order_group`:

| Wave | Components | Units | Sources | Translated sources | Excluded sources |
| --- | ---: | ---: | ---: | ---: | ---: |
| `g0` | 45 | 47 | 53 | 51 | 2 |
| `g1` | 20 | 21 | 36 | 35 | 1 |
| `g2` | 36 | 36 | 55 | 50 | 5 |
| `g3` | 7 | 10 | 17 | 15 | 2 |
| `g4` | 5 | 9 | 15 | 13 | 2 |
| `g5` | 1 | 1 | 2 | 2 | 0 |
| `g6` | 1 | 11 | 22 | 22 | 0 |
| Total | 115 | 135 | 200 | 188 | 12 |

Partial admission of a component in `gN` first requires every component in
`g0` through `g(N-1)` to have a structurally valid tracked receipt. Other
components in the same wave are independent and may run in parallel. A red
receipt in a prior wave is structurally valid and therefore admits the next
wave; open findings are preserved for the later correction gate.

After `g6`, partial admission of `support.source-review.toml` requires all 115
component receipts to validate. After support, partial admission of
`overlays.source-review.toml` requires all 115 component receipts and the
support receipt to validate. This gives the only valid structural sequence:

```text
g0 -> g1 -> g2 -> g3 -> g4 -> g5 -> g6 -> support -> overlays -> global
```

No backend-specific subset can advance the global gate.

## Component receipt

The canonical filename is
`docs/backend-port-source-reviews/component-NNN.source-review.toml`, and its
`component_id` must match the filename. The following is the exact TOML shape;
the concrete component shown has one unit and one translated owner:

```toml
schema_version = 1
component_id = "component-097"
units = ["webgl2:renderer:pls_impl_webgl"]
receipt_kind = "source-review-component"
upstream_ref = "4ac7b32798da0482e441ef09304dc3b480ed3ee5"
workspace_base_ref = "5967e72706cf63702ecbc84a982e4172b8d6245f"
role = "sol-high"
review_run_id = "source-review-g4-component-097"
review_wave = "g4"
coverage = [
  "owned-source-lines",
  "translated-target-lines",
  "declarations",
  "conditionals",
  "include-owners",
  "source-semantics",
  "pinned-build-exclusions",
]
findings = []
open_findings = 0

[[sources]]
path = "renderer/src/gl/pls_impl_webgl.cpp"
sha256 = "72332a776d363a5bc81360a3a152f525dd74b7f0456903c69522884a6f268bb7"
citation = "source:renderer/src/gl/pls_impl_webgl.cpp:1-337"
disposition = "translate"

[[targets]]
path = "crates/nuxie-renderer/src/mechanical_port/webgl2/renderer_src_gl_pls_impl_webgl_cpp__impl.rs"
sha256 = "972baaa524b370e777108868eeddbacefe32f5a02ff723a3b3b54fe61c316b15"
citation = "target:crates/nuxie-renderer/src/mechanical_port/webgl2/renderer_src_gl_pls_impl_webgl_cpp__impl.rs:1-1127"
```

Top-level keys are exact; missing or invented keys fail. `units` is the exact
ordered unit list from the SCC ledger. `review_wave` is the component's frozen
`g0`-through-`g6` group. `role` is derived from shared authority and is
currently `sol-high`. `review_run_id` must start with an alphanumeric, contain
at least eight characters total, use only alphanumerics plus `.`, `_`, `:`, or
`-`, and must not contain `placeholder`, case-insensitively.

Each source record has exactly `path`, `sha256`, `citation`, and `disposition`.
Its membership and values must mirror the frozen owner ledger, and its citation
must be the exact full current upstream `source:path:1-N` range. Every source in
the component appears once.

Each target record has exactly `path`, `sha256`, and `citation`. Its membership,
path, and hash come exclusively from the replayed translation receipt, and its
citation is the exact full current workspace `target:path:1-N` range. Every
translated source in the component contributes one target; a nontranslated
source contributes none. Source and target record order is not authoritative,
but membership must be exact and duplicate-free.

For a red component receipt, replace the empty `findings` value, set
`open_findings` to the exact array length, and use this exact finding shape:

```toml
[[findings]]
id = "SR-C097-01"
severity = "P1"
summary = "Describe one concrete, observable source-semantics mismatch"
citations = [
  "source:renderer/src/gl/pls_impl_webgl.cpp:20-35",
  "target:crates/nuxie-renderer/src/mechanical_port/webgl2/renderer_src_gl_pls_impl_webgl_cpp__impl.rs:40-61",
]
```

Component finding IDs are
`SR-C<three-digit component number>-<01 through 99>`. Severity is exactly one
of `P0`, `P1`, `P2`, or `P3`; the summary is nonempty; and citations are
nonempty and duplicate-free. A citation may name only a source or target owned
by this component and must be an in-range
`source:path:first-last` or `target:path:first-last` range.

## Support receipt

The one canonical support receipt is
`docs/backend-port-source-reviews/support.source-review.toml`. Its top-level
shape is exact:

```toml
schema_version = 1
receipt_kind = "source-review-support"
upstream_ref = "4ac7b32798da0482e441ef09304dc3b480ed3ee5"
workspace_base_ref = "5967e72706cf63702ecbc84a982e4172b8d6245f"
role = "sol-high"
review_run_id = "source-review-support-20260823"
review_wave = "support"
coverage = [
  "owned-source-lines",
  "translated-target-lines",
  "declarations",
  "conditionals",
  "include-owners",
  "source-semantics",
  "pinned-build-exclusions",
]
findings = []
open_findings = 0

[[artifacts]]
path = "Cargo.lock"
sha256 = "5f525f173c78e3dae0525712574269d57fbf888ca6961124d5b1bf6cdbefe7e4"
logical_lines = 3096
citation = "support:Cargo.lock:1-3096"
artifact_role = "cargo-integration"
review_overlay = "backend-identity-and-browser-bridges"
source_authority = "workspace-cargo-graph"
disposition = "review-full-source-semantics"
```

The example shows one artifact record; the real receipt repeats that exact
eight-key shape for all 52 inventory rows. Membership is exact and
duplicate-free. Every value mirrors the support TSV except `citation`, which is
the exact full `support:path:1-N` range. Artifact order is not authoritative.

Support findings have the same four keys as component findings, use IDs
`SR-SUP-01` through `SR-SUP-99`, and may cite only in-range `support:` paths
from the 52-file support set.

## Overlay receipt

The one canonical overlay receipt is
`docs/backend-port-source-reviews/overlays.source-review.toml`. It is admitted
only after all component and support evidence. Its top-level shape is exact:

```toml
schema_version = 1
receipt_kind = "source-review-overlays"
upstream_ref = "4ac7b32798da0482e441ef09304dc3b480ed3ee5"
workspace_base_ref = "5967e72706cf63702ecbc84a982e4172b8d6245f"
role = "sol-high"
review_run_id = "source-review-overlays-20260823"
review_wave = "overlays"
coverage = [
  "owned-source-lines",
  "translated-target-lines",
  "declarations",
  "conditionals",
  "include-owners",
  "source-semantics",
  "pinned-build-exclusions",
  "cross-backend-overlays",
]
findings = []
open_findings = 0
```

Every overlay record has exactly twelve keys. This is the exact second-record
shape, shown separately because the real receipt must place it after the much
larger first record. The two SHA-256 metavariables are replaced with the hashes
of the already admitted canonical component receipt files:

```toml
[[overlays]]
id = "webgpu-to-webgl2-load-store"
authority_record_count = 4
authority_sha256 = "b45f4d9b77e9856bdea375e3ca6e96f4192f30214a21c98505d1a47796e83e60"
component_ids = ["component-094", "component-109"]
support_paths = []
tree_bindings = []
external_bindings = []
generated_bindings = []
authority_keys = [
  "component:component-094",
  "component:component-109",
  "dependency-pair:webgpu:renderer:render_context_webgpu_impl->webgl2:renderer:load_store_actions_ext",
  "dependency-raw:campaign=webgpu\u001fdependency_syntax=cpp-include\u001fdependency_token=rive/renderer/gl/load_store_actions_ext.hpp\u001fdependency_unit=webgl2:renderer:load_store_actions_ext\u001fline=245\u001fresolution_kind=owned-source\u001fresolved_path=renderer/include/rive/renderer/gl/load_store_actions_ext.hpp\u001fresolved_sha256=1257e79033825a39b8004f2258733cfdadbee99e6cbd9777e1679d0a70349ecf\u001fsource_path=renderer/src/webgpu/render_context_webgpu_impl.cpp\u001fsource_unit=webgpu:renderer:render_context_webgpu_impl",
]
component_receipts = [
  { id = "component-094", path = "docs/backend-port-source-reviews/component-094.source-review.toml", sha256 = "<current component-094 receipt SHA-256>" },
  { id = "component-109", path = "docs/backend-port-source-reviews/component-109.source-review.toml", sha256 = "<current component-109 receipt SHA-256>" },
]
support_receipts = []
attestation = "reviewed-complete-derived-overlay-authority"
```

The `\u001f` escapes decode to the unit separator used between sorted field
pairs in a canonical raw TSV row. Each record's `authority_keys` is the exact
sorted, duplicate-free typed-record set. `authority_record_count` equals its
length, and `authority_sha256` is SHA-256 over the keys joined in that order by
a single newline. Count, digest, and full keys are all required; none can stand
in for another.

The real receipt contains exactly nine overlay records in this fixed order:

| Ordinal | Overlay ID | Authority records | Authority SHA-256 |
| ---: | --- | ---: | --- |
| 1 | `shared-authority-consumers` | 373 | `cd31ec1195880aa8756ee66eacc59628433974907e7531796060d073dad7d6a1` |
| 2 | `webgpu-to-webgl2-load-store` | 4 | `b45f4d9b77e9856bdea375e3ca6e96f4192f30214a21c98505d1a47796e83e60` |
| 3 | `generated-authority` | 1,913 | `1b1a49bfb5ec429acaed62de858f1772d5c7652e405bb7835d6dc3d327cc3175` |
| 4 | `webgpu-abi` | 70 | `12da060a0a297d35bd28b6970668d4d9c52cfbc0f5995d3d2c11fdef6ed3d2a7` |
| 5 | `shared-ore-contracts` | 110 | `2eaecaa75c637e2cab436cef72d579cbee280bc762dad4b3e67b5dbaa64f842e` |
| 6 | `shared-renderer-contracts` | 199 | `09b8fb1087ff555294e277326b71c419e510be1b837150b82fb9ffd5fbb8baac` |
| 7 | `vulkan-vma-adaptation` | 43 | `8dad68730becd028c4652b24951a6db769c4e36610c9ea44989052748f2c3fe2` |
| 8 | `backend-identity-and-browser-bridges` | 488 | `906e203bfd09cd484a06eacf3cc90ca69390d40b4d8b5e34598252d56f29994c` |
| 9 | `classification-boundary` | 52 | `22d78d511bef480af6c9ab696a401271e2e2614beb21615c434bea9ee09aa702` |

The remaining exact plan counters behind those record totals are:

| Overlay | Components | Support | Dependency keys | Semantic dependency keys | Configuration rows | Build predicates | Generated rows | Browser-bridge records | Physical generated | External files | Artifacts | Trees | Exclusions |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `shared-authority-consumers` | 68 | 0 | 305 | 297 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 |
| `webgpu-to-webgl2-load-store` | 2 | 0 | 2 | 2 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 |
| `generated-authority` | 67 | 4 | 791 | 787 | 0 | 0 | 520 | 0 | 514 | 0 | 14 | 3 | 0 |
| `webgpu-abi` | 17 | 2 | 39 | 39 | 0 | 0 | 0 | 0 | 0 | 0 | 12 | 0 | 0 |
| `shared-ore-contracts` | 21 | 7 | 70 | 70 | 0 | 0 | 0 | 0 | 0 | 12 | 0 | 0 | 0 |
| `shared-renderer-contracts` | 15 | 14 | 147 | 147 | 0 | 0 | 0 | 0 | 0 | 23 | 0 | 0 | 0 |
| `vulkan-vma-adaptation` | 4 | 11 | 24 | 24 | 0 | 0 | 0 | 0 | 0 | 0 | 3 | 1 | 0 |
| `backend-identity-and-browser-bridges` | 10 | 6 | 0 | 0 | 0 | 0 | 0 | 472 | 0 | 0 | 0 | 0 | 0 |
| `classification-boundary` | 13 | 8 | 0 | 0 | 1 | 18 | 0 | 0 | 0 | 0 | 0 | 0 | 12 |

The typed key closure is intentionally explicit:

- shared-authority consumers bind 198 raw edges and 107 unique unit pairs;
- the WebGPU-to-WebGL2 bridge binds its one raw edge and one unit pair;
- generated authority binds 440 raw edges, 351 unique unit pairs, all 520
  generated rows, all 514 retained generated-output byte bindings, 14 dependency
  files, and three dependency trees;
- WebGPU ABI binds 20 raw edges, 19 unit pairs, and its 12 exact dependency
  files;
- shared ORE binds 35 raw edges, 35 exact unit-to-authority seams, and all 12
  distinct physical external authority files;
- shared renderer binds 75 raw edges, 72 exact unit-to-authority seams, and all
  23 remaining physical external authority files;
- Vulkan VMA binds six raw `vk_mem_alloc.h` includes, 18 exact
  `source:line:vmaFunction` probes, three dependency files, and one dependency
  tree;
- backend identity binds ten exact components and six support files plus 472
  typed factory/module/browser records: seven backend factory definitions, nine
  public factory declarations/overloads, four Cargo feature arrays, four Rust
  backend root gates, 14 exact source-to-target module member wires, 113 WebGL
  command variants, 23 WebGL execution-provider queries, ten literal
  Emscripten-to-Rust WebGL extension queries, nine `EM_JS`-to-Rust semantic
  bridges, 274 classified WebGPU host symbol relationships, two `addToLibrary`
  registrations, two generator library wires, and the one explicit rejection of
  deprecated Emscripten `USE_WEBGPU`;
- classification boundary binds all 12 nontranslated `source:disposition`
  records, the exact `RIVE_WEBGL` configuration row, component `000`'s pinned
  build source, ten per-source platform-exclusion predicates, component `097`,
  and the eight positive Emscripten `.cpp`/WebGL selections that replace those
  native sources without fallback.

The browser overlay attests only to the current translated factory, module, and
host-bridge authority. At this checkpoint, `webgl2` is still gated by
`native-webgpu-experimental`, the default feature is still `rust-wgpu`, and no
WebGPU/WebGL2 editor selector exists. The `USE_WEBGPU` record rejects Emscripten's
deprecated built-in linkage option; it is not evidence of product-level backend
selection or no-fallback behavior. Explicit editor selection, physical browser
execution, product no-fallback checks, and legacy Rust-WGPU deletion remain
later ordered gates.

Every typed closure also includes its exact `component:` and `support:` keys.
File artifacts use `artifact:path:sha256`; trees use `tree:path:tree_sha256`;
physical upstream dependencies use `external:path:sha256`; retained generated
files use `generated-output:path:sha256`; canonical raw and generated rows list
their lexicographically sorted TSV field names separated by U+001F. This exposes
the full reviewed authority rather than allowing a count and digest to hide a
substituted seam.

For every record, `component_ids`, `support_paths`, `tree_bindings`,
`external_bindings`, `generated_bindings`, `authority_keys`,
`component_receipts`, and `support_receipts` are exact sorted lists rederived for
that overlay. `tree_bindings` is an array of zero or more tables, each with
exactly these keys:

```toml
[[overlays.tree_bindings]]
path = "<exact dependency-tree path>"
tree_sha256 = "<exact replayed tree SHA-256>"
```

`external_bindings` and `generated_bindings` are arrays of zero or more physical
upstream-file tables. Each table has exactly the path, replayed SHA-256, and
logical-line denominator:

```toml
[[overlays.external_bindings]]
path = "<exact pinned external upstream path>"
sha256 = "<exact replayed file SHA-256>"
logical_lines = 123
```

The `generated_bindings` form is identical except for the table name. The
external and generated paths, hashes, line counts, and sorted membership must
exactly match the overlay's replayed authority; ledger rows cannot stand in for
physical bytes.

`component_receipts` contains one binding for every `component_ids` member, in
the same sorted order. Each binding has exactly `id`, canonical repository-
relative `path`, and the SHA-256 of the complete tracked receipt bytes:

```toml
[[overlays.component_receipts]]
id = "component-094"
path = "docs/backend-port-source-reviews/component-094.source-review.toml"
sha256 = "<current component receipt SHA-256>"
```

If an overlay has any `support_paths`, `support_receipts` contains exactly one
binding to the complete support receipt; otherwise it is empty. The binding is
exactly:

```toml
[[overlays.support_receipts]]
id = "support"
path = "docs/backend-port-source-reviews/support.source-review.toml"
sha256 = "<current support receipt SHA-256>"
```

These receipt bindings are deliberately separate from `authority_keys` and its
digest. The keys expose the derived semantic authority, while the receipt
bindings identify the exact already admitted component and support evidence
bytes presented to the overlay reviewer. Any later byte change to those
prerequisite receipts invalidates the overlay receipt until its bindings are
refreshed.

The checker does not accept extra overlay fields. The attestation is always
exactly `reviewed-complete-derived-overlay-authority`.

An overlay finding adds `overlay_id` to the normal finding shape:

```toml
[[findings]]
id = "SR-OVL-02-01"
overlay_id = "webgpu-to-webgl2-load-store"
severity = "P1"
summary = "Describe one concrete mismatch across the derived seam"
citations = [
  "source:renderer/src/webgpu/render_context_webgpu_impl.cpp:100-120",
]
```

Overlay finding IDs are
`SR-OVL-<two-digit overlay ordinal>-<01 through 99>`, where the ordinal comes
from the fixed table above. Citations may name only source, translated target,
support file, dependency artifact, dependency-tree member, pinned external, or
retained generated-output bytes admitted to that overlay and must remain in
range. Their prefixes are respectively `source:`, `target:`, `support:`,
`artifact:`, `tree:`, `external:`, and `generated:`. Artifact, tree, external,
and generated citations are accepted only for the exact file or hashed-tree
membership of the named overlay; component and support receipts still accept
only their own source/target or support scopes.

## Structural red is success

For every receipt kind, `open_findings` must be an integer equal to the exact
number of finding records. It is not required to be zero. Component, support,
overlay, partial, and global validation all succeed structurally with P0-P3
findings still open. Global output reports `audit=red` and the exact open count
while exiting successfully when all structure and authority are valid. It
reports `audit=green` only when the count is zero.

Finding IDs must also be globally unique across all 117 receipts. Structural
red success records a complete first pass; it does not waive, downgrade, or
close any finding. The later correction gate owns closure.

## CLI modes

All modes require the same repository, upstream, and campaign arguments and
perform the complete admission replay described above.

Admission mode validates authority only. It does not require any source-review
receipt to exist, and is valid only while the active queue is `source-review`
with source-review status `active`:

```text
python3 tools/backend-port/check_source_review.py \
  --repo-root . \
  --upstream-root /Users/levi/dev/oss/rive-runtime \
  --manifest docs/backend-port-campaign.toml \
  --admission
```

Partial mode validates one tracked canonical receipt plus its structural
prerequisites. A relative receipt path is resolved from the repository root;
an absolute path is also accepted, but it must resolve directly inside the
configured receipt directory:

```text
python3 tools/backend-port/check_source_review.py \
  --repo-root . \
  --upstream-root /Users/levi/dev/oss/rive-runtime \
  --manifest docs/backend-port-campaign.toml \
  --receipt docs/backend-port-source-reviews/component-097.source-review.toml
```

For a component, partial mode validates every prior wave and then the
candidate; it does not require other same-wave or later receipts. For support,
it validates all 115 components first. For overlays, it validates all 115
components and support first.

Global mode is selected by omitting both mutually exclusive mode flags:

```text
python3 tools/backend-port/check_source_review.py \
  --repo-root . \
  --upstream-root /Users/levi/dev/oss/rive-runtime \
  --manifest docs/backend-port-campaign.toml
```

Global mode requires every file recursively present in the receipt directory
to equal the exact 117-file set: no missing, duplicate, nested, noncanonical,
or invented evidence file is allowed. It
then proves exact, nonoverlapping global coverage of all 135 units, 200 sources,
188 targets, 52 support artifacts, and nine overlays, and proves global finding
ID uniqueness.
