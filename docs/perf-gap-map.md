# Rust vs C++ runtime performance: map and menu

Status: 2026-10-31. Main `b9d4c38b07`, C++ pin `022a0fa9`; every Rust
number is measured at `ca1739617d` (main plus a one-line runner compile fix,
section 2). Measured numbers cite their source file under
`~/dev/worktrees/perf-scripts/out/`; estimates are marked.

Extended 2026-11-10 in two rounds:

- **Measurements taken 2026-11-01:** the counter-read correction (2.1),
  per-window counts (2.2), the allocation map (2.3, 3.7).
- **Measurements taken 2026-11-10 on main `6cc1a07141`** against C++
  `cd5351c3`, the pin main now names:
  - the frame-0 correction (2.4)
  - zombie, spotify, data_viz steady state and the idle-frame map (3.2)
  - the walk split (3.4)
  - the script bridge on both sides (3.8)
  - a bucketing fix that moves C++ path work out of "dirt" (3.2)

  Every Nov 10 Rust number carries the M1 band-aid
  (perf-scripts/out/pics-bandaid.patch, applied uncommitted). The M1/M2 fix
  PRs do not exist yet, and main without it measures the BorrowScope, not
  the runtime.

Those numbers name their tree where it differs from the above; section 5
lists what is still open.

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
| 2026-10-31 | same counts, net of the counter read (2.1) | 2.99x | 3.22x | **3.07x** | out/net-main-oct31.txt |

Every row above includes, on both sides, one instruction-counter read
(~6,470 instructions) per phase per frame; it is harness, not runtime
(2.1). The gross rows stay comparable with each other; the net row is the
runtime gap. The absolute excess (22.3G per 10k frames) is the same either
way; only the C++ denominator shrinks, from 13.79G to 10.81G.

What the map shows, in one screen:

- Rust runs the same operations the same number of times as C++ (every
  ledger row in 3.3 with matching call counts) and pays more per operation:
  1.5-2x on geometry-adjacent work, 2-3x on animation and draw, 5-15x on
  per-artboard, per-layer and per-bind bookkeeping.
- Net of the counter read, the small fixtures are not near parity: 19 of
  the 23 run 6-14x C++ in advance (all but zombie, spotify,
  layout_grid_stack and text_vertical_trim_test), and the fixtures where C++ does almost
  nothing per frame (library, jellyfish advance) show the largest ratios.
  That is the fixed per-frame cost of the Rust frame (walks, scopes, image
  and layout queries) on content that does not change (3.1, M15).
- Today's 2.62x is 1.64x of structural cost plus two commits: `0588c46b1b`
  (Oct 23) taxes every object access through a thread-local scope (+21-102%
  advance, +30-77% draw per fixture), and `10ba454b9b` (Oct 28) walks every
  converter's bind container every frame (car advance +44%, data_viz +11%).
- The structural 1.64x is the handle model: each object touch pays Rc
  traffic, a generation check, a `RefCell` flag, a vtable probe and a
  closure (3.5), multiplied by walks that C++ does with pointer loads.
- Geometry kernels (path building, skinning, stroking, HarfBuzz shaping)
  are at or near parity. Nothing in the map says Rust is slow at arithmetic.
- Nov 10 traces of current main (band-aid applied, frame 0 subtracted)
  confirm the shape three more ways:
  - Zombie's update walk is 1.44x C++ (345.8k vs 240.8k per frame). Its
    kernels run at 1.15x as a whole; the excess is per-node dispatch,
    `update_entry` ~78 per node vs ~17 (3.4).
  - On gamepad the Luau interpreter runs within 1.3x of C++, while the
    Rust bridge around it is 13x in draw: a draw call into Lua costs 21.0k
    vs 3.9k (3.8).
  - An idle frame costs 6-8x C++ because every per-frame pass runs its
    plumbing before its dirt check (3.2).
- Two unmerged branches matter differently. `claude/dod-compiled-animation`
  (the data-oriented core, 1.24x on the Oct 16 tree) was an experiment:
  its commits are evidence for menu items, not a landing plan. The Luau
  bridge commits on `claude/luau-scripted-perf` (14 commits after
  `3ad92ca574`, 27 files, all in `vendor/luaur-rt-0.1.8`, `vendor/luaur-vm-0.1.8`,
  `crates/nuxie-scripting` and the luaur test harness, no runtime files)
  are wanted and separable from the data-oriented base they sit on (M16).

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


### 2.1 The counter read inside every phase (found 2026-11-01)

Both golden runners count a phase as `proc_pid_rusage(RUSAGE_INFO_V4)`
before and after it (`tools/golden-runner/main.cpp` `processInstructions`,
`tools/rust-golden-runner/src/main.rs` `process_instructions`), and
`ri_instructions` includes the kernel instructions of the read itself. One
read retires 6,421-6,504 instructions (three rounds of 10,000 reads,
`perf-scripts/probe/rusage_cost.c`); an empty phase measures 6.4k. So every
counted phase of every frame carries about 6,470 instructions of harness,
on both sides: 12.9k per frame, 129M per 10k frames per fixture.

Checked against Processor Trace on the same C++ binary and the same frames
(trace phases carry no counter reads): jellyfish 300 frames, count 7.45k
advance and 13.25k draw per frame vs trace 0.70k and 6.72k; car 1500
frames, count 37.1k / 174.2k vs trace 29.0k / 167.1k; zombie 100 frames,
count 443.1k / 30.0k vs trace 425.9k / 22.9k. Draw differs by 6.4-7.1k
everywhere; advance by 6.8-8.1k except zombie's 17k, so treat net figures
under ~5k per phase-frame as ±1k and prefer the trace there.

Consequences: (1) the gross ratios understate the gap most where C++ does
least: library's advance is 2.02x gross and 8.9x net; (2) the "first 300
frames are lighter than the 10k average" reading of jellyfish in the Oct 31
map was this read, not the content (3.2); (3) `perf-compare` ceilings and
`baseline_ratio` in `perf-corpus.toml` are gross ratios. `net_counts.py
COUNTS.txt` subtracts the read per phase and prints both.

### 2.2 Per-window counts (steady state without a trace)

`window_counts.sh <runner|cpp> <fixture> <window> <total>` counts frames
0..k for k = window, 2·window, ... and prints the difference of consecutive
counts per frame. The runs are deterministic, so the differences are
windows of one run; single runs carry ±3M noise, so read windows of ≥500
frames for shape, not for decimals. (`RIVE_GOLDEN_COVERAGE_STEADY_ONLY`
would skip the cumulative runs, but both runners accept it only with one
sample.) Results: 3.2.

### 2.3 Allocation map

`alloc_map.py RUST.json CPP.json --frames N` attributes in-loop allocator
and copy/zero leaves (xzone malloc/free, operator new/delete, the Rust
allocator shims; memmove/memset/bzero) to the first frame above them that
is not allocator, container or smart-pointer plumbing, and counts
allocations and frees as calls of the xzone malloc and free entries.
Results: 3.7.

### 2.4 The first frame in a short trace (found 2026-11-10)

A trace covers every sample the runner is given, and the first advance does
the file's one-time work: the first layout solve, text shaping, script
initialization, the first state-machine entry. On a 100-1500-frame trace
that one frame is a visible share of the per-frame figure.

