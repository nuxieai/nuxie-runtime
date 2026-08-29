//! Public-interface coverage for Nuxie-authored data converters in the slim
//! Apple configured-import path.

#![cfg(all(feature = "apple-runtime", any(target_os = "ios", target_os = "macos")))]

use nux_apple_product_extension::nux_product_file_import_configured;
use nux_capi::*;
use nuxie_project_data::{
    ProjectDataConverterCatalog, ProjectDataConverterDefinition, ProjectDataConverterEasing,
    ProjectDataConverterKind, ProjectDataConverterOutputType, ProjectDataConverterSpec,
};
use nuxie_schema::definition_by_name;
use std::ffi::CString;
use std::ptr;

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
    std::iter::once(definition.name)
        .chain(definition.ancestors.iter().copied())
        .filter_map(definition_by_name)
        .flat_map(|owner| owner.properties)
        .find(|property| property.name == property_name)
        .unwrap_or_else(|| panic!("fixture property exists: {type_name}.{property_name}"))
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

fn push_blob(bytes: &mut Vec<u8>, type_name: &str, name: &str, value: &[u8]) {
    push_var_uint(bytes, u64::from(property_key(type_name, name)));
    push_var_uint(bytes, value.len() as u64);
    bytes.extend_from_slice(value);
}

fn push_string(bytes: &mut Vec<u8>, type_name: &str, name: &str, value: &str) {
    push_blob(bytes, type_name, name, value.as_bytes());
}

fn interpolation_payload() -> Vec<u8> {
    let program = ProjectDataConverterCatalog::compile([ProjectDataConverterDefinition {
        id: "interpolate".to_owned(),
        spec: ProjectDataConverterSpec {
            output_type: Some(ProjectDataConverterOutputType::Number),
            kind: ProjectDataConverterKind::Interpolate {
                duration_ms: 100.0,
                easing: ProjectDataConverterEasing::Linear,
            },
        },
    }])
    .expect("valid authored-data catalog")
    .encode_program("interpolate")
    .expect("authored-data program encodes");
    // ScriptAsset contents always begin with SignedContentHeader flags. The
    // exact importer removes this unsigned flag byte before registration.
    let mut payload = vec![0];
    payload.extend_from_slice(&program);
    payload
}

fn product_converter_scene() -> Vec<u8> {
    let mut bytes = b"RIVE".to_vec();
    for value in [7, 0, 18_253, 0] {
        push_var_uint(&mut bytes, value);
    }
    push_object(&mut bytes, "Backboard", |_| {});
    push_object(&mut bytes, "ScriptAsset", |bytes| {
        push_uint(bytes, "ScriptAsset", "assetId", 0);
        push_string(bytes, "ScriptAsset", "name", "Authored data converter");
        // Pinned ScriptAsset dispatch is driven by the serialized method mask;
        // this stateful converter implements advance (bit 0) and convert
        // (bit 10). The host adapter must not infer or rewrite that metadata.
        push_uint(
            bytes,
            "ScriptAsset",
            "serializedImplementedMethods",
            (1 << 0) | (1 << 10),
        );
    });
    push_object(&mut bytes, "FileAssetContents", |bytes| {
        push_blob(
            bytes,
            "FileAssetContents",
            "bytes",
            &interpolation_payload(),
        );
    });
    push_object(&mut bytes, "ScriptedDataConverter", |bytes| {
        push_uint(bytes, "ScriptedDataConverter", "scriptAssetId", 0);
    });
    push_object(&mut bytes, "ViewModel", |bytes| {
        push_string(bytes, "ViewModel", "name", "Root");
    });
    push_object(&mut bytes, "ViewModelPropertyNumber", |bytes| {
        push_string(bytes, "ViewModelPropertyNumber", "name", "position");
    });
    push_object(&mut bytes, "ViewModelInstance", |bytes| {
        push_string(bytes, "ViewModelInstance", "name", "Defaults");
        push_uint(bytes, "ViewModelInstance", "viewModelId", 0);
    });
    push_object(&mut bytes, "ViewModelInstanceNumber", |bytes| {
        push_uint(bytes, "ViewModelInstanceNumber", "viewModelPropertyId", 0);
        push_f32(bytes, "ViewModelInstanceNumber", "propertyValue", 0.0);
    });
    push_object(&mut bytes, "Artboard", |bytes| {
        push_f32(bytes, "Artboard", "width", 100.0);
        push_f32(bytes, "Artboard", "height", 100.0);
        push_uint(bytes, "Artboard", "viewModelId", 0);
    });
    push_object(&mut bytes, "Shape", |bytes| {
        push_uint(bytes, "Node", "parentId", 0);
        push_f32(bytes, "Node", "x", 0.0);
    });
    let mut source_path = Vec::new();
    push_var_uint(&mut source_path, 0);
    push_var_uint(&mut source_path, 0);
    push_object(&mut bytes, "DataBindContext", |bytes| {
        push_uint(
            bytes,
            "DataBindContext",
            "propertyKey",
            u64::from(property_key("Node", "x")),
        );
        push_blob(bytes, "DataBindContext", "sourcePathIds", &source_path);
        push_uint(bytes, "DataBindContext", "converterId", 0);
    });
    push_object(&mut bytes, "Fill", |bytes| {
        push_uint(bytes, "Component", "parentId", 1);
    });
    push_object(&mut bytes, "SolidColor", |bytes| {
        push_uint(bytes, "Component", "parentId", 2);
        push_uint(bytes, "SolidColor", "colorValue", 0xff33_66aa);
    });
    push_object(&mut bytes, "Rectangle", |bytes| {
        push_uint(bytes, "Node", "parentId", 1);
        push_f32(bytes, "ParametricPath", "width", 20.0);
        push_f32(bytes, "ParametricPath", "height", 20.0);
    });
    bytes
}

