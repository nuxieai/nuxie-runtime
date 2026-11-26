//! A plans screen whose cards are keyed list rows a host mounts by key.
//!
//! The file has no view model and no binds: every record is written by hand.
//! The page ("Plans", artboard 0) is a CSS block column: a "Heading" box
//! holding a title, the "Cards" list, and a "Footer" line. The list's record
//! names the card artboard for its rows (`rowArtboardId`), so the list starts
//! empty and its host mounts each card by a key. The card ("Plan card",
//! artboard 1) is a CSS block column: a 140 point wide "Label" box holding a
//! text, a "Button" box that hears a click and reports "chosen", and a
//! "Scroller" viewport 60 points tall scrolling a 200 point "Line". Its
//! machine enters a one-shot animation, "intro", that moves the "Dot" node
//! right. The page also has an animation that keys nothing, [`STILL`], for
//! a player that plays the page without a state machine.
//!
//! [`scripted_keyed_plans_fixture`] adds a scripted drawable to the card, so
//! each row runs its own instance of one script.
//!
//! [`Cards::WithBanner`] adds a second artboard the list declares for its
//! rows through a `KeyedRowArtboard` child: the "Banner" (artboard 2), a 48
//! point tall "Strip" box filled with [`BANNER_COLOR`].
#![allow(dead_code)]
use crate::live_move_fixture::{V, obj};
use crate::source::drawable_flag::DrawableFlag;
use crate::source::listener_type::ListenerType;

/// The page's local ids.
pub mod page {
    pub const HEADING: usize = 2;
    pub const CARDS: usize = 9;
    pub const FOOTER: usize = 10;
    pub const FOOTER_RECORD: usize = 11;
}

/// The card's local ids, the same in each row.
pub mod card {
    pub const RECORD: usize = 1;
    pub const LABEL: usize = 2;
    pub const WORDS: usize = 4;
    pub const RUN: usize = 5;
    pub const BUTTON: usize = 10;
    pub const SCROLLER: usize = 13;
    pub const LINES: usize = 15;
    pub const SCROLL: usize = 17;
    pub const LINE: usize = 18;
    pub const DOT: usize = 20;
    pub const CHOSEN: usize = 21;
    /// In [`Cards::Badged`] only: the card's badge host.
    pub const BADGE: usize = 28;
    /// In [`scripted_keyed_plans_fixture`] only: the card's scripted
    /// drawable.
    pub const SCRIPT: usize = 28;
    /// In [`Cards::DescribedLine`] only: the scrolled line's semantic data.
    pub const LINE_SEMANTICS: usize = 28;
}

/// The banner's local ids, in [`Cards::WithBanner`] and
/// [`Cards::MisplacedBanner`].
pub mod banner {
    pub const STRIP: usize = 2;
    pub const STRIP_COLOR: usize = 5;
}

/// The card artboard's index in the file's artboards: the list's
/// `rowArtboardId`.
pub const CARD_ARTBOARD: u32 = 1;
/// The banner artboard's index in the file's artboards, in
/// [`Cards::WithBanner`] and [`Cards::MisplacedBanner`].
pub const BANNER_ARTBOARD: u32 = 2;
/// An index past the file's artboards.
pub const MISSING_ARTBOARD: u32 = 9;
/// The banner strip's color, as the file writes it.
pub const BANNER_COLOR: u32 = 0xff99_3366;
/// The banner's height.
pub const BANNER_HEIGHT: f32 = 48.0;

/// The badge's local ids, in [`Cards::Badged`].
pub mod badge {
    pub const TAG: usize = 2;
    pub const TAG_COLOR: usize = 5;
}

/// The heading's title, as the file writes it.
pub const TITLE: &str = "Choose a plan";
/// The card's run text, as the file writes it.
pub const RUN_TEXT: &str = "Monthly";
/// The card's button label.
pub const BUTTON_LABEL: &str = "Choose";
/// The scrolled line's label, in [`Cards::DescribedLine`].
pub const LINE_LABEL: &str = "Line";
/// The page's animation, which keys nothing.
pub const STILL: &str = "still";
/// How far the card's "intro" moves its dot.
pub const DOT_TRAVEL: f32 = 100.0;
/// The heading's height.
pub const HEADING_HEIGHT: f32 = 32.0;
/// The footer's height.
pub const FOOTER_HEIGHT: f32 = 24.0;
/// The badge's tag color, as the file writes it.
pub const TAG_COLOR: u32 = 0xff33_6699;
/// The scroll offset property of a scroll constraint, vertical.
pub const SCROLL_OFFSET_Y: u16 = 760;

