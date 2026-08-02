Implemented the P2-b Blob/Image/Mesh wrapper lane without committing.

Highlights:

- Blob userdata, ordered asset lookup, fresh-copy data buffers, and live positive C++ differential.
- Image/ImageSampler wrappers and renderer integration, while leaving decode scheduling and width/height/view to the concurrent lane.
- Vertex/Triangle buffer userdata and validated `drawImageMesh` factory wiring.
- Three manifest rows updated honestly; Image remains `TRACKED-GAP`.
- Added direct upstream mesh cases and image/context/property coverage.
- Full report: [P2B-report.md](/Users/levi/dev/worktrees/nuxie-fld1/P2B-report.md)

All gates pass:

- `cargo test -p nuxie-scripting`
- `cargo test -p nuxie --features scripting`
- Blob live differential
- Scripted golden compare: 320/320 entries, 652 exact segments, zero divergences
- Runtime frame-loop port check
- Rust attribution check
- Formatting and fixed-HEAD diff checks

No commit was created.