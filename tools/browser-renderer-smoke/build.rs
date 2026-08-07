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
        vertex = { module = shader, entryPoint = "chosen_vertex" },
        fragment = { module = shader, entryPoint = "chosen_fragment" },
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

const REJECTED_SHADER_SCRIPT: &[u8] = br##"
return function(context)
    local rejected = context:shader("rejected")
    local rejectedReturnCount = select("#", context:shader("rejected"))
    if rejected ~= nil or rejectedReturnCount ~= 0 then
        error("a device-rejected ShaderAsset must return zero Lua values")
    end

    local canvas = context:gpuCanvas()
    local shader = context:shader("valid")
    local pipeline = GPUPipeline.new {
        vertex = { module = shader, entryPoint = "chosen_vertex" },
        fragment = { module = shader, entryPoint = "chosen_fragment" },
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
"##;

const WGSL: &str = r#"
@vertex
fn physical_vertex_0(@builtin(vertex_index) index: u32) -> @builtin(position) vec4<f32> {
    let x = f32(i32(index) - 1);
    let y = f32(i32(index & 1u) * 2 - 1);
    return vec4<f32>(x, y, 0.0, 1.0);
}

@vertex
fn physical_vertex_1(@builtin(vertex_index) index: u32) -> @builtin(position) vec4<f32> {
    let x = f32(i32(index) - 1);
    let y = f32(i32(index & 1u) * 2 - 1);
    return vec4<f32>(x, y, 0.0, 1.0);
}

@fragment
fn physical_fragment_0() -> @location(0) vec4<f32> {
    return vec4<f32>(0.0, 1.0, 0.0, 1.0);
}

@fragment
fn physical_fragment_1() -> @location(0) vec4<f32> {
    return vec4<f32>(1.0, 0.0, 0.0, 1.0);
}
"#;

fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    let output = PathBuf::from(env::var_os("OUT_DIR").expect("OUT_DIR is set"));
    fs::write(output.join("imported-gpu-canvas.riv"), imported_file())
        .expect("write imported GPU-canvas browser fixture");
    fs::write(
        output.join("rejected-shader-gpu-canvas.riv"),
        rejected_shader_file(),
    )
    .expect("write device-rejected ShaderAsset browser fixture");
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

fn rejected_shader_file() -> Vec<u8> {
    let mut script_payload = vec![0];
    script_payload.extend(compile_luau(REJECTED_SHADER_SCRIPT));
    let mut bytes = b"RIVE".to_vec();
    push_var_uint(&mut bytes, 7);
    push_var_uint(&mut bytes, 0);
    push_var_uint(&mut bytes, 991);
    push_var_uint(&mut bytes, 0);
    push_object(&mut bytes, "Backboard", |_| {});
    push_object(&mut bytes, "ShaderAsset", |bytes| {
        push_uint(bytes, "ShaderAsset", "assetId", 0);
        push_string(bytes, "ShaderAsset", "name", "rejected");
    });
    push_object(&mut bytes, "FileAssetContents", |bytes| {
        push_blob(
            bytes,
            "FileAssetContents",
            "bytes",
            &rejected_shader_payload(),
        );
    });
    push_object(&mut bytes, "ShaderAsset", |bytes| {
        push_uint(bytes, "ShaderAsset", "assetId", 1);
        push_string(bytes, "ShaderAsset", "name", "valid");
    });
    push_object(&mut bytes, "FileAssetContents", |bytes| {
        push_blob(bytes, "FileAssetContents", "bytes", &shader_payload());
    });
    push_object(&mut bytes, "ScriptAsset", |bytes| {
        push_uint(bytes, "ScriptAsset", "assetId", 2);
        push_string(bytes, "ScriptAsset", "name", "RejectedShaderNode");
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
        push_uint(bytes, "ScriptedDrawable", "scriptAssetId", 2);
    });
    bytes
}

const EMPTY_BINDING_MAP: &[u8] = &[2, 1, 14, 0, 0, 0, 0, 0];

fn shader_payload() -> Vec<u8> {
    shader_payload_with(WGSL, EMPTY_BINDING_MAP)
}

/// Nested-`if` depth for the device-rejected fixture. Chrome's Tint counts
/// two statement-nesting levels per `if` ("statement nesting depth / chaining
/// length exceeds limit of 127" at `createShaderModule`; measured: 63 nested
/// `if`s accepted, 64 rejected), while the runtime's CPU-side naga parse
/// counts one brace per `if` against its own 127 cap — so 90 nested `if`s
/// (91 braces) pass CPU validation (pinned by
/// `browser_rejected_fixture_construct_passes_cpu_validation` in
/// nuxie-renderer) yet the browser device rejects the module. The rejection
/// therefore genuinely comes from WebGPU's asynchronous validation scope —
/// the fail-closed path UNIV-1764 locks down. Simpler "invalid" constructs
/// are unusable here: Chrome's `createShaderModule` was measured accepting
/// `@group(255)`, oversized private storage, and 24-deep composite types
/// (those validate at pipeline creation instead), and anything naga itself
/// rejects fails closed at the wrong layer.
const REJECTED_WGSL_NESTED_IF_DEPTH: usize = 90;

fn rejected_wgsl() -> String {
    let mut wgsl = String::from(
        r#"
@vertex
fn physical_vertex_0(@builtin(vertex_index) index: u32) -> @builtin(position) vec4<f32> {
    let x = f32(i32(index) - 1);
    let y = f32(i32(index & 1u) * 2 - 1);
    return vec4<f32>(x, y, 0.0, 1.0);
}

@vertex
fn physical_vertex_1(@builtin(vertex_index) index: u32) -> @builtin(position) vec4<f32> {
    let x = f32(i32(index) - 1);
    let y = f32(i32(index & 1u) * 2 - 1);
    return vec4<f32>(x, y, 0.0, 1.0);
}

@fragment
fn physical_fragment_0() -> @location(0) vec4<f32> {
    return vec4<f32>(0.0, 1.0, 0.0, 1.0);
}

@fragment
fn physical_fragment_1() -> @location(0) vec4<f32> {
"#,
    );
    for _ in 0..REJECTED_WGSL_NESTED_IF_DEPTH {
        wgsl.push_str("if true {\n");
    }
    for _ in 0..REJECTED_WGSL_NESTED_IF_DEPTH {
        wgsl.push_str("}\n");
    }
    wgsl.push_str("    return vec4<f32>(1.0, 0.0, 0.0, 1.0);\n}\n");
    wgsl
}

fn rejected_shader_payload() -> Vec<u8> {
    shader_payload_with(&rejected_wgsl(), EMPTY_BINDING_MAP)
}

fn shader_payload_with(wgsl: &str, binding_map: &[u8]) -> Vec<u8> {
    let entries = [
        (0, "default_vertex", "physical_vertex_0"),
        (0, "chosen_vertex", "physical_vertex_1"),
        (1, "default_fragment", "physical_fragment_0"),
        (1, "chosen_fragment", "physical_fragment_1"),
    ];
    let mut source = vec![entries.len() as u8];
    for (stage, logical, physical) in entries {
        source.push(stage);
        put_string(&mut source, logical);
        put_string(&mut source, physical);
    }
    put_u32(&mut source, wgsl.len() as u32);
    source.extend_from_slice(wgsl.as_bytes());

    let mut payload = vec![0];
    put_u32(&mut payload, 0x5253_5442);
    put_u16(&mut payload, 4);
    payload.extend_from_slice(&[2, 0]);
    payload.push(0);
    put_u32(&mut payload, 0);
    put_u32(&mut payload, source.len() as u32);
    payload.push(16);
    put_u32(&mut payload, source.len() as u32);
    put_u32(&mut payload, binding_map.len() as u32);
    payload.extend(source);
    payload.extend_from_slice(binding_map);
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
