//! Android product-distribution coverage for authored data converters entering
//! through the portable configured-import surface used by the SDK.

#![cfg(all(feature = "android-vulkan", feature = "scripting"))]

use luaur_compiler::functions::luau_compile::luau_compile;
use nux_capi::*;
use nuxie_project_data::{
    ProjectDataConverterCatalog, ProjectDataConverterDefinition, ProjectDataConverterEasing,
    ProjectDataConverterKind, ProjectDataConverterOutputType, ProjectDataConverterSpec,
};
use nuxie_render_api::{PersistentFactory, RecordingFactory};
use nuxie_schema::definition_by_name;
use std::ptr;
use std::sync::Arc;

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

fn converter_payload() -> Vec<u8> {
    ProjectDataConverterCatalog::compile([ProjectDataConverterDefinition {
        id: "copy.trim".to_owned(),
        spec: ProjectDataConverterSpec {
            output_type: Some(ProjectDataConverterOutputType::Number),
            kind: ProjectDataConverterKind::Interpolate {
                duration_ms: 100.0,
                easing: ProjectDataConverterEasing::Linear,
            },
        },
    }])
    .expect("valid authored-data catalog")
    .encode_program("copy.trim")
    .expect("authored-data program encodes")
}

fn ordinary_script_payload() -> Vec<u8> {
    luaur_common::set_all_flags(true);
    let source = br#"
        return function(_context)
            return {
                init = function(_self) return true end,
                advance = function(_self, _seconds) return true end,
            }
        end
    "#;
    let mut output_size = 0;
    let output = luau_compile(
        source.as_ptr().cast(),
        source.len(),
        ptr::null_mut(),
        &mut output_size,
    );
    assert!(!output.is_null(), "fixture Luau compiles");
    let mut payload = vec![0];
    payload.extend_from_slice(unsafe { std::slice::from_raw_parts(output.cast(), output_size) });
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
        push_string(
            bytes,
            "ScriptAsset",
            "name",
            "ProjectDO.converter.copy.trim",
        );
    });
    push_object(&mut bytes, "FileAssetContents", |bytes| {
        push_blob(bytes, "FileAssetContents", "bytes", &converter_payload());
    });
    push_object(&mut bytes, "ScriptAsset", |bytes| {
        push_uint(bytes, "ScriptAsset", "assetId", 1);
        push_string(bytes, "ScriptAsset", "name", "OrdinaryInterpolator");
    });
    push_object(&mut bytes, "FileAssetContents", |bytes| {
        push_blob(
            bytes,
            "FileAssetContents",
            "bytes",
            &ordinary_script_payload(),
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
    push_object(&mut bytes, "ScriptedDrawable", |bytes| {
        push_uint(bytes, "ScriptedDrawable", "parentId", 0);
        push_uint(bytes, "ScriptedDrawable", "scriptAssetId", 1);
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
    push_object(&mut bytes, "StateMachine", |bytes| {
        push_string(bytes, "StateMachine", "name", "Ordinary");
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

fn step_result_message(result: *mut NuxPlayerStepResult) -> String {
    if result.is_null() {
        return "missing step result".to_owned();
    }
    let mut diagnostic = NuxCapiDiagnosticView::default();
    if unsafe { nux_player_step_result_diagnostic(result, &mut diagnostic) } != NuxStatus::Ok {
        return "missing step diagnostic".to_owned();
    }
    let bytes = unsafe {
        std::slice::from_raw_parts(diagnostic.message.data.cast::<u8>(), diagnostic.message.len)
    };
    String::from_utf8_lossy(bytes).into_owned()
}

fn string_view(value: &str) -> NuxStringView {
    NuxStringView {
        data: value.as_ptr().cast(),
        len: value.len(),
    }
}

#[test]
#[allow(clippy::arc_with_non_send_sync)] // The public owned-artboard API requires Arc<File>.
fn portable_android_import_routes_project_converter_away_from_luau() {
    let payload = converter_payload();
    assert!(
        !nuxie_runtime::runtime_external_data_payload_is_claimed(&payload),
        "the test process must begin without the product registry installed"
    );
    let bytes = product_converter_scene();
    assert!(
        bytes.windows(8).any(|window| window == b"NUXPCV1\0"),
        "the regression must exercise the authored-data converter envelope"
    );

    let hooks = NuxAssetHooks::default();
    let host_commands = NuxHostCommandImportConfig {
        module_name: string_view("bridge"),
        ..NuxHostCommandImportConfig::default()
    };
    let config = NuxFileImportConfig {
        host_commands: &host_commands,
        asset_hooks: &hooks,
        ..NuxFileImportConfig::default()
    };
    let mut file = ptr::null_mut();
    let mut result = ptr::null_mut();
    let status = unsafe {
        nux_file_import_configured(bytes.as_ptr(), bytes.len(), &config, &mut file, &mut result)
    };
    assert_eq!(status, NuxStatus::Ok, "{}", result_message(result));

    // Advance a second import through the same process-global registry so the
    // assertion preserves the rich script diagnostic on the killable mutant.
    let rust_host_commands = nuxie::HostCommandImportConfig::new(
        "bridge",
        nuxie::ScriptExecutionLimits::new(),
        nuxie::HostCommandLimits::new(),
    )
    .expect("valid host-command fixture config");
    let rust_file = Arc::new(
        unsafe { nuxie::File::import_trusted_with_host_commands(&bytes, rust_host_commands) }
            .expect("configured import installs the registry before any file classifies scripts"),
    );
    let mut rust_artboard = nuxie::OwnedArtboardInstance::instantiate_default(rust_file)
        .expect("fixture artboard instantiates");
    let mut factory = PersistentFactory::new(RecordingFactory::new());
    rust_artboard
        .try_advance_with_factory(&mut factory, 0.0)
        .expect("NUXPCV1 converter envelope must not be offered to the Luau VM");

    unsafe {
        assert_eq!(nux_capi_result_free(result), NuxStatus::Ok);
        let mut artboard = ptr::null_mut();
        assert_eq!(
            nux_artboard_instance_new(file, 0, &mut artboard),
            NuxStatus::Ok,
            "the converter-carrying scene must instantiate"
        );
        let mut view_model = ptr::null_mut();
        assert_eq!(
            nux_view_model_instance_new_default(artboard, &mut view_model),
            NuxStatus::Ok
        );
        assert_eq!(
            nux_artboard_instance_bind_view_model(artboard, view_model),
            NuxStatus::Ok
        );
        assert_eq!(
            nux_artboard_instance_draw(artboard, &NuxRenderCallbacks::default()),
            NuxStatus::Ok,
            "the C-imported occurrence must materialize its scripted drawable"
        );
        let mut player = ptr::null_mut();
        assert_eq!(nux_player_new_default(artboard, &mut player), NuxStatus::Ok);
        let mut step_result = ptr::null_mut();
        let step_status = nux_player_step(player, &NuxPlayerStep::default(), &mut step_result);
        assert_eq!(
            step_status,
            NuxStatus::Ok,
            "the C-imported scene must route NUXPCV1 away from Luau: {}",
            step_result_message(step_result)
        );
        assert_eq!(nux_player_step_result_free(step_result), NuxStatus::Ok);
        assert!(
            nuxie_runtime::runtime_external_data_payload_is_claimed(&payload),
            "the configured Android import must install the project-data registry"
        );
        assert_eq!(nux_player_free(player), NuxStatus::Ok);
        assert_eq!(nux_view_model_instance_free(view_model), NuxStatus::Ok);
        assert_eq!(nux_artboard_instance_free(artboard), NuxStatus::Ok);
        assert_eq!(nux_file_free(file), NuxStatus::Ok);
    }
}
