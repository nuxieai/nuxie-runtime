//! Device-shaped video identity regression across moves, collapse and wake.
// A narrow source-slot viewport identifies the actual moving video independently of ID order.
mod fixture {
    // Frozen round 3 device fixture. Keep its authored source coordinates
    // independent of the current runtime fixture builder.
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
    pub fn control_fixture() -> Vec<u8> {
        build(2, true).0
    }
    fn video_element(
        artboard: usize,
        owner: &str,
        view: &str,
        component: u64,
    ) -> serde_json::Value {
        serde_json::json!({
            "sourceArtboardIndex": artboard, "artboardId": owner,
            "viewNodeId": view, "renderedNodeId": view, "componentId": component,
            "readinessTimeoutSeconds": 5.0, "optional": false
        })
    }
    fn build(rows: usize, controls: bool) -> (Vec<u8>, serde_json::Value) {
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
                Bytes(include_bytes!("../../../fixtures/fonts/roboto-input.ttf")),
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
                "ViewModelPropertyNumber",
                &[("name", S("progress"))],
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
                "ViewModelInstanceNumber",
                &[("viewModelPropertyId", U(1)), ("propertyValue", F(0.0))],
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
            if rows > 1 {
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
                    bind(&mut b, 13, &[i as u8 + 1, 1]);
                }
            }
            // Same content inline. No NestedArtboard around it.
            if i == 0 {
                let inline = layer_id + if rows > 1 { 10 } else { 0 };
                obj(
                    &mut b,
                    "Node",
                    &[("name", S("inline")), ("parentId", U(0)), ("x", F(420.0))],
                );
                let video = content(&mut b, inline, inline + 1, false);
                video_elements.push(video_element(i + 1, &name, "inlineClip", video));
            }
            machine(&mut b, false);
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
        obj(
            b,
            "Video",
            &[
                ("name", S(if nested { "clip" } else { "inlineClip" })),
                ("parentId", U(parent)),
                ("assetId", U(0)),
                ("x", F(80.0)),
                ("y", F(85.0)),
                ("autoplay", U(1)),
                ("loop", B(true)),
                ("paused", B(false)),
            ],
        ); // +11

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
}
const MOVES: &str = r#"{"transitions":[{"id":"t1","units":[{"unit":"mascot","from":{"screen":"A","path":"card/mascot"},"to":{"screen":"B","path":"header/clip/mascot"},"flight":{"screen":"B","path":"flight/mascot"},"size":"layout"}]},{"id":"back","units":[{"unit":"mascot","from":{"screen":"B","path":"header/clip/mascot"},"to":{"screen":"A","path":"card/mascot"},"flight":{"screen":"B","path":"flight/mascot"},"size":"scale"}]}]}"#;

