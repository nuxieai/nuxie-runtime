#![cfg(feature = "scripting")]

use anyhow::Result;
use luaur_compiler::functions::luau_compile::luau_compile;
use nuxie::{
    ArtboardSpec, DrawError, File, NodeSpec, Parent, RecordingFactory, Scene, SceneEvent,
    ScriptAssetSpec, ScriptExecutionLimits, ScriptedDrawableSpec,
};
use nuxie_schema::definition_by_name;

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
        push_f32(bytes, "LayoutComponent", "width", 160.0);
        push_f32(bytes, "LayoutComponent", "height", 100.0);
    });
    push_object(&mut bytes, "Shape", |bytes| {
        push_uint(bytes, "Node", "parentId", 0);
    });
    push_object(&mut bytes, "Fill", |bytes| {
        push_uint(bytes, "Component", "parentId", 1);
    });
    push_object(&mut bytes, "SolidColor", |bytes| {
        push_uint(bytes, "Component", "parentId", 2);
        push_color(bytes, "SolidColor", "colorValue", 0xff11_aa22);
    });
    push_object(&mut bytes, "Rectangle", |bytes| {
        push_uint(bytes, "Node", "parentId", 1);
        push_f32(bytes, "ParametricPath", "width", 20.0);
        push_f32(bytes, "ParametricPath", "height", 20.0);
    });
    push_object(&mut bytes, "ScriptedDrawable", |bytes| {
        push_uint(bytes, "ScriptedDrawable", "parentId", 0);
        push_uint(bytes, "ScriptedDrawable", "scriptAssetId", 0);
    });
    bytes
}

fn imported_single_scripted_file(source: &[u8]) -> Vec<u8> {
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
    push_object(&mut bytes, "Artboard", |_| {});
    push_object(&mut bytes, "ScriptedDrawable", |bytes| {
        push_uint(bytes, "ScriptedDrawable", "parentId", 0);
        push_uint(bytes, "ScriptedDrawable", "scriptAssetId", 0);
    });
    bytes
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
fn imported_file_scripts_require_an_explicit_unsigned_trust_opt_in() -> Result<()> {
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
    assert!(
        inert_stream.contains("color=0xff11aa22"),
        "ordinary sibling content must still draw: {inert_stream}"
    );
    assert!(
        !inert_stream.contains("color=0xff3366cc"),
        "arbitrary File::import bytecode must execute nothing: {inert_stream}"
    );

    let trusted = File::import_with_unsigned_scripts(&bytes)?;
    let mut trusted_instance = trusted
        .default_artboard()
        .expect("fixture artboard")
        .instantiate()?;
    let mut trusted_factory = RecordingFactory::new();
    let mut trusted_renderer = trusted_factory.make_renderer();
    trusted_instance.draw(&mut trusted_factory, &mut trusted_renderer)?;
    let stream = trusted_factory.stream();
    assert!(stream.contains("color=0xff11aa22"), "{stream}");
    assert!(stream.contains("color=0xff3366cc"), "{stream}");
    assert!(stream.contains("drawPath "), "{stream}");
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
