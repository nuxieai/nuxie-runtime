# Nuxie vendored taffy 0.15.0 (fork baseline)

This directory started as the complete crates.io `taffy` 0.15.0 package,
copied byte for byte into `vendor/taffy-0.15.0-nuxie-css` in runtime commit
`3b62c2136d`. The baseline is the published package, not a checkout of the
upstream repository. Local changes are listed in `NUXIE_PATCH.md`, and the
fork decision is recorded in `docs/taffy-fork.md` at the repository root.

- Package: crates.io `taffy` 0.15.0
- Package checksum (SHA-256 of the `.crate` archive, as in the fork point's
  `Cargo.lock`):
  `3c8480887e8803986d96a7173729565520d4bbb721ad6b7ad6941a22897207aa`
- Upstream repository: <https://github.com/DioxusLabs/taffy>
- Upstream commit: `eb6450694f33e2b7413e40677f230ff63f0fc194`
  (repository root, per `.cargo_vcs_info.json`)
- Copy method: the registry's `taffy-0.15.0.crate` archive, whose SHA-256
  matches the checksum above, was extracted into this directory with every
  file kept, `.cargo_vcs_info.json` included. A recursive diff of the
  extracted archive against commit `3b62c2136d` is empty.
- Patches: see `NUXIE_PATCH.md`. Nothing has been posted upstream.
