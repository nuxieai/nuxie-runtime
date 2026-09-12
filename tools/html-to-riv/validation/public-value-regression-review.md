The value recovery repair preserves **731 of 731 prior public outputs**. Every
request compiles successfully, and every generated Rive file and source map is
byte-identical to its bound reference. There are no changed or excluded cases.
The frozen compiler SHA-256 is
`034084cd868e9b33ba849bc328ea4c540d63115192eba7918e7d7ec00c3c2b2d`.

The reference set retains all 694 requests from the numeric checkpoint's
`numeric-token-prior-regression-r1` run, using that run's **actual** files and
maps. In particular, historical rows 557, 558, and 560 retain the corrected
width `100.71428680419922`; their earlier lossy width is not the expectation.
The source strings and viewport fields are unchanged.

The latest numeric native corpus supplies 38 additional scene candidates:
26 visible cases, two fractional controls, and ten large-number controls.
Thirty-seven are distinct additions. Its `large/content-large-rounded-outer`
request is exactly the same as historical row 665 and has identical bytes and
map; the reference manifest records this single alias. Deduplication compares
the complete request object, ignoring only JSON formatting and key order.
It preserves HTML/CSS spelling and every request field, and never deduplicates
by output hash alone. All 694 historical rows remain independently represented.
The 26 earlier admission-only preflights used the compiler before the numeric
token repair, so the latest numeric render receipts supply their current files.

A conservative scan of all 731 HTML/CSS inputs found **zero** suspicious
non-CSS whitespace cases. The scan covers literal Unicode whitespace, CSS
escapes, HTML entities, and CSS escapes after HTML entity decoding, including
inline style attributes. Positive checks cover literal and escaped NBSP and
HTML entity forms; negative checks retain ordinary CSS whitespace and escaped
backslashes. No previously accepted NBSP source was silently removed. Any scan
finding would remain in the reference set for explicit review.

All 118 files in the new source freeze match their recorded hashes; the copied
source snapshots also match. The driver snapshots the compiler, checks every
reference request/file/map before execution, retains each exact request and
compile log, and records each individual compile command and result. The
evidence pass rechecks every reference and new artifact. Its full-input file
embeds all 731 complete request objects and binds both old and new artifacts.

Commands were run from
`/Users/levi/.codex/worktrees/html-css-immutable`:

```sh
python3 tools/html-to-riv/validation/public-value-regression-references.py tools/html-to-riv/output/public-value-regression-references-r1
python3 tools/html-to-riv/validation/check-output-regression.py tools/html-to-riv/output/public-value-build-r1/frozen/html-to-riv tools/html-to-riv/output/public-value-regression-references-r1/manifest.json tools/html-to-riv/output/public-value-regression-r1
python3 tools/html-to-riv/validation/public-value-regression-evidence.py
```

The existing driver is unchanged. This run checks deterministic compiler
artifacts only. It does not rerender scenes or claim new browser/pixel
qualification. The numeric native receipts' recorded failing controls remain
explicitly identified in the reference metadata; byte stability does not
upgrade their earlier visual or geometry status.

[The receipt](public-value-regression-receipt.json) binds the commands, compiler,
source freeze, complete reference and result manifests, full inputs, and final
hash checks. This audit changes no production code, runtime, renderer,
dependencies, or shared support documents.
