# Runtime performance campaign report

Date: October 9, 2026 (America/Los_Angeles). Status: optimization work stopped; accepted changes preserved on a branch. Performance parity has not been achieved.

## Result and delivery

The strongest final result is a direct comparison of the accepted profile-guided Rust executable against the original Rust executable: **38.8313% less elapsed time**, **36.1358% fewer executed instructions**, and **31.3742% less elapsed time on the 21 fixtures excluded from PGO training**. Against the pinned C++ executable, that same accepted Rust executable still takes **2.3628× as long overall**. Every measured fixture and phase remains slower than C++.

These are whole-build results, including source changes and profile-guided optimization (PGO). They are not a claim that compiling this branch normally produces a 38.8% improvement. PGO is an explicit experiment in this branch, not a production build default. No aggregate gain was calculated by adding individual experiment percentages.

The branch is `levi/runtime-performance-campaign-2026-10-09`. Commit `75968bd6be7bc9fcb5956e36c605ed30bb961038` preserves the accepted runtime and the PGO experiment tooling. It is based on `b8a3b3574a6ef0ae04eedd004fad6e5b2c3575cd`, the original campaign checkout. It has not been rebased onto subsequent `origin/main` changes. No pull request is part of this delivery.

The implementation commit changes 94 files, including 52 previously tracked runtime files, 38 added runtime test files, and four PGO tooling/documentation files. Large portions of the diff restore upstream semantics or exercise ownership contracts; line count is not a measure of optimization value. All **1,396 runtime files** match the accepted source manifest at packaging time. Creating the commit did not change their contents.

Two unfinished experiments are included as **unapplied patches**, with their source reviews and available debug result, under [experiments](performance-campaign-2026-10/experiments/). They are not part of the live runtime on this branch. The report and compact evidence copies are committed so the essential results do not depend solely on a temporary directory. Full traces, executables, recordings, and build logs remain local; this branch is not a complete archival copy of those much larger artifacts.

## Scope and starting point

The user asked for a more effective investigation after the October 3–5 work had failed to explain most of an approximately fourfold Rust/C++ gap. The subsequent request expanded to a sustained campaign: use actual profiles, accumulate small improvements where justified, compare complete source owners against C++, work in parallel, and leave a resumable record.

The earlier October 3–5 work is the starting point, not new credit for this branch. It had already landed borrowed path handoff, contour changes, authored-object access changes, and several correctness repairs in the base history. The original investigation reported a historical 1.57% whole-corpus elapsed improvement for borrowed paths and much larger allocation-count reductions that did not translate into similarly large time savings. Different revisions and measurement populations prevented a defensible cumulative three-day number.

This campaign used upstream revision `160085c654874d35ad654a750e782d7c78e050d9` as its C++ authority. Older porting documents mention other revisions; those are not the comparison pin for the results below. The old local upstream checkout was reported absent during the final read-only review, so a continuation must restore that exact revision and verify its fixtures rather than silently use a current checkout.

The benchmark measures scripted runtime advance and draw through a **NullRenderer**. It does not establish end-to-end application, GPU rendering, iOS, Android, or browser parity. It also does not establish equivalence for every possible callback, script, animation, or asset.

## What changed in the accepted implementation

The accepted result is cumulative. Work was divided among source owners, with the C++ header and implementation read together and a separate Rust integration review. The major areas were:

| Area | Work retained in the branch | Why it mattered |
|---|---|---|
| Core handles and generated dispatch | Changes to checked access, native dispatch, completion handling, and generated property/owner forwarding | Reduced repeated access and dispatch work while retaining stale-generation, borrow, retirement, and custom-object behavior |
| Animation application | KeyedObject, KeyedProperty, KeyframeDouble, LinearAnimation, LinearAnimationInstance, loop and state-machine work | Reduced unnecessary traversal/transport and corrected target/context, callback, loop, and state ordering |
| Artboard and drawing | Animation application, initialization, binding forwarding, draw traversal, Drawable and layout integration | Preserved source ordering while reducing repeated ownership and access work in hot advance/draw paths |
| Geometry and deformation | Skin, Weight, PointsPath, Mesh, vertices and shape owners | Removed avoidable temporary handle collections, reduced checked kernel overhead, and restored upstream deformation/owner semantics |
| Bindings and converters | DataBindContainer, converter owners, scripted converter and view-model cloning | Corrected lifecycle, clone/destruction, propagation and virtual-dispatch behavior; not all of this was a speed optimization |
| Events and ownership | Event, AudioEvent, scene, node and associated tests | Preserved callback and destruction chronology around optimized paths |
| PGO experiment tooling | `runner_pgo.py`, fixture manifest, orchestration tests and usage documentation | Made source-frozen training, profile-use builds and exact output checks reproducible |

