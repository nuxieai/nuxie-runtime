//! A literal two-Screen page, also imported through the C API tests.
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
    bind(b, 575, &[screen_model, 0]);
}

pub fn fixture(rows: usize) -> Vec<u8> {
    build(rows, false)
}
pub fn control_fixture() -> Vec<u8> {
    build(2, true)
}
fn build(rows: usize, controls: bool) -> Vec<u8> {
    use V::*;
    assert!(rows >= 2);
    let mut b = b"RIVE".to_vec();
    for n in [7, 0, 0, 0] {
        var(&mut b, n);
    }
    obj(&mut b, "Backboard", &[]);
    obj(
        &mut b,
        "VideoAsset",
        &[
            ("sourceKey", S("asset:mascot")),
            ("contentType", S("video/mp4")),
        ],
    );
    obj(&mut b, "ViewModel", &[("name", S("Unit"))]);
    for name in ["count", "prop"] {
        obj(&mut b, "ViewModelPropertyNumber", &[("name", S(name))]);
    }
    obj(&mut b, "ViewModelInstance", &[("viewModelId", U(0))]);
    for i in 0..2 {
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
    }
    obj(&mut b, "ViewModel", &[("name", S("Page"))]);
    obj(&mut b, "ViewModelPropertyList", &[("name", S("screens"))]);
    obj(
        &mut b,
        "ViewModelInstance",
        &[("viewModelId", U(rows as u64 + 1))],
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
    obj(
        &mut b,
        "ArtboardComponentList",
        &[("name", S("screens")), ("parentId", U(1))],
    ); // 2
    bind(&mut b, 800, &[rows as u8 + 1, 0]);
    machine(&mut b, false);
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
            rows as u64 + 1,
            5.0,
            i as u8 + 1,
        );
        let layer_id = host_id + 6;
        obj(
            &mut b,
            "Node",
            &[("name", S("flight")), ("parentId", U(0)), ("x", F(35.0))],
        );
        slot(
            &mut b,
            layer_id,
            layer_id + 1,
            "mascot",
            rows as u64 + 1,
            0.0,
            i as u8 + 1,
        );
        if controls {
            obj(
                &mut b,
                "Node",
                &[
                    ("name", S("progress")),
                    ("parentId", U(layer_id + 1)),
                    ("x", F(0.25)),
                ],
            );
            obj(
                &mut b,
                "Node",
                &[
                    ("name", S("path")),
                    ("parentId", U(layer_id + 1)),
                    ("x", F(7.0)),
                    ("y", F(11.0)),
                ],
            );
        }
        if i == 1 {
            timeline(&mut b, "entrance", 1, 13, 200.0, 300.0);
        }
        machine(&mut b, i == 1);
    }
    obj(
        &mut b,
        "Artboard",
        &[
            ("name", S("mascot")),
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
        &[("parentId", U(5)), ("colorValue", Color(0xffff0000))],
    ); // 6
    obj(
        &mut b,
        "Video",
        &[("name", S("clip")), ("parentId", U(0)), ("assetId", U(0))],
    );
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
    b
}

pub const MOVES: &str = r#"{"transitions":[{"id":"t1","units":[{"unit":"mascot","from":{"screen":"A","path":"card/mascot"},"to":{"screen":"B","path":"header/clip/mascot"},"flight":{"screen":"B","path":"flight/mascot"},"size":"layout"}]},{"id":"back","units":[{"unit":"mascot","from":{"screen":"B","path":"header/clip/mascot"},"to":{"screen":"A","path":"card/mascot"},"flight":{"screen":"B","path":"flight/mascot"},"size":"scale"}]}]}"#;

/// A flex sibling after a NestedArtboardLayout, exercising its real layout node.
pub fn layout_fixture() -> Vec<u8> {
    use V::*;
    let mut b = b"RIVE".to_vec();
    for n in [7, 0, 0, 0] {
        var(&mut b, n);
    }
    obj(&mut b, "Backboard", &[]);
    obj(
        &mut b,
        "Artboard",
        &[
            ("name", S("LayoutPage")),
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
    b
}
