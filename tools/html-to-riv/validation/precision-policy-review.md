# CSS shaping precision capability

Every emitted Text object now adds `text-css-shaping-precision-v1` to the
versioned runtime requirements. It changes font advances and offsets, not CSS
syntax, glyph outlines, authored layout or the Rive binary schema. Text-free
scenes do not request it. Older manifests without this capability retain their
original font policy; unsupported hosts reject new text scenes before drawing.

A host claiming the capability installs `ShapingPrecision::CssExperimental`
through `FontAsset::set_shaping_precision_occurrence` on imported font assets.
The asset retains the policy across decode, replacement and restoration. Font
feature/variation/spacing/tab clones preserve it too. The checked native probe
selects it from the manifest; no environment variable is required. The optional
validation override remains available for old artifacts, but cannot bypass a
host's capability rejection. The default-precision replay diagnostic explicitly
refuses new manifests that require precision rather than silently mislabeling
its output.

The policy targets 16 fractional bits per CSS pixel, with an upper scale bound
of UPM × 16384 to avoid overflowing wide unsigned-16-bit hmtx advances. Ordinary
Rive uses its existing 2048 scale. Actual shaping regressions cover two-em and
maximum-u16 advances, in addition to the independent Chrome q-origin reference.
This does not claim every exotic font/feature/size combination is qualified.

## Verification

- 89 feature-enabled module tests pass, including compiler requirements,
  public compile/import, policy retention and native glyph adapters.
- The host contract test passes: plain text needs precision, spaced text needs
  precision plus its spacing policy, a host missing precision rejects before
  writing a stream, and text-free content loads without it. Missing, malformed
  and unsupported metadata checks remain intact.
- Rebuilt WASM passes all five JavaScript tests, including native/WASM artifact
  equality across the entire accepted corpus. TypeScript API checks pass.
- Module Clippy with warnings denied passes. It caught an excessively precise
  f32 test literal; the exact legacy value is now compared after f64 conversion.
- Automatic manifest selection passes all seven Chrome clip-phase comparisons.
  Their native PNGs are byte-identical to the already visually inspected
  post-bound-fix precision results.
- The full underline lane without an override passes **90/90** at all three
  widths. All 90 native PNGs are byte-identical to the previously visually
  reviewed explicit-precision results, retaining that visual evidence.
- The pure runtime boundary passes (28 protected packages / 57 dependency tables).

Logs: `/tmp/html-precision-capability-{module,build,host,wasm,parity,types,clippy,phase}.log`.
Phase artifacts: `output/playwright/html-to-riv/underline-capability-phase/`.
Earlier full-corpus and visual evidence, the overflow discovery, and remaining
28 vector failures / host affine and P05 group-opacity limitations are recorded
in `underline-transform-diagnosis.md`. This capability does not mark A20 complete.

Automatic underline artifacts: `output/playwright/html-to-riv/underline-capability-glyph/`;
logs `/tmp/html-precision-capability-underline.log` and
`/tmp/html-precision-capability-boundary.log`. No additional full-corpus rerun
is claimed for metadata wiring; the 90-case byte comparison isolates its result.
