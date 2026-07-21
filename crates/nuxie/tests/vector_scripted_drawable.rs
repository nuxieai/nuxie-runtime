#![cfg(feature = "scripting")]

use std::{collections::BTreeMap, sync::Arc};

use anyhow::Result;
use ed25519_dalek::{Signer as _, SigningKey};
use luaur_compiler::functions::luau_compile::luau_compile;
use nuxie::{
    ArtboardSpec, DrawError, File, NodeSpec, OwnedArtboardInstance, Parent, RecordingFactory,
    Scene, SceneEvent, ScriptAssetSpec, ScriptExecutionLimits, ScriptImportCapability,
    ScriptedDrawableSpec,
    flow_session::{
        FlowAdvance, FlowHostValue, FlowOperation, FlowOutputPayload, FlowOutputPhase,
        FlowPointerBatch, FlowPointerEvent, FlowPointerKind, FlowQuery, FlowSession,
        FlowSessionConfig, FlowSessionErrorKind,
    },
};
use nuxie_schema::definition_by_name;
use sha2::{Digest as _, Sha256};

fn compile_luau(source: &[u8]) -> Vec<u8> {
    luaur_common::set_all_flags(true);
    let mut output_size = 0;
    let output = luau_compile(
        source.as_ptr().cast(),
        source.len(),
        std::ptr::null_mut(),
        &mut output_size,
    );
    assert!(!output.is_null(), "pinned Luau compiler returned null");
    // SAFETY: the compiler returned a non-null allocation of output_size
    // bytes. Copying detaches the fixture from that allocation.
    unsafe { std::slice::from_raw_parts(output.cast(), output_size) }.to_vec()
}

fn push_var_uint(bytes: &mut Vec<u8>, mut value: u64) {
    loop {
        let mut byte = (value & 0x7f) as u8;
        value >>= 7;
        if value != 0 {
            byte |= 0x80;
        }
        bytes.push(byte);
        if value == 0 {
            break;
        }
    }
}

fn property_key(type_name: &str, property_name: &str) -> u16 {
    let definition = definition_by_name(type_name).expect("fixture type exists");
    definition
        .properties
        .iter()
        .chain(definition.ancestors.iter().flat_map(|ancestor| {
            definition_by_name(ancestor)
                .expect("fixture ancestor exists")
                .properties
                .iter()
        }))
        .find(|property| property.name == property_name)
        .expect("fixture property exists")
        .key
        .int
}

fn push_object(bytes: &mut Vec<u8>, type_name: &str, properties: impl FnOnce(&mut Vec<u8>)) {
    push_var_uint(
        bytes,
        u64::from(
            definition_by_name(type_name)
                .expect("fixture type exists")
                .type_key
                .int,
        ),
    );
    properties(bytes);
    push_var_uint(bytes, 0);
}

fn push_uint(bytes: &mut Vec<u8>, type_name: &str, name: &str, value: u64) {
    push_var_uint(bytes, u64::from(property_key(type_name, name)));
    push_var_uint(bytes, value);
}

fn push_f32(bytes: &mut Vec<u8>, type_name: &str, name: &str, value: f32) {
    push_var_uint(bytes, u64::from(property_key(type_name, name)));
    bytes.extend_from_slice(&value.to_le_bytes());
}

fn push_color(bytes: &mut Vec<u8>, type_name: &str, name: &str, value: u32) {
    push_var_uint(bytes, u64::from(property_key(type_name, name)));
    bytes.extend_from_slice(&value.to_le_bytes());
}

fn push_blob(bytes: &mut Vec<u8>, type_name: &str, name: &str, value: &[u8]) {
    push_var_uint(bytes, u64::from(property_key(type_name, name)));
    push_var_uint(bytes, value.len() as u64);
    bytes.extend_from_slice(value);
}

fn push_string(bytes: &mut Vec<u8>, type_name: &str, name: &str, value: &str) {
    push_blob(bytes, type_name, name, value.as_bytes());
}

fn imported_scripted_file() -> Vec<u8> {
    imported_scripted_file_with_protocol(include_bytes!("fixtures/vector-scripted-drawable.luau"))
}

