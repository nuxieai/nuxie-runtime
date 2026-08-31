//! Repository-owned authenticated MSL/RSTB/Luau fixture shared by ORE and CAPI probes.
use luaur_compiler::functions::luau_compile::luau_compile;
use nuxie_schema::definition_by_name;
use sha2::{Digest as _, Sha256};

pub const WIDTH: u32 = 16;
pub const HEIGHT: u32 = 16;
pub const EXPECTED_PIXEL: [u8; 4] = [64, 128, 191, 255];

pub const PROBE_MSL: &str = r#"
#include <metal_stdlib>
using namespace metal;

struct VertexOutput {
    float4 position [[position]];
};

vertex VertexOutput vertex_main(uint vertex_index [[vertex_id]]) {
    const float2 positions[3] = {
        float2(-1.0, -1.0),
        float2(3.0, -1.0),
        float2(-1.0, 3.0),
    };
    VertexOutput output;
    output.position = float4(positions[vertex_index], 0.0, 1.0);
    return output;
}

fragment float4 fragment_main(
    constant float4& first [[buffer(0)]],
    constant float4& second [[buffer(1)]]) {
    return first + second;
}
"#;

pub const BINDING_MAP: &[u8] = &[
    2, 1, 14, 0, 2, 0, 0, 0, // v2 header, two 14-byte rows.
    0, 0, 0, 2, 0, 0xff, 0xff, 0, 0, 0xff, 0xff, 2, 1, 0, // group 0, binding 0.
    2, 3, 0, 2, 2, 0xff, 0xff, 1, 0, 0xff, 0xff, 2, 1, 0, // group 2, binding 3.
];

const SCRIPT: &[u8] = br#"
return function(context)
    local canvas = context:gpuCanvas()
    local shader = context:shader("scene")
    assert(shader ~= nil, "scene requires native shader authority")

    local firstBytes = buffer.create(272)
    buffer.writef32(firstBytes, 256, 0.125)
    buffer.writef32(firstBytes, 260, 0.25)
    buffer.writef32(firstBytes, 264, 0.375)
    buffer.writef32(firstBytes, 268, 0.5)
    local secondBytes = buffer.create(16)
    buffer.writef32(secondBytes, 0, 0.125)
    buffer.writef32(secondBytes, 4, 0.25)
    buffer.writef32(secondBytes, 8, 0.375)
    buffer.writef32(secondBytes, 12, 0.5)

    local firstBuffer = GPUBuffer.new {
        size = 272, usage = "uniform", data = firstBytes, immutable = true,
    }
    local secondBuffer = GPUBuffer.new {
        size = 16, usage = "uniform", data = secondBytes, immutable = true,
    }
    local firstLayout = GPUBindGroupLayout.new { groupIndex = 0, shader = shader }
    -- Explicit layouts are indexed by group, including unused groups.
    local emptyLayout = GPUBindGroupLayout.new { groupIndex = 1, shader = shader }
    local secondLayout = GPUBindGroupLayout.new { groupIndex = 2, shader = shader }
    local firstGroup = GPUBindGroup.new {
        layout = firstLayout,
        -- e949 DeferredBindGroup leaves its base dynamicOffsetCount at zero,
        -- even for dynamic layouts. Exercise this offset statically instead.
        ubos = { { slot = 0, buffer = firstBuffer, offset = 256, size = 16 } },
    }
    local secondGroup = GPUBindGroup.new {
        layout = secondLayout,
        ubos = { { slot = 3, buffer = secondBuffer, offset = 0, size = 16 } },
    }
    local pipeline = GPUPipeline.new {
        vertex = { module = shader, entryPoint = "vertex_main" },
        fragment = { module = shader, entryPoint = "fragment_main" },
        vertexLayout = {},
        colorTargets = { { format = "rgba8unorm" } },
        bindGroupLayouts = { firstLayout, emptyLayout, secondLayout },
    }
    local sampler = ImageSampler("clamp", "clamp", "nearest")
    canvas:resize(16, 16)
    return {
        draw = function(self, renderer)
            local pass = canvas:beginRenderPass {
                color = { {
                    loadOp = "clear",
                    storeOp = "store",
                    clearColor = { 0, 0, 0, 1 },
                } },
            }
            pass:setPipeline(pipeline)
            pass:setBindGroup(0, firstGroup)
            pass:setBindGroup(2, secondGroup)
            pass:draw(3)
            pass:finish()
            renderer:drawImage(canvas.image, sampler, "srcOver", 1.0)
        end,
    }
end
"#;