This is an inventory of changed areas, not isolated speed attribution. Some correctness repairs add required work. A complete owner rewrite can expose a Rust adaptation cost without making that cost disappear, and a smaller function can move work into a caller or helper.

The last accepted source change illustrates the successful narrow approach. A private completion operation handles ordinary Double-property writes in place. It removes transport through the general consuming completion path while preserving the public completion path, layout/special cases, and callback destruction before notification, including unwinding. The bounded machine-code review found less work on several completion routes; the acceptance decision came from measuring the entire rebuilt candidate, not from counting instructions in that suffix.

## How the measurements were made

### Frozen workload and comparisons

The retained workload contains 25 fixture files with pinned hashes. Each timing process advances progressively through **10,000 samples at `i / 60`**, executing scripts. State is not reset between those samples. The campaign explicitly excluded `--benchmark-repeat`, which would answer a different question.

The mature comparison sequence was:

1. Four opening rounds comparing the control binary with itself on the four hot fixtures.
2. Twelve paired rounds on Zombie, Spotify, Car Widgets and Data Viz.
3. Four paired rounds on all 25 fixtures.
4. Four closing identical-control rounds on the hot fixtures.

Baseline/candidate order and fixture order were reversed across rounds. Raw observations, warmups and outliers were retained. Identical-control drift was reported, not subtracted from candidate results. Four full-corpus rounds are limited evidence, especially for small fixture-level effects; a tiny median change with mixed signs was not treated as a reliable improvement.

Elapsed time, advance/draw time, executed instructions, allocations and static code shape were kept separate. An instruction reduction can support a mechanism without establishing an elapsed win. Whole-process instructions also do not have the same boundaries as the timed CPU loop.

### Build and PGO discipline

The measured local build configuration used Apple Silicon, Rust 1.97.1 / LLVM 22.1.8, Clang 21, optimized release builds, fat LTO, one codegen unit, unwind, and the relevant tools/scripting features. Source manifests, actual compiler commands and binary hashes identified each candidate. The remote Mac had a different toolchain, so measured campaign binaries were built locally and transferred; the remote host was not used to rebuild them.

PGO trained only the four hot fixtures. The other 21 were excluded from training and reported separately. Each accepted source/toolchain change required fresh training rather than silently reusing a prior source's profile. Missing-function profile warnings were retained; hash mismatch diagnostics and provenance mattered separately. Counts of indirect-call promotion remarks were not presented as proven final-LTO promotions merely because of where they appeared in interleaved Cargo output.

The reusable [PGO instructions](../tools/perf-gate/PGO.md) describe the committed experiment tool. It is a native-host reproduction aid; it is not the complete remote timing coordinator or a deployment recipe. The measured campaign used explicit compiler identities, and reproducing those results requires matching them rather than assuming whichever `clang` is currently on PATH is equivalent.

### Correctness and review

The source pass compared the complete affected C++ owner, followed by a distinct review of the Rust integration. Checked object access, arena retirement, stale handles, reentrant callbacks, custom projections, and destruction order were central constraints. No measured candidate was accepted by removing checks or inventing invalidation behavior solely to fit sampled output.

Candidate executables were compared against the matching accepted executable on 100 progressive samples for each fixture, with exact stdout and stderr comparisons. The retained Golden classification was 364 entries: 362 matches and two declared differences, with 1,176 exact segments and 1,168 side-channel segments in the cited qualified runs. Declared differences are not passes for universal equivalence.

Focused owner, lifecycle and integration tests varied by change. Counts from successive candidates overlap and must not be added. There is also a historical validation limitation: the broad suite on earlier source `47fb7fe…` recorded 1,643 passes, one failure and 11 ignored tests under default arithmetic. The failing converter/Silver case passed with `strict-fp`. That is a documented configuration-dependent result, not permission to call every feature combination green. The [historical validation summary](performance-campaign-2026-10/evidence/historical-source-validation.json) preserves it; it is not a fresh full-suite run on the final branch.

