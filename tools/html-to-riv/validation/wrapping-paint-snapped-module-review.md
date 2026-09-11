# Private snapped paint lowering checkpoint

The current `src/wrapping_paint.rs` reproduces all 48 final files from the dynamic snapped-layout experiment byte for byte, twice. The harness imports the actual compiler source, decodes and round-trips the existing sized files, and supplies explicit slot and paint ownership. Source, harness, copied direct dependency libraries, build command, executable and reproduction hashes are retained in `output/wrapping-paint-snapped-module-r1` and the accompanying receipt.

This connects the private implementation to the experiment's 384 passing Chrome/native geometry and pixel comparisons and 768 clear controls. It adds no new visual inspection claim; visual coverage and exact image transfers remain bound by `wrapped-snapped-layout-receipt.json`. Historical unsigned/signed gate experiments and the failing threshold transition remain preserved.

The graph computes absolute projected anchor difference, subtracts the caller's epsilon, clamps at zero, normalizes, doubles and clamps at 65536. Leader masks always invert as D minus gate. Nonfinite, negative and oversized thresholds fail before mutation. This numeric validation does not prove a caller's threshold semantically sound.

For nonempty paint plans, record cost is 8N−5+20M+3R+2(R−1)+2S, where M=N(N−1)/2, R=sum((j+1)*Pj), and S=sum(j*Pj). Three items with two paints each add149records, versus110 in the previous gate construction. Checked preflight and ownership validation remain enforced.

Public wrapping remains rejected. The reproduced corpus uses experimental epsilon1/64. Layout-derived error and minimum separation certificates, sizing-normalizer repair, finite mask coverage, intrinsic/default-stretch cases and whole-scene resource admission remain outstanding. See the new slot-size invariant and flex-factor audits for subsequent work.
