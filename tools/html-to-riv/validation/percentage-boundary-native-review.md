# Percentage overflow boundaries on the immutable runtime

Fourteen ordinary files exercise seven boundary controls on width and height independently. The frozen public compiler accepts every request at a one-pixel main viewport. The immutable observer then resizes that same original and its clone through1→1024→16384→1. Ten cases complete all80native geometry/recording frames; four fail with nonfinite layout at frame1/object36. No browser comparison or pixel qualification is claimed.

Eight nested1000000% sizes remain finite across this sequence; nine fail after the first resize. A fixed maximum of1000000px on every level keeps the sequence finite. A percentage minimum of1000000% with maximum100px fails because the minimum wins. A zero-pixel root or a fixed100px middle node resets amplification and stays finite. A maximum100px only on the ninth node also stays finite: guard logic must consider effective clamped bounds rather than reject every overflow in a pre-clamp preferred expression.

The width and height matrices produce the same result. These controls demonstrate why a guard cannot rely only on the compile viewport or per-token size limits. They test finite geometry on the unchanged target, not arbitrary rendering correctness for huge finite layouts. The new compiler-side guard is separate implementation work; the accepted failing files and logs remain preserved.

See `percentage-boundary-native-receipt.json` and `output/percentage-boundary-native-r2/receipt.json` for input/file/tool hashes and commands. The earlier width-only r1 corpus is retained.