### Profiling and parallel work

Profiles were tied to executable identity, image UUID, process lifetime and the advance/draw intervals. A conflicting image-load child in the latest Spotify capture was quarantined as a whole rather than corrected until it looked plausible. Assembly inspection followed the changed route, callees and stack frames, avoiding claims based only on an isolated helper.

Parallel agents handled disjoint owner analysis, implementation and independent review. One coordinator owned remote transfers, training, profiling and timing to prevent overlapping measurement jobs. This parallelized development; it could not make simultaneous benchmarks on the same CPU valid. Local compilation, missing caches and interruptions still imposed serial costs.

## What worked

### Accepted progression

These historical checkpoints show where the cumulative result came from. Each percentage is a separately measured comparison, not an increment to add to the others. Snapshot identifiers name retained experiment sources; some historical Git objects are no longer resolvable locally.

| Snapshot | Main additions | Ordinary comparison against preceding accepted source | Direct original Rust → this PGO build |
|---|---|---:|---:|
| `47fb7fe8` | Compact immutable handle identity, borrowed weak upgrades, fewer runtime-root accesses, callback-free owner operations, lazy/empty paint paths, concrete event dispatch, COW animation membership and source chronology repairs | 9.2350% less elapsed versus original | 24.9658% less elapsed |
| `5c0c3ed2` | Generation-checked direct slot storage, borrowed Skin/PointsPath deformation, Weight extent checking, Cubic/Weight and Mesh/Skin corrections | 4.0050% less elapsed versus 47 | 30.3292% less elapsed |
| `0f7635ce` | Path prefix fusion, scalar dirt reads, typed animation-context lookup, borrowed Keyframe/KeyedObject identities, converter and scripted lifecycle corrections | 0.7482% less elapsed versus 5c0 | 31.2448% less elapsed |
| `3b131a29` | Core identity/slot co-location, shared retirement-domain safety, cold teardown outlining, native empty-effects paint preparation | 6.5090% less elapsed versus 0f | 36.4670% less elapsed |
| `af074640` | Further state-machine/layer, generated dispatch, layout, binding and view-model owner work | No source-only number asserted here | 37.7968% less elapsed |
| `cccc555f` | In-place completion for ordinary Double writes | 0.4566% less elapsed versus af074, three of four rounds | 38.8313% less elapsed |

At the initial 47 checkpoint, PGO versus its **same-source ordinary** executable measured 17.8367% less elapsed. The direct combined result was 24.9658%; adding 9.2350% and 17.8367% would be wrong. Later ordinary and PGO comparisons also need their own baselines. The 5c0 fresh-PGO step measured 7.7928% less elapsed versus the preceding PGO binary; 3b measured 7.7414%, with a separately recorded 7.9623% confirmation. These are whole-composition effects.

The selected Weight influence kernel fell from 36 to 16 static instructions while retaining arithmetic order and zero-weight behavior. Its individual elapsed result was inconclusive; it should not receive a standalone speedup claim. Similarly, several individual ordinary candidates contributing to 0f were neutral or slower even though the tested composition improved overall.

Aggregate acceptance did not erase fixture debt. In particular, 3b retained a `background_measure` elapsed regression of roughly 5.33–6.29% across eight repeated pairs against its preceding baseline. Later direct comparisons showed the final accepted build faster than the original Rust baseline on every fixture; that does not retroactively remove the incremental regression.

The preserved [early history](performance-campaign-2026-10/evidence/accepted-early-history.md), [3b history](performance-campaign-2026-10/evidence/accepted-3b-history.md), and [af074 integration](performance-campaign-2026-10/evidence/af074-integration.json) retain detailed lineage and caveats. Their historical status text is superseded by this report.

### Final direct result

Trials 99 and 100 directly refreshed the comparison against the original Rust and pinned C++ executables. These are the current headline numbers, rather than a multiplication of earlier gains.