fn imported_scripted_file_with_protocol(protocol_source: &[u8]) -> Vec<u8> {
    let protocol = compile_luau(protocol_source);
    let module = compile_luau(include_bytes!("fixtures/vector-scripted-module.luau"));
    let mut protocol_payload = vec![0];
    protocol_payload.extend(protocol);
    let mut module_payload = vec![0];
    module_payload.extend(module);

    let mut bytes = b"RIVE".to_vec();
    push_var_uint(&mut bytes, 7);
    push_var_uint(&mut bytes, 0);
    push_var_uint(&mut bytes, 991);
    push_var_uint(&mut bytes, 0);
    push_object(&mut bytes, "Backboard", |_| {});
    push_object(&mut bytes, "ScriptAsset", |bytes| {
        push_uint(bytes, "ScriptAsset", "assetId", 0);
        push_string(bytes, "ScriptAsset", "name", "VectorDrawable");
    });
    push_object(&mut bytes, "FileAssetContents", |bytes| {
        push_blob(bytes, "FileAssetContents", "bytes", &protocol_payload);
    });
    push_object(&mut bytes, "ScriptAsset", |bytes| {
        push_uint(bytes, "ScriptAsset", "assetId", 1);
        push_string(bytes, "ScriptAsset", "name", "Palette");
        push_uint(bytes, "ScriptAsset", "isModule", 1);
    });
    push_object(&mut bytes, "FileAssetContents", |bytes| {
        push_blob(bytes, "FileAssetContents", "bytes", &module_payload);
    });
    push_object(&mut bytes, "Artboard", |bytes| {
        push_f32(bytes, "Artboard", "width", 160.0);
        push_f32(bytes, "Artboard", "height", 100.0);
    });
    push_object(&mut bytes, "Shape", |bytes| {
        push_uint(bytes, "Node", "parentId", 0);
    });
    push_object(&mut bytes, "Fill", |bytes| {
        push_uint(bytes, "Component", "parentId", 1);
    });
    push_object(&mut bytes, "SolidColor", |bytes| {
        push_uint(bytes, "Component", "parentId", 2);
        push_color(bytes, "SolidColor", "colorValue", 0xffcc_3300);
    });
    push_object(&mut bytes, "Rectangle", |bytes| {
        push_uint(bytes, "Node", "parentId", 1);
        push_f32(bytes, "ParametricPath", "width", 40.0);
        push_f32(bytes, "ParametricPath", "height", 20.0);
    });
    push_object(&mut bytes, "ScriptedDrawable", |bytes| {
        push_uint(bytes, "ScriptedDrawable", "parentId", 0);
        push_uint(bytes, "ScriptedDrawable", "scriptAssetId", 0);
    });
    bytes
}

fn imported_single_scripted_file(source: &[u8]) -> Vec<u8> {
    imported_single_scripted_file_with_font(source, false)
}

fn imported_single_scripted_file_with_external_font(source: &[u8]) -> Vec<u8> {
    imported_single_scripted_file_with_font(source, true)
}

fn imported_single_scripted_file_with_font(source: &[u8], include_external_font: bool) -> Vec<u8> {
    let mut script_payload = vec![0];
    script_payload.extend(compile_luau(source));
    let mut bytes = b"RIVE".to_vec();
    push_var_uint(&mut bytes, 7);
    push_var_uint(&mut bytes, 0);
    push_var_uint(&mut bytes, 991);
    push_var_uint(&mut bytes, 0);
    push_object(&mut bytes, "Backboard", |_| {});
    push_object(&mut bytes, "ScriptAsset", |bytes| {
        push_uint(bytes, "ScriptAsset", "assetId", 0);
        push_string(bytes, "ScriptAsset", "name", "BoundedDrawable");
    });
    push_object(&mut bytes, "FileAssetContents", |bytes| {
        push_blob(bytes, "FileAssetContents", "bytes", &script_payload);
    });
    if include_external_font {
        push_object(&mut bytes, "FontAsset", |bytes| {
            push_uint(bytes, "FontAsset", "assetId", 77);
        });
    }
    push_object(&mut bytes, "Artboard", |_| {});
    push_object(&mut bytes, "ScriptedDrawable", |bytes| {
        push_uint(bytes, "ScriptedDrawable", "parentId", 0);
        push_uint(bytes, "ScriptedDrawable", "scriptAssetId", 0);
    });
    bytes
}

#[allow(clippy::arithmetic_side_effects)]
fn fixture_font_bytes() -> Vec<u8> {
    let mut accumulator = 0u32;
    let mut bit_count = 0u8;
    let mut decoded = Vec::new();
    for byte in include_bytes!("fixtures/roboto-a.ttf.base64")
        .iter()
        .copied()
        .filter(|byte| !byte.is_ascii_whitespace())
    {
        if byte == b'=' {
            break;
        }
        let value = match byte {
            b'A'..=b'Z' => byte - b'A',
            b'a'..=b'z' => byte - b'a' + 26,
            b'0'..=b'9' => byte - b'0' + 52,
            b'+' => 62,
            b'/' => 63,
            _ => panic!("invalid base64 font fixture"),
        };
        accumulator = (accumulator << 6) | u32::from(value);
        bit_count += 6;
        if bit_count >= 8 {
            bit_count -= 8;
            decoded.push((accumulator >> bit_count) as u8);
            accumulator &= (1u32 << bit_count) - 1;
        }
    }
    decoded
}

