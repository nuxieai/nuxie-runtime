# UNIV-1687 data-viz performance evidence

This dossier enrolls `data_viz_demo` in the blocking runtime performance
ratchet after the retained text/layout work made a 100-frame session practical.
The qualified source is commit `8aabc827` (tree
`ca03565e14c2b09c482a9ea1b2c7ab559cc653e3`). The Rust runner SHA-256 is
`a58bfd445ba51ec9bf69a6ca8f73c0da695edb62757b324ba91a7180562d4bcf`; the
pinned C++ `4ac7b327` runner SHA-256 is
`3be7c09908310f88fb322f821c16ffb80f3df33d10655b50ddd62ddbea7d43df`.

## Blocking measurement

Three independent release sessions each measured 100 frames at 60 Hz, C++
first, five iterations, no warmups, with Rust script execution enabled. The
first complete results stand; no sample was discarded or rerun.

| Session | `advance + draw` ratio | Rust total hot loop | Raw SHA-256 |
| --- | ---: | ---: | --- |
| 1 | 215.911086x | 1,181.097 ms | `3873e93b7f97ff33b31052cd0138e7a3b51e6344da184530a8b9e28419da7a21` |
| 2 | 227.396295x | 1,175.914 ms | `a9975212108770e1d030a4e04b84af11d035257c0fd5f41965306b5a197bb6eb` |
| 3 | 244.770162x | 1,156.417 ms | `78a885eb1dcc6322428a8b2935b389ccb0bcf0c330b6b471733586a460beed3e` |

The checked-in [worst-session.json](worst-session.json) is a path-sanitized
copy of session 3, with SHA-256
`08e40d60d040307add43286353d183c275f9d14973f6ffad74289e7f616c76ee`.
The raw report remains in the external evidence archive at
`univ-1687-data-viz-20260810/reports/data-viz-ratchet-3.json`; the SHA above
authenticates the unsanitized authority. The blocking baseline is the worst
measured ratio, `244.770162`; its exact 15% ceiling is
`ceil(244.770162 * 1.15) = 282`.

### Full landing-gate confirmation

The mandatory 25-file `make perf-gate-tighten` ran three additional sessions
after the row was enrolled. Per the concurrent-work authorization, the quiet
wait was disabled and the first complete set stands. Recorded one-minute loads
were 30, 19, and 16 against the script's quiet threshold of 9.

`data_viz_demo` passed its 282x ceiling in all three sessions:

| Session | `advance + draw` ratio | Full-report SHA-256 |
| --- | ---: | --- |
| 1 | 191.637377x | `1d1e42a24a9a4dff2e963c52426a708e564d869dd243ccbb8bd02db22e8efeb6` |
| 2 | 272.445238x | `1e8c5d7b368e68ec833c7edb59cc8ef3275c331bff9dd088c37cf44a3c181c38` |
| 3 | 210.653252x | `a1af2ae1d434b7ab6ef5f267e978f7c3e49175eab4aa44c69fba55a87d3adc64` |

The overall tighten command correctly remained RED and made no manifest
mutation because four pre-existing small-workload rows exceeded their shipped
ceilings under that ambient load:
`script_dependency_test_using_library_v2` 2.019x > 2x,
`viewmodel_based_condition` 6.221x > 6x,
`image_scripting_property_value` 27.286x > 21x, and `multi_listeners` 5.613x >
4x. Those unrelated ratchets were neither loosened nor rerun. The complete raw
reports are retained in the external evidence archive
`univ-1687-data-viz-20260810/full-gate/` with the hashes above.

## Flamegraph attribution

The original pre-fix Time Profiler capture contained 5,716 fully resolved 1 ms
samples, but its ignored `target/` XML and trace were later reclaimed. Its
task transcript retained the exact successful capture/export commands, source
sequence, and parsed attribution. A first-run recapture reconstructed that
exact production state from `8aabc827` plus
[`pre-fix-source.patch`](pre-fix-source.patch): the patch restores the one
final convergence hunk that the accepted commit removed. The reconstructed
file byte-matches the `8aabc827^` version and the other six production files
remain exactly `8aabc827`.

The recapture contains 5,780 fully resolved 1 ms samples across 1,859 unique
stacks and independently reproduces the original attribution. Its folded
stacks have this dominant inclusive path:

