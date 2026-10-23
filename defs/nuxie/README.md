# Nuxie scene extensions

These definitions extend the ordinary RIV record stream. They are inputs to
`nuxie-codegen`, alongside the pinned upstream `dev/defs` directory. They do
not define a package/archive or duplicate the upstream definitions.

Type keys and property keys are independent namespaces. Allocate both from
60000–65535. The generator rejects upstream use of this band (including
alternate keys), owned keys outside it, and duplicate definitions/keys.
Upstream has not reserved this band; a future overlap blocks adoption until
resolved explicitly.

- VideoAsset: type 60000; properties 60000–60002.
- Video: type 60001; properties 60003–60021.
- TextStyleDecoration: type 60002; property 60022.
- LayoutCssStyle: type 61000; properties 61000–61082, 61099–61100 and 61106 (`generatedListRow`, compiler-only).
- BoxBorder: type 61001; properties 61083 and 61103.
- BoxGradient: type 61002; properties 61084–61095.
- BoxShadow: type 61003; property 61096.
- BoxPaint: type 61004; properties 61097–61098 and 61101–61102.
- TextStyleCssText: type 61005; properties 61104–61105.

Types 61000–61099 and properties 61000–61999 are reserved for CSS layout,
box paint and CSS text (docs/CSS_LAYOUT.md); other definitions allocate outside that block.

The JSON files are authoritative for allocations and defaults. Generate the
schema with `cargo run -p nuxie-codegen -- --defs <upstream>/dev/defs --out crates/nuxie-schema/src/generated/schema.rs`.
The optional `--extensions` argument overrides this directory.

Loop, muted, paused and ended are bool fields. Volume, playbackRate, loopStart,
loopEnd, position, duration and state are number fields (`double` in the schema).
Duration, ended and state are read only; the schema has no read-only marker, so
the descriptions record this and Video refuses their setters. Autoplay retains
its uint starting attribute.

Video inherits the Image asset reference and component properties; VideoAsset
inherits DrawableAsset metadata and FileAsset loading, including ordinary
FileAssetContents records.

Qualification regeneration uses the repository Makefile upstream pin
`5892bb05be7ae966b751625b4ee12239e6860dc1`. The upstream sampler metadata
(keys 1073–1078) is present in that input and is retained by generation.

Video property 60013 stores an optional normalized caption track as JSON:
`{"version":1,"language":"en","cues":[{"start":0,"end":1,"text":"Hello"}]}`.
Times are seconds, starts inclusive and ends exclusive. Text is plain text;
source subtitle formats must be normalized before serialization. Unknown
versions, malformed cues, and oversized tracks fail scene validation.

Video loopStart (60014) and loopEnd (60015) are seconds. Zero loopEnd selects
the source duration; explicit ends are exclusive and clamped to duration.
Looping is enabled independently by property 60004. Invalid intervals reject
scene import; a start beyond the loaded duration fails playback admission.

TextStyleDecoration (60002) is a child of a TextStylePaint, as upstream's
TextStyleBackground is. Its `line` (60022) is a bit set: 1 draws an underline
and 2 a line-through on every run drawn with that style. Text adds one
rectangle per decorated run on each drawn line to the style's own glyph path,
so the lines share the glyphs' paint. Thickness and position come from the
run font's `post` underline and `OS/2` strikeout metrics, with a fallback of
0.06 em thick at 0.1 em below (underline) or 0.3 em above (line-through) the
baseline. Readers without the type skip the record and its typed property, and
draw the text undecorated.
The CSS layout, box paint and text records (LayoutCssStyle, BoxBorder,
BoxGradient, BoxShadow, BoxPaint, TextStyleCssText)
are specified in [docs/CSS_LAYOUT.md](../../docs/CSS_LAYOUT.md): their
encodings, what is refused at load, and how the runtime uses them.
