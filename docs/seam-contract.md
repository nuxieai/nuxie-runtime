> **DRAFT — pending Levi review.** The guard runs grandfathered/report-only until the migration decisions below are ratified.

# Parity, C ABI, and product seam contract (draft)

Status: P3-h architecture-review draft. This document records the intended
dependency direction and the current migration debt; it does not move code or
close any parity-gap register row.

The behavioral reference for this review is rive-runtime `d788e8ec`. The
working branch already contains the S4 advance through `4ac7b327`, but that
advance does not change the classification below: product additions are not
promoted to parity merely because they share a crate with ported code.

## The seam

There are three modules, plus the oracle consumers that police the lowest one:

```text
                         product-specific ABI
                                  |
                                  v
                         product / authoring
                                  |
                                  v
portable C ABI adapter ------> parity baseline <------ replay/oracle tools
```

The portable C ABI and product layer are sibling consumers of the parity
baseline. The product layer does not have to call Rust through the portable C
ABI when an in-process Rust interface is available. A product-specific C or
Swift boundary may adapt the product layer, but it is not `nux-capi`.

### Parity baseline

The parity baseline owns behavior whose specification is the pinned C++
runtime: `.riv` import, object/graph construction, frame advance and apply,
state-machine and animation execution, scripting semantics, renderer
interfaces, and faithful embedder operations. Its interface includes the
ordering, lifetime, error, and factory-domain rules recorded in
`docs/PORTING.md`; it is not only a collection of Rust type signatures.

Baseline code may depend only on other baseline modules and general-purpose
implementation dependencies. It must not know about product flows, authoring
transactions, ProjectDO identities, Nuxie artifact manifests, or product host
commands. A product requirement that needs runtime participation must enter
through a small baseline-owned interface at a real seam, with the product
implementation supplied as an adapter.

The baseline currently spans the low-level `nuxie-*` crates and the faithful
part of the `nuxie` facade. `File`, `Artboard`, `ArtboardInstance`,
`OwnedArtboardInstance`, `ViewModelInstance`, renderer re-exports, asset-loader
interfaces, the standalone `RawText` facade, and the audio facade stay here:
they are SDK surface, but they expose pinned runtime capabilities rather than
product policy. `FileImportLimits` also stays as the approved D11 host-safety
adaptation; its status must remain explicit rather than being mistaken for
pinned C++ behavior.

### Portable C ABI

`crates/nux-capi` is a thin adapter over the parity baseline. It owns C calling
conventions, opaque handles, lifetime checks, buffer negotiation, stable error
codes, and callback marshalling. It does not own runtime semantics and must not
choose a product player, synthesize a flow protocol, author a scene, interpret
a ProjectDO envelope, or install a product scripting module.

The portable C ABI may expose an operation once the baseline owns that
operation. A3--A5 are therefore capability-fragmentation work, not permission
for `nux-capi` to call `FlowSession`. Product-only operations belong in a
separately named product ABI above the product layer.

Today `nux-capi` depends on the mixed `nuxie` package, but its source imports
only baseline facade symbols. That package edge is temporary migration debt:
after the split, its manifest must resolve only to the baseline facade.

### Product and authoring layer

The product layer owns Nuxie policy and durable product vocabulary. It may
depend on the parity baseline. It may combine several baseline calls into a
deep module, but it may not duplicate or replace the pinned frame loop. In
particular, it may select a player, validate a host batch, lower an authored
document, and translate outputs; the actual import, advance/apply, hit,
settlement, event, and draw semantics remain baseline-owned.

Product additions are classified as additions in the Rust attribution ledger,
not mapped to a C++ file as `faithful`. If a product adapter deliberately
changes pinned behavior, that requires the normal user-approved D-row; calling
the behavior “SDK policy” is not sufficient.

### Replay and oracle tools

Golden runners, silver-corpus execution, renderer replay, fuzz replay, and
performance replay import the parity baseline directly. They must not import
product or authoring modules, even for fixture convenience. A product-corpus
gate may prepare a product artifact in a separate step, but the parity oracle
must execute the resulting `.riv` through the same baseline interface used for
the C++ comparison. This keeps V1--V5 evidence independent of product glue.