```text
advance_scene_to
└─ ArtboardInstance::update_pass_with_script_mode
   ├─ ArtboardInstance::sync_style_changes
   │  └─ TaffyRuntimeLayoutEngine::compute_layout_with_root_hug
   └─ ArtboardInstance::apply_nested_artboard_layout_bounds
      └─ ArtboardInstance::refresh_layout_constraint_bounds
         └─ TaffyRuntimeLayoutEngine::compute_layout_with_root_hug
            └─ static_text_layout_measure_bounds
               └─ StaticTextSlice::layout_bounds_with_constraint
                  └─ shape_text_glyphs_with_features
                     └─ harfrust::hb_font_t::shape
```

Inclusive samples overlap because a flamegraph attributes one sample to every
ancestor in its stack:

| Frame | Inclusive samples | Inclusive time | Share of profile |
| --- | ---: | ---: | ---: |
| `TaffyRuntimeLayoutEngine::compute_layout_with_root_hug` | 5,382 | 5,382 ms | 93.1% |
| `static_text_layout_measure_bounds` | 5,022 | 5,022 ms | 86.9% |
| `harfrust::hb_font_t::shape` | 3,884 | 3,884 ms | 67.2% |
| `ArtboardInstance::sync_style_changes` | 2,739 | 2,739 ms | 47.4% |
| `nested_artboard_layout_axis_hug_size` | 2,568 | 2,568 ms | 44.4% |
| `ArtboardInstance::apply_nested_artboard_layout_bounds` | 2,464 | 2,464 ms | 42.6% |
| `ArtboardInstance::refresh_layout_constraint_bounds` | 2,408 | 2,408 ms | 41.7% |

The capture preceded only the final convergence correction. Test-only solve
accounting then made the causal link deterministic: a single data-bound gap
entered 505 Taffy solves because applying the accepted parent Yoga result
re-dirtied the same host. The production correction reduced that to one bounded
9-entry wave; the unchanged next frame performs zero solves while preserving
the exact animated child width.

Durable checked-in evidence:

- [`pre-fix-profile.folded.gz`](pre-fix-profile.folded.gz) is the complete
  semicolon-delimited folded-stack source, deterministically compressed with
  `gzip -9 -n`; SHA-256
  `f019028868e454195117590f68b462df0f27fc0895cb81449b60ed732d40dc62`.
- [`pre-fix-profile-summary.json`](pre-fix-profile-summary.json) records the
  sample period, total, unique-stack count, and ranked leaf/inclusive frames;
  SHA-256
  `88d4743f1138ae8abb0b8c9248ace83cc8594f3f669c6881f657fc26c1432468`.
- [`pre-fix-source.patch`](pre-fix-source.patch) reconstructs the profiled
  source from `8aabc827`; SHA-256
  `c4c3ce44e72d0c748880cb871a0e8920829761bcd7b3b43a125a6f3096e90b8d`.

`test_data_viz_profile_attribution_is_retained_and_derivable` decompresses the
folded source and derives every number in the table, so deleting or replacing
the retained profile fails the focused evidence contract. A standard flamegraph
can consume it directly with `gzip -dc pre-fix-profile.folded.gz`.

The first-run recapture used the same command shape as the original:

```sh
xcrun xctrace record --template 'Time Profiler' \
  --output data-viz-pre-fix.trace --launch -- \
  rust-golden-runner-scripted-pre-fix \
  --file "$RIVE_RUNTIME_DIR/tests/unit_tests/assets/data_viz_demo.riv" \
  --samples 0 --execute-scripts --benchmark --benchmark-repeat 1
xcrun xctrace export --input data-viz-pre-fix.trace \
  --xpath '//trace-toc[1]/run[1]/data[1]/table[@schema="time-profile"]' \
  --output data-viz-pre-fix-time-profile.xml
```

The profiled runner SHA-256 is
`be85c39a38ee4300a903e6d3802c8c953a65da8378c224f5e02f677d5707508a`
(Mach-O UUID `82EDDEDE-2611-3077-BEBE-5AECEB376853`). The raw XML SHA-256 is
`95cf5514b3162cc5e28c019285f9ef7f20f81a3512f67e07bc91bfe1a105b75c`;
the trace's sorted path-and-content SHA-256 is
`0b37fe245bb68349a3b3ecfc1adebcc5546a4863b5a2a10d4f323e6edcd325fc`.
Those large raw artifacts and the immutable runner remain in the external
`univ-1687-profile-recapture` evidence archive; the compact checked-in source
is the durable review authority.

## Ratchet discipline

`tools/land.sh` now runs `perf-gate-tighten`, not the non-mutating gate. If the
three-session result lowers any ratio, landing stops and prints the manifest
delta until the tightened `perf-corpus.toml` is committed. This prevents later
performance work from inheriting avoidable slack.