fn imported_scripted_listener_file(protocol_source: &[u8]) -> Vec<u8> {
    let mut protocol_payload = vec![0];
    protocol_payload.extend(compile_luau(protocol_source));

    let mut bytes = b"RIVE".to_vec();
    push_var_uint(&mut bytes, 7);
    push_var_uint(&mut bytes, 0);
    push_var_uint(&mut bytes, 9_402);
    push_var_uint(&mut bytes, 0);
    push_object(&mut bytes, "Backboard", |_| {});
    push_object(&mut bytes, "ScriptAsset", |bytes| {
        push_uint(bytes, "ScriptAsset", "assetId", 0);
        push_string(bytes, "ScriptAsset", "name", "PointerBudgetListener");
    });
    push_object(&mut bytes, "FileAssetContents", |bytes| {
        push_blob(bytes, "FileAssetContents", "bytes", &protocol_payload);
    });
    push_object(&mut bytes, "Artboard", |bytes| {
        push_f32(bytes, "Artboard", "width", 100.0);
        push_f32(bytes, "Artboard", "height", 100.0);
    });
    push_object(&mut bytes, "Shape", |bytes| {
        push_uint(bytes, "Node", "parentId", 0);
    });
    push_object(&mut bytes, "Rectangle", |bytes| {
        push_uint(bytes, "Node", "parentId", 1);
        push_f32(bytes, "ParametricPath", "width", 100.0);
        push_f32(bytes, "ParametricPath", "height", 100.0);
    });
    push_object(&mut bytes, "StateMachine", |bytes| {
        push_string(bytes, "StateMachine", "name", "PointerBudgetMachine");
    });
    push_object(&mut bytes, "StateMachineListenerSingle", |bytes| {
        push_uint(bytes, "StateMachineListener", "targetId", 1);
        push_uint(bytes, "StateMachineListenerSingle", "listenerTypeValue", 2);
    });
    push_object(&mut bytes, "ScriptedListenerAction", |bytes| {
        push_uint(bytes, "ScriptedListenerAction", "scriptAssetId", 0);
    });
    bytes
}

fn authenticated_capability(bytes: &[u8]) -> ScriptImportCapability {
    let signing_key = SigningKey::from_bytes(&[7; 32]);
    let artifact_sha256 = Sha256::digest(bytes)
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect::<String>();
    let manifest = serde_json::to_vec(&serde_json::json!({
        "riv": {
            "sha256": artifact_sha256,
            "sizeBytes": bytes.len(),
        },
    }))
    .expect("manifest encodes");
    let signature = signing_key.sign(&manifest);
    ScriptImportCapability::authenticate_ed25519(
        bytes,
        &manifest,
        &signature.to_bytes(),
        &signing_key.verifying_key().to_bytes(),
    )
    .expect("exact signed artifact authenticates")
}

fn scripted_scene() -> Result<(Scene, nuxie::ArtboardId)> {
    let mut scene = Scene::new();
    let artboard = scene
        .edit(|tx| {
            // Protocol is deliberately declared first. File bootstrap must
            // register every module before executing any protocol chunk.
            let protocol = tx.create_script_asset(ScriptAssetSpec {
                name: "VectorDrawable".into(),
                is_module: false,
                bytes: compile_luau(include_bytes!("fixtures/vector-scripted-drawable.luau")),
            })?;
            tx.create_script_asset(ScriptAssetSpec {
                name: "Palette".into(),
                is_module: true,
                bytes: compile_luau(include_bytes!("fixtures/vector-scripted-module.luau")),
            })?;
            let artboard = tx.create_artboard(ArtboardSpec {
                name: "Scripted vector".into(),
                width: 160.0,
                height: 100.0,
            })?;
            tx.create(
                Parent::Artboard(artboard),
                NodeSpec::ScriptedDrawable(ScriptedDrawableSpec {
                    name: "Triangle".into(),
                    x: 0.0,
                    y: 0.0,
                    opacity: 1.0,
                    rotation: 0.0,
                    scale_x: 1.0,
                    scale_y: 1.0,
                    script: protocol,
                }),
            )?;
            Ok(artboard)
        })?
        .0;
    Ok((scene, artboard))
}

