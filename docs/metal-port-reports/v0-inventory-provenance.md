# V0 — inventory and provenance

Status: GREEN on 2026-08-22.

Authority: pinned Rive checkout `4ac7b32798da0482e441ef09304dc3b480ed3ee5` and the checked campaign manifests.

Results:

- 111 unique pinned source rows are owned by exactly 41 translation units.
- Compiled target inventories root 79 renderer targets and 32 ORE targets.
- Five generated authority ledgers cover every declared source/configuration/include/dispatch owner.
- Test census: renderer 753 total/713 active/40 declared ignores; tracer 27/27; ORE default 118/118; ORE tools 133/133.
- Corpus contract: 1,469 unique rows, comprising 736 native-Metal-compatible rows and 733 explicit WebGPU-style MSAA exclusions.
- `make metal-port-check` and `make metal-port-progress-check` replay the tracked receipts and regenerate the checked dashboard exactly.
