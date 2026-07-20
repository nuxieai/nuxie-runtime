#![cfg(feature = "scripting")]

use std::sync::Arc;

use anyhow::Result;
use ed25519_dalek::{Signer as _, SigningKey};
use luaur_compiler::functions::luau_compile::luau_compile;
use nuxie::{
    ArtboardSpec, DrawError, File, NodeSpec, Parent, RecordingFactory, Scene, SceneEvent,
    ScriptAssetSpec, ScriptImportCapability, ScriptedDrawableSpec,
    flow_session::{FlowSession, FlowSessionConfig},
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
    let protocol = compile_luau(include_bytes!("fixtures/vector-scripted-drawable.luau"));
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
        "factory-bearing advance executes advance/update before draw: {advanced_second}"
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