fn draw(
    scene: &mut Scene,
    instance: nuxie::InstanceId,
    factory: &mut RecordingFactory,
) -> std::result::Result<String, DrawError> {
    let mut cache = scene
        .new_render_cache(instance)
        .map_err(|_| DrawError::UnknownInstance)?;
    let mut renderer = factory.make_renderer();
    scene
        .frame()
        .draw(instance, factory, &mut renderer, &mut cache)?;
    Ok(factory.stream())
}

#[test]
fn authored_vector_script_uses_one_file_program_and_fresh_occurrence_tables() -> Result<()> {
    let (mut scene, artboard) = scripted_scene()?;
    let first = scene.instantiate(artboard)?;
    let second = scene.instantiate(artboard)?;
    let mut events = Vec::<SceneEvent>::new();
    let mut factory = RecordingFactory::new();

    assert!(scene.frame().advance(first, 0.1, &mut events));
    let first_stream = draw(&mut scene, first, &mut factory)?;
    assert!(
        first_stream.contains("transform matrix=[1,0,0,1,103,2]"),
        "the exact pre-first-draw advance must replay before update: {first_stream}"
    );
    assert!(first_stream.contains("color=0xff3366cc"), "{first_stream}");
    assert!(first_stream.contains("drawPath "), "{first_stream}");

    let before_second = factory.stream().len();
    let second_stream = draw(&mut scene, second, &mut factory)?;
    let second_frame = second_stream.get(before_second..).unwrap_or_default();
    assert!(
        second_frame.contains("transform matrix=[1,0,0,1,100,2]"),
        "the protocol chunk runs once per File while each occurrence starts with fresh state: {second_frame}"
    );

    assert!(
        scene
            .frame()
            .try_advance_with_factory(second, 0.1, &mut events, &mut factory)?
    );
    let before_advanced_second = factory.stream().len();
    let second_stream = draw(&mut scene, second, &mut factory)?;
    let advanced_second = second_stream
        .get(before_advanced_second..)
        .unwrap_or_default();
    assert!(
        advanced_second.contains("transform matrix=[1,0,0,1,103,2]"),
        "factory-bearing advance mutates draw state before the invalidated paint is rendered: {advanced_second}"
    );

    let mut different_factory = RecordingFactory::new();
    let error = draw(&mut scene, first, &mut different_factory)
        .expect_err("a distinct Factory object is a distinct script resource domain");
    assert_eq!(error, DrawError::RuntimeRejected);
    Ok(())
}

#[test]
fn visual_only_import_skips_scripts_but_keeps_ordinary_visuals_live() -> Result<()> {
    let bytes = imported_scripted_file();
    let inert = File::import(&bytes)?;
    let mut inert_instance = inert
        .default_artboard()
        .expect("fixture artboard")
        .instantiate()?;
    let mut inert_factory = RecordingFactory::new();
    let mut inert_renderer = inert_factory.make_renderer();
    inert_instance.draw(&mut inert_factory, &mut inert_renderer)?;
    let inert_stream = inert_factory.stream();
    assert!(inert_stream.contains("color=0xffcc3300"), "{inert_stream}");
    assert!(inert_stream.contains("drawPath "), "{inert_stream}");
    assert!(
        !inert_stream.contains("color=0xff3366cc"),
        "visual-only imports must not execute ScriptAsset bytecode: {inert_stream}"
    );
    let _ = inert_instance.advance(0.0);
    assert!(
        !inert_instance.advance(0.1),
        "inert ScriptedDrawable topology must not keep a static visual dirty"
    );

    Ok(())
}

#[test]
fn only_an_exact_authenticated_artifact_can_execute_imported_scripts() -> Result<()> {
    let bytes = imported_scripted_file();
    let capability = authenticated_capability(&bytes);
    let trusted = Arc::new(File::import_with_script_capability(&bytes, capability)?);
    let (mut first_session, _) =
        FlowSession::create(Arc::clone(&trusted), FlowSessionConfig::default())?;
    let (mut second_session, _) =
        FlowSession::create(Arc::clone(&trusted), FlowSessionConfig::default())?;

    let mut trusted_factory = RecordingFactory::new();
    let mut trusted_renderer = trusted_factory.make_renderer();
    let mut first_cache = first_session.new_render_cache();
    first_session.draw(
        &mut trusted_factory,
        &mut trusted_renderer,
        &mut first_cache,
    )?;
    let stream = trusted_factory.stream();
    assert!(stream.contains("color=0xffcc3300"), "{stream}");
    assert!(stream.contains("color=0xff3366cc"), "{stream}");
    assert!(stream.contains("drawPath "), "{stream}");

    let mut clone_factory = RecordingFactory::new();
    let mut clone_renderer = clone_factory.make_renderer();
    let mut second_cache = second_session.new_render_cache();
    second_session.draw(&mut clone_factory, &mut clone_renderer, &mut second_cache)?;
    assert!(
        clone_factory.stream().contains("color=0xff3366cc"),
        "two sessions from one source File own fresh script VMs and distinct Factory domains"
    );

    let mut changed_bytes = bytes.clone();
    changed_bytes.push(0);
    assert!(
        File::import_with_script_capability(&changed_bytes, authenticated_capability(&bytes))
            .is_err(),
        "an authenticated capability must remain bound to the exact artifact bytes"
    );
    Ok(())
}