fn result_message(result: *mut NuxCapiResult) -> String {
    if result.is_null() {
        return "missing result".to_owned();
    }
    let mut diagnostic = NuxCapiDiagnosticView::default();
    if unsafe { nux_capi_result_diagnostic(result, &mut diagnostic) } != NuxStatus::Ok {
        return "missing diagnostic".to_owned();
    }
    let bytes = unsafe {
        std::slice::from_raw_parts(diagnostic.message.data.cast::<u8>(), diagnostic.message.len)
    };
    String::from_utf8_lossy(bytes).into_owned()
}

fn metal_renderer() -> *mut NuxRenderer {
    let mut renderer = ptr::null_mut();
    let mut result = ptr::null_mut();
    assert_eq!(
        unsafe { nux_renderer_new_metal(1, 1, &mut renderer, &mut result) },
        NuxStatus::Ok
    );
    assert_eq!(unsafe { nux_capi_result_free(result) }, NuxStatus::Ok);
    assert!(!renderer.is_null());
    renderer
}

#[test]
fn product_configured_import_prepares_and_draws_authored_data_converter_scene() {
    let bytes = product_converter_scene();
    let renderer = metal_renderer();
    assert!(
        bytes.windows(8).any(|window| window == b"NUXPCV1\0"),
        "the regression must exercise the authored-data converter envelope"
    );

    let config = NuxFileImportConfig::default();
    let mut file = ptr::null_mut();
    let mut result = ptr::null_mut();
    let status = unsafe {
        nux_product_file_import_configured(
            renderer,
            bytes.as_ptr(),
            bytes.len(),
            &config,
            &mut file,
            &mut result,
        )
    };
    assert_eq!(status, NuxStatus::Ok, "{}", result_message(result));
    unsafe { nux_capi_result_free(result) };

    let mut artboard = ptr::null_mut();
    assert_eq!(
        unsafe { nux_artboard_instance_new(file, 0, &mut artboard) },
        NuxStatus::Ok,
        "authored-data converter scene must instantiate"
    );
    let mut view_model = ptr::null_mut();
    assert_eq!(
        unsafe { nux_view_model_instance_new_default(artboard, &mut view_model) },
        NuxStatus::Ok
    );
    let position = CString::new("position").expect("static string");
    assert_eq!(
        unsafe { nux_artboard_instance_bind_view_model(artboard, view_model) },
        NuxStatus::Ok
    );
    assert_eq!(
        unsafe { nux_artboard_instance_advance(artboard, 0.0, ptr::null_mut()) },
        NuxStatus::Ok
    );
    assert_eq!(
        unsafe { nux_view_model_instance_set_number(view_model, position.as_ptr(), 10.0) },
        NuxStatus::Ok
    );
    assert_eq!(
        unsafe { nux_artboard_instance_bind_view_model(artboard, view_model) },
        NuxStatus::Ok
    );
    assert_eq!(
        unsafe { nux_artboard_instance_advance(artboard, 0.0, ptr::null_mut()) },
        NuxStatus::Ok
    );
    let mut changed = false;
    assert_eq!(
        unsafe { nux_artboard_instance_advance(artboard, 0.05, &mut changed) },
        NuxStatus::Ok
    );
    assert!(
        changed,
        "the authored-data interpolation must advance exact runtime state"
    );

    unsafe {
        nux_view_model_instance_free(view_model);
        nux_artboard_instance_free(artboard);
        nux_file_free(file);
        nux_renderer_free(renderer);
    }
}