fn compile_luau(source: &[u8]) -> Vec<u8> {
    luaur_common::set_all_flags(true);
    let mut output_size = 0;
    let output = luau_compile(
        source.as_ptr().cast(),
        source.len(),
        std::ptr::null_mut(),
        &mut output_size,
    );
    assert!(!output.is_null());
    // SAFETY: luaur returns a valid allocation containing output_size bytes.
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

fn push_blob(bytes: &mut Vec<u8>, type_name: &str, name: &str, value: &[u8]) {
    push_var_uint(bytes, u64::from(property_key(type_name, name)));
    push_var_uint(bytes, value.len() as u64);
    bytes.extend_from_slice(value);
}

fn push_string(bytes: &mut Vec<u8>, type_name: &str, name: &str, value: &str) {
    push_blob(bytes, type_name, name, value.as_bytes());
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

fn source_container() -> Vec<u8> {
    let entries = [
        (0_u8, "vertex_main", "vertex_main"),
        (1, "fragment_main", "fragment_main"),
    ];
    let mut source = vec![entries.len() as u8];
    for (stage, logical, physical) in entries {
        source.push(stage);
        put_string(&mut source, logical);
        put_string(&mut source, physical);
    }
    put_u32(&mut source, PROBE_MSL.len() as u32);
    source.extend_from_slice(PROBE_MSL.as_bytes());
    source
}

fn put_interface(
    bytes: &mut Vec<u8>,
    kind: u8,
    value: u16,
    interface_type: u8,
    interpolation: u8,
    sampling: u8,
) {
    bytes.push(kind);
    put_u16(bytes, value);
    bytes.extend_from_slice(&[interface_type, interpolation, sampling]);
}

fn supplemental_reflection(source: &[u8]) -> Vec<u8> {
    let mut bytes = vec![1];
    bytes.extend_from_slice(&Sha256::digest(source));
    bytes.extend_from_slice(&Sha256::digest(BINDING_MAP));
    bytes.push(2);

    bytes.push(0);
    put_string(&mut bytes, "vertex_main");
    put_string(&mut bytes, "vertex_main");
    for dimension in [1_u32; 3] {
        put_u32(&mut bytes, dimension);
    }
    bytes.extend_from_slice(&[1, 1]);
    put_interface(&mut bytes, 1, 0, 8, 0xff, 0xff);
    put_interface(&mut bytes, 1, 2, 3, 0xff, 0xff);

    bytes.push(1);
    put_string(&mut bytes, "fragment_main");
    put_string(&mut bytes, "fragment_main");
    for dimension in [1_u32; 3] {
        put_u32(&mut bytes, dimension);
    }
    bytes.extend_from_slice(&[0, 1]);
    put_interface(&mut bytes, 0, 0, 3, 0xff, 0xff);

    put_u16(&mut bytes, 2);
    for (group, binding) in [(0_u8, 0_u8), (2, 3)] {
        bytes.extend_from_slice(&[group, binding]);
        put_u16(&mut bytes, 1);
        bytes.extend_from_slice(&16_u64.to_le_bytes());
    }
    bytes
}

fn shader_payload() -> Vec<u8> {
    let source = source_container();
    let reflection = supplemental_reflection(&source);
    let variants = [(2_u8, source), (10, BINDING_MAP.to_vec())];
    let mut offset = 0_u32;
    let mut descriptors = Vec::new();
    for (target, blob) in &variants {
        descriptors.push((*target, offset, blob.len()));
        offset = offset
            .checked_add(u32::try_from(blob.len()).expect("small fixture"))
            .expect("small fixture offset");
    }
    let mut payload = vec![0];
    put_u32(&mut payload, 0x5253_5442);
    put_u16(&mut payload, 4);
    payload.extend_from_slice(&[2, 1]);
    for (target, offset, size) in descriptors {
        payload.push(target);
        put_u32(&mut payload, offset);
        put_u32(&mut payload, u32::try_from(size).expect("small fixture"));
    }
    payload.push(2);
    put_u16(
        &mut payload,
        u16::try_from(reflection.len()).expect("small fixture reflection"),
    );
    payload.extend_from_slice(&reflection);
    for (_, blob) in variants {
        payload.extend_from_slice(&blob);
    }
    payload
}

pub fn imported_file() -> Vec<u8> {
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
        push_string(bytes, "ScriptAsset", "name", "OreMetalProbe");
    });
    push_object(&mut bytes, "FileAssetContents", |bytes| {
        push_blob(bytes, "FileAssetContents", "bytes", &script_payload);
    });
    push_object(&mut bytes, "Artboard", |bytes| {
        push_f32(bytes, "Artboard", "width", WIDTH as f32);
        push_f32(bytes, "Artboard", "height", HEIGHT as f32);
    });
    push_object(&mut bytes, "ScriptedDrawable", |bytes| {
        push_uint(bytes, "ScriptedDrawable", "parentId", 0);
        push_uint(bytes, "ScriptedDrawable", "scriptAssetId", 1);
    });
    bytes
}
