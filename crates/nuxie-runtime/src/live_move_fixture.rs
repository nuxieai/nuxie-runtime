//! A literal published live-move page, also imported through the C API tests.
#![allow(dead_code)]
use nuxie_schema::definition_by_name;
#[derive(Clone, Copy)]
enum V<'a> {
    U(u64),
    F(f32),
    S(&'a str),
    B(bool),
    Bytes(&'a [u8]),
    Color(u32),
}
fn var(b: &mut Vec<u8>, mut n: u64) {
    while n >= 128 {
        b.push((n as u8 & 127) | 128);
        n >>= 7;
    }
    b.push(n as u8);
}
fn key(kind: &str, name: &str) -> u16 {
    let d = definition_by_name(kind).unwrap_or_else(|| panic!("type {kind}"));
    std::iter::once(d.name)
        .chain(d.ancestors.iter().copied())
        .filter_map(definition_by_name)
        .flat_map(|d| d.properties)
        .find(|p| p.name == name)
        .unwrap_or_else(|| panic!("{kind}.{name}"))
        .key
        .int
}
fn obj(b: &mut Vec<u8>, kind: &str, props: &[(&str, V<'_>)]) {
    var(
        b,
        definition_by_name(kind)
            .unwrap_or_else(|| panic!("type {kind}"))
            .type_key
            .int
            .into(),
    );
    for (name, v) in props {
        var(b, key(kind, name).into());
        match v {
            V::U(n) => var(b, *n),
            V::F(v) => b.extend(v.to_le_bytes()),
            V::S(s) => {
                var(b, s.len() as u64);
                b.extend(s.as_bytes());
            }
            V::B(v) => b.push(u8::from(*v)),
            V::Bytes(v) => {
                var(b, v.len() as u64);
                b.extend(*v);
            }
            V::Color(c) => b.extend(c.to_le_bytes()),
        }
    }
    var(b, 0);
}
fn machine(b: &mut Vec<u8>, animated: bool) {
    use V::*;
    obj(b, "StateMachine", &[("name", S("run"))]);
    if animated {
        obj(b, "StateMachineLayer", &[("name", S("main"))]);
        obj(b, "EntryState", &[]);
        obj(b, "StateTransition", &[("stateToId", U(1))]);
        obj(b, "AnimationState", &[("animationId", U(0))]);
    }
}
fn timeline(b: &mut Vec<u8>, name: &str, target: u64, property: u64, a: f32, z: f32) {
    use V::*;
    obj(
        b,
        "LinearAnimation",
        &[
            ("name", S(name)),
            ("fps", U(60)),
            ("duration", U(60)),
            ("loopValue", U(1)),
        ],
    );
    obj(b, "KeyedObject", &[("objectId", U(target))]);
    obj(b, "KeyedProperty", &[("propertyKey", U(property))]);
    obj(
        b,
        "KeyFrameDouble",
        &[
            ("frame", U(0)),
            ("value", F(a)),
            ("interpolationType", U(1)),
        ],
    );
    obj(
        b,
        "KeyFrameDouble",
        &[
            ("frame", U(60)),
            ("value", F(z)),
            ("interpolationType", U(1)),
        ],
    );
}
fn bind(b: &mut Vec<u8>, property: u64, path: &[u8]) {
    use V::*;
    obj(
        b,
        "DataBindContext",
        &[
            ("propertyKey", U(property)),
            ("sourcePathIds", Bytes(path)),
            ("flags", U(0)),
        ],
    );
}
fn slot(
    b: &mut Vec<u8>,
    parent: u64,
    host_id: u64,
    name: &str,
    unit: u64,
    x: f32,
    screen_model: u8,
    value_props: bool,
    text_two_way: bool,
) {
    slot_with_props(b, parent, host_id, name, unit, x, Some(screen_model));
    if value_props {
        for (kind, id, value, key) in [
            ("ViewModelInstanceColor", 2, V::Color(0), 555),
            ("ViewModelInstanceString", 3, V::S(""), 561),
            ("ViewModelInstanceString", 4, V::S(""), 561),
        ] {
            obj(
                b,
                kind,
                &[
                    ("parentId", V::U(host_id + 3)),
                    ("viewModelPropertyId", V::U(id)),
                    ("propertyValue", value),
                ],
            );
            if id != 4 || screen_model != 1 {
                obj(
                    b,
                    "DataBindContext",
                    &[
                        ("propertyKey", V::U(key)),
                        ("sourcePathIds", V::Bytes(&[screen_model, (id - 1) as u8])),
                        ("flags", V::U(if id == 4 && text_two_way { 2 } else { 0 })),
                    ],
                );
            }
        }
    }
}
fn slot_with_props(
    b: &mut Vec<u8>,
    parent: u64,
    host_id: u64,
    name: &str,
    unit: u64,
    x: f32,
    screen_model: Option<u8>,
) {
    use V::*;
    obj(
        b,
        "NestedArtboard",
        &[
            ("name", S(name)),
            ("parentId", U(parent)),
            ("artboardId", U(unit)),
            ("x", F(x)),
            ("isStateful", B(true)),
        ],
    );
    obj(
        b,
        "NestedStateMachine",
        &[("parentId", U(host_id)), ("animationId", U(0))],
    );
    obj(
        b,
        "NestedSimpleAnimation",
        &[
            ("parentId", U(host_id)),
            ("animationId", U(1)),
            ("isPlaying", B(true)),
        ],
    );
    obj(
        b,
        "ViewModelInstance",
        &[("viewModelId", U(0)), ("parentId", U(host_id))],
    );
    obj(
        b,
        "ViewModelInstanceNumber",
        &[
            ("parentId", U(host_id + 3)),
            ("viewModelPropertyId", U(0)),
            ("propertyValue", F(0.0)),
        ],
    );
    obj(
        b,
        "ViewModelInstanceNumber",
        &[
            ("parentId", U(host_id + 3)),
            ("viewModelPropertyId", U(1)),
            ("propertyValue", F(0.0)),
        ],
    );
    if let Some(screen_model) = screen_model {
        bind(b, 575, &[screen_model, 0]);
    }
}

pub fn fixture(rows: usize) -> Vec<u8> {
    build(rows, Options::default())
}
/// Live steps and marked slots under the compiler's root naming contract.
pub fn live_page_fixture() -> Vec<u8> {
    build(
        2,
        Options {
            page_shape: true,
            ..Options::default()
        },
    )
}
/// Decoder lifecycle regression, separate from all exported device fixtures.
pub fn playing_video_fixture() -> Vec<u8> {
    build(
        2,
        Options {
            playing_video: true,
            ..Options::default()
        },
    )
}
pub fn control_fixture() -> Vec<u8> {
    build(
        2,
        Options {
            controls: true,
            ..Options::default()
        },
    )
}
/// Slots and unit nodes have explicit semantics for snapshot identity tests.
pub fn semantic_fixture() -> Vec<u8> {
    build(
        2,
        Options {
            semantics: true,
            ..Options::default()
        },
    )
}
/// Drawing regression with actual per-row path tangents and unequal row scales.
pub fn follow_path_fixture() -> Vec<u8> {
    build(
        2,
        Options {
            follow_path: true,
            ..Options::default()
        },
    )
}
/// A unit can itself host a list. Tests populate it through the native list API.
pub fn semantic_nested_list_fixture() -> Vec<u8> {
    build(
        2,
        Options {
            semantics: true,
            nested_list: true,
            ..Options::default()
        },
    )
}
/// The page is supplied through a global override, without a main-model bind.
pub fn global_fixture() -> Vec<u8> {
    build(
        2,
        Options {
            globals: true,
            ..Options::default()
        },
    )
}
#[derive(Default)]
struct Options {
    parity_counter: bool,
    controls: bool,
    playing_video: bool,
    semantics: bool,
    scripted: bool,
    keyed: bool,
    two_versions: bool,
    ordinary: bool,
    page_shape: bool,
    scale: bool,
    invalid_plan: u8,
    incoming_away: bool,
    overlapping_units: bool,
    follow_path: bool,
    nested_list: bool,
    globals: bool,
    page_unit: bool,
    value_props: bool,
    text_two_way: bool,
    paint_list: bool,
    unnamed_screen: bool,
    empty_screens: bool,
}
/// Number, color, switched string and locally bound text on the same unit.
pub fn values_fixture(text_two_way: bool) -> Vec<u8> {
    build(
        2,
        Options {
            value_props: true,
            text_two_way,
            ..Options::default()
        },
    )
}
pub fn two_version_values_fixture() -> Vec<u8> {
    build(
        2,
        Options {
            value_props: true,
            two_versions: true,
            ..Options::default()
        },
    )
}
pub fn scripted_fixture() -> Vec<u8> {
    build(
        2,
        Options {
            scripted: true,
            ..Options::default()
        },
    )
}
pub fn keyed_fixture() -> Vec<u8> {
    build(
        2,
        Options {
            controls: true,
            keyed: true,
            ..Options::default()
        },
    )
}
/// Deliberately malformed published plans for import/prepare validation.
/// 1: unknown flight, 2: mismatched unit artboard, 3: duplicate flight key.
pub fn invalid_plan_fixture(kind: u8) -> Vec<u8> {
    build(
        2,
        Options {
            two_versions: true,
            invalid_plan: kind,
            ..Options::default()
        },
    )
}
pub fn two_version_incoming_away_fixture() -> Vec<u8> {
    build(
        3,
        Options {
            two_versions: true,
            incoming_away: true,
            ..Options::default()
        },
    )
}
/// Two distinct mappings share both unit definitions but use four instances.
/// Kinds 4 to 6 deliberately overlap an incoming unit, outgoing unit or flight.
pub fn overlapping_plan_fixture(kind: u8) -> Vec<u8> {
    build(
        2,
        Options {
            two_versions: true,
            overlapping_units: true,
            invalid_plan: kind,
            ..Options::default()
        },
    )
}
/// An explicit Unit model stored by the root scene rather than a Screen row.
pub fn root_owned_vm_fixture() -> Vec<u8> {
    build(
        2,
        Options {
            page_unit: true,
            ..Options::default()
        },
    )
}
pub fn two_version_fixture() -> Vec<u8> {
    build(
        2,
        Options {
            two_versions: true,
            ..Options::default()
        },
    )
}
pub fn two_version_mixed_size_fixture() -> Vec<u8> {
    build(
        2,
        Options {
            two_versions: true,
            scale: true,
            ..Options::default()
        },
    )
}
/// Record-free page using the SDK root naming contract.
pub fn record_free_page_fixture() -> Vec<u8> {
    build(
        2,
        Options {
            ordinary: true,
            page_shape: true,
            ..Options::default()
        },
    )
}
pub fn ordinary_fixture(rows: usize) -> Vec<u8> {
    build(
        rows,
        Options {
            ordinary: true,
            ..Options::default()
        },
    )
}
pub fn scale_fixture() -> Vec<u8> {
    build(
        2,
        Options {
            scale: true,
            ..Options::default()
        },
    )
}
pub fn paint_list_fixture() -> Vec<u8> {
    build(
        2,
        Options {
            paint_list: true,
            nested_list: true,
            ..Options::default()
        },
    )
}
/// R4 carries misplaced slots; named screens must not depend on default rows.
pub fn endpoint_shape_fixture(empty_screens: bool) -> Vec<u8> {
    build(
        5,
        Options {
            unnamed_screen: true,
            empty_screens,
            ..Options::default()
        },
    )
}
pub fn command_fixture(payload: &[u8]) -> Vec<u8> {
    build_with_script(2, Options::default(), Some(payload))
}
pub fn command_parity_fixture(payload: &[u8]) -> Vec<u8> {
    build_with_script(
        2,
        Options {
            parity_counter: true,
            ..Options::default()
        },
        Some(payload),
    )
}
fn build(rows: usize, options: Options) -> Vec<u8> {
    build_with_script(rows, options, None)
}
fn build_with_script(rows: usize, options: Options, script: Option<&[u8]>) -> Vec<u8> {
    let controls = options.controls;
    let semantics = options.semantics;
    use V::*;
    assert!(rows >= 1);
    let mut b = b"RIVE".to_vec();
    for n in [7, 0, 0, 0] {
        var(&mut b, n);
    }
    obj(&mut b, "Backboard", &[]);
    let mut video_asset = vec![
        ("sourceKey", S("asset:mascot")),
        ("contentType", S("video/mp4")),
    ];
    if options.playing_video {
        video_asset.extend([("width", F(40.0)), ("height", F(40.0))]);
    }
    if script.is_none() {
        obj(&mut b, "VideoAsset", &video_asset);
    }
    if let Some(payload) = script {
        obj(
            &mut b,
            "ScriptAsset",
            &[("assetId", U(0)), ("name", S("MovingUnit"))],
        );
        obj(&mut b, "FileAssetContents", &[("bytes", Bytes(payload))]);
    }
    let mut unit = vec![("name", S("Unit"))];
    if options.globals {
        unit.push(("viewModelType", U(2)));
    }
    obj(&mut b, "ViewModel", &unit);
    for name in ["count", "prop"] {
        obj(&mut b, "ViewModelPropertyNumber", &[("name", S(name))]);
    }
    if options.value_props {
        for (kind, name) in [
            ("ViewModelPropertyColor", "accent"),
            ("ViewModelPropertyString", "label"),
            ("ViewModelPropertyString", "text"),
        ] {
            obj(&mut b, kind, &[("name", S(name))]);
        }
    }
    obj(&mut b, "ViewModelInstance", &[("viewModelId", U(0))]);
    for i in 0..2 {
        obj(
            &mut b,
            "ViewModelInstanceNumber",
            &[("viewModelPropertyId", U(i)), ("propertyValue", F(0.0))],
        );
    }
    if options.value_props {
        for (kind, id, value) in [
            ("ViewModelInstanceColor", 2, Color(0)),
            ("ViewModelInstanceString", 3, S("")),
            ("ViewModelInstanceString", 4, S("")),
        ] {
            obj(
                &mut b,
                kind,
                &[("viewModelPropertyId", U(id)), ("propertyValue", value)],
            );
        }
    }
    for i in 0..rows {
        let name = format!("Screen{i}");
        obj(&mut b, "ViewModel", &[("name", S(&name))]);
        obj(&mut b, "ViewModelPropertyNumber", &[("name", S("offset"))]);
        if options.value_props {
            for (kind, name) in [
                ("ViewModelPropertyColor", "accent"),
                ("ViewModelPropertyString", "label"),
                ("ViewModelPropertyString", "text"),
            ] {
                obj(&mut b, kind, &[("name", S(name))]);
            }
        }
        obj(
            &mut b,
            "ViewModelInstance",
            &[("viewModelId", U(i as u64 + 1))],
        );
        obj(
            &mut b,
            "ViewModelInstanceNumber",
            &[
                ("viewModelPropertyId", U(0)),
                ("propertyValue", F(10.0 + i as f32 * 190.0)),
            ],
        );
        if options.value_props {
            for (kind, id, value) in [
                (
                    "ViewModelInstanceColor",
                    1,
                    Color(if i == 0 { 0x20406080 } else { 0xe0c0a080 }),
                ),
                (
                    "ViewModelInstanceString",
                    2,
                    S(if i == 0 { "outgoing" } else { "incoming" }),
                ),
                ("ViewModelInstanceString", 3, S("destination text")),
            ] {
                obj(
                    &mut b,
                    kind,
                    &[("viewModelPropertyId", U(id)), ("propertyValue", value)],
                );
            }
        }
    }
    let mut page = vec![("name", S("Page"))];
    if options.globals {
        page.push(("viewModelType", U(2)));
    }
    obj(&mut b, "ViewModel", &page);
    obj(&mut b, "ViewModelPropertyList", &[("name", S("screens"))]);
    if options.parity_counter {
        obj(&mut b, "ViewModelPropertyNumber", &[("name", S("counter"))]);
    }

    obj(
        &mut b,
        "ViewModelInstance",
        &[("viewModelId", U(rows as u64 + 1))],
    );
    if options.parity_counter {
        obj(
            &mut b,
            "ViewModelInstanceNumber",
            &[("viewModelPropertyId", U(1)), ("propertyValue", F(0.0))],
        );
    }
    obj(
        &mut b,
        "ViewModelInstanceList",
        &[("viewModelPropertyId", U(0))],
    );
    for i in 0..if options.empty_screens { 0 } else { rows } {
        obj(
            &mut b,
            "ViewModelInstanceListItem",
            &[
                ("viewModelId", U(i as u64 + 1)),
                ("viewModelInstanceId", U(0)),
            ],
        );
    }
    obj(
        &mut b,
        "Artboard",
        &[
            (
                "name",
                S(if options.page_shape {
                    "$nuxie.page"
                } else {
                    "Page"
                }),
            ),
            ("width", F(800.0)),
            ("height", F(600.0)),
            ("viewModelId", U(rows as u64 + 1)),
            ("clip", B(false)),
            ("defaultStateMachineId", U(0)),
        ],
    );
    obj(
        &mut b,
        "Node",
        &[("name", S("overlap")), ("parentId", U(0))],
    ); // 1
    if options.parity_counter {
        obj(
            &mut b,
            "DataBindContext",
            &[
                ("propertyKey", U(13)),
                ("sourcePathIds", Bytes(&[rows as u8 + 1, 1])),
                ("flags", U(1)),
            ],
        );
    }
    obj(
        &mut b,
        "ArtboardComponentList",
        &[("name", S("screens")), ("parentId", U(1))],
    ); // 2
    bind(&mut b, 800, &[rows as u8 + 1, 0]);
    if rows > 1 && !options.ordinary {
        obj(
            &mut b,
            "LiveStep",
            &[
                ("parentId", U(0)),
                ("name", S("t1")),
                ("fromArtboard", S("A")),
                ("toArtboard", S("B")),
            ],
        ); // 3
        obj(
            &mut b,
            "LiveStepUnit",
            &[
                ("parentId", U(3)),
                (
                    "incomingUnitKey",
                    U(if options.two_versions { 2 } else { 1 }),
                ),
                ("outgoingUnitKey", U(1)),
                (
                    "incomingFlightKey",
                    U(if options.invalid_plan == 1 {
                        999
                    } else if options.two_versions {
                        12
                    } else {
                        2
                    }),
                ),
                (
                    "outgoingFlightKey",
                    U(if options.two_versions { 11 } else { 2 }),
                ),
            ],
        );
        obj(
            &mut b,
            "LiveStep",
            &[
                ("parentId", U(0)),
                ("name", S("back")),
                ("fromArtboard", S("B")),
                ("toArtboard", S("A")),
            ],
        ); // 5
        obj(
            &mut b,
            "LiveStepUnit",
            &[
                ("parentId", U(5)),
                ("incomingUnitKey", U(1)),
                (
                    "outgoingUnitKey",
                    U(if options.two_versions { 2 } else { 1 }),
                ),
                (
                    "incomingFlightKey",
                    U(if options.two_versions { 14 } else { 2 }),
                ),
                (
                    "outgoingFlightKey",
                    U(if options.two_versions { 13 } else { 2 }),
                ),
            ],
        );
    }
    if options.overlapping_units {
        obj(
            &mut b,
            "LiveStepUnit",
            &[
                ("parentId", U(3)),
                (
                    "incomingUnitKey",
                    U(if options.invalid_plan == 4 { 2 } else { 1 }),
                ),
                (
                    "outgoingUnitKey",
                    U(if options.invalid_plan == 5 { 1 } else { 2 }),
                ),
                (
                    "incomingFlightKey",
                    U(match options.invalid_plan {
                        4 => 13,
                        5 | 6 => 11,
                        _ => 14,
                    }),
                ),
                (
                    "outgoingFlightKey",
                    U(if options.invalid_plan == 5 { 11 } else { 13 }),
                ),
            ],
        );
    }
    if rows > 2 && !options.ordinary {
        obj(
            &mut b,
            "LiveStep",
            &[
                ("parentId", U(0)),
                ("name", S("third")),
                ("fromArtboard", S("B")),
                ("toArtboard", S("R2")),
            ],
        );
        obj(
            &mut b,
            "LiveStepUnit",
            &[
                ("parentId", U(7)),
                (
                    "incomingUnitKey",
                    U(if options.incoming_away { 2 } else { 1 }),
                ),
                (
                    "outgoingUnitKey",
                    U(if options.incoming_away { 2 } else { 1 }),
                ),
                (
                    "incomingFlightKey",
                    U(if options.incoming_away { 22 } else { 3 }),
                ),
                (
                    "outgoingFlightKey",
                    U(if options.incoming_away { 22 } else { 3 }),
                ),
            ],
        );
    }
    // Non-flying rows still have compiler-authored zero-unit steps. Keep
    // their declarations eligible without depending on the default list.
    if !options.ordinary {
        for i in 3..rows - usize::from(options.unnamed_screen) {
            obj(
                &mut b,
                "LiveStep",
                &[
                    ("parentId", U(0)),
                    ("name", S(&format!("cut{i}"))),
                    ("fromArtboard", S("A")),
                    ("toArtboard", S(&format!("R{i}"))),
                ],
            );
        }
    }
    if options.page_unit {
        slot_with_props(&mut b, 0, 7, "root unit", rows as u64 + 1, 0.0, None);
        obj(
            &mut b,
            "LiveSlot",
            &[
                ("parentId", U(7)),
                ("unitKey", U(1)),
                ("role", U(0)),
                ("sizeMode", U(1)),
            ],
        );
    }
    if options.paint_list {
        obj(&mut b, "Shape", &[("parentId", U(0))]); // 7
        obj(
            &mut b,
            "Rectangle",
            &[("parentId", U(7)), ("width", F(100.0)), ("height", F(80.0))],
        ); // 8
        obj(&mut b, "Fill", &[("parentId", U(7))]); // 9
        obj(
            &mut b,
            "PaintImage",
            &[
                ("name", S("painted")),
                ("parentId", U(9)),
                ("artboardId", U(rows as u64 + 1)),
            ],
        ); // 10
    }
    if options.follow_path {
        obj(&mut b, "Shape", &[("name", S("route")), ("parentId", U(0))]); // 7
        obj(
            &mut b,
            "PointsPath",
            &[("parentId", U(7)), ("isClosed", B(false))],
        ); // 8
        for (x, y) in [(0.0, 0.0), (100.0, 0.0), (100.0, 200.0)] {
            obj(
                &mut b,
                "StraightVertex",
                &[("parentId", U(8)), ("x", F(x)), ("y", F(y))],
            );
        }
        obj(
            &mut b,
            "ListFollowPathConstraint",
            &[
                ("parentId", U(2)),
                ("targetId", U(7)),
                ("distance", F(0.15)),
                ("distanceEnd", F(0.85)),
                ("orient", B(true)),
            ],
        );
    }
    if options.parity_counter {
        timeline(&mut b, "counter", 1, 13, 0.0, 100.0);
    }
    machine(&mut b, options.parity_counter);
    for i in 0..rows {
        let name = if i == 0 {
            "A".to_owned()
        } else if i == 1 {
            "B".to_owned()
        } else {
            format!("R{i}")
        };
        obj(
            &mut b,
            "Artboard",
            &[
                ("name", S(&name)),
                ("width", F(800.0)),
                ("height", F(600.0)),
                ("viewModelId", U(i as u64 + 1)),
                ("clip", B(false)),
                ("defaultStateMachineId", U(0)),
            ],
        );
        let parent = if i == 0 { "card" } else { "header" };
        obj(
            &mut b,
            "Node",
            &[
                ("name", S(parent)),
                ("parentId", U(0)),
                ("x", F(if i == 0 { 10.0 } else { 200.0 })),
            ],
        ); // 1
        let clipped = i == 1;
        if clipped {
            obj(
                &mut b,
                "LayoutComponent",
                &[
                    ("name", S("clip")),
                    ("parentId", U(1)),
                    ("width", F(300.0)),
                    ("height", F(200.0)),
                    ("clip", B(true)),
                ],
            ); // 2
        }
        let host_id = if clipped { 3 } else { 2 };
        slot(
            &mut b,
            if clipped { 2 } else { 1 },
            host_id,
            "mascot",
            rows as u64
                + 1
                + u64::from(options.two_versions && (i == 1 || options.incoming_away && i == 2)),
            5.0,
            i as u8 + 1,
            options.value_props,
            options.text_two_way,
        );
        let slot_width = if options.value_props { 9 } else { 6 };
        let layer_id = host_id + slot_width;
        obj(
            &mut b,
            "Node",
            &[("name", S("flight")), ("parentId", U(0)), ("x", F(35.0))],
        );
        let flight_id = layer_id + 1;
        if options.two_versions {
            // Rive draws sibling objects in reverse file order. Author B then
            // A for forward, A then B for back, so incoming draws above.
            // The same Screen owns both named pairs; runtime never reorders.
            for (j, key) in [2, 1, 1, 2].into_iter().enumerate() {
                let id = layer_id + 1 + j as u64 * (slot_width + 1);
                slot(
                    &mut b,
                    layer_id,
                    id,
                    if key == 1 {
                        "version A flight"
                    } else {
                        "version B flight"
                    },
                    rows as u64 + key + u64::from(options.invalid_plan == 2 && i == 1 && j == 1),
                    0.0,
                    i as u8 + 1,
                    options.value_props,
                    options.text_two_way,
                );
                obj(
                    &mut b,
                    "LiveSlot",
                    &[
                        ("parentId", U(id)),
                        ("unitKey", U(key)),
                        ("role", U(1)),
                        (
                            "flightKey",
                            U(i as u64 * 10
                                + if options.invalid_plan == 3 && j == 1 {
                                    2
                                } else {
                                    [2, 1, 4, 3][j]
                                }),
                        ),
                        ("sizeMode", U(if options.scale && key == 2 { 0 } else { 1 })),
                    ],
                );
            }
        } else {
            slot(
                &mut b,
                layer_id,
                flight_id,
                "mascot",
                rows as u64 + 1,
                0.0,
                i as u8 + 1,
                options.value_props,
                options.text_two_way,
            );
        }
        if !options.ordinary {
            obj(
                &mut b,
                "LiveSlot",
                &[
                    ("parentId", U(host_id)),
                    (
                        "unitKey",
                        U(
                            if options.two_versions && (i == 1 || options.incoming_away && i == 2) {
                                2
                            } else {
                                1
                            },
                        ),
                    ),
                    ("role", U(0)),
                    ("sizeMode", U(1)),
                ],
            );
            if !options.two_versions {
                obj(
                    &mut b,
                    "LiveSlot",
                    &[
                        ("parentId", U(flight_id)),
                        ("unitKey", U(1)),
                        ("role", U(1)),
                        ("flightKey", U(i as u64 + 1)),
                        ("sizeMode", U(if options.scale { 0 } else { 1 })),
                        ("progress", F(if controls { 0.25 } else { 0.0 })),
                        ("pathX", F(if controls { 7.0 } else { 0.0 })),
                        ("pathY", F(if controls { 11.0 } else { 0.0 })),
                    ],
                );
            }
        }
        if options.overlapping_units {
            let id = host_id + 36;
            let key = if i == 0 { 2 } else { 1 };
            slot(
                &mut b,
                0,
                id,
                "second unit",
                rows as u64 + key,
                80.0,
                i as u8 + 1,
                options.value_props,
                options.text_two_way,
            );
            obj(
                &mut b,
                "LiveSlot",
                &[("parentId", U(id)), ("unitKey", U(key)), ("role", U(0))],
            );
        }
        if semantics {
            for (parent, label) in [(host_id, "slot"), (flight_id, "flight slot")] {
                obj(
                    &mut b,
                    "SemanticData",
                    &[("parentId", U(parent)), ("role", U(9)), ("label", S(label))],
                );
            }
        }
        if i == 1 {
            timeline(&mut b, "entrance", 1, 13, 200.0, 300.0);
            if options.keyed {
                obj(&mut b, "KeyedObject", &[("objectId", U(host_id + 14))]);
                obj(&mut b, "KeyedProperty", &[("propertyKey", U(62003))]);
                obj(
                    &mut b,
                    "KeyFrameDouble",
                    &[
                        ("frame", U(0)),
                        ("value", F(0.0)),
                        ("interpolationType", U(1)),
                    ],
                );
                obj(
                    &mut b,
                    "KeyFrameDouble",
                    &[
                        ("frame", U(60)),
                        ("value", F(1.0)),
                        ("interpolationType", U(1)),
                    ],
                );
                obj(&mut b, "KeyedProperty", &[("propertyKey", U(62014))]);
                for (frame, value) in [(0, 0.0), (60, 1.0)] {
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
            }
        }
        machine(&mut b, i == 1);
    }
    for version in 0..if options.two_versions || options.paint_list {
        2
    } else {
        1
    } {
        obj(
            &mut b,
            "Artboard",
            &[
                ("name", S(if version == 0 { "mascot" } else { "mascot B" })),
                ("styleId", U(9)),
                ("width", F(100.0)),
                ("height", F(80.0)),
                ("viewModelId", U(0)),
                ("clip", B(false)),
                ("defaultStateMachineId", U(0)),
            ],
        );
        obj(&mut b, "Node", &[("name", S("keyed")), ("parentId", U(0))]); // 1
        obj(
            &mut b,
            "TextInput",
            &[("name", S("edit")), ("parentId", U(0)), ("text", S(""))],
        ); // 2
        if options.value_props {
            obj(
                &mut b,
                "DataBindContext",
                &[
                    ("propertyKey", U(817)),
                    ("sourcePathIds", Bytes(&[0, 4])),
                    ("flags", U(2)),
                ],
            );
        }
        // One visible rectangle makes draw counts concrete, not host-loop counters.
        obj(
            &mut b,
            "Shape",
            &[("name", S("button")), ("parentId", U(0))],
        ); // 3
        obj(
            &mut b,
            "Rectangle",
            &[
                ("parentId", U(3)),
                ("width", F(40.0)),
                ("height", F(40.0)),
                ("x", F(20.0)),
                ("y", F(20.0)),
            ],
        ); // 4
        obj(&mut b, "Fill", &[("parentId", U(3))]); // 5
        obj(
            &mut b,
            "SolidColor",
            &[
                ("parentId", U(5)),
                (
                    "colorValue",
                    Color(if version == 0 { 0xffff0000 } else { 0xff0000ff }),
                ),
            ],
        ); // 6
        let mut video = vec![("name", S("clip")), ("parentId", U(0)), ("assetId", U(0))];
        if options.playing_video {
            video.extend([
                ("autoplay", U(1)),
                ("paused", B(false)),
                ("position", F(2.5)),
            ]);
        }
        if script.is_some() {
            obj(&mut b, "Node", &[("name", S("clip")), ("parentId", U(0))]);
        } else {
            obj(&mut b, "Video", &video);
        }
        obj(&mut b, "FocusData", &[("parentId", U(2))]);
        obj(&mut b, "LayoutComponentStyle", &[("parentId", U(0))]); // 9
        obj(
            &mut b,
            "LayoutComponent",
            &[
                ("name", S("reflow")),
                ("parentId", U(0)),
                ("width", F(100.0)),
                ("height", F(10.0)),
                ("styleId", U(11)),
            ],
        ); // 10
        obj(
            &mut b,
            "LayoutComponentStyle",
            &[("parentId", U(10)), ("layoutWidthScaleType", U(1))],
        ); // 11

        if semantics {
            for (parent, role, label) in [
                (0, 9, "mascot unit"),
                (2, 6, "mascot field"),
                (3, 1, "mascot button"),
            ] {
                obj(
                    &mut b,
                    "SemanticData",
                    &[
                        ("parentId", U(parent)),
                        ("role", U(role)),
                        ("label", S(label)),
                    ],
                );
            }
        }
        if options.nested_list {
            obj(
                &mut b,
                "ArtboardComponentList",
                &[("name", S("inner")), ("parentId", U(0))],
            );
        }
        if options.paint_list {
            // Ordinary paint content has no Page.screens. Ignore even a
            // malformed plan there before validating supported Page plans.
            obj(
                &mut b,
                "LiveStep",
                &[
                    ("parentId", U(0)),
                    ("name", S("ignored")),
                    ("fromArtboard", S("A")),
                    ("toArtboard", S("B")),
                ],
            ); // 13, after inner
            for _ in 0..2 {
                obj(
                    &mut b,
                    "LiveStepUnit",
                    &[
                        ("parentId", U(13)),
                        ("incomingUnitKey", U(1)),
                        ("outgoingUnitKey", U(1)),
                        ("incomingFlightKey", U(1)),
                        ("outgoingFlightKey", U(1)),
                    ],
                );
            }
        }
        if script.is_some() {
            obj(
                &mut b,
                "ScriptedDrawable",
                &[
                    ("parentId", U(0)),
                    ("name", S("script")),
                    ("scriptAssetId", U(0)),
                ],
            ); // 12
            obj(
                &mut b,
                "ScriptInputNumber",
                &[
                    ("parentId", U(12)),
                    ("name", S("value")),
                    ("propertyValue", F(0.0)),
                ],
            );
            bind(
                &mut b,
                key("ScriptInputNumber", "propertyValue").into(),
                &[0, 1],
            );
        }
        if options.scripted {
            obj(
                &mut b,
                "ScriptedDrawable",
                &[("parentId", U(0)), ("name", S("script"))],
            );
        }
        timeline(&mut b, "walk", 1, 13, 0.0, 100.0);
        timeline(&mut b, "bob", 1, 14, 0.0, 100.0);
        machine(&mut b, true);
        obj(
            &mut b,
            "StateMachineNumber",
            &[("name", S("hits")), ("value", F(0.0))],
        );
        obj(
            &mut b,
            "StateMachineListenerSingle",
            &[("targetId", U(3)), ("listenerTypeValue", U(2))],
        );
        obj(
            &mut b,
            "ListenerNumberChange",
            &[("inputId", U(0)), ("value", F(9.0))],
        );
    }
    b
}

/// A flex sibling after a NestedArtboardLayout, exercising its real layout node.
pub fn layout_fixture() -> Vec<u8> {
    build_layout_fixture(false)
}
pub fn layout_flight_fixture() -> Vec<u8> {
    build_layout_fixture(true)
}
fn build_layout_fixture(flight: bool) -> Vec<u8> {
    use V::*;
    let mut b = b"RIVE".to_vec();
    for n in [7, 0, 0, 0] {
        var(&mut b, n);
    }
    obj(&mut b, "Backboard", &[]);
    obj(&mut b, "ViewModel", &[("name", S("LayoutScreen"))]);
    obj(&mut b, "ViewModelInstance", &[("viewModelId", U(0))]);
    obj(&mut b, "ViewModel", &[("name", S("LayoutPage"))]);
    obj(&mut b, "ViewModelPropertyList", &[("name", S("screens"))]);
    obj(&mut b, "ViewModelInstance", &[("viewModelId", U(1))]);
    obj(
        &mut b,
        "ViewModelInstanceList",
        &[("viewModelPropertyId", U(0))],
    );
    obj(
        &mut b,
        "ViewModelInstanceListItem",
        &[("viewModelId", U(0)), ("viewModelInstanceId", U(0))],
    );
    obj(
        &mut b,
        "Artboard",
        &[
            ("name", S("LayoutScreen")),
            ("viewModelId", U(0)),
            ("width", F(400.0)),
            ("height", F(100.0)),
            ("styleId", U(1)),
            ("clip", B(false)),
        ],
    ); // 0
    obj(
        &mut b,
        "LayoutComponentStyle",
        &[("parentId", U(0)), ("flexDirectionValue", U(2))],
    ); // 1
    obj(
        &mut b,
        "NestedArtboardLayout",
        &[
            ("name", S("source")),
            ("parentId", U(0)),
            ("artboardId", U(1)),
            ("instanceWidth", F(150.0)),
            ("instanceHeight", F(80.0)),
        ],
    ); // 2
    obj(
        &mut b,
        "LayoutComponent",
        &[
            ("name", S("sibling")),
            ("parentId", U(0)),
            ("width", F(20.0)),
            ("height", F(80.0)),
            ("styleId", U(4)),
        ],
    ); // 3
    obj(&mut b, "LayoutComponentStyle", &[("parentId", U(3))]); // 4
    obj(
        &mut b,
        "Node",
        &[("name", S("overlay")), ("parentId", U(0))],
    ); // 5
    obj(
        &mut b,
        "NestedArtboard",
        &[
            ("name", S("destination")),
            ("parentId", U(5)),
            ("artboardId", U(1)),
        ],
    ); // 6
    if flight {
        obj(
            &mut b,
            "NestedArtboardLayout",
            &[
                ("name", S("flight")),
                ("parentId", U(0)),
                ("artboardId", U(1)),
                ("instanceWidth", F(150.0)),
                ("instanceHeight", F(80.0)),
            ],
        ); // 7
        obj(
            &mut b,
            "LiveSlot",
            &[
                ("parentId", U(7)),
                ("unitKey", U(1)),
                ("role", U(1)),
                ("flightKey", U(1)),
                ("sizeMode", U(1)),
            ],
        );
    }
    // Both endpoints are physical slots in the Screen row.
    obj(&mut b, "LiveSlot", &[("parentId", U(2)), ("unitKey", U(1))]);
    obj(&mut b, "LiveSlot", &[("parentId", U(6)), ("unitKey", U(2))]);
    obj(
        &mut b,
        "Artboard",
        &[
            ("name", S("unit")),
            ("width", F(100.0)),
            ("height", F(80.0)),
            ("styleId", U(2)),
            ("clip", B(false)),
        ],
    );
    obj(&mut b, "Node", &[("name", S("inside")), ("parentId", U(0))]);
    obj(&mut b, "LayoutComponentStyle", &[("parentId", U(0))]);
    obj(
        &mut b,
        "Artboard",
        &[
            ("name", S("LayoutPage")),
            ("viewModelId", U(1)),
            ("width", F(400.0)),
            ("height", F(100.0)),
            ("clip", B(false)),
            ("defaultStateMachineId", U(0)),
        ],
    );
    obj(
        &mut b,
        "ArtboardComponentList",
        &[("name", S("screens")), ("parentId", U(0))],
    );
    bind(&mut b, 800, &[1, 0]);
    obj(
        &mut b,
        "LiveStep",
        &[
            ("parentId", U(0)),
            ("name", S("layout")),
            ("fromArtboard", S("LayoutScreen")),
            ("toArtboard", S("LayoutScreen")),
        ],
    );
    machine(&mut b, false);
    b
}

/// A malformed record under a Node, for import diagnostics and drop behavior.
pub fn invalid_parent_fixture() -> Vec<u8> {
    use V::*;
    let mut b = b"RIVE".to_vec();
    for n in [7, 0, 0, 0] {
        var(&mut b, n);
    }
    obj(&mut b, "Backboard", &[]);
    obj(&mut b, "Artboard", &[("name", S("Bad"))]);
    obj(&mut b, "Node", &[("parentId", U(0))]);
    obj(&mut b, "LiveSlot", &[("parentId", U(1)), ("unitKey", U(1))]);
    b
}

/// Real playback writes a wide graph through authored to-source bindings.
/// Two instances share one schema, keeping import metadata within its bound.
/// Separate from every device fixture and compiled only with test-support.
pub fn checkpoint_overflow_fixture(properties: usize) -> Vec<u8> {
    use V::*;
    let width = properties.div_ceil(2);
    let mut b = b"RIVE".to_vec();
    for n in [7, 0, 0, 0] {
        var(&mut b, n);
    }
    obj(&mut b, "Backboard", &[]);
    obj(&mut b, "ViewModel", &[("name", S("Values"))]);
    for p in 0..width {
        obj(
            &mut b,
            "ViewModelPropertyNumber",
            &[("name", S(&format!("p{p}")))],
        );
    }
    for _ in 0..2 {
        obj(&mut b, "ViewModelInstance", &[("viewModelId", U(0))]);
        for p in 0..width {
            obj(
                &mut b,
                "ViewModelInstanceNumber",
                &[
                    ("viewModelPropertyId", U(p as u64)),
                    ("propertyValue", F(0.0)),
                ],
            );
        }
    }
    obj(&mut b, "ViewModel", &[("name", S("Screen"))]);
    obj(&mut b, "ViewModelPropertyNumber", &[("name", S("offset"))]);
    obj(&mut b, "ViewModelInstance", &[("viewModelId", U(1))]);
    obj(
        &mut b,
        "ViewModelInstanceNumber",
        &[("viewModelPropertyId", U(0)), ("propertyValue", F(0.0))],
    );
    obj(&mut b, "ViewModel", &[("name", S("Page"))]);
    obj(&mut b, "ViewModelPropertyList", &[("name", S("screens"))]);
    for name in ["left", "right"] {
        obj(
            &mut b,
            "ViewModelPropertyViewModel",
            &[("name", S(name)), ("viewModelReferenceId", U(0))],
        );
    }
    obj(&mut b, "ViewModelInstance", &[("viewModelId", U(2))]);
    obj(
        &mut b,
        "ViewModelInstanceList",
        &[("viewModelPropertyId", U(0))],
    );
    obj(
        &mut b,
        "ViewModelInstanceListItem",
        &[("viewModelId", U(1)), ("viewModelInstanceId", U(0))],
    );
    for p in 0..2 {
        obj(
            &mut b,
            "ViewModelInstanceViewModel",
            &[("viewModelPropertyId", U(p + 1)), ("propertyValue", U(p))],
        );
    }
    obj(
        &mut b,
        "Artboard",
        &[
            ("name", S("Page")),
            ("width", F(400.0)),
            ("height", F(100.0)),
            ("viewModelId", U(2)),
            ("defaultStateMachineId", U(0)),
        ],
    );
    obj(&mut b, "Node", &[("name", S("keyed")), ("parentId", U(0))]);
    for p in 0..properties {
        let mut path = vec![];
        for id in [2, (p / width + 1) as u64, (p % width) as u64] {
            var(&mut path, id);
        }
        obj(
            &mut b,
            "DataBindContext",
            &[
                ("propertyKey", U(13)),
                ("sourcePathIds", Bytes(&path)),
                ("flags", U(1)),
            ],
        );
    }
    obj(
        &mut b,
        "ArtboardComponentList",
        &[("name", S("screens")), ("parentId", U(0))],
    );
    bind(&mut b, 800, &[2, 0]);
    // Declare a live page before its list mounts on the first player advance.
    obj(
        &mut b,
        "LiveStep",
        &[
            ("parentId", U(0)),
            ("name", S("stay")),
            ("fromArtboard", S("A")),
            ("toArtboard", S("A")),
        ],
    );
    timeline(&mut b, "walk", 1, 13, 0.0, 100.0);
    machine(&mut b, true);
    obj(
        &mut b,
        "Artboard",
        &[
            ("name", S("A")),
            ("width", F(400.0)),
            ("height", F(100.0)),
            ("viewModelId", U(1)),
            ("defaultStateMachineId", U(0)),
        ],
    );
    obj(
        &mut b,
        "NestedArtboard",
        &[("parentId", U(0)), ("artboardId", U(2))],
    );
    obj(&mut b, "LiveSlot", &[("parentId", U(1)), ("unitKey", U(1))]);
    machine(&mut b, false);
    obj(
        &mut b,
        "Artboard",
        &[("name", S("unit")), ("width", F(20.0)), ("height", F(20.0))],
    );
    b
}

/// A large ordinary file with cheap realized rows and no live-move records.
pub fn ordinary_list_fixture(rows: usize) -> Vec<u8> {
    use V::*;
    let mut b = b"RIVE".to_vec();
    for n in [7, 0, 0, 0] {
        var(&mut b, n);
    }
    obj(&mut b, "Backboard", &[]);
    obj(&mut b, "ViewModel", &[("name", S("Row"))]);
    obj(&mut b, "ViewModelInstance", &[("viewModelId", U(0))]);
    obj(&mut b, "ViewModel", &[("name", S("Page"))]);
    obj(&mut b, "ViewModelPropertyList", &[("name", S("screens"))]);
    obj(&mut b, "ViewModelInstance", &[("viewModelId", U(1))]);
    obj(
        &mut b,
        "ViewModelInstanceList",
        &[("viewModelPropertyId", U(0))],
    );
    for _ in 0..rows {
        obj(
            &mut b,
            "ViewModelInstanceListItem",
            &[("viewModelId", U(0)), ("viewModelInstanceId", U(0))],
        );
    }
    obj(
        &mut b,
        "Artboard",
        &[
            ("name", S("Page")),
            ("width", F(100.0)),
            ("height", F(100.0)),
            ("viewModelId", U(1)),
            ("defaultStateMachineId", U(0)),
        ],
    );
    obj(
        &mut b,
        "ArtboardComponentList",
        &[("name", S("screens")), ("parentId", U(0))],
    );
    bind(&mut b, 800, &[1, 0]);
    machine(&mut b, false);
    obj(
        &mut b,
        "Artboard",
        &[
            ("name", S("Row")),
            ("width", F(10.0)),
            ("height", F(10.0)),
            ("viewModelId", U(0)),
            ("defaultStateMachineId", U(0)),
        ],
    );
    obj(&mut b, "Node", &[("parentId", U(0))]);
    machine(&mut b, false);
    b
}

/// A populated ordinary list inside a row inserted into an initially empty
/// outer list. No page records, map rules, fonts, scripts, or nested hosts.
pub fn focus_list_fixture() -> Vec<u8> {
    use V::*;
    let mut b = b"RIVE".to_vec();
    for n in [7, 0, 0, 0] {
        var(&mut b, n);
    }
    obj(&mut b, "Backboard", &[]);
    obj(&mut b, "ViewModel", &[("name", S("Button"))]);
    obj(&mut b, "ViewModelInstance", &[("viewModelId", U(0))]);
    for (id, name, property) in [(1, "Row", "children"), (2, "Page", "screens")] {
        obj(&mut b, "ViewModel", &[("name", S(name))]);
        obj(&mut b, "ViewModelPropertyList", &[("name", S(property))]);
        obj(&mut b, "ViewModelInstance", &[("viewModelId", U(id))]);
        obj(
            &mut b,
            "ViewModelInstanceList",
            &[("viewModelPropertyId", U(0))],
        );
        if id == 1 {
            obj(
                &mut b,
                "ViewModelInstanceListItem",
                &[("viewModelId", U(0)), ("viewModelInstanceId", U(0))],
            );
        }
    }
    for (id, name, property) in [(2, "Page", "screens"), (1, "Row", "children")] {
        obj(
            &mut b,
            "Artboard",
            &[
                ("name", S(name)),
                ("width", F(100.0)),
                ("height", F(100.0)),
                ("viewModelId", U(id)),
                ("defaultStateMachineId", U(0)),
            ],
        );
        obj(
            &mut b,
            "ArtboardComponentList",
            &[("name", S(property)), ("parentId", U(0))],
        );
        bind(&mut b, 800, &[id as u8, 0]);
        machine(&mut b, false);
    }
    obj(
        &mut b,
        "Artboard",
        &[
            ("name", S("Button")),
            ("width", F(10.0)),
            ("height", F(10.0)),
            ("viewModelId", U(0)),
            ("defaultStateMachineId", U(0)),
        ],
    );
    obj(
        &mut b,
        "LayoutComponent",
        &[
            ("parentId", U(0)),
            ("name", S("button")),
            ("width", F(10.0)),
            ("height", F(10.0)),
            ("styleId", U(2)),
        ],
    );
    obj(&mut b, "LayoutComponentStyle", &[("parentId", U(1))]);
    obj(&mut b, "FocusData", &[("parentId", U(1))]);
    obj(
        &mut b,
        "SemanticData",
        &[("parentId", U(1)), ("role", U(1)), ("label", S("button"))],
    );
    machine(&mut b, false);
    b
}

/// Two full-height Screen rows with different intrinsic content and bound text.
pub fn fit_content_page_fixture() -> Vec<u8> {
    use V::*;
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
    for (id, name) in [(0, "A"), (1, "B")] {
        obj(&mut b, "ViewModel", &[("name", S(name))]);
        obj(&mut b, "ViewModelPropertyString", &[("name", S("label"))]);
        obj(&mut b, "ViewModelInstance", &[("viewModelId", U(id))]);
        obj(
            &mut b,
            "ViewModelInstanceString",
            &[
                ("viewModelPropertyId", U(0)),
                ("propertyValue", S("Short label")),
            ],
        );
    }
    obj(&mut b, "ViewModel", &[("name", S("Page"))]);
    obj(&mut b, "ViewModelPropertyList", &[("name", S("screens"))]);
    obj(&mut b, "ViewModelInstance", &[("viewModelId", U(2))]);
    obj(
        &mut b,
        "ViewModelInstanceList",
        &[("viewModelPropertyId", U(0))],
    );
    for id in [0, 1] {
        obj(
            &mut b,
            "ViewModelInstanceListItem",
            &[("viewModelId", U(id)), ("viewModelInstanceId", U(0))],
        );
    }
    obj(
        &mut b,
        "Artboard",
        &[
            ("name", S("$nuxie.page")),
            ("width", F(320.0)),
            ("height", F(600.0)),
            ("viewModelId", U(2)),
            ("defaultStateMachineId", U(0)),
        ],
    );
    obj(
        &mut b,
        "ArtboardComponentList",
        &[("name", S("screens")), ("parentId", U(0))],
    );
    bind(&mut b, 800, &[2, 0]);
    machine(&mut b, false);
    for (id, name, height) in [(0, "A", 80.0), (1, "B", 160.0)] {
        obj(
            &mut b,
            "Artboard",
            &[
                ("name", S(name)),
                ("width", F(320.0)),
                ("height", F(600.0)),
                ("viewModelId", U(id)),
                ("styleId", U(1)),
                ("defaultStateMachineId", U(0)),
            ],
        );
        obj(
            &mut b,
            "LayoutComponentStyle",
            &[("parentId", U(0)), ("flexDirectionValue", U(0))],
        ); // 1
        obj(
            &mut b,
            "LayoutComponent",
            &[
                ("name", S("spacer")),
                ("parentId", U(0)),
                ("width", F(320.0)),
                ("height", F(height)),
                ("styleId", U(3)),
            ],
        ); // 2
        obj(&mut b, "LayoutComponentStyle", &[("parentId", U(2))]); // 3
        obj(
            &mut b,
            "LayoutComponent",
            &[
                ("name", S("labelBox")),
                ("parentId", U(0)),
                ("width", F(320.0)),
                ("styleId", U(5)),
            ],
        ); // 4
        obj(
            &mut b,
            "LayoutComponentStyle",
            &[
                ("parentId", U(4)),
                ("layoutHeightScaleType", U(2)),
                ("intrinsicallySizedValue", B(true)),
                ("heightUnitsValue", U(3)),
            ],
        ); // 5
        obj(
            &mut b,
            "Text",
            &[
                ("parentId", U(4)),
                ("sizingValue", U(1)),
                ("width", F(320.0)),
                ("wrapValue", U(0)),
            ],
        ); // 6
        obj(
            &mut b,
            "TextStylePaint",
            &[
                ("parentId", U(6)),
                ("fontAssetId", U(0)),
                ("fontSize", F(20.0)),
                ("lineHeight", F(24.0)),
            ],
        ); // 7
        obj(&mut b, "Fill", &[("parentId", U(7))]); // 8
        obj(
            &mut b,
            "SolidColor",
            &[("parentId", U(8)), ("colorValue", Color(0xff000000))],
        ); // 9
        obj(
            &mut b,
            "TextValueRun",
            &[
                ("name", S("label")),
                ("parentId", U(6)),
                ("styleId", U(7)),
                ("text", S("Short label")),
            ],
        ); // 10
        bind(&mut b, key("TextValueRun", "text").into(), &[id as u8, 0]);
        machine(&mut b, false);
    }
    b
}

/// One Screen with repeated items sharing a VM and hosting event-firing units.
pub fn event_context_fixture(page: bool) -> Vec<u8> {
    use V::*;
    let mut b = b"RIVE".to_vec();
    for n in [7, 0, 0, 0] {
        var(&mut b, n);
    }
    obj(&mut b, "Backboard", &[]);
    obj(&mut b, "ViewModel", &[("name", S("Row"))]);
    obj(&mut b, "ViewModelPropertyNumber", &[("name", S("value"))]);
    obj(&mut b, "ViewModelInstance", &[("viewModelId", U(0))]);
    obj(
        &mut b,
        "ViewModelInstanceNumber",
        &[("viewModelPropertyId", U(0)), ("propertyValue", F(0.0))],
    );
    for (model, name, list, count) in [(1, "Screen", "items", 2), (2, "Page", "screens", 1)] {
        obj(&mut b, "ViewModel", &[("name", S(name))]);
        obj(&mut b, "ViewModelPropertyList", &[("name", S(list))]);
        obj(&mut b, "ViewModelInstance", &[("viewModelId", U(model))]);
        obj(
            &mut b,
            "ViewModelInstanceList",
            &[("viewModelPropertyId", U(0))],
        );
        for _ in 0..count {
            obj(
                &mut b,
                "ViewModelInstanceListItem",
                &[("viewModelId", U(model - 1)), ("viewModelInstanceId", U(0))],
            );
        }
    }
    for (model, name, list) in [
        (2, if page { "$nuxie.page" } else { "ordinary" }, "screens"),
        (1, "Screen", "items"),
    ] {
        obj(
            &mut b,
            "Artboard",
            &[
                ("name", S(name)),
                ("width", F(200.0)),
                ("height", F(200.0)),
                ("viewModelId", U(model)),
                ("defaultStateMachineId", U(0)),
            ],
        );
        obj(
            &mut b,
            "ArtboardComponentList",
            &[("name", S(list)), ("parentId", U(0))],
        );
        bind(&mut b, 800, &[model as u8, 0]);
        machine(&mut b, false);
    }
    obj(
        &mut b,
        "Artboard",
        &[
            ("name", S("Row")),
            ("width", F(100.0)),
            ("height", F(100.0)),
            ("viewModelId", U(0)),
            ("defaultStateMachineId", U(0)),
        ],
    );
    obj(
        &mut b,
        "NestedArtboard",
        &[
            ("name", S("unit")),
            ("parentId", U(0)),
            ("artboardId", U(3)),
            ("isStateful", B(true)),
        ],
    );
    obj(
        &mut b,
        "NestedStateMachine",
        &[("parentId", U(1)), ("animationId", U(0))],
    );
    machine(&mut b, false);
    obj(
        &mut b,
        "Artboard",
        &[
            ("name", S("Unit")),
            ("width", F(100.0)),
            ("height", F(100.0)),
            ("viewModelId", U(0)),
            ("defaultStateMachineId", U(0)),
        ],
    );
    obj(
        &mut b,
        "Event",
        &[("parentId", U(0)), ("name", S("item-event"))],
    );
    machine(&mut b, false);
    b
}
