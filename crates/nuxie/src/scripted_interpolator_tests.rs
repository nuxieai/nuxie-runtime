use super::*;

use luaur_compiler::functions::luau_compile::luau_compile;
use nuxie_render_api::{PersistentFactory, RecordingFactory};
use nuxie_schema::definition_by_name;

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
        .unwrap_or_else(|| panic!("fixture property {type_name}.{property_name} exists"))
        .key
        .int
}

fn push_object(bytes: &mut Vec<u8>, type_name: &str, properties: impl FnOnce(&mut Vec<u8>)) {
    push_var_uint(
        bytes,
        u64::from(definition_by_name(type_name).unwrap().type_key.int),
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

fn compile_protocol(source: &[u8]) -> Vec<u8> {
    luaur_common::set_all_flags(true);
    let mut output_size = 0;
    let output = luau_compile(
        source.as_ptr().cast(),
        source.len(),
        std::ptr::null_mut(),
        &mut output_size,
    );
    assert!(!output.is_null());
    let bytecode: Vec<u8> =
        unsafe { std::slice::from_raw_parts(output.cast::<u8>(), output_size) }.to_vec();
    unsafe extern "C" {
        fn free(pointer: *mut std::ffi::c_void);
    }
    unsafe { free(output.cast()) };
    let mut payload = vec![0];
    payload.extend(bytecode);
    payload
}

fn scripted_interpolator_file(protocol: &[u8]) -> Vec<u8> {
    let payload = compile_protocol(protocol);
    let mut bytes = b"RIVE".to_vec();
    push_var_uint(&mut bytes, 7);
    push_var_uint(&mut bytes, 0);
    push_var_uint(&mut bytes, 9_721);
    push_var_uint(&mut bytes, 0);
    push_object(&mut bytes, "Backboard", |_| {});
    push_object(&mut bytes, "ScriptAsset", |bytes| {
        push_uint(bytes, "ScriptAsset", "assetId", 0);
        push_string(bytes, "ScriptAsset", "name", "SquaredInterpolator");
    });
    push_object(&mut bytes, "FileAssetContents", |bytes| {
        push_blob(bytes, "FileAssetContents", "bytes", &payload);
    });
    push_object(&mut bytes, "Artboard", |bytes| {
        push_f32(bytes, "Artboard", "width", 100.0);
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
        push_color(bytes, "SolidColor", "colorValue", 0xff33_66cc);
    });
    push_object(&mut bytes, "Rectangle", |bytes| {
        push_uint(bytes, "Node", "parentId", 1);
        push_f32(bytes, "ParametricPath", "width", 20.0);
        push_f32(bytes, "ParametricPath", "height", 20.0);
    });
    // Artboard-local id 5; the outgoing keyframe refers to this local id.
    push_object(&mut bytes, "ScriptedInterpolator", |bytes| {
        push_uint(bytes, "ScriptedInterpolator", "scriptAssetId", 0);
    });
    push_object(&mut bytes, "ScriptInputNumber", |bytes| {
        push_uint(bytes, "Component", "parentId", 5);
        push_string(bytes, "ScriptInputNumber", "name", "scale");
        push_f32(bytes, "ScriptInputNumber", "propertyValue", 1.0);
    });
    push_object(&mut bytes, "LinearAnimation", |bytes| {
        push_string(bytes, "LinearAnimation", "name", "Scripted");
        push_uint(bytes, "LinearAnimation", "fps", 10);
        push_uint(bytes, "LinearAnimation", "duration", 10);
    });
    push_object(&mut bytes, "KeyedObject", |bytes| {
        push_uint(bytes, "KeyedObject", "objectId", 1);
    });
    push_object(&mut bytes, "KeyedProperty", |bytes| {
        push_uint(
            bytes,
            "KeyedProperty",
            "propertyKey",
            u64::from(property_key("Node", "x")),
        );
    });
    push_object(&mut bytes, "KeyFrameDouble", |bytes| {
        push_uint(bytes, "KeyFrameDouble", "frame", 0);
        push_uint(bytes, "KeyFrameDouble", "interpolationType", 1);
        push_uint(bytes, "KeyFrameDouble", "interpolatorId", 5);
        push_f32(bytes, "KeyFrameDouble", "value", 10.0);
    });
    push_object(&mut bytes, "KeyFrameDouble", |bytes| {
        push_uint(bytes, "KeyFrameDouble", "frame", 10);
        push_f32(bytes, "KeyFrameDouble", "value", 30.0);
    });
    push_object(&mut bytes, "KeyedObject", |bytes| {
        push_uint(bytes, "KeyedObject", "objectId", 3);
    });
    push_object(&mut bytes, "KeyedProperty", |bytes| {
        push_uint(
            bytes,
            "KeyedProperty",
            "propertyKey",
            u64::from(property_key("SolidColor", "colorValue")),
        );
    });
    push_object(&mut bytes, "KeyFrameColor", |bytes| {
        push_uint(bytes, "KeyFrameColor", "frame", 0);
        push_uint(bytes, "KeyFrameColor", "interpolationType", 1);
        push_uint(bytes, "KeyFrameColor", "interpolatorId", 5);
        push_color(bytes, "KeyFrameColor", "value", 0xff00_0000);
    });
    push_object(&mut bytes, "KeyFrameColor", |bytes| {
        push_uint(bytes, "KeyFrameColor", "frame", 10);
        push_color(bytes, "KeyFrameColor", "value", 0xffff_ffff);
    });
    bytes
}

#[test]
#[ignore = "fixture generator; set P2D_SCRIPTED_INTERPOLATOR_FIXTURE"]
fn write_scripted_interpolator_golden_fixture() {
    let path = std::env::var_os("P2D_SCRIPTED_INTERPOLATOR_FIXTURE")
        .expect("P2D_SCRIPTED_INTERPOLATOR_FIXTURE is set");
    std::fs::write(
        path,
        scripted_interpolator_file(
            br#"
                return function(_context)
                    return {
                        init = function(self)
                            assert(self.scale == 1)
                            self.calls = 0
                            return true
                        end,
                        transformValue = function(_self, from, to, factor)
                            _self.calls += 1
                            return from + (to - from) * factor * factor * _self.scale
                                + (_self.calls - 1) * 0.125
                        end,
                        transform = function(self, factor)
                            self.calls += 1
                            return factor * factor * self.scale
                                + (self.calls - 1) * 0.005
                        end,
                    }
                end
            "#,
        ),
    )
    .expect("write scripted interpolator golden fixture");
}

fn apply_at_half(
    protocol: &[u8],
) -> (
    f32,
    Vec<nuxie_runtime::RuntimeScriptedInterpolatorDiagnostic>,
) {
    let bytes = scripted_interpolator_file(protocol);
    let file = Arc::new(File::import_with_unsigned_scripts(&bytes).expect("fixture imports"));
    let mut artboard =
        OwnedArtboardInstance::instantiate_default(file).expect("artboard instantiates");
    let mut factory = PersistentFactory::new(RecordingFactory::new());
    artboard
        .try_advance_with_factory(&mut factory, 0.0)
        .expect("script factories mount");
    let mut animation = artboard
        .linear_animation_instance_named("Scripted")
        .expect("animation instantiates");
    artboard
        .raw()
        .advance_linear_animation_instance(&mut animation, 0.5);
    artboard
        .raw_mut()
        .apply_linear_animation_instance(&animation, 1.0);
    (
        artboard
            .raw()
            .transform_property(1, nuxie_runtime::TransformProperty::X)
            .expect("animated x exists"),
        animation.scripted_interpolator_diagnostics(),
    )
}

#[test]
fn imported_lua_transform_value_drives_keyframe_apply() {
    let (value, diagnostics) = apply_at_half(
        br#"
            return function(_context)
                return {
                    init = function(self)
                        assert(self.scale == 1)
                        self.ready = true
                        return true
                    end,
                    transformValue = function(self, from, to, factor)
                        assert(self.ready)
                        return from + (to - from) * factor * factor * self.scale
                    end,
                }
            end
        "#,
    );
    assert_eq!(value, 15.0);
    assert!(diagnostics.is_empty());
}

#[test]
fn definition_level_apply_uses_the_shared_scripted_interpolator() {
    let bytes = scripted_interpolator_file(
        br#"
            return function(_context)
                return {
                    init = function(self)
                        assert(self.scale == 1)
                        self.ready = true
                        return true
                    end,
                    transformValue = function(self, from, to, factor)
                        assert(self.ready)
                        return from + (to - from) * factor * factor * self.scale
                    end,
                }
            end
        "#,
    );
    let file = Arc::new(File::import_with_unsigned_scripts(&bytes).expect("fixture imports"));
    let mut artboard =
        OwnedArtboardInstance::instantiate_default(file).expect("artboard instantiates");
    let mut factory = PersistentFactory::new(RecordingFactory::new());
    artboard
        .try_advance_with_factory(&mut factory, 0.0)
        .expect("script factories mount");

    assert!(artboard.raw_mut().apply_linear_animation(0, 0.5, 1.0));
    assert_eq!(
        artboard
            .raw()
            .transform_property(1, nuxie_runtime::TransformProperty::X),
        Some(15.0)
    );
    assert!(
        artboard
            .raw()
            .shared_scripted_interpolator_diagnostics()
            .is_empty()
    );
}

#[test]
fn missing_and_erroring_imported_callbacks_fall_back_and_report_diagnostics() {
    let (missing_value, missing) = apply_at_half(
        br#"
            return function(_context)
                return {}
            end
        "#,
    );
    assert_eq!(missing_value, 20.0);
    assert_eq!(missing.len(), 1);
    assert!(
        missing[0]
            .error()
            .message()
            .contains("missing transformValue")
    );

    let (error_value, erroring) = apply_at_half(
        br#"
            return function(_context)
                return {
                    transformValue = function()
                        error("interpolator exploded")
                    end,
                }
            end
        "#,
    );
    assert_eq!(error_value, 20.0);
    assert_eq!(erroring.len(), 1);
    assert!(
        erroring[0]
            .error()
            .message()
            .contains("interpolator exploded")
    );
}