#[test]
fn factory_bound_session_returns_typed_creation_and_cycle_host_work_in_fifo_order() -> Result<()> {
    let bytes = imported_scripted_file_with_protocol(
        br#"
            local nuxie = require("nuxie")
            nuxie.trigger("protocol_loaded", {
                zeta = 3,
                alpha = "first",
                nested = { true, { id = "sku-1", enabled = false } },
            })

            return function(_context)
                nuxie.response.set("selection", { "sku-1", "sku-2" })
                return {
                    init = function(_self)
                        nuxie.trigger("initialized")
                        return true
                    end,
                    advance = function(_self, seconds)
                        nuxie.trigger("advanced", { delta = seconds })
                        return false
                    end,
                    draw = function(_self, _renderer)
                        nuxie.trigger("drawn")
                    end,
                }
            end
        "#,
    );
    let capability = authenticated_capability(&bytes);
    let file = Arc::new(File::import_with_script_capability(&bytes, capability)?);
    let mut factory = RecordingFactory::new();
    let (mut session, creation) = FlowSession::create_with_factory(
        Arc::clone(&file),
        FlowSessionConfig::default(),
        &mut factory,
    )?;

    assert_eq!(
        creation
            .outputs
            .iter()
            .map(|output| (output.sequence, output.cycle, output.phase))
            .collect::<Vec<_>>(),
        vec![
            (1, 0, FlowOutputPhase::HostWork),
            (2, 0, FlowOutputPhase::HostWork),
            (3, 0, FlowOutputPhase::HostWork),
        ]
    );
    assert_eq!(
        creation
            .outputs
            .iter()
            .map(|output| match &output.payload {
                FlowOutputPayload::HostCommand { name, .. } => name.as_str(),
                payload => panic!("creation emitted non-host payload {payload:?}"),
            })
            .collect::<Vec<_>>(),
        vec!["protocol_loaded", "$response_set", "initialized"]
    );
    assert_eq!(
        creation.outputs[0].payload,
        FlowOutputPayload::HostCommand {
            name: "protocol_loaded".to_owned(),
            payload: FlowHostValue::Object(BTreeMap::from([
                (
                    "alpha".to_owned(),
                    FlowHostValue::String("first".to_owned())
                ),
                (
                    "nested".to_owned(),
                    FlowHostValue::List(vec![
                        FlowHostValue::Bool(true),
                        FlowHostValue::Object(BTreeMap::from([
                            ("enabled".to_owned(), FlowHostValue::Bool(false)),
                            ("id".to_owned(), FlowHostValue::String("sku-1".to_owned())),
                        ])),
                    ]),
                ),
                ("zeta".to_owned(), FlowHostValue::Number(3.0)),
            ])),
        }
    );
    assert_eq!(
        creation.outputs[1].payload,
        FlowOutputPayload::HostCommand {
            name: "$response_set".to_owned(),
            payload: FlowHostValue::Object(BTreeMap::from([
                (
                    "field".to_owned(),
                    FlowHostValue::String("selection".to_owned()),
                ),
                (
                    "value".to_owned(),
                    FlowHostValue::List(vec![
                        FlowHostValue::String("sku-1".to_owned()),
                        FlowHostValue::String("sku-2".to_owned()),
                    ]),
                ),
            ])),
        }
    );

    let mut result = session.perform_with_factory(
        FlowOperation::Advance(FlowAdvance {
            timestamp_seconds: 0.25,
            delta_seconds: 0.25,
            render: true,
        }),
        &mut factory,
    )?;
    let mut renderer = factory.make_renderer();
    let mut render_cache = session.new_render_cache();
    session.draw_into_result(&mut factory, &mut renderer, &mut render_cache, &mut result)?;
    assert_eq!(
        result
            .outputs
            .iter()
            .map(|output| (output.sequence, output.cycle, output.phase))
            .collect::<Vec<_>>(),
        vec![
            (4, 1, FlowOutputPhase::RuntimeAdvance),
            (5, 1, FlowOutputPhase::HostWork),
            (6, 1, FlowOutputPhase::HostWork),
            (7, 1, FlowOutputPhase::Render),
        ]
    );
    assert_eq!(
        result.outputs[2].payload,
        FlowOutputPayload::HostCommand {
            name: "drawn".to_owned(),
            payload: FlowHostValue::Object(BTreeMap::new()),
        }
    );
    assert_eq!(
        result.outputs[1].payload,
        FlowOutputPayload::HostCommand {
            name: "advanced".to_owned(),
            payload: FlowHostValue::Object(BTreeMap::from([(
                "delta".to_owned(),
                FlowHostValue::Number(0.25),
            )])),
        }
    );
    Ok(())
}

