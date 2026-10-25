# HarfBuzz outline emoji fixture

`Zycon.ttf` is the exact `test/api/fonts/Zycon.ttf` from the existing pinned
`rive-app_harfbuzz_rive_13.1.1` dependency, commit
`08d34675f64b1ac4880f4f10c9fd474a56dd4399`. Its bytes were checked against that Git object.

SHA-256: `5188939e66d3fd5b20eef5bb9b3e44d0ef73465a7cb9a2b74cd8af0abc8fe903`

The raw-text outline regression previously read this same font from an absolute
checkout path. It now embeds this retained copy so Cargo and Bazel/pool runs use
the same fixture. The runtime package's existing `_package_inputs` glob includes
it in Bazel `compile_data`; no developer-local font directory is required.
