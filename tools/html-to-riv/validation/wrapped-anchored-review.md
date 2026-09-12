# Private anchored wrapping checkpoint

The compiler-private sizing graph now lands an ordinary ComponentOrigin on the visible child. Its target is the existing slot anchor plus line maximum times the desired-minus-line fraction. Native anchor landing consumes the visible child's actual current layout extent, repairing unequal visible/slot alignment without a measurement proxy, feedback edge or runtime change. Generated fields/operands, unique origin ownership and final anchored error bounds are attached to the same private candidate. Sizing cost drops from61N-33 to58N-33 (141 atN3); paint cost is unchanged.

The source-derived candidate is still not public admission. Paint scalar/default bindings, mask viewport/rectangle/clip premises, and the full derived-epsilon native/Chrome pixel campaign remain open. Earlier zero-anchor final-copy proofs are superseded by the anchored arithmetic and instruction audits. Native geometry evidence below is separate and deliberately uses experimental epsilon1/64; it does not qualify the source-derived epsilon by association.

## Evidence

Frozen public build `output/wrapped-anchored-build-r1/frozen` passes397 Rust/56 Node tests, strict TypeScript, native/WASM builds and the immutable source guard, with297 frozen source/artifact entries. Both public binaries are byte-identical to the previous checkpoint. The verifier rechecks the actual282 image and794 historical output artifacts against that exact compiler identity; it does not rerun unchanged compilation or rendering.

The new ordinary-file native experiment covers48 direct-record recipes/384original-and-clone resize frames and1,152visible cross positions. Every position equals the independently expected value exactly; the existing0.1geometry gate was not widened. Native slot geometry, visible used sizes and main positions match the unaugmented baseline exactly. Each case visits one, two and three lines, then repeats the first viewport. Full original/clone/repeat geometry is exact.

The same base bytes with the old49169f2bfb emitter fail all384frames at768positions under the unchanged gate. This demonstrates that the experiment detects the repaired semantic gap. Frozen reemission reproduces144base/scene/trace artifacts. Initial schema-field and negative-control file-glob setup failures are preserved in the r1attempt directories, separately from native failures. See the native review for exact paths.

`validation/wrapped-anchored-evidence.py` verifies11,453artifact bindings, recomputes all positive expected coordinates from actual unaugmented geometry, checks unchanged layout/main positions and clone/repeat data, and recomputes the old failures. No new Chrome capture, raster pixel comparison or visual inspection is claimed. Every historical visual failure remains retained.

## Next

Bind the paint graph and rectangle mask/domain semantics to the actual emitted candidate, including inverse leader gates and clip/draw ordering. Verify the pinned renderer's rectangular clip-intersection path and matrices. Then run the source-derived anchored candidate through same-file original/clone resize against Chrome with edge/overflow pixels and visual review before public admission. Broader topology, text, intrinsic layout and all other scoped backlog items remain in scope. Counts remain13qualified/23partial/4investigating/59pending; no remote push.
