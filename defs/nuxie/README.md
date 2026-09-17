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
- Video: type 60001; properties 60003–60015.

The JSON files are authoritative for allocations and defaults. Generate the
schema with `cargo run -p nuxie-codegen -- --defs <upstream>/dev/defs --out crates/nuxie-schema/src/generated/schema.rs`.
The optional `--extensions` argument overrides this directory.

Boolean playback flags use the uint wire family so unknown-property skipping
uses the existing header field-type metadata. Video inherits the Image asset
reference and component properties; VideoAsset inherits DrawableAsset metadata
and FileAsset loading, including ordinary FileAssetContents records.

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
