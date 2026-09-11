# Width and kana transforms: reference gap

The [CSS Text 3 draft](https://drafts.csswg.org/css-text-3/#text-transform-property)
defines full-width and full-size-kana, permits combining them with casing, and
applies casing before width conversion before kana conversion. Its Appendix G
provides small-kana mappings. These values are listed as at-risk in the draft;
that does not itself prohibit implementing them.

Measured Chromium 153.0.8010.12 rejects both individual values and sampled
combinations through CSS.supports. Computed text-transform is none and innerText
is unchanged. See width-kana-browser-probe.json. Comparing compiled conversions
to that untransformed browser text would not be a valid conformance test.

Next: investigate Firefox support and suitable licensed CJK fonts. Use a browser
that implements the operations for logical and rendering reference, and retain
Chromium checks for the rest of the module. Explicitly document any reference
split. Do not claim the feature is qualified, or silently reduce it to no-op,
merely because Chromium does not implement it. Source-map composition across
casing expansion, half-width voiced pairs and width/kana conversion also needs
coverage. This is an open validation design task, not an external block on all
compiler work.


Firefox 155.0 (Playwright v1543) was installed and measured separately. It accepts
all four sampled values/combinations and retains them in computed style, but
innerText does not include width/kana conversions (uppercase itself is exposed).
See width-kana-firefox-probe.json. Therefore innerText alone is insufficient as
an oracle here. Next evidence should compare transformed rendering against
explicit expected Unicode text with the same pinned font, before using that
browser as the visual reference. Acceptance alone is not rendering proof.


## Rendering oracle established

`npm run test:transform-reference` (after `npx playwright install firefox`) now
compares Firefox 155.0 painted text against explicit Unicode in the same pinned
Nuxie Japanese Fixture font. Ten positive cases match pixel-for-pixel; two
negative controls retaining narrow collapsed spaces differ by 318 pixels each.
All twelve pairs were visually inspected. Source/reference font coverage is
also checked through the public compiler, which rejects missing glyphs.

The first font subset omitted U+FB03; a cmap audit caught the fallback despite
pixel equality. The subset now includes that glyph, and the public coverage
regression prevents recurrence. Font provenance, license, source/subset hashes
and a pinned fontTools reproduction script are recorded in tests/assets/README.md.

Cases establish fullwidth ASCII, collapsed and preserved spaces, voiced kana,
small/halfwidth kana, combination ordering and unchanged non-width compatibility
characters. Surviving collapsed spaces widen too; initial narrow-space readings
of the spec prose were disproved by both Firefox and WPT. Voiced mappings retain
combining scalars even though shaping paints a composed visual cluster.

Artifacts: output/playwright/html-to-riv/width-kana-reference/{gallery.html,
report.json,*-actual.png,*-expected.png,contact-0.png,contact-1.png,contact-2.png}.
Commands/logs: /tmp/html-width-reference.log and /tmp/html-width-font-contract.log.
The targeted public compiler font-coverage test passes. This establishes a
browser reference only: compiler transformation, source-map composition,
native/WASM parity, responsive native pixels and complete mapping coverage
remain to be implemented/qualified. Existing Chromium comparisons remain intact.


## Qualification target clarification

Chrome remains authoritative following the user's question about Firefox.
This investigation is supplemental; it does not change the compiler's target
or qualify width/kana support. Those operations remain deferred and rejected
until their Chrome qualification can be established. A20 underline is the next
independent implementation task.