#[test]
fn aggregate_host_trees_overflow_before_crossing_the_apple_result_seam_and_poison_session()
-> Result<()> {
    let bytes = imported_scripted_file_with_protocol(
        br#"
            local nuxie = require("nuxie")

            return function(_context)
                return {
                    init = function(_self)
                        return true
                    end,
                    advance = function(_self, _seconds)
                        -- The script-side arena counts the authored array while the
                        -- Apple result arena also counts each response object wrapper.
                        -- This stays within the former and deliberately exceeds the latter.
                        for command = 1, 240 do
                            local values = {}
                            for index = 1, 16 do
                                values[index] = true
                            end
                            nuxie.response.set("field_" .. command, values)
                        end
                        return false
                    end,
                }
            end
        "#,
    );
    let file = Arc::new(File::import_with_script_capability(
        &bytes,
        authenticated_capability(&bytes),
    )?);
    let mut factory = RecordingFactory::new();
    let (mut session, creation) =
        FlowSession::create_with_factory(file, FlowSessionConfig::default(), &mut factory)?;
    assert!(creation.outputs.is_empty());

    let error = session
        .perform_with_factory(
            FlowOperation::Advance(FlowAdvance {
                timestamp_seconds: 1.0,
                delta_seconds: 0.016,
                render: false,
            }),
            &mut factory,
        )
        .expect_err("all host trees must fit the one result value arena");
    assert_eq!(error.kind(), FlowSessionErrorKind::ResultLimitExceeded);
    assert!(error.message().contains("result value arena"));

    let terminal = session
        .perform_with_factory(FlowOperation::Query(FlowQuery::Values), &mut factory)
        .expect_err("a post-mutation projection failure must terminally poison the session");
    assert_eq!(terminal.kind(), FlowSessionErrorKind::Runtime);
    assert!(terminal.message().contains("flow session is terminal"));
    Ok(())
}