## Current inventory

The inventory distinguishes product surface that must move, mixed-file glue
that must be cut at a seam, and faithful SDK surface that should remain. Line
counts are review aids at the current tree, not ratchets.

### Product surface to move

| Concept | Current owners and evidence | Why it is product-owned |
|---|---|---|
| Flow execution protocol | `crates/nuxie/src/flow_session.rs` (7,207 lines), `crates/nuxie/tests/flow_session_contract.rs`, and FlowSession users in `vector_scripted_drawable.rs` and scripted-listener tests | Defines session ids, player-selection policy, catalogs, recursive value arenas, state/text/pointer batches, output phases, host mutation correlation, quotas, wake scheduling, terminal poisoning, and a renderer-neutral remote-UI protocol. Pinned C++ has command queue/server instead (F3); FlowSession is not its port. |
| Dynamic scene authoring | `crates/nuxie/src/scene.rs` (36,930 lines), `crates/nuxie/build.rs` (4,453 lines), and `crates/nuxie/tests/scene_authoring.rs` (15,541 lines) | Defines stable authored identities, transactional edits and rollback diagnostics, typed specs, property tokens, view-model/data-converter/state-machine/animation authoring, asset ownership, export/lowering, instance remounting, and authored observation/hit/text geometry. The register explicitly calls `scene::Scene` a Nuxie-only additive surface. |
| Component-list authoring and layout policy | Generated `ArtboardComponentListSpec`, `ArtboardComponentListFlow`, `ArtboardComponentListAxis`, map rules and source paths in `build.rs`/`scene.rs`; lowering around `scene.rs:22110`; authoring tests | PR #161 (`da424ed4`) made the synthesized flow wrapper absolute so it does not join its parent flow. PR #184 (`a83abc32`) added `layout_hosted` and hug sizing, then was reverted by `1f69b3a5`. The surviving absolute-wrapper rule is an authoring/lowering policy, not runtime parity behavior. Future work must not put a second version in runtime or replay code. |
| ProjectDO converter vocabulary and execution | `crates/nuxie-runtime/src/project_data_converter.rs` (2,686 lines), its public re-exports in `nuxie-runtime/src/lib.rs`, ProjectDO decode/evaluation in `data_bind/context/context_value.rs`, and the list-length call from `data_bind/converters/data_converter_number_to_list.rs` | The durable ProjectDO ids, JSON envelope, React-time convention, compile/evaluate/reverse contract, and resolver are absent from the pinned C++ tree. This is the clearest product-to-baseline dependency inversion. |
| Nuxie Luau host-effect protocol | `crates/nuxie-scripting/src/vm/host_commands.rs` (637 lines), `host_commands` fields/exports/installation in `vm.rs`, and host-specific portions of `vm/resource_limits.rs` | Installs private `require("nuxie")`, normalizes `HostValue`, queues `Trigger`/`ResponseSet`, defines cycle checkpoints, and applies product host payload/command limits. These are consumed by FlowSession, not by pinned Rive scripting. |
| Product artifact trust | `crates/nuxie/src/script_import.rs`, the optional `nux-container` dependency, and authenticated import entry points/glue in `crates/nuxie/src/lib.rs` | Ed25519 manifest verification and binding authority to exact artifact bytes are product distribution policy. The baseline still needs an explicit embedder-controlled decision to execute bytecode, but it must not know the Nux package or manifest format. |

The existing `rust-additions.toml` supports this classification. It labels
`flow_session.rs`, `script_import.rs`, the host-command/resource files, and
`project_data_converter.rs` as `flowsession-abi` or `scene-api`. That ledger is
file-granular, so it does not expose all mixed-file glue described next.

### Mixed-file glue and transitive reach

The product modules are not isolated merely because `flow_session.rs` and
`scene.rs` have recognizable names:

- `crates/nuxie/src/lib.rs` detects ProjectDO envelopes, retains product flags
  in `FileScriptAsset`, begins/rolls back/drains host cycles, and exposes
  `prepare_flow_*` helpers on the baseline facade owner.