/// What the "Cards" list is.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Cards {
    /// A keyed list of cards.
    Keyed,
    /// A keyed list of cards that each host a badge artboard.
    Badged,
    /// A keyed list whose `rowArtboardId` names no artboard of the file.
    MissingArtboard,
    /// A list with no `rowArtboardId`, whose rows come from list items as a
    /// bound list hands them over.
    Unkeyed,
    /// A keyed list of cards whose scrolled "Line" has semantic data, a
    /// text labeled [`LINE_LABEL`].
    DescribedLine,
    /// A keyed list of cards that also declares the banner artboard
    /// ([`BANNER_ARTBOARD`]) for its rows, through a `KeyedRowArtboard`
    /// child.
    WithBanner,
    /// A keyed list of cards that also declares, through a
    /// `KeyedRowArtboard` child, an artboard the file does not have
    /// ([`MISSING_ARTBOARD`]).
    DeclaresMissing,
    /// A keyed list of cards, and a `KeyedRowArtboard` naming the banner
    /// artboard under the "Heading" box rather than under the list.
    MisplacedBanner,
}

fn var(b: &mut Vec<u8>, mut n: u64) {
    while n >= 128 {
        b.push((n as u8 & 127) | 128);
        n >>= 7;
    }
    b.push(n as u8);
}

/// A state machine layer: 0 entry, 1 any state, 2 exit, then, with
/// `plays`, 3 an animation state playing animation 0, which entry leads to.
fn layer(b: &mut Vec<u8>, plays: bool) {
    use V::*;
    obj(b, "StateMachineLayer", &[("name", S("layer"))]);
    obj(b, "EntryState", &[]);
    if plays {
        obj(b, "StateTransition", &[("stateToId", U(3))]);
    }
    obj(b, "AnyState", &[]);
    obj(b, "ExitState", &[]);
    if plays {
        obj(b, "AnimationState", &[("animationId", U(0))]);
    }
}

/// A click listener on `target` that fires the event at `event`.
fn click(b: &mut Vec<u8>, target: usize, event: usize) {
    use V::*;
    obj(
        b,
        "StateMachineListenerSingle",
        &[
            ("targetId", U(target as u64)),
            ("listenerTypeValue", U(ListenerType::Click as u64)),
        ],
    );
    obj(b, "ListenerFireEvent", &[("eventId", U(event as u64))]);
}

/// A CSS record on `parent` sized in points; `None` leaves a side auto.
fn record(b: &mut Vec<u8>, parent: usize, width: Option<f32>, height: Option<f32>) {
    record_with(b, parent, width, height, &[]);
}

