use std::env;
use std::fs;
use std::path::PathBuf;

use luaur_compiler::functions::luau_compile::luau_compile;
use nuxie_schema::definition_by_name;

const SCRIPT: &[u8] = br#"
return function(context)
    local canvas = context:gpuCanvas()
    local shader = context:shader("scene")
    local pipeline = GPUPipeline.new {
        vertex = shader,
        fragment = shader,
        vertexLayout = {},
        colorTargets = { { format = "rgba8unorm" } },
    }
    local sampler = ImageSampler("clamp", "clamp", "nearest")
    canvas:resize(32, 24)
    return {
        drawCanvas = function(self)
            local pass = canvas:beginRenderPass {
                color = { {
                    loadOp = "clear",
                    storeOp = "store",
                    clearColor = { 0, 0, 0, 1 },
                } },
            }
            pass:setPipeline(pipeline)
            pass:draw(3)
            pass:finish()
        end,
        draw = function(self, renderer)
            renderer:drawImage(canvas.image, sampler, "srcOver", 1.0)
        end,
    }
end
"#;

const VERTEX_GLSL: &str = r#"#version 300 es
precision highp float;
precision highp int;
void main() {
    uint index = uint(gl_VertexID);
    float x = float(int(index) - 1);
    float y = float(int(index & 1u) * 2 - 1);
    gl_Position = vec4(x, y, 0.0, 1.0);
    gl_Position.yz = vec2(-gl_Position.y, gl_Position.z * 2.0 - gl_Position.w);
}
"#;

const FRAGMENT_GLSL: &str = r#"#version 300 es
precision highp float;
layout(location = 0) out vec4 color;
void main() { color = vec4(1.0, 0.0, 0.0, 1.0); }
"#;

fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    let output = PathBuf::from(env::var_os("OUT_DIR").expect("OUT_DIR is set"));
    fs::write(output.join("imported-gpu-canvas.riv"), imported_file())
        .expect("write imported GPU-canvas browser fixture");
}

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
    // SAFETY: luaur returned a non-null allocation containing output_size bytes.
    unsafe { std::slice::from_raw_parts(output.cast(), output_size) }.to_vec()
}

fn imported_file() -> Vec<u8> {
    let mut script_payload = vec![0];
    script_payload.extend(compile_luau(SCRIPT));
    let mut bytes = b"RIVE".to_vec();
    push_var_uint(&mut bytes, 7);
    push_var_uint(&mut bytes, 0);
    push_var_uint(&mut bytes, 991);
    push_var_uint(&mut bytes, 0);
    push_object(&mut bytes, "Backboard", |_| {});
    push_object(&mut bytes, "ShaderAsset", |bytes| {
        push_uint(bytes, "ShaderAsset", "assetId", 0);
        push_string(bytes, "ShaderAsset", "name", "scene");
    });
    push_object(&mut bytes, "FileAssetContents", |bytes| {
        push_blob(bytes, "FileAssetContents", "bytes", &shader_payload());
    });
    push_object(&mut bytes, "ScriptAsset", |bytes| {
        push_uint(bytes, "ScriptAsset", "assetId", 1);
        push_string(bytes, "ScriptAsset", "name", "GpuNode");
    });
    push_object(&mut bytes, "FileAssetContents", |bytes| {
        push_blob(bytes, "FileAssetContents", "bytes", &script_payload);
    });
    push_object(&mut bytes, "Artboard", |bytes| {
        push_f32(bytes, "Artboard", "width", 32.0);
        push_f32(bytes, "Artboard", "height", 24.0);
    });
    push_object(&mut bytes, "ScriptedDrawable", |bytes| {
        push_uint(bytes, "ScriptedDrawable", "parentId", 0);
        push_uint(bytes, "ScriptedDrawable", "scriptAssetId", 1);
    });
    bytes
}

fn shader_payload() -> Vec<u8> {
    let mut entries = vec![2];
    for (stage, logical, source) in [(0, "vs_main", VERTEX_GLSL), (1, "fs_main", FRAGMENT_GLSL)] {
        entries.push(stage);
        put_string(&mut entries, logical);
        put_string(&mut entries, "main");
        put_u32(&mut entries, source.len() as u32);
        entries.extend_from_slice(source.as_bytes());
    }
    let mut payload = vec![0];
    put_u32(&mut payload, 0x5253_5442);
    put_u16(&mut payload, 4);
    payload.extend_from_slice(&[1, 0, 1]);
    put_u32(&mut payload, 0);
    put_u32(&mut payload, entries.len() as u32);
    payload.extend(entries);
    payload
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

fn push_blob(bytes: &mut Vec<u8>, type_name: &str, name: &str, value: &[u8]) {
    push_var_uint(bytes, u64::from(property_key(type_name, name)));
    push_var_uint(bytes, value.len() as u64);
    bytes.extend_from_slice(value);
}

fn push_string(bytes: &mut Vec<u8>, type_name: &str, name: &str, value: &str) {
    push_blob(bytes, type_name, name, value.as_bytes());
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

fn put_u16(bytes: &mut Vec<u8>, value: u16) {
    bytes.extend_from_slice(&value.to_le_bytes());
}

fn put_u32(bytes: &mut Vec<u8>, value: u32) {
    bytes.extend_from_slice(&value.to_le_bytes());
}

fn put_string(bytes: &mut Vec<u8>, value: &str) {
    put_u16(bytes, value.len() as u16);
    bytes.extend_from_slice(value.as_bytes());
}
