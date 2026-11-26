# Nuxie scene extensions

These definitions extend the ordinary RIV record stream. They are inputs to
`nuxie-codegen`, alongside the pinned upstream `dev/defs` directory. They do
not define a package/archive or duplicate the upstream definitions.

Type keys and property keys are independent namespaces. Allocate both from
60000–65535. The generator rejects upstream definitions that claim this band
(including alternate keys), other than properties a Nuxie `augments` file adds
to them (StateTransition.isHold, 60023; ArtboardComponentList.rowArtboardId,
60024, and isPageRows, 60026); it also rejects owned keys outside the
band, and duplicate definitions and keys.
Upstream has not reserved this band; a future overlap blocks adoption until
resolved explicitly.

- VideoAsset: type 60000; properties 60000–60002.
- Video: type 60001; properties 60003–60021.
- TextStyleDecoration: type 60002; property 60022.
- KeyedRowArtboard: type 60003; property 60025.
- StateTransition (upstream type 65, augmented by `state_transition_hold.json`): property 60023.
- ArtboardComponentList (upstream type 559, augmented by `artboard_component_list_rows.json`): properties 60024 and 60026.
- LayoutCssStyle: type 61000; properties 61000–61082, 61099–61100 and 61106–61108.
- BoxBorder: type 61001; properties 61083 and 61103.
- BoxGradient: type 61002; properties 61084–61095.
- BoxShadow: type 61003; property 61096.
- LiveStep: type 62001; properties 62006–62008 and 62030.
- LiveStepUnit: type 62002; properties 62009–62012, 62026 and 62029.
- LiveFlight: type 62003; properties 62031–62047.
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

An `augments` file adds properties to one upstream definition and nothing else:
it names the upstream file and carries only `properties`. Its keys stay in this
band, and its property names must be new across the target's whole hierarchy
(the target, its ancestors and every definition that extends it).
StateTransition's `isHold` (60023), from `state_transition_hold.json`, marks a
state hold (docs/live-moves.md, State holds). It is a bool on the wire, and
writers list it in the property ToC as a uint so readers without it skip it.
ArtboardComponentList's `rowArtboardId` (60024), from
`artboard_component_list_rows.json`, makes the list keyed: its rows are
instances of that artboard, named by the index in the file's artboards as
`ArtboardListMapRule.artboardId` names one, and a host driver mounts, removes
and moves them by key (docs/page-player.md, Host driver). An id that names
no artboard leaves the list keyed and empty: every mount is refused. It is a
uint, unset at -1 (the largest uint), not an upstream `Id`. Readers without it
skip it and see an empty list.

ArtboardComponentList's `isPageRows` (60026), from the same file, marks a
keyed list's rows as a page's rows. An unhosted root whose own artboard holds
a keyed list with the flag is a page (docs/page-player.md, Driver pages): a
host can put the list's rows to sleep and wake them by key, and the page's
content height is read from its top row. It counts only on a keyed list; on
a list that is not keyed it is ignored. A root that is already a page through
its `screens` list keeps that and ignores the flag, and with two flagged
lists the first in file order is the page's. Like `isHold`, it is a bool on
the wire, and writers list it in the property ToC as a uint so readers
without it skip it and see an ordinary keyed list.

KeyedRowArtboard (60003), from `keyed_row_artboard.json`, is a child of a
keyed list and declares one more artboard its rows may be made from: its
`artboardId` (60025) names it by its index in the file's artboards, as
`rowArtboardId` does. The list's declared set is its `rowArtboardId` and each
child's `artboardId`; a host mounting a row names which of them the row is
made from (docs/page-player.md, Host driver), and a list with no children
makes every row from `rowArtboardId`. Under any parent other than an
ArtboardComponentList the record is refused at import. Upstream chooses a
row's artboard through `ArtboardListMapRule`, keyed by the item's view model;
a keyed list has no view models, so it declares the set instead. It is a
uint, unset at -1. Readers without the type skip the record and mount rows of
`rowArtboardId` only.

The CSS layout, box paint and text records (LayoutCssStyle, BoxBorder,
BoxGradient, BoxShadow, BoxPaint, TextStyleCssText)
are specified in [docs/CSS_LAYOUT.md](../../docs/CSS_LAYOUT.md): their
encodings, what is refused at load, and how the runtime uses them.

Types 62000–62009 and properties 62000–62099 are reserved for live moves.
LiveStep is attached to the page root, LiveStepUnit to a LiveStep, and
LiveFlight to the Screen artboard that holds the step's flight timeline.
Progress, path offsets, fade, the four corners and the shadow controls are
keyable doubles. Type 62000 and properties 62000–62005 and 62013–62025
belonged to the retired LiveSlot and are never reused: published files still
carry them (docs/live-moves.md, Files of the retired slot shape).

Each matched element carries a nonzero `flightKey` (61108) on its
LayoutCssStyle, unique on its Screen; each flying copy has a LiveFlight with a
nonzero `key` unique among that Screen's LiveFlight records. A step unit's
`outgoingUnitKey` and `incomingUnitKey` name the two elements' flight keys,
and its `outgoingFlightKey` and `incomingFlightKey` two different LiveFlight
records, or one record twice when the unit sets `transfers`: one instance, one
flight. Import refuses a duplicate flight key or LiveFlight key on one Screen.
A reader without LiveFlight or `flightKey` skips both, as it skips any record
or property whose keys the file lists, so it draws the Screens as authored and refuses
every step.
