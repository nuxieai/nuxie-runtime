# FL-E2 upstream-test evidence

This note records the image, mesh, and image-asset slice of the pinned
`W65-unit-test-triage.md` assignments. Tests read the real fixtures from the
pinned C++ checkout; they do not replace them with synthetic images.

| W65 class | Upstream file | Cases | Rust evidence |
|---|---|---:|---|
| C | `image_asset_test.cpp` | 2 | `upstream_image_asset_in_band_literals_and_sharing_are_ported` and `upstream_image_asset_out_of_band_literals_and_sharing_are_ported` preserve encoded sizes and shared ImageAsset identity |
| C | `image_decoders_test.cpp` | 5 | five `cpp_probe` tests read `placeholder.png`, `open_source.jpg`, `bad.jpg`, `bad.png`, and `1.webp`, pin encoded sizes, and assert decode dimensions or bounded rejection |
| B | `image_mesh_test.cpp` | 2 | `upstream_image_mesh_fixture_literals_are_ported` pins the Tape mesh/asset/index contract; `artboard_clone_shares_file_image_and_mesh_source_but_not_occurrence_buffers` pins shared source indices and clone-local dynamic buffers; ordinary and scripted goldens execute `tape.riv` exactly |
| B | `in_band_asset_load_test.cpp` | 3 | the three `upstream_in_band_*` tests cover metadata, loader responsibility, and rejection fallback; the C++ probe and both golden lanes execute `in_band_asset.riv` |

The malformed `bad.jpg` assertion is an explicit decoder-policy divergence,
not a weakened test. Pinned non-Apple C++ accepts the `24566 × 58278` header to
prove its decoder does not overflow; that case is excluded on Apple. Rust's
cross-platform import policy rejects either dimension above 8192 before a
decoded allocation. The test therefore pins the same real 88,731-byte input
and requires early rejection. `bad.png` is rejected for the same bounded-host
policy (matching the pinned non-Apple outcome; Apple CoreGraphics differs).

The broader draw scenarios are retained in the ordinary/scripted golden and
silver corpora. Silver classifications move only when the action interpreter
can execute the authored scenario; unrelated view-model mutation or scripted
replay blockers remain named rather than being reclassified optimistically.