fn record_with(
    b: &mut Vec<u8>,
    parent: usize,
    width: Option<f32>,
    height: Option<f32>,
    more: &[(&str, V<'_>)],
) {
    use V::*;
    let mut props = vec![("parentId", U(parent as u64))];
    if let Some(width) = width {
        props.extend([("widthKind", U(1)), ("width", F(width))]);
    }
    if let Some(height) = height {
        props.extend([("heightKind", U(1)), ("height", F(height))]);
    }
    for (name, value) in more {
        let value = match value {
            U(n) => U(*n),
            F(n) => F(*n),
            _ => panic!("a record property is a number"),
        };
        props.push((*name, value));
    }
    obj(b, "LayoutCssStyle", &props);
}

/// A text holding one run of `text` at `size` points, its paint and color,
/// written as the five records from `at`.
fn text(b: &mut Vec<u8>, name: &str, parent: usize, at: usize, words: &str, size: f32) {
    use V::*;
    obj(
        b,
        "Text",
        &[("name", S(name)), ("parentId", U(parent as u64))],
    ); // at
    obj(
        b,
        "TextValueRun",
        &[
            ("parentId", U(at as u64)),
            ("styleId", U(at as u64 + 2)),
            ("text", S(words)),
        ],
    ); // at + 1
    obj(
        b,
        "TextStylePaint",
        &[
            ("parentId", U(at as u64)),
            ("fontAssetId", U(0)),
            ("fontSize", F(size)),
        ],
    ); // at + 2
    obj(b, "Fill", &[("parentId", U(at as u64 + 2))]); // at + 3
    obj(
        b,
        "SolidColor",
        &[
            ("parentId", U(at as u64 + 3)),
            ("colorValue", Color(0xff11_2233)),
        ],
    ); // at + 4
}

/// The methods a fixture's script implements: `init` and `advance`.
pub const SCRIPT_METHODS: u32 = crate::source::scripted::scripted_object::INITS_BIT
    | crate::source::scripted::scripted_object::ADVANCES_BIT;

/// A script asset "Ticker" at the next asset index, implementing
/// [`SCRIPT_METHODS`], whose contents are `payload`.
pub(crate) fn script_asset(b: &mut Vec<u8>, payload: &[u8]) {
    use V::*;
    obj(
        b,
        "ScriptAsset",
        &[
            ("assetId", U(1)),
            ("name", S("Ticker")),
            ("serializedImplementedMethods", U(u64::from(SCRIPT_METHODS))),
        ],
    );
    obj(b, "FileAssetContents", &[("bytes", Bytes(payload))]);
}

/// A scripted drawable "Script" on the artboard's root running asset 1, the
/// script [`script_asset`] writes after the font.
pub(crate) fn scripted_drawable(b: &mut Vec<u8>) {
    use V::*;
    obj(
        b,
        "ScriptedDrawable",
        &[
            ("name", S("Script")),
            ("parentId", U(0)),
            ("scriptAssetId", U(1)),
        ],
    );
}

/// The plans screen with its "Cards" list as `cards`.
pub fn keyed_plans_fixture(cards: Cards) -> Vec<u8> {
    build(cards, None)
}

/// As [`keyed_plans_fixture`] with [`Cards::Keyed`], with the card holding a
/// scripted drawable, "Script", after its styles: a script asset whose
/// contents are `payload` (a signed content header and Luau bytecode), named
/// "Ticker", that implements `init` and `advance`. Each row runs its own
/// instance.
pub fn scripted_keyed_plans_fixture(payload: &[u8]) -> Vec<u8> {
    build(Cards::Keyed, Some(payload))
}

fn build(cards: Cards, script: Option<&[u8]>) -> Vec<u8> {
    use V::*;
    let opaque = u64::from(DrawableFlag::OPAQUE.0);
    let badges = cards == Cards::Badged;
    let banners = matches!(cards, Cards::WithBanner | Cards::MisplacedBanner);
    let mut b = b"RIVE".to_vec();
    for n in [7, 0, 0, 0] {
        var(&mut b, n);
    }
    obj(&mut b, "Backboard", &[]);
    obj(
        &mut b,
        "FontAsset",
        &[("assetId", U(0)), ("name", S("Roboto"))],
    );
    obj(
        &mut b,
        "FileAssetContents",
        &[(
            "bytes",
            Bytes(include_bytes!("../../../fixtures/fonts/roboto-input.ttf")),
        )],
    );
    if let Some(payload) = script {
        script_asset(&mut b, payload);
    }

    // Plans.
    obj(
        &mut b,
        "Artboard",
        &[
            ("name", S("Plans")),
            ("width", F(320.0)),
            ("height", F(640.0)),
            ("styleId", U(12)),
        ],
    ); // 0
    record(&mut b, 0, None, None); // 1
    obj(
        &mut b,
        "LayoutComponent",
        &[
            ("name", S("Heading")),
            ("parentId", U(0)),
            ("styleId", U(13)),
        ],
    ); // 2
    record(&mut b, page::HEADING, Some(280.0), Some(HEADING_HEIGHT)); // 3
    text(&mut b, "Title", page::HEADING, 4, TITLE, 20.0); // 4 to 8
    let mut list = vec![("name", S("Cards")), ("parentId", U(0))];
    match cards {
        Cards::Keyed
        | Cards::Badged
        | Cards::DescribedLine
        | Cards::WithBanner
        | Cards::DeclaresMissing
        | Cards::MisplacedBanner => list.push(("rowArtboardId", U(u64::from(CARD_ARTBOARD)))),
        Cards::MissingArtboard => list.push(("rowArtboardId", U(9))),
        Cards::Unkeyed => {}
    }
    obj(&mut b, "ArtboardComponentList", &list); // 9
    obj(
        &mut b,
        "LayoutComponent",
        &[
            ("name", S("Footer")),
            ("parentId", U(0)),
            ("styleId", U(14)),
            ("drawableFlags", U(opaque)),
        ],
    ); // 10
    record(&mut b, page::FOOTER, Some(280.0), Some(FOOTER_HEIGHT)); // 11
    for parent in [0, page::HEADING, page::FOOTER] {
        obj(
            &mut b,
            "LayoutComponentStyle",
            &[("parentId", U(parent as u64))],
        ); // 12 to 14
    }
    let declared = match cards {
        Cards::WithBanner => Some((page::CARDS, BANNER_ARTBOARD)),
        Cards::DeclaresMissing => Some((page::CARDS, MISSING_ARTBOARD)),
        Cards::MisplacedBanner => Some((page::HEADING, BANNER_ARTBOARD)),
        _ => None,
    };
    if let Some((parent, artboard)) = declared {
        obj(
            &mut b,
            "KeyedRowArtboard",
            &[
                ("parentId", U(parent as u64)),
                ("artboardId", U(u64::from(artboard))),
            ],
        ); // 15
    }
    obj(
        &mut b,
        "LinearAnimation",
        &[("name", S(STILL)), ("fps", U(60)), ("duration", U(30))],
    );
    obj(&mut b, "StateMachine", &[("name", S("plans"))]);
    layer(&mut b, false);

    // Plan card.
    obj(
        &mut b,
        "Artboard",
        &[
            ("name", S("Plan card")),
            ("width", F(280.0)),
            ("height", F(160.0)),
            ("styleId", U(22)),
        ],
    ); // 0
    record(&mut b, 0, None, None); // 1
    obj(
        &mut b,
        "LayoutComponent",
        &[("name", S("Label")), ("parentId", U(0)), ("styleId", U(23))],
    ); // 2
    record(&mut b, card::LABEL, Some(140.0), None); // 3
    text(&mut b, "Words", card::LABEL, card::WORDS, RUN_TEXT, 16.0); // 4 to 8
    // Role 0 and no label: the text's own role and words.
    obj(
        &mut b,
        "SemanticData",
        &[("parentId", U(card::WORDS as u64))],
    ); // 9
    obj(
        &mut b,
        "LayoutComponent",
        &[
            ("name", S("Button")),
            ("parentId", U(0)),
            ("styleId", U(24)),
            ("drawableFlags", U(opaque)),
        ],
    ); // 10
    record(&mut b, card::BUTTON, Some(100.0), Some(40.0)); // 11
    obj(
        &mut b,
        "SemanticData",
        &[
            ("parentId", U(card::BUTTON as u64)),
            ("role", U(1)),
            ("label", S(BUTTON_LABEL)),
        ],
    ); // 12
    obj(
        &mut b,
        "LayoutComponent",
        &[
            ("name", S("Scroller")),
            ("parentId", U(0)),
            ("styleId", U(25)),
        ],
    ); // 13
    // A scroll viewport as the compiler writes one: relative, scrolling
    // vertically.
    record_with(
        &mut b,
        card::SCROLLER,
        Some(200.0),
        Some(60.0),
        &[("position", U(1)), ("overflowY", U(3))],
    ); // 14
    obj(
        &mut b,
        "LayoutComponent",
        &[
            ("name", S("Lines")),
            ("parentId", U(card::SCROLLER as u64)),
            ("styleId", U(26)),
        ],
    ); // 15
    record(&mut b, card::LINES, None, None); // 16
    obj(
        &mut b,
        "ScrollConstraint",
        &[
            ("parentId", U(card::LINES as u64)),
            ("directionValue", U(1)),
            ("threshold", F(10.0)),
        ],
    ); // 17
    obj(
        &mut b,
        "LayoutComponent",
        &[
            ("name", S("Line")),
            ("parentId", U(card::LINES as u64)),
            ("styleId", U(27)),
        ],
    ); // 18
    record(&mut b, 18, Some(200.0), Some(200.0)); // 19
    obj(&mut b, "Node", &[("name", S("Dot")), ("parentId", U(0))]); // 20
    obj(
        &mut b,
        "Event",
        &[("name", S("chosen")), ("parentId", U(0))],
    ); // 21
    for parent in [
        0,
        card::LABEL,
        card::BUTTON,
        card::SCROLLER,
        card::LINES,
        18,
    ] {
        obj(
            &mut b,
            "LayoutComponentStyle",
            &[("parentId", U(parent as u64))],
        ); // 22 to 27
    }
    if badges {
        obj(
            &mut b,
            "NestedArtboardLayout",
            &[
                ("name", S("Badge")),
                ("parentId", U(0)),
                ("artboardId", U(2)),
            ],
        ); // 28
        record(&mut b, card::BADGE, Some(60.0), None); // 29
        obj(
            &mut b,
            "NestedStateMachine",
            &[("parentId", U(card::BADGE as u64)), ("animationId", U(0))],
        ); // 30
    }
    if script.is_some() {
        scripted_drawable(&mut b); // 28
    }
    if cards == Cards::DescribedLine {
        obj(
            &mut b,
            "SemanticData",
            &[
                ("parentId", U(card::LINE as u64)),
                ("role", U(7)),
                ("label", S(LINE_LABEL)),
            ],
        ); // 28
    }
    // "intro": half a second at 60 frames a second, once, moving the dot.
    obj(
        &mut b,
        "LinearAnimation",
        &[
            ("name", S("intro")),
            ("fps", U(60)),
            ("duration", U(30)),
            ("loopValue", U(0)),
        ],
    );
    obj(&mut b, "KeyedObject", &[("objectId", U(card::DOT as u64))]);
    obj(&mut b, "KeyedProperty", &[("propertyKey", U(13))]);
    for (frame, value) in [(0, 0.0), (30, DOT_TRAVEL)] {
        obj(
            &mut b,
            "KeyFrameDouble",
            &[
                ("frame", U(frame)),
                ("value", F(value)),
                ("interpolationType", U(1)),
            ],
        );
    }
    obj(&mut b, "StateMachine", &[("name", S("card"))]);
    click(&mut b, card::BUTTON, card::CHOSEN);
    layer(&mut b, true);

    if badges {
        // Badge.
        obj(
            &mut b,
            "Artboard",
            &[
                ("name", S("Badge")),
                ("width", F(60.0)),
                ("height", F(20.0)),
                ("styleId", U(6)),
            ],
        ); // 0
        record(&mut b, 0, None, None); // 1
        obj(
            &mut b,
            "LayoutComponent",
            &[
                ("name", S("Tag")),
                ("parentId", U(0)),
                ("styleId", U(7)),
                ("drawableFlags", U(opaque)),
            ],
        ); // 2
        record(&mut b, badge::TAG, Some(60.0), Some(20.0)); // 3
        obj(&mut b, "Fill", &[("parentId", U(badge::TAG as u64))]); // 4
        obj(
            &mut b,
            "SolidColor",
            &[("parentId", U(4)), ("colorValue", Color(TAG_COLOR))],
        ); // 5
        for parent in [0, badge::TAG] {
            obj(
                &mut b,
                "LayoutComponentStyle",
                &[("parentId", U(parent as u64))],
            ); // 6, 7
        }
        obj(&mut b, "StateMachine", &[("name", S("badge"))]);
        layer(&mut b, false);
    }

    if banners {
        // Banner.
        obj(
            &mut b,
            "Artboard",
            &[
                ("name", S("Banner")),
                ("width", F(280.0)),
                ("height", F(BANNER_HEIGHT)),
                ("styleId", U(6)),
            ],
        ); // 0
        record(&mut b, 0, None, None); // 1
        obj(
            &mut b,
            "LayoutComponent",
            &[
                ("name", S("Strip")),
                ("parentId", U(0)),
                ("styleId", U(7)),
                ("drawableFlags", U(opaque)),
            ],
        ); // 2
        record(&mut b, banner::STRIP, Some(280.0), Some(BANNER_HEIGHT)); // 3
        obj(&mut b, "Fill", &[("parentId", U(banner::STRIP as u64))]); // 4
        obj(
            &mut b,
            "SolidColor",
            &[("parentId", U(4)), ("colorValue", Color(BANNER_COLOR))],
        ); // 5
        for parent in [0, banner::STRIP] {
            obj(
                &mut b,
                "LayoutComponentStyle",
                &[("parentId", U(parent as u64))],
            ); // 6, 7
        }
        obj(&mut b, "StateMachine", &[("name", S("banner"))]);
        layer(&mut b, false);
    }
    b
}