- `crates/nuxie-scripting/src/vm.rs` constructs a host-command queue and
  installs the private Nuxie module during ordinary VM setup. Thus a baseline
  VM carries product state even when no FlowSession exists.
- `crates/nuxie-runtime/src/data_bind/context/context_value.rs` decodes and
  runs ProjectDO programs inside the core data-bind path.
- `crates/nuxie-runtime/src/data_bind/converters/data_converter_number_to_list.rs`
  calls a ProjectDO helper for a pinned converter's bounded length. That arrow
  must be reversed: the baseline owns its numeric conversion, and a product
  adapter may reuse the baseline rule.
- `crates/nuxie/src/lib.rs` publicly re-exports ProjectDO types from
  `nuxie-runtime`, so crate-root imports can bypass an obvious product module
  path.

This is why a source grep alone cannot prove the final seam in the current
package shape. The final proof is a Cargo package graph with product code in
separate crates, plus a source check for prohibited package/module imports.

### SDK surface reviewed and retained in the baseline

The following are not seam violations:

- the `nuxie` File/Artboard/instance facade and renderer re-exports;
- `crates/nuxie/src/raw_text.rs` and `nuxie-runtime::RuntimeRawText`, which are
  direct standalone RawText counterparts at the current S4 pin;
- `crates/nuxie-audio`'s public facade over the pinned headless audio owners;
- `nuxie-scripting::envelope::SignedContent`, which mirrors pinned
  `include/rive/signed_content_header.hpp` even though the additions ledger
  currently groups its Rust file with FlowSession work;
- generic Luau bytecode validation, VM memory/safepoint protection, and the
  ordinary Rive scripting host interface. Host-command payload limits and the
  private Nuxie module must be separated from those generic protections;
- richer caret/hit/selection observations over a runtime instance. The
  low-level observation primitives may stay baseline; authored identity paths
  and transactional Scene projections move with authoring.

## Mechanical guard

`tools/seam-check/check.py` is the stage-one guard. It checks every protected
runtime/parity manifest and replay/oracle manifest for direct dependencies on
the mixed `nuxie` facade, `nux-container`, or current/future product package
names. It also rejects explicit `nuxie::flow_session` and product-authoring
module paths in protected Rust sources. The package rule is deliberately
fail-closed for renamed Cargo dependencies by checking both the dependency key
and its `package` value.

The checker is useful today because all runtime and replay packages already
avoid the mixed `nuxie` package. Its focused tests prove ordinary, renamed, and
target-specific dependency failures, explicit module-path failures, and the
current repository pass.

It is not the final seam proof. ProjectDO and host commands are presently
inside protected packages, and root re-exports erase their module path. The
checker therefore reports those two grandfathered internal debt families in
its success output rather than claiming they are clean. No new grandfathered
family is permitted. After extraction, both debt entries are deleted and the
product package names become ordinary forbidden dependencies; a clean result
must then report zero internal debt.

The final CI command should be:

```sh
python3 tools/seam-check/check.py --repo-root .
python3 -m unittest discover -s tools/seam-check -p 'test_*.py'
```

Wiring that command into a shared Makefile or CI workflow is landing-owner
work, not part of this lane's disjoint diff.

## Migration sketch (no code moves in P3-h)

### 1. Establish package ownership

Keep `nuxie` as the parity SDK facade, or rename it once and update all
consumers atomically. Create product crates with names the checker already
reserves:

- `nuxie-authoring` for Scene, its schema generator, and authored observation;
- `nuxie-flow` for FlowSession and its stable host protocol;
- `nuxie-project-data` for ProjectDO vocabulary/programs/adapters;
- optionally `nuxie-product-scripting` for the private Nuxie Luau module and
  host-effect normalization if it is useful outside FlowSession.

`nux-container` remains a separate product package and becomes a dependency of
the product import layer, not of the baseline facade.

Exit evidence: `cargo metadata` shows no baseline, C ABI, or replay package
depending on a product package; the seam checker is green with its current
package rules.

### 2. Move Scene as one deep authoring module