#[test]
fn pointer_subcycles_reset_script_budgets_and_roll_back_overflowing_host_work() -> Result<()> {
    let bytes = imported_scripted_listener_file(
        br#"
            local nuxie = require("nuxie")

            return function(_context)
                return {
                    init = function(_self)
                        return true
                    end,
                    performAction = function(_self, invocation)
                        local pointer = invocation:asPointerEvent()
                        local count = if pointer.id == 3 then 257 else 200
                        for command = 1, count do
                            nuxie.trigger("pointer_" .. pointer.id .. "_" .. command)
                        end
                    end,
                }
            end
        "#,
    );
    let file = Arc::new(File::import_with_script_capability(
        &bytes,
        authenticated_capability(&bytes),
    )?);
    let mut factory = RecordingFactory::new();
    let (mut session, creation) = FlowSession::create_with_factory(
        Arc::clone(&file),
        FlowSessionConfig::default(),
        &mut factory,
    )?;
    assert!(creation.outputs.is_empty());

    let pointer_down = |pointer_id| FlowPointerEvent {
        kind: FlowPointerKind::Down,
        pointer_id,
        x: 0.0,
        y: 0.0,
        timestamp_seconds: 0.0,
    };
    let result = session.perform_with_factory(
        FlowOperation::PointerBatch(FlowPointerBatch {
            events: vec![pointer_down(1), pointer_down(2)],
        }),
        &mut factory,
    )?;
    let host_work = result
        .outputs
        .iter()
        .filter_map(|output| match &output.payload {
            FlowOutputPayload::HostCommand { name, .. } => {
                assert_eq!(output.phase, FlowOutputPhase::HostWork);
                Some((output.cycle, name.as_str()))
            }
            _ => None,
        })
        .collect::<Vec<_>>();
    assert_eq!(host_work.len(), 400);
    assert_eq!(
        host_work
            .iter()
            .map(|(cycle, _)| *cycle)
            .collect::<std::collections::BTreeSet<_>>(),
        [1, 2].into_iter().collect()
    );
    for command in 1..=200 {
        assert_eq!(host_work[command - 1].1, format!("pointer_1_{command}"));
        assert_eq!(
            host_work[200 + command - 1].1,
            format!("pointer_2_{command}")
        );
    }

    let error = session
        .perform_with_factory(
            FlowOperation::PointerBatch(FlowPointerBatch {
                events: vec![pointer_down(3)],
            }),
            &mut factory,
        )
        .expect_err("one pointer subcycle may not emit 257 host commands");
    assert_eq!(error.kind(), FlowSessionErrorKind::ScriptResourceExceeded);
    assert!(error.message().contains("256 host commands"));

    let terminal = session
        .perform_with_factory(FlowOperation::Query(FlowQuery::Values), &mut factory)
        .expect_err("resource exhaustion must poison only this session");
    assert_eq!(terminal.kind(), FlowSessionErrorKind::Runtime);
    assert!(terminal.message().contains("flow session is terminal"));

    let mut sibling_factory = RecordingFactory::new();
    let (mut sibling, sibling_creation) =
        FlowSession::create_with_factory(file, FlowSessionConfig::default(), &mut sibling_factory)?;
    assert!(sibling_creation.outputs.is_empty());
    let sibling_result = sibling.perform_with_factory(
        FlowOperation::PointerBatch(FlowPointerBatch {
            events: vec![pointer_down(1)],
        }),
        &mut sibling_factory,
    )?;
    let sibling_commands = sibling_result
        .outputs
        .iter()
        .filter_map(|output| match &output.payload {
            FlowOutputPayload::HostCommand { name, .. } => Some(name.as_str()),
            _ => None,
        })
        .collect::<Vec<_>>();
    assert_eq!(sibling_commands.len(), 200);
    assert_eq!(sibling_commands.first(), Some(&"pointer_1_1"));
    assert_eq!(sibling_commands.last(), Some(&"pointer_1_200"));
    assert!(
        sibling_commands
            .iter()
            .all(|name| !name.starts_with("pointer_3_")),
        "partial HostWork from the poisoned session must not escape to its sibling"
    );
    Ok(())
}

#[test]
fn trusted_import_interrupts_an_infinite_draw_callback() -> Result<()> {
    let bytes = imported_single_scripted_file(
        br#"
return function(context)
    return {
        draw = function(self, renderer)
            while true do end
        end,
    }
end
"#,
    );
    let limits = ScriptExecutionLimits::new()
        .with_max_memory_bytes(4 * 1024 * 1024)
        .with_max_interrupts_per_callback(8);
    let file = File::import_with_trusted_scripts(&bytes, limits)?;
    let mut instance = file
        .default_artboard()
        .expect("fixture artboard")
        .instantiate()?;
    let mut factory = RecordingFactory::new();
    let mut renderer = factory.make_renderer();

    let error = instance
        .draw(&mut factory, &mut renderer)
        .expect_err("the imported callback must be interrupted");
    assert!(
        format!("{error:#}").contains("exceeded 8 interrupt safepoints"),
        "{error:#}"
    );
    Ok(())
}

#[test]
fn owned_font_attachment_keeps_the_retained_trusted_script_vm() -> Result<()> {
    let bytes = imported_single_scripted_file_with_external_font(
        br#"
return function(context)
    return {
        init = function(self)
            return true
        end,
        advance = function(self, seconds)
            return seconds > 0
        end,
    }
end
"#,
    );
    let file = Arc::new(File::import_with_unsigned_scripts(&bytes)?);
    let retained = Arc::clone(&file);
    let mut instance = OwnedArtboardInstance::instantiate_default(Arc::clone(&file))?;
    let mut sibling = OwnedArtboardInstance::instantiate_default(file)?;
    let mut factory = RecordingFactory::new();

    instance.try_advance_with_factory(&mut factory, 0.0)?;
    sibling.try_advance_with_factory(&mut factory, 0.0)?;
    assert!(
        instance.raw().has_script_instance_for_global(5),
        "the trusted script table must be live before the compatibility attachment"
    );

    let font_bytes = fixture_font_bytes();
    instance.attach_font_asset_bytes(77, font_bytes.clone())?;
    assert_eq!(
        sibling.raw().external_font_asset_bytes(77),
        None,
        "an existing sibling retains its own snapshot until explicitly refreshed"
    );
    sibling.attach_font_asset_bytes(77, font_bytes.clone())?;
    assert_eq!(
        sibling.raw().external_font_asset_bytes(77),
        Some(font_bytes.as_slice()),
        "an idempotent File attachment must still refresh a stale sibling instance"
    );
    assert!(
        Arc::ptr_eq(instance.file(), &retained),
        "font attachment must not COW the File away from its live script VM"
    );
    drop(retained);

    assert!(instance.try_advance_with_factory(&mut factory, 1.0 / 60.0)?);
    assert!(sibling.try_advance_with_factory(&mut factory, 1.0 / 60.0)?);
    Ok(())
}