| Metric | Accepted result |
|---|---:|
| Elapsed reduction versus original Rust | 38.8313% |
| Executed-instruction reduction versus original Rust | 36.1358% |
| Elapsed reduction on the 21 PGO-held-out fixtures | 31.3742% |
| Total Rust / C++ elapsed ratio | 2.3628× |
| Advance Rust / C++ ratio | 2.7261× |
| Draw Rust / C++ ratio | 1.7522× |
| Held-out total Rust / C++ ratio | 3.2093× |
| Whole-process instruction Rust / C++ ratio | 2.6060× |

All 25 fixtures improved over the original Rust executable in all four elapsed pairs. That does not imply parity: every fixture and phase still exceeded C++. At the overall 2.3628× ratio, reaching the same C++ time would require roughly **57.7% less time than the current accepted Rust result**. This arithmetic is a description of the remaining gap, not a forecast.

The [direct decision](performance-campaign-2026-10/evidence/trials99-100-decision.json) and [accepted state](performance-campaign-2026-10/evidence/accepted-state.json) retain exact values and executable identities. The original executables and bound historical summaries were retained, but the original raw build receipts were no longer available. That provenance limitation must remain visible.

### Last accepted increment

The completion change's ordinary comparison showed approximately 0.457% less full-corpus elapsed time in three of four rounds, followed by one fresh PGO comparison. Trial 97 measured **0.9491% less total elapsed time in all four rounds**, **1.2420% less advance time**, and **0.5325% less held-out elapsed time in three of four rounds** against the previous accepted PGO binary.

It had no full fixture regressing in all four elapsed pairs, but some phase and small-fixture regressions remained. The result was accepted with those limitations. It is a whole-candidate, freshly trained PGO result, not proof that the completion suffix alone saves 0.9491%. The [decision](performance-campaign-2026-10/evidence/trial97-decision.json) predates integration; the [integration record](performance-campaign-2026-10/evidence/integration.json) records the later application.

## What did not work

The experiment numbers are local identifiers, not a claim that every number corresponds to a novel optimization or a completed independent experiment. Important rejected or unresolved results include:

| Experiment | Result | Decision / lesson |
|---|---|---|
| Broader borrowed Artboard/dirt access | Violated RefCell/reentry or retirement behavior; repaired trial 25 did not establish a worthwhile whole-corpus win | A longer-lived loan cannot cross callbacks just because the common path appears safe |
| Pure Core metadata access, 28 | About 0.245% slower overall despite a small instruction reduction | Removing apparent metadata overhead did not establish elapsed benefit |
| Dirty/update variants, 77/80/82/84 | Narrow route improvements accompanied by clean, runtime-root or caller costs | Do not rediscover the same dispatch/cleanup variants without a new mechanism |
| Data Viz advance, 86 | Approximately 4.95% fixture advance gain; held-out elapsed approximately 0.774% worse | Useful local lead, insufficient evidence for broad acceptance |
| Whole Shape, 87 | About 2.295% slower overall; draw approximately 11.762% worse | Owner correctness and a plausible fast path do not guarantee a faster whole build |
| Combined 86+89 with fresh PGO, 92 | About 0.161% total improvement in three of four rounds; held-out approximately 0.967% worse in all four | Individual gains do not compose additively |
| Typed storage plus corrected Shape, 93 | **5.529% slower overall; 12.372% slower draw** | Native-path improvements were outweighed elsewhere; representation and code-size costs matter |
| Converter advance, 95 | About 0.158% full improvement, mixed evidence; held-out slightly worse | Data Viz advance remains a lead, not a whole-corpus accepted optimization |
| Qualified Text projection, 96 | **1.173% slower overall; 3.565% slower draw; held-out 1.788% slower** | Fresh checks and correctness restoration increased work; no unchanged retry/PGO |
| ClippingShape owner, 101 | Full elapsed 0.191% lower but only two of four rounds; Spotify consistently faster, Car draw frequently slower | Keep a targeted lead; do not integrate as a broad win |
| PathComposer order, 102 | **0.521% slower overall in all four rounds**, 0.755% slower advance | Correct source read chronology retained as an experimental dependency; unchanged performance candidate parked |
| State-definition borrow, 103 | 0.039% fewer instructions, but **0.113% slower elapsed** and held-out 0.280% slower | Redundant reference-count traffic was real; removing it did not establish a useful time gain |

