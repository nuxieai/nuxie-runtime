# Public fixed wrapping remainder allocation

The source-owned stretch plan now distributes leftover raw1/64px units using pinned Chrome's LayoutUnitDiffuser algorithm. Allocation occurs in physical line order after wrap reversal, then maps back to logical source-line membership. The plan retains reverse_cross in both live state and source witness; validation rechecks distribution, conservation, bounds and every generated slot. The previous Remainder diagnostic is removed. All existing sizing/position/scalar/paint proofs remain; no runtime changes or browser-derived compiler inputs are involved.

## Evidence

Frozen `output/public-wrapping-remainder-product-build-r1` passes476Rust/57Node tests plus native/WASM/TypeScript/source guard. CLI029b257ef777f8e7288a79ddca1c36bdb0adb3d9a51770861c478c86d9f3356b; WASM892cb00e0f8ea52ae6d12547dce6260fc00ffabbe02b184d94ca4787994d86ab.

32public fixtures cover K2/3/4/7 and every nonzero remainder for those counts, wrap/reverse mates, all directions, unequal line heights, min-over-max bounds and centered odd-unit differences. `output/playwright/public-wrapping-remainder-r1` passes256geometry/pixel frames and512alternate-clear checks.96representative pairs on24sheets receive direct review;160repeat frames transfer by exact image/geometry/file identity. The unchanged capture driver retains historical private labels, but invokes the actual frozen public CLI without an adapter.

New CLI and raw WASM exactly reproduce all1,116previous scene inputs/maps:282image +794transport +16positional wrapping +24exact stretch. No historical native rerender is claimed.

## Retained geometry limitation

`geometry-deltas.json` checks7,040named-owner axis values and records all552nonzero differences.400position values differ by approximately1/128px; the maximum reported delta is0.007815px. The other152smaller deltas include finite serialization precision. These are below the existing0.1px geometry gate; thresholds were not changed. Pinned Chrome quantizes centered offsets in LayoutUnits while the current runtime graph applies a float half. Do not call layout universally exact or silently discard the differences. All corresponding pixel gates pass in this corpus, but broader compositions may expose the offset. Correct source-bound center positioning remains work.

## Remaining work

Preserve current failures and measurements. Next investigate source-proven paint constant folding from public-fixed-folding-plan.md to reduce expensive helper graphs, alongside exact centered offsets. Folding must prove viewport independence from authored CSS and retain dynamic artboard clipping; browser/native measurements cannot become compiler inputs. Responsive percentages, intrinsic sizes, nested content and the rest of the99-item backlog remain in scope. No new performance improvement or general wrapping qualification is claimed here.