#[test]
fn default_trusted_import_interrupts_an_infinite_advance_callback() -> Result<()> {
    let bytes = imported_single_scripted_file(
        br#"
return function(context)
    return {
        advance = function(self, seconds)
            while true do end
        end,
    }
end
"#,
    );
    let file = File::import_with_unsigned_scripts(&bytes)?;
    let mut instance = file
        .default_artboard()
        .expect("fixture artboard")
        .instantiate()?;
    let mut factory = RecordingFactory::new();

    let error = instance
        .try_advance_with_factory(&mut factory, 1.0 / 60.0)
        .expect_err("the default trusted-import budget must interrupt advance");
    assert!(
        format!("{error:#}").contains("exceeded 50000 interrupt safepoints"),
        "{error:#}"
    );
    Ok(())
}

#[test]
fn direct_trusted_callbacks_start_a_fresh_cycle_after_interrupt_exhaustion() -> Result<()> {
    let bytes = imported_single_scripted_file(
        br#"
local advances = 0
return function(context)
    return {
        advance = function(self, seconds)
            advances += 1
            if advances == 1 then
                while true do end
            end
            local ok, value = pcall(function()
                return seconds > 0
            end)
            if not ok then
                error(value)
            end
            return value
        end,
    }
end
"#,
    );
    let limits = ScriptExecutionLimits::new()
        .with_max_memory_bytes(4 * 1024 * 1024)
        .with_max_interrupts_per_callback(8);
    let file = File::import_with_trusted_scripts(&bytes, limits)?;
    let mut instance = file
        .default_artboard()
        .expect("fixture artboard")
        .instantiate()?;
    let mut factory = RecordingFactory::new();

    let error = instance
        .try_advance_with_factory(&mut factory, 1.0 / 60.0)
        .expect_err("the first callback must exhaust its interrupt budget");
    assert!(
        format!("{error:#}").contains("exceeded 8 interrupt safepoints"),
        "{error:#}"
    );
    assert!(
        instance.try_advance_with_factory(&mut factory, 1.0 / 60.0)?,
        "a later direct callback must not inherit the previous callback's terminal limit"
    );
    Ok(())
}

#[test]
fn trusted_import_rejects_unbounded_zero_limits_before_execution() {
    let bytes = imported_single_scripted_file(b"return function(context) return {} end");
    for limits in [
        ScriptExecutionLimits::new().with_max_memory_bytes(0),
        ScriptExecutionLimits::new().with_max_interrupts_per_callback(0),
    ] {
        let error = File::import_with_trusted_scripts(&bytes, limits)
            .expect_err("zero must never mean unlimited at the trusted import seam");
        assert!(
            format!("{error:#}").contains("must be greater than zero"),
            "{error:#}"
        );
    }
}

#[test]
fn trusted_import_rejects_callback_allocations_above_its_vm_limit() -> Result<()> {
    let bytes = imported_single_scripted_file(
        br#"
return function(context)
    return {
        draw = function(self, renderer)
            local oversized = string.rep("x", 8 * 1024 * 1024)
            if #oversized == 0 then error("unreachable") end
        end,
    }
end
"#,
    );
    let limits = ScriptExecutionLimits::new()
        .with_max_memory_bytes(2 * 1024 * 1024)
        .with_max_interrupts_per_callback(100_000);
    let file = File::import_with_trusted_scripts(&bytes, limits)?;
    let mut instance = file
        .default_artboard()
        .expect("fixture artboard")
        .instantiate()?;
    let mut factory = RecordingFactory::new();
    let mut renderer = factory.make_renderer();

    let error = instance
        .draw(&mut factory, &mut renderer)
        .expect_err("the imported allocation must remain VM-bounded");
    let diagnostic = format!("{error:#}");
    assert!(
        diagnostic.contains("memory") || diagnostic.contains("allocation"),
        "{diagnostic}"
    );
    Ok(())
}