Trial 93 is an especially useful warning. A selected native Fill route became smaller, while generic operations grew, identity storage increased from 200 to 208 bytes, and text size increased by roughly 454 KB. All 25 fixture elapsed medians regressed. Because that candidate also contained the corrected Shape work, the measurement does not isolate the effect of typed storage by itself. The [decision](performance-campaign-2026-10/evidence/trial93-decision.json) preserves the whole-candidate conclusion.

Likewise, trial 96 compared a correctness-restored owner as well as a projection change. It cannot justify undoing required fresh reads to recover the old timing. [Trial 96](performance-campaign-2026-10/evidence/trial96-decision.json) records the rejection.

For the latest three completed experiments, the detailed decisions are preserved: [101](performance-campaign-2026-10/evidence/trial101-decision.json), [102](performance-campaign-2026-10/evidence/trial102-decision.json), and [103](performance-campaign-2026-10/evidence/trial103-decision.json). No new PGO build, unchanged timing repeat or silent composition was used to turn those mixed or negative results into accepted wins.

## What the evidence says about the remaining slowdown

There is evidence for particular avoidable costs. There is still no experimentally verified, complete decomposition of the Rust/C++ gap.

The latest admitted Spotify capture contained 4,414 one-millisecond advance samples and 418 draw samples across 15 admitted children. One whole 336-row child was excluded for conflicting image-load evidence; all 16 children remained accounted for in the coverage description. Physical advance attribution included:

| Physical symbol / category | Sampled milliseconds |
|---|---:|
| `update_components` | 1,366 |
| `add_dirt` | 909 |
| LinearAnimationInstance application | 410 |
| KeyedObject application | 338 |
| ClippingShape update | 186 |
| All allocator-library leaves | 501 |

Allocator leaves account for **11.3502%** of admitted advance samples. A previous narrative said 521 / 11.8%; that was an arithmetic error, corrected by independent recomputation. The [reproduced allocator report](performance-campaign-2026-10/evidence/spotify-allocator.json) preserves the corrected attribution. Samples are not allocation counts, bytes, or removable time.

The clipping paths provided concrete causes: copying the shape's path list for an emptiness query and copying the paint list during fill-path processing. The candidate that removed those copies improved Spotify consistently in trial 101; its measurement does not isolate the copies from the accompanying chronology, dispatch and stack changes: elapsed, advance and instructions improved in all 16 full/hot pairs. Full-corpus Spotify elapsed improved about 1.81%, and hot Spotify elapsed about 2.27%. However, Car draw regressed in 13 of 16 pairs, so the whole candidate was not accepted.

PathComposer exposed a second real tradeoff. Moving reads to the C++-correct points removed an eager list copy for warm LOCAL-only updates, but additional selected blocks could acquire more snapshots than before. The accepted access/lifetime model also imposed fresh checked projections. Trial 102 made Spotify worse despite the apparently cheaper warm branch.

Large symbols do not settle the diagnosis. `update_components` contains inlined owner work, and `add_dirt` participates in recursive propagation. Their physical attribution is not a dispatch-only budget. Inclusive stack weights overlap other functions and must not be added to the table. The last bounded review did not identify a new generic dirty/update change that was distinct from the rejected experiments.

The clearest current interpretation is that several costs contribute: ownership traffic and copies, checked access and dispatch, code layout/inlining, and correctness-required reads/callback boundaries. Which parts account for most of the remaining excess over C++ is still unresolved. It would be misleading to describe the remaining 57.7% required reduction as a bag of already-identified 1% fixes.

## Unfinished candidates preserved with this report

### ClippingShape no-save repair, 104

[Full patch](performance-campaign-2026-10/experiments/clipping-no-save-104.patch) against the accepted branch implementation.

This contains the complete reviewed 101 owner and its minimal Shape/registry dependencies, plus a no-save-only route intended to avoid unnecessary release/reacquire work. The true/save route remains byte-identical to the reviewed parent. Custom projection and renderer resource access retain their checked lifetime boundaries.

Thirteen focused tests, 355 library tests with two ignored, and 34 tests across six integration targets passed before the report-only stop. These cover the original owner corrections and additional no-save, visibility, projection, retirement, stale-generation and unwind behavior. The [source review](performance-campaign-2026-10/experiments/clipping-no-save-source-review.json) and [debug result](performance-campaign-2026-10/experiments/clipping-no-save-debug.json) are retained.

