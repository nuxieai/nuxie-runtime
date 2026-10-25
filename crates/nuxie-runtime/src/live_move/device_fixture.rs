//! Literal device-shaped page. Shared only behind test-support.
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
fn text_listeners(b: &mut Vec<u8>, target: u64) {
    use V::*;
    // Publication emits both listeners on the TextInput endpoint. FocusData alone
    // can focus but advertises no keyboard input without these machine listeners.
    for kind in [8, 15] {
        obj(
            b,
            "StateMachineListenerSingle",
            &[("targetId", U(target)), ("listenerTypeValue", U(kind))],
        );
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
// Published lifecycle guards bind only machine comparands. Flight properties are
// ordinary animation tracks and never take their values from a view model.
fn lifecycle_condition(b: &mut Vec<u8>, model: u8, phase: bool, op: u64, step: &str) {
    use V::*;
    obj(b, "BindablePropertyString", &[]);
    bind(
        b,
        key("BindablePropertyString", "propertyValue").into(),
        &[model, 1, if phase { 0 } else { 1 }],
    );
    obj(b, "TransitionViewModelCondition", &[("opValue", U(op))]);
    obj(b, "TransitionPropertyViewModelComparator", &[]);
    obj(
        b,
        "TransitionValueStringComparator",
        &[("value", S(if phase { "entering" } else { step }))],
    );
}
fn arrival_machine(b: &mut Vec<u8>, model: u8, step: &str, start_event: Option<u64>) {
    use V::*;
    machine(b, false);
    obj(
        b,
        "StateMachineLayer",
        &[("name", S(&format!("nx.transition.{step}.enter")))],
    );
    obj(b, "EntryState", &[]); // 0
    obj(b, "StateTransition", &[("stateToId", U(1))]);
    obj(b, "BlendStateDirect", &[]); // 1, contributes no idle pose
    obj(b, "StateTransition", &[("stateToId", U(2))]);
    lifecycle_condition(b, model, true, 0, step);
    lifecycle_condition(b, model, false, 0, step);
    obj(b, "AnimationState", &[("animationId", U(0))]); // 2, one-shot holds its last pose
    if let Some(event) = start_event {
        obj(b, "StateMachineFireEvent", &[("eventId", U(event))]);
    }
    obj(b, "StateTransition", &[("stateToId", U(1))]);
    lifecycle_condition(b, model, true, 1, step);
    obj(b, "StateTransition", &[("stateToId", U(1))]);
    lifecycle_condition(b, model, false, 1, step);
}
fn flight(b: &mut Vec<u8>, layer: u64, unit: u64, model: u8, flight_key: u64) {
    use V::*;
    obj(
        b,
        "Node",
        &[("name", S("flight")), ("parentId", U(0)), ("x", F(35.0))],
    );
    slot(b, layer, layer + 1, "mascot", unit, 0.0, model);
    obj(
        b,
        "LiveSlot",
        &[
            ("parentId", U(layer + 1)),
            ("unitKey", U(1)),
            ("role", U(1)),
            ("flightKey", U(flight_key)),
            ("sizeMode", U(1)),
        ],
    );
}
fn arrival_timeline(b: &mut Vec<u8>, step: &str, target: u64, event: u64) {
    use V::*;
    obj(
        b,
        "LinearAnimation",
        &[
            ("name", S(&format!("nx_enter_done:{step}"))),
            ("fps", U(60)),
            ("duration", U(60)),
            ("loopValue", U(0)),
        ],
    );
    obj(b, "KeyedObject", &[("objectId", U(target))]);
    obj(b, "KeyedProperty", &[("propertyKey", U(62003))]);
    for (frame, value) in [(0, 0.0), (60, 1.0)] {
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
    obj(b, "KeyedObject", &[("objectId", U(event))]);
    obj(b, "KeyedProperty", &[("propertyKey", U(395))]);
    obj(b, "KeyFrameCallback", &[("frame", U(60))]);
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
    for p in 0..4 {
        obj(
            b,
            "ViewModelInstanceNumber",
            &[
                ("parentId", U(host_id + 3)),
                ("viewModelPropertyId", U(p)),
                ("propertyValue", F(if p == 0 { 7.0 } else { 0.0 })),
            ],
        );
        if p == 1 {
            // This binding belongs to `prop`, not the later phase/scroll probes.
            bind(b, 575, &[screen_model, 0]);
        }
    }
}

pub fn fixture(rows: usize) -> Vec<u8> {
    build(rows, false).0
}
// The device exporter includes the authored lifecycle arrival layers.
pub fn device_fixture(rows: usize) -> Vec<u8> {
    build(rows, true).0
}
pub fn control_fixture() -> Vec<u8> {
    build(2, true).0
}
fn video_element(artboard: usize, owner: &str, view: &str, component: u64) -> serde_json::Value {
    serde_json::json!({
        "sourceArtboardIndex": artboard, "artboardId": owner,
        "viewNodeId": view, "renderedNodeId": view, "componentId": component,
        "readinessTimeoutSeconds": 5.0, "optional": false
    })
}
pub fn status_event_fixture() -> Vec<u8> {
    build_with_start_event(2, true).0
}
fn build(rows: usize, _controls: bool) -> (Vec<u8>, serde_json::Value) {
    build_with_start_event(rows, false)
}
pub fn status_animation_event_fixture() -> Vec<u8> {
    build_with_events(2, true, true).0
}
fn build_with_start_event(rows: usize, start_event: bool) -> (Vec<u8>, serde_json::Value) {
    build_with_events(rows, start_event, false)
}
fn build_with_events(
    rows: usize,
    start_event: bool,
    page_animation: bool,
) -> (Vec<u8>, serde_json::Value) {
    use V::*;
    assert!(rows >= 1);
    let mut b = b"RIVE".to_vec();
    let mut video_elements = Vec::new();
    for n in [7, 0, 0, 0] {
        var(&mut b, n);
    }
    obj(&mut b, "Backboard", &[]);
    obj(
        &mut b,
        "VideoAsset",
        &[
            // Published files use the same file-global ordinal here and on Video.assetId.
            ("assetId", U(0)),
            ("name", S("loop")),
            ("sourceKey", S("asset:mascot")),
            ("contentType", S("video/mp4")),
            ("width", F(96.0)),
            ("height", F(64.0)),
            ("duration", F(2.0)),
        ],
    );
    obj(
        &mut b,
        "FontAsset",
        &[("assetId", U(1)), ("name", S("Roboto"))],
    );
    obj(
        &mut b,
        "FileAssetContents",
        &[(
            "bytes",
            Bytes(include_bytes!(
                "../../../../fixtures/fonts/roboto-input.ttf"
            )),
        )],
    );
    obj(&mut b, "ViewModel", &[("name", S("Unit"))]);
    for name in ["count", "prop", "phase", "scroll"] {
        obj(&mut b, "ViewModelPropertyNumber", &[("name", S(name))]);
    }
    obj(&mut b, "ViewModelInstance", &[("viewModelId", U(0))]);
    for i in 0..4 {
        obj(
            &mut b,
            "ViewModelInstanceNumber",
            &[("viewModelPropertyId", U(i)), ("propertyValue", F(0.0))],
        );
    }
    for i in 0..rows {
        let name = format!("Screen{i}");
        obj(&mut b, "ViewModel", &[("name", S(&name))]);
        obj(&mut b, "ViewModelPropertyNumber", &[("name", S("offset"))]);
        obj(
            &mut b,
            "ViewModelPropertyViewModel",
            &[
                ("name", S("screen")),
                ("viewModelReferenceId", U(rows as u64 + 1)),
            ],
        );
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
        obj(
            &mut b,
            "ViewModelInstanceViewModel",
            &[
                ("viewModelPropertyId", U(1)),
                ("propertyValue", U(i as u64)),
            ],
        );
    }
    obj(&mut b, "ViewModel", &[("name", S("ScreenLifecycle"))]);
    obj(&mut b, "ViewModelPropertyString", &[("name", S("phase"))]);
    obj(
        &mut b,
        "ViewModelPropertyString",
        &[("name", S("transition"))],
    );
    obj(
        &mut b,
        "ViewModelPropertyNumber",
        &[("name", S("appearances"))],
    );
    for _ in 0..rows {
        obj(
            &mut b,
            "ViewModelInstance",
            &[("viewModelId", U(rows as u64 + 1))],
        );
        obj(
            &mut b,
            "ViewModelInstanceString",
            &[
                ("viewModelPropertyId", U(0)),
                ("propertyValue", S("hidden")),
            ],
        );
        obj(
            &mut b,
            "ViewModelInstanceString",
            &[("viewModelPropertyId", U(1)), ("propertyValue", S(""))],
        );
        obj(
            &mut b,
            "ViewModelInstanceNumber",
            &[("viewModelPropertyId", U(2)), ("propertyValue", F(0.0))],
        );
    }
    obj(&mut b, "ViewModel", &[("name", S("Page"))]);
    obj(&mut b, "ViewModelPropertyList", &[("name", S("screens"))]);
    obj(
        &mut b,
        "ViewModelInstance",
        &[("viewModelId", U(rows as u64 + 2))],
    );
    obj(
        &mut b,
        "ViewModelInstanceList",
        &[("viewModelPropertyId", U(0))],
    );
    for i in 0..rows {
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
            ("name", S("Page")),
            ("width", F(800.0)),
            ("height", F(600.0)),
            ("viewModelId", U(rows as u64 + 2)),
            ("clip", B(false)),
            ("defaultStateMachineId", U(0)),
        ],
    );
    obj(
        &mut b,
        "Node",
        &[("name", S("overlap")), ("parentId", U(0))],
    ); // 1
    obj(
        &mut b,
        "ArtboardComponentList",
        &[("name", S("screens")), ("parentId", U(1))],
    ); // 2
    bind(&mut b, 800, &[rows as u8 + 2, 0]);
    if rows > 1 {
        obj(
            &mut b,
            "LiveStep",
            &[
                ("parentId", U(0)),
                ("name", S("t1")),
                ("fromArtboard", S("A")),
                ("toArtboard", S("B")),
            ],
        );
        obj(
            &mut b,
            "LiveStepUnit",
            &[
                ("parentId", U(3)),
                ("incomingUnitKey", U(1)),
                ("outgoingUnitKey", U(1)),
                ("incomingFlightKey", U(1)),
                ("outgoingFlightKey", U(1)),
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
        );
        obj(
            &mut b,
            "LiveStepUnit",
            &[
                ("parentId", U(5)),
                ("incomingUnitKey", U(1)),
                ("outgoingUnitKey", U(1)),
                ("incomingFlightKey", U(2)),
                ("outgoingFlightKey", U(2)),
            ],
        );
    }
    machine(&mut b, page_animation);
    if start_event {
        obj(
            &mut b,
            "Event",
            &[("parentId", U(0)), ("name", S("page-event"))],
        );
    }
    if page_animation {
        assert!(start_event);
        obj(
            &mut b,
            "LinearAnimation",
            &[
                ("name", S("page-report")),
                ("fps", U(60)),
                ("duration", U(60)),
                ("loopValue", U(0)),
            ],
        );
        // The page Event follows the list and, with multiple rows, two step/unit pairs.
        obj(
            &mut b,
            "KeyedObject",
            &[("objectId", U(if rows > 1 { 7 } else { 3 }))],
        );
        obj(&mut b, "KeyedProperty", &[("propertyKey", U(395))]);
        obj(&mut b, "KeyFrameCallback", &[("frame", U(30))]);
    }
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
                ("x", F(if i == 0 { 15.0 } else { 215.0 })),
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
                    ("width", F(180.0)),
                    ("height", F(280.0)),
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
            rows as u64 + 1,
            5.0,
            i as u8 + 1,
        );
        let layer_id = host_id + 8;
        let has_flight = rows > 1 && i < 2;
        // Preserve inline component IDs used by the published video inventory.
        let flight_layer = if i == 0 { layer_id + 151 } else { layer_id };
        if has_flight && i == 1 {
            flight(&mut b, flight_layer, rows as u64 + 1, i as u8 + 1, 1);
        }
        // Same content inline. No NestedArtboard around it.
        if i == 0 {
            let inline = layer_id;
            obj(
                &mut b,
                "Node",
                &[("name", S("inline")), ("parentId", U(0)), ("x", F(420.0))],
            );
            let video = content(&mut b, inline, inline + 1, false);
            video_elements.push(video_element(i + 1, &name, "inlineClip", video));
        }
        if has_flight && i == 0 {
            flight(&mut b, flight_layer, rows as u64 + 1, i as u8 + 1, 2);
        }
        obj(
            &mut b,
            "LiveSlot",
            &[
                ("parentId", U(host_id)),
                ("unitKey", U(1)),
                ("role", U(0)),
                ("sizeMode", U(1)),
            ],
        );
        if has_flight {
            let step = if i == 0 { "back" } else { "t1" };
            let event = flight_layer + 11;
            obj(
                &mut b,
                "Event",
                &[
                    ("parentId", U(0)),
                    ("name", S(&format!("nx_enter_done:{step}"))),
                ],
            );
            arrival_timeline(&mut b, step, flight_layer + 9, event);
            arrival_machine(&mut b, i as u8 + 1, step, start_event.then_some(event));
        } else {
            machine(&mut b, false);
        }
        if i == 0 {
            let inline = layer_id;
            text_listeners(&mut b, inline + 8);
        }
    }
    obj(
        &mut b,
        "Artboard",
        &[
            ("name", S("mascot")),
            ("width", F(160.0)),
            ("height", F(260.0)),
            ("viewModelId", U(0)),
            ("clip", B(false)),
            ("defaultStateMachineId", U(0)),
        ],
    );
    let video = content(&mut b, 0, 1, true);
    // A definition-local target covers every mounted row/flight occurrence of this unit.
    video_elements.push(video_element(rows + 1, "mascot", "clip", video));
    timeline(&mut b, "walk", 1, 13, 0.0, 100.0);
    timeline(&mut b, "bob", 1, 14, 0.0, 100.0);
    machine(&mut b, true);
    text_listeners(&mut b, 8);
    obj(
        &mut b,
        "StateMachineNumber",
        &[("name", S("hits")), ("value", F(0.0))],
    );
    obj(
        &mut b,
        "StateMachineListenerSingle",
        &[("targetId", U(2)), ("listenerTypeValue", U(2))],
    );
    obj(
        &mut b,
        "ListenerNumberChange",
        &[("inputId", U(0)), ("value", F(9.0))],
    );
    let inventory = serde_json::json!({
        "videos": [{"authoredAssetId": 0, "assetUniqueName": "loop-0",
            "sourceAssetKey": "asset:mascot", "key": "loop.mp4", "required": true,
            "width": 96, "height": 64, "durationMs": 2000}],
        "videoElements": video_elements
    });
    (b, inventory)
}

fn box_shape(b: &mut Vec<u8>, parent: u64, id: u64, w: f32, h: f32, color: u32) {
    use V::*;
    obj(b, "Shape", &[("parentId", U(parent))]);
    obj(
        b,
        "Rectangle",
        &[
            ("parentId", U(id)),
            ("width", F(w)),
            ("height", F(h)),
            ("x", F(w / 2.0)),
            ("y", F(h / 2.0)),
        ],
    );
    obj(b, "Fill", &[("parentId", U(id))]);
    obj(
        b,
        "SolidColor",
        &[("parentId", U(id + 2)), ("colorValue", Color(color))],
    );
}

// All ids are relative. Both forms emit precisely these same primitives.
fn content(b: &mut Vec<u8>, parent: u64, first: u64, nested: bool) -> u64 {
    use V::*;
    obj(b, "Node", &[("name", S("keyed")), ("parentId", U(parent))]); // +0
    if nested {
        obj(
            b,
            "DataBindContext",
            &[
                ("propertyKey", U(13)),
                ("sourcePathIds", Bytes(&[0, 2])),
                ("flags", U(1)),
            ],
        );
    }
    box_shape(b, parent, first + 1, 160.0, 260.0, 0xffe02020); // +1..4
    obj(
        b,
        "LayoutComponent",
        &[
            ("name", S("fieldBox")),
            ("parentId", U(parent)),
            ("width", F(150.0)),
            ("height", F(42.0)),
            ("styleId", U(first + 6)),
        ],
    ); // +5
    obj(
        b,
        "LayoutComponentStyle",
        &[
            ("positionTypeValue", U(2)),
            ("positionLeft", F(0.0)),
            ("positionTop", F(5.0)),
        ],
    ); // +6
    obj(
        b,
        "TextInput",
        &[
            ("name", S(if nested { "edit" } else { "inlineEdit" })),
            ("parentId", U(first + 5)),
            ("text", S("")),
            ("multiline", B(false)),
        ],
    ); // +7
    obj(b, "FocusData", &[("parentId", U(first + 7))]); // +8
    obj(
        b,
        "SemanticData",
        &[("parentId", U(first + 5)), ("role", U(6))],
    ); // +9
    obj(
        b,
        "TextStylePaint",
        &[
            ("parentId", U(first + 7)),
            ("fontAssetId", U(1)),
            ("fontSize", F(20.0)),
        ],
    ); // +10
    let mut video_props = vec![
        ("name", S(if nested { "clip" } else { "inlineClip" })),
        ("parentId", U(parent)),
        ("assetId", U(0)),
        ("x", F(80.0)),
        ("y", F(85.0)),
        ("autoplay", U(1)),
        ("loop", B(true)),
        ("paused", B(false)),
    ];
    if !nested {
        // The comparison copy must not compete with the unit for Android audio focus.
        video_props.push(("muted", B(true)));
    }
    obj(b, "Video", &video_props); // +11
    obj(
        b,
        "LayoutComponent",
        &[
            ("name", S("viewport")),
            ("parentId", U(parent)),
            ("width", F(150.0)),
            ("height", F(100.0)),
            ("clip", B(true)),
            ("styleId", U(first + 13)),
        ],
    ); // +12
    obj(
        b,
        "LayoutComponentStyle",
        &[
            ("positionTypeValue", U(2)),
            ("positionLeft", F(0.0)),
            ("positionTop", F(145.0)),
        ],
    ); // +13
    obj(
        b,
        "LayoutComponent",
        &[
            ("name", S("list")),
            ("parentId", U(first + 12)),
            ("width", F(150.0)),
            ("height", F(600.0)),
            ("styleId", U(first + 15)),
        ],
    ); // +14
    obj(b, "LayoutComponentStyle", &[("flexDirectionValue", U(2))]); // +15
    obj(
        b,
        "ScrollConstraint",
        &[
            ("name", S("scroll")),
            ("parentId", U(first + 14)),
            ("directionValue", U(1)),
            ("physicsTypeValue", U(0)),
        ],
    ); // +16
    if nested {
        obj(
            b,
            "DataBindContext",
            &[
                ("propertyKey", U(760)),
                ("sourcePathIds", Bytes(&[0, 3])),
                ("flags", U(2)),
            ],
        );
    }
    for row in 0..20 {
        let id = first + 17 + row * 6;
        obj(
            b,
            "LayoutComponent",
            &[
                ("parentId", U(first + 14)),
                ("width", F(150.0)),
                ("height", F(30.0)),
                ("styleId", U(id + 1)),
            ],
        );
        obj(b, "LayoutComponentStyle", &[]);
        box_shape(
            b,
            id,
            id + 2,
            150.0,
            30.0,
            if row % 2 == 0 { 0xff3060b0 } else { 0xff80a0d0 },
        );
    }
    // The SDK's native editors are transparent. These are the real text and caret paints.
    for (offset, kind, color) in [
        (137, "TextInputText", 0xff101010),
        (140, "TextInputCursor", 0xff101010),
        (143, "TextInputSelection", 0x663355ff),
    ] {
        obj(b, kind, &[("parentId", U(first + 7))]);
        obj(b, "Fill", &[("parentId", U(first + offset))]);
        obj(
            b,
            "SolidColor",
            &[
                ("parentId", U(first + offset + 1)),
                ("colorValue", Color(color)),
            ],
        );
    }
    // The keyed marker remains wholly inside the red unit's bounds throughout its loop.
    box_shape(b, first, first + 146, 5.0, 5.0, 0xff101010);
    first + 11
}

// This runs before writing any device payload. A corrupt fixture cannot reach an app.
fn validate(bytes: &[u8], expected_rows: usize, inventory: &serde_json::Value) {
    use nuxie_render_api::{PersistentFactory, RecordingFactory};
    use nuxie_runtime::{
        File, RuntimeFactoryHandle,
        live_move::*,
        source::{
            focus_data::FocusData, generated::core_registry::CoreRegistry,
            layout_component::LayoutComponent, nested_artboard::NestedArtboard, node::Node,
        },
    };
    let mut factory = PersistentFactory::new(RecordingFactory::new());
    let file = File::import(
        bytes,
        RuntimeFactoryHandle::from_factory(&mut factory).unwrap(),
        None,
        None,
        None,
    )
    .expect("device fixture imports");
    // Read every definition, not only currently mounted/visible occurrences.
    // This is the same definition-local address space used by publication_video_metadata.
    let targets = file.with_file(|f| {
        for (ordinal, asset) in f.assets().iter().enumerate() {
            let id = asset
                .with(|o| o.as_file_asset().unwrap().file_asset_base().asset_id())
                .unwrap();
            assert_eq!(
                id as usize, ordinal,
                "published FileAsset ID must equal its ordinal"
            );
        }
        let mut targets = std::collections::BTreeSet::new();
        for (index, artboard) in f.artboards().iter().enumerate() {
            let objects = artboard
                .with(|o| o.as_artboard().unwrap().objects().to_vec())
                .unwrap();
            for (component, object) in objects.iter().enumerate() {
                let Some(object) = object else { continue };
                if let Some(reference) =
                    object.with_downcast::<nuxie_runtime::video::Video, _>(|v| v.asset_id())
                {
                    assert_eq!(reference, 0, "all videos reference the first file asset");
                    let name =
                        CoreRegistry::get_string_handle(object, i32::from(key("Video", "name")))
                            .unwrap();
                    assert_eq!(
                        CoreRegistry::get_bool_handle(object, i32::from(key("Video", "muted"))),
                        Some(name == "inlineClip"),
                        "only the inline comparison video is muted"
                    );
                    targets.insert((index as u64, component as u64));
                }
            }
        }
        targets
    });
    let declared = inventory["videoElements"]
        .as_array()
        .unwrap()
        .iter()
        .map(|v| {
            (
                v["sourceArtboardIndex"].as_u64().unwrap(),
                v["componentId"].as_u64().unwrap(),
            )
        })
        .collect::<std::collections::BTreeSet<_>>();
    assert_eq!(
        targets, declared,
        "every authored Video must have one declared definition target"
    );
    assert_eq!(targets.len(), 2, "inline and nested definitions");
    println!("PASS fixture-video-inventory rows={expected_rows} authored=0 targets={targets:?}");
    let root = file.with_file(|f| f.artboard_named("Page")).unwrap();
    let vm = file
        .with_file(|f| f.create_default_view_model_instance_for_artboard(root.core_handle()))
        .unwrap();
    let machine = root.default_state_machine_handle().unwrap();
    machine.with_instance_mut(|m| m.bind_view_model_instance(vm));
    machine.advance_and_apply(0.0);
    let rows = screens(&root).unwrap();
    assert_eq!(rows.len(), expected_rows);
    let slots = rows["A"].with_artboard(|a| a.find_all_handles::<NestedArtboard>());
    let unit = slots[0]
        .with(|o| o.as_nested_artboard().unwrap().artboard_instance_default())
        .flatten()
        .unwrap();
    let fields = unit.with_artboard(|a| a.find_all_handles::<FocusData>());
    assert_eq!(fields.len(), 1);
    assert!(
        fields[0]
            .with_downcast::<FocusData, _>(|f| f.accepts_keyboard_input())
            .unwrap(),
        "published keyboard and text-input listeners must be registered on the native field"
    );
    assert!(
        fields[0]
            .with_downcast::<FocusData, _>(|f| f.accepts_text_input())
            .unwrap()
    );
    println!("PASS fixture-native-input rows={expected_rows} fields=1 keyboard=1 text=1");
    let layout_count = unit.with_artboard(|a| a.find_all_handles::<LayoutComponent>().len());
    assert_eq!(
        layout_count, 24,
        "artboard, field, viewport, list, and twenty rows"
    );
    let keyed = unit
        .with_artboard(|a| a.find_handle::<Node>("keyed"))
        .unwrap();
    machine.advance_and_apply(0.3);
    let x = CoreRegistry::get_double_handle(&keyed, 13).unwrap();
    assert!((x - 30.0).abs() < 0.01, "keyed x={x}");
    if expected_rows > 1 {
        let mut moves = LiveMoves::default();
        moves.prepare(&root).unwrap();
        moves.after_advance(&root).unwrap();
        moves.step_begin(&root, "B", Some("t1")).unwrap();
        machine.advance_and_apply(0.2);
        moves.after_advance(&root).unwrap();
        moves.step_end(&root, "B", Some("t1")).unwrap();
        let x = CoreRegistry::get_double_handle(&keyed, 13).unwrap();
        assert!((x - 50.0).abs() < 0.01, "moved keyed x={x}");
    }
    println!(
        "PASS fixture-import rows={expected_rows} list_rows=20 layouts={layout_count} keyed_x={x}"
    );
}

pub fn emit_device_fixtures() {
    // Bazel retains this directory as undeclared test outputs. No sandbox escape is needed.
    let out = std::env::var_os("TEST_UNDECLARED_OUTPUTS_DIR")
        .map(std::path::PathBuf::from)
        .unwrap_or_else(|| std::env::temp_dir().join("nuxie-live-moves-device-v2"));
    std::fs::create_dir_all(&out).unwrap();
    for rows in [1, 2, 40] {
        let (bytes, inventory) = build(rows, true);
        validate(&bytes, rows, &inventory);
        std::fs::write(out.join(format!("page{rows}.riv")), &bytes).unwrap();
        std::fs::write(
            out.join(format!("video-inventory{rows}.json")),
            serde_json::to_vec_pretty(&inventory).unwrap(),
        )
        .unwrap();
        println!("PASS fixture rows={rows} bytes={}", bytes.len());
    }
    println!("device fixtures: {}", out.display());
}
