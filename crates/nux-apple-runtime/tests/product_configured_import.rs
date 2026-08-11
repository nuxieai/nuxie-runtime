//! Product extension coverage for the slim Apple configured-import path.

#![cfg(all(
    feature = "migration-distribution",
    any(target_os = "ios", target_os = "macos")
))]

use nux_apple_runtime::nux_product_file_import_configured;
use nux_capi::*;
use nuxie_product::project_data::{
    ProjectDataConverterCatalog, ProjectDataConverterDefinition, ProjectDataConverterEasing,
    ProjectDataConverterKind, ProjectDataConverterOutputType, ProjectDataConverterSpec,
};
use nuxie_schema::definition_by_name;
use std::ffi::{CString, c_void};
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
    ProjectDataConverterCatalog::compile([ProjectDataConverterDefinition {
        id: "interpolate".to_owned(),
        spec: ProjectDataConverterSpec {
            output_type: Some(ProjectDataConverterOutputType::Number),
            kind: ProjectDataConverterKind::Interpolate {
                duration_ms: 100.0,
                easing: ProjectDataConverterEasing::Linear,
            },
        },
    }])
    .expect("valid ProjectData catalog")
    .encode_program("interpolate")
    .expect("ProjectData program encodes")
}

fn product_converter_scene() -> Vec<u8> {
    let mut bytes = b"RIVE".to_vec();
    for value in [7, 0, 18_253, 0] {
        push_var_uint(&mut bytes, value);
    }
    push_object(&mut bytes, "Backboard", |_| {});
    push_object(&mut bytes, "ScriptAsset", |bytes| {
        push_uint(bytes, "ScriptAsset", "assetId", 0);
        push_string(bytes, "ScriptAsset", "name", "Project converter");
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

#[derive(Default)]
struct TransformProbe {
    next_handle: u64,
    checksum: f64,
    samples: u64,
    drawn_paths: usize,
}

impl TransformProbe {
    fn mix(&mut self, value: f32) {
        self.samples = self.samples.saturating_add(1);
        self.checksum += f64::from(value) * self.samples as f64;
    }
}

unsafe extern "C" fn make_render_path(
    user_data: *mut c_void,
    path: *const NuxRawPathView,
    fill_rule: u8,
) -> u64 {
    let probe = unsafe { &mut *user_data.cast::<TransformProbe>() };
    probe.mix(f32::from(fill_rule));
    let view = unsafe { &*path };
    if view.point_count != 0 {
        let point_values = view
            .point_count
            .checked_mul(2)
            .expect("fixture point count fits usize");
        let points = unsafe { std::slice::from_raw_parts(view.points, point_values) };
        for value in points {
            probe.mix(*value);
        }
    }
    probe.next_handle = probe
        .next_handle
        .checked_add(1)
        .expect("fixture handles fit");
    probe.next_handle
}

unsafe extern "C" fn make_handle(user_data: *mut c_void) -> u64 {
    let probe = unsafe { &mut *user_data.cast::<TransformProbe>() };
    probe.next_handle = probe
        .next_handle
        .checked_add(1)
        .expect("fixture handles fit");
    probe.next_handle
}

unsafe extern "C" fn capture_transform(user_data: *mut c_void, transform: *const f32) {
    let probe = unsafe { &mut *user_data.cast::<TransformProbe>() };
    let values = unsafe { std::slice::from_raw_parts(transform, 6) };
    for value in values {
        probe.mix(*value);
    }
}

unsafe extern "C" fn capture_draw_path(user_data: *mut c_void, _path: u64, _paint: u64) {
    let probe = unsafe { &mut *user_data.cast::<TransformProbe>() };
    probe.drawn_paths = probe.drawn_paths.saturating_add(1);
}

fn draw_checksum(artboard: *mut NuxArtboardInstance) -> f64 {
    let mut probe = TransformProbe::default();
    let callbacks = NuxRenderCallbacks {
        user_data: (&mut probe as *mut TransformProbe).cast::<c_void>(),
        make_render_path: Some(make_render_path),
        make_empty_render_path: Some(make_handle),
        make_render_paint: Some(make_handle),
        transform: Some(capture_transform),
        draw_path: Some(capture_draw_path),
        ..NuxRenderCallbacks::default()
    };
    assert_eq!(
        unsafe { nux_artboard_instance_draw(artboard, &callbacks) },
        NuxStatus::Ok,
        "configured product converter scene must reach a real draw"
    );
    assert_ne!(probe.drawn_paths, 0, "the fixture must draw real geometry");
    probe.checksum
}

#[test]
fn product_configured_import_prepares_and_draws_project_converter_scene() {
    let bytes = product_converter_scene();
    assert!(
        bytes.windows(8).any(|window| window == b"NUXPCV1\0"),
        "the regression must exercise the product converter envelope"
    );

    let config = NuxFileImportConfig::default();
    let mut file = ptr::null_mut();
    let mut result = ptr::null_mut();
    let status = unsafe {
        nux_product_file_import_configured(
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
        "project converter scene must instantiate"
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
    let initial_checksum = draw_checksum(artboard);

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
    assert_eq!(draw_checksum(artboard), initial_checksum);
    assert_eq!(
        unsafe { nux_artboard_instance_advance(artboard, 0.05, ptr::null_mut()) },
        NuxStatus::Ok
    );
    assert_ne!(
        draw_checksum(artboard),
        initial_checksum,
        "the ProjectData interpolation must change rendered geometry"
    );

    unsafe {
        nux_view_model_instance_free(view_model);
        nux_artboard_instance_free(artboard);
        nux_file_free(file);
    }
}