Ordinary release qualification is **incomplete**. The original build was interrupted without a completion receipt. The resumed build was stopped on the user's report-only clarification; no ordinary recording, Golden or performance result exists. The four new tests are not claimed to have been observed failing on the baseline. The hypothesis is that the added no-save path reduces the Car regression while retaining Spotify's benefit; that hypothesis is unmeasured.

### Shared path-list ownership

[Full patch](performance-campaign-2026-10/experiments/shared-paths.patch) against the accepted branch implementation, including the corrected 102 PathComposer dependency. It does not include 101/104 or state-definition borrowing.

Shape keeps a single authoritative `Rc<Vec<CoreHandle>>`. Four PathComposer reads acquire released immutable snapshots instead of cloning each handle into a fresh vector. Public `paths()` still returns an independent vector. Append uses copy-on-write so a callback can append while the current traversal retains old membership and later selected blocks see the new list.

This is a storage/lifetime adaptation, not a cached payload or alternate object graph. It preserves generation and arena retirement behavior. It also has real costs: one backing allocation even for an empty Shape, extra indirection/reference-count traffic, and potentially expensive append while a snapshot is live. No gain is assumed.

The [independent source review](performance-campaign-2026-10/experiments/shared-paths-source-review.json) cleared the changed owners and four added behavioral tests. Debug qualification is **incomplete**. An empty copied cache caused Cargo cleanup to fail before compilation; the repaired fresh-target attempt was stopped during compilation, before tests ran. No ordinary build or timing exists.

Both full patches were checked with `git apply --check` against the packaged accepted runtime. They should be tried separately in disposable branches if work resumes; checking patch applicability is not correctness or performance validation.

## Where the effort was inefficient

The campaign produced a substantial measured improvement, but the process also spent too much effort on administration and low-yield variants. That needs to be acknowledged directly.

- **Too much repeated evidence plumbing.** Frozen binaries, correct workload boundaries and reliable comparisons were necessary. Repeated receipt copying, long hash-heavy checkpoints and repeated reviews of unchanged machinery consumed time and made the current state harder to read. The repository workflow explicitly discourages a parallel certification bureaucracy; the campaign accumulated more of one than it should have.
- **Small static improvements were sometimes pursued past their value.** Several candidates removed a few instructions or reference-count operations yet did not improve elapsed time. Future work should require a plausible dynamic cost large enough to repay implementation and validation effort.
- **Whole-owner corrections could add cost.** Restoring C++ callback/read chronology sometimes increased work. Treating every rewrite as a likely optimization obscured the distinction between a necessary correctness repair and a performance win.
- **The interaction tax was underestimated.** Stack frames, caller code, generic fallbacks, layout and PGO changed along with the intended fast path. Trial 93 and the failed 86+89 composition are direct counterexamples to adding isolated gains.
- **Build-cache handling caused avoidable delay.** A copied cache once made Cargo report the changed runtime and runner as Fresh. The actual compile check caught it before candidate publication; a package clean forced a real rebuild. At the October 9 resume, dependency directories were absent and a cache marker was missing. The disappearance was observed, not explained. Several local compiles then had to restart.
- **Artifact volume and host availability were operational costs.** Recovery storage was large, obsolete caches required cleanup, and the remote host could be busy with CI or unrelated work. APFS directory-size totals were not treated as unique reclaimed bytes. Those chores did not reduce runtime time.
- **Scope was misread on October 9.** I restarted local work when the user intended a report. I stopped the owned build/test processes after clarification, preserved their incomplete outcomes, and made no remote measurement or new runtime integration. The subsequent branch/report request authorizes packaging, not resuming the optimization campaign.

Parallel agents helped with disjoint analysis and review. They did not compensate for an overly elaborate serial acceptance process or make an unpromising hypothesis valuable. A future campaign should keep one concise result table, preserve raw logs once, and stop a candidate quickly when whole-program evidence is unfavorable.

## Recommended continuation

These are recommendations, not work started by this report.

