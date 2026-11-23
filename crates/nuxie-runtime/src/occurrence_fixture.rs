//! A page that uses one card artboard twice, for writes that reach one use.
//!
//! The file has no view model and no binds: every record is written by hand.
//! The page ("Page", artboard 0) is a CSS block column holding two
//! `NestedArtboardLayout` hosts of the card ("First" and "Second"), each with
//! its item record and a `NestedStateMachine` playing the card's machine, and
//! a "Below" box after them. The card ("Card", artboard 1) is a CSS block
//! column: a 100 point wide "Label" box holding a text, a "Button" box that
//! hears a click and reports "chosen", and a "Field box" holding a text
//! field. Its machine enters a one-shot animation, "intro", that moves the
//! "Dot" node right.
#![allow(dead_code)]
use crate::live_move_fixture::{V, obj};
use crate::source::drawable_flag::DrawableFlag;
use crate::source::listener_type::ListenerType;

/// The page's local ids.
pub mod page {
    pub const FIRST: usize = 2;
    pub const FIRST_RECORD: usize = 3;
    pub const SECOND: usize = 5;
    pub const SECOND_RECORD: usize = 6;
    pub const BELOW: usize = 8;
    pub const BELOW_RECORD: usize = 9;
}

/// The card's local ids, the same in each use.
pub mod card {
    /// The card artboard's own record, which gives a hosted card its
    /// display.
    pub const RECORD: usize = 1;
    pub const LABEL: usize = 2;
    pub const LABEL_RECORD: usize = 3;
    pub const WORDS: usize = 4;
    pub const RUN: usize = 5;
    pub const RUN_COLOR: usize = 8;
    pub const BUTTON: usize = 10;
    pub const BUTTON_RECORD: usize = 11;
    pub const FIELD_BOX: usize = 13;
    pub const FIELD: usize = 15;
    pub const DOT: usize = 21;
    pub const CHOSEN: usize = 22;
    /// In [`badged_fixture`] only: the card's two badge hosts.
    pub const FIRST_BADGE: usize = 27;
    pub const SECOND_BADGE: usize = 30;
}

/// The badge's local ids, the same in each use, in [`badged_fixture`].
pub mod badge {
    pub const TAG: usize = 2;
    pub const TAG_COLOR: usize = 5;
    pub const TAPPED: usize = 6;
}

/// The badge's tag color, as the file writes it.
pub const TAG_COLOR: u32 = 0xff33_6699;

/// The card's run text, as the file writes it.
pub const RUN_TEXT: &str = "Monthly";
/// The card's field text, as the file writes it.
pub const FIELD_TEXT: &str = "hello";
/// The card's button label.
pub const BUTTON_LABEL: &str = "Choose";
/// How far the card's "intro" moves its dot.
pub const DOT_TRAVEL: f32 = 100.0;

/// What the page's own state machine holds.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PageMachine {
    /// No state machine on the page.
    None,
    /// A state machine with no layer and no listener.
    Bare,
    /// A state machine with one layer (entry, any state, exit) and no
    /// listener.
    OneLayer,
    /// As `OneLayer`, with a click listener on "Below" reporting "below
    /// tapped".
    Listening,
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
    use V::*;
    let mut props = vec![("parentId", U(parent as u64))];
    if let Some(width) = width {
        props.extend([("widthKind", U(1)), ("width", F(width))]);
    }
    if let Some(height) = height {
        props.extend([("heightKind", U(1)), ("height", F(height))]);
    }
    obj(b, "LayoutCssStyle", &props);
}

/// The page and card file, with the page's state machine as `machine`.
pub fn two_uses_fixture(machine: PageMachine) -> Vec<u8> {
    build(machine, false)
}

/// As [`two_uses_fixture`], with the card hosting a third artboard,
/// "Badge", twice after its field box: "First badge" and "Second badge",
/// each a `NestedArtboardLayout` with its item record and a
/// `NestedStateMachine`. So the page holds four badges, two in each card.
/// The badge is a CSS artboard holding a 60 by 20 "Tag" box painted
/// [`TAG_COLOR`], whose click its machine reports as "badge tapped".
pub fn badged_fixture(machine: PageMachine) -> Vec<u8> {
    build(machine, true)
}