Move `scene.rs`, the scene-schema part of `build.rs`, and scene-authoring tests
together. Keep stable ids, transaction validation, lowering, remounting, and
authored observation behind the Scene interface; do not split each spec into a
shallow crate. Move `ArtboardComponentListFlow` and the surviving PR #161
absolute-wrapper rule with Scene. Record the reverted PR #184 `layout_hosted`
experiment in product history, not as a baseline compatibility obligation.

Where Scene needs a runtime operation, first use an existing baseline facade
operation. Add a baseline interface only when Scene and another real consumer
need distinct adapters or when private runtime access is the only remaining
dependency.

Exit evidence: all existing scene-authoring focused tests pass from the product
crate; baseline/replay manifests do not mention `nuxie-authoring`; ordinary
golden execution has no authoring dependency.

### 3. Extract FlowSession and product scripting effects

Move `flow_session.rs` and its protocol tests to `nuxie-flow`. Move
`HostCommand`, `HostValue`, queue/checkpoint logic, private module installation,
and host-specific resource-limit variants above the scripting baseline. The
baseline VM should accept module installation/host effects through its existing
open scripting-host seam or a smaller injected module adapter; generic VM
memory, bytecode, and safepoint protection stays below.

FlowSession remains single-threaded unless HD-1 resolves F3/A6 differently.
This migration does not choose between the command-server port and the
documented FlowSession model, and therefore does not close F3 or A6.

Exit evidence: a baseline VM contains no `host_commands` field and does not
install `require("nuxie")`; FlowSession focused tests pass through the product
adapter; no parity/replay dependency points upward.

### 4. Reverse the ProjectDO dependency

Define the narrow baseline-owned data-converter extension interface at the
existing converter construction/evaluation seam. It must traffic in baseline
runtime values and stable errors, not ProjectDO ids or JSON. The built-in Rive
converter implementation and the external ProjectDO adapter are the two real
adapters, so this is a justified seam rather than hypothetical indirection.

Move program compilation, JSON envelope encoding/decoding, durable ProjectDO
paths, React-time convention, and resolver logic to `nuxie-project-data`.
Replace the core NumberToList call into ProjectDO with a baseline-owned numeric
rule. Remove ProjectDO re-exports from `nuxie-runtime` and `nuxie`.

Exit evidence: `rg ProjectData crates/nuxie-runtime` is empty outside adapter
interface tests; runtime converter tests exercise the baseline interface;
product converter tests exercise the same interface through the ProjectDO
adapter; the internal-debt line disappears from the seam checker.

### 5. Split execution permission from product authentication

Keep a neutral, explicit embedder decision for executable scripts in the
baseline interface. Move Ed25519 manifest verification, Nux package parsing,
and exact-artifact binding to a product verifier that yields or invokes that
neutral permission. Product types must not appear in `File` storage or import
errors.

Exit evidence: the baseline builds without `ed25519-dalek`, `nux-container`,
or product manifest types; product authentication tests retain their current
tamper and exact-byte guarantees.

### 6. Repoint and expand the C ABI

Repoint `nux-capi` to the clean baseline facade. Implement A3--A5 against
baseline operations, not FlowSession. If a product C boundary is still needed,
place it in a separately named crate that depends on `nuxie-flow` and uses
distinct symbols/headers.

Exit evidence: the portable header contains no Flow/Scene/ProjectDO terms;
`nux-capi` has no product dependency; focused C API smoke tests cover each
newly exposed baseline operation. Register rows move only after those named
mechanical gates, not after the package move itself.

## Register and provenance consequences

- F3 and A6 remain pending until HD-1's user decision and its selected gate.
- A3--A5 remain pending until the portable C ABI exposes and tests the baseline
  capabilities. Moving FlowSession cannot close them.
- The additive Scene note remains true; migration changes ownership, not
  support.
- No D-row is created by this review. None of the proposed package moves
  changes runtime behavior.
- No correspondence or frame-loop row moves in this lane. New product files in
  a migration must be classified in `rust-additions.toml`; new baseline Rust
  owners must update both correspondence ledgers and frame-loop ownership when
  applicable, following FLR-16. Product adapters must carry an explicit
  “Nuxie-only; no pinned C++ counterpart” attribution comment.

