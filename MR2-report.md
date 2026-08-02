# MR-2 C17 report

## Scope

C17 completed the step-1 exception-note seed defined by `.parity-decomp/mr-move-plan.md`. The 75 justified-exception sentences were appended verbatim to their corresponding `file-correspondence-manifest.toml` note fields.

- Manifest rows updated: 75
- `rust_module` fields changed: 0
- `audit_record` fields changed: 0
- B6 verdict fields changed: 0
- Rust source files changed: 0

Seeded rows: B6-0068, B6-0077, B6-0094, B6-0106, B6-0107, B6-0113, B6-0172–B6-0196, B6-0200, B6-0208, B6-0213, B6-0214, B6-0228, B6-0232, B6-0234, B6-0260, B6-0261, B6-0262, B6-0266, B6-0282, B6-0298, B6-0319–B6-0325, B6-0383, B6-0409–B6-0411, and B6-0426–B6-0445.

## Moved rows

None. This step was manifest-only and performed no code moves.

## Skipped rows

None. All 75 justified-exception rows in the plan were updated.

## Cross-root queue

None for the step-1 exception-note seed. No root owned by another cluster was touched.

## Verification

- The manifest parses as TOML with 448 rows.
- A byte-level row comparison against `HEAD` confirms that only the 75 targeted `note` fields changed.
- Every appended sentence exactly matches the plan's manifest-note text.
- `cargo check --workspace`: blocked in `nux-capi` header generation because `cbindgen` attempted to download uncached crates while network access was unavailable (`combine 4.6.7` online; `alsa 0.9.1` with `CARGO_NET_OFFLINE=true`). Compilation reached the unchanged `nux-capi` build script before failing.