Measured from traces of frame 0 alone (`PT_START=<s>` with one frame;
out/*-f0-nov10-pt.json; shares in out/frame0-share-trace-nov10.txt). Frame
0's share of the traced window's advance, Rust / C++:

- data_viz (600 frames from 3000, so the first frame is a 50 s jump): 43% / 46%
- jellyfish (300): 38% / 62%
- gamepad (600): 7% / 27%
- zombie, spotify (100), library_with_text_and_image (300),
  image_scripting_property_value (600): 1-4%
- draw: 0-4% everywhere (zombie Rust 3%, spotify 4%)

Since Nov 10 every per-frame trace figure in this map has that frame
subtracted. `pt_sub.py RUN.json FIRST.json OUT.json` subtracts the
frame-0 trace from the N-frame trace stack by stack. The tables are then
made from frames 1..N-1, divided by N-1. Figures dated Oct 31 still
include frame 0; where that matters (data_viz, component_list_1,
gamepad, jellyfish) the section says so.

Two caveats:

- The 10k-frame counts behind 3.1 also carry frame 0, amortized over
  10,000 frames: at most ~3% of a fixture's advance (data_viz ~6k of
  177k).
- A single-frame count is not a usable measure of frame 0. It includes
  kernel work such as page-ins on a binary's first run: gamepad frame 0
  read 9.9M on a binary's first run and 1.67M on four repeats, while the
  trace puts its advance at 0.9M (out/first-frame-nov10.txt,
  out/frame0-all-nov10.txt).

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

The same counts net of the counter read (2.1), per frame, with the gross
ratio beside the net one (out/net-main-oct31.txt; net figures under ~5k
per phase are ±1k):

| Fixture | adv C++ /frame | adv Rust /frame | gross | net | draw C++ /frame | draw Rust /frame | gross | net |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| zombie_skins | 419302 | 820549 | 1.94 | 1.96 | 22424 | 54866 | 2.12 | 2.45 |
| car_widgets_v01 | 22028 | 247451 | 8.91 | 11.23 | 166979 | 368806 | 2.16 | 2.21 |
| data_viz_demo | 22300 | 177175 | 6.38 | 7.94 | 67715 | 225053 | 3.12 | 3.32 |
| spotify_kids_demo | 160813 | 306579 | 1.87 | 1.91 | 23049 | 52916 | 2.01 | 2.30 |
| component_list_1 | 17769 | 128152 | 5.55 | 7.21 | 8669 | 57396 | 4.22 | 6.62 |
| collapsing_elements | 8644 | 53099 | 3.94 | 6.14 | 14520 | 80515 | 4.14 | 5.55 |
| virtualize_blendmode | 11519 | 86039 | 5.14 | 7.47 | 4266 | 37881 | 4.13 | 8.88 |
| script_create_text_runs | 8858 | 66291 | 4.75 | 7.48 | 9646 | 51110 | 3.57 | 5.30 |
| layout_grid_stack | 11205 | 46935 | 3.02 | 4.19 | 3305 | 16745 | 2.37 | 5.07 |
| jellyfish_test | 233 | 3267 | 1.45 | 14.00 | 6596 | 48150 | 4.18 | 7.30 |
| gamepad_test | 3790 | 24478 | 3.02 | 6.46 | 4085 | 26238 | 3.10 | 6.42 |
| multi_listeners | 3696 | 26781 | 3.27 | 7.25 | 4400 | 20793 | 2.51 | 4.73 |
| text_vertical_trim_test | 17394 | 43712 | 2.10 | 2.51 | 3288 | 14658 | 2.17 | 4.46 |
| audio_script | 1811 | 16710 | 2.80 | 9.23 | 6258 | 23646 | 2.37 | 3.78 |
| viewmodel_based_condition | 3150 | 27301 | 3.51 | 8.67 | 1167 | 4840 | 1.48 | 4.15 |
| image_scripting_property_value | 2319 | 15245 | 2.47 | 6.57 | 2154 | 16070 | 2.61 | 7.46 |
| clear_viewmodel_list | 1175 | 10661 | 2.24 | 9.07 | 1977 | 12794 | 2.28 | 6.47 |
| local_bounds | 1233 | 10478 | 2.20 | 8.50 | 3359 | 13954 | 2.08 | 4.15 |
| background_measure | 2092 | 15504 | 2.57 | 7.41 | 731 | 3445 | 1.38 | 4.71 |
| script_dependency_test_using_library_v2 | 1451 | 11747 | 2.30 | 8.10 | 600 | 4605 | 1.57 | 7.67 |
| script_dependency_test_using_library | 1491 | 11734 | 2.29 | 7.87 | 651 | 4610 | 1.56 | 7.08 |
| library | 954 | 8524 | 2.02 | 8.93 | 1038 | 6943 | 1.79 | 6.69 |
| library_with_text_and_image | 869 | 7826 | 1.95 | 9.01 | 252 | 3539 | 1.49 | 14.03 |
| **corpus** | | | 2.65 | **2.99** | | | 2.57 | **3.22** |

Three regression shapes are visible without a trace: (1) a floor on every
fixture, advance and draw alike. Gross it reads ~1.25-1.6x; net of the
counter read it is ~2x on the heavy fixtures (zombie, spotify) and 4-14x
on the light ones, where Rust pays a fixed per-frame cost that C++ does
not (M15); (2) layout/list/nested fixtures at
2-3x on top of that (car, data_viz, component_list, collapsing, virtualize,
text runs); (3) car alone grew another 57% in advance after Oct 26 (the
state-holds series of Oct 30-31 is the only hot-path change in that window:
"a layer that owes its pose never settles").


### 3.2 Where the frame goes by subsystem: exact traces, Rust `ca1739617d` vs C++ `022a0fa9` (Oct 31) and main `6cc1a07141` + band-aid vs `cd5351c3` (Nov 10, frame 0 subtracted) (Processor Trace, self instructions per frame, bucketed by the leaf-most subsystem frame; `pt_buckets.py`, tables in perf-scripts/out/buckets/)

A bucketing correction (2026-11-10). Until Nov 10, `pt_buckets.py` matched
C++ names together with their argument lists. Every
`X::update(rive::ComponentDirt)` therefore matched the dirt pattern, and the
C++ dirt bucket absorbed `PathComposer::update`, `Path::update` and similar
work that belongs in paths/geometry. Re-bucketing the Oct 31 C++ traces
(out/buckets/*-oct31-v2.txt) moves 67k/frame on zombie (C++ dirt 174.8k →
107.7k, paths/geometry 83.5k → 142.1k) and 36k on spotify (dirt 63.4k →
27.3k). It moves under 1k elsewhere. Rust names have no argument lists, so
the Rust columns do not change. So Rust's dirt cascade is about 2x C++ on
both fixtures, not 1.2x on zombie and 0.84x on spotify as this map said;
the operation ledger (3.3), built from inclusive costs per operation,
already had it at 1.62x.

Zombie, 100 frames (trace total matches the 10k-frame count within 1.5%).
Nov 10: main `6cc1a07141` + the M1 band-aid against C++ `cd5351c3`, frames
1-99 (frame 0 subtracted, 2.4; out/zb-rest-nov10-pt.json,
out/zb-cpp-rest-nov10-pt.json, out/buckets/zb-rest-nov10.txt). C++ zombie is
unchanged between the two pins. The Oct 31 Rust column is `ca1739617d`
without the band-aid, frames 0-99:

| advance bucket | Rust Oct 31 | Rust Nov 10 | C++ Nov 10 | ratio Nov 10 | excess Nov 10 |
| --- | ---: | ---: | ---: | ---: | ---: |
| dirt propagation | 214.6k | 215.9k | 108.8k | 1.98 | 107.1k |
| paths/geometry | 162.5k | 150.2k | 138.9k | 1.08 | 11.3k |
| transforms/update walk | 161.6k | 146.3k | 61.6k | 2.37 | 84.7k |
| animation apply | 88.1k | 92.4k | 29.2k | 3.17 | 63.2k |
| borrow scope (thread-local) | 50.0k | 17.7k | 0 | — | 17.7k |
| state machine | 38.5k | 27.3k | 7.4k | 3.70 | 19.9k |
| alloc | 30.5k | 23.4k | 23.3k | 1.00 | 0.1k |
| paints/effects | 29.1k | 24.0k | 46.1k | 0.52 | -22.1k |
| data bind | 17.9k | 17.9k | 1.2k | 15.2 | 16.7k |
| artboard root (walk self) | 17.4k | 4.9k | 0.7k | 7.6 | 4.3k |
| text | 13.3k | 0.0k | 0 | — | 0 |
| nested/host/quiet | 9.4k | 5.6k | 1.4k | 4.1 | 4.2k |
| layout | — | 3.8k | 0.3k | 13.5 | 3.5k |
| **advance total** | **839.3k** | **729.4k** | **418.8k** | **1.74** | **310.6k** |
| **draw total** | **57.2k** | **49.6k** | **22.4k** | **2.22** | **27.2k** |

Paths/geometry is near parity once C++'s path work is in the right row. The
zombie excess is the cascade (107k), the walk (85k), keyed apply (63k), the
state machine (20k), the scope (18k) and data binding (17k). Paints/effects
runs at half of C++'s cost: Rust batches stroke-effect invalidation.

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

Spotify, 100 frames, Nov 10 (frames 1-99, out/buckets/sp-rest-nov10.txt):
advance 281.5k vs 161.8k (1.74x), draw 49.8k vs 23.1k (2.15x). By bucket:

- animation apply 2.74x (65.9k vs 24.0k)
- dirt 1.94x (53.4k vs 27.5k)
- transforms 2.20x (45.6k vs 20.8k)
- paths/geometry 1.09x (82.1k vs 75.6k)
- alloc 2.39x (12.9k vs 5.4k)
- borrow scope 6.7k vs 0
- data bind 5.5k vs 0.2k

Oct 31 (Rust `ca1739617d`, no band-aid, frames 0-99, re-bucketed): advance
319k vs 168k (1.91x), draw 54.9k vs 23.9k (2.30x); dirt 1.96x (53.5k vs
27.3k), paths/geometry 1.13x (87.1k vs 77.4k).

Data_viz settles after about 1,000 frames. Per-1000-frame windows, net of
the counter read (out/win-dv-cpp.txt, out/win-dv-rust-bandaid.txt; Rust is
the fast-profile Oct 31 runner with the M1 band-aid):

- Advance, frames 0-999: C++ 35.5k, Rust 186.1k (5.2x).
- Advance, frames 1000-9999: C++ 19.3-22.0k, Rust 115.5-127.2k (5.8x on the
  mean, 21.0k vs 121.7k).
- Draw: flat at 68k vs 184k (2.7x) throughout.

Data_viz, steady state, Nov 10: frames 3001-3599 (main `6cc1a07141` +
band-aid vs `cd5351c3`; the 600-frame trace from frame 3000 minus a trace
of the jump frame alone; out/dv-late-rest-nov10-pt.json,
out/dv-late-cpp-rest-nov10-pt.json, out/buckets/dv-late-rest-nov10.txt).
Advance 122.2k vs 21.0k (5.82x), the window counts to the hundred; draw
203.4k vs 67.6k (3.01x).

Nothing is solved or shaped in these frames on either side: Taffy is under
1k per frame, Yoga and `Font::shapeText` are at 0. So the steady-state
excess is all per-frame bookkeeping over unchanged content:

| advance bucket | Rust | C++ | ratio | what runs every frame on the Rust side |
| --- | ---: | ---: | ---: | --- |
| data bind | 37.9k | 4.2k | 9.1 | `checkpoint_live_value` 4.3k self (8.4k inclusive), `advance_data_binds` 3.5k, `DataConverter::advance_handle` 3.3k, `update_data_binds` 3.1k, `data_bind_container` lookups 2.9k |
| text | 28.0k | 3.3k | 8.5 | `Text::sync_style_list_items_occurrence` 16.6k inclusive (style lists re-synced although no style changed), `Text::advance_component_occurrence` 3.9k, leases on the style vectors |
| layout | 20.4k | 5.8k | 3.5 | `advance_layout_component` 18.1k self against C++ `LayoutComponent::advanceComponent` 4.5k |
| borrow scope | 10.6k | 0 | — | the thread-local scope (M1) |
| artboard root | 7.5k | 4.2k | 1.8 | |
| state machine | 5.9k | 1.8k | 3.3 | |
| nested/host | 5.4k | 1.2k | 4.4 | |
| alloc | 4.4k | 0 | — | C++ allocates nothing per steady frame |
| animation apply | 1.6k | 0 | — | |

Draw (203k vs 68k) is the same as at the start:

- the draw root, 78.4k (`draw_content_with_visitor` 20.0k,
  `ShapeCells::draw` 17.3k, the range dispatch 12.8k), against C++
  `drawContent` 14.1k plus `ShapePaint::draw` 19.4k
- layout 52.8k vs 6.1k (`draw_own_paints` 11.4k, `is_type_of` 7.7k,
  `draw_listed_layout` 7.0k, `group_opacity::draw_box` 4.7k)
- text 34.3k vs 10.2k
- the scope, 13.2k

Data_viz, first 300 frames, Oct 31 (advance 374k vs 61k, 6.1x; draw 233k
vs 68k, 3.4x). About half of this advance is frame 0: the jump frame of the
Nov 10 trace costs 55.6M vs 10.8M instructions, ~185k vs ~36k per frame over
300 frames. The Taffy and measure figures below are mostly that first
solve:
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

Component_list_1, 300 frames, Oct 31, frame 0 included (advance 147k vs 25.6k, 5.7x; draw 58k vs
8.8k, 6.6x): state machine 36.7k vs 7.1k (5.2x; rows' machines never
settle), artboard root 29.7k vs 3.1k (per-row walks), borrow scope 18.7k,
data bind 16.6k vs 2.1k, layout 14.3k vs 2.6k, text 11.1k vs 3.8k. Draw
root 31k vs 0.4k, layout 10k vs 0.9k.

Idle frames: what each runtime pays per frame when nothing moves. Two
fixtures, traced 2026-11-10 on main `6cc1a07141` + the M1 band-aid against
C++ `cd5351c3`, frames 1-299 (frame 0 subtracted, 2.4;
out/buckets/jf-rest-nov10.txt and lti-rest-nov10.txt, which list the top
self leaves per phase on both sides).
Jellyfish is also its own steady state: C++ runs it as a `StaticScene`, and
per-500-frame counts are flat from frame 0 to 10,000 on both sides (C++
6.7k advance / 13.2k draw gross, Rust fast-profile band-aid runner 8.5k /
43.5k gross; out/win-jf-cpp.txt, out/win-jf-rust-bandaid.txt). The Oct 31
map read the gap between the trace and the 10k count as "early frames are
lighter"; it was the counter read (2.1).

| Idle fixture, per frame | Rust advance | C++ advance | ratio | Rust draw | C++ draw | ratio |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| jellyfish_test (images and a skinned mesh, no animation) | 2,082 | 264 | 7.9x | 39,500 | 6,632 | 6.0x |
| library_with_text_and_image (a settled state machine, nested artboard, text, image) | 5,340 | 921 | 5.8x | 2,457 | 319 | 7.7x |
| jellyfish, Oct 31 (Rust `ca1739617d`, no band-aid, C++ `022a0fa9`, frame 0 included) | 5.0k | 0.7k | 7.2x | 48.6k | 6.7k | 7.2x |

By subsystem (self instructions per frame, Rust vs C++):

- **Advance, both fixtures:** the per-frame pass over a settled artboard.
  - lti spends 1,362 vs 102 in the artboard root. That is
    `RuntimeArtboardInstanceHandle::drop` 360, `advance_internal` 326,
    `update_pass` 226 and `sync_style_changes_with_parent_style` 215, which
    C++ skips when nothing is dirty.
  - Data binding: 1,420 vs 133. `advance_and_apply_view_models` 291 and
    `update_data_binds` 255 run with nothing bound changing.
  - The settled state machine: 1,430 vs 477, of which
    `StateMachineInstance::focus_manager` is 192 per frame.
  - The thread-local borrow scope: 592 vs 0.
  - The walk, layout and nested hosts: ~530 vs ~120.
  - Jellyfish pays the same root pass: 1,116 vs 133 (`update_pass` 226,
    the handle drop 216, `sync_style_changes_with_parent_style` 214,
    `advance_internal` 154). It also pays data binding 390 vs 47, the
    scope 232 and the walk 196 vs 17.
  - The skinned mesh deforms only in frame 0; after that it costs nothing
    in advance on either side.
- **Draw, jellyfish:** the image path.
  - Each image asks `Image::natural_size_for_layout` at draw time although
    nothing in the file is laid out. This is the "layout" bucket, 11,243 vs
    5: a `with` lease 3,652, `RenderImageSizeState::visible` 2,156, a
    `pthread_mutex` lock/unlock pair 2,948, and the query itself 1,056.
  - Around it, `Image::draw_occurrence` 4,956, `image_sampler` 2,860 and
    `render_image` 2,068 against C++ `Image::draw` 1,663 +
    `Image::willDraw` 408.
  - The draw-range dispatch: `draw_range_entry` 2,922, `draw_range_drawable`
    1,364, `drawable_draw_handle` 1,189 and `draw_content_with_visitor`
    2,505, against `Artboard::drawContent` 1,295.
  - The thread-local scope: 2,735.
  - The mesh draw: 1,919 vs `Mesh::draw` + `drawImageMesh` 2,147, at parity.
- **Draw, lti:** the visitor.
  - `draw_content_with_visitor` 732, `CoreHandle::is_type_of` 332 per
    frame, the `DrawVisitorScope` drop 115 and the borrow scope 216,
    against C++ `drawContent` 143.

(C++ draw shows the draw loop, `Artboard::drawContent` and `Image::draw`, as
"other"; Rust's lands in "artboard root". Compare totals and named leaves,
not those two rows.)

So an idle frame costs Rust 6-8x C++. On a settled file nothing in the gap
is algorithmic: the same passes run every frame on both sides, but C++'s
passes return on a dirt check while Rust's pay handle, lease and scope
traffic before reaching theirs. The jellyfish draw adds one Rust-only
operation, the image size query (28% of its draw).

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

**The walk split, Nov 10.** Sources:

- An attribution build of main `6cc1a07141` + the M1 band-aid with
  `#[inline(never)]` on the transform, path, composer, stroke, shape-paint
  and shape kernels and on `ComponentGraph::update_entry` / `update_owner` /
  `mark` / `cascade` (perf-scripts/attrib_patch.py; out/attrib-nov10-runner,
  never committed).
- Zombie and spotify, frames 1-99 (frame 0 subtracted), against C++
  `cd5351c3` (`walk_split.py`; out/walk-split-zb-nov10.txt,
  out/walk-split-sp-nov10.txt).

The split counts every stack inside the walk (`update_components_handle` /
`Artboard::updateComponents`) once: kernels when a kernel frame is on it,
otherwise other components' updates, otherwise plumbing (the walk's loop,
per-node dispatch, leases, closures, scopes).

| zombie, per frame | Rust (attribution build) | share | C++ | share | ratio |
| --- | ---: | ---: | ---: | ---: | ---: |
| walk, inclusive | 396.5k | | 240.8k | | 1.65 |
| kernels | 255.0k | 64% | 221.5k | 92% | 1.15 |
| other component updates | 16.9k | 4% | 2.8k | 1% | 6.0 |
| plumbing | 124.5k | 31% | 16.4k | 7% | 7.6 |

Spotify: walk 157.7k vs 103.9k (1.52x); kernels 89.0k vs 77.8k (1.14x),
other updates 30.3k vs 20.8k, plumbing 38.5k vs 5.3k (7.3x).

The attribution build is not free. Zombie's walk is 345.8k in the shipping
build and 396.5k with the kernels out of line (+50.7k, +14.7%); spotify's is
142.4k and 157.7k (+10.8%). That overhead is call, return and spill
traffic, and it lands in both kernel and plumbing stacks. So in the
shipping build zombie's plumbing is between ~74k and ~124k against C++'s
16.4k, and its kernels between ~204k and ~255k against 221.5k. Either way:

- **The kernels are at parity**, 0.9-1.15x, as a whole. The walk's excess,
  105k per frame on zombie (345.8k vs 240.8k) and 39k on spotify, is
  per-node plumbing: `ComponentGraph::update_entry` 74.9k self (958 node
  dispatches per frame, ~78 each), `update_components_handle` 28.2k self,
  `update_owner` 12.7k, `checkpoint_live_value` 3.0k and the thread-local
  scope 2.9k. Against these C++ has `updateComponents` self 16.4k: about
  17 per node.
- **Per kernel, per update**, where the two sides call one function per
  update (equal calls per frame):
  - `PathComposer::update`: 290 vs 292 (zombie, 175 per frame), 600 vs 582
    (spotify)
  - `Stroke::update`: 515 vs 426 (1.21x) and 1,073 vs 935 (1.15x)
  - `ShapePaint::update`: 488 vs 416 (1.17x) and 1,046 vs 925 (1.13x)
- **Transforms are the one kernel family above parity.** Zombie: 140.3k vs
  51.0k per frame, 231 vs 84 per transform update over 608 updates (2.75x).
  Spotify: 35.0k vs 16.9k (2.08x). Rust's 608 updates split into 560 plain
  kernels (79 each) and 48 `TransformComponent::update_occurrence` at ~2.0k
  inclusive (constraint hosts and their `update_constraints_super`, 1.7k per
  call). C++ makes 147 `TransformComponent::update` calls at 137,
  `updateWorldTransform` 608 times at 39 and `updateConstraints` 608 times at
  39.
- **Path kernels are cheaper in the walk on the Rust side** (115.3k vs
  150.6k on zombie, 55.9k vs 63.3k on spotify), because Rust moved
  path-change work out of `Shape::update` (15 per call vs 107) into the
  cascade (`ShapeCells::path_changed`, `PathComposer::local_inputs_changed`,
  3.2). Count the walk and the cascade together for paths.

The family rows overlap where one kernel calls another, so they do not sum
to the kernel total.

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


### 3.7 Allocation map: what allocates per frame, by call site

From the Oct 31 car and data_viz traces (Rust `ca1739617d`, C++
`022a0fa9`), frame loop only, `alloc_map.py` (2.3); tables in
out/alloc/car-oct31.txt and out/alloc/dv-oct31.txt. Both traces include
frame 0 (2.4). Data_viz's 300 frames are about half frame 0 in advance, so
the Nov 10 steady-state rows (main `6cc1a07141` + band-aid vs `cd5351c3`,
frame 0 subtracted; out/alloc/dv-late-rest-nov10.txt) say what a settled
data_viz frame allocates. Car's 1500 frames carry frame 0 at ~5% of Rust's
advance and ~20% of C++'s, so car's C++ allocation counts are partly first
frame. One malloc+free costs
the same on both sides (171-182 allocator instructions per call, both use
the system xzone allocator), so the allocator excess is entirely the count.

| Fixture, phase | allocs/frame Rust | C++ | frees/frame Rust | C++ | allocator instr/frame Rust | C++ | copy/zero instr/frame Rust | C++ |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| car advance | 46.9 | 7.7 | 41.2 | 3.7 | 15,079 | 2,058 | 1,273 | 225 |
| car draw | 5.5 | 5.3 | 4.1 | 4.0 | 1,637 | 1,601 | 146 | 97 |
| data_viz advance, frames 0-299 | 111.9 | 31.0 | 93.4 | 19.5 | 36,156 | 9,206 | 4,721 | 1,114 |
| data_viz draw, frames 0-299 | 9.3 | 3.2 | 3.4 | 1.0 | 2,611 | 737 | 437 | 124 |
| data_viz advance, steady (frames 3001-3599, Nov 10) | 12.0 | 0.0 | 12.0 | 0.0 | 3,798 | 0 | 554 | 0 |
| data_viz draw, steady (Nov 10) | 1.1 | 1.0 | 1.0 | 1.0 | 380 | 347 | 34 | 14 |

Call sites (allocations, frees and allocator instructions per frame):

| Site | Fixture | Rust | C++ counterpart | Why Rust allocates |
| --- | --- | --- | --- | --- |
| `DataConverterGroup` advance → `DataBind::advance_handle` | car | 38 allocs (6.5k) made by the group, 38 frees (6.3k + 1.0k copy) in the bind | none | a boxed data value per converted bind per frame, made and dropped every frame (M2, M13) |
| same | data_viz | 12 allocs / 12 frees, 4.1k | none | same |
| `Text::make_styled` | data_viz | 6.7 / 6.4, 2.2k | `StyledText::append` 0.7 / 0.4, 0.2k | styled text rebuilt per measure (M9) |
| `natural_bounds_for_layout_host` | data_viz | 6.5 / 6.5, 2.1k | none | a temporary per natural-size query |
| `Shape::build_natural_parametric_path` + `Rectangle::new` + `VertexBase::default` + `Rectangle::clone_boxed` | data_viz | 11.3 allocs, 17.1 frees, 4.9k together | none | a whole `Rectangle` (with its vertices) is built and dropped per natural-bounds query of a parametric shape |
| Taffy `compute_preliminary`, `generate_anonymous_flex_items`, `new_vec_with_capacity` | data_viz | 10.5 allocs / 9.6 frees, 3.6k | Yoga `YGNodelayoutImpl` 0.7 / 0.7, 0.2k | Taffy's per-pass scratch vectors |
| `LayoutComponent::measure_layout_node`, `measured_hosts`, `natural_content_hosts`, `layout_providers_nested_with_solo` | data_viz | ~12 allocs / ~11 frees, ~4k | none | per-frame host lists |
| `RawPath` building (`line_to`, `cubic_to`, `add_path`, `push_verb`) | both | data_viz advance ~10 allocs, car ~3 | `RawPath::line`/`cubic`/`addPath` ~10 (data_viz), ~5 (car) | parity: both rebuild paths into growing vectors |
| HarfBuzz shaping | data_viz | (inside text rows above) | `onShapeText`, `shape_run`, `withOptions`, `hb_buffer` ~9 allocs, 2.3k | C++ allocates more here |
| artboard draw | car | `draw_range_entry` 4 allocs, `draw_listed_handle` 4 frees | `Artboard::drawContent` 4 / 4 | parity |

So on car the excess allocations are the converter boxing (38 of the 39
extra per frame). On data_viz's first 300 frames they are natural-size
queries, per-frame layout host lists, Taffy scratch and text restyling
(most of the 81 extra), all in advance. These are mostly the first solve:
once data_viz settles, C++ allocates nothing per advance, and Rust's only
allocation site left is the same converter boxing (12 per frame, 3.8k
allocator instructions, 3% of the steady advance). Draw allocates at parity on car and 3x on data_viz (path
building in draw). The allocator itself is 6% of car's Rust advance and
10% of data_viz's.

### 3.8 The script bridge (gamepad_test, image_scripting_property_value)

Measured 2026-11-10. Rust: main `6cc1a07141` + the M1 band-aid
(out/bin/main-nov10-runner); C++: `cd5351c3`, the pin main now names
(out/bin/cpp-cd5351c3-runner). Release, scripting on.

Gamepad sources: 600 frames with frame 0 subtracted (2.4), so frames
1-599. Traces out/gp-rest-nov10-pt.json and out/gp-cpp-rest-nov10-pt.json;
buckets out/buckets/gp-rest-nov10.txt; ledger out/ops/gp-rest-nov10.txt
(`pt_ops.py` with perf-scripts/ops-bridge.json).

Per frame gamepad makes one scripted draw call into Lua and one scripted
advance call. Lua calls back into the runtime four times: three
`ScriptedRenderer` methods and one `Mat2D` constructor. Frame: advance
18,578 vs 3,671 (5.06x), draw 27,186 vs 5,226 (5.20x).

By bucket, with Luau split out (`pt_buckets.py`; self instructions per frame,
so a collector's finalizers land in their own subsystem):

| Bucket | advance Rust | advance C++ | draw Rust | draw C++ |
| --- | ---: | ---: | ---: | ---: |
| script bridge (runtime-side script code: `nuxie_scripting`, `luaur_rt` / `rive::Scripting*`, the callbacks) | 4,164 | 540 | 9,846 | 740 |
| Luau VM C API (`luaur_vm` push/get/pcall / `lua_*`, `luaD_*`) | 589 | 514 | 2,004 | 767 |
| Luau interpreter (`luau_execute`, `luaV_*`) | 435 | 428 | 1,093 | 962 |
| Luau collector (`luaC_*`, `freeobj`, mark) | 0 | 87 | 277 | 439 |
| allocation (all callers) | 1,095 | 10 | 6,878 | 663 |

The interpreter is at parity: 1.0-1.1x, about 1.5k per frame on both sides.
The Luau C API is 1.15x in advance and 2.6x in draw. The excess sits in the
bridge, 13.3x in draw, and in the allocation it causes.

The ledger, one operation at a time (inclusive instructions per call; calls
per frame are equal on both sides):

| Operation | calls/frame | Rust | C++ | ratio | What the gap is |
| --- | ---: | ---: | ---: | ---: | --- |
| runtime → Lua: one draw call (`LuaScriptInstance::call_draw` / `ScriptingVM::callDraw`) | 1 | 21,042 | 3,887 | 5.41x | Rust: `protected_call` ~14.6k, `get_field("draw")` 3.9k, a `ScriptedRenderer` userdata 979, `reset_execution_budget` 613. C++: `pCall` 3,371, `lua_getfield` 239, `newuserdata` 124 |
| runtime → Lua: one advance call (`call_advance_truthy` / `callAdvance`) | 1 | 5,873 | 1,424 | 4.12x | same shape: the pcall wrapper and a field lookup |
| the pcall wrapper (`protected_call` / `CPPRuntimeScriptingContext::pCall`) | 2 | 8,938 | 2,187 | 4.09x | Rust: `lua_pcall` 6.2k, argument conversion `into_lua_multi` 707, GPU scope exit 556 and enter 440, frees 238. C++: `luaD_pcall` 1,771, `pushcclosurek` 159, `steady_clock::now` 118 |
| `lua_pcall` itself (includes the interpreter and the callbacks below) | 2 | 6,206 | 1,772 | 3.50x | the callbacks and the interrupt hook below |
| runtime reads a script-table field (`Table::get_field` / `lua_getfield`) | 2 | 2,371 | 258 | 9.17x | Rust interns the key with `lua_pushlstring` every call (1,664, including a GC step), then `take_pushed_field` 551. C++: `luaS_newlstr` 115 + `luaV_gettable` 73 |
| Lua → runtime: one callback (`luaur_rt::callback::trampoline` / the C function) | 4 | 2,085 | 401 | 5.20x | Rust: `value_from_stack` 464, a heap argument list and an `Rc<LuaRef>` per userdata argument, result conversion, frees and mallocs. C++: the function body; its 401 is mostly the Mat2D constructor's GC step |
| one draw call issued from Lua (`ScriptedRenderer` method / `renderer_namecall`) | 3 | 717 | 59 | 12.1x | Rust: `free` 265+45 (the argument list), `LuaRef::push` 79, borrow drop 28, `lua_unref` 12; the renderer work itself is ~40 |
| one value constructor from Lua (`Mat2D` global / `mat2d_withTranslation`) | 1 | 1,135 | 1,425 | 0.80x | both create a userdata. C++'s includes a GC step (1,384 in `newuserdata`); Rust's GC step runs in `get_field` instead |
| create a userdata (`create_userdata` / `lua_newuserdatataggedwithmetatable`) | 2 | 774 | 755 | 1.02x | Rust: `newuserdatadtor`, a Rust-side `Box`, metatable cache, registry ref. C++: 639 of the 755 is the GC step |
| GC step (`luaC_step`) | 0.08 vs 0.09 | 40,706 | 16,932 | 2.40x | `freeobj`: Rust finalizers drop `ScriptedArtboard` and renderer userdata through `Drop` (per frame 3,126 vs 1,583) |
| the interpreter (`luau_execute`): self | 2 | 619 | 488 | 1.27x | bytecode execution |
| the interpreter: inclusive | 2 | 5,909 | 1,412 | 4.18x | the four callbacks, the interrupt hook (`interrupt_trampoline` 1,119 vs `interruptCPP` 92) |

Property reads and writes from Lua. Neither gamepad nor
image_scripting_property_value touches a property after frame 0, so these
rows are per call inside frame 0. They come from out/isp-f0-nov10-pt.json and
out/isp-cpp-f0-nov10-pt.json, the same Lua code on both sides, and include
first-use costs such as building a userdata type's metatable:

| Operation (first frame) | calls | Rust per call | C++ per call | ratio | What the gap is |
| --- | ---: | ---: | ---: | ---: | --- |
| Lua reads a view-model property object (`vm.name` → a property userdata: `create_named_scripted_property` / `vm_index` → `ScriptedViewModel::pushValue`) | 2 / 3 | 58,706 (+ `patch_property_userdata` 22,583) | 5,022 | ~12-16x | Rust builds per-property field dispatchers (15.4k), sets methods into tables by `String` key (18.1k), clones the `ScriptViewModel` and its dirt sink, makes callback functions |
| Lua reads an image property's value (`ScriptedPropertyImage` field getter / `property_image_index`) | 2 / 2 | 22,965 | 371 | 62x | Rust creates a `ScriptedImage` userdata through `create_userdata` (18.8k per call, including its metatable on first use) and a `MainThreadRef` (2.9k); C++ makes one `newuserdata` (212) and one `lua_ref` |
| Lua writes an image property (`ScriptedPropertyImage` field setter / `property_image_newindex`) | 1 / 1 | 12,947 | 864 | 15x | Rust: `ScriptViewModel::set_render_image` 11.8k (the asset value set 6.1k, host mutation notification commit 3.2k, property-change finish). C++: `ViewModelInstanceAssetImage::value` 740 + `onValueChanged` 68 |

So the cost is not in Luau. Per steady frame the interpreter runs within
1.3x of C++, and together with the VM API it makes up ~4.1k of Rust's 45.8k
per frame. The excess is the Rust-side bridge, ~14k in script code and ~8k
in allocation per frame. It comes from:

1. Per-call heap traffic: argument lists, an `Rc<LuaRef>` per userdata
   argument, a userdata per draw call. Each is freed on the way out, which is
   why the Rust renderer method spends 310 of its 717 in `free`.
2. Method names are re-interned per call (`get_field` → `lua_pushlstring`, 9.2x).
3. The GPU scope enter/exit around every pcall (~1k per call), which C++ does
   not have.
4. The interrupt hook. Rust goes through a trampoline into a closure (1,119
   per pcall); C++ makes one direct call (92).
5. Finalizers that run Rust `Drop` for each collected userdata (`freeobj`
   2.4x).

View-model property access from Lua has the same shape at 12-60x, because it
builds per-property dispatch tables and userdata where C++ pushes one
userdata. On these fixtures it is a frame-0 cost; a script that reads
properties every frame would pay it per frame. The bridge commits on
`claude/luau-scripted-perf` (M16) target 1, 2, 3 and part of 4; they do not
touch the property path.

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
| M1 | Per-access thread-local borrow scope | 14% of zombie's and 25% of car's frame as leaf cost; fast A/B of the commit: +21-102% advance, +30-77% draw; est. 5-7G of the 22.3G corpus excess; with the band-aid the residual scope is still 2-9% of a Nov 10 frame (zombie 17.7k, data_viz steady 10.6k advance + 13.2k draw, jellyfish draw 2.7k; 3.2) | `0588c46b1b` enters/leaves a lazily initialized thread-local `RefCell` on every `with`/`with_mut`/artboard access (3.5, 3.6 item 1) | back to the Oct 23 parent's cost on every fixture: zombie advance -20%, component_list -50%, draw -25..-45% | band-aid measured: advance -9..-30%, draw -8..-25%, golden-exact, tests unchanged (3.6 item 1); the rest needs draining at boundaries | S for the band-aid (patch exists, uncommitted); M for the boundary drain (4 request sites, one invariant to prove); low risk |
| M2 | Recursive converter advance walking empty bind containers | car +82k/frame (+44% advance), data_viz +23k (+11%); est. 1.05G corpus | `10ba454b9b` (3.3 data bind rows, 3.4) | car advance back to ~188M/1000 frames; data_viz -10% | five-point sweep, flat except this commit (3.6 item 5); the gate pattern exists unmerged (`7f1c36e46c`, `5de7f16bd3`) | S-M; low risk if advance order is kept |
| M3 | Per-artboard / per-advancing-component fixed cost | `advanceInternal` 5.5-11.9x per artboard; car 225k of its 263k advance is nested-artboard advance; zombie 12 idle nested artboards 4,796 each vs 432 | `AdvancingComponentHandle::advance` resolves, scopes and closures per advancing component every frame; idle hosts re-asked every frame (3.4) | UI fixtures' advance 2-4x → ~1.5x (est.); overlaps M2, M4 | unmerged quiet-nested-hosts commit: text runs -21%, data_viz -3%, car +5% without backoff | M; exactness net exists (verify-every-skipped-pass in testing builds) |
| M4 | State-machine layer bookkeeping per frame | `updateState` 1,751 vs 227 per layer (7.7x), 13-16x on UI fixtures; component_list 37k/frame vs 7k; rows never settle | `try_change_state_from` through handles + four Rust-only per-layer checks (3.4) | component_list advance -18..-49%; every fixture with machines -2..-5% | unmerged input-condition settling + frozen-layer verdict (golden-corrected) | M; risk: over-settling (the `settled=` flag is in the golden stream and catches it) |
| M5 | Animation apply plumbing | ~700 vs ~300 instr per keyed object; zombie ~60-80k/frame, spotify ~45k; est. 1.2G corpus | per-apply handle resolve, cells read, keyframe closure, setter dispatch (3.4) | zombie advance -6%, spotify -7% measured; more with typed sinks for colors/vertices | unmerged compiled tracks `a988a9909e` | M (landed once, needs a rebase); risk: lazily built keyframe holders (golden caught once) |
| M6 | Dirt cascade per visit | 45-79 vs 27 per visit; zombie 84k/frame excess, spotify 20k (ledger, inclusive); Nov 10 dirt bucket after the bucketing fix: zombie 215.9k vs 108.8k (1.98x, 107k excess), spotify 53.4k vs 27.5k (1.94x) (3.2); est. 1.0G corpus | per-edge kind dispatch, liveness, notification batching (3.4) | zombie advance -10% (est.); fusing into the walk measured +5.6% (dead end), precomputed reach ranges unfinished | attribution build; `75942ad4dc` inline mark -2%; fp4/fp5 prototypes | M-L; exactness: onDirty handlers must commute |
| M7 | Update walk per-node plumbing | walk 1.66x (zombie) to 3.15x (data_viz); Nov 10 split (3.4): zombie walk 345.8k vs 240.8k, kernels 1.15x as a whole (transforms 2.75x, path/composer/stroke/paint 0.8-1.2x), per-node plumbing 74-124k vs 16.4k (`update_entry` ~78 per node vs ~17); spotify plumbing 7.3x | per-node member read through the handle, cells behind liveness, scope, `component_graph()` Rc clone per write (3.4) | zombie advance -15% (est.) with SoA transform columns written by compiled tracks | DOD design in perf-scripts/dod-notes.md; graph spike beat C++'s pointer graph by 30% in isolation | L (representation change); constraints (IK, follow-path) in the column pass |
| M8 | Draw dispatch per entry | per shape +138 (1.48-1.98x), per text +657 (2.8x), per image +1,424 (15x); car draw 202k/frame excess, data_viz 165k, jellyfish 42k; est. 7.9G corpus draw excess in total, ~3G of it this | draw root self 62k per car frame, composer-path lease per shape, leased generic path for text/image paints, scope per entry (3.4) | car draw -30%, data_viz -40% (est.); retained replay for resting artboards measured data_viz draw -71%, car -38% | unmerged `44a3b9fd71` (retained draws), cells for text paints need a mutator-visibility cell | M for cells; L for replay (keyed on content revision + write epoch; testing builds verify every replay) |
| M9 | Layout re-measure and per-frame layout plumbing | data_viz layout 132k/frame vs 12.7k over frames 0-299, mostly the first solve (2.4); one solve 11.4M vs 1.2M; `Text::measure` 19x more calls; settled data_viz solves and shapes nothing on either side, and its layout bucket is 20.4k vs 5.8k of per-frame `advance_layout_component` (3.2) | Taffy asks measure per node per pass and each measure reshapes; `sync_minimum_contracts`/`adopt_layout_node` per solve; css_record type queries and style leases per frame (3.4) | data_viz advance -27% (idle layouts from flags) and -19.5% (measure memo) measured on the Oct 16 tree; Taffy's own arithmetic ~15k/frame is not the problem | unmerged `f9fd2bf5a1`, `35b4926d01` | M; CSS semantics stay (decision 2026-10-14) |
| M10 | Text draw path and render styles | per text 1,022 vs 365; `buildRenderStyles` 1.33-1.66x; car 45k/frame, data_viz 32k | style/paint reached through leased generic handles; scope + checkpoint + downcasts per text (3.4) | car/data_viz draw -10% (est.) | cells for layout-component and text-style plain paints landed (`f72be2ff7c`); mutator visibility cell missing | M; low risk |
| M11 | Image draw and natural-size query | 1,516 vs 100-182 per image; jellyfish 33k/frame of 48k (Oct 31); Nov 10 the size query alone is 11.2k of jellyfish's 39.5k draw, and the image path ~21k vs ~2.1k (3.2) | `natural_size_for_layout` per draw through a mutex; `transform_raster` per draw; sampler conversion (3.4) | jellyfish draw -60% on idle frames (est.) | trace leaves | S; low risk |
| M12 | Rc traffic, downcasts and wrappers (the handle tax itself) | Rc clone/drop 6-12% of a frame, `as_any`/downcast 3-7%, `with` wrappers and `RefCell` 1-9% (3.5) | type-erased handle model; every typed access is Rc + generation + RefCell + vtable + closure | the remaining ~1.3-1.5x after M1-M11 (est.); only a representation change reaches 1.0x | ceiling experiment: RefCell alone 12%; per-object cells beat per-site estimates ~2x where applied | L (plan 001 / DOD core); exactness contract: bit-identical end-of-advance state |
| M13 | Allocation per frame | car advance 46.9 allocs/frame vs 7.7 (15.1k allocator instr vs 2.1k), data_viz first 300 frames 111.9 vs 31.0 (36.2k vs 9.2k), settled 12.0 vs 0 (3.8k vs 0, the converter boxing only); draw at parity on car, 3x on data_viz's first frames and parity once settled (3.7) | car: one boxed data value per converted bind per frame (38 of 39 extra); data_viz: a `Rectangle` built per natural-bounds query of a parametric shape, per-frame layout host lists, Taffy scratch vectors, text restyling | car advance -5%, data_viz advance -7% (the excess allocator instructions alone, measured; est. more with the copies and drops around them) | alloc_map.py over the Oct 31 traces | S-M per site; the converter boxing goes with M2 |
| M14 | Live-move checkpoint probe on every mutable access | 0.7-2.9% advance, 0.4-1.9% draw | `fa5e4995ae` vtable probe per `with_mut` (3.6 item 7) | that much | fast A/B with the call removed | S; low risk |
| M16 | Luau bridge: script-call overhead (wanted) | Nov 10 gamepad trace, both sides (3.8): one draw call 21.0k vs 3.9k (5.4x), one callback 2.1k vs 0.4k, one renderer method 717 vs 59, a field lookup 2.4k vs 0.26k; the interpreter itself 1.27x self; bridge 9.8k vs 0.7k per draw frame; Lua property read 23.0k vs 0.37k and write 12.9k vs 0.86k per call (frame 0 of image_scripting_property_value) (Oct 15: ~6.7k of bridge per call around ~1.2k of Lua); scripted fixtures (gamepad 3.0x adv / 3.1x draw, audio_script 2.8x, image_scripting 2.5x, text runs 4.8x, data_viz) | app-data lookups, interrupt closures re-inserted per interrupt, callback arguments and results converted through the heap, a `Rc` per bindings handle, GPU scope exit work, method names re-resolved per call (`luau-scripted-perf` commit messages) | scripted fixtures -2..-15% each (est.; the branch's 7-fixture set measured 1.22x on its DOD base, the Luau-only share was not isolated) | the 14 commits after `3ad92ca574` on `origin/claude/luau-scripted-perf`: `f161e91c71`, `1a99c8cb42`, `18fc2d9eba`, `213f91cff3`, `e38207f8e5`, `09b916bb4d`, `9499fc85da`, `c1f216c022`, `a4f75d627f`, `1d841f7e53`, `cd54bf7de2`, `7e05445ac5`, `ade0256369`, `14ed7f5b2b`; gated green on Oct 17 | S-M to cherry-pick (vendor and scripting crate only, 6.3k/4.2k lines); GC-phase noise means ≥3000-frame windows when measuring |
| M15 | Fixed per-frame cost on quiet content | net of the counter read, 19 of 23 fixtures run 6-14x C++ in advance; idle-frame map (3.2, Nov 10, frame 0 subtracted): jellyfish 2.1k vs 0.26k advance and 39.5k vs 6.6k draw, library_with_text_and_image 5.3k vs 0.9k advance and 2.5k vs 0.3k draw per frame, flat over 10k frames; by subsystem the artboard root pass (~1.3k vs ~0.1k), data binding (1.4k vs 0.1k), a settled state machine (1.4k vs 0.5k, `focus_manager` 192), the thread-local scope (0.3-2.7k vs 0) | per-frame walks that cost even when nothing changed: draw root per entry, image size queries, layout re-measure, rows' machines, advancing components | the quiescence family of M3/M4/M8/M9/M11; on the light fixtures it is most of the frame | window counts (2.2), net counts (2.1) | covered above |

Reading the menu against the headline: M1 + M2 + M14 undo the Oct 15→31
regression (1.64x → 2.62x). M3-M11 are where the data-oriented experiment
found its 1.64x → 1.24x on the Oct 16 tree; the experiment's commits are
evidence that those sizes are real, not a plan to land them as they are.
M16 is the one unmerged piece with intent behind it. M12 is the structural
remainder and the only item that needs a new representation.

## 5. Open questions the map does not answer yet

Status 2026-11-10. Answered since Oct 31:

- the counter read (2.1) and the first frame (2.4)
- jellyfish and data_viz steady state, and the idle-frame map on jellyfish
  and library_with_text_and_image (3.2)
- the walk split: kernels at parity as a whole, transforms 2.75x, the excess
  per-node plumbing (3.4, M7)
- the allocation map, steady state for data_viz (3.7)
- the script bridge on both sides, with the Luau interpreter bucketed apart
  from the bridge and property reads/writes from Lua (3.8, M16)

The C++ pin: main moved from `022a0fa9` to `cd5351c3`, 8 upstream commits
with text shape reuse and list row focus among them. Over the first 300
frames, C++ gamepad advance is 15% cheaper at `cd5351c3` and data_viz 5%;
zombie is unchanged. Nov 10 traces use `cd5351c3`; the sections still at
`022a0fa9` say so.

Still open:

- **Sections at the Oct 31 trees.** 3.1 (the 23-fixture net table), the
  car and component_list_1 rows of 3.2, the ledger and call paths in
  3.3-3.6 and the car allocation map are at Rust `ca1739617d` (no
  band-aid) and C++ `022a0fa9`, and the 300- to 1500-frame traces behind
  them include frame 0. On Nov 10 the band-aid and 127 main commits moved
  zombie's Rust advance from 839k to 729k per frame. Re-taking these is a
  full re-run of the count battery and the car/component_list traces on
  current main.
- **Steady-state Lua property access.** No fixture in the exact corpus
  reads or writes a view-model property from Lua after frame 0, so 3.8's
  property rows are first-frame costs (they include building userdata
  metatables). A fixture that touches a property every frame would give
  the per-frame figure.
- **The `BorrowScope` question.** Whether the scope can be limited to the
  four request sites without counting borrows at all (M1's second half) is
  a design question, not a measurement.

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
- `net_counts.py` (2.1), `probe/rusage_cost.c` (the counter read),
  `window_counts.sh` (2.2), `alloc_map.py` (2.3, `out/alloc/`), `pt2.sh`
  (`pt.sh` with `CPP_RUNNER=` and `PIN_WORKTREE=` overrides),
  `attrib_patch.py` (the measurement-only `#[inline(never)]` patch).
- Nov 10: `pt_sub.py` (subtract a frame-0 trace, 2.4), `walk_split.py`
  (3.4), `ops-bridge.json` (the script-bridge ledger for `pt_ops.py`, 3.8),
  `pt_buckets.py --top K` (top self leaves per phase) with Luau buckets and
  name-only matching, `chain-nov10*.sh` (the trace queue); traces
  `out/*-nov10-pt.json`, frame-0 traces `out/*-f0-nov10-pt.json`, frame-0-free
  aggregates `out/*-rest-nov10-pt.json`, `out/walk-split-*-nov10.txt`,
  `out/frame0-share-trace-nov10.txt`.
- `out/perf-progress.html`: the progress chart (Oct 26 and Oct 31 points added).