fn build(machine: PageMachine, badges: bool) -> Vec<u8> {
    use V::*;
    let opaque = u64::from(DrawableFlag::OPAQUE.0);
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

    // Page.
    obj(
        &mut b,
        "Artboard",
        &[
            ("name", S("Page")),
            ("width", F(400.0)),
            ("height", F(300.0)),
            ("styleId", U(11)),
        ],
    ); // 0
    record(&mut b, 0, None, None); // 1
    for (name, host) in [("First", page::FIRST), ("Second", page::SECOND)] {
        obj(
            &mut b,
            "NestedArtboardLayout",
            &[("name", S(name)), ("parentId", U(0)), ("artboardId", U(1))],
        ); // 2, 5
        record(&mut b, host, Some(200.0), None); // 3, 6
        obj(
            &mut b,
            "NestedStateMachine",
            &[("parentId", U(host as u64)), ("animationId", U(0))],
        ); // 4, 7
    }
    obj(
        &mut b,
        "LayoutComponent",
        &[
            ("name", S("Below")),
            ("parentId", U(0)),
            ("styleId", U(12)),
            ("drawableFlags", U(opaque)),
        ],
    ); // 8
    record(&mut b, page::BELOW, Some(100.0), Some(40.0)); // 9
    obj(
        &mut b,
        "Event",
        &[("name", S("below tapped")), ("parentId", U(0))],
    ); // 10
    obj(&mut b, "LayoutComponentStyle", &[("parentId", U(0))]); // 11
    obj(
        &mut b,
        "LayoutComponentStyle",
        &[("parentId", U(page::BELOW as u64))],
    ); // 12
    match machine {
        PageMachine::None => {}
        PageMachine::Bare => obj(&mut b, "StateMachine", &[("name", S("page"))]),
        PageMachine::OneLayer | PageMachine::Listening => {
            obj(&mut b, "StateMachine", &[("name", S("page"))]);
            if machine == PageMachine::Listening {
                click(&mut b, page::BELOW, 10);
            }
            layer(&mut b, false);
        }
    }

    // Card.
    obj(
        &mut b,
        "Artboard",
        &[
            ("name", S("Card")),
            ("width", F(200.0)),
            ("height", F(100.0)),
            ("styleId", U(23)),
        ],
    ); // 0
    record(&mut b, 0, None, None); // 1
    obj(
        &mut b,
        "LayoutComponent",
        &[("name", S("Label")), ("parentId", U(0)), ("styleId", U(24))],
    ); // 2
    record(&mut b, card::LABEL, Some(100.0), None); // 3
    obj(
        &mut b,
        "Text",
        &[("name", S("Words")), ("parentId", U(card::LABEL as u64))],
    ); // 4
    obj(
        &mut b,
        "TextValueRun",
        &[
            ("parentId", U(card::WORDS as u64)),
            ("styleId", U(6)),
            ("text", S(RUN_TEXT)),
        ],
    ); // 5
    obj(
        &mut b,
        "TextStylePaint",
        &[
            ("parentId", U(card::WORDS as u64)),
            ("fontAssetId", U(0)),
            ("fontSize", F(16.0)),
        ],
    ); // 6
    obj(&mut b, "Fill", &[("parentId", U(6))]); // 7
    obj(
        &mut b,
        "SolidColor",
        &[("parentId", U(7)), ("colorValue", Color(0xff11_2233))],
    ); // 8
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
            ("styleId", U(25)),
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
            ("name", S("Field box")),
            ("parentId", U(0)),
            ("styleId", U(26)),
        ],
    ); // 13
    record(&mut b, card::FIELD_BOX, Some(180.0), Some(40.0)); // 14
    obj(
        &mut b,
        "TextInput",
        &[
            ("name", S("Field")),
            ("parentId", U(card::FIELD_BOX as u64)),
            ("text", S(FIELD_TEXT)),
        ],
    ); // 15
    obj(&mut b, "FocusData", &[("parentId", U(card::FIELD as u64))]); // 16
    obj(
        &mut b,
        "TextStylePaint",
        &[
            ("parentId", U(card::FIELD as u64)),
            ("fontAssetId", U(0)),
            ("fontSize", F(16.0)),
        ],
    ); // 17
    obj(
        &mut b,
        "TextInputText",
        &[("parentId", U(card::FIELD as u64))],
    ); // 18
    obj(&mut b, "Fill", &[("parentId", U(18))]); // 19
    obj(
        &mut b,
        "SolidColor",
        &[("parentId", U(19)), ("colorValue", Color(0xff00_0000))],
    ); // 20
    obj(&mut b, "Node", &[("name", S("Dot")), ("parentId", U(0))]); // 21
    obj(
        &mut b,
        "Event",
        &[("name", S("chosen")), ("parentId", U(0))],
    ); // 22
    for parent in [0, card::LABEL, card::BUTTON, card::FIELD_BOX] {
        obj(
            &mut b,
            "LayoutComponentStyle",
            &[("parentId", U(parent as u64))],
        ); // 23 to 26
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
    if badges {
        for (name, host) in [
            ("First badge", card::FIRST_BADGE),
            ("Second badge", card::SECOND_BADGE),
        ] {
            obj(
                &mut b,
                "NestedArtboardLayout",
                &[("name", S(name)), ("parentId", U(0)), ("artboardId", U(2))],
            ); // 27, 30
            record(&mut b, host, Some(60.0), None); // 28, 31
            obj(
                &mut b,
                "NestedStateMachine",
                &[("parentId", U(host as u64)), ("animationId", U(0))],
            ); // 29, 32
        }
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
                ("styleId", U(7)),
            ],
        ); // 0
        record(&mut b, 0, None, None); // 1
        obj(
            &mut b,
            "LayoutComponent",
            &[
                ("name", S("Tag")),
                ("parentId", U(0)),
                ("styleId", U(8)),
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
        obj(
            &mut b,
            "Event",
            &[("name", S("badge tapped")), ("parentId", U(0))],
        ); // 6
        for parent in [0, badge::TAG] {
            obj(
                &mut b,
                "LayoutComponentStyle",
                &[("parentId", U(parent as u64))],
            ); // 7, 8
        }
        obj(&mut b, "StateMachine", &[("name", S("badge"))]);
        click(&mut b, badge::TAG, badge::TAPPED);
        layer(&mut b, false);
    }
    b
}
