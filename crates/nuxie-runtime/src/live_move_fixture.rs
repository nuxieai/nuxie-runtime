//! A literal published live-move page, also imported through the C API tests.
#![allow(dead_code)]
use nuxie_schema::definition_by_name;
#[derive(Clone, Copy)]
pub(crate) enum V<'a> {
    U(u64),
    F(f32),
    S(&'a str),
    B(bool),
    Bytes(&'a [u8]),
    Color(u32),
    /// A signed integer field, zigzag encoded.
    I(i64),
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
pub(crate) fn obj(b: &mut Vec<u8>, kind: &str, props: &[(&str, V<'_>)]) {
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
            V::I(n) => var(b, ((n << 1) ^ (n >> 63)) as u64),
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
pub(crate) fn bind(b: &mut Vec<u8>, property: u64, path: &[u8]) {
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
    reference_prop: bool,
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
    if reference_prop {
        // A model reference can't push, so this two-way binding is polled.
        obj(
            b,
            "ViewModelInstanceViewModel",
            &[
                ("parentId", V::U(host_id + 3)),
                ("viewModelPropertyId", V::U(5)),
                ("propertyValue", V::U(0)),
            ],
        );
        obj(
            b,
            "DataBindContext",
            &[
                ("propertyKey", V::U(577)),
                ("sourcePathIds", V::Bytes(&[screen_model, 4])),
                ("flags", V::U(2)),
            ],
        );
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
    corner_keys: bool,
    shadow_keys: bool,
    feathered: bool,
    overshoot: bool,
    spare_writes: bool,
    entry_event: bool,
    reference_prop: bool,
}
/// `fixture(2)` whose unit holds a nested "button" component with its own
/// stateful Button model. The button's machine keys its knob's x to 30, and a
/// two-way binding writes x back into the model's `value`, which starts at 0.
/// So every copy of the unit writes its own model on its first advance,
/// including a flight slot's spare copy. The model also has `isset:value`,
/// the marker the SDKs pair with `value`. With `empty_screens`, the page's
/// screens list starts empty.
pub fn spare_writes_fixture(empty_screens: bool) -> Vec<u8> {
    build(
        2,
        Options {
            spare_writes: true,
            empty_screens,
            ..Options::default()
        },
    )
}
/// `fixture(2)` whose Screen B fires an "entered" event on entering its first
/// state. A listener on that event sets `entered` in B's model from 0 to 1.
pub fn entry_event_fixture() -> Vec<u8> {
    build(
        2,
        Options {
            entry_event: true,
            ..Options::default()
        },
    )
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
/// `values_fixture` plus a model-reference prop. The unit and each Screen
/// have `ref`, a view-model property pointing at a `Ref` instance, and each
/// Screen's slot binds it two-way. `Ref` has a second instance, `second`.
pub fn values_reference_fixture() -> Vec<u8> {
    build(
        2,
        Options {
            value_props: true,
            reference_prop: true,
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
/// `keyed_fixture` with a key that runs past 1: the flight's progress is
/// keyed 0 at frame 0, 1.25 at frame 30 and 1 at frame 60.
pub fn keyed_overshoot_fixture() -> Vec<u8> {
    build(
        2,
        Options {
            controls: true,
            keyed: true,
            overshoot: true,
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
/// As `two_version_fixture`, with B's timeline also keying the four corner
/// controls of step t1's two flight slots, 4 to 8. Its units are translated
/// artboards.
pub fn two_version_corner_keys_fixture() -> Vec<u8> {
    build(
        2,
        Options {
            two_versions: true,
            corner_keys: true,
            ..Options::default()
        },
    )
}
/// As `two_version_fixture`, each unit's root also drawing a Fill with a
/// Feather of strength 12, Rive's own soft shadow. When `keyed`, B's timeline
/// also keys the seven shadow controls of step t1's two flight slots, as
/// `SHADOW_KEYS`. Its units are translated artboards.
pub fn two_version_shadow_keys_fixture(keyed: bool) -> Vec<u8> {
    build(
        2,
        Options {
            two_versions: true,
            feathered: true,
            shadow_keys: keyed,
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
    // These options use fixed object ids on the default shapes.
    if options.spare_writes {
        assert!(
            !options.feathered
                && !semantics
                && !options.nested_list
                && !options.paint_list
                && !options.two_versions
                && !options.scripted
                && script.is_none()
        );
    }
    if options.entry_event {
        assert!(
            rows >= 2
                && !options.value_props
                && !options.two_versions
                && !options.ordinary
                && !options.overlapping_units
                && !semantics
        );
    }
    // Screen models' `entered` number, after `offset`.
    let entered = 1;
    let button_view_model = rows as u64 + 2;
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
    // The model reference follows the value props: unit property 5, Screen
    // property 4. `Ref` is the view model after the page's.
    assert!(!options.reference_prop || options.value_props);
    let reference = (
        "ViewModelPropertyViewModel",
        [("name", S("ref")), ("viewModelReferenceId", U(rows as u64 + 2))],
    );
    if options.reference_prop {
        obj(&mut b, reference.0, &reference.1);
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
    if options.reference_prop {
        obj(
            &mut b,
            "ViewModelInstanceViewModel",
            &[("viewModelPropertyId", U(5)), ("propertyValue", U(0))],
        );
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
        if options.entry_event {
            obj(&mut b, "ViewModelPropertyNumber", &[("name", S("entered"))]);
        }
        if options.reference_prop {
            obj(&mut b, reference.0, &reference.1);
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
        if options.entry_event {
            obj(
                &mut b,
                "ViewModelInstanceNumber",
                &[
                    ("viewModelPropertyId", U(entered)),
                    ("propertyValue", F(0.0)),
                ],
            );
        }
        if options.reference_prop {
            obj(
                &mut b,
                "ViewModelInstanceViewModel",
                &[("viewModelPropertyId", U(4)), ("propertyValue", U(0))],
            );
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
    if options.spare_writes {
        obj(&mut b, "ViewModel", &[("name", S("Button"))]);
        obj(&mut b, "ViewModelPropertyNumber", &[("name", S("value"))]);
        obj(
            &mut b,
            "ViewModelPropertyBoolean",
            &[("name", S("isset:value"))],
        );
        obj(
            &mut b,
            "ViewModelInstance",
            &[("viewModelId", U(button_view_model))],
        );
        obj(
            &mut b,
            "ViewModelInstanceNumber",
            &[("viewModelPropertyId", U(0)), ("propertyValue", F(0.0))],
        );
        obj(
            &mut b,
            "ViewModelInstanceBoolean",
            &[("viewModelPropertyId", U(1)), ("propertyValue", B(false))],
        );
    }
    if options.reference_prop {
        obj(&mut b, "ViewModel", &[("name", S("Ref"))]);
        obj(&mut b, "ViewModelPropertyNumber", &[("name", S("n"))]);
        for (name, n) in [("first", 1.0), ("second", 2.0)] {
            obj(
                &mut b,
                "ViewModelInstance",
                &[("name", S(name)), ("viewModelId", U(rows as u64 + 2))],
            );
            obj(
                &mut b,
                "ViewModelInstanceNumber",
                &[("viewModelPropertyId", U(0)), ("propertyValue", F(n))],
            );
        }
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
            options.reference_prop,
        );
        let slot_width =
            (if options.value_props { 9 } else { 6 }) + u64::from(options.reference_prop);
        let layer_id = host_id + slot_width;
        obj(
            &mut b,
            "Node",
            &[("name", S("flight")), ("parentId", U(0)), ("x", F(35.0))],
        );
        let flight_id = layer_id + 1;
        // Step t1's flight records on this Screen, for its corner keys.
        let mut corner_records = Vec::new();
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
                    options.reference_prop,
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
                if j < 2 {
                    corner_records.push(id + slot_width);
                }
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
                options.reference_prop,
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
                options.reference_prop,
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
        // Screen B's "entered" event, the next object after its slot records.
        let event_id = flight_id + slot_width + 2;
        if options.entry_event && i == 1 {
            obj(
                &mut b,
                "Event",
                &[("name", S("entered")), ("parentId", U(0))],
            );
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
                if options.overshoot {
                    obj(
                        &mut b,
                        "KeyFrameDouble",
                        &[
                            ("frame", U(30)),
                            ("value", F(1.25)),
                            ("interpolationType", U(1)),
                        ],
                    );
                }
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
            if options.corner_keys {
                for record in &corner_records {
                    obj(&mut b, "KeyedObject", &[("objectId", U(*record))]);
                    for corner in CORNERS {
                        obj(
                            &mut b,
                            "KeyedProperty",
                            &[("propertyKey", U(key("LiveSlot", corner).into()))],
                        );
                        for (frame, value) in [(0, 4.0), (60, 8.0)] {
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
            }
            if options.shadow_keys {
                for record in &corner_records {
                    obj(&mut b, "KeyedObject", &[("objectId", U(*record))]);
                    for ((_, property), (from, to)) in SHADOW_CONTROLS.into_iter().zip(SHADOW_KEYS)
                    {
                        linear_track(&mut b, property, from, to);
                    }
                }
            }
        }
        machine(&mut b, i == 1);
        if options.entry_event && i == 1 {
            // Fired as the machine enters "entrance", the state written just
            // before it. The listener's view-model change sets `entered` to 1.
            obj(
                &mut b,
                "StateMachineFireEvent",
                &[("eventId", U(event_id)), ("occursValue", U(0))],
            );
            obj(
                &mut b,
                "StateMachineListenerSingle",
                &[
                    ("targetId", U(0)),
                    ("listenerTypeValue", U(5)),
                    ("eventId", U(event_id)),
                ],
            );
            obj(
                &mut b,
                "BindablePropertyNumber",
                &[("propertyValue", F(1.0))],
            );
            obj(
                &mut b,
                "DataBindContext",
                &[
                    (
                        "propertyKey",
                        U(key("BindablePropertyNumber", "propertyValue").into()),
                    ),
                    ("sourcePathIds", Bytes(&[i as u8 + 1, entered as u8])),
                    ("flags", U(1)),
                ],
            );
            obj(&mut b, "ListenerViewModelChange", &[]);
        }
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
        if options.feathered {
            obj(&mut b, "Fill", &[("parentId", U(0))]); // 12
            obj(
                &mut b,
                "SolidColor",
                &[("parentId", U(12)), ("colorValue", Color(0x8010_2030))],
            );
            obj(
                &mut b,
                "Feather",
                &[
                    ("parentId", U(12)),
                    ("spaceValue", U(1)),
                    ("strength", F(12.0)),
                ],
            );
        }
        if options.spare_writes {
            // A nested component with its own stateful Button model, which
            // lives and dies with this copy of the unit.
            obj(
                &mut b,
                "NestedArtboard",
                &[
                    ("name", S("button")),
                    ("parentId", U(0)),
                    ("artboardId", U(rows as u64 + 2)),
                    ("isStateful", B(true)),
                ],
            ); // 12
            obj(
                &mut b,
                "NestedStateMachine",
                &[("parentId", U(12)), ("animationId", U(0))],
            ); // 13
            obj(
                &mut b,
                "ViewModelInstance",
                &[("viewModelId", U(button_view_model)), ("parentId", U(12))],
            ); // 14
            obj(
                &mut b,
                "ViewModelInstanceNumber",
                &[
                    ("parentId", U(14)),
                    ("viewModelPropertyId", U(0)),
                    ("propertyValue", F(0.0)),
                ],
            ); // 15
            obj(
                &mut b,
                "ViewModelInstanceBoolean",
                &[
                    ("parentId", U(14)),
                    ("viewModelPropertyId", U(1)),
                    ("propertyValue", B(false)),
                ],
            ); // 16
        }

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
    if options.spare_writes {
        obj(
            &mut b,
            "Artboard",
            &[
                ("name", S("button")),
                ("width", F(40.0)),
                ("height", F(40.0)),
                ("viewModelId", U(button_view_model)),
                ("clip", B(false)),
                ("defaultStateMachineId", U(0)),
            ],
        );
        obj(
            &mut b,
            "Node",
            &[("name", S("knob")), ("parentId", U(0)), ("x", F(0.0))],
        ); // 1
        // Two-way: the machine's key on x writes the model's `value`.
        obj(
            &mut b,
            "DataBindContext",
            &[
                ("propertyKey", U(13)),
                ("sourcePathIds", Bytes(&[button_view_model as u8, 0])),
                ("flags", U(2)),
            ],
        );
        // The first key, 30, differs from the model's 0.
        timeline(&mut b, "press", 1, 13, 30.0, 30.0);
        machine(&mut b, true);
    }
    b
}

/// A flex sibling after a NestedArtboardLayout, exercising its real layout node.
pub fn layout_fixture() -> Vec<u8> {
    build_layout_fixture(false, None, false, false, None, false)
}
pub fn layout_flight_fixture() -> Vec<u8> {
    build_layout_fixture(true, None, false, false, None, false)
}
pub fn hosted_css_layout_fixture(destination_css: bool) -> Vec<u8> {
    build_layout_fixture(true, Some(destination_css), false, false, None, false)
}
pub fn hosted_css_plain_flight_fixture() -> Vec<u8> {
    build_layout_fixture(true, Some(true), true, false, None, false)
}
pub fn native_layout_flight_reflow_fixture() -> Vec<u8> {
    build_layout_fixture(true, None, false, true, None, false)
}

/// A content-sized flight row followed by a fixed-size, animated hit target.
pub fn shared_layout_flight_fixture() -> Vec<u8> {
    build_shared_layout_flight_fixture(false, false)
}
/// The landing slot follows the content-sized flight row in the shared tree.
pub fn shared_layout_endpoint_fixture() -> Vec<u8> {
    build_shared_layout_flight_fixture(true, false)
}
pub fn shared_layout_two_flight_endpoint_fixture() -> Vec<u8> {
    build_shared_layout_flight_fixture(true, true)
}
/// An absolute, hugging layout box at `index`, with its style after it.
fn absolute_box(b: &mut Vec<u8>, name: &str, index: u64) {
    use V::*;
    obj(
        b,
        "LayoutComponent",
        &[
            ("name", S(name)),
            ("parentId", U(0)),
            ("styleId", U(index + 1)),
        ],
    );
    obj(
        b,
        "LayoutComponentStyle",
        &[
            ("parentId", U(index)),
            ("positionTypeValue", U(2)),
            ("layoutWidthScaleType", U(2)),
            ("layoutHeightScaleType", U(2)),
        ],
    );
}
/// A stateful 60 x 40 layout host at `index`, in the box before it or in the
/// Screen, with its state machine after it.
fn flight_layout_host(b: &mut Vec<u8>, name: &str, index: u64, boxed: bool) {
    use V::*;
    obj(
        b,
        "NestedArtboardLayout",
        &[
            ("name", S(name)),
            ("parentId", U(if boxed { index - 2 } else { 0 })),
            ("artboardId", U(1)),
            ("instanceWidth", F(60.0)),
            ("instanceHeight", F(40.0)),
            ("isStateful", B(true)),
        ],
    );
    obj(
        b,
        "NestedStateMachine",
        &[("parentId", U(index)), ("animationId", U(0))],
    );
}
fn build_shared_layout_flight_fixture(moving_endpoint: bool, two_flights: bool) -> Vec<u8> {
    use V::*;
    let mut b = b"RIVE".to_vec();
    for n in [7, 0, 0, 0] {
        var(&mut b, n);
    }
    obj(&mut b, "Backboard", &[]);
    for (model, name) in ["FlightScreen", "Sibling", "SharedPage"]
        .into_iter()
        .enumerate()
    {
        obj(&mut b, "ViewModel", &[("name", S(name))]);
        if name == "SharedPage" {
            obj(&mut b, "ViewModelPropertyList", &[("name", S("screens"))]);
        }
        obj(
            &mut b,
            "ViewModelInstance",
            &[("viewModelId", U(model as u64))],
        );
        if name == "SharedPage" {
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
        }
    }
    obj(
        &mut b,
        "Artboard",
        &[
            ("name", S("FlightScreen")),
            ("viewModelId", U(0)),
            ("width", F(400.0)),
            ("height", F(80.0)),
            ("styleId", U(1)),
            ("clip", B(false)),
        ],
    );
    let mut style = vec![("parentId", U(0)), ("layoutHeightScaleType", U(2))];
    if two_flights || !moving_endpoint {
        style.push(("flexDirectionValue", U(0)));
    }
    obj(&mut b, "LayoutComponentStyle", &style); // 1: Hug the Screen's rows
    obj(&mut b, "Node", &[("parentId", U(0))]); // 2: plain endpoints
    for (name, host) in [("source", 3), ("destination", 5)] {
        obj(
            &mut b,
            "NestedArtboard",
            &[
                ("name", S(name)),
                ("parentId", U(2)),
                ("artboardId", U(1)),
                ("isStateful", B(true)),
            ],
        );
        obj(
            &mut b,
            "NestedStateMachine",
            &[("parentId", U(host)), ("animationId", U(0))],
        );
    }
    // A native layout flight slot must be out of flow. A plain Node joins its
    // contents to the layout, so it sits in an absolute layout box, upstream's
    // route. The runtime takes a CSS flight slot out of flow itself, so the
    // in-flow fixture keeps its slot in the Screen's column.
    let flight = if moving_endpoint {
        absolute_box(&mut b, "flightBox", 7); // 7, 8
        9
    } else {
        7
    };
    flight_layout_host(&mut b, "flight", flight, moving_endpoint); // +0, +1
    for (host, unit, role) in [(3, 1, 0), (5, 2, 0), (flight, 1, 1)] {
        obj(
            &mut b,
            "LiveSlot",
            &[
                ("parentId", U(host)),
                ("unitKey", U(unit)),
                ("role", U(role)),
                ("flightKey", U(role)),
                ("sizeMode", U(1)),
            ],
        );
    }
    if !moving_endpoint {
        obj(
            &mut b,
            "LayoutCssStyle",
            &[
                ("parentId", U(7)),
                ("boxSizing", U(1)),
                ("widthKind", U(1)),
                ("width", F(60.0)),
                ("heightKind", U(1)),
                ("height", F(40.0)),
            ],
        ); // 12
        obj(
            &mut b,
            "LayoutComponent",
            &[
                ("name", S("spacer")),
                ("parentId", U(0)),
                ("width", F(20.0)),
                ("height", F(30.0)),
                ("styleId", U(14)),
            ],
        ); // 13: the next row in the Screen
        obj(&mut b, "LayoutComponentStyle", &[("parentId", U(13))]); // 14
    }
    if two_flights {
        obj(
            &mut b,
            "NestedArtboard",
            &[
                ("name", S("source2")),
                ("parentId", U(2)),
                ("artboardId", U(1)),
                ("isStateful", B(true)),
            ],
        ); // 14
        obj(
            &mut b,
            "NestedStateMachine",
            &[("parentId", U(14)), ("animationId", U(0))],
        ); // 15
        absolute_box(&mut b, "flight2Box", 16); // 16, 17
        flight_layout_host(&mut b, "flight2", 18, true); // 18, 19
        obj(
            &mut b,
            "LiveSlot",
            &[("parentId", U(14)), ("unitKey", U(3))],
        ); // 20
        obj(
            &mut b,
            "LiveSlot",
            &[
                ("parentId", U(18)),
                ("unitKey", U(3)),
                ("role", U(1)),
                ("flightKey", U(2)),
                ("sizeMode", U(1)),
            ],
        ); // 21
    }
    obj(
        &mut b,
        "Artboard",
        &[
            ("name", S("unit")),
            ("width", F(100.0)),
            ("height", F(80.0)),
            ("styleId", U(2)),
            ("defaultStateMachineId", U(0)),
            ("clip", B(false)),
        ],
    );
    obj(&mut b, "Node", &[("name", S("clock")), ("parentId", U(0))]); // 1
    obj(&mut b, "LayoutComponentStyle", &[("parentId", U(0))]); // 2
    if !moving_endpoint {
        // A CSS unit, so the flight host's item record sizes and places it.
        obj(&mut b, "LayoutCssStyle", &[("parentId", U(0))]); // 3
    }
    if moving_endpoint {
        obj(
            &mut b,
            "Shape",
            &[("name", S("marker")), ("parentId", U(0))],
        ); // 3
        obj(
            &mut b,
            "Rectangle",
            &[
                ("parentId", U(3)),
                ("width", F(10.0)),
                ("height", F(10.0)),
                ("x", F(5.0)),
                ("y", F(5.0)),
            ],
        ); // 4
        obj(&mut b, "Fill", &[("parentId", U(3))]); // 5
        obj(
            &mut b,
            "SolidColor",
            &[
                ("name", S("markerColor")),
                ("parentId", U(5)),
                ("colorValue", Color(0xffedcba9)),
            ],
        ); // 6
        obj(
            &mut b,
            "LayoutComponent",
            &[("name", S("reflow")), ("parentId", U(0)), ("styleId", U(8))],
        ); // 7
        obj(
            &mut b,
            "LayoutComponentStyle",
            &[
                ("parentId", U(7)),
                ("layoutWidthScaleType", U(1)),
                ("layoutHeightScaleType", U(1)),
            ],
        ); // 8
    }
    timeline(&mut b, "clock", 1, 13, 0.0, 100.0);
    machine(&mut b, true);

    obj(
        &mut b,
        "Artboard",
        &[
            ("name", S("Sibling")),
            ("viewModelId", U(1)),
            ("width", F(400.0)),
            ("height", F(40.0)),
            ("styleId", U(2)),
            ("defaultStateMachineId", U(0)),
            ("clip", B(false)),
        ],
    );
    obj(&mut b, "Node", &[("name", S("clock")), ("parentId", U(0))]); // 1
    obj(&mut b, "LayoutComponentStyle", &[("parentId", U(0))]); // 2
    obj(
        &mut b,
        "Shape",
        &[("name", S("target")), ("parentId", U(0))],
    ); // 3
    obj(
        &mut b,
        "Rectangle",
        &[
            ("parentId", U(3)),
            ("width", F(20.0)),
            ("height", F(20.0)),
            ("x", F(10.0)),
            ("y", F(10.0)),
        ],
    ); // 4
    obj(&mut b, "Fill", &[("parentId", U(3))]); // 5
    obj(
        &mut b,
        "SolidColor",
        &[("parentId", U(5)), ("colorValue", Color(0xff123456))],
    ); // 6
    obj(
        &mut b,
        "LayoutCssStyle",
        &[
            ("parentId", U(0)),
            ("widthKind", U(1)),
            ("width", F(400.0)),
            ("heightKind", U(1)),
            ("height", F(40.0)),
            ("flexShrink", F(0.0)),
        ],
    );
    if moving_endpoint {
        obj(
            &mut b,
            "NestedArtboard",
            &[
                ("name", S("landing")),
                ("parentId", U(0)),
                ("artboardId", U(1)),
                ("isStateful", B(true)),
            ],
        ); // 8
        obj(
            &mut b,
            "NestedStateMachine",
            &[("parentId", U(8)), ("animationId", U(0))],
        ); // 9
        obj(&mut b, "LiveSlot", &[("parentId", U(8)), ("unitKey", U(1))]); // 10
    }
    if two_flights {
        obj(
            &mut b,
            "NestedArtboard",
            &[
                ("name", S("landing2")),
                ("parentId", U(0)),
                ("artboardId", U(1)),
                ("isStateful", B(true)),
            ],
        ); // 11
        obj(
            &mut b,
            "NestedStateMachine",
            &[("parentId", U(11)), ("animationId", U(0))],
        ); // 12
        obj(
            &mut b,
            "LiveSlot",
            &[("parentId", U(11)), ("unitKey", U(3))],
        ); // 13
    }
    timeline(&mut b, "clock", 1, 13, 0.0, 100.0);
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

    obj(
        &mut b,
        "Artboard",
        &[
            ("name", S("SharedPage")),
            ("viewModelId", U(2)),
            ("width", F(400.0)),
            ("height", F(600.0)),
            ("styleId", U(2)),
            ("clip", B(false)),
            ("defaultStateMachineId", U(0)),
        ],
    );
    obj(
        &mut b,
        "ArtboardComponentList",
        &[("name", S("screens")), ("parentId", U(0))],
    ); // 1
    bind(&mut b, 800, &[2, 0]);
    obj(&mut b, "LayoutComponentStyle", &[("parentId", U(0))]); // 2
    obj(
        &mut b,
        "LayoutCssStyle",
        &[
            ("parentId", U(0)),
            ("display", U(1)),
            ("flexDirection", U(2)),
        ],
    );
    obj(
        &mut b,
        "LiveStep",
        &[
            ("parentId", U(0)),
            ("name", S("shared")),
            ("fromArtboard", S("FlightScreen")),
            ("toArtboard", S("Sibling")),
        ],
    );
    if moving_endpoint {
        obj(
            &mut b,
            "LiveStepUnit",
            &[
                ("parentId", U(4)),
                ("outgoingUnitKey", U(1)),
                ("incomingUnitKey", U(1)),
                ("outgoingFlightKey", U(1)),
                ("incomingFlightKey", U(1)),
            ],
        );
    }
    if two_flights {
        obj(
            &mut b,
            "LiveStepUnit",
            &[
                ("parentId", U(4)),
                ("outgoingUnitKey", U(3)),
                ("incomingUnitKey", U(3)),
                ("outgoingFlightKey", U(2)),
                ("incomingFlightKey", U(2)),
            ],
        );
    }
    machine(&mut b, false);
    b
}
/// Sources: 0 plain, 1 CSS item carrier, 2 native layout overrides.
pub fn css_follower_fixture(source: u8) -> Vec<u8> {
    build_layout_fixture(true, Some(true), false, true, Some(source), false)
}
/// As `css_follower_fixture(1)`, with the unit artboard turned 0.35 rad in
/// the file about its own center, and no slot `ComponentOrigin`.
pub fn css_follower_turned_fixture() -> Vec<u8> {
    build_layout_fixture(true, Some(true), false, true, Some(1), true)
}
fn build_layout_fixture(
    flight: bool,
    hosted_css: Option<bool>,
    plain_flight: bool,
    reflow: bool,
    follower_source: Option<u8>,
    own_pivot: bool,
) -> Vec<u8> {
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
            ("width", F(if hosted_css.is_some() { 800.0 } else { 400.0 })),
            (
                "height",
                F(if hosted_css.is_some() { 600.0 } else { 100.0 }),
            ),
            ("styleId", U(1)),
            ("clip", B(false)),
        ],
    ); // 0
    obj(
        &mut b,
        "LayoutComponentStyle",
        &[("parentId", U(0)), ("flexDirectionValue", U(2))],
    ); // 1
    let mut source_props = vec![
        ("name", S("source")),
        ("parentId", U(0)),
        ("artboardId", U(1)),
    ];
    if follower_source != Some(0) {
        source_props.extend([
            (
                "instanceWidth",
                F(if follower_source.is_some() {
                    100.0
                } else {
                    150.0
                }),
            ),
            ("instanceHeight", F(80.0)),
        ]);
    }
    obj(
        &mut b,
        if follower_source == Some(0) {
            "NestedArtboard"
        } else {
            "NestedArtboardLayout"
        },
        &source_props,
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
        if hosted_css.is_some() {
            "NestedArtboardLayout"
        } else {
            "NestedArtboard"
        },
        &[
            ("name", S("destination")),
            ("parentId", U(if hosted_css.is_some() { 0 } else { 5 })),
            ("artboardId", U(1)),
        ],
    ); // 6
    // A native layout flight slot must be out of flow, so it comes last, in
    // an absolute layout box where the row would have placed it.
    let native = flight && hosted_css.is_none() && !plain_flight;
    if flight && !native {
        let mut props = vec![
            ("name", S("flight")),
            ("parentId", U(0)),
            ("artboardId", U(1)),
        ];
        if !plain_flight {
            props.extend([("instanceWidth", F(150.0)), ("instanceHeight", F(80.0))]);
        }
        obj(
            &mut b,
            if plain_flight {
                "NestedArtboard"
            } else {
                "NestedArtboardLayout"
            },
            &props,
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
    if native {
        obj(
            &mut b,
            "LayoutComponent",
            &[
                ("name", S("flightBox")),
                ("parentId", U(0)),
                ("styleId", U(10)),
            ],
        ); // 9
        obj(
            &mut b,
            "LayoutComponentStyle",
            &[
                ("parentId", U(9)),
                ("positionTypeValue", U(2)),
                ("positionLeft", F(170.0)),
                ("positionLeftUnitsValue", U(1)),
                ("layoutWidthScaleType", U(2)),
                ("layoutHeightScaleType", U(2)),
            ],
        ); // 10
        obj(
            &mut b,
            "NestedArtboardLayout",
            &[
                ("name", S("flight")),
                ("parentId", U(9)),
                ("artboardId", U(1)),
                ("instanceWidth", F(150.0)),
                ("instanceHeight", F(80.0)),
            ],
        ); // 11
        obj(
            &mut b,
            "LiveSlot",
            &[
                ("parentId", U(11)),
                ("unitKey", U(1)),
                ("role", U(1)),
                ("flightKey", U(1)),
                ("sizeMode", U(1)),
            ],
        ); // 12
    }
    if let Some(destination_css) = hosted_css {
        obj(
            &mut b,
            "LayoutCssStyle",
            &[
                ("parentId", U(0)),
                ("display", U(1)),
                ("flexDirection", U(2)),
            ],
        );
        for (host, width, height, left) in [
            (2, 260.0, 108.0, 36.0),
            (6, 300.0, 108.0, 56.0),
            (7, 180.0, 90.0, 20.0),
        ] {
            if (host == 6 && !destination_css)
                || (host == 7 && plain_flight)
                || (host == 2 && follower_source.is_some_and(|source| source != 1))
            {
                continue;
            }
            let (width, height) = if follower_source.is_some() {
                match host {
                    2 => (100.0, 80.0),
                    6 => (200.0, 120.0),
                    _ => (width, height),
                }
            } else {
                (width, height)
            };
            let mut props = vec![
                ("parentId", U(host)),
                ("widthKind", U(1)),
                ("width", F(width)),
                ("heightKind", U(1)),
                ("height", F(height)),
                ("marginLeft", F(left)),
            ];
            if follower_source.is_some() {
                props.push(("boxSizing", U(1)));
            }
            obj(&mut b, "LayoutCssStyle", &props);
        }
    }
    if hosted_css.is_some() && !own_pivot {
        // Different native origins exercise borrowed-host dirt on every move.
        for (host, origin) in [(2, 0.5), (6, 0.25), (7, 0.0)] {
            obj(
                &mut b,
                "ComponentOrigin",
                &[
                    ("parentId", U(host)),
                    ("originX", F(origin)),
                    ("originY", F(origin)),
                ],
            );
        }
    }
    let mut unit = vec![
        ("name", S("unit")),
        ("width", F(100.0)),
        ("height", F(80.0)),
        ("styleId", U(2)),
        ("clip", B(false)),
    ];
    if own_pivot {
        unit.extend([
            ("rotation", F(0.35)),
            ("originX", F(0.5)),
            ("originY", F(0.5)),
        ]);
    }
    obj(&mut b, "Artboard", &unit);
    obj(&mut b, "Node", &[("name", S("inside")), ("parentId", U(0))]);
    obj(&mut b, "LayoutComponentStyle", &[("parentId", U(0))]);
    if reflow {
        obj(
            &mut b,
            "LayoutComponent",
            &[("name", S("reflow")), ("parentId", U(0)), ("styleId", U(4))],
        ); // 3
        obj(
            &mut b,
            "LayoutComponentStyle",
            &[
                ("parentId", U(3)),
                ("layoutWidthScaleType", U(1)),
                ("layoutHeightScaleType", U(1)),
            ],
        ); // 4
    }
    if follower_source.is_some() {
        obj(
            &mut b,
            "LayoutCssStyle",
            &[
                ("parentId", U(3)),
                ("widthKind", U(7)),
                ("widthPercent", F(100.0)),
                ("heightKind", U(7)),
                ("heightPercent", F(100.0)),
            ],
        ); // 5
        obj(&mut b, "Fill", &[("parentId", U(3))]); // 6
        obj(
            &mut b,
            "SolidColor",
            &[
                ("name", S("ink")),
                ("parentId", U(6)),
                ("colorValue", Color(0xff123456)),
            ],
        ); // 7
    }
    if hosted_css.is_some() {
        obj(&mut b, "LayoutCssStyle", &[("parentId", U(0))]);
    }
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

/// Two CSS Screens, A and B. Each holds a CSS slot for one unit, 100 by 60
/// at (36, 20), then a later 200 by 100 flow box 30 points up into it. The
/// unit is turned at rest about its center, so wherever a CSS slot places
/// it, it is a CSS stacking context.
pub fn css_two_screen_stacking_fixture() -> Vec<u8> {
    use V::*;
    let mut b = b"RIVE".to_vec();
    for n in [7, 0, 0, 0] {
        var(&mut b, n);
    }
    obj(&mut b, "Backboard", &[]);
    for name in ["A", "B"] {
        obj(&mut b, "ViewModel", &[("name", S(name))]);
    }
    obj(&mut b, "ViewModelInstance", &[("viewModelId", U(0))]);
    obj(&mut b, "ViewModelInstance", &[("viewModelId", U(1))]);
    obj(&mut b, "ViewModel", &[("name", S("StackingPage"))]);
    obj(&mut b, "ViewModelPropertyList", &[("name", S("screens"))]);
    obj(&mut b, "ViewModelInstance", &[("viewModelId", U(2))]);
    obj(
        &mut b,
        "ViewModelInstanceList",
        &[("viewModelPropertyId", U(0))],
    );
    for model in [0, 1] {
        obj(
            &mut b,
            "ViewModelInstanceListItem",
            &[("viewModelId", U(model)), ("viewModelInstanceId", U(0))],
        );
    }
    for (model, name) in [(0, "A"), (1, "B")] {
        obj(
            &mut b,
            "Artboard",
            &[
                ("name", S(name)),
                ("viewModelId", U(model)),
                ("width", F(400.0)),
                ("height", F(300.0)),
                ("styleId", U(2)),
                ("clip", B(false)),
            ],
        ); // 0
        obj(&mut b, "LayoutCssStyle", &[("parentId", U(0))]); // 1
        obj(&mut b, "LayoutComponentStyle", &[("parentId", U(0))]); // 2
        obj(
            &mut b,
            "NestedArtboardLayout",
            &[
                ("name", S("slot")),
                ("parentId", U(0)),
                ("artboardId", U(2)),
            ],
        ); // 3
        obj(
            &mut b,
            "LayoutCssStyle",
            &[
                ("parentId", U(3)),
                ("widthKind", U(1)),
                ("width", F(100.0)),
                ("heightKind", U(1)),
                ("height", F(60.0)),
                ("marginTop", F(20.0)),
                ("marginLeft", F(36.0)),
            ],
        ); // 4
        obj(&mut b, "LiveSlot", &[("parentId", U(3)), ("unitKey", U(1))]); // 5
        obj(
            &mut b,
            "LayoutComponent",
            &[("name", S("later")), ("parentId", U(0)), ("styleId", U(7))],
        ); // 6
        obj(&mut b, "LayoutComponentStyle", &[("parentId", U(6))]); // 7
        obj(
            &mut b,
            "LayoutCssStyle",
            &[
                ("parentId", U(6)),
                ("widthKind", U(1)),
                ("width", F(200.0)),
                ("heightKind", U(1)),
                ("height", F(100.0)),
                ("marginTop", F(-30.0)),
            ],
        ); // 8
        obj(&mut b, "Fill", &[("parentId", U(6))]); // 9
        obj(
            &mut b,
            "SolidColor",
            &[("parentId", U(9)), ("colorValue", Color(0xff3333cc))],
        ); // 10
    }
    obj(
        &mut b,
        "Artboard",
        &[
            ("name", S("unit")),
            ("width", F(100.0)),
            ("height", F(60.0)),
            ("styleId", U(2)),
            ("clip", B(false)),
            ("rotation", F(0.35)),
            ("originX", F(0.5)),
            ("originY", F(0.5)),
        ],
    ); // 0
    obj(&mut b, "LayoutCssStyle", &[("parentId", U(0))]); // 1
    obj(&mut b, "LayoutComponentStyle", &[("parentId", U(0))]); // 2
    obj(&mut b, "Fill", &[("parentId", U(0))]); // 3
    obj(
        &mut b,
        "SolidColor",
        &[("parentId", U(3)), ("colorValue", Color(0xffcc3333))],
    ); // 4
    obj(
        &mut b,
        "Artboard",
        &[
            ("name", S("StackingPage")),
            ("viewModelId", U(2)),
            ("width", F(400.0)),
            ("height", F(600.0)),
            ("clip", B(false)),
            ("defaultStateMachineId", U(0)),
        ],
    );
    obj(
        &mut b,
        "ArtboardComponentList",
        &[("name", S("screens")), ("parentId", U(0))],
    );
    bind(&mut b, 800, &[2, 0]);
    obj(
        &mut b,
        "LiveStep",
        &[
            ("parentId", U(0)),
            ("name", S("go")),
            ("fromArtboard", S("A")),
            ("toArtboard", S("B")),
        ],
    );
    machine(&mut b, false);
    b
}

/// LiveSlot's four corner controls, in CSS order.
const CORNERS: [&str; 4] = [
    "cornerTopLeft",
    "cornerTopRight",
    "cornerBottomRight",
    "cornerBottomLeft",
];

/// Two versions of one element on two CSS Screens (docs/live-moves.md).
/// Unit a, 80 by 60 with 4 point corners, is at home on A; unit b, 160 by
/// 120 with 8 point corners, is at home on B. Step "t1" flies them from A
/// to B through two Layout-mode CSS flight slots on B. Step "t2" flies a
/// third unit, c, square, alone from its home on A to its home on B. When
/// `keyed`, B's timeline keys every flight slot's progress 0 to 1 and its
/// four corners 4 to 8, linear, over one second.
pub fn css_two_version_corner_fixture(keyed: bool) -> Vec<u8> {
    build_css_versions_fixture(if keyed { 4 } else { 0 }, None, UnitShadows::default())
}
/// As `css_two_version_corner_fixture(true)`, with only the top left, top
/// right and bottom right corners keyed: the bottom left stays -1.
pub fn css_three_corner_keys_fixture() -> Vec<u8> {
    build_css_versions_fixture(3, None, UnitShadows::default())
}
/// As `css_two_version_corner_fixture(false)`, with one malformed corner
/// record. 1: A's resting slot for unit a carries a corner of 5. 2: unit
/// a's flight slot carries a NaN corner.
pub fn css_bad_corner_record_fixture(kind: u8) -> Vec<u8> {
    build_css_versions_fixture(0, Some((kind, CORNERS[0])), UnitShadows::default())
}

/// A CSS shadow, authored or drawn: its color, its blur (its Feather's
/// strength) and its offset, in points.
#[derive(Clone, Copy, Debug)]
pub struct Shadow {
    pub color: u32,
    pub blur: f32,
    pub offset: (f32, f32),
}
/// Unit a's shadow in `css_two_version_shadow_fixture`.
pub const UNIT_A_SHADOW: Shadow = Shadow {
    color: 0x6611_2233,
    blur: 10.0,
    offset: (1.0, 2.0),
};
/// Unit b's, which the fixture authors clear: this color with alpha 0.
pub const UNIT_B_SHADOW: Shadow = Shadow {
    color: 0x6644_5566,
    blur: 6.0,
    offset: (0.0, -3.0),
};
/// LiveSlot's seven shadow controls, in key order. Their tracks are written
/// by number: the keys are the published contract, and a runtime without them
/// prunes such a track at import.
pub const SHADOW_CONTROLS: [(&str, u64); 7] = [
    ("shadowAlpha", 62019),
    ("shadowRed", 62020),
    ("shadowGreen", 62021),
    ("shadowBlue", 62022),
    ("shadowBlur", 62023),
    ("shadowOffsetX", 62024),
    ("shadowOffsetY", 62025),
];
/// Each shadow control's keyed value at frames 0 and 60, in that order.
pub const SHADOW_KEYS: [(f32, f32); 7] = [
    (0.56, 0.16),
    (200.2, 8.6),
    (73.4, 0.6),
    (80.3, 19.7),
    (20.0, 6.0),
    (2.0, 6.0),
    (5.0, -3.0),
];

/// Two versions of one element with a shadow each, and a third unit with
/// none, on two CSS Screens: `css_two_version_corner_fixture`'s units and
/// steps. Unit a has one outer shadow, `UNIT_A_SHADOW`, and unit b a clear
/// one, `UNIT_B_SHADOW` with alpha 0. Unit a also holds a timeline nothing
/// plays, "pulse", keying its shadow's strength 10 to 30 over one second.
/// When `keyed`, B's timeline keys every flight slot's progress 0 to 1, a's
/// and c's seven shadow controls by `SHADOW_KEYS`, linear, over one second,
/// and b's alpha to 0 with one hold key.
pub fn css_two_version_shadow_fixture(keyed: bool) -> Vec<u8> {
    build_css_versions_fixture(
        0,
        None,
        UnitShadows {
            drawn: true,
            keyed,
            ..UnitShadows::default()
        },
    )
}
/// As `css_two_version_shadow_fixture(true)`, with every flight slot's alpha
/// held at -1: a's and c's other six stay keyed.
pub fn css_shadow_alpha_off_fixture() -> Vec<u8> {
    build_css_versions_fixture(
        0,
        None,
        UnitShadows {
            drawn: true,
            keyed: true,
            alpha_off: true,
            ..UnitShadows::default()
        },
    )
}
/// As `css_two_version_shadow_fixture(keyed)`, with unit a's shadow inset.
pub fn css_inset_shadow_fixture(keyed: bool) -> Vec<u8> {
    build_css_versions_fixture(
        0,
        None,
        UnitShadows {
            drawn: true,
            keyed,
            inset: true,
            ..UnitShadows::default()
        },
    )
}
/// `css_two_version_corner_fixture(false)` with its units in the compiler's
/// shape for a box root, so each unit clips by its record and not by its
/// frame. Each unit's child, `CSS_BOX_ROOT_CHILD`, sits over its background.
pub fn css_box_root_versions_fixture() -> Vec<u8> {
    build_css_versions_fixture(
        0,
        None,
        UnitShadows {
            box_root: true,
            ..UnitShadows::default()
        },
    )
}
/// The color of the child `css_box_root_versions_fixture` puts in each unit.
pub const CSS_BOX_ROOT_CHILD: u32 = 0xff22bb88;
/// `css_two_version_corner_fixture(false)` with unit c's home on A inside a
/// CSS box that clips both axes at `opacity`. The box sits at (0, 150) on A,
/// 400 by 150, clear of unit a, and paints nothing itself. Unit c is gray
/// `FADING_BOX_UNIT`. Step "t2" flies unit c alone, from that home. The page's
/// root clips by its frame, as the compiler writes a page.
pub fn css_flight_under_fading_box_fixture(opacity: f32) -> Vec<u8> {
    build_css_versions_fixture(
        0,
        None,
        UnitShadows {
            fading_home: Some(opacity),
            page_clips: true,
            ..UnitShadows::default()
        },
    )
}
/// Unit c's color in `css_flight_under_fading_box_fixture` and
/// `css_flight_over_backing_fixture`.
pub const FADING_BOX_UNIT: u32 = 0xffc8_c8c8;
/// `css_two_version_corner_fixture(false)` on a page whose root clips by its
/// frame, as the compiler writes a page. Screen B paints `SCREEN_B_BACKING`
/// under its slots, and unit c is gray `FADING_BOX_UNIT`. Step "t2" flies
/// unit c alone, from A to B, so its flight draws over B's backing.
pub fn css_flight_over_backing_fixture() -> Vec<u8> {
    build_css_versions_fixture(0, None, flight_over_backing())
}
/// Screen B's backing in `css_flight_over_backing_fixture`.
pub const SCREEN_B_BACKING: u32 = 0xff3c_3c3c;
/// `css_flight_over_backing_fixture` with videos: one named "clip" in unit c,
/// whose root also carries a button's semantics, and a plain one named
/// "plain" on Screen B, in front of B's slots.
pub fn css_flight_video_fixture() -> Vec<u8> {
    build_css_versions_fixture(
        0,
        None,
        UnitShadows {
            videos: true,
            ..flight_over_backing()
        },
    )
}
fn flight_over_backing() -> UnitShadows {
    UnitShadows {
        page_clips: true,
        b_backing: Some(SCREEN_B_BACKING),
        unit_c: Some(FADING_BOX_UNIT),
        ..UnitShadows::default()
    }
}
/// As `css_two_version_shadow_fixture(false)`, with one malformed record, on
/// shadow control number `control`. 1: A's resting slot for unit a carries a
/// value of 5. 2: unit a's flight slot carries a NaN.
pub fn css_bad_shadow_record_fixture(kind: u8, control: usize) -> Vec<u8> {
    build_css_versions_fixture(
        0,
        Some((kind, SHADOW_CONTROLS[control].0)),
        UnitShadows {
            drawn: true,
            ..UnitShadows::default()
        },
    )
}
/// How `build_css_versions_fixture` shapes and shadows its units.
#[derive(Clone, Copy, Default)]
struct UnitShadows {
    /// Units a and b draw their shadows, a's as `inset` says.
    drawn: bool,
    inset: bool,
    /// B's timeline keys them, or keys alpha -1 when `alpha_off`.
    keyed: bool,
    alpha_off: bool,
    /// Units take the compiler's shape for a box root: `clip: false`, the
    /// record's overflow hidden on both axes, the background as an own paint
    /// and a child directly in the draw list.
    box_root: bool,
    /// A's home slot for unit c sits in a box that clips both axes at this
    /// opacity, and unit c is gray `FADING_BOX_UNIT`.
    fading_home: Option<f32>,
    /// The page's root clips by its frame, as the compiler writes a page.
    page_clips: bool,
    /// Screen B paints this backing under its slots.
    b_backing: Option<u32>,
    /// Unit c's color, over the one the other options give it.
    unit_c: Option<u32>,
    /// A video named "clip" and a button's semantics in unit c, and a plain
    /// video named "plain" on Screen B.
    videos: bool,
}
/// A property keyed linearly from frame 0 to frame 60.
fn linear_track(b: &mut Vec<u8>, property: u64, from: f32, to: f32) {
    use V::*;
    obj(b, "KeyedProperty", &[("propertyKey", U(property))]);
    for (frame, value) in [(0, from), (60, to)] {
        obj(
            b,
            "KeyFrameDouble",
            &[
                ("frame", U(frame)),
                ("value", F(value)),
                ("interpolationType", U(1)),
            ],
        );
    }
}
/// A property held at `value` by one key at frame 0.
fn hold_track(b: &mut Vec<u8>, property: u64, value: f32) {
    use V::*;
    obj(b, "KeyedProperty", &[("propertyKey", U(property))]);
    obj(
        b,
        "KeyFrameDouble",
        &[
            ("frame", U(0)),
            ("value", F(value)),
            ("interpolationType", U(0)),
        ],
    );
}
/// B's shadow tracks for one flight slot: the seven controls on a's and c's,
/// alpha alone on b's.
fn slot_shadow_tracks(b: &mut Vec<u8>, unit_key: u64, shadows: UnitShadows) {
    let (alpha_key, (from, to)) = (SHADOW_CONTROLS[0].1, SHADOW_KEYS[0]);
    match (unit_key == 2, shadows.alpha_off) {
        (true, false) => hold_track(b, alpha_key, 0.0),
        (_, true) => hold_track(b, alpha_key, -1.0),
        (false, false) => linear_track(b, alpha_key, from, to),
    }
    if unit_key != 2 {
        for ((_, property), (from, to)) in SHADOW_CONTROLS.into_iter().zip(SHADOW_KEYS).skip(1) {
            linear_track(b, property, from, to);
        }
    }
}
/// A shadow paint on a unit's root, as the compiler writes one: a Fill with a
/// SolidColor named "shadow" and a local Feather holding a BoxShadow, objects
/// 3 to 6 of the unit's artboard.
fn unit_shadow_paint(b: &mut Vec<u8>, color: u32, shadow: Shadow, inner: bool) {
    use V::*;
    obj(b, "Fill", &[("parentId", U(0))]); // 3
    obj(
        b,
        "SolidColor",
        &[
            ("name", S("shadow")),
            ("parentId", U(3)),
            ("colorValue", Color(color)),
        ],
    ); // 4
    obj(
        b,
        "Feather",
        &[
            ("parentId", U(3)),
            ("spaceValue", U(1)),
            ("strength", F(shadow.blur)),
            ("offsetX", F(shadow.offset.0)),
            ("offsetY", F(shadow.offset.1)),
            ("inner", B(inner)),
        ],
    ); // 5
    obj(b, "BoxShadow", &[("parentId", U(5))]); // 6
}
/// A CSS slot on a Screen root: its host at `host`, then its item record,
/// `size` points at `margin` (top, left) in the flow, then its LiveSlot.
fn css_corner_slot(
    b: &mut Vec<u8>,
    host: u64,
    name: &str,
    unit: u64,
    size: (f32, f32),
    margin: (f32, f32),
    record: &[(&str, V<'_>)],
) {
    css_corner_slot_in(b, 0, host, name, unit, size, margin, record);
}
/// As `css_corner_slot`, in the box at `parent`.
#[allow(clippy::too_many_arguments)]
fn css_corner_slot_in(
    b: &mut Vec<u8>,
    parent: u64,
    host: u64,
    name: &str,
    unit: u64,
    size: (f32, f32),
    margin: (f32, f32),
    record: &[(&str, V<'_>)],
) {
    use V::*;
    obj(
        b,
        "NestedArtboardLayout",
        &[
            ("name", S(name)),
            ("parentId", U(parent)),
            ("artboardId", U(unit)),
        ],
    );
    let mut item = vec![
        ("parentId", U(host)),
        ("widthKind", U(1)),
        ("width", F(size.0)),
        ("heightKind", U(1)),
        ("height", F(size.1)),
    ];
    if margin != (0.0, 0.0) {
        item.extend([("marginTop", F(margin.0)), ("marginLeft", F(margin.1))]);
    }
    obj(b, "LayoutCssStyle", &item);
    let mut props = vec![("parentId", U(host))];
    props.extend_from_slice(record);
    obj(b, "LiveSlot", &props);
}
fn build_css_versions_fixture(
    keyed_corners: usize,
    bad_record: Option<(u8, &str)>,
    shadows: UnitShadows,
) -> Vec<u8> {
    use V::*;
    let mut b = b"RIVE".to_vec();
    for n in [7, 0, 0, 0] {
        var(&mut b, n);
    }
    obj(&mut b, "Backboard", &[]);
    if shadows.videos {
        obj(
            &mut b,
            "VideoAsset",
            &[
                ("sourceKey", S("asset:mascot")),
                ("contentType", S("video/mp4")),
                ("width", F(40.0)),
                ("height", F(40.0)),
            ],
        );
    }
    for name in ["A", "B"] {
        obj(&mut b, "ViewModel", &[("name", S(name))]);
    }
    obj(&mut b, "ViewModelInstance", &[("viewModelId", U(0))]);
    obj(&mut b, "ViewModelInstance", &[("viewModelId", U(1))]);
    obj(&mut b, "ViewModel", &[("name", S("Page"))]);
    obj(&mut b, "ViewModelPropertyList", &[("name", S("screens"))]);
    obj(&mut b, "ViewModelInstance", &[("viewModelId", U(2))]);
    obj(
        &mut b,
        "ViewModelInstanceList",
        &[("viewModelPropertyId", U(0))],
    );
    for model in [0, 1] {
        obj(
            &mut b,
            "ViewModelInstanceListItem",
            &[("viewModelId", U(model)), ("viewModelInstanceId", U(0))],
        );
    }
    // Screen A: unit a's home (3 to 5) and unit c's home (6 to 8).
    obj(
        &mut b,
        "Artboard",
        &[
            ("name", S("A")),
            ("viewModelId", U(0)),
            ("width", F(400.0)),
            ("height", F(300.0)),
            ("styleId", U(2)),
            ("clip", B(false)),
        ],
    ); // 0
    obj(&mut b, "LayoutCssStyle", &[("parentId", U(0))]); // 1
    obj(&mut b, "LayoutComponentStyle", &[("parentId", U(0))]); // 2
    let mut home_a = vec![("unitKey", U(1))];
    if let Some((1, control)) = bad_record {
        home_a.push((control, F(5.0)));
    }
    css_corner_slot(&mut b, 3, "a", 2, (80.0, 60.0), (20.0, 36.0), &home_a);
    match shadows.fading_home {
        None => css_corner_slot(
            &mut b,
            6,
            "c",
            4,
            (50.0, 40.0),
            (20.0, 36.0),
            &[("unitKey", U(3))],
        ),
        // The box (6 to 8), then unit c's home inside it (9 to 11).
        Some(opacity) => {
            obj(
                &mut b,
                "LayoutComponent",
                &[
                    ("name", S("fading box")),
                    ("parentId", U(0)),
                    ("styleId", U(7)),
                    ("opacity", F(opacity)),
                ],
            ); // 6
            obj(&mut b, "LayoutComponentStyle", &[("parentId", U(6))]); // 7
            let mut record = vec![("parentId", U(6))];
            record.extend(css_at(0.0, 150.0, 400.0, 150.0));
            record.extend([("overflowX", U(2)), ("overflowY", U(2))]);
            obj(&mut b, "LayoutCssStyle", &record); // 8
            css_corner_slot_in(
                &mut b,
                6,
                9,
                "c",
                4,
                (50.0, 40.0),
                (20.0, 36.0),
                &[("unitKey", U(3))],
            );
        }
    }
    // Screen B: unit b's home (3 to 5), unit c's landing home (6 to 8), and
    // the flight slots of unit a (9 to 11), unit b (12 to 14) and unit c
    // (15 to 17). A CSS flight slot leaves the flow as its Screen is built.
    obj(
        &mut b,
        "Artboard",
        &[
            ("name", S("B")),
            ("viewModelId", U(1)),
            ("width", F(400.0)),
            ("height", F(300.0)),
            ("styleId", U(2)),
            ("clip", B(false)),
            ("defaultStateMachineId", U(0)),
        ],
    ); // 0
    obj(&mut b, "LayoutCssStyle", &[("parentId", U(0))]); // 1
    obj(&mut b, "LayoutComponentStyle", &[("parentId", U(0))]); // 2
    css_corner_slot(
        &mut b,
        3,
        "b",
        3,
        (160.0, 120.0),
        (40.0, 120.0),
        &[("unitKey", U(2))],
    );
    css_corner_slot(
        &mut b,
        6,
        "c",
        4,
        (100.0, 80.0),
        (10.0, 20.0),
        &[("unitKey", U(3))],
    );
    let mut flight_records = Vec::new();
    for (name, unit, unit_key, host, size) in [
        ("a flight", 2, 1, 9, (80.0, 60.0)),
        ("b flight", 3, 2, 12, (160.0, 120.0)),
        ("c flight", 4, 3, 15, (50.0, 40.0)),
    ] {
        let mut record = vec![
            ("unitKey", U(unit_key)),
            ("role", U(1)),
            ("flightKey", U(10 + unit_key)),
            ("sizeMode", U(1)),
        ];
        if let (Some((2, control)), 1) = (bad_record, unit_key) {
            record.push((control, F(f32::NAN)));
        }
        css_corner_slot(&mut b, host, name, unit, size, (0.0, 0.0), &record);
        flight_records.push((unit_key, host + 2));
    }
    // The root's own paint draws under its slots (18 and 19), and the plain
    // video draws in front of them.
    if let Some(color) = shadows.b_backing {
        obj(&mut b, "Fill", &[("parentId", U(0))]); // 18
        obj(
            &mut b,
            "SolidColor",
            &[("parentId", U(18)), ("colorValue", Color(color))],
        ); // 19
    }
    if shadows.videos {
        obj(
            &mut b,
            "Video",
            &[("name", S("plain")), ("parentId", U(0)), ("assetId", U(0))],
        );
    }
    let keyed = keyed_corners > 0 || shadows.keyed;
    if keyed {
        obj(
            &mut b,
            "LinearAnimation",
            &[
                ("name", S("t1")),
                ("fps", U(60)),
                ("duration", U(60)),
                ("loopValue", U(0)),
            ],
        );
        for (unit_key, record) in flight_records {
            obj(&mut b, "KeyedObject", &[("objectId", U(record))]);
            for (property, from, to) in std::iter::once(("progress", 0.0, 1.0))
                .chain(CORNERS[..keyed_corners].iter().map(|c| (*c, 4.0, 8.0)))
            {
                linear_track(&mut b, key("LiveSlot", property).into(), from, to);
            }
            if shadows.keyed {
                slot_shadow_tracks(&mut b, unit_key, shadows);
            }
        }
    }
    machine(&mut b, keyed);
    // The units: CSS artboards that draw their own background.
    for (name, width, height, corner, color) in [
        ("unit a", 80.0, 60.0, 4.0, 0xffcc3333),
        ("unit b", 160.0, 120.0, 8.0, 0xff3333cc),
        (
            "unit c",
            50.0,
            40.0,
            0.0,
            shadows.unit_c.unwrap_or(if shadows.fading_home.is_some() {
                FADING_BOX_UNIT
            } else {
                0xff33cc33
            }),
        ),
    ] {
        obj(
            &mut b,
            "Artboard",
            &[
                ("name", S(name)),
                ("width", F(width)),
                ("height", F(height)),
                ("styleId", U(2)),
                ("clip", B(false)),
            ],
        ); // 0
        let mut record = vec![("parentId", U(0))];
        if shadows.box_root {
            record.extend([("overflowX", U(2)), ("overflowY", U(2))]);
        }
        obj(&mut b, "LayoutCssStyle", &record); // 1
        obj(
            &mut b,
            "LayoutComponentStyle",
            &[("parentId", U(0)), ("cornerRadiusTL", F(corner))],
        ); // 2
        // A shadow's Fill comes first, so it draws below the background.
        let shadow = match name {
            "unit a" if shadows.drawn => Some((UNIT_A_SHADOW.color, UNIT_A_SHADOW, shadows.inset)),
            "unit b" if shadows.drawn => {
                Some((UNIT_B_SHADOW.color & 0x00ff_ffff, UNIT_B_SHADOW, false))
            }
            _ => None,
        };
        if let Some((color, shadow, inner)) = shadow {
            unit_shadow_paint(&mut b, color, shadow, inner);
        }
        let fill = 3 + if shadow.is_some() { 4 } else { 0 };
        obj(&mut b, "Fill", &[("parentId", U(0))]); // 3, or 7 after a shadow
        obj(
            &mut b,
            "SolidColor",
            &[
                ("name", S("ink")),
                ("parentId", U(fill)),
                ("colorValue", Color(color)),
            ],
        ); // 4, or 8
        if shadows.box_root {
            let child = fill + 2;
            obj(
                &mut b,
                "LayoutComponent",
                &[
                    ("name", S("child")),
                    ("parentId", U(0)),
                    ("styleId", U(child + 1)),
                ],
            );
            obj(&mut b, "LayoutComponentStyle", &[("parentId", U(child))]);
            let mut record = vec![("parentId", U(child))];
            record.extend(css_at(10.0, 10.0, 30.0, 20.0));
            obj(&mut b, "LayoutCssStyle", &record);
            obj(&mut b, "Fill", &[("parentId", U(child))]);
            obj(
                &mut b,
                "SolidColor",
                &[
                    ("parentId", U(child + 3)),
                    ("colorValue", Color(CSS_BOX_ROOT_CHILD)),
                ],
            );
        }
        if shadows.videos && name == "unit c" {
            obj(
                &mut b,
                "Video",
                &[("name", S("clip")), ("parentId", U(0)), ("assetId", U(0))],
            );
            obj(
                &mut b,
                "SemanticData",
                &[("parentId", U(0)), ("role", U(1)), ("label", S("c button"))],
            );
        }
        if shadow.is_some() && name == "unit a" {
            obj(
                &mut b,
                "LinearAnimation",
                &[
                    ("name", S("pulse")),
                    ("fps", U(60)),
                    ("duration", U(60)),
                    ("loopValue", U(0)),
                ],
            );
            obj(&mut b, "KeyedObject", &[("objectId", U(5))]);
            linear_track(&mut b, key("Feather", "strength").into(), 10.0, 30.0);
        }
    }
    obj(
        &mut b,
        "Artboard",
        &[
            ("name", S("Page")),
            ("viewModelId", U(2)),
            ("width", F(400.0)),
            ("height", F(600.0)),
            ("clip", B(shadows.page_clips)),
            ("defaultStateMachineId", U(0)),
        ],
    ); // 0
    obj(
        &mut b,
        "ArtboardComponentList",
        &[("name", S("screens")), ("parentId", U(0))],
    ); // 1
    bind(&mut b, 800, &[2, 0]);
    for (step, id, outgoing, incoming) in [("t1", 2, 1, 2), ("t2", 4, 3, 3)] {
        obj(
            &mut b,
            "LiveStep",
            &[
                ("parentId", U(0)),
                ("name", S(step)),
                ("fromArtboard", S("A")),
                ("toArtboard", S("B")),
            ],
        ); // 2, 4
        obj(
            &mut b,
            "LiveStepUnit",
            &[
                ("parentId", U(id)),
                ("incomingUnitKey", U(incoming)),
                ("outgoingUnitKey", U(outgoing)),
                ("incomingFlightKey", U(10 + incoming)),
                ("outgoingFlightKey", U(10 + outgoing)),
            ],
        ); // 3, 5
    }
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

/// A record-free page of `rows` Screens, each with a Form answer and a tap
/// target whose listener action runs `payload` (a form script). Each Screen's
/// Form has a one-pick answer `field` (options x and y, x picked) and a text
/// answer `title` ("before"); its `mode` number starts at 1.
pub fn form_script_page_fixture(payload: &[u8], rows: usize) -> Vec<u8> {
    use V::*;
    assert!((2..=64).contains(&rows));
    let rows = rows as u64;
    let page_model = rows + 3;
    let width = rows as f32 * 300.0 + 200.0;
    let mut b = b"RIVE".to_vec();
    for n in [7, 0, 0, 0] {
        var(&mut b, n);
    }
    obj(&mut b, "Backboard", &[]);
    obj(
        &mut b,
        "ScriptAsset",
        &[("assetId", U(0)), ("name", S("ResponseAction"))],
    );
    obj(&mut b, "FileAssetContents", &[("bytes", Bytes(payload))]);
    obj(&mut b, "ViewModel", &[("name", S("Choice"))]);
    obj(&mut b, "ViewModelPropertyString", &[("name", S("value"))]);
    obj(&mut b, "ViewModelPropertyBoolean", &[("name", S("picked"))]);
    for i in 0..rows * 2 {
        obj(&mut b, "ViewModelInstance", &[("viewModelId", U(0))]);
        obj(
            &mut b,
            "ViewModelInstanceString",
            &[
                ("viewModelPropertyId", U(0)),
                ("propertyValue", S(if i % 2 == 0 { "x" } else { "y" })),
            ],
        );
        obj(
            &mut b,
            "ViewModelInstanceBoolean",
            &[
                ("viewModelPropertyId", U(1)),
                ("propertyValue", B(i % 2 == 0)),
            ],
        );
    }
    obj(&mut b, "ViewModel", &[("name", S("Form"))]);
    obj(&mut b, "ViewModelPropertyList", &[("name", S("field"))]);
    for name in ["option:field:78", "option:field:79"] {
        obj(
            &mut b,
            "ViewModelPropertyViewModel",
            &[("name", S(&name)), ("viewModelReferenceId", U(0))],
        );
    }
    obj(&mut b, "ViewModelPropertyString", &[("name", S("title"))]);
    for row in 0..rows {
        obj(&mut b, "ViewModelInstance", &[("viewModelId", U(1))]);
        obj(
            &mut b,
            "ViewModelInstanceList",
            &[("viewModelPropertyId", U(0))],
        );
        for i in 0..2 {
            obj(
                &mut b,
                "ViewModelInstanceListItem",
                &[
                    ("viewModelId", U(0)),
                    ("viewModelInstanceId", U(row * 2 + i)),
                ],
            );
        }
        for i in 0..2 {
            obj(
                &mut b,
                "ViewModelInstanceViewModel",
                &[
                    ("viewModelPropertyId", U(i + 1)),
                    ("propertyValue", U(row * 2 + i)),
                ],
            );
        }
        obj(
            &mut b,
            "ViewModelInstanceString",
            &[
                ("viewModelPropertyId", U(3)),
                ("propertyValue", S("before")),
            ],
        );
    }
    obj(&mut b, "ViewModel", &[("name", S("Experience"))]);
    obj(
        &mut b,
        "ViewModelPropertyViewModel",
        &[
            ("name", S("responses:form")),
            ("viewModelReferenceId", U(1)),
        ],
    );
    for row in 0..rows {
        obj(&mut b, "ViewModelInstance", &[("viewModelId", U(2))]);
        obj(
            &mut b,
            "ViewModelInstanceViewModel",
            &[("viewModelPropertyId", U(0)), ("propertyValue", U(row))],
        );
    }
    for row in 0..rows {
        obj(&mut b, "ViewModel", &[("name", S(&format!("Screen{row}")))]);
        obj(
            &mut b,
            "ViewModelPropertyViewModel",
            &[("name", S("experience")), ("viewModelReferenceId", U(2))],
        );
        obj(&mut b, "ViewModelPropertyNumber", &[("name", S("mode"))]);
        obj(
            &mut b,
            "ViewModelInstance",
            &[("viewModelId", U(row as u64 + 3))],
        );
        obj(
            &mut b,
            "ViewModelInstanceViewModel",
            &[
                ("viewModelPropertyId", U(0)),
                ("propertyValue", U(row as u64)),
            ],
        );
        obj(
            &mut b,
            "ViewModelInstanceNumber",
            &[("viewModelPropertyId", U(1)), ("propertyValue", F(1.0))],
        );
    }
    obj(&mut b, "ViewModel", &[("name", S("Page"))]);
    obj(&mut b, "ViewModelPropertyList", &[("name", S("screens"))]);
    obj(
        &mut b,
        "ViewModelInstance",
        &[("viewModelId", U(page_model))],
    );
    obj(
        &mut b,
        "ViewModelInstanceList",
        &[("viewModelPropertyId", U(0))],
    );
    for row in 0..rows {
        obj(
            &mut b,
            "ViewModelInstanceListItem",
            &[("viewModelId", U(row + 3)), ("viewModelInstanceId", U(0))],
        );
    }
    obj(
        &mut b,
        "Artboard",
        &[
            ("name", S("$nuxie.page")),
            ("width", F(width)),
            ("height", F(200.0)),
            ("viewModelId", U(page_model)),
            ("defaultStateMachineId", U(0)),
        ],
    );
    obj(&mut b, "Node", &[("parentId", U(0))]);
    obj(
        &mut b,
        "ArtboardComponentList",
        &[("name", S("screens")), ("parentId", U(1))],
    );
    bind(&mut b, 800, &[page_model as u8, 0]);
    machine(&mut b, false);
    for row in 0..rows {
        obj(
            &mut b,
            "Artboard",
            &[
                ("name", S(&format!("S{row}"))),
                ("width", F(width)),
                ("height", F(200.0)),
                ("viewModelId", U(row as u64 + 3)),
                ("defaultStateMachineId", U(0)),
            ],
        );
        obj(
            &mut b,
            "Shape",
            &[
                ("name", S("action")),
                ("parentId", U(0)),
                ("x", F(100.0 + row as f32 * 300.0)),
                ("y", F(50.0)),
            ],
        );
        obj(
            &mut b,
            "Rectangle",
            &[("parentId", U(1)), ("width", F(100.0)), ("height", F(80.0))],
        );
        machine(&mut b, false);
        obj(
            &mut b,
            "StateMachineListenerSingle",
            &[("targetId", U(1)), ("listenerTypeValue", U(2))],
        );
        obj(&mut b, "ScriptedListenerAction", &[("scriptAssetId", U(0))]);
    }
    b
}

/// The view models of `css_paint_fixture`, in file order.
const PAINT_UNIT_VM: u64 = 0;
const PAINT_A_VM: u64 = 1;
const PAINT_B_VM: u64 = 2;
const PAINT_PAGE_VM: u64 = 3;
/// `DrawableFlag::OPAQUE`: a hit on this drawable blocks what is behind it.
const PAINT_OPAQUE: u64 = 1 << 3;

/// The paint order cases (UNIV-5635). Each is a page of two CSS Screens,
/// A and B, 400 by 300, both rows at the page's origin, B drawn above A.
/// Every box and unit fills with `paint_color` of its own name. B, the
/// upper Screen, holds the flight slots; step "go" flies A to B, and "back"
/// flies B to A. B's "entrance" timeline plays from load over one second,
/// so a test that advances by each flight's progress sees B's keyed tracks
/// at that progress going forward, and at rest on the way back.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum PaintCase {
    AppStoreCard,
    Match {
        red_above: bool,
    },
    Calmly,
    ListToGrid {
        grid_above: bool,
    },
    Catzy,
    ClippingHolder,
    BadgeInCard,
    OverlayInWrapper,
    DisplayWrite,
    ThreeAnchors,
    /// Test 11's cases, 1 to 11.
    Resting(u8),
    /// Test 12's keyed layout, by the box the test reads: 'C', 'D', 'F', 'G',
    /// 'H', 'K', 'L', 'M', 'N', 'O', 'P', 'Q', 'R', 'T', 'U', 'V' or 'W'.
    KeyedLayout(char),
    BoundZIndex,
    Hinge {
        flex: bool,
    },
    Sizes,
    TwoVersions,
    PlainAnchor,
    TurnedFrame,
    FadeWrite,
    FadeAtRest {
        native: bool,
    },
    MaskedGroup,
    Cancel,
}

/// The opaque fill color of a box or unit named `name` in
/// `css_paint_fixture`: its FNV-1a hash.
pub fn paint_color(name: &str) -> u32 {
    let mut hash: u32 = 0x811c_9dc5;
    for byte in name.bytes() {
        hash ^= u32::from(byte);
        hash = hash.wrapping_mul(0x0100_0193);
    }
    0xff00_0000 | (hash & 0x00ff_ffff)
}

pub(crate) type Css = Vec<(&'static str, V<'static>)>;

pub(crate) fn css_size(width: f32, height: f32) -> Css {
    vec![
        ("widthKind", V::U(1)),
        ("width", V::F(width)),
        ("heightKind", V::U(1)),
        ("height", V::F(height)),
    ]
}
/// Absolute, at (`left`, `top`) in its containing block.
pub(crate) fn css_at(left: f32, top: f32, width: f32, height: f32) -> Css {
    let mut css = vec![
        ("position", V::U(2)),
        ("insetLeftKind", V::U(1)),
        ("insetLeft", V::F(left)),
        ("insetTopKind", V::U(1)),
        ("insetTop", V::F(top)),
    ];
    css.extend(css_size(width, height));
    css
}
/// In the flow, with a top and a left margin.
fn css_flow(width: f32, height: f32, top: f32, left: f32) -> Css {
    let mut css = css_size(width, height);
    if top != 0.0 {
        css.push(("marginTop", V::F(top)));
    }
    if left != 0.0 {
        css.push(("marginLeft", V::F(left)));
    }
    css
}
fn css_with(mut css: Css, more: &[(&'static str, V<'static>)]) -> Css {
    css.extend_from_slice(more);
    css
}
fn css_rel(css: Css) -> Css {
    css_with(css, &[("position", V::U(1))])
}
fn css_z(css: Css, z: f32) -> Css {
    css_with(css, &[("zIndexKind", V::U(1)), ("zIndex", V::F(z))])
}

#[derive(Clone)]
enum PaintKind {
    /// A CSS box.
    Box,
    /// A CSS slot for a unit, with a LiveSlot of this unit key unless 0.
    Slot { unit: &'static str, key: u64 },
    /// A plain NestedArtboard at (x, y), with an empty record.
    Plain {
        unit: &'static str,
        key: u64,
        x: f32,
        y: f32,
    },
    /// A CSS flight slot, sized to its unit.
    Flight {
        unit: &'static str,
        key: u64,
        flight: u64,
        mode: u64,
    },
    /// A native layout flight slot, in an absolute Rive layout box.
    Native {
        unit: &'static str,
        key: u64,
        flight: u64,
        mode: u64,
    },
    /// A plain node, which layout sees through.
    Group,
    /// A paintless translated layout box, with no record, that hugs what it
    /// holds. Its `css` is more of its `LayoutComponentStyle`.
    Translated,
    /// An `ArtboardComponentList`, bound to B's list of rows.
    List,
}

#[derive(Clone)]
struct PaintEl {
    name: String,
    kind: PaintKind,
    css: Css,
    props: Vec<(&'static str, V<'static>)>,
    fill: bool,
    kids: Vec<PaintEl>,
    /// (property key, B number) bindings on the element itself.
    binds: Vec<(u64, &'static str)>,
    /// (property key, B number) bindings on its record.
    css_binds: Vec<(u64, &'static str)>,
    origin: Option<f32>,
    /// Grid column tracks, in points.
    tracks: Vec<f32>,
    column: Option<i64>,
    /// The name of the box masking this one.
    mask: Option<&'static str>,
}
impl PaintEl {
    fn new(name: &str, kind: PaintKind, css: Css) -> Self {
        Self {
            name: name.to_owned(),
            kind,
            css,
            props: Vec::new(),
            fill: true,
            kids: Vec::new(),
            binds: Vec::new(),
            css_binds: Vec::new(),
            origin: None,
            tracks: Vec::new(),
            column: None,
            mask: None,
        }
    }
    fn paintless(mut self) -> Self {
        self.fill = false;
        self
    }
    fn kids(mut self, kids: Vec<PaintEl>) -> Self {
        self.kids = kids;
        self
    }
    fn props(mut self, props: &[(&'static str, V<'static>)]) -> Self {
        self.props.extend_from_slice(props);
        self
    }
    fn bind(mut self, key: u64, number: &'static str) -> Self {
        self.binds.push((key, number));
        self
    }
    fn css_bind(mut self, key: u64, number: &'static str) -> Self {
        self.css_binds.push((key, number));
        self
    }
    fn origin(mut self, origin: f32) -> Self {
        self.origin = Some(origin);
        self
    }
    fn tracks(mut self, tracks: &[f32]) -> Self {
        self.tracks = tracks.to_vec();
        self
    }
    fn column(mut self, column: i64) -> Self {
        self.column = Some(column);
        self
    }
    fn mask(mut self, source: &'static str) -> Self {
        self.mask = Some(source);
        self
    }
}
fn bx(name: &str, css: Css) -> PaintEl {
    PaintEl::new(name, PaintKind::Box, css)
}
/// A resting slot named for its unit.
fn rest(unit: &'static str, key: u64, css: Css) -> PaintEl {
    PaintEl::new(unit, PaintKind::Slot { unit, key }, css)
}
/// A resting slot named `name` for `unit`.
fn rest_as(name: &str, unit: &'static str, key: u64, css: Css) -> PaintEl {
    PaintEl::new(name, PaintKind::Slot { unit, key }, css)
}
fn at_rest(unit: &'static str, key: u64, left: f32, top: f32, width: f32, height: f32) -> PaintEl {
    rest(unit, key, css_at(left, top, width, height))
}
fn plain(unit: &'static str, key: u64, x: f32, y: f32) -> PaintEl {
    PaintEl::new(unit, PaintKind::Plain { unit, key, x, y }, Vec::new())
}
fn flight_slot(name: &str, unit: &'static str, key: u64, flight: u64, mode: u64) -> PaintEl {
    PaintEl::new(
        name,
        PaintKind::Flight {
            unit,
            key,
            flight,
            mode,
        },
        Vec::new(),
    )
}
/// Unit `unit`'s Layout-mode CSS flight slot, "`unit` flight".
fn fly(unit: &'static str, key: u64, flight: u64) -> PaintEl {
    flight_slot(&format!("{unit} flight"), unit, key, flight, 1)
}
fn native_slot(name: &str, unit: &'static str, key: u64, flight: u64, mode: u64) -> PaintEl {
    PaintEl::new(
        name,
        PaintKind::Native {
            unit,
            key,
            flight,
            mode,
        },
        Vec::new(),
    )
}

/// A unit: a CSS artboard filled with its own color. A turned or scaled
/// unit pivots about its center. A scaled unit binds its scale to the Unit
/// model's "scale", 2 in every resting slot that holds it. A tappable unit
/// has an opaque, paintless target of its size and a machine whose "hits"
/// a pointer down on that target sets to 9, and each slot that holds it
/// has a `NestedStateMachine`. A keyed unit plays its own timeline, one
/// second from load, through such a machine.
#[derive(Clone, Copy)]
struct PaintUnit {
    name: &'static str,
    size: (f32, f32),
    turn: f32,
    scaled: bool,
    opacity: f32,
    tappable: bool,
    keyed: Option<UnitKey>,
}
/// What a unit's own timeline keys, from 0 to this many points.
#[derive(Clone, Copy)]
enum UnitKey {
    /// The height of a box S, 300 wide, in flow inside it.
    Child(f32),
    /// The top padding on its own record.
    Padding(f32),
    /// As a list row: the height on its own record, which also gives it its
    /// width and size, and its default machine plays the timeline.
    RowHeight(f32),
}
fn unit(name: &'static str, width: f32, height: f32) -> PaintUnit {
    PaintUnit {
        name,
        size: (width, height),
        turn: 0.0,
        scaled: false,
        opacity: 1.0,
        tappable: false,
        keyed: None,
    }
}
impl PaintUnit {
    fn turned(mut self, turn: f32) -> Self {
        self.turn = turn;
        self
    }
    fn scaled(mut self) -> Self {
        self.scaled = true;
        self
    }
    fn faded(mut self, opacity: f32) -> Self {
        self.opacity = opacity;
        self
    }
    fn tappable(mut self) -> Self {
        self.tappable = true;
        self
    }
    fn keyed(mut self, keyed: UnitKey) -> Self {
        self.keyed = Some(keyed);
        self
    }
}
/// A number "hits", which a pointer down on object `target` sets to 9.
fn hit_listener(b: &mut Vec<u8>, target: u64) {
    use V::*;
    obj(
        b,
        "StateMachineNumber",
        &[("name", S("hits")), ("value", F(0.0))],
    );
    obj(
        b,
        "StateMachineListenerSingle",
        &[("targetId", U(target)), ("listenerTypeValue", U(2))],
    );
    obj(
        b,
        "ListenerNumberChange",
        &[("inputId", U(0)), ("value", F(9.0))],
    );
}
fn paint_unit(b: &mut Vec<u8>, u: &PaintUnit) {
    use V::*;
    let mut props = vec![
        ("name", S(u.name)),
        ("width", F(u.size.0)),
        ("height", F(u.size.1)),
        ("styleId", U(2)),
        ("clip", B(false)),
    ];
    if u.turn != 0.0 || u.scaled {
        props.extend([("originX", F(0.5)), ("originY", F(0.5))]);
    }
    if u.turn != 0.0 {
        props.push(("rotation", F(u.turn)));
    }
    if u.opacity != 1.0 {
        props.push(("opacity", F(u.opacity)));
    }
    if u.scaled {
        props.push(("viewModelId", U(PAINT_UNIT_VM)));
    }
    let row = matches!(u.keyed, Some(UnitKey::RowHeight(_)));
    if row {
        props.extend([
            ("viewModelId", U(PAINT_UNIT_VM)),
            ("defaultStateMachineId", U(0)),
        ]);
    }
    obj(b, "Artboard", &props); // 0
    if u.scaled {
        bind(b, 16, &[PAINT_UNIT_VM as u8, 0]);
        bind(b, 17, &[PAINT_UNIT_VM as u8, 0]);
    }
    let mut record = vec![("parentId", U(0))];
    if row {
        record.extend(css_size(u.size.0, u.size.1));
    }
    obj(b, "LayoutCssStyle", &record); // 1
    obj(b, "LayoutComponentStyle", &[("parentId", U(0))]); // 2
    obj(b, "Fill", &[("parentId", U(0))]); // 3
    obj(
        b,
        "SolidColor",
        &[
            ("parentId", U(3)),
            ("colorValue", Color(paint_color(u.name))),
        ],
    ); // 4
    if u.tappable {
        obj(
            b,
            "Shape",
            &[
                ("parentId", U(0)),
                ("x", F(u.size.0 / 2.0)),
                ("y", F(u.size.1 / 2.0)),
                ("drawableFlags", U(PAINT_OPAQUE)),
            ],
        ); // 5
        obj(
            b,
            "Rectangle",
            &[
                ("parentId", U(5)),
                ("width", F(u.size.0)),
                ("height", F(u.size.1)),
            ],
        );
        machine(b, false);
        hit_listener(b, 5);
    }
    if let Some(keyed) = u.keyed {
        let mut art = PaintArt {
            b: &mut *b,
            units: &[],
            numbers: &[],
            next: 5,
            names: [("frame.css".to_owned(), 1)].into(),
            masks: Vec::new(),
        };
        let (target, key, to) = match keyed {
            UnitKey::Child(to) => {
                art.emit(&bx("S", css_size(300.0, 0.0)).paintless(), 0);
                ("S.css", 61025, to)
            }
            UnitKey::Padding(to) => ("frame.css", 61063, to),
            UnitKey::RowHeight(to) => ("frame.css", 61025, to),
        };
        let names = art.names;
        let grows = track(target, key, &[(0, 0.0), (60, to)]);
        paint_entrance(b, &[grows], &names);
        machine(b, true);
    }
}

/// A track of B's entrance timeline, on a named object of B: an element,
/// "`name`.css" (its record), "`name`.track`n`" (its nth grid track) or
/// "`name`.place" (its grid placement).
struct PaintTrack {
    target: String,
    key: u64,
    frames: Vec<(u64, f32)>,
    hold: bool,
    int: bool,
}
fn track(target: &str, key: u64, frames: &[(u64, f32)]) -> PaintTrack {
    PaintTrack {
        target: target.to_owned(),
        key,
        frames: frames.to_vec(),
        hold: false,
        int: false,
    }
}
fn hold(target: &str, key: u64, frames: &[(u64, f32)]) -> PaintTrack {
    PaintTrack {
        hold: true,
        ..track(target, key, frames)
    }
}

/// A step: name, from, to, and its units as [incoming unit key, outgoing
/// unit key, incoming flight key, outgoing flight key].
type PaintStep = (&'static str, &'static str, &'static str, Vec<[u64; 4]>);

struct PaintSpec {
    size: (f32, f32),
    units: Vec<PaintUnit>,
    a: Vec<PaintEl>,
    b: Vec<PaintEl>,
    tracks: Vec<PaintTrack>,
    /// B's model numbers.
    numbers: Vec<(&'static str, f32)>,
    /// The B box whose pointer down sets B's "hits" to 9.
    listener: Option<&'static str>,
    steps: Vec<PaintStep>,
    /// B's root as a CSS grid with these column tracks, in points, named
    /// "root.track0" on.
    root_grid: Vec<f32>,
    /// B's model has a list of one row, an item of the Unit model, so the
    /// row is the unit whose artboard has that model.
    list: bool,
}
impl PaintSpec {
    fn new(units: Vec<PaintUnit>, a: Vec<PaintEl>, b: Vec<PaintEl>) -> Self {
        Self {
            size: (400.0, 300.0),
            units,
            a,
            b,
            tracks: Vec::new(),
            numbers: Vec::new(),
            listener: None,
            steps: Vec::new(),
            root_grid: Vec::new(),
            list: false,
        }
    }
    fn root_grid(mut self, tracks: &[f32]) -> Self {
        self.root_grid = tracks.to_vec();
        self
    }
    /// One-version units, (unit key, flight key): "go" A to B, "back" B to A.
    fn one_version(mut self, units: &[(u64, u64)]) -> Self {
        let units: Vec<_> = units.iter().map(|&(k, f)| [k, k, f, f]).collect();
        self.steps.push(("go", "A", "B", units.clone()));
        self.steps.push(("back", "B", "A", units));
        self
    }
    fn tracks(mut self, tracks: Vec<PaintTrack>) -> Self {
        self.tracks = tracks;
        self
    }
    fn numbers(mut self, numbers: &[(&'static str, f32)]) -> Self {
        self.numbers = numbers.to_vec();
        self
    }
}

struct PaintArt<'a> {
    b: &'a mut Vec<u8>,
    units: &'a [PaintUnit],
    numbers: &'a [(&'static str, f32)],
    next: u64,
    names: std::collections::HashMap<String, u64>,
    masks: Vec<(u64, &'static str)>,
}
impl PaintArt<'_> {
    fn add(&mut self, kind: &str, props: &[(&str, V<'_>)]) -> u64 {
        obj(self.b, kind, props);
        self.next += 1;
        self.next - 1
    }
    fn unit(&self, name: &str) -> (u64, PaintUnit) {
        let at = self
            .units
            .iter()
            .position(|u| u.name == name)
            .unwrap_or_else(|| panic!("unit {name}"));
        (2 + at as u64, self.units[at])
    }
    fn number(&self, name: &str) -> u8 {
        self.numbers
            .iter()
            .position(|(n, _)| *n == name)
            .unwrap_or_else(|| panic!("number {name}")) as u8
    }
    fn binds(&mut self, binds: &[(u64, &'static str)]) {
        for &(key, number) in binds {
            let number = self.number(number);
            bind(self.b, key, &[PAINT_B_VM as u8, number]);
        }
    }
    fn record(&mut self, el: &PaintEl, parent: u64, css: &[(&'static str, V<'static>)]) {
        let mut props = vec![("parentId", V::U(parent))];
        props.extend_from_slice(css);
        let id = self.add("LayoutCssStyle", &props);
        self.names.insert(format!("{}.css", el.name), id);
        self.binds(&el.css_binds);
    }
    fn scale_model(&mut self, slot: u64) {
        let model = self.add(
            "ViewModelInstance",
            &[
                ("viewModelId", V::U(PAINT_UNIT_VM)),
                ("parentId", V::U(slot)),
            ],
        );
        self.add(
            "ViewModelInstanceNumber",
            &[
                ("parentId", V::U(model)),
                ("viewModelPropertyId", V::U(0)),
                ("propertyValue", V::F(2.0)),
            ],
        );
    }
    /// The wrapper that holds a tappable or keyed unit's machine.
    fn unit_machine(&mut self, u: PaintUnit, slot: u64) {
        if u.tappable || u.keyed.is_some() {
            self.add(
                "NestedStateMachine",
                &[("parentId", V::U(slot)), ("animationId", V::U(0))],
            );
        }
    }
    fn origin(&mut self, el: &PaintEl, slot: u64) {
        if let Some(origin) = el.origin {
            self.add(
                "ComponentOrigin",
                &[
                    ("parentId", V::U(slot)),
                    ("originX", V::F(origin)),
                    ("originY", V::F(origin)),
                ],
            );
        }
    }
    fn emit(&mut self, el: &PaintEl, parent: u64) {
        use V::*;
        let id = self.next;
        self.names.insert(el.name.clone(), id);
        match el.kind.clone() {
            PaintKind::Box => {
                let mut props = vec![
                    ("name", S(&el.name)),
                    ("parentId", U(parent)),
                    ("styleId", U(id + 1)),
                ];
                props.extend_from_slice(&el.props);
                self.add("LayoutComponent", &props);
                self.binds(&el.binds);
                self.add("LayoutComponentStyle", &[("parentId", U(id))]);
                self.record(el, id, &el.css);
                if el.fill {
                    let fill = self.add("Fill", &[("parentId", U(id))]);
                    self.add(
                        "SolidColor",
                        &[
                            ("parentId", U(fill)),
                            ("colorValue", Color(paint_color(&el.name))),
                        ],
                    );
                }
                for (n, &size) in el.tracks.iter().enumerate() {
                    let track = self.add(
                        "GridTrack",
                        &[
                            ("parentId", U(id)),
                            ("collection", U(0)),
                            ("trackType", U(1)),
                            ("trackValue", F(size)),
                        ],
                    );
                    self.names.insert(format!("{}.track{n}", el.name), track);
                }
                if let Some(column) = el.column {
                    let place = self.add(
                        "GridItemPlacement",
                        &[("parentId", U(id)), ("gridColumn", I(column))],
                    );
                    self.names.insert(format!("{}.place", el.name), place);
                }
                if let Some(source) = el.mask {
                    self.masks.push((id, source));
                }
                for kid in &el.kids {
                    self.emit(kid, id);
                }
            }
            PaintKind::Slot { unit, key } | PaintKind::Plain { unit, key, .. } => {
                let (artboard, u) = self.unit(unit);
                let mut props = vec![
                    ("name", S(&el.name)),
                    ("parentId", U(parent)),
                    ("artboardId", U(artboard)),
                ];
                let host = if let PaintKind::Plain { x, y, .. } = el.kind {
                    props.extend([("x", F(x)), ("y", F(y))]);
                    "NestedArtboard"
                } else {
                    "NestedArtboardLayout"
                };
                self.add(host, &props);
                self.binds(&el.binds);
                self.record(el, id, &el.css);
                if key != 0 {
                    self.add("LiveSlot", &[("parentId", U(id)), ("unitKey", U(key))]);
                }
                if u.scaled {
                    self.scale_model(id);
                }
                self.unit_machine(u, id);
                self.origin(el, id);
            }
            PaintKind::Flight {
                unit,
                key,
                flight,
                mode,
            } => {
                let (artboard, u) = self.unit(unit);
                self.add(
                    "NestedArtboardLayout",
                    &[
                        ("name", S(&el.name)),
                        ("parentId", U(parent)),
                        ("artboardId", U(artboard)),
                    ],
                );
                let css = if el.css.is_empty() {
                    css_size(u.size.0, u.size.1)
                } else {
                    el.css.clone()
                };
                self.record(el, id, &css);
                self.add(
                    "LiveSlot",
                    &[
                        ("parentId", U(id)),
                        ("unitKey", U(key)),
                        ("role", U(1)),
                        ("flightKey", U(flight)),
                        ("sizeMode", U(mode)),
                    ],
                );
                self.unit_machine(u, id);
                self.origin(el, id);
            }
            PaintKind::Native {
                unit,
                key,
                flight,
                mode,
            } => {
                let (artboard, u) = self.unit(unit);
                let wrapper = format!("{} box", el.name);
                self.add(
                    "LayoutComponent",
                    &[
                        ("name", S(&wrapper)),
                        ("parentId", U(parent)),
                        ("styleId", U(id + 1)),
                    ],
                );
                self.add(
                    "LayoutComponentStyle",
                    &[
                        ("parentId", U(id)),
                        ("positionTypeValue", U(2)),
                        ("layoutWidthScaleType", U(2)),
                        ("layoutHeightScaleType", U(2)),
                    ],
                );
                let slot = self.add(
                    "NestedArtboardLayout",
                    &[
                        ("name", S(&el.name)),
                        ("parentId", U(id)),
                        ("artboardId", U(artboard)),
                        ("instanceWidth", F(u.size.0)),
                        ("instanceHeight", F(u.size.1)),
                    ],
                );
                self.names.insert(el.name.clone(), slot);
                self.add(
                    "LiveSlot",
                    &[
                        ("parentId", U(slot)),
                        ("unitKey", U(key)),
                        ("role", U(1)),
                        ("flightKey", U(flight)),
                        ("sizeMode", U(mode)),
                    ],
                );
            }
            PaintKind::Group => {
                self.add("Node", &[("name", S(&el.name)), ("parentId", U(parent))]);
                for kid in &el.kids {
                    self.emit(kid, id);
                }
            }
            PaintKind::Translated => {
                self.add(
                    "LayoutComponent",
                    &[
                        ("name", S(&el.name)),
                        ("parentId", U(parent)),
                        ("styleId", U(id + 1)),
                        ("clip", B(false)),
                    ],
                );
                let mut style = vec![
                    ("parentId", U(id)),
                    ("layoutWidthScaleType", U(2)),
                    ("layoutHeightScaleType", U(2)),
                ];
                style.extend_from_slice(&el.css);
                self.add("LayoutComponentStyle", &style);
                for kid in &el.kids {
                    self.emit(kid, id);
                }
            }
            PaintKind::List => {
                self.add(
                    "ArtboardComponentList",
                    &[("name", S(&el.name)), ("parentId", U(parent))],
                );
                bind(self.b, 800, &[PAINT_B_VM as u8, self.numbers.len() as u8]);
            }
        }
    }
}

/// An artboard's "entrance" timeline, one second from load, once through:
/// `tracks`, on objects `names` knows. The machine after it plays it.
fn paint_entrance(
    b: &mut Vec<u8>,
    tracks: &[PaintTrack],
    names: &std::collections::HashMap<String, u64>,
) {
    use V::*;
    obj(
        b,
        "LinearAnimation",
        &[
            ("name", S("entrance")),
            ("fps", U(60)),
            ("duration", U(60)),
            ("loopValue", U(0)),
        ],
    );
    let mut targets: Vec<&str> = Vec::new();
    for t in tracks {
        if !targets.contains(&t.target.as_str()) {
            targets.push(&t.target);
        }
    }
    for target in targets {
        let id = *names
            .get(target)
            .unwrap_or_else(|| panic!("track target {target}"));
        obj(b, "KeyedObject", &[("objectId", U(id))]);
        for t in tracks.iter().filter(|t| t.target == target) {
            obj(b, "KeyedProperty", &[("propertyKey", U(t.key))]);
            for &(frame, value) in &t.frames {
                let interpolation = U(if t.hold { 0 } else { 1 });
                if t.int {
                    obj(
                        b,
                        "KeyFrameInt",
                        &[
                            ("frame", U(frame)),
                            ("value", I(value as i64)),
                            ("interpolationType", interpolation),
                        ],
                    );
                } else {
                    obj(
                        b,
                        "KeyFrameDouble",
                        &[
                            ("frame", U(frame)),
                            ("value", F(value)),
                            ("interpolationType", interpolation),
                        ],
                    );
                }
            }
        }
    }
}

/// A Screen artboard: its root (0), root record (1) and style (2), then its
/// elements. The upper Screen also gets its timeline, machine and listener.
fn paint_screen(b: &mut Vec<u8>, spec: &PaintSpec, upper: bool) {
    use V::*;
    let (name, model, els) = if upper {
        ("B", PAINT_B_VM, &spec.b)
    } else {
        ("A", PAINT_A_VM, &spec.a)
    };
    let mut props = vec![
        ("name", S(name)),
        ("viewModelId", U(model)),
        ("width", F(spec.size.0)),
        ("height", F(spec.size.1)),
        ("styleId", U(2)),
        ("clip", B(false)),
    ];
    if upper {
        props.push(("defaultStateMachineId", U(0)));
    }
    obj(b, "Artboard", &props); // 0
    let root_grid: &[f32] = if upper { &spec.root_grid } else { &[] };
    let mut root_css = vec![("parentId", U(0))];
    if !root_grid.is_empty() {
        root_css.push(("display", U(2)));
    }
    obj(b, "LayoutCssStyle", &root_css); // 1
    obj(b, "LayoutComponentStyle", &[("parentId", U(0))]); // 2
    let mut art = PaintArt {
        b: &mut *b,
        units: &spec.units,
        numbers: &spec.numbers,
        next: 3,
        names: Default::default(),
        masks: Vec::new(),
    };
    for (n, &size) in root_grid.iter().enumerate() {
        let track = art.add(
            "GridTrack",
            &[
                ("parentId", U(0)),
                ("collection", U(0)),
                ("trackType", U(1)),
                ("trackValue", F(size)),
            ],
        );
        art.names.insert(format!("root.track{n}"), track);
    }
    for el in els {
        art.emit(el, 0);
    }
    for (masked, source) in std::mem::take(&mut art.masks) {
        let source = art.names[source];
        art.add(
            "LayerMask",
            &[("parentId", U(masked)), ("sourceId", U(source))],
        );
    }
    let names = art.names;
    if !upper {
        return;
    }
    let animated = !spec.tracks.is_empty();
    if animated {
        paint_entrance(b, &spec.tracks, &names);
    }
    machine(b, animated);
    if let Some(target) = spec.listener {
        hit_listener(b, names[target]);
    }
}

/// A file at the format version the compiler writes, 7.5: from 7.3 a CSS
/// box draws its own x, y, turn and scale (MP `layout_component.rs:1039`).
pub(crate) fn paint_header() -> Vec<u8> {
    let mut b = b"RIVE".to_vec();
    for n in [7, 5, 0, 0] {
        var(&mut b, n);
    }
    b
}

fn paint_file(spec: &PaintSpec) -> Vec<u8> {
    use V::*;
    let mut b = paint_header();
    obj(&mut b, "Backboard", &[]);
    obj(&mut b, "ViewModel", &[("name", S("Unit"))]);
    obj(&mut b, "ViewModelPropertyNumber", &[("name", S("scale"))]);
    obj(
        &mut b,
        "ViewModelInstance",
        &[("viewModelId", U(PAINT_UNIT_VM))],
    );
    obj(
        &mut b,
        "ViewModelInstanceNumber",
        &[("viewModelPropertyId", U(0)), ("propertyValue", F(2.0))],
    );
    obj(&mut b, "ViewModel", &[("name", S("A"))]);
    obj(
        &mut b,
        "ViewModelInstance",
        &[("viewModelId", U(PAINT_A_VM))],
    );
    obj(&mut b, "ViewModel", &[("name", S("B"))]);
    for (name, _) in &spec.numbers {
        obj(&mut b, "ViewModelPropertyNumber", &[("name", S(name))]);
    }
    if spec.list {
        obj(&mut b, "ViewModelPropertyList", &[("name", S("rows"))]);
    }
    obj(
        &mut b,
        "ViewModelInstance",
        &[("viewModelId", U(PAINT_B_VM))],
    );
    for (i, (_, value)) in spec.numbers.iter().enumerate() {
        obj(
            &mut b,
            "ViewModelInstanceNumber",
            &[
                ("viewModelPropertyId", U(i as u64)),
                ("propertyValue", F(*value)),
            ],
        );
    }
    if spec.list {
        obj(
            &mut b,
            "ViewModelInstanceList",
            &[("viewModelPropertyId", U(spec.numbers.len() as u64))],
        );
        obj(
            &mut b,
            "ViewModelInstanceListItem",
            &[
                ("viewModelId", U(PAINT_UNIT_VM)),
                ("viewModelInstanceId", U(0)),
            ],
        );
    }
    obj(&mut b, "ViewModel", &[("name", S("Page"))]);
    obj(&mut b, "ViewModelPropertyList", &[("name", S("screens"))]);
    obj(
        &mut b,
        "ViewModelInstance",
        &[("viewModelId", U(PAINT_PAGE_VM))],
    );
    obj(
        &mut b,
        "ViewModelInstanceList",
        &[("viewModelPropertyId", U(0))],
    );
    for model in [PAINT_A_VM, PAINT_B_VM] {
        obj(
            &mut b,
            "ViewModelInstanceListItem",
            &[("viewModelId", U(model)), ("viewModelInstanceId", U(0))],
        );
    }
    paint_screen(&mut b, spec, false);
    paint_screen(&mut b, spec, true);
    for u in &spec.units {
        paint_unit(&mut b, u);
    }
    obj(
        &mut b,
        "Artboard",
        &[
            ("name", S("Page")),
            ("viewModelId", U(PAINT_PAGE_VM)),
            ("width", F(spec.size.0)),
            ("height", F(spec.size.1)),
            ("clip", B(false)),
            ("defaultStateMachineId", U(0)),
        ],
    ); // 0
    obj(
        &mut b,
        "ArtboardComponentList",
        &[("name", S("screens")), ("parentId", U(0))],
    ); // 1
    bind(&mut b, 800, &[PAINT_PAGE_VM as u8, 0]);
    let mut next = 2;
    for (name, from, to, units) in &spec.steps {
        obj(
            &mut b,
            "LiveStep",
            &[
                ("parentId", U(0)),
                ("name", S(name)),
                ("fromArtboard", S(from)),
                ("toArtboard", S(to)),
            ],
        );
        let step = next;
        next += 1;
        for [incoming, outgoing, incoming_flight, outgoing_flight] in units {
            obj(
                &mut b,
                "LiveStepUnit",
                &[
                    ("parentId", U(step)),
                    ("incomingUnitKey", U(*incoming)),
                    ("outgoingUnitKey", U(*outgoing)),
                    ("incomingFlightKey", U(*incoming_flight)),
                    ("outgoingFlightKey", U(*outgoing_flight)),
                ],
            );
            next += 1;
        }
    }
    machine(&mut b, false);
    b
}

/// The page for one paint order case (UNIV-5635). The geometry of each is
/// stated where the test reads it, in `tests/live_moves.rs`.
pub fn css_paint_fixture(case: PaintCase) -> Vec<u8> {
    paint_file(&paint_spec(case))
}

fn paint_spec(case: PaintCase) -> PaintSpec {
    use V::*;
    use std::f32::consts::FRAC_PI_4;
    match case {
        // Back in flow, then `#blur` (absolute, full screen, keyed opacity),
        // then `#page` (absolute, keyed layout) holding `#card0`, then a
        // paintless `.catch`.
        PaintCase::AppStoreCard => PaintSpec::new(
            vec![unit("back", 40.0, 40.0), unit("card0", 160.0, 100.0)],
            vec![
                at_rest("back", 1, 10.0, 10.0, 40.0, 40.0),
                at_rest("card0", 2, 30.0, 70.0, 160.0, 100.0),
            ],
            vec![
                rest("back", 1, css_flow(40.0, 40.0, 10.0, 10.0)),
                bx("blur", css_at(0.0, 0.0, 400.0, 300.0)).props(&[("opacity", F(0.1))]),
                bx("page", css_at(20.0, 60.0, 360.0, 220.0)).kids(vec![rest(
                    "card0",
                    2,
                    css_flow(160.0, 100.0, 10.0, 10.0),
                )]),
                bx("catch", css_at(0.0, 0.0, 400.0, 300.0)).paintless(),
                fly("back", 1, 11),
                fly("card0", 2, 12),
            ],
        )
        .one_version(&[(1, 11), (2, 12)])
        .tracks(vec![
            track("blur", 18, &[(0, 0.1), (60, 1.0)]),
            track("page.css", 61061, &[(0, 40.0), (60, 20.0)]),
            track("page.css", 61052, &[(0, 80.0), (60, 60.0)]),
            track("page.css", 61022, &[(0, 320.0), (60, 360.0)]),
            track("page.css", 61025, &[(0, 180.0), (60, 220.0)]),
        ]),
        // `.catch`, the red picture, Back, the bar, and `#panel`, opaque to
        // the pointer, which slides up from 150 below its place. With
        // `red_above`, the red comes after the panel.
        PaintCase::Match { red_above } => {
            let red = at_rest("red", 1, 0.0, 0.0, 400.0, 200.0);
            let mut b = vec![bx("catch", css_at(0.0, 0.0, 400.0, 300.0)).paintless()];
            if !red_above {
                b.push(red.clone());
            }
            b.extend([
                at_rest("back", 2, 10.0, 10.0, 40.0, 40.0),
                at_rest("bar", 3, 0.0, 0.0, 400.0, 50.0),
                bx("panel", css_at(0.0, 150.0, 400.0, 150.0))
                    .props(&[("y", F(150.0)), ("drawableFlags", U(PAINT_OPAQUE))]),
            ]);
            if red_above {
                b.push(red);
            }
            b.extend([fly("red", 1, 11), fly("back", 2, 12), fly("bar", 3, 13)]);
            let mut spec = PaintSpec::new(
                vec![
                    unit("red", 400.0, 200.0).tappable(),
                    unit("back", 40.0, 40.0),
                    unit("bar", 400.0, 50.0),
                ],
                vec![
                    at_rest("red", 1, 100.0, 100.0, 200.0, 120.0),
                    at_rest("back", 2, 10.0, 250.0, 40.0, 40.0),
                    at_rest("bar", 3, 0.0, 250.0, 400.0, 50.0),
                ],
                b,
            )
            .one_version(&[(1, 11), (2, 12), (3, 13)])
            .tracks(vec![track("panel", 14, &[(0, 150.0), (60, 0.0)])]);
            spec.listener = Some("panel");
            spec
        }
        // `#lake` and `#scrim` absolute, a relative flex header holding the
        // chevron and the title, then the orb and the card.
        PaintCase::Calmly => PaintSpec::new(
            vec![
                unit("lake", 400.0, 300.0),
                unit("title", 200.0, 30.0),
                unit("orb", 80.0, 80.0),
                unit("card", 300.0, 80.0),
            ],
            vec![
                at_rest("lake", 1, 0.0, 0.0, 400.0, 300.0),
                at_rest("title", 2, 50.0, 10.0, 200.0, 30.0),
                at_rest("orb", 3, 160.0, 100.0, 80.0, 80.0),
                at_rest("card", 4, 50.0, 200.0, 300.0, 80.0),
            ],
            vec![
                at_rest("lake", 1, 0.0, 0.0, 400.0, 300.0),
                bx("scrim", css_at(0.0, 0.0, 400.0, 300.0)).props(&[("opacity", F(0.2))]),
                bx(
                    "header",
                    css_rel(css_with(
                        css_flow(400.0, 50.0, 0.0, 0.0),
                        &[("display", U(1))],
                    )),
                )
                .paintless()
                .kids(vec![
                    bx("chevron", css_flow(30.0, 30.0, 10.0, 10.0)),
                    rest("title", 2, css_flow(200.0, 30.0, 10.0, 10.0)),
                ]),
                at_rest("orb", 3, 160.0, 100.0, 80.0, 80.0),
                at_rest("card", 4, 50.0, 200.0, 300.0, 80.0),
                fly("lake", 1, 11),
                fly("title", 2, 12),
                fly("orb", 3, 13),
                fly("card", 4, 14),
            ],
        )
        .one_version(&[(1, 11), (2, 12), (3, 13), (4, 14)])
        .tracks(vec![track("scrim", 18, &[(0, 0.2), (60, 0.6)])]),
        // A list of three flex rows (picture, title, detail) and a grid of
        // three cells (picture over title). The pictures touch the next row
        // or cell and never overlap it. The upper Screen's rows or cells
        // slide in from 100 to the left and fade in, both held until the
        // half.
        PaintCase::ListToGrid { grid_above } => {
            const PICS: [&str; 3] = ["pic1", "pic2", "pic3"];
            let list: Vec<PaintEl> = (1..=3)
                .map(|n| {
                    bx(
                        &format!("row{n}"),
                        css_with(css_size(400.0, 100.0), &[("display", U(1))]),
                    )
                    .kids(vec![
                        rest(PICS[n - 1], n as u64, css_size(80.0, 100.0)),
                        bx(&format!("title{n}"), css_size(200.0, 40.0)),
                        bx(&format!("detail{n}"), css_size(100.0, 40.0)),
                    ])
                })
                .collect();
            let grid = bx(
                "grid",
                css_with(css_size(300.0, 120.0), &[("display", U(1))]),
            )
            .paintless()
            .kids(
                (1..=3)
                    .map(|n| {
                        bx(&format!("cell{n}"), css_size(100.0, 120.0)).kids(vec![
                            rest(PICS[n - 1], n as u64, css_size(100.0, 80.0)),
                            bx(&format!("ctitle{n}"), css_size(100.0, 40.0)),
                        ])
                    })
                    .collect(),
            );
            let (a, mut b) = if grid_above {
                (list, vec![grid])
            } else {
                (vec![grid], list)
            };
            for n in 1..=3 {
                b.push(fly(PICS[n - 1], n as u64, 10 + n as u64));
            }
            let mut tracks = Vec::new();
            for n in 1..=3 {
                let target = if grid_above {
                    format!("cell{n}")
                } else {
                    format!("row{n}")
                };
                tracks.push(hold(&target, 13, &[(0, -100.0), (30, 0.0)]));
                tracks.push(hold(&target, 18, &[(0, 0.5), (30, 1.0)]));
            }
            PaintSpec::new(
                PICS.iter().map(|&name| unit(name, 80.0, 100.0)).collect(),
                a,
                b,
            )
            .one_version(&[(1, 11), (2, 12), (3, 13)])
            .tracks(tracks)
        }
        // An absolute dim first, the in-flow title and cards, a relative
        // `.plan-slot` holding Yearly and the absolute ribbon (keyed opacity
        // and scale), Continue in the flow, then the absolute deal.
        PaintCase::Catzy => {
            let ribbon = css_with(
                css_size(80.0, 24.0),
                &[
                    ("position", U(2)),
                    ("insetTopKind", U(1)),
                    ("insetTop", F(-10.0)),
                    ("insetRightKind", U(1)),
                    ("insetRight", F(10.0)),
                ],
            );
            PaintSpec::new(
                vec![
                    unit("dim", 400.0, 300.0),
                    unit("title", 200.0, 30.0),
                    unit("card1", 360.0, 60.0),
                    unit("card2", 360.0, 60.0),
                    unit("yearly", 360.0, 60.0),
                    unit("cont", 360.0, 50.0),
                    unit("deal", 360.0, 30.0),
                ],
                vec![
                    at_rest("dim", 1, 0.0, 0.0, 400.0, 300.0),
                    at_rest("title", 2, 20.0, 0.0, 200.0, 30.0),
                    at_rest("card1", 3, 0.0, 30.0, 360.0, 60.0),
                    at_rest("card2", 4, 0.0, 90.0, 360.0, 60.0),
                    at_rest("yearly", 5, 0.0, 150.0, 360.0, 60.0),
                    at_rest("cont", 6, 0.0, 210.0, 360.0, 50.0),
                    at_rest("deal", 7, 20.0, 260.0, 360.0, 30.0),
                ],
                vec![
                    at_rest("dim", 1, 0.0, 0.0, 400.0, 300.0),
                    rest("title", 2, css_flow(200.0, 30.0, 0.0, 20.0)),
                    rest("card1", 3, css_size(360.0, 60.0)),
                    rest("card2", 4, css_size(360.0, 60.0)),
                    bx("plan", css_rel(css_size(360.0, 60.0)))
                        .paintless()
                        .kids(vec![
                            rest("yearly", 5, css_size(360.0, 60.0)),
                            bx("ribbon", ribbon).props(&[
                                ("opacity", F(0.3)),
                                ("scaleX", F(0.5)),
                                ("scaleY", F(0.5)),
                            ]),
                        ]),
                    rest("cont", 6, css_size(360.0, 50.0)),
                    at_rest("deal", 7, 20.0, 260.0, 360.0, 30.0),
                    fly("dim", 1, 11),
                    fly("title", 2, 12),
                    fly("card1", 3, 13),
                    fly("card2", 4, 14),
                    fly("yearly", 5, 15),
                    fly("cont", 6, 16),
                    fly("deal", 7, 17),
                ],
            )
            .one_version(&[
                (1, 11),
                (2, 12),
                (3, 13),
                (4, 14),
                (5, 15),
                (6, 16),
                (7, 17),
            ])
            .tracks(vec![
                track("ribbon", 18, &[(0, 0.3), (60, 1.0)]),
                track("ribbon", 16, &[(0, 0.5), (60, 1.0)]),
                track("ribbon", 17, &[(0, 0.5), (60, 1.0)]),
            ])
        }
        // E in an `overflow: hidden` holder H, a badge after E in H covering
        // it, and a box C after H covering E.
        PaintCase::ClippingHolder => PaintSpec::new(
            vec![unit("E", 100.0, 60.0)],
            vec![at_rest("E", 1, 20.0, 20.0, 100.0, 60.0)],
            vec![
                bx(
                    "H",
                    css_with(
                        css_size(300.0, 150.0),
                        &[("overflowX", U(2)), ("overflowY", U(2))],
                    ),
                )
                .kids(vec![
                    rest("E", 1, css_flow(100.0, 60.0, 20.0, 20.0)),
                    bx("badge", css_flow(40.0, 40.0, -40.0, 60.0)),
                ]),
                bx("C", css_flow(200.0, 100.0, -100.0, 0.0)),
                fly("E", 1, 11),
            ],
        )
        .one_version(&[(1, 11)]),
        // An absolute badge in a static, non-clipping card covers E there.
        PaintCase::BadgeInCard => PaintSpec::new(
            vec![unit("E", 100.0, 60.0)],
            vec![at_rest("E", 1, 20.0, 20.0, 100.0, 60.0)],
            vec![
                bx("card", css_size(300.0, 150.0)).kids(vec![
                    rest("E", 1, css_flow(100.0, 60.0, 20.0, 20.0)),
                    bx("badge", css_at(90.0, 30.0, 40.0, 40.0)),
                ]),
                fly("E", 1, 11),
            ],
        )
        .one_version(&[(1, 11)]),
        // A z-index 2 overlay O in an auto-z absolute wrapper W before E,
        // which is relative with z-index 1.
        PaintCase::OverlayInWrapper => PaintSpec::new(
            vec![unit("E", 100.0, 60.0)],
            vec![at_rest("E", 1, 20.0, 20.0, 100.0, 60.0)],
            vec![
                bx("W", css_at(0.0, 0.0, 400.0, 300.0))
                    .paintless()
                    .kids(vec![bx("O", css_z(css_at(60.0, 30.0, 100.0, 60.0), 2.0))]),
                rest(
                    "E",
                    1,
                    css_z(css_rel(css_flow(100.0, 60.0, 20.0, 20.0)), 1.0),
                ),
                fly("E", 1, 11),
            ],
        )
        .one_version(&[(1, 11)]),
        // A relative E, then a static block wrapper W holding a static
        // z-index 2 child K that reaches 40 up into E. The test writes W's
        // display to flex mid-flight.
        PaintCase::DisplayWrite => PaintSpec::new(
            vec![unit("E", 100.0, 60.0)],
            vec![at_rest("E", 1, 36.0, 20.0, 100.0, 60.0)],
            vec![
                rest("E", 1, css_rel(css_flow(100.0, 60.0, 20.0, 36.0))),
                bx("W", css_size(300.0, 150.0)).paintless().kids(vec![bx(
                    "K",
                    css_z(css_flow(200.0, 100.0, -40.0, 0.0), 2.0),
                )]),
                fly("E", 1, 11),
            ],
        )
        .one_version(&[(1, 11)]),
        // E1 in flow, E2 relative and E3 relative with z-index 2, each 40
        // up into the one before; their flight slots in reverse order.
        PaintCase::ThreeAnchors => PaintSpec::new(
            vec![
                unit("E1", 100.0, 60.0),
                unit("E2", 100.0, 60.0),
                unit("E3", 100.0, 60.0),
            ],
            vec![
                at_rest("E1", 1, 20.0, 20.0, 100.0, 60.0),
                at_rest("E2", 2, 60.0, 40.0, 100.0, 60.0),
                at_rest("E3", 3, 100.0, 60.0, 100.0, 60.0),
            ],
            vec![
                rest("E1", 1, css_flow(100.0, 60.0, 20.0, 20.0)),
                rest("E2", 2, css_rel(css_flow(100.0, 60.0, -40.0, 60.0))),
                rest(
                    "E3",
                    3,
                    css_z(css_rel(css_flow(100.0, 60.0, -40.0, 100.0)), 2.0),
                ),
                fly("E3", 3, 13),
                fly("E2", 2, 12),
                fly("E1", 1, 11),
            ],
        )
        .one_version(&[(1, 11), (2, 12), (3, 13)]),
        PaintCase::Resting(n) => {
            let square = unit("E", 100.0, 100.0);
            let (e, mut a_e, mut b_e, mut flight) = match n {
                6 => (
                    square.scaled(),
                    at_rest("E", 1, 100.0, 100.0, 100.0, 100.0),
                    at_rest("E", 1, 100.0, 100.0, 100.0, 100.0),
                    fly("E", 1, 11),
                ),
                7 => (
                    unit("E", 100.0, 60.0).turned(0.35),
                    at_rest("E", 1, 100.0, 100.0, 100.0, 60.0),
                    at_rest("E", 1, 100.0, 100.0, 100.0, 60.0),
                    fly("E", 1, 11),
                ),
                9..=11 => (
                    unit("E", 100.0, 60.0).scaled(),
                    at_rest(
                        "E",
                        1,
                        0.0,
                        100.0,
                        if n == 11 { 100.0 } else { 200.0 },
                        60.0,
                    ),
                    at_rest("E", 1, 0.0, 100.0, 100.0, 60.0),
                    flight_slot("E flight", "E", 1, 11, if n == 10 { 1 } else { 0 }),
                ),
                _ => (
                    square,
                    at_rest("E", 1, 100.0, 100.0, 100.0, 100.0),
                    at_rest("E", 1, 100.0, 100.0, 100.0, 100.0),
                    fly("E", 1, 11),
                ),
            };
            if n == 11 {
                a_e = a_e.origin(0.25);
                b_e = b_e.origin(0.5);
                flight = flight.origin(1.0);
            }
            let mut units = vec![e];
            let mut numbers = Vec::new();
            let mut tracks = Vec::new();
            let cover = match n {
                1 => bx("C", css_at(150.0, 150.0, 100.0, 100.0)),
                2 => bx("C", css_at(150.0, 260.0, 100.0, 100.0)).props(&[("y", F(-110.0))]),
                3 => {
                    tracks.push(track("C", 13, &[(0, 250.0), (60, 0.0)]));
                    bx("C", css_at(150.0, 150.0, 100.0, 100.0))
                }
                4 => {
                    numbers.push(("turn", FRAC_PI_4));
                    bx("C", css_at(200.0, 100.0, 100.0, 100.0)).bind(15, "turn")
                }
                5 => {
                    tracks.push(track("C", 15, &[(0, 0.0), (30, FRAC_PI_4), (60, 0.0)]));
                    bx("C", css_at(200.0, 100.0, 100.0, 100.0))
                }
                6 => bx("C", css_at(220.0, 140.0, 10.0, 20.0)),
                7 => bx("C", css_at(202.0, 116.0, 4.0, 6.0)),
                8 => {
                    units.push(unit("D", 100.0, 100.0).scaled());
                    rest("D", 0, css_at(220.0, 100.0, 100.0, 100.0))
                }
                _ => bx("C", css_at(120.0, 110.0, 10.0, 20.0)),
            };
            PaintSpec::new(units, vec![a_e], vec![b_e, cover, flight])
                .one_version(&[(1, 11)])
                .tracks(tracks)
                .numbers(&numbers)
        }
        // Keyed layout inside a hosted frame or a list row, on a timeline of
        // its own. H and N are relative, each after an auto-high slot whose
        // frame holds the keyed S or pads itself; L is relative, after a list
        // whose row's own height is keyed. U is what follows an in-flow E
        // whose own frame keys S.
        PaintCase::KeyedLayout(which @ ('H' | 'L' | 'N' | 'U')) => {
            let name = which.to_string();
            let e = at_rest("E", 1, 100.0, 100.0, 100.0, 60.0);
            let after = bx(&name, css_rel(css_size(300.0, 60.0)));
            let host = rest_as(
                "host",
                "hosted",
                0,
                vec![("widthKind", U(1)), ("width", F(300.0))],
            );
            let hosted = |keyed| unit("hosted", 300.0, 0.0).keyed(keyed);
            let (first, e, boxes) = match which {
                'H' => (hosted(UnitKey::Child(100.0)), e, vec![host, after]),
                'N' => (hosted(UnitKey::Padding(100.0)), e, vec![host, after]),
                'L' => (
                    unit("row", 300.0, 0.0).keyed(UnitKey::RowHeight(100.0)),
                    e,
                    vec![PaintEl::new("rows", PaintKind::List, Vec::new()), after],
                ),
                // E is in flow at the top, and U is clear of it, under it.
                _ => (
                    unit("E", 100.0, 60.0).keyed(UnitKey::Child(100.0)),
                    rest("E", 1, css_flow(100.0, 60.0, 0.0, 100.0)),
                    vec![bx(&name, css_rel(css_flow(300.0, 60.0, 40.0, 0.0)))],
                ),
            };
            let mut units = vec![first];
            if which != 'U' {
                units.push(unit("E", 100.0, 60.0));
            }
            let mut b = vec![e.clone()];
            b.extend(boxes);
            b.push(fly("E", 1, 11));
            let mut spec = PaintSpec::new(units, vec![e], b).one_version(&[(1, 11)]);
            spec.list = which == 'L';
            spec
        }
        PaintCase::KeyedLayout(which) => {
            let e = at_rest("E", 1, 100.0, 100.0, 100.0, 60.0);
            let grid = |tracks: &[f32]| {
                bx(
                    "grid",
                    css_rel(css_with(
                        css_flow(400.0, 60.0, 100.0, 0.0),
                        &[("display", U(2))],
                    )),
                )
                .paintless()
                .tracks(tracks)
            };
            // S grows from 0 to `to` high, inside a box or beside one.
            let grower = || bx("S", css_size(300.0, 0.0)).paintless();
            let grows = |to: f32| track("S.css", 61025, &[(0, 0.0), (60, to)]);
            // A translated box, with no record, that hugs S.
            let translated = |style: Css| {
                PaintEl::new("wrapper", PaintKind::Translated, style).kids(vec![grower()])
            };
            // A box 300 wide whose height comes from what it holds.
            let auto_high = |name: &str, more: &[(&'static str, V<'static>)]| {
                bx(
                    name,
                    css_with(vec![("widthKind", U(1)), ("width", F(300.0))], more),
                )
            };
            // An absolute box 300 by 60, at the left, with no top.
            let at_left = |name: &str, more: &[(&'static str, V<'static>)]| {
                bx(
                    name,
                    css_with(
                        css_with(
                            css_size(300.0, 60.0),
                            &[
                                ("position", U(2)),
                                ("insetLeftKind", U(1)),
                                ("insetLeft", F(0.0)),
                            ],
                        ),
                        more,
                    ),
                )
            };
            let (boxes, keyed) = match which {
                // C grows from 0 to 200 wide, reaching E (100 to 200) past
                // half way.
                'C' => (
                    vec![bx("C", css_at(0.0, 100.0, 0.0, 60.0))],
                    track("C.css", 61022, &[(0, 0.0), (60, 200.0)]),
                ),
                // S grows from 0 to 100 high and pushes the relative D down
                // onto E (100 to 160) from 40.
                'D' => (
                    vec![
                        bx("S", css_size(300.0, 0.0)).paintless(),
                        bx("D", css_rel(css_size(300.0, 60.0))),
                    ],
                    track("S.css", 61025, &[(0, 0.0), (60, 100.0)]),
                ),
                // S, in an auto-high wrapper, pushes the relative W down onto
                // E (100 to 160) from 40.
                'W' => (
                    vec![
                        auto_high("wrapper", &[]).paintless().kids(vec![grower()]),
                        bx("W", css_rel(css_size(300.0, 60.0))),
                    ],
                    grows(100.0),
                ),
                // S, inside a plain node, pushes the relative O down onto E
                // (100 to 160) from 40.
                'O' => (
                    vec![
                        PaintEl::new("group", PaintKind::Group, Vec::new()).kids(vec![grower()]),
                        bx("O", css_rel(css_size(300.0, 60.0))),
                    ],
                    grows(100.0),
                ),
                // S, inside a translated wrapper that hugs it, pushes the
                // relative X down onto E (100 to 160) from 40.
                'X' => (
                    vec![
                        translated(Vec::new()),
                        bx("X", css_rel(css_size(300.0, 60.0))),
                    ],
                    grows(100.0),
                ),
                // Y, relative, stays clear of E. The translated wrapper that
                // holds the keyed S is absolute, so it pushes nothing.
                'Y' => (
                    vec![
                        translated(vec![("positionTypeValue", U(2))]),
                        bx("Y", css_rel(css_size(300.0, 60.0))),
                    ],
                    grows(100.0),
                ),
                // F, relative and painted, grows with the S inside it, past
                // E's top (100) after half way.
                'F' => (
                    vec![auto_high("F", &[("position", U(1))]).kids(vec![grower()])],
                    grows(200.0),
                ),
                // K, 60 high, sits on the bottom edge of a relative wrapper
                // that grows with the S inside it, past E's top (100) after
                // half way.
                'K' => (
                    vec![
                        auto_high("wrapper", &[("position", U(1))])
                            .paintless()
                            .kids(vec![
                                grower(),
                                at_left("K", &[("insetBottomKind", U(1)), ("insetBottom", F(0.0))]),
                            ]),
                    ],
                    grows(200.0),
                ),
                // J moves from the second column to the first at the half,
                // under I, so the auto-high grid goes from one row of 30 to
                // two and pushes the relative V onto E (100 to 160).
                'V' => (
                    vec![
                        auto_high("grid", &[("display", U(2))])
                            .paintless()
                            .tracks(&[100.0, 100.0])
                            .kids(vec![
                                bx("I", css_size(100.0, 30.0)).paintless().column(1),
                                bx("J", css_size(100.0, 30.0)).paintless().column(2),
                            ]),
                        bx("V", css_rel(css_size(300.0, 60.0))),
                    ],
                    PaintTrack {
                        int: true,
                        ..hold("J.place", 1047, &[(0, 2.0), (30, 1.0)])
                    },
                ),
                // T is absolute with no top or bottom, so its static position
                // follows the auto-high wrapper before it onto E from 40.
                'T' => (
                    vec![
                        auto_high("wrapper", &[]).paintless().kids(vec![grower()]),
                        at_left("T", &[]),
                    ],
                    grows(100.0),
                ),
                // M, absolute with insets on both axes, stays clear of E
                // beside the keyed S.
                'M' => (
                    vec![grower(), bx("M", css_at(0.0, 0.0, 60.0, 60.0))],
                    grows(100.0),
                ),
                // Q, relative, stays clear of E. Only its paintless absolute
                // child is keyed.
                'Q' => (
                    vec![
                        bx("Q", css_rel(css_size(300.0, 60.0)))
                            .kids(vec![bx("keyed", css_at(0.0, 0.0, 0.0, 60.0)).paintless()]),
                    ],
                    track("keyed.css", 61022, &[(0, 0.0), (60, 100.0)]),
                ),
                // G fills a grid column whose track grows from 0 to 200.
                'G' => (
                    vec![grid(&[0.0]).kids(vec![bx(
                        "G",
                        vec![
                            ("widthKind", U(7)),
                            ("widthPercent", F(100.0)),
                            ("heightKind", U(1)),
                            ("height", F(60.0)),
                        ],
                    )])],
                    track("grid.track0", 1063, &[(0, 0.0), (60, 200.0)]),
                ),
                // R, relative and 100 down, fills a column of B's root grid
                // whose track grows from 0 to 200.
                'R' => (
                    vec![bx(
                        "R",
                        css_rel(vec![
                            ("widthKind", U(7)),
                            ("widthPercent", F(100.0)),
                            ("heightKind", U(1)),
                            ("height", F(60.0)),
                            ("marginTop", F(100.0)),
                        ]),
                    )],
                    track("root.track0", 1063, &[(0, 0.0), (60, 200.0)]),
                ),
                // P moves from the first 100 point column to the second, E's
                // place, at the half.
                _ => (
                    vec![
                        grid(&[100.0, 100.0]).kids(vec![bx("P", css_size(100.0, 60.0)).column(1)]),
                    ],
                    PaintTrack {
                        int: true,
                        ..hold("P.place", 1047, &[(0, 1.0), (30, 2.0)])
                    },
                ),
            };
            let mut b = vec![e.clone()];
            b.extend(boxes);
            b.push(fly("E", 1, 11));
            PaintSpec::new(vec![unit("E", 100.0, 60.0)], vec![e], b)
                .one_version(&[(1, 11)])
                .tracks(vec![keyed])
                .root_grid(if which == 'R' { &[0.0] } else { &[] })
        }
        // E in flow, then a relative C 40 up into it, its z-index 2 bound
        // to B's "z".
        PaintCase::BoundZIndex => PaintSpec::new(
            vec![unit("E", 100.0, 60.0)],
            vec![at_rest("E", 1, 20.0, 20.0, 100.0, 60.0)],
            vec![
                rest("E", 1, css_flow(100.0, 60.0, 20.0, 20.0)),
                bx("C", css_z(css_rel(css_flow(200.0, 100.0, -40.0, 0.0)), 2.0))
                    .css_bind(61100, "z"),
                fly("E", 1, 11),
            ],
        )
        .one_version(&[(1, 11)])
        .numbers(&[("z", 2.0)]),
        // Container R, block or a flex column, holding E and a static
        // z-index 2 child K 40 up into E.
        PaintCase::Hinge { flex } => PaintSpec::new(
            vec![unit("E", 100.0, 60.0)],
            vec![at_rest("E", 1, 20.0, 20.0, 100.0, 60.0)],
            vec![
                bx(
                    "R",
                    css_with(
                        css_size(400.0, 200.0),
                        &[
                            ("display", U(if flex { 1 } else { 0 })),
                            ("flexDirection", U(2)),
                        ],
                    ),
                )
                .paintless()
                .kids(vec![
                    rest("E", 1, css_flow(100.0, 60.0, 20.0, 20.0)),
                    bx("K", css_z(css_flow(200.0, 100.0, -40.0, 0.0), 2.0)),
                ]),
                fly("E", 1, 11),
            ],
        )
        .one_version(&[(1, 11)]),
        // E, whose place the test writes per size, and C, 80 by 60 at the
        // bottom right corner.
        PaintCase::Sizes => PaintSpec::new(
            vec![unit("E", 100.0, 60.0)],
            vec![at_rest("E", 1, 150.0, 120.0, 100.0, 60.0)],
            vec![
                at_rest("E", 1, 150.0, 120.0, 100.0, 60.0),
                bx(
                    "C",
                    css_with(
                        css_size(80.0, 60.0),
                        &[
                            ("position", U(2)),
                            ("insetRightKind", U(1)),
                            ("insetRight", F(0.0)),
                            ("insetBottomKind", U(1)),
                            ("insetBottom", F(0.0)),
                        ],
                    ),
                ),
                fly("E", 1, 11),
            ],
        )
        .one_version(&[(1, 11)]),
        // Two versions of E, Ea on A and Eb on B, and C over E. Each step
        // has its own flight slot per version. As the compiler writes a
        // Screen's flight layer, B's own version, Eb, comes later both ways.
        PaintCase::TwoVersions => {
            let mut spec = PaintSpec::new(
                vec![unit("Ea", 100.0, 60.0), unit("Eb", 100.0, 60.0)],
                vec![rest_as("E", "Ea", 1, css_at(100.0, 100.0, 100.0, 60.0))],
                vec![
                    rest_as("E", "Eb", 2, css_at(100.0, 100.0, 100.0, 60.0)),
                    bx("C", css_at(150.0, 120.0, 100.0, 60.0)),
                    flight_slot("Ea go", "Ea", 1, 11, 1),
                    flight_slot("Eb go", "Eb", 2, 12, 1),
                    flight_slot("Ea back", "Ea", 1, 14, 1),
                    flight_slot("Eb back", "Eb", 2, 13, 1),
                ],
            );
            spec.steps = vec![
                ("go", "A", "B", vec![[2, 1, 12, 11]]),
                ("back", "B", "A", vec![[1, 2, 14, 13]]),
            ];
            spec
        }
        // A static plain host at (50, 40) holding E's CSS frame, turned at
        // rest, and a later in-flow C over it.
        PaintCase::PlainAnchor => PaintSpec::new(
            vec![unit("E", 100.0, 60.0).turned(0.35)],
            vec![plain("E", 1, 50.0, 40.0)],
            vec![
                plain("E", 1, 50.0, 40.0),
                bx("C", css_size(200.0, 100.0)),
                fly("E", 1, 11),
            ],
        )
        .one_version(&[(1, 11)]),
        // E in flow at (36, 20), then C 30 up into it, as in
        // `css_two_screen_stacking_fixture`. E's frame is turned at rest,
        // plain, or at opacity 0.5; in the last it may fly in a native
        // layout flight slot.
        PaintCase::TurnedFrame | PaintCase::FadeWrite | PaintCase::FadeAtRest { .. } => {
            let e = match case {
                PaintCase::TurnedFrame => unit("E", 100.0, 60.0).turned(0.35),
                PaintCase::FadeWrite => unit("E", 100.0, 60.0),
                _ => unit("E", 100.0, 60.0).faded(0.5),
            };
            let flight = if case == (PaintCase::FadeAtRest { native: true }) {
                native_slot("E flight", "E", 1, 11, 1)
            } else {
                fly("E", 1, 11)
            };
            PaintSpec::new(
                vec![e],
                vec![at_rest("E", 1, 36.0, 20.0, 100.0, 60.0)],
                vec![
                    rest("E", 1, css_flow(100.0, 60.0, 20.0, 36.0)),
                    bx("C", css_flow(200.0, 100.0, -30.0, 0.0)),
                    flight,
                ],
            )
            .one_version(&[(1, 11)])
        }
        // E and C in a group M masked by S.
        PaintCase::MaskedGroup => PaintSpec::new(
            vec![unit("E", 100.0, 60.0)],
            vec![at_rest("E", 1, 20.0, 20.0, 100.0, 60.0)],
            vec![
                bx("M", css_size(300.0, 150.0)).mask("S").kids(vec![
                    rest("E", 1, css_flow(100.0, 60.0, 20.0, 20.0)),
                    bx("C", css_flow(200.0, 100.0, -40.0, 0.0)),
                ]),
                bx("S", css_at(0.0, 0.0, 300.0, 150.0)),
                fly("E", 1, 11),
            ],
        )
        .one_version(&[(1, 11)]),
        // Two versions of E, turned at rest, with C over B's; E2, which C
        // does not cover, earlier on B. "go" flies E; "back2" flies only E2
        // from B to A.
        PaintCase::Cancel => {
            let mut spec = PaintSpec::new(
                vec![
                    unit("Ea", 100.0, 60.0).turned(0.35),
                    unit("Eb", 100.0, 60.0).turned(0.35),
                    unit("E2", 60.0, 40.0),
                ],
                vec![
                    rest_as("E", "Ea", 1, css_at(150.0, 120.0, 100.0, 60.0)),
                    at_rest("E2", 3, 220.0, 160.0, 60.0, 40.0),
                ],
                vec![
                    at_rest("E2", 3, 20.0, 20.0, 60.0, 40.0),
                    rest_as("E", "Eb", 2, css_at(150.0, 120.0, 100.0, 60.0)),
                    bx("C", css_at(180.0, 140.0, 120.0, 80.0)),
                    flight_slot("Ea go", "Ea", 1, 11, 1),
                    flight_slot("Eb go", "Eb", 2, 12, 1),
                    flight_slot("E2 flight", "E2", 3, 13, 1),
                ],
            );
            spec.steps = vec![
                ("go", "A", "B", vec![[2, 1, 12, 11]]),
                ("back2", "B", "A", vec![[3, 3, 13, 13]]),
            ];
            spec
        }
    }
}

/// A CSS artboard "Plain" with no live-move records: a static plain
/// `NestedArtboard` "host" at (50, 40) with an empty record, holding unit
/// E's CSS frame turned 0.35 about its center, then an in-flow box C, 200
/// by 100, over it.
pub fn css_paint_plain_host_fixture() -> Vec<u8> {
    use V::*;
    let mut b = paint_header();
    obj(&mut b, "Backboard", &[]);
    obj(
        &mut b,
        "Artboard",
        &[
            ("name", S("Plain")),
            ("width", F(400.0)),
            ("height", F(300.0)),
            ("styleId", U(2)),
            ("clip", B(false)),
            ("defaultStateMachineId", U(0)),
        ],
    ); // 0
    obj(&mut b, "LayoutCssStyle", &[("parentId", U(0))]); // 1
    obj(&mut b, "LayoutComponentStyle", &[("parentId", U(0))]); // 2
    obj(
        &mut b,
        "NestedArtboard",
        &[
            ("name", S("host")),
            ("parentId", U(0)),
            ("artboardId", U(1)),
            ("x", F(50.0)),
            ("y", F(40.0)),
        ],
    ); // 3
    obj(&mut b, "LayoutCssStyle", &[("parentId", U(3))]); // 4
    obj(
        &mut b,
        "LayoutComponent",
        &[("name", S("C")), ("parentId", U(0)), ("styleId", U(6))],
    ); // 5
    obj(&mut b, "LayoutComponentStyle", &[("parentId", U(5))]); // 6
    let mut record = vec![("parentId", U(5))];
    record.extend(css_size(200.0, 100.0));
    obj(&mut b, "LayoutCssStyle", &record); // 7
    obj(&mut b, "Fill", &[("parentId", U(5))]); // 8
    obj(
        &mut b,
        "SolidColor",
        &[("parentId", U(8)), ("colorValue", Color(paint_color("C")))],
    ); // 9
    machine(&mut b, false);
    paint_unit(&mut b, &unit("E", 100.0, 60.0).turned(0.35));
    b
}