use nux_capi::*;
use std::{ffi::c_void, mem::size_of, ptr};
fn sv(s: &str) -> NuxStringView {
    NuxStringView {
        data: s.as_ptr().cast(),
        len: s.len(),
    }
}
#[derive(Clone, Debug)]
struct Occurrence {
    id: usize,
    generation: u64,
    source: (usize, usize),
    visible: bool,
    source_roi: bool,
}
unsafe extern "C" fn collect(ctx: *mut c_void, info: *const NuxVideoInfo) {
    let v = unsafe { &*info };
    unsafe { &mut *ctx.cast::<Vec<Occurrence>>() }.push(Occurrence {
        id: v.component_id,
        generation: v.generation,
        source: (v.source_artboard_index, v.source_component_id),
        visible: false,
        source_roi: false,
    });
}
unsafe fn step(p: *mut NuxPlayer) {
    let mut out = ptr::null_mut();
    let input = NuxPlayerStep {
        struct_size: size_of::<NuxPlayerStep>() as u32,
        elapsed_seconds: 0.0,
        ..Default::default()
    };
    assert_eq!(
        unsafe { nux_player_step(p, &input, &mut out) },
        NuxStatus::Ok
    );
    unsafe { nux_player_step_result_free(out) };
}
unsafe fn snapshot(p: *mut NuxPlayer, label: &str) -> Vec<Occurrence> {
    let mut rows = Vec::<Occurrence>::new();
    assert_eq!(
        unsafe { nux_player_visit_videos(p, Some(collect), ptr::from_mut(&mut rows).cast()) },
        NuxStatus::Ok
    );
    for r in &mut rows {
        let mut visible = 0;
        assert_eq!(
            unsafe { nux_player_video_is_visible(p, r.id, 0., 0., 800., 600., &mut visible) },
            NuxStatus::Ok
        );
        r.visible = visible != 0;
        let mut roi = 0;
        assert_eq!(
            unsafe { nux_player_video_is_visible(p, r.id, 53., 70., 54., 100., &mut roi) },
            NuxStatus::Ok
        );
        r.source_roi = roi != 0;
        println!(
            "ROI boundary={label} id={} source_roi={}",
            r.id, r.source_roi
        );
        println!(
            "VIDEO boundary={label} id={} source={}:{} generation={} visible={}",
            r.id, r.source.0, r.source.1, r.generation, r.visible
        );
    }
    rows
}
#[test]
fn live_moves_video_source_identity_survives_landing_and_collapse() {
    unsafe {
        let bytes = fixture::control_fixture();
        let (mut file, mut artboard, mut vm, mut player, mut result) = (
            ptr::null_mut(),
            ptr::null_mut(),
            ptr::null_mut(),
            ptr::null_mut(),
            ptr::null_mut(),
        );
        assert_eq!(
            nux_file_import_with_video_capabilities(
                bytes.as_ptr(),
                bytes.len(),
                &NuxRenderCallbacks::default(),
                &NuxVideoPlaybackCapabilities {
                    playback_available: 1,
                    ..Default::default()
                },
                &mut file,
                &mut result
            ),
            NuxStatus::Ok
        );
        nux_capi_result_free(result);
        assert_eq!(
            nux_artboard_instance_new_named(file, sv("Page"), &mut artboard),
            NuxStatus::Ok
        );
        assert_eq!(
            nux_view_model_instance_new_default(artboard, &mut vm),
            NuxStatus::Ok
        );
        assert_eq!(
            nux_artboard_instance_bind_view_model(artboard, vm),
            NuxStatus::Ok
        );
        assert_eq!(nux_player_new_default(artboard, &mut player), NuxStatus::Ok);
        step(player);
        assert_eq!(nux_player_set_moves(player, sv(MOVES)), NuxStatus::Ok);
        assert_eq!(
            nux_player_set_row_collapsed(player, sv("screens"), 1, 1),
            NuxStatus::Ok
        );
        step(player);
        let initial = snapshot(player, "source");
        assert_eq!(
            initial.iter().filter(|r| r.source_roi).count(),
            1,
            "source ROI is unambiguous"
        );
        let source = initial
            .iter()
            .find(|r| r.source_roi)
            .expect("source video intersects x=53..54 y=70..100")
            .clone();
        let harness_min = initial
            .iter()
            .filter(|r| r.source.0 == 3)
            .min_by_key(|r| r.id)
            .unwrap();
        println!(
            "TRACKING actual_source={} harness_min={} harness_is_source={}",
            source.id, harness_min.id, harness_min.source_roi
        );
        assert_eq!(
            nux_player_set_row_collapsed(player, sv("screens"), 1, 0),
            NuxStatus::Ok
        );
        snapshot(player, "expand-B");
        let transition = sv("t1");
        assert_eq!(
            nux_player_step_begin(player, sv("B"), &transition),
            NuxStatus::Ok
        );
        snapshot(player, "takeoff");
        step(player);
        snapshot(player, "flight");
        assert_eq!(
            nux_player_step_end(player, sv("B"), &transition),
            NuxStatus::Ok
        );
        let landed = snapshot(player, "land-before-advance");
        step(player);
        let advanced = snapshot(player, "land-after-advance");
        assert_eq!(
            nux_player_set_row_collapsed(player, sv("screens"), 0, 1),
            NuxStatus::Ok
        );
        let collapsed = snapshot(player, "collapse-A-before-advance");
        step(player);
        let final_rows = snapshot(player, "collapse-A-after-advance");
        for rows in [&landed, &advanced, &collapsed, &final_rows] {
            let moved = rows
                .iter()
                .find(|r| r.id == source.id)
                .expect("moved source remains enumerable");
            assert_eq!(
                moved.id, source.id,
                "source Video ID changed at landing/collapse"
            );
            assert_eq!(
                moved.generation, source.generation,
                "source generation changed without SDK reclaim"
            );
        }
        assert_eq!(
            nux_player_set_row_collapsed(player, sv("screens"), 1, 1),
            NuxStatus::Ok
        );
        let dormant = snapshot(player, "collapse-unit-before-advance");
        assert!(dormant.iter().all(|video| video.id != source.id));
        step(player);
        assert!(
            snapshot(player, "collapse-unit-after-advance")
                .iter()
                .all(|video| video.id != source.id)
        );
        assert_eq!(
            nux_player_set_row_collapsed(player, sv("screens"), 1, 0),
            NuxStatus::Ok
        );
        let woken = snapshot(player, "wake-unit-before-advance");
        let restored = woken
            .iter()
            .find(|video| video.id == source.id)
            .expect("same video id returns immediately on wake");
        assert_eq!(restored.generation, source.generation);
        assert_eq!(restored.source, source.source);
        assert!(restored.visible);
        step(player);
        assert!(
            snapshot(player, "wake-unit-after-advance")
                .iter()
                .any(|video| video.id == source.id && video.visible)
        );
        println!(
            "PASS actual_source_survives id={} harness_min_was_source={}",
            source.id,
            harness_min.id == source.id
        );
        nux_player_free(player);
        nux_view_model_instance_free(vm);
        nux_artboard_instance_free(artboard);
        nux_file_free(file);
    }
}
