# Collapsed device path investigation

Throwaway evidence for [UNIV-5289](https://universe.basis.dev/issue/UNIV-5289). No runtime implementation was changed. The new regression intentionally fails on cb5e0504be.

Evidence paths in this report are relative to `/Users/levi/dev/nuxie-handoff/recreation-loop/live-moves-device-check`.

The primary fault is in the runtime video query path. The runtime's advance/draw counters do not cover all C API work. A retained collapsed row is still visited by video enumeration and is incorrectly reported as visible. Each per-video query rebuilds the entire occurrence list. Both production SDKs call those queries for the enumerated videos every frame.

## Numbers before changing performance behavior

The round 4 Simulator evidence is `data/ios-20261022T033929Z-83264/`. It reports 561 baseline frames at 7.53 ms mean and 157 collapsed frames at 55.4 ms mean. Android reports 297 frames at 17.427 ms and 150 frames at 56.175 ms. These are complete device frame costs. The existing logs do not split them by phase.

The request-file runner executed the same fixture with the actual C API, without either SDK. Each case runs 20 iterations. It uses the exported device fixture controls and installs the same move plan as the apps. That empties the fresh flight slot. It advances, acknowledges the presented render revision, enumerates videos, queries visibility for each occurrence and captures semantics. The baseline has one row. The other case retains forty rows and collapses rows 1 through 39. Both cases use the same viewport, active unit and inline copy.

| Host fastbuild measurement | One row | Forty rows, 39 collapsed |
| --- | ---: | ---: |
| Video occurrences | 2 | 41 |
| Reported visible video occurrences | 2 | 41 |
| Published semantic nodes | 2 | 2 |
| Step plus presentation acknowledgement, ms | 0.199819 | 4.083992 |
| Video enumeration, ms | 0.055815 | 1.208440 |
| Visibility queries for the whole video batch, ms | 0.167021 | 47.353469 |
| Semantic snapshot, ms | 0.248146 | 5.227690 |

These are host measurements. They are not substituted for Simulator or emulator measurements. The 47 ms visibility batch isolates substantial runtime cost with no SDK reconciliation, native overlay, decoder or renderer involved. It is of the same order as the extra device frame cost. The regression fails with 41 visible occurrences versus 2. See `data/collapsed-runtime-probe.txt` for the exact output.

## Runtime ownership

- `crates/nux-capi/src/video.rs`, `VideoOccurrences::collect`: traverses every `ArtboardHost.artboard_instance` without consulting the list's collapsed row state. It also builds the file's source-artboard list and scans it for each mounted artboard. It visits the forty retained row instances and B's empty flight wrapper.
- The same file, `with_video`: calls `collect` again for each occurrence lookup. `nux_player_video_is_visible`, video stepping, clock updates and other per-video operations use it. A loop over all occurrences therefore repeats a whole-scene walk for every video.
- The same file, `nux_player_video_is_visible`: delegates to `video::visibility::is_visible_in`. That path checks component ancestry, authored visibility, geometry and clipping. The returned result does not account for the collapsed list row in this fixture. Every retained video requests a visible decoder even though only two draw.
- `crates/nux-capi/src/semantic_snapshot.rs`, `nux_player_semantic_snapshot`: calls `validate_semantic_geometry` before exposing the snapshot. `crates/nuxie-runtime/src/mechanical_port/source/semantic/semantic_provider.rs`, `append_semantic_children`, enters every hosted artboard during that validation. The measured output stays at two nodes, but the snapshot work grows twentyfold. Validation intentionally covers excluded nodes today. A runtime fix must preserve geometry safety while making steady frames cheap, for example by validating unchanged retained subtrees once. This is a secondary measured cost, not a claim that hidden nodes are published.
- Step time also grows. `crates/nux-capi/src/player_view_models.rs`, `scene_roots_across`, walks retained hosted instances when the C API captures bound-owner changes. The phase timer measures step plus acknowledgement. It does not separately attribute that increase to this walk. The device runtime-step timer will show its actual frame share.

The regression permits retained video IDs to remain enumerable. It requires collapsed videos not to request visible decoders. Do not fix it by deleting the retained state or replacing the moving decoder. The runtime branch must also address repeated whole-scene lookups to meet the collapsed-stack budget. Passing the visibility assertion alone does not prove constant frame cost.

## Production SDK and harness behavior

The iOS production presentation loop sets `capturesSemantics` when it has a semantics subscriber. The interactive screen requests the editable native input names in that capture. Android's production `ExperienceSurfaceHost.publishSemantics` captures native fields on presented frames. Keeping those captures in this harness matches the native text path under test.

The iOS production `ExperienceVideoPlayback.admit` steps occurrences, enumerates again and queries visibility for the batch. Android's production `ExperienceVideoPlayback.admit` also refreshes visibility and steps occurrences. Their calls expose the runtime query cost. This investigation changed no SDK admission, playback, capture or reconciliation policy. It added observations only.

## Device phase measurements

Both harnesses now save `frame-timings.csv` and print `PASS frame-timing mode=one|collapsed40 part=... samples=... mean=... p95=...` after each scored ten-second window. Warm-up frames are marked `scored=0`. The per-frame CSV stays in memory until completion, so timing does not add per-frame disk writes to either benchmark window.

The columns cover runtime step, semantic capture, native input capture, video enumeration, SDK video reconciliation, the entire video tick, native overlay reconciliation, renderer submission/poll CPU, presentation wait, remaining work and total time. Counts cover semantic nodes, native fields, video occurrences and enumeration calls. Time columns use milliseconds. Count columns use raw counts. The enumeration time is a subset of video reconciliation/tick time. Reconciliation is a subset of the whole video tick. Do not add those nested columns when accounting for a frame. `other_ms` uses only non-overlapping phases.

On iOS, semantic and renderer timers run inside the pinned executor. Presentation wait measures the wait after submission and capture return. GPU work can overlap semantic capture. On Android, render submission includes the actual JNI submit and poll calls. Presentation wait measures the gaps between pending-presentation polls. It includes Choreographer scheduling delay. Both platforms' total time measures frame start to completed host work, not physical scanout.

Claude must run the updated apps to collect these device phase measurements. This sandbox cannot produce them. Until the runtime fix lands, the collapsed-budget check is expected to remain red. The harness does not hide collapsed occurrences or disable semantics to pass it.

## Other round 4 fixes

The iOS flight checkpoint compares 371837.178925 with 371837.266837. Wall time is 0.087912 seconds. The media clock advances 0.089040916 seconds. Playback kept pace. The old minimum of 0.1 came from scheduled display timestamps. Both harnesses now calculate the minimum as `min(0.1, wall interval * 0.25)` using monotonic observation time and log that interval. The independent total advancement and decoded-frame checks remain.

`decoder-ready` checks the tracked source, not an exact total decoder count. The count of three is diagnostic. It was never an asserted cardinality. The initial log enumerates the moving unit (1073741826), the inline copy (1073741825) and the collapsed destination unit (1073741824). The third decoder reflects the runtime visibility fault. The tracked video was sampled at exactly 2.0 seconds on a 2.0-second loop, where instantaneous `playing` can be false during the SDK's normal loop seek. Readiness now requires a ready, error-free AVPlayerItem or a prepared, error-free Android decoder with a real MediaPlayer identity. Object identity, delivered frames and advancing time are still tested separately.

Android's harness left `ExperienceVideoPlayback.hidden` true. Production calls `setVisible(shouldRun)` from frame scheduling. The harness now calls `setVisible(true)` before its first non-rest video tick. The rest comparison still runs before decoder frames arrive.

Android also queued a native edit, stepped and presented a newer capture, then executed the edit against the newer serial. The strict capture check rejected it. Its retry was queued behind the next presentation, so writes stayed one capture behind forever. The harness now has a dedicated text-write queue. Like production, it drains when no presentation is pending, before another step invalidates the current capture. It posts write completion to the UI queue. Capture validation, occurrence ownership and the production focus-input sequence stay intact. It still types through the real system InputConnection. No runtime TextInput or VM string setter is used. Both harnesses retain Claude's left-most native field selection.

## Reproduce

The new runtime test file is `crates/nux-capi/tests/live_moves_collapsed_device_probe.rs`. It imports the existing throwaway fixture helper `live_moves_harness_fixture.rs`. `tools/probe-runtime.sh` stages both from the handoff and submits this runner line:

```text
test -p nux-capi --test live_moves_collapsed_device_probe live_moves_collapsed_rows_do_not_request_visible_video_decoders
```

The completed request was `request-devicecollapsedprobe3`. It exited nonzero on the visibility assertion after printing both measurements. The first measured probe left an unused flight instance in place. The final probe installs the actual move plan before collapse and reproduces the failure with that instance removed. An earlier request with unsupported extra test arguments was rejected by the runner and provided no test evidence.
