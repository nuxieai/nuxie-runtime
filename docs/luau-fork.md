# In-house luaur fork (Luau scripting engine)

Decision (Levi, 2026-08-04): fork `luaur` — the pure-Rust Luau translation
that is Nuxie's scripting engine — and maintain the Luau engine port
in-house rather than waiting on upstream luaur releases. This document
records the fork point, the carried patches, and the port plan. It owns the
exit path for the standing WATCH `deferred-2026-07-19-luau-engine`
(docs/parity-gap-register.md): that row's exit criterion is now **fork
parity with the pinned C++ engine**, not "luaur publishes a newer base".

## Why

`deferred-2026-07-19-luau-engine` is the highest-urgency WATCH (rated 8/10,
3+ triage cycles stale). Upstream C++ runs Luau `rive_0_732` internals
(docs/sync/triage-2026-08-02-e0d4913f.md) while luaur 0.1.8 is based on
Luau 0.724-era sources, and luaur's release cadence is the bottleneck.
Owning the fork makes the engine bump our own port lane instead of an
external dependency.

## Fork point

The fork baseline is the complete luaur 0.1.8 crate family as published on
crates.io, vendored byte-for-byte under `vendor/`:

| crate | vendored at | modified? | provenance file |
|---|---|---|---|
| `luaur-ast` | `vendor/luaur-ast-0.1.8` | no | `NUXIE_PROVENANCE.md` |
| `luaur-bytecode` | `vendor/luaur-bytecode-0.1.8` | no | `NUXIE_PROVENANCE.md` |
| `luaur-common` | `vendor/luaur-common-0.1.8` | yes (Apple clock) | `NUXIE_PATCH.md` |
| `luaur-compiler` | `vendor/luaur-compiler-0.1.8` | no | `NUXIE_PROVENANCE.md` |
| `luaur-rt` | `vendor/luaur-rt-0.1.8` | no | `NUXIE_PROVENANCE.md` |
| `luaur-vm` | `vendor/luaur-vm-0.1.8` | yes (Apple clock) | `NUXIE_PATCH.md` |

- Upstream repository: `https://github.com/pjankiewicz/luaur`
- Upstream commit for all six packages:
  `f0eac7f7cce691d0cdb0b93c3eef9d599f71d739` (per each package's
  `.cargo_vcs_info.json`)
- Luau base: 0.724-era — luaur 0.1.8 advertises validation against upstream
  Luau commit `8f33df9` (luaur README; see also the pin comment in
  `crates/nuxie-scripting/Cargo.toml`)
- Original crates.io checksums are recorded in each vendored package's
  provenance file. `luaur-common` and `luaur-vm` predate this fork as
  vendored patched packages; the other four were added at the fork point,
  verified against the Cargo.lock checksums before extraction.
- `luaur-analysis` and `luaur-config` (optional `luaur-rt` dependencies)
  are not in Nuxie's dependency graph and are not vendored. If a future
  feature pulls them in, vendor them at the same fork point first.

## Workspace wiring

`[patch.crates-io]` in the workspace `Cargo.toml` redirects all six crate
names to the vendored packages; the consuming crates (`nuxie-scripting`,
`nuxie`) keep their `=0.1.8` crates.io-style pins. The vendored directories
are workspace-`exclude`d, same as the wgpu vendor packages. The switch is
behavior-preserving by construction: the four newly vendored packages are
the exact bytes cargo previously compiled from the registry cache, and the
Cargo.lock delta is only the source/checksum lines flipping from registry
to path for those four.

Verified at the fork switch (both green, no behavioral diff):

- `cargo test -p nuxie --features scripting`
- `make scripted-golden-compare` (byte-level side-channel differential
  against the pinned C++ scripted runner)

## Carried patches

1. **Apple Mach-clock widening** (`luaur-common`, `luaur-vm`; predates the
   fork). Widens the upstream Mach monotonic-clock branches from
   `target_os = "macos"` to `target_vendor = "apple"` so iOS device and
   simulator builds compile and keep macOS clock semantics. Files:
   `get_clock_timestamp.rs`/`get_clock_period.rs` (common),
   `clock_timestamp.rs`/`clock_period.rs` (vm). Details in each package's
   `NUXIE_PATCH.md`.

Every future engine change lands as a documented entry in the affected
package's `NUXIE_PATCH.md` (create one when a baseline package is first
modified) plus a row in this document, so fork-vs-crates.io drift stays
enumerable.

## Port plan: Luau 0.724 → 0.732

Upstream C++ is on Luau `rive_0_732` (the `luigi-rosso/luau` fork; final
pin landed via S4-43 `395defdb`, progressing `rive_0_730` → `rive_0_731` →
`rive_0_732`; the exact 0.732 C++ runner completes all 321 scripted corpus
entries — docs/sync/triage-2026-08-02-e0d4913f.md).

Plan, as structure-preserving lanes against the pinned C++ scripted corpus:

1. Port the Luau 0.725→0.732 VM/JIT/GC deltas into the vendored fork,
   lane by lane, keeping luaur's translation structure so future upstream
   luaur diffs stay comparable.
2. **Known target symptom to retire:** `scope_probe` SIGTRAPs on
   `lua_pushcclosurek`'s stack assertion under newer-engine comparison
   (S4-3 carry-forward; C++ completes the stream while Rust traps). Fork
   parity is not claimed until this reproduces green.
3. Oracles: the real bytecode-v7 fixture and the full forced-scripted
   ratchet remain the floor; the exact-0.732 C++ runner is the target
   comparison. The editor-emitted-bytecode compatibility matrix
   (upstream_ref `4ac7b327` discipline — read `port-manifest.toml`, not
   triage briefs, for the pin) is a valid interim evidence step.
4. Versioning: vendored crate versions stay `0.1.8` while the `=0.1.8`
   pins hold (a `[patch]` must satisfy the dependency requirement); the
   fork state is identified by this document plus per-package patch files,
   not by version bumps. Revisit the version scheme if the fork diverges
   enough that a distinct version communicates better.
5. When fork parity lands, update the `deferred-2026-07-19-luau-engine`
   WATCH row to CLOSED with the fork-parity evidence.

Out of scope for the fork-setup change that introduced this document: no
engine internals were ported; the vendor expansion and workspace switch are
provenance-only.
