# Scalar offsets for contextual case mapping

Based on icu_casemap 2.3.0 (Unicode-3.0 license retained).
Adds three CaseMapperBorrowed methods returning text and scalar-boundary offsets:
lowercase_with_scalar_offsets, uppercase_with_scalar_offsets and
titlecase_segment_with_scalar_offsets. Titlecase has no leading adjustment and
retains tail case; the compiler supplies word segments and head adjustment.

FullCaseWriteable routes its original mapping loop through an optional observer.
The ordinary Writeable path keeps bulk unchanged-tail writes. The offset path
records output scalar count after each source scalar, including deletions,
expansions and Dutch titlecase pairs. Case tables and contextual mapping rules
are unchanged. No matching/diff heuristic is used to infer edits.

The extension is used by the HTML/CSS compiler only. Keep default casing output
identical to upstream and verify traced output/offsets against contextual cases.
