# Rust vs C++ runtime performance: map and menu

Status: 2026-10-31. Main `b9d4c38b07`, C++ pin `022a0fa9`; every Rust
number is measured at `ca1739617d` (main plus a one-line runner compile fix,
section 2). Measured numbers cite their source file under
`~/dev/worktrees/perf-scripts/out/`; estimates are marked.

Two parts. The **map** (section 3) says what the Rust runtime does per
frame and what the C++ runtime does for the same work, at three
resolutions: per fixture, per subsystem, and per operation (one keyframe
applied, one dirt edge marked, one shape drawn), with the call path under
each so "why this many more instructions for the same operation" can be
read off the row. The **menu** (section 4) lists every opportunity the map
exposes, sized and ordered, without deciding what to do.

## 1. Headline

Metric: retired instructions for advance + draw over the perf corpus
(`perf-corpus.toml`; 23 of its 25 entries are still classified exact),
10,000 frames at 60 Hz, scripting ON, Rust fat LTO vs the pinned C++ release
runner, each phase's minimum of 3 runs, ratios of summed counts.

| Date | Tree | advance | draw | advance+draw | Source |
| --- | --- | ---: | ---: | ---: | --- |
| 2026-10-09 | main before the perf campaigns | 3.66x | 2.55x | 3.26x | memory, PR #1001 era |
| 2026-10-15 | main `c3a4b6092f` (PR #1072 landed) | 1.61x | 1.70x | **1.64x** | out/instr-nx17.txt |
| 2026-10-16 | `claude/dod-compiled-animation` tip (never merged) | 1.28x | 1.19x | **1.24x** | out/instr-dod16.txt |
| 2026-10-26 | main `2509c5d0b8`, same 23 fixtures | 2.53x | 2.57x | **2.54x** | Codex Stage 1 baseline.json |
| 2026-10-31 | main `b9d4c38b07` + runner fix, 23 fixtures | 2.65x | 2.57x | **2.62x** | out/main-oct31.txt |

What the map shows, in one screen:

- Rust runs the same operations the same number of times as C++ (every
  ledger row in 3.3 with matching call counts) and pays more per operation:
  1.5-2x on geometry-adjacent work, 2-3x on animation and draw, 5-15x on
  per-artboard, per-layer and per-bind bookkeeping.
- Today's 2.62x is 1.64x of structural cost plus two commits: `0588c46b1b`
  (Oct 23) taxes every object access through a thread-local scope (+21-102%
  advance, +30-77% draw per fixture), and `10ba454b9b` (Oct 28) walks every
  converter's bind container every frame (car advance +44%, data_viz +11%).
- The structural 1.64x is the handle model: each object touch pays Rc
  traffic, a generation check, a `RefCell` flag, a vtable probe and a
  closure (3.5), multiplied by walks that C++ does with pointer loads.
- Geometry kernels (path building, skinning, stroking, HarfBuzz shaping)
  are at or near parity. Nothing in the map says Rust is slow at arithmetic.
- A branch that reached 1.24x on the Oct 16 tree exists and was never
  merged; its commits are evidence for several menu items.

## 2. How the numbers are made (the method)

The method that worked, in the order to apply it. Every tool named here lives
in `~/dev/worktrees/perf-scripts` unless given a repo path.

| Step | Question | Tool | Notes |
| --- | --- | --- | --- |
| 1 | How far off, per fixture and phase? | `count.sh` / `measure.sh <commit> <name>` (fat LTO, `RIVE_GOLDEN_INSTRUCTIONS=1`, `perf-compare --runner-benchmark`) then `instr_summary.py`, `instr_diff.py` | min of 3 iterations; C++ control drifts 0.2-0.4%; never read a single run |
| 2 | Which commit moved it? | `fastcount.sh <worktree> <commit> <name>` (no fat LTO, builds in minutes) | fast counts compare only with fast counts; a 1.5x step is far above fast-build noise (2-5%) |
| 3 | Where exactly do the extra instructions go? | `pt.sh <worktree|cpp|binary> <fixture> <name> [frames]` → `pt_tree.py`, `pt_diff.py`, `pt_excl.py`, `pt_callers.py` | Instruments Processor Trace, exact per-function retired instructions, Rust and C++ side by side. Time profiles (samply) misled twice; trust PT. Traces come back empty under host load: check the aggregate's total and rerun |
| 4 | Is the extra work *more operations* or *dearer operations*? | `tools/perf-gate/callcounts.py`; `census.sh`/`census.py` (`NUXIE_COMPONENT_GRAPH_TRACE`) | so far: same op counts, dearer ops. Census gives applies, writes, dirt marks, cascade visits, walk kinds per frame |
| 5 | What is the ceiling of one suspect? | `PATCH_CMD=... fastcount.sh` (delete the suspect, count, restore) | measurement-only; unsafe or unsound patches never ship |
| 6 | Did a change stay exact? | `golden_quick.sh <fast-name>`, then the full gate (`gate3.sh`/`gate4.sh`: runtime, nuxie, scripting suites + golden-compare) | goldens and Silver are the correctness contract; bit-identical end-of-advance state |
| 7 | Per line inside an inlined hot function | `line_profile.py`, `inline_profile.py` | dSYM must match the profiled build |

Traps that cost days (all previously hit): fat-LTO inlining drift of ±1-3%
per commit on unrelated phases (judge a change by its target functions in a
PT diff); a `Cell` field inline in a hot `&self` struct removes `Freeze` and
reloads slice pointers every iteration; giving a hot helper a second call
site can outline it; short windows on Luau fixtures swing ±25% with GC
phase (use ≥3000 frames); never run two gate chains in one measurement
worktree; never poll with `pgrep` on your own command line.


Side finding: main's Rust benchmark runner stopped compiling at
`792aab98fb` (Oct 25; `ArtboardInstance::state_machine_handles` was removed
and `tools/rust-golden-runner/src/main.rs` still called it), so no count of
main after Oct 26 existed. `ca1739617d` on this branch is that one-line fix;
`perf-compare` also refuses corpus entries classified `diverges`, so counts
run on the 23 exact entries.

## 3. The map

### 3.1 Where each fixture's frame goes: fat-LTO count, 23 exact fixtures (data_bind_test_cmdq and hit_test_test are now classified diverges and perf-compare benchmarks only exact entries)

Totals: advance 2.652x, draw 2.566x, advance+draw **2.621x** C++; excess
14.42G advance + 7.92G draw per 10k frames. On the same 23 fixtures Oct 15
main was 1.641x and Oct 26 main 2.542x. Rust moved 1.656x (advance) and
1.541x (draw) since Oct 15 while C++ moved 1.004x / 1.019x; since Oct 26
Rust advance moved another 1.048x, almost all of it car (advance 1613M →
2539M, +57%).

| Fixture | adv C++ M | adv Rust M | ratio | draw C++ M | draw Rust M | ratio | excess M | Rust adv vs Oct 15 | Rust draw vs Oct 15 |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| zombie_skins | 4257.7 | 8270.2 | 1.94 | 288.9 | 613.4 | 2.12 | 4337 | 1.28x | 1.34x |
| car_widgets_v01 | 285.0 | 2539.2 | 8.91 | 1734.5 | 3752.8 | 2.16 | 4272 | 6.26x | 1.36x |
| data_viz_demo | 287.7 | 1836.5 | 6.38 | 741.9 | 2315.2 | 3.12 | 3122 | 3.28x | 1.72x |
| spotify_kids_demo | 1672.8 | 3130.5 | 1.87 | 295.2 | 593.9 | 2.01 | 1756 | 1.26x | 1.31x |
| component_list_1 | 242.4 | 1346.2 | 5.55 | 151.4 | 638.7 | 4.22 | 1591 | 2.16x | 2.14x |
| collapsing_elements | 151.1 | 595.7 | 3.94 | 209.9 | 869.9 | 4.14 | 1105 | 2.07x | 2.01x |
| virtualize_blendmode | 179.9 | 925.1 | 5.14 | 107.4 | 443.5 | 4.13 | 1081 | 2.15x | 2.09x |
| script_create_text_runs | 153.3 | 727.6 | 4.75 | 161.2 | 575.8 | 3.57 | 989 | 2.15x | 1.79x |
| layout_grid_stack | 176.8 | 534.1 | 3.02 | 97.8 | 232.2 | 2.37 | 492 | 1.74x | 1.63x |
| jellyfish_test | 67.0 | 97.4 | 1.45 | 130.7 | 546.2 | 4.18 | 446 | 1.25x | 2.13x |
| gamepad_test | 102.6 | 309.5 | 3.02 | 105.5 | 327.1 | 3.10 | 428 | 1.57x | 1.17x |
| multi_listeners | 101.7 | 332.5 | 3.27 | 108.7 | 272.6 | 2.51 | 395 | 2.10x | 1.75x |
| text_vertical_trim_test | 238.6 | 501.8 | 2.10 | 97.6 | 211.3 | 2.17 | 377 | 1.17x | 1.59x |
| audio_script | 82.8 | 231.8 | 2.80 | 127.3 | 301.2 | 2.37 | 323 | 2.13x | 1.37x |
| viewmodel_based_condition | 96.2 | 337.7 | 3.51 | 76.4 | 113.1 | 1.48 | 278 | 1.76x | 1.27x |
| image_scripting_property_value | 87.9 | 217.1 | 2.47 | 86.2 | 225.4 | 2.61 | 268 | 1.38x | 1.36x |
| clear_viewmodel_list | 76.4 | 171.3 | 2.24 | 84.5 | 192.6 | 2.28 | 203 | 1.55x | 1.24x |
| local_bounds | 77.0 | 169.5 | 2.20 | 98.3 | 204.2 | 2.08 | 198 | 1.64x | 1.53x |
| background_measure | 85.6 | 219.7 | 2.57 | 72.0 | 99.1 | 1.38 | 161 | 1.61x | 1.22x |
| script_dependency_test_using_library_v2 | 79.2 | 182.2 | 2.30 | 70.7 | 110.7 | 1.57 | 143 | 1.58x | 1.34x |
| script_dependency_test_using_library | 79.6 | 182.0 | 2.29 | 71.2 | 110.8 | 1.56 | 142 | 1.57x | 1.34x |
| library | 74.2 | 149.9 | 2.02 | 75.1 | 134.1 | 1.79 | 135 | 1.51x | 1.45x |
| library_with_text_and_image | 73.4 | 143.0 | 1.95 | 67.2 | 100.1 | 1.49 | 102 | 1.42x | 1.29x |

Three regression shapes are visible without a trace: (1) a floor of
~1.25-1.6x on every fixture, advance and draw alike, including jellyfish
and zombie (uniform per-access cost); (2) layout/list/nested fixtures at
2-3x on top of that (car, data_viz, component_list, collapsing, virtualize,
text runs); (3) car alone grew another 57% in advance after Oct 26 (the
state-holds series of Oct 30-31 is the only hot-path change in that window:
"a layer that owes its pose never settles").


### 3.2 Where the frame goes by subsystem: exact traces, Rust `ca1739617d` vs C++ `022a0fa9` (Processor Trace, self instructions per frame, bucketed by the leaf-most subsystem frame; `pt_buckets.py`, tables in perf-scripts/out/buckets/)

Zombie, 100 frames (trace total matches the 10k-frame count within 1.5%):

| advance bucket | Rust | C++ | ratio | excess |
| --- | ---: | ---: | ---: | ---: |
| dirt propagation | 214.6k | 174.8k | 1.23 | 39.8k |
| paths/geometry | 162.5k | 83.5k | 1.95 | 79.0k |
| transforms/update walk | 161.6k | 60.0k | 2.69 | 101.6k |
| animation apply | 88.1k | 29.2k | 3.02 | 58.9k |
| borrow scope (thread-local) | 50.0k | 0 | — | 50.0k |
| state machine | 38.5k | 7.4k | 5.21 | 31.1k |
| alloc | 30.5k | 26.5k | 1.15 | 4.0k |
| paints/effects | 29.1k | 40.5k | 0.72 | -11.4k |
| data bind | 17.9k | 1.2k | 15.1 | 16.7k |
| artboard root (walk self) | 17.4k | 0.7k | 24.4 | 16.7k |
| text | 13.3k | 0 | — | 13.3k |
| nested/host/quiet | 9.4k | 1.4k | 6.9 | 8.1k |
| **advance total** | **839.3k** | **425.9k** | **1.97** | **413.4k** |
| **draw total** | **57.2k** | **22.9k** | **2.50** | **34.3k** |

Car, 1500 frames:

| bucket | Rust | C++ | ratio | excess |
| --- | ---: | ---: | ---: | ---: |
| advance: data bind | 99.7k | 8.9k | 11.2 | 90.9k |
| advance: text (measure/shape per frame) | 70.8k | 6.8k | 10.4 | 63.9k |
| advance: layout (incl. borrow scope) | 35.1k | 0.5k | 67.9 | 34.6k |
| advance: alloc | 18.9k | 2.1k | 9.0 | 16.7k |
| advance: state machine | 12.4k | 2.6k | 4.7 | 9.8k |
| **advance total** | **263.1k** | **29.0k** | **9.09** | **234.1k** |
| draw: artboard root (inlined dispatch) | 220.6k | 0 | — | 220.6k |
| draw: text | 60.9k | 16.1k | 3.8 | 44.8k |
| draw: paints/effects | 37.4k | 71.5k | 0.52 | -34.1k |
| draw: layout (incl. borrow scope) | 23.5k | 0 | — | 23.5k |
| draw: paths/geometry | 19.0k | 41.1k | 0.46 | -22.1k |
| draw: nested/host | 2.0k | 32.9k | 0.06 | -30.9k |
| **draw total** | **369.2k** | **167.1k** | **2.21** | **202.1k** |

(C++ draw attributes shape work to `ShapePaint::draw`/`PathComposer` and
nested artboards; Rust inlines the same work into the artboard draw root,
so compare totals and the text/data-bind rows, not the paint/path rows.)

Direct self cost of the access machinery, same traces (leaf frames only;
everything inlined into callers is on top of this):

| Leaf | zombie /frame | share | calls/frame | instr/call | car /frame | share |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| `BorrowScope::new`/`drop` | 87.5k | 9.2% | 3,123 | 28 | 100.9k | 15.8% |
| `_tlv_get_addr` (macOS thread-local) | 19.6k | 2.1% | 2,447 | 8 | 28.7k | 4.5% |
| empty `VecDeque<Refresh>` drop per scope exit | 28.5k | 3.0% | 1,779 | 16 | 28.0k | 4.4% |
| `Rc` clone/drop | 57.9k | 6.1% | 1,387 | 42 | 79.2k | 12.4% |
| `as_any` + `downcast` | 28.9k | 3.0% | 70 | 410 | 43.2k | 6.7% |
| `CoreHandle::with`/`with_mut` self | 5.6k | 0.6% | 51 | 110 | 29.7k | 4.6% |
| `RefCell` borrow helpers | 1.5k | 0.2% | | | 29.8k | 4.7% |
| `checkpoint_live_value` | 8.0k | 0.8% | 473 | 17 | 7.2k | 1.1% |
| `read_slot`/`is_alive`/`core_type` | 3.5k | 0.4% | 184 | 19 | 5.5k | 0.9% |

So the Oct 23 scope alone is 14% of zombie's and 25% of car's in-loop
instructions as leaf self cost (the fast A/B says +25-70% including what
inlined around it), and the pre-existing handle tax (Rc traffic, downcasts,
wrappers, RefCell) is another 10-30%. Zombie's top leaves are the walk
(`update_components_handle` 121k), the cascade (`cascade` 101k +
`add_new_dirt` 34k), `invalidate_stroke_effects` 48k, keyed apply 39k and
`path_changed` 26k per frame.

Spotify, 100 frames (advance 319k vs 168k, 1.91x; draw 54.9k vs 23.9k,
2.30x): paths/geometry 2.05x (87k vs 43k), animation apply 2.85x (68k vs
24k), transforms 2.53x (51k vs 20k), dirt 0.84x (53k vs 63k: the cascade
batching that landed in PR #1072 is cheaper than C++ here), borrow scope
15.9k, alloc 1.95x, data bind 37x (6.1k vs 0.2k).

Data_viz, 300 frames (advance 374k vs 61k, 6.1x; draw 233k vs 68k, 3.4x):
layout 132k vs 12.7k, text 67k vs 15.7k, data bind 47k vs 5.3k, alloc 45k
vs 9.7k, borrow scope 35k, nested/host 8.6k vs 1.2k. Inside the layout
bucket Taffy itself is ~15k/frame (`compute_preliminary` 11.5k,
`generate_anonymous_flex_items` 3.5k); the other ~117k is Rust plumbing
around it: `LayoutComponent::advance_component_occurrence` self 17.7k,
borrow scopes 8.1k, `natural_bounds_for_layout_host` 15k, `measure_tree_node`
6.5k, `css_record` type queries (`is_type_of`) 5.9k, `with_style` leases
3.7k. Draw: `draw_own_paints` 12.3k, `draw_listed_layout` 7k + 3.8k scope +
2.8k visitor reads, `css_record` queries 5.6k again. So "CSS layout by
design" is at most ~15% of data_viz's layout excess; the rest is per-frame
re-measurement and handle traffic that C++'s Yoga path never does.

Component_list_1, 300 frames (advance 147k vs 25.6k, 5.7x; draw 58k vs
8.8k, 6.6x): state machine 36.7k vs 7.1k (5.2x; rows' machines never
settle), artboard root 29.7k vs 3.1k (per-row walks), borrow scope 18.7k,
data bind 16.6k vs 2.1k, layout 14.3k vs 2.6k, text 11.1k vs 3.8k. Draw
root 31k vs 0.4k, layout 10k vs 0.9k.

Jellyfish, 300 frames (advance 5.0k vs 0.7k, 7.2x; draw 48.6k vs 6.7k,
7.2x; its first 300 frames are much lighter than its 10k average, so these
are idle-frame overheads): the draw root pays a borrow scope per entry
(6.4k), and every image asks `Image::natural_size_for_layout` at draw time
(11.7k/frame: a `with` lease, a `pthread_mutex` lock/unlock pair inside
`RenderImageSizeState`, and an `Any::type_id` probe) although nothing in the
file is laid out. C++ does none of this on an idle frame.


### 3.3 The operation ledger: the same operation on both sides

Each row is one operation that exists in both runtimes, matched by
function. Columns: calls per frame on each side (equal calls mean the same
work is being done), inclusive instructions per call, and the ratio.
Source: `pt_ops.py` over the six traces (`out/ops/summary.txt`, per-row
child breakdowns in `out/ops/<fixture>.txt`). Rows whose Rust function is
inlined away (the transform, path, composer and stroke kernels inside the
walk) have no per-call figure here; their per-call cost comes from the
Oct 16 attribution build in 3.4.

Per-frame work (one call each):

| Operation | Fixture | Rust /call | C++ /call | ratio | Where the difference is (3.4) |
| --- | --- | ---: | ---: | ---: | --- |
| whole-frame advance (`advanceAndApply`) | zombie | 839,048 | 425,861 | 1.97 | everything below |
| | spotify | 319,120 | 167,458 | 1.91 | |
| | car | 263,031 | 28,638 | 9.18 | data binds, nested advance |
| | data_viz | 380,463 | 60,494 | 6.29 | layout, text, binds |
| | component_list | 147,330 | 25,146 | 5.86 | rows' state machines |
| state machine advance (no apply) | zombie | 331,515 | 161,963 | 2.05 | layer bookkeeping + apply |
| | car (4 machines) | 4,666 | 833 | 5.60 | per-layer bookkeeping |
| | component_list (9) | 4,028 | 772 | 5.21 | per-layer bookkeeping |
| update components (the dirty walk) | zombie | 389,988 | 235,045 | 1.66 | per-node plumbing, kernels inlined |
| | spotify | 161,844 | 109,546 | 1.48 | |
| | car | 13,468 | 6,886 | 1.96 | |
| | data_viz | 70,906 | 22,483 | 3.15 | layout nodes |
| | jellyfish (idle) | 2,013 | 414 | 4.87 | fixed per-walk cost |
| one artboard draw (`drawContent`) | zombie (7 artboards) | 8,142 | 3,270 | 2.49 | per-entry dispatch |
| | car (4) | 92,211 | 41,763 | 2.21 | shapes, text, images |
| | data_viz (7) | 32,352 | 9,759 | 3.31 | layout proxies, text |
| | component_list (9) | 6,376 | 981 | 6.50 | rows |
| | jellyfish (1, idle) | 48,439 | 6,691 | 7.24 | images |

Per-object operations (calls per frame equal on both sides unless noted):

| Operation | Fixture | calls/frame | Rust /call | C++ /call | ratio | Where the difference is |
| --- | --- | ---: | ---: | ---: | ---: | --- |
| apply one linear animation | zombie | 15 | 22,635 | 10,972 | 2.06 | per keyed object ~700 vs ~300 of plumbing |
| | spotify | 2 vs 7 | 51,161 | 7,969 | 1.83/frame | joystick animations applied through a different entry |
| | component_list | 8 | 1,123 | 146 | 7.67 | settled rows still resolve and lease |
| apply one keyed object | zombie | 159 | 2,087 | (inlined) | | self 244 + `resolve_object` 108 + property apply 1,727 |
| set a double property, incl. its dirt cascade | zombie | 182 | 1,384 | 768 | 1.80 | setter self 100 vs 19; cascade 1,233 vs 745 |
| | spotify | 119 | 695 | 292 | 2.38 | |
| | car | 3.4 | 379 | 84 | 4.50 | |
| dirt cascade, per frame | zombie | | 220,050 | 135,529 | 1.62 | ~45-79 vs 27 per visit (3.4) |
| | spotify | | 50,450 | 30,224 | 1.67 | |
| shape path changed | zombie | 395 | 262 | 170 | 1.55 | stroke-effect invalidation per shape 136 vs per effect 20 |
| | spotify | 78 | 278 | 159 | 1.75 | |
| invalidate stroke effects, per frame | zombie | 395 vs 1,095 | 136 | 20 | 2.41 | Rust once per shape, dearer; C++ once per effect |
| transform constraints | zombie | 15 vs 610 | 2,289 | 39 | 1.47/frame | Rust batches per constrained node |
| trim path effect | zombie | 129 vs 14 | 402 | 3,234 | 1.15/frame | different granularity, near parity |
| draw one shape | zombie | 45 | 423 | 285 | 1.48 | self 246 + composer path lease 164 vs self 55 + `ShapePaint::draw` 209 |
| | car | 498 | 289 | 175 | 1.65 | |
| | data_viz | 59 | 535 | 281 | 1.91 | |
| | jellyfish | 2 | 334 | 169 | 1.98 | |
| draw one text | car | 67 | 1,022 | 365 | 2.80 | `Text::draw` body 483 + wrappers 160 + scope 59 + checkpoint 20 + downcasts; geometry 425 vs 271 |
| | data_viz | 44 | 942 | 341 | 2.76 | |
| | component_list | 8 | 940 | 341 | 2.76 | |
| text style geometry draw | car | 71 | 425 | 271 | 1.57 | |
| draw one image | car | 5 | 1,524 | 100 | 15.2 | `transform_raster` 252, sampler conversion, scope; plus `natural_size_for_layout` 574 per draw (C++ 0) |
| | jellyfish | 22 | 1,516 | 182 | 8.33 | |
| `Shape::willDraw` | data_viz | 72 vs 59 | 110 | 19 | 5.80 | |
| advance one artboard (`advanceInternal`) | zombie | 14 | 4,200 | 756 | 5.55 | `AdvancingComponentHandle::advance` dispatch per advancing component |
| | spotify | 2 | 570 | 59 | 9.66 | |
| | car | 8 | 28,160 | 2,374 | 11.9 | data binds (below) |
| | data_viz | 14 | 10,186 | 1,266 | 8.05 | binds 2,478 + advancing components 7,641 |
| | component_list | 18 | 5,842 | 831 | 7.03 | |
| nested artboard advance | zombie | 12 vs 24 | 4,796 | 432 | 5.55/frame | nested animation dispatch 72% |
| | car | 6 vs 12 | 36,882 | 1,549 | 11.9/frame | |
| data bind container update | car | 200 vs 12 | 604 | 32 | 304/frame | `10ba454b9b`: every converter walks its container; `DataBind::advance_handle` 574 |
| | data_viz | 73 vs 21 | 491 | 132 | 12.8/frame | |
| | component_list | 27 vs 35 | 56 | 94 | 0.46/frame | gated containers are cheap |
| converter base advance | car | 300 vs 112 | 167 | 22 | 20.4/frame | wrapper self 60 + closure 35 + scope 20 per call |
| converter group advance | car | 148 vs 38 | 154 | 203 | 2.96/frame | |
| converter interpolator advance | car | 38 | 81 | 66 | 1.23 | near parity |
| layer `updateState` | zombie | 22 | 1,751 | 227 | 7.73 | `try_change_state_from` 1,149 (find_allowed_transition 542) + `is_settle_safe` 124 + `is_transitioning` 108 + `mark_view_model_changes_searched` 99 + `ensure_any_state_instance` 85 vs `tryChangeState` 174 |
| | component_list | 17 | 1,757 | 228 | 7.72 | |
| | car | 9 | 943 | 62 | 15.2 | |
| | data_viz | 7 | 582 | 43 | 13.6 | |
| component list advance | component_list | 2 vs 4 | 51,034 | 3,636 | 7.02/frame | rows' state machines 25k, `advance_internal` 10k, row closure 5k |
| component list draw | component_list | 1 | 51,975 | 8,199 | 6.34 | `draw_hosted` 43.6k (rows); list field reads through handles 3k |
| layout solve (Taffy vs Yoga) | data_viz | 0.013 | 11.4M | 1.21M | 9.43 | Taffy `compute_layout_with_measure` 10.2M incl. measure callbacks; `sync_minimum_contracts` 540k; `adopt_layout_node` 506k per solve |
| | component_list | | 1.78M | 0.93M | 1.92 | |
| | zombie | 0.1 | 17,221 | 2,445 | 7.04 | |
| `Text::measure` | data_viz | 3.2 vs 0.2 | 7,152 | 52,530 | 2.59/frame | Taffy measures 19x more often; each measure reshapes (`make_styled` 2,788, `shape_text` 1,705, `break_styled_lines` 944) |
| HarfBuzz shaping (`shapeText`) | data_viz / car | | 34,250 / 20,820 | 30,505 / 21,666 | 1.12 / 0.96 | parity |
| `Text::buildRenderStyles` | data_viz / car | | 77,751 / 27,742 | 58,297 / 16,687 | 1.33 / 1.66 | |
| layout draw proxy | data_viz | 32 vs 64 | 1,270 | 185 | 3.43/frame | proxy body 1,101 vs `ShapePaint::draw` 126 |

### 3.4 Why each operation costs more: the two call paths

For each hot operation, what the C++ side does and what the Rust side does
instead, with the measured per-call pieces. "scope" means the Oct 23
`picture_layout_mode::BorrowScope` (28 instr + 8 for `_tlv_get_addr` + 16
for an empty `VecDeque` drop per access; 3.6 item 1).

**Apply one keyed property (zombie, 182 writes/frame).** C++:
`LinearAnimation::apply` → `KeyedObject::apply` (inlined) → `KeyFrameDouble::apply`
309 → `applyInterpolation` 385 → `CoreRegistry::setDouble` 19 self → the
setter's `addDirt`. Rust: `LinearAnimationOwner::apply` → `apply_keyed_object_entry`
→ `KeyedObject::apply_cells` (self 244: cells read, interpolation host
check) → `resolve_object` 108 (handle lookup by id each frame) →
`KeyedProperty::apply_cells` (closure, keyframe search and interpolation) →
`set_double_handle` self 100 (key dispatch, scope, checkpoint probe,
completion) → `set_plain_transform_double` → `add_new_dirt`. Per keyed
object Rust spends roughly 700 instructions of plumbing against C++'s
~300; the arithmetic is the same. The unmerged compiled-tracks commit
(`a988a9909e`) resolves targets once and writes typed sinks: zombie advance
-6.1% measured.

**Mark one dirt edge (zombie, ~1,240 edges/frame).** C++: `addDirtToDependents`
109 per visit inclusive, 46 self: load dependents vector, OR the bits,
virtual `onDirty` when new. Rust: `ComponentGraph::cascade` 767 self per
cascade over ~10 edges + `mark` 1,077 per cascade: per visit kind
dispatch, liveness check (`is_alive`), notification batching, plus the
scope on every `with` reached from `onDirty`. Attribution build on the
Oct 16 tree measured ~45/visit vs 27 before the scope was added; today's
per-frame ratio is 1.62. `Shape::path_changed` 262 vs 170 because Rust
invalidates stroke effects per shape (136) where C++ does it per effect (20).

**Update one node in the walk (zombie, 966 nodes/frame, 959 dirty).** C++:
`updateComponents` self 15.5k per frame plus virtual `update` per dirty
node: `TransformComponent::update` 137, `Node::updateWorldTransform` 39,
`updateConstraints` 39, `Shape::update` 108, `Path::update` 212,
`PathComposer::update` 298, `Stroke::update` 428, `PointsPath::update` 92.
Rust: `update_components_handle` self 114k per frame (the kernels are inlined
here) plus `update_entry` dispatch per node reading the member through its
handle, cells behind liveness checks, and the scope. The Oct 16 attribution
build (hot helpers `#[inline(never)]`) measured the transform kernel at 48 +
21 per update and the walk's own cost ~3-4% of advance; today's walk-level
ratio is 1.66 (zombie) to 3.15 (data_viz), the extra being per-node handle
reads and layout nodes.

**Draw one shape (car, 498/frame).** C++: `Shape::draw` self 55 →
`ShapePaint::draw` 209 (`renderPath` 55, effect path 4, renderer calls ~6).
Rust: `ShapeCells::draw` self 211-246 → `composer_path_mut` lease 52-164 →
`PaintCells::draw_plain` closure → renderer. The cells path already skips
the generic handle lease; the remaining +138 per shape is the draw-list
entry read, the range/visitor state, the composer path borrow and the scope.

**Draw one text (car, 67/frame).** C++: `Text::draw` self 76 →
`TextStylePaint::drawGeometry` 287. Rust: `Text::draw` closure 483 +
wrapper self 160 + scope 59 + `checkpoint_live_value` 20 + two downcasts
→ `draw_geometry` 425. Text paints are not on cells yet (the mutator
visibility cell is missing), so every style and paint is reached through a
leased generic handle.

**Advance one artboard (car, 8/frame).** C++: `advanceInternal` self 23 →
`NestedArtboard::advanceComponent` 2,322 → nested `advanceInternal`,
`DataConverterGroup::advance` 203 per group (38 groups), `Text::advanceComponent`
30 per text. Rust: `advance_internal_handle` →
`AdvancingComponentHandle::advance` 28,047 per artboard: the data-bind
containers (3.3), `nested_animation_advance_handle`, and per advancing
component a handle resolve, a scope and a closure. For zombie's 12 idle
nested artboards it is 4,796 per nested advance against 432.

**Update one state-machine layer (component_list, 17 layers/frame).** C++:
`updateState` self 52 → `tryChangeState` 174 (walk transitions, evaluate
conditions). Rust: `update_state` self 90-186 → `try_change_state_from`
1,053-1,310 (`find_allowed_transition` 303-542, the rest is transition
instance resolution and condition reads through handles) +
`is_settle_safe` 124 + `is_transitioning` 108 +
`mark_view_model_changes_searched` 99 + `ensure_any_state_instance` 85.
Four of those five are Rust-only per-frame bookkeeping. The unmerged
input-condition settling and frozen-layer commits removed most of it on
rows that do not change (component_list advance -18% after the golden
correction).

**Advance one data bind container (car, 200/frame).** C++: `updateDataBinds`
32 per container, 12 containers per frame: the artboard owns the binds.
Rust after `10ba454b9b`: every converter advances its own container
(`advance_base_handle` 167 per call, 300 calls), groups clone their item
vector per frame and lease each item, `DataBind::advance_handle` 574 per bind
reaches the converter group through a `with`, a scope and a boxed value.
Per frame 121k against C++'s 0.4k; car's advance is this.

**Solve one layout (data_viz, one solve per ~75 frames; 3.2 text measures
per frame).** C++ (Yoga): `calculateLayoutInternal` 1.21M per solve,
`Text::measure` 0.2 per frame at 52,530 (shapes once). Rust (Taffy 0.15,
CSS semantics by decision): `compute_layout_with_measure` 10.2M per solve
because Taffy asks `Text::measure` several times per node per solve
(19x the C++ measure count; each measure re-shapes: `make_styled` 2,788,
`shape_text` 1,705, `break_styled_lines` 944), plus `sync_minimum_contracts`
540k and `adopt_layout_node` 506k per solve, plus the per-frame layout
bucket of 132k (`advance_component_occurrence` self 17.7k,
`natural_bounds_for_layout_host` 15k, `measure_tree_node` 6.5k, `css_record`
type queries 5.9k, `with_style` leases 3.7k, scope 8.1k). The Taffy
arithmetic itself is ~15k per frame. The unmerged measure memo keyed on the
styled input (`35b4926d01`) measured data_viz advance -19.5%.

**Draw one image (jellyfish, 22/frame).** C++: `Image::draw` 100: renderer
`drawImage` with the cached render image. Rust: `draw_occurrence` 1,516:
`transform_raster` 252, sampler conversion 18, scope 32, the asset loan, and
`natural_size_for_layout` 574 per draw (a `with` lease, a `pthread_mutex`
lock/unlock in `RenderImageSizeState`, an `Any::type_id` probe) although the
file has no layout.

### 3.5 The access model: what one object touch costs on each side

C++ (`rive-runtime`): every object is a heap `Core*`; parents, children,
dependents, keyed targets, and paints are raw pointers; a property write is a
field store plus a virtual `onDirty`/`addDirt`; `is<T>()` is an integer
`coreType` compare. One access costs a load and a possible virtual call.

Rust (`crates/nuxie-runtime/src/mechanical_port/source/core.rs`): every
object reference is a `CoreHandle { Rc<CoreHandleIdentity> }`; the identity
holds a `CoreArenaSlot` with `RefCell<Option<Box<dyn CoreObject>>>`,
generation cells, retirement state, and a dozen side fields. One typed
access (`with_downcast::<T,_>`) is: Rc deref → generation compare → thread-local
`picture_layout_mode::BorrowScope` enter (since `0588c46b1b`, Oct 23) →
`RefCell` borrow flag → `Box<dyn>` deref → `as_any` vtable call → `downcast_ref`
TypeId compare → closure → release guard (`retirement_pending` check) →
`BorrowScope` drop (thread-local again, pending-refresh check). Mutable
access adds `host_viewmodel::checkpoint_live_value` (an `as_view_model_instance_value`
vtable probe) and a component-list type check.

Counts in the runtime source today: 1,306 `.with(` sites, 955 `.with_mut(`,
1,840 `.with_downcast*`, ~3,800 `as_*()` probe calls, 446 `Box<dyn`, 542
`Rc<`, 317 `RefCell<`.

Measured per-access costs (PT, Oct 10-16, instructions):

| Access | Rust | C++ equivalent | Where measured |
| --- | ---: | ---: | --- |
| `RuntimeArtboardObjectContext::resolve_handle` (keyed target) | 77 | 8 | zombie, 2026-10-10 |
| `CoreHandle::is_alive` | 42 (now lease-free unless retiring) | 0-2 | zombie |
| authored scalar read through a handle | 40-60 | 1-3 | small fixtures steady state |
| dirt cascade, per visit | 79 → 45 after inlining the mark | 27 | zombie, before/after `f5afc885d4` |
| `KeyedObject::apply_occurrence`, per keyed object | 285 → ~110 (compiled tracks) | ~55 (841 per animation) | zombie |
| data bind apply, per bind | ~2,600 | ~270 | virtualize_blendmode |
| per-shape draw | ~425 | ~260 | car |
| `component_graph()` Rc clone per setter write | 31 | 0 | zombie |

Ceiling experiment (unsound, never shipped, branch
`claude/access-ceiling-experiment`): removing only the `RefCell` flag work and
the release guard from `with`/`with_mut` cut 12.3% of instructions. Borrow
checking is not the bulk; the rest of the tax is generation checks, Rc
clones, vtable probes, closures, and the thread-local scopes.


### 3.5a Per-site cost of one access, both sides (summary)

| Step | C++ | Rust today |
| --- | --- | --- |
| reach the object | pointer load | `Rc` deref + generation compare |
| know it is alive | implicit | `occupied`/`retirement_pending` cells (lease-free since Oct 10) |
| borrow it | none | `RefCell` flag set/clear; release guard checks retirement |
| know its type | `coreType()` integer compare | `as_any` vtable call + `TypeId` compare (`with_downcast`), or `as_*` probe |
| run the code | direct or one virtual call | closure through `FnOnce`, often two levels (`with` + `with_downcast`) |
| extra since Oct 23 | none | thread-local scope enter/leave (28 + 8 + 16) |
| extra since Oct 23, mutable | none | `checkpoint_live_value` vtable probe (17) |

### 3.6 The regression ledger: what changed since Oct 15

Window: main `c3a4b6092f` (Oct 15, 1.641x) → `2509c5d0b8` (Oct 26, 2.558x),
407 first-parent commits; C++ moved 1.01x, Rust 1.575x. Per-fixture deltas
in 3.1. Findings, by evidence strength:

1. **CONFIRMED** `0588c46b1b` (Oct 23): `picture_layout_mode::BorrowScope`
   on every `CoreHandle::with`/`with_mut`/downcast/`with_artboard`. Fast
   counts (1000 frames, min of 5) of its parent `cad3daf08e` vs itself,
   perf-scripts/out/fast-pics-before.txt vs fast-pics-after.txt:

   | Fixture | advance | draw |
   | --- | ---: | ---: |
   | zombie_skins | +25.4% | +39.7% |
   | spotify_kids_demo | +21.1% | +30.0% |
   | car_widgets_v01 | +70.0% | +30.9% |
   | component_list_1 | +102.2% | +77.3% |
   | data_viz_demo | +59.5% | +53.6% |

   Band-aid measured (`fast-head-oct31` → `fast-head-bandaid`: the same
   accounting on `const`-initialized `Cell`s, no `RefCell`, no lazy init, the
   cold completion path out of line; patch perf-scripts/out/pics-bandaid.patch,
   applied uncommitted in this worktree): zombie advance -11.0% / draw -14.9%,
   spotify -8.7% / -10.2%, car -25.4% / -8.4%, component_list -29.6% /
   -24.7%, data_viz -23.2% / -17.7%. Against the Oct 23 parent the band-aid
   tree is still +8% zombie / +8% spotify advance and +13-15% draw, which is
   the remaining inc/dec per access plus the other commits since; the
   boundary-drain redesign removes the rest. Exactness: the band-aid
   runner's scripted golden compare is 348 exact + 16 classified diverges of
   364, same as main; the one reported failure is corpus bookkeeping
   (`semantic_text_inference_lt1` "is now exact; remove the reviewed
   signature"), which is pre-existing. Runtime lib layout unit tests: 474
   passed, 2 failed (`css_rounded_viewport_passes_corner_hits_through`,
   `css_animated_radius_clips_child_hits_live`, both Picture hit tests);
   both fail identically on the unpatched tree, so they are pre-existing
   main failures, not the band-aid's.

   One commit is the bulk of the Oct 15→26 regression. For scale, the whole
   Oct 15→Oct 23 window before it moved zombie +6%, spotify +6.5%, car +18%,
   component_list +4.6%, data_viz +40% in advance (the CSS layout series)
   and +1..+22% in draw. The scope costs a lazily initialized thread-local
   `RefCell` borrow on entry and on exit per access, times the ~2.3k handle
   access sites that run millions of times per frame.
2. `fa5e4995ae` (Oct 23, live moves): `checkpoint_live_value` on every
   `with_mut`; `f96388b1f4` (Oct 24): component-list type check on every
   `with_mut` exit.
3. CSS layout series (Oct 17-25, ~60 commits touching `layout_component.rs`,
   `css_layout/`): layout fixtures' advance 2-4x. Expected partly by design
   (CSS semantics), but idle-layout quiescence may have been lost.
4. Upstream ports of Oct 25 (`faa6acff97` global text styles,
   `dc77e5a99b` artboard lazy source storage, `792aab98fb` artboard sharing):
   text_vertical_trim 1.1x → 2.1x (C++ itself got 38% cheaper there, so
   part of that ratio is C++ moving, not Rust).
5. **PINNED** `10ba454b9b` (Oct 28, "port recursive converter data
   bindings"): the five-point sweep of Oct 26-30 (`fast-w2-*`, car and
   data_viz, 1000 frames) is flat to ±0.5% at every point except this one
   commit: car advance 188.1M → 270.7M (+43.9%), data_viz 221.1M → 244.3M
   (+10.5%), draw unchanged. State holds (Oct 30-31) add only car +3.4%.
   Why: every converter now advances its own data-bind container every
   frame (`DataConverter::advance_base_handle` → `advance_data_binds`),
   groups clone their item `Vec<CoreHandle>` per advance
   (`advance_group_items`) and lease each item, and interpolators advance
   their binds before themselves. Upstream owns the same recursion with
   pointer walks; here each level costs leases and clones even when the
   container holds no binds. Fix: a per-converter "has advancing binds"
   gate (the UNMERGED `7f1c36e46c`/`5de7f16bd3` pattern: close the advance
   gate while converters rest) and iterate items by borrow, no clone.
6. Oct 23 → Oct 26 (`fast-pics-after` → `fast-main-oct26`): car advance
   +57%, data_viz +19% (live moves, upstream ports of Oct 25); zombie,
   spotify and component_list -1..-3%. So car's advance regressed in three
   steps: CSS layout (+18%), the borrow scope (+70%), live moves (+57%),
   state holds (+48%), compounding to 4.7x the Oct 15 cost.
7. Live-move checkpoint on every `with_mut` (`fa5e4995ae`): removing the
   call (`fast-head-nocheckpoint`) saves 0.7-2.9% advance and 0.4-1.9%
   draw on the five fixtures. Real but second-order; the `as_view_model_instance_value`
   vtable probe could be a `core_type` key test.

The ceiling build with every `BorrowScope` deleted (`fast-head-nopics-all`)
did not count: the runner panics (`RefCell already borrowed` in `with_mut`)
because the deferred refresh then runs under a held receiver, which is the
bug the commit fixed; hence the band-aid measurement in item 1.

Bisect protocol used: `fastcount.sh` at HEAD, HEAD minus suspect, Oct 26,
Oct 23 before/after `0588c46b1b`, Oct 15, on zombie, spotify, car,
component_list, data_viz (1000 frames, min of 5), then a five-point sweep
of Oct 26-30 on car and data_viz.


## 4. The menu

Every opportunity the map exposes, sized by the excess instructions it
accounts for, with the cause, what eliminating it would recover, the
evidence, and the cost and risk of the fix. Sizes are per frame on the
fixture where they were traced; corpus shares are estimates from the
fat-LTO per-fixture excess (22.3G per 10k frames across the 23 fixtures:
zombie 4.34G, car 4.27G, data_viz 3.12G, spotify 1.76G, component_list
1.59G, collapsing 1.10G, virtualize 1.08G, text runs 0.99G, the rest 0.1-0.5G
each). Nothing here is scheduled; the order is by size.

| # | Opportunity | Size (measured) | Cause (map ref) | If eliminated | Evidence | Fix cost / risk |
| --- | --- | --- | --- | --- | --- | --- |
| M1 | Per-access thread-local borrow scope | 14% of zombie's and 25% of car's frame as leaf cost; fast A/B of the commit: +21-102% advance, +30-77% draw; est. 5-7G of the 22.3G corpus excess | `0588c46b1b` enters/leaves a lazily initialized thread-local `RefCell` on every `with`/`with_mut`/artboard access (3.5, 3.6 item 1) | back to the Oct 23 parent's cost on every fixture: zombie advance -20%, component_list -50%, draw -25..-45% | band-aid measured: advance -9..-30%, draw -8..-25%, golden-exact, tests unchanged (3.6 item 1); the rest needs draining at boundaries | S for the band-aid (patch exists, uncommitted); M for the boundary drain (4 request sites, one invariant to prove); low risk |
| M2 | Recursive converter advance walking empty bind containers | car +82k/frame (+44% advance), data_viz +23k (+11%); est. 1.05G corpus | `10ba454b9b` (3.3 data bind rows, 3.4) | car advance back to ~188M/1000 frames; data_viz -10% | five-point sweep, flat except this commit (3.6 item 5); the gate pattern exists unmerged (`7f1c36e46c`, `5de7f16bd3`) | S-M; low risk if advance order is kept |
| M3 | Per-artboard / per-advancing-component fixed cost | `advanceInternal` 5.5-11.9x per artboard; car 225k of its 263k advance is nested-artboard advance; zombie 12 idle nested artboards 4,796 each vs 432 | `AdvancingComponentHandle::advance` resolves, scopes and closures per advancing component every frame; idle hosts re-asked every frame (3.4) | UI fixtures' advance 2-4x → ~1.5x (est.); overlaps M2, M4 | unmerged quiet-nested-hosts commit: text runs -21%, data_viz -3%, car +5% without backoff | M; exactness net exists (verify-every-skipped-pass in testing builds) |
| M4 | State-machine layer bookkeeping per frame | `updateState` 1,751 vs 227 per layer (7.7x), 13-16x on UI fixtures; component_list 37k/frame vs 7k; rows never settle | `try_change_state_from` through handles + four Rust-only per-layer checks (3.4) | component_list advance -18..-49%; every fixture with machines -2..-5% | unmerged input-condition settling + frozen-layer verdict (golden-corrected) | M; risk: over-settling (the `settled=` flag is in the golden stream and catches it) |
| M5 | Animation apply plumbing | ~700 vs ~300 instr per keyed object; zombie ~60-80k/frame, spotify ~45k; est. 1.2G corpus | per-apply handle resolve, cells read, keyframe closure, setter dispatch (3.4) | zombie advance -6%, spotify -7% measured; more with typed sinks for colors/vertices | unmerged compiled tracks `a988a9909e` | M (landed once, needs a rebase); risk: lazily built keyframe holders (golden caught once) |
| M6 | Dirt cascade per visit | 45-79 vs 27 per visit; zombie 84k/frame excess, spotify 20k; est. 1.0G corpus | per-edge kind dispatch, liveness, notification batching (3.4) | zombie advance -10% (est.); fusing into the walk measured +5.6% (dead end), precomputed reach ranges unfinished | attribution build; `75942ad4dc` inline mark -2%; fp4/fp5 prototypes | M-L; exactness: onDirty handlers must commute |
| M7 | Update walk per-node plumbing | walk 1.66x (zombie) to 3.15x (data_viz); zombie 155k/frame excess, of which kernels ~0 | per-node member read through the handle, cells behind liveness, scope, `component_graph()` Rc clone per write (3.4) | zombie advance -15% (est.) with SoA transform columns written by compiled tracks | DOD design in perf-scripts/dod-notes.md; graph spike beat C++'s pointer graph by 30% in isolation | L (representation change); constraints (IK, follow-path) in the column pass |
| M8 | Draw dispatch per entry | per shape +138 (1.48-1.98x), per text +657 (2.8x), per image +1,424 (15x); car draw 202k/frame excess, data_viz 165k, jellyfish 42k; est. 7.9G corpus draw excess in total, ~3G of it this | draw root self 62k per car frame, composer-path lease per shape, leased generic path for text/image paints, scope per entry (3.4) | car draw -30%, data_viz -40% (est.); retained replay for resting artboards measured data_viz draw -71%, car -38% | unmerged `44a3b9fd71` (retained draws), cells for text paints need a mutator-visibility cell | M for cells; L for replay (keyed on content revision + write epoch; testing builds verify every replay) |
| M9 | Layout re-measure and per-frame layout plumbing | data_viz layout 132k/frame vs 12.7k; one solve 11.4M vs 1.2M; `Text::measure` 19x more calls | Taffy asks measure per node per pass and each measure reshapes; `sync_minimum_contracts`/`adopt_layout_node` per solve; css_record type queries and style leases per frame (3.4) | data_viz advance -27% (idle layouts from flags) and -19.5% (measure memo) measured on the Oct 16 tree; Taffy's own arithmetic ~15k/frame is not the problem | unmerged `f9fd2bf5a1`, `35b4926d01` | M; CSS semantics stay (decision 2026-10-14) |
| M10 | Text draw path and render styles | per text 1,022 vs 365; `buildRenderStyles` 1.33-1.66x; car 45k/frame, data_viz 32k | style/paint reached through leased generic handles; scope + checkpoint + downcasts per text (3.4) | car/data_viz draw -10% (est.) | cells for layout-component and text-style plain paints landed (`f72be2ff7c`); mutator visibility cell missing | M; low risk |
| M11 | Image draw and natural-size query | 1,516 vs 100-182 per image; jellyfish 33k/frame of 48k | `natural_size_for_layout` per draw through a mutex; `transform_raster` per draw; sampler conversion (3.4) | jellyfish draw -60% on idle frames (est.) | trace leaves | S; low risk |
| M12 | Rc traffic, downcasts and wrappers (the handle tax itself) | Rc clone/drop 6-12% of a frame, `as_any`/downcast 3-7%, `with` wrappers and `RefCell` 1-9% (3.5) | type-erased handle model; every typed access is Rc + generation + RefCell + vtable + closure | the remaining ~1.3-1.5x after M1-M11 (est.); only a representation change reaches 1.0x | ceiling experiment: RefCell alone 12%; per-object cells beat per-site estimates ~2x where applied | L (plan 001 / DOD core); exactness contract: bit-identical end-of-advance state |
| M13 | Allocation per frame | car advance alloc 18.9k vs 2.1k, data_viz 45k vs 9.7k, zombie 30.5k vs 26.5k | boxed bind values per convert, group item vectors cloned per frame, effect buffers | 2-5% on UI fixtures (est.) | trace leaves; Codex audit 002 priority 1 | S-M |
| M14 | Live-move checkpoint probe on every mutable access | 0.7-2.9% advance, 0.4-1.9% draw | `fa5e4995ae` vtable probe per `with_mut` (3.6 item 7) | that much | fast A/B with the call removed | S; low risk |
| M15 | Idle-frame walks on quiet content | jellyfish early frames 7.2x in both phases; small fixtures 1.5-3.5x | per-frame walks that cost even when nothing changed: draw root per entry, image size queries, layout re-measure, rows' machines | the quiescence family of M3/M4/M8/M9 | unmerged commits | covered above |

Reading the menu against the headline: M1 + M2 + M14 undo the Oct 15→31
regression (1.64x → 2.62x). M3-M11 are the unmerged branch's territory
(1.64x → 1.24x on the Oct 16 tree) plus what the traces found since. M12 is
the structural remainder and the only item that needs a new representation.

## 5. Open questions the map does not answer yet

- The per-kernel costs inside the walk (M7) are from the Oct 16 attribution
  build; a fresh `#[inline(never)]` build on today's tree would give the
  current split between kernels and plumbing.
- Jellyfish's trace covers its first 300 frames, which are far lighter than
  its 10k-frame average on both sides; a steady-state trace (`PT_START`)
  would size M11 on busy frames.
- Scripted fixtures (gamepad, audio_script, image_scripting, text runs) are
  counted but not traced today; the Oct 15-17 bridge work (`luau-scripted-perf`)
  is the evidence for them.
- Whether the `BorrowScope` can be scoped to the four request sites without
  counting borrows at all (M1's second half) is a design question, not a
  measurement.

## Appendix: tools and files

All under `~/dev/worktrees/perf-scripts/` unless given a repo path.

- `count.sh`, `measure.sh`, `compare_counts.py`: fat-LTO counts and
  per-fixture comparison; `out/main-oct31.txt`, `out/instr-nx17.txt`.
- `fastcount.sh`: fast-profile count of any commit, `PATCH_CMD=` for ceiling
  experiments; `ab_diff.py`; `out/fast-*.txt`.
- `pt.sh`, `pt_buckets.py` (subsystem buckets, `out/buckets/`), `pt_ops.py`
  (operation ledger, `ops.json`, `out/ops/`), `pt_tree.py`, `pt_diff.py`.
- `golden_fast.sh`: scripted golden compare with a fast runner.
- `out/pics-bandaid.patch`: the M1 band-aid, also applied uncommitted in this
  worktree.
- `out/perf-progress.html`: the progress chart (Oct 26 and Oct 31 points added).
