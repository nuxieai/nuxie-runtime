# W45 independent FL-C5 oracle round-three review

Verdict: **REJECT** at publication commit `ff94a5f2`.

Blocking findings:

- O4 used deferred FIFO delivery, producing
  `leaf-local, leaf-audio, parent-local, parent-audio, root-local, root-audio`
  instead of synchronous depth-first dispatch with root-first audio unwind.
- The final pixel receipts did not carry the production candidate SHA.
- W41's unqualified claim that both rereviews had no findings was unsupported;
  W39 and W40 were independent rejections.

The review also disclosed that `floor2-apple.log` ended in a dirty-tree
packaging refusal, while the separate clean XCFramework rerun passed. The
complete contemporaneous review is preserved in `.flc5/out/W45-oracle-round3.md`.
