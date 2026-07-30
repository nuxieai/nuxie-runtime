# W46 independent FL-C5 standards round-three review

Verdict: **REJECT** at publication commit `ff94a5f2`.

Blocking findings:

- Public field checks allowed deref coercion and the hydration closures did
  not exclude extra bounds.
- `RuntimeNestedRemapAnimationReport` and its query were ungated probe-only
  public surface, including an unauthorized `lib.rs` export.
- The semantic forbidden-projection ratchet was identifier-specific and
  rename-vacuous.
- Five of six `floor2-*` receipts lacked an internal candidate SHA, and the
  Apple receipt did not disclose its dirty-tree failure/rerun story.

The review also noted the stale status-document instruction to land the
already-landed publication commit. The complete contemporaneous review is
preserved in `.flc5/out/W46-standards-round3.md`.