1. **Resolve the two concrete pending hypotheses first.** Complete 104's ordinary output qualification and one paired comparison. For shared paths, finish debug/lifetime tests, then ordinary output checks and one standalone timing comparison. Evaluate full corpus and held-out fixtures, not just the targeted hot fixture. A repeat requires a changed hypothesis or invalid run, not disappointment in the result.
2. **Separate correctness debt from optimization acceptance.** Preserve the C++ chronology findings even when the corrected owner is slower. If those corrections need to ship independently, plan that explicitly rather than hiding them in a supposedly faster candidate or dropping them because performance regressed.
3. **Use the absolute gap to prioritize.** In the last direct C++ comparison, Zombie had about 515.3 ms excess per 10,000 frames, with about 502.7 ms in advance; Spotify about 205.9 ms, with 197.2 ms in advance; Car about 141.4 ms, with 82.5 ms in draw; Data Viz about 98.9 ms, with 46.3 ms in draw. These phase figures guide investigation but are not an exact additive decomposition of total elapsed outputs.
4. **Demand a new mechanism before reopening generic Core/dirty work.** The obvious borrow/dispatch/storage variants have already been explored. A new proposal should state which dynamic work disappears, why callbacks/lifetimes remain valid, what compensating work it adds, and which observation would reject it. A controlled C++/Rust operation-count or matching profile comparison may be more useful than another generic rewrite; that comparison has not been completed here.
5. **Treat PGO as a separate product decision.** It helped the benchmark configuration, but deploying it requires representative product training, exact-output checks and measurements for the actual embedding and renderer. The four training fixtures are not a production workload specification.
6. **Integrate with modern main separately.** This branch intentionally preserves the measured baseline. Current main has moved on. A future rebase/merge must resolve semantic interactions, restore the pinned oracle, run applicable checks, retrain if using PGO, and measure again. The historical numbers cannot simply be attached to a merged revision.

There is no evidence-based date or fixed number of micro-optimizations that guarantees parity. The useful next milestone is a candidate that improves the full workload without material regressions, followed by another direct C++ comparison when the accepted source changes enough to justify it.

## Current operational state and reproduction

The optimization goal remains paused. There are no campaign builds or timing jobs intentionally left running. On October 9 the old Buildkite API pause had expired, but an existing external host lock prevented the MacBook's CI agent from launching. Other Rust builds were running there. I did not release that lock, stop unrelated work, or modify CI configuration during report packaging. A continuation must establish actual quiet-host conditions; an old paused-agent receipt is not current exclusivity evidence.

The branch's runtime identity can be checked without compiling:

```sh
python3 - <<'PY'
import hashlib, json
from pathlib import Path
manifest = json.loads(Path('docs/performance-campaign-2026-10/evidence/accepted-runtime-files.json').read_text())
for name, expected in manifest.items():
    assert hashlib.sha256(Path(name).read_bytes()).hexdigest() == expected, name
print(f'{len(manifest)} runtime files match the accepted snapshot')
PY
```

Packaging validation on October 9: all 1,396 runtime hashes matched; `git diff --check` passed before the implementation commit; all seven `test_runner_pgo.py` orchestration tests passed; both archived experimental patches passed applicability checks. No full Rust rebuild, benchmark, profile capture or new PGO training was run for this packaging request. Historical candidate validation remains historical evidence, not a fresh validation of every configuration or current main.

Executable identities for matching retained local artifacts:

| Artifact | SHA-256 |
|---|---|
| Accepted ordinary | `c58d55daace6d26fe26d64c4e15c3ca9738732e43b5393b38385de7248858408` |
| Accepted PGO | `8d1d24b33d04b372a746c73034b509c1efd6e93da9590ed7a02163c545a430f9` |
| Original Rust control | `1d0e7c104ccfb7bab211c5c4539e805eced9f5a91bef5033b8f86940f0a83768` |
| Pinned C++ control | `c07e5a237c4848c934badf95d783b09ff124a889cdebc3ede7b4de80155ead54` |

`cccc555f95d9242daf377885a9880ebadde2f5f6` is the historical accepted snapshot identifier used in the retained evidence. That object was not resolvable in this worktree at packaging time. The new implementation commit and committed per-file manifest provide the durable branch identity; do not assume the historical identifier is a fetchable branch commit.

The [evidence manifest](performance-campaign-2026-10/evidence/manifest.json) gives hashes and original local paths for the copied decisions and reviews. Those JSON files retain their historical wording and paths; a later integration or stop is explained here rather than rewriting an old decision. [The checkpoint](runtime-performance-campaign-checkpoint.md) contains the more detailed local continuation history. Its superseded process statements are historical, not instructions to restart the paused campaign.
