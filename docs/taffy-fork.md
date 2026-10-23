# In-house Taffy fork

Decision (Levi, 2026-10-22): keep Taffy in a local directory, change it in
place, and record each change so a later Taffy update keeps it. Nothing is
posted upstream.

## Fork point

- crates.io `taffy` 0.15.0, vendored byte for byte at
  `vendor/taffy-0.15.0-nuxie-css` in runtime commit `3b62c2136d`.
- Upstream <https://github.com/DioxusLabs/taffy>, commit
  `eb6450694f33e2b7413e40677f230ff63f0fc194` (from `.cargo_vcs_info.json`).
- Identity, checksum and copy method:
  [NUXIE_PROVENANCE.md](../vendor/taffy-0.15.0-nuxie-css/NUXIE_PROVENANCE.md).

## Wiring

The root `Cargo.toml` redirects `taffy` to the vendor directory under
`[patch.crates-io]` and excludes that directory from the workspace. The
runtime keeps its `=0.15.0` requirement and feature list. The vendor
directory's `BUILD.bazel` and the `bazel/cargo` and
`bazel/native-tools-cargo` stubs come from `python3 tools/bazel/generate.py`;
do not edit them by hand.

## Carried patch

In-flow block, flex and grid children count their margin boxes in
scrollable overflow, at their in-flow position (block flow counts its end
after the trailing collapsed margins), so a last item's end margin extends a
CSS scroll range as it does in Chrome. Translated scrollers do not use Taffy's overflow.
[NUXIE_PATCH.md](../vendor/taffy-0.15.0-nuxie-css/NUXIE_PATCH.md) lists the
changed functions, the rules the change keeps, Chrome's numbers, the known
differences that remain, and the guard tests.

## Updating Taffy

1. Vendor the new published crate unchanged in its own commit, with its
   checksum and `.cargo_vcs_info.json` commit in `NUXIE_PROVENANCE.md`.
2. Reapply each change in `NUXIE_PATCH.md`: find the old sites by their
   `// nuxie-css:` comments, and check whether the new release's overflow
   code still leaves margins out. If it now counts them, drop the change and
   let the guard tests confirm it.
3. Update the root patch and exclude entries and the runtime's version
   requirement, then run `python3 tools/bazel/generate.py --reset-lock` and
   repin both crate universes with `CARGO_BAZEL_REPIN=1` and
   `CARGO_BAZEL_REPIN_ONLY` set to `runtime_crates`, then
   `runtime_native_tools_crates`.
4. Run the whole runtime package. The guard
   `css_scroll_overflow_taffy_fork_counts_the_last_items_margin` must still
   read 57. Treat any changed Chrome number in the CSS tests as a
   regression to investigate, not a value to update.
