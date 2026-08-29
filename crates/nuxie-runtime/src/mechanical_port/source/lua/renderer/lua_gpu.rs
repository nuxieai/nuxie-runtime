use crate::mechanical_port::source::{
    assets::{script_asset::ScriptAsset, shader_asset::ShaderAsset},
    lua::rive_lua_libs::*,
    renderer::{
        render_context::*, rive_render_image::RiveRenderImage, rive_renderer::RiveRenderer,
    },
};

use std::{cmp::max, collections::BTreeSet, ptr};

fn buffer_usage_from_string(state: &mut LuaState, value: &str) -> BufferUsage {
    match value {
        "vertex" => BufferUsage::Vertex,
        "index" => BufferUsage::Index,
        "uniform" => BufferUsage::Uniform,
        _ => state.error(format!(
            "invalid BufferUsage '{value}' (expected 'vertex', 'index', or 'uniform')"
        )),
    }
}

fn buffer_usage_field(state: &mut LuaState, index: i32) -> BufferUsage {
    state.get_field(index, "usage");
    let usage = if state.is_string(-1) {
        let value = state.to_string(-1);
        buffer_usage_from_string(state, &value)
    } else if state.is_table(-1) {
        let count = state.object_len(-1);
        if count != 1 {
            state.error::<()>(
                "GPUBuffer.new: usage array must hold exactly one value; multiple usages are not yet supported",
            );
        }
        state.raw_get_i(-1, 1);
        if !state.is_string(-1) {
            state.error::<()>("GPUBuffer.new: usage array must hold strings");
        }
        let value = state.to_string(-1);
        let usage = buffer_usage_from_string(state, &value);
        state.pop(1);
        usage
    } else {
        state.error("GPUBuffer.new: 'usage' is required (a string or array of strings)")
    };
    state.pop(1);
    usage
}

fn texture_format(state: &mut LuaState, value: &str) -> TextureFormat {
    match value {
        "r8unorm" => TextureFormat::R8Unorm,
        "rg8unorm" => TextureFormat::Rg8Unorm,
        "rgba8unorm" => TextureFormat::Rgba8Unorm,
        "bgra8unorm" => TextureFormat::Bgra8Unorm,
        "rgba16float" => TextureFormat::Rgba16Float,
        "rg16float" => TextureFormat::Rg16Float,
        "r16float" => TextureFormat::R16Float,
        "rgba32float" => TextureFormat::Rgba32Float,
        "rgb10a2unorm" => TextureFormat::Rgb10A2Unorm,
        "rg11b10ufloat" => TextureFormat::R11G11B10Float,
        "depth16unorm" => TextureFormat::Depth16Unorm,
        "depth24plus-stencil8" => TextureFormat::Depth24PlusStencil8,
        "depth32float" => TextureFormat::Depth32Float,
        "depth32float-stencil8" => TextureFormat::Depth32FloatStencil8,
        "bc1-rgba-unorm" => TextureFormat::Bc1Unorm,
        "bc3-rgba-unorm" => TextureFormat::Bc3Unorm,
        "bc7-rgba-unorm" => TextureFormat::Bc7Unorm,
        "etc2-rgb8unorm" => TextureFormat::Etc2Rgb8,
        "etc2-rgba8unorm" => TextureFormat::Etc2Rgba8,
        "astc-4x4-unorm" => TextureFormat::Astc4x4,
        "astc-6x6-unorm" => TextureFormat::Astc6x6,
        "astc-8x8-unorm" => TextureFormat::Astc8x8,
        _ => state.error(format!("invalid TextureFormat: {value}")),
    }
}

fn texture_format_string(format: TextureFormat) -> &'static str {
    match format {
        TextureFormat::R8Unorm => "r8unorm",
        TextureFormat::Rg8Unorm => "rg8unorm",
        TextureFormat::Rgba8Unorm => "rgba8unorm",
        TextureFormat::Bgra8Unorm => "bgra8unorm",
        TextureFormat::Rgba16Float => "rgba16float",
        TextureFormat::Rg16Float => "rg16float",
        TextureFormat::R16Float => "r16float",
        TextureFormat::Rgba32Float => "rgba32float",
        TextureFormat::Rg32Float => "rg32float",
        TextureFormat::R32Float => "r32float",
        TextureFormat::Rgb10A2Unorm => "rgb10a2unorm",
        TextureFormat::R11G11B10Float => "rg11b10ufloat",
        TextureFormat::Depth16Unorm => "depth16unorm",
        TextureFormat::Depth24PlusStencil8 => "depth24plus-stencil8",
        TextureFormat::Depth32Float => "depth32float",
        TextureFormat::Depth32FloatStencil8 => "depth32float-stencil8",
        _ => "rgba8unorm",
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum FloatColorClass {
    None,
    Half,
    Full,
}

fn float_color_class(format: TextureFormat) -> FloatColorClass {
    match format {
        TextureFormat::Rgba16Float | TextureFormat::Rg16Float | TextureFormat::R16Float => {
            FloatColorClass::Half
        }
        TextureFormat::Rgba32Float
        | TextureFormat::Rg32Float
        | TextureFormat::R32Float
        | TextureFormat::R11G11B10Float => FloatColorClass::Full,
        _ => FloatColorClass::None,
    }
}

fn texture_type(state: &mut LuaState, value: &str) -> TextureType {
    match value {
        "2d" => TextureType::Texture2D,
        "cube" => TextureType::Cube,
        "3d" => TextureType::Texture3D,
        "2d-array" => TextureType::Array2D,
        _ => state.error(format!("invalid TextureType: {value}")),
    }
}

fn compare_function(state: &mut LuaState, value: &str) -> CompareFunction {
    match value {
        "never" => CompareFunction::Never,
        "less" => CompareFunction::Less,
        "equal" => CompareFunction::Equal,
        "less-equal" => CompareFunction::LessEqual,
        "greater" => CompareFunction::Greater,
        "not-equal" => CompareFunction::NotEqual,
        "greater-equal" => CompareFunction::GreaterEqual,
        "always" => CompareFunction::Always,
        _ => state.error(format!("invalid CompareFunction: {value}")),
    }
}

fn filter(state: &mut LuaState, value: &str) -> Filter {
    match value {
        "nearest" => Filter::Nearest,
        "linear" => Filter::Linear,
        _ => state.error(format!("invalid Filter: {value}")),
    }
}

fn wrap_mode(state: &mut LuaState, value: &str) -> WrapMode {
    match value {
        "repeat" => WrapMode::Repeat,
        "mirror-repeat" => WrapMode::MirrorRepeat,
        "clamp-to-edge" => WrapMode::ClampToEdge,
        _ => state.error(format!("invalid WrapMode: {value}")),
    }
}

fn vertex_format(state: &mut LuaState, value: &str) -> VertexFormat {
    match value {
        "float32" => VertexFormat::Float1,
        "float32x2" => VertexFormat::Float2,
        "float32x3" => VertexFormat::Float3,
        "float32x4" => VertexFormat::Float4,
        "uint8x4" => VertexFormat::Uint8x4,
        "unorm8x4" => VertexFormat::Unorm8x4,
        "snorm8x4" => VertexFormat::Snorm8x4,
        "float16x2" => VertexFormat::Float16x2,
        "float16x4" => VertexFormat::Float16x4,
        _ => state.error(format!("invalid VertexFormat: {value}")),
    }
}

fn cull_mode(state: &mut LuaState, value: &str) -> CullMode {
    match value {
        "none" => CullMode::None,
        "front" => CullMode::Front,
        "back" => CullMode::Back,
        _ => state.error(format!("invalid CullMode: {value}")),
    }
}

fn topology(state: &mut LuaState, value: &str) -> PrimitiveTopology {
    match value {
        "triangle-list" => PrimitiveTopology::TriangleList,
        "triangle-strip" => PrimitiveTopology::TriangleStrip,
        "line-list" => PrimitiveTopology::LineList,
        "line-strip" => PrimitiveTopology::LineStrip,
        "point-list" => PrimitiveTopology::PointList,
        _ => state.error(format!("invalid PrimitiveTopology: {value}")),
    }
}

fn blend_factor(state: &mut LuaState, value: &str) -> BlendFactor {
    match value {
        "zero" => BlendFactor::Zero,
        "one" => BlendFactor::One,
        "src" => BlendFactor::SrcColor,
        "one-minus-src" => BlendFactor::OneMinusSrcColor,
        "src-alpha" => BlendFactor::SrcAlpha,
        "one-minus-src-alpha" => BlendFactor::OneMinusSrcAlpha,
        "dst" => BlendFactor::DstColor,
        "one-minus-dst" => BlendFactor::OneMinusDstColor,
        "dst-alpha" => BlendFactor::DstAlpha,
        "one-minus-dst-alpha" => BlendFactor::OneMinusDstAlpha,
        "src-alpha-saturated" => BlendFactor::SrcAlphaSaturated,
        "constant" => BlendFactor::BlendColor,
        "one-minus-constant" => BlendFactor::OneMinusBlendColor,
        _ => state.error(format!("invalid BlendFactor: {value}")),
    }
}

fn blend_op(state: &mut LuaState, value: &str) -> BlendOp {
    match value {
        "add" => BlendOp::Add,
        "subtract" => BlendOp::Subtract,
        "reverse-subtract" => BlendOp::ReverseSubtract,
        "min" => BlendOp::Min,
        "max" => BlendOp::Max,
        _ => state.error(format!("invalid BlendOp: {value}")),
    }
}

fn winding(state: &mut LuaState, value: &str) -> FaceWinding {
    match value {
        "cw" => FaceWinding::Clockwise,
        "ccw" => FaceWinding::CounterClockwise,
        _ => state.error(format!("invalid FaceWinding: {value}")),
    }
}

fn stencil_op(state: &mut LuaState, value: &str) -> StencilOp {
    match value {
        "keep" => StencilOp::Keep,
        "zero" => StencilOp::Zero,
        "replace" => StencilOp::Replace,
        "increment-clamp" => StencilOp::IncrementClamp,
        "decrement-clamp" => StencilOp::DecrementClamp,
        "invert" => StencilOp::Invert,
        "increment-wrap" => StencilOp::IncrementWrap,
        "decrement-wrap" => StencilOp::DecrementWrap,
        _ => state.error(format!("invalid StencilOp: {value}")),
    }
}

fn write_mask(state: &mut LuaState, value: Option<&str>) -> ColorWriteMask {
    let Some(value) = value else {
        return ColorWriteMask::ALL;
    };
    if value == "all" || value == "rgba" {
        return ColorWriteMask::ALL;
    }
    if value.is_empty() || value == "none" {
        return ColorWriteMask::NONE;
    }
    let mut result = ColorWriteMask::NONE;
    for channel in value.bytes() {
        result |= match channel {
            b'r' | b'R' => ColorWriteMask::RED,
            b'g' | b'G' => ColorWriteMask::GREEN,
            b'b' | b'B' => ColorWriteMask::BLUE,
            b'a' | b'A' => ColorWriteMask::ALPHA,
            _ => state.error(format!(
                "invalid ColorWriteMask: '{value}' (expected r/g/b/a chars or 'all'/'none')"
            )),
        };
    }
    result
}

fn optional_string_field(state: &mut LuaState, index: i32, field: &str) -> Option<String> {
    state.get_field(index, field);
    let value = state.is_string(-1).then(|| state.to_string(-1));
    state.pop(1);
    value
}

fn optional_number_field(state: &mut LuaState, index: i32, field: &str, default: f64) -> f64 {
    state.get_field(index, field);
    let value = if state.is_number(-1) {
        state.to_number(-1)
    } else {
        default
    };
    state.pop(1);
    value
}

fn optional_bool_field(state: &mut LuaState, index: i32, field: &str, default: bool) -> bool {
    state.get_field(index, field);
    let value = if state.is_boolean(-1) {
        state.to_boolean(-1)
    } else {
        default
    };
    state.pop(1);
    value
}

fn stencil_face(state: &mut LuaState, index: i32, face: &mut StencilFaceState) {
    if !state.is_table(index) {
        return;
    }
    if let Some(value) = optional_string_field(state, index, "compare") {
        face.compare = compare_function(state, &value);
    }
    if let Some(value) = optional_string_field(state, index, "failOp") {
        face.fail_op = stencil_op(state, &value);
    }
    if let Some(value) = optional_string_field(state, index, "depthFailOp") {
        face.depth_fail_op = stencil_op(state, &value);
    }
    if let Some(value) = optional_string_field(state, index, "passOp") {
        face.pass_op = stencil_op(state, &value);
    }
}

fn ore_context(state: &mut LuaState) -> Option<&mut OreContext> {
    state.thread_data::<dyn ScriptingContext>().ore_context()
}

fn current_shader_target(context: Option<&OreContext>) -> ShaderTarget {
    context.map_or(ShaderTarget::Glsl, OreContext::shader_target)
}

fn binding_map_target(target: ShaderTarget) -> u8 {
    match target {
        ShaderTarget::Wgsl => 16,
        ShaderTarget::Glsl => 11,
        ShaderTarget::Msl => 10,
        ShaderTarget::Hlsl => 12,
        ShaderTarget::Spirv => 13,
    }
}

fn build_shader_entries(
    context: &mut OreContext,
    asset: &ShaderAsset,
    output: &mut ScriptedShader,
) -> bool {
    output.entries.clear();
    let target = current_shader_target(Some(context));
    let blob = asset.find_shader(target as u8);
    if blob.is_empty() {
        return false;
    }
    let binding_map_blob = asset.find_shader(binding_map_target(target));
    let vs_fixup = (target == ShaderTarget::Glsl).then(|| asset.find_shader(14));
    let fs_fixup = (target == ShaderTarget::Glsl).then(|| asset.find_shader(15));
    let pairs = asset
        .texture_sampler_pairs()
        .iter()
        .map(|pair| TextureSamplerPair {
            texture_group: pair.tex_group,
            texture_binding: pair.tex_binding,
            sampler_group: pair.samp_group,
            sampler_binding: pair.samp_binding,
        })
        .collect::<Vec<_>>();

    if matches!(target, ShaderTarget::Glsl | ShaderTarget::Hlsl) {
        let Some(views) = parse_per_entry_container(blob) else {
            return false;
        };
        for view in views.into_iter().filter(|view| view.stage < 2) {
            let mut desc = ShaderModuleDesc {
                stage: if view.stage == 0 {
                    ShaderStage::Vertex
                } else {
                    ShaderStage::Fragment
                },
                binding_map_bytes: binding_map_blob,
                shader_asset_id: asset.asset_id(),
                ..Default::default()
            };
            if target == ShaderTarget::Hlsl {
                desc.hlsl_source = Some(view.source);
                desc.hlsl_entry_point = Some(view.physical.clone());
            } else {
                desc.code = view.source;
                desc.gl_fixup_bytes = if view.stage == 0 {
                    vs_fixup.as_deref()
                } else {
                    fs_fixup.as_deref()
                };
            }
            let Some(mut module) = context.make_shader_module(desc) else {
                return false;
            };
            if !pairs.is_empty() {
                module.texture_sampler_pairs = pairs.clone();
            }
            output.entries.push(ScriptedShaderEntry {
                stage: view.stage,
                logical: view.logical,
                physical: view.physical,
                module,
            });
        }
        return !output.entries.is_empty();
    }

    let Some((views, source)) = parse_whole_module_container(blob) else {
        return false;
    };
    let Some(mut module) = context.make_shader_module(ShaderModuleDesc {
        code: source,
        binding_map_bytes: binding_map_blob,
        shader_asset_id: asset.asset_id(),
        ..Default::default()
    }) else {
        return false;
    };
    if !pairs.is_empty() {
        module.texture_sampler_pairs = pairs;
    }
    for view in views {
        output.entries.push(ScriptedShaderEntry {
            stage: view.stage,
            logical: view.logical,
            physical: view.physical,
            module: module.clone(),
        });
    }
    !output.entries.is_empty()
}

#[cfg(feature = "tools")]
fn make_shader_from_rstb(
    context: &mut OreContext,
    data: &[u8],
    output: &mut ScriptedShader,
) -> bool {
    if data.is_empty() {
        return false;
    }
    let mut bytes = Vec::with_capacity(data.len() + 1);
    bytes.push(0);
    bytes.extend_from_slice(data);
    let Some(asset) = ShaderAsset::decode(&bytes, None) else {
        return false;
    };
    build_shader_entries(context, &asset, output)
}

pub fn lua_gpu_load_shader_by_name(
    output: &mut ScriptedShader,
    context: Option<&mut dyn ScriptingContext>,
    reference: &ScopedAssetReference,
    file_asset: Option<CoreHandle>,
) -> bool {
    let Some(context) = context else {
        return false;
    };
    #[cfg(feature = "tools")]
    if let (Some(rstb), Some(ore)) = (context.find_shader_rstb(reference), context.ore_context()) {
        return make_shader_from_rstb(ore, rstb, output);
    }
    match (file_asset, context.ore_context()) {
        (Some(asset), Some(ore)) => asset
            .with_downcast::<ShaderAsset, _>(|asset| build_shader_entries(ore, asset, output))
            .unwrap_or(false),
        _ => false,
    }
}

pub fn lua_gpu_find_shader_asset(
    file: Option<RuntimeFileWeakHandle>,
    reference: &ScopedAssetReference,
) -> Option<CoreHandle> {
    file?.with_file(|file| {
        file.assets()
            .iter()
            .filter_map(|asset| {
                asset.with_downcast::<ShaderAsset, _>(|shader| {
                    let registered = if shader.folder_path().is_empty() {
                        shader.name().to_owned()
                    } else {
                        format!("{}/{}", shader.folder_path(), shader.name())
                    };
                    (
                        reference.match_name(&registered, shader.name()),
                        asset.clone(),
                    )
                })
            })
            .max_by_key(|(rank, _)| *rank)
            .filter(|(rank, _)| *rank > 0)
            .map(|(_, asset)| asset)
    })
}

pub fn lua_gpu_push_shader_by_name(state: &mut LuaState, name: &str) -> i32 {
    let reference = ScopedAssetReference::new(state, name);
    let context = state.thread_data::<dyn ScriptingContext>();
    let file_asset = context
        .current_scripted_object()
        .and_then(|object| {
            object
                .with(|object| object.scripted_object_file())
                .flatten()
        })
        .and_then(|file| lua_gpu_find_shader_asset(file, &reference));
    let mut scripted = ScriptedShader::default();
    if !lua_gpu_load_shader_by_name(&mut scripted, Some(context), &reference, file_asset) {
        return 0;
    }
    state.new_rive(scripted);
    1
}

fn gpu_buffer_construct(state: &mut LuaState) -> i32 {
    if ore_context(state).is_none() {
        state.error::<()>("GPU context not initialized");
    }
    state.check_type(1, LuaType::Table);
    state.get_field(1, "size");
    if !state.is_number(-1) {
        state.error::<()>("GPUBuffer.new: 'size' is required (bytes)");
    }
    let size_number = state.to_number(-1);
    state.pop(1);
    let valid_integer = (1.0..=4_294_967_295.0).contains(&size_number)
        && size_number == (size_number as u32) as f64;
    if !valid_integer {
        state.error::<()>("GPUBuffer.new: 'size' must be a positive integer number of bytes");
    }
    let immutable = optional_bool_field(state, 1, "immutable", false);
    let label = optional_string_field(state, 1, "label");
    state.get_field(1, "data");
    let data = if state.is_nil(-1) {
        None
    } else {
        let Some(data) = state.to_buffer(-1) else {
            state.error::<()>("GPUBuffer.new: 'data' must be a Luau buffer");
            unreachable!()
        };
        if data.len() != size_number as usize {
            state.error::<()>(format!(
                "GPUBuffer.new: data length ({}) must equal size ({})",
                data.len(),
                size_number as u32
            ));
        }
        Some(data.to_vec())
    };
    state.pop(1);
    if immutable && data.is_none() {
        state.error::<()>("GPUBuffer.new: immutable=true requires 'data' (the GPU buffer is GPU-only after creation)");
    }
    let desc = BufferDesc {
        size: size_number as u32,
        usage: buffer_usage_field(state, 1),
        immutable,
        label,
        data,
    };
    let context = ore_context(state).unwrap();
    context.clear_last_error();
    let Some(buffer) = context.make_buffer(desc) else {
        let error = context.last_error();
        if error.is_empty() {
            state.error::<()>("GPUBuffer.new: failed to create buffer");
        }
        state.error::<()>(format!("GPUBuffer.new: {error}"));
        unreachable!()
    };
    state.new_rive(ScriptedGPUBuffer { buffer, immutable });
    1
}

fn gpu_buffer_write(state: &mut LuaState) -> i32 {
    let buffer = state.to_rive_mut::<ScriptedGPUBuffer>(1);
    if buffer.immutable {
        state.error::<()>("GPUBuffer:write: buffer was created with immutable=true; its contents are fixed at construction");
    }
    let Some(data) = state.to_buffer(2) else {
        state.type_error::<()>(2, "buffer");
        unreachable!()
    };
    let destination_offset = state.number_or(3, 0.0) as u32;
    let source_offset = state.number_or(4, 0.0) as u32;
    if source_offset as usize > data.len() {
        state.error::<()>(format!(
            "GPUBuffer:write: srcOffset({source_offset}) exceeds source buffer size({})",
            data.len()
        ));
    }
    let byte_length = state.number_or(5, (data.len() - source_offset as usize) as f64) as u32;
    if source_offset as u64 + byte_length as u64 > data.len() as u64 {
        state.error::<()>(format!(
            "GPUBuffer:write: srcOffset({source_offset}) + byteLength({byte_length}) exceeds source buffer size({})",
            data.len()
        ));
    }
    if destination_offset as u64 + byte_length as u64 > buffer.buffer.size() as u64 {
        state.error::<()>(format!(
            "GPUBuffer:write: offset({destination_offset}) + size({byte_length}) = {} exceeds buffer size({})",
            destination_offset + byte_length,
            buffer.buffer.size()
        ));
    }
    buffer.buffer.update(
        &data[source_offset as usize..(source_offset + byte_length) as usize],
        destination_offset,
    );
    0
}

fn gpu_buffer_namecall(state: &mut LuaState) -> i32 {
    let (name, atom) = state.namecall_atom();
    match atom {
        LuaAtoms::Write => gpu_buffer_write(state),
        _ => state.error(format!(
            "{} is not a valid method of {}",
            name.unwrap_or_default(),
            ScriptedGPUBuffer::LUA_NAME
        )),
    }
}

fn gpu_buffer_index(state: &mut LuaState) -> i32 {
    let (key, atom) = state.string_atom(2);
    match atom {
        LuaAtoms::Size => {
            let size = state.to_rive::<ScriptedGPUBuffer>(1).buffer.size();
            state.push_number(size as f64);
            1
        }
        _ => state.error(format!(
            "'{}' is not a valid index of GPUBuffer",
            key.unwrap_or_default()
        )),
    }
}

fn check_sample_count(state: &mut LuaState, sample_count: u32) {
    if sample_count <= 1 {
        return;
    }
    if !sample_count.is_power_of_two() {
        state.error::<()>(format!(
            "sampleCount must be a power of two (got {sample_count})"
        ));
    }
    if let Some(context) = ore_context(state) {
        let maximum = context.features().max_samples;
        if sample_count > maximum {
            state.error::<()>(format!("sampleCount {sample_count} exceeds device maximum of {maximum} — query context:features().maxSamples before creating MSAA textures"));
        }
    }
}

fn gpu_texture_construct(state: &mut LuaState) -> i32 {
    if ore_context(state).is_none() {
        state.error::<()>("GPU context not initialized");
    }
    state.check_type(1, LuaType::Table);
    let mut desc = TextureDesc {
        width: optional_number_field(state, 1, "width", 0.0) as u32,
        height: optional_number_field(state, 1, "height", 0.0) as u32,
        render_target: optional_bool_field(state, 1, "renderTarget", false),
        sample_count: optional_number_field(state, 1, "sampleCount", 1.0) as u32,
        num_mipmaps: optional_number_field(state, 1, "mipmaps", 1.0) as u32,
        depth_or_array_layers: optional_number_field(state, 1, "layers", 1.0) as u32,
        ..Default::default()
    };
    if desc.width == 0 || desc.height == 0 {
        state.error::<()>("GPUTexture requires width and height");
    }
    if let Some(value) = optional_string_field(state, 1, "format") {
        desc.format = texture_format(state, &value);
    }
    if let Some(value) = optional_string_field(state, 1, "type") {
        desc.texture_type = texture_type(state, &value);
    }
    check_sample_count(state, desc.sample_count);
    let context = ore_context(state).unwrap();
    if desc.render_target {
        let class = float_color_class(desc.format);
        let features = context.features();
        let supported = class == FloatColorClass::None
            || class == FloatColorClass::Half && features.color_buffer_half_float
            || class == FloatColorClass::Full && features.color_buffer_float;
        if !supported {
            let capability = if class == FloatColorClass::Half {
                "colorBufferHalfFloat"
            } else {
                "colorBufferFloat"
            };
            state.error::<()>(format!(
                "GPUTexture.new: float format {} as a renderTarget requires the {capability} feature, which the active backend does not support",
                texture_format_string(desc.format)
            ));
        }
    }
    context.clear_last_error();
    let Some(texture) = context.make_texture(desc) else {
        let error = context.last_error();
        if error.is_empty() {
            state.error::<()>("GPUTexture.new: failed to create texture");
        }
        state.error::<()>(format!("GPUTexture.new: {error}"));
        unreachable!()
    };
    state.new_rive(ScriptedGPUTexture { texture });
    1
}

fn gpu_texture_view(state: &mut LuaState) -> i32 {
    let texture = state.to_rive::<ScriptedGPUTexture>(1).texture.clone();
    let mut desc = TextureViewDesc {
        texture: texture.clone(),
        mip_count: texture.num_mipmaps(),
        layer_count: texture.depth_or_array_layers(),
        dimension: match texture.texture_type() {
            TextureType::Texture2D => TextureViewDimension::Texture2D,
            TextureType::Cube => TextureViewDimension::Cube,
            TextureType::Texture3D => TextureViewDimension::Texture3D,
            TextureType::Array2D => TextureViewDimension::Array2D,
        },
        ..Default::default()
    };
    if state.is_table(2) {
        if let Some(dimension) = optional_string_field(state, 2, "dimension") {
            desc.dimension = match dimension.as_str() {
                "2d" => TextureViewDimension::Texture2D,
                "cube" => TextureViewDimension::Cube,
                "3d" => TextureViewDimension::Texture3D,
                "2d-array" => TextureViewDimension::Array2D,
                _ => desc.dimension,
            };
        }
        desc.base_mip_level = optional_number_field(state, 2, "baseMipLevel", 0.0) as u32;
        desc.mip_count = optional_number_field(state, 2, "mipCount", desc.mip_count as f64) as u32;
        desc.base_layer = optional_number_field(state, 2, "baseLayer", 0.0) as u32;
        desc.layer_count =
            optional_number_field(state, 2, "layerCount", desc.layer_count as f64) as u32;
    }
    let context = ore_context(state).unwrap();
    context.clear_last_error();
    let Some(view) = context.make_texture_view(desc) else {
        let error = context.last_error();
        if error.is_empty() {
            state.error::<()>("GPUTexture:view: failed to create texture view");
        }
        state.error::<()>(format!("GPUTexture:view: {error}"));
        unreachable!()
    };
    state.new_rive(ScriptedGPUTextureView {
        view,
        retained_image: None,
    });
    1
}

fn gpu_texture_upload(state: &mut LuaState) -> i32 {
    let texture = state.to_rive::<ScriptedGPUTexture>(1).texture.clone();
    state.check_type(2, LuaType::Table);
    state.get_field(2, "data");
    let Some(data) = state.to_buffer(-1).map(<[u8]>::to_vec) else {
        state.error::<()>("upload requires 'data' field of type buffer");
        unreachable!()
    };
    state.pop(1);
    let mut desc = TextureDataDesc {
        data,
        width: optional_number_field(state, 2, "width", texture.width() as f64) as u32,
        height: optional_number_field(state, 2, "height", texture.height() as f64) as u32,
        depth: optional_number_field(state, 2, "depth", 1.0) as u32,
        x: optional_number_field(state, 2, "x", 0.0) as u32,
        y: optional_number_field(state, 2, "y", 0.0) as u32,
        z: optional_number_field(state, 2, "z", 0.0) as u32,
        mip_level: optional_number_field(state, 2, "mipLevel", 0.0) as u32,
        layer: optional_number_field(state, 2, "layer", 0.0) as u32,
        bytes_per_row: optional_number_field(state, 2, "bytesPerRow", 0.0) as u32,
        rows_per_image: optional_number_field(state, 2, "rowsPerImage", 0.0) as u32,
    };
    if desc.mip_level >= texture.num_mipmaps() {
        state.error::<()>(format!(
            "upload: mipLevel {} out of range [0, {})",
            desc.mip_level,
            texture.num_mipmaps()
        ));
    }
    if desc.layer >= texture.depth_or_array_layers() {
        state.error::<()>(format!(
            "upload: layer {} out of range [0, {})",
            desc.layer,
            texture.depth_or_array_layers()
        ));
    }
    let mip_width = max(1, texture.width() >> desc.mip_level);
    let mip_height = max(1, texture.height() >> desc.mip_level);
    if desc.x > mip_width || desc.width > mip_width - desc.x {
        state.error::<()>(format!(
            "upload: x+width ({}+{}) exceeds mip {} width {mip_width}",
            desc.x, desc.width, desc.mip_level
        ));
    }
    if desc.y > mip_height || desc.height > mip_height - desc.y {
        state.error::<()>(format!(
            "upload: y+height ({}+{}) exceeds mip {} height {mip_height}",
            desc.y, desc.height, desc.mip_level
        ));
    }
    if desc.bytes_per_row == 0 {
        let bytes_per_texel = texture_format_bytes_per_texel(texture.format());
        if bytes_per_texel == 0 {
            state.error::<()>("upload: bytesPerRow must be provided for block-compressed formats");
        }
        desc.bytes_per_row = desc.width * bytes_per_texel;
    }
    if desc.rows_per_image == 0 {
        desc.rows_per_image = desc.height;
    }
    let required =
        desc.bytes_per_row as u64 * desc.rows_per_image as u64 * max(1, desc.depth) as u64;
    if (desc.data.len() as u64) < required {
        state.error::<()>(format!(
            "upload: data buffer is {} bytes but region requires {required} (bytesPerRow={} * rowsPerImage={} * depth={})",
            desc.data.len(), desc.bytes_per_row, desc.rows_per_image, max(1, desc.depth)
        ));
    }
    texture.upload(desc);
    0
}

fn gpu_texture_namecall(state: &mut LuaState) -> i32 {
    let (name, atom) = state.namecall_atom();
    match atom {
        LuaAtoms::View => gpu_texture_view(state),
        LuaAtoms::Upload => gpu_texture_upload(state),
        _ => state.error(format!(
            "{} is not a valid method of {}",
            name.unwrap_or_default(),
            ScriptedGPUTexture::LUA_NAME
        )),
    }
}

fn gpu_texture_index(state: &mut LuaState) -> i32 {
    let (key, atom) = state.string_atom(2);
    let texture = state.to_rive::<ScriptedGPUTexture>(1).texture.clone();
    match atom {
        LuaAtoms::Width => state.push_number(texture.width() as f64),
        LuaAtoms::Height => state.push_number(texture.height() as f64),
        LuaAtoms::Format => state.push_string(texture_format_string(texture.format())),
        _ => {
            return state.error(format!(
                "'{}' is not a valid index of GPUTexture",
                key.unwrap_or_default()
            ));
        }
    }
    1
}

fn gpu_texture_view_index(state: &mut LuaState) -> i32 {
    let (key, atom) = state.string_atom(2);
    if atom != LuaAtoms::Format {
        return state.error(format!(
            "'{}' is not a valid index of GPUTextureView",
            key.unwrap_or_default()
        ));
    }
    let format = state
        .to_rive::<ScriptedGPUTextureView>(1)
        .view
        .texture()
        .map(|texture| texture.format());
    match format {
        Some(format) => state.push_string(texture_format_string(format)),
        None => state.push_nil(),
    }
    1
}

fn gpu_sampler_construct(state: &mut LuaState) -> i32 {
    if ore_context(state).is_none() {
        state.error::<()>("GPU context not initialized");
    }
    let mut desc = SamplerDesc::default();
    if state.is_table(1) {
        if let Some(value) = optional_string_field(state, 1, "min") {
            desc.min_filter = filter(state, &value);
        }
        if let Some(value) = optional_string_field(state, 1, "mag") {
            desc.mag_filter = filter(state, &value);
        }
        if let Some(value) = optional_string_field(state, 1, "mipmap") {
            desc.mipmap_filter = filter(state, &value);
        }
        if let Some(value) = optional_string_field(state, 1, "wrapU") {
            desc.wrap_u = wrap_mode(state, &value);
        }
        if let Some(value) = optional_string_field(state, 1, "wrapV") {
            desc.wrap_v = wrap_mode(state, &value);
        }
        if let Some(value) = optional_string_field(state, 1, "wrapW") {
            desc.wrap_w = wrap_mode(state, &value);
        }
        if let Some(value) = optional_string_field(state, 1, "compare") {
            desc.compare = compare_function(state, &value);
        }
        desc.min_lod = optional_number_field(state, 1, "minLod", 0.0) as f32;
        desc.max_lod = optional_number_field(state, 1, "maxLod", 32.0) as f32;
        if desc.min_lod > desc.max_lod {
            state.error::<()>(format!(
                "GPUSampler.new: minLod ({}) > maxLod ({})",
                desc.min_lod, desc.max_lod
            ));
        }
        desc.max_anisotropy = optional_number_field(state, 1, "maxAnisotropy", 1.0) as u32;
        let anisotropy = desc.max_anisotropy;
        if !(1..=16).contains(&anisotropy) || !anisotropy.is_power_of_two() {
            state.error::<()>(format!(
                "GPUSampler.new: maxAnisotropy must be a power of two in [1, 16] (got {anisotropy})"
            ));
        }
        if anisotropy > 1 && !ore_context(state).unwrap().features().anisotropic_filtering {
            state.error::<()>(format!("GPUSampler.new: maxAnisotropy={anisotropy} requires anisotropicFiltering feature, which the active backend does not support"));
        }
    }
    let context = ore_context(state).unwrap();
    context.clear_last_error();
    let Some(sampler) = context.make_sampler(desc) else {
        let error = context.last_error();
        if error.is_empty() {
            state.error::<()>("GPUSampler.new: failed to create sampler");
        }
        state.error::<()>(format!("GPUSampler.new: {error}"));
        unreachable!()
    };
    state.new_rive(ScriptedGPUSampler { sampler });
    1
}

fn binding_kind(kind: ResourceKind) -> BindingKind {
    match kind {
        ResourceKind::UniformBuffer => BindingKind::UniformBuffer,
        ResourceKind::StorageBufferRo => BindingKind::StorageBufferRo,
        ResourceKind::StorageBufferRw => BindingKind::StorageBufferRw,
        ResourceKind::SampledTexture => BindingKind::SampledTexture,
        ResourceKind::StorageTexture => BindingKind::StorageTexture,
        ResourceKind::Sampler => BindingKind::Sampler,
        ResourceKind::ComparisonSampler => BindingKind::ComparisonSampler,
    }
}

fn view_dimension(dimension: TextureViewDim) -> TextureViewDimension {
    match dimension {
        TextureViewDim::Cube => TextureViewDimension::Cube,
        TextureViewDim::CubeArray => TextureViewDimension::CubeArray,
        TextureViewDim::D3 => TextureViewDimension::Texture3D,
        TextureViewDim::D2Array => TextureViewDimension::Array2D,
        TextureViewDim::D1 | TextureViewDim::D2 | TextureViewDim::Undefined => {
            TextureViewDimension::Texture2D
        }
    }
}

fn sample_type(sample: TextureSampleType) -> TextureSampleTypeLayout {
    match sample {
        TextureSampleType::UnfilterableFloat => TextureSampleTypeLayout::FloatUnfilterable,
        TextureSampleType::Depth => TextureSampleTypeLayout::Depth,
        TextureSampleType::Sint => TextureSampleTypeLayout::Sint,
        TextureSampleType::Uint => TextureSampleTypeLayout::Uint,
        TextureSampleType::Float | TextureSampleType::Undefined => {
            TextureSampleTypeLayout::FloatFilterable
        }
    }
}

fn entries_from_shader(
    shader: Option<&ShaderModule>,
    group: u32,
    dynamic_ubos: &[u32],
) -> Vec<BindGroupLayoutEntry> {
    let Some(shader) = shader else {
        return Vec::new();
    };
    shader
        .binding_map
        .iter()
        .filter(|entry| entry.group == group)
        .take(16)
        .map(|entry| {
            let kind = binding_kind(entry.kind);
            BindGroupLayoutEntry {
                binding: entry.binding,
                kind,
                visibility: StageVisibility {
                    vertex: entry.stage_mask.contains(StageMask::VERTEX),
                    fragment: entry.stage_mask.contains(StageMask::FRAGMENT),
                    compute: entry.stage_mask.contains(StageMask::COMPUTE),
                },
                has_dynamic_offset: kind == BindingKind::UniformBuffer
                    && dynamic_ubos.contains(&entry.binding),
                texture_view_dimension: view_dimension(entry.texture_view_dimension),
                texture_sample_type: sample_type(entry.texture_sample_type),
                texture_multisampled: entry.texture_multisampled,
                native_slot_vs: entry.native_slot(ShaderStage::Vertex),
                native_slot_fs: entry.native_slot(ShaderStage::Fragment),
            }
        })
        .collect()
}

fn gpu_bind_group_layout_construct(state: &mut LuaState) -> i32 {
    if ore_context(state).is_none() {
        state.error::<()>("GPU context not initialized");
    }
    state.check_type(1, LuaType::Table);
    let group_index = optional_number_field(state, 1, "groupIndex", 0.0) as u32;
    if group_index >= MAX_BIND_GROUPS {
        state.error::<()>(format!(
            "GPUBindGroupLayout.new: groupIndex must be in [0, {MAX_BIND_GROUPS})"
        ));
    }
    state.get_field(1, "shader");
    let shader = state.to_rive::<ScriptedShader>(-1);
    if !shader.has_module() {
        state.error::<()>("GPUBindGroupLayout.new: 'shader' must be a Shader with a loaded module");
    }
    let module = shader.vertex_module().cloned();
    state.pop(1);
    let mut dynamic = Vec::new();
    state.get_field(1, "dynamicUBOs");
    if state.is_table(-1) {
        for index in 1..=state.object_len(-1).min(16) {
            state.raw_get_i(-1, index);
            if state.is_number(-1) {
                dynamic.push(state.to_number(-1) as u32);
            }
            state.pop(1);
        }
    }
    state.pop(1);
    let entries = entries_from_shader(module.as_ref(), group_index, &dynamic);
    let context = ore_context(state).unwrap();
    let Some(layout) = context.make_bind_group_layout(BindGroupLayoutDesc {
        group_index,
        entries,
    }) else {
        let error = context.last_error().to_owned();
        context.clear_last_error();
        state.error::<()>(format!(
            "GPUBindGroupLayout.new: {}",
            if error.is_empty() { "failed" } else { &error }
        ));
        unreachable!()
    };
    state.new_rive(ScriptedGPUBindGroupLayout { layout });
    1
}

fn gpu_bind_group_construct(state: &mut LuaState) -> i32 {
    if ore_context(state).is_none() {
        state.error::<()>("GPU context not initialized");
    }
    state.check_type(1, LuaType::Table);
    state.get_field(1, "layout");
    let layout = state
        .try_to_rive::<ScriptedGPUBindGroupLayout>(-1)
        .map(|wrapper| wrapper.layout.clone())
        .unwrap_or_else(|| {
            state.error(
                "GPUBindGroup.new: 'layout' field is required and must be a GPUBindGroupLayout",
            )
        });
    state.pop(1);
    let mut ubos = Vec::new();
    state.get_field(1, "ubos");
    if state.is_table(-1) {
        for index in 1..=state.object_len(-1).min(8) {
            state.raw_get_i(-1, index);
            let entry = state.top();
            let slot = optional_number_field(state, entry, "slot", 0.0) as u32;
            if slot > 7 {
                state.error::<()>(format!(
                    "GPUBindGroup.new: UBO slot must be 0-7 (got {slot})"
                ));
            }
            state.get_field(entry, "buffer");
            let buffer = state.to_rive::<ScriptedGPUBuffer>(-1).buffer.clone();
            state.pop(1);
            ubos.push(BindGroupUboEntry {
                slot,
                buffer,
                offset: optional_number_field(state, entry, "offset", 0.0) as u32,
                size: optional_number_field(state, entry, "size", 0.0) as u32,
            });
            state.pop(1);
        }
    }
    state.pop(1);
    let mut textures = Vec::new();
    state.get_field(1, "textures");
    if state.is_table(-1) {
        for index in 1..=state.object_len(-1).min(8) {
            state.raw_get_i(-1, index);
            let entry = state.top();
            let slot = optional_number_field(state, entry, "slot", 0.0) as u32;
            if slot > 7 {
                state.error::<()>(format!(
                    "GPUBindGroup.new: texture slot must be 0-7 (got {slot})"
                ));
            }
            state.get_field(entry, "view");
            let view = state.to_rive::<ScriptedGPUTextureView>(-1).view.clone();
            state.pop(1);
            textures.push(BindGroupTextureEntry { slot, view });
            state.pop(1);
        }
    }
    state.pop(1);
    let mut samplers = Vec::new();
    state.get_field(1, "samplers");
    if state.is_table(-1) {
        for index in 1..=state.object_len(-1).min(8) {
            state.raw_get_i(-1, index);
            let entry = state.top();
            let slot = optional_number_field(state, entry, "slot", 0.0) as u32;
            if slot > 7 {
                state.error::<()>(format!(
                    "GPUBindGroup.new: sampler slot must be 0-7 (got {slot})"
                ));
            }
            state.get_field(entry, "sampler");
            let sampler = state.to_rive::<ScriptedGPUSampler>(-1).sampler.clone();
            state.pop(1);
            samplers.push(BindGroupSamplerEntry { slot, sampler });
            state.pop(1);
        }
    }
    state.pop(1);
    let context = ore_context(state).unwrap();
    context.clear_last_error();
    let Some(bind_group) = context.make_bind_group(BindGroupDesc {
        layout,
        ubos,
        textures,
        samplers,
    }) else {
        let error = context.last_error();
        if error.is_empty() {
            state.error::<()>("GPUBindGroup.new: failed to create bind group");
        }
        state.error::<()>(format!("GPUBindGroup.new: {error}"));
        unreachable!()
    };
    state.new_rive(ScriptedGPUBindGroup { bind_group });
    1
}

fn resolve_shader_entry(
    state: &mut LuaState,
    shader: &ScriptedShader,
    stage: u8,
    requested: Option<&str>,
    stage_name: &str,
) -> (ShaderModule, String) {
    let Some(entry) = shader.resolve_entry(stage, requested) else {
        let available = shader
            .entries
            .iter()
            .filter(|entry| entry.stage == stage)
            .map(|entry| entry.logical.as_str())
            .collect::<Vec<_>>()
            .join(", ");
        if let Some(requested) = requested.filter(|value| !value.is_empty()) {
            state.error::<()>(format!(
                "GPUPipeline.new: {stage_name} entry point '{requested}' not found (available: {})",
                if available.is_empty() {
                    "<none>"
                } else {
                    &available
                }
            ));
        }
        state.error::<()>(format!(
            "GPUPipeline.new: {stage_name} shader has no {stage_name} entry point"
        ));
        unreachable!()
    };
    (entry.module.clone(), entry.physical.clone())
}

fn resolve_stage_entry(
    state: &mut LuaState,
    value_index: i32,
    stage: u8,
    stage_name: &str,
) -> Option<(ScriptedShader, ShaderModule, String)> {
    if state.is_nil(value_index) {
        return None;
    }
    let (shader, requested) = if state.is_table(value_index) {
        state.get_field(value_index, "module");
        let shader = state.try_to_rive::<ScriptedShader>(-1).cloned();
        state.pop(1);
        (
            shader,
            optional_string_field(state, value_index, "entryPoint"),
        )
    } else {
        (
            state.try_to_rive::<ScriptedShader>(value_index).cloned(),
            None,
        )
    };
    let Some(shader) = shader.filter(ScriptedShader::has_module) else {
        state.error::<()>(format!("GPUPipeline.new: '{stage_name}' must be a Shader or {{ module = Shader, entryPoint = string? }}"));
        unreachable!()
    };
    let (module, physical) =
        resolve_shader_entry(state, &shader, stage, requested.as_deref(), stage_name);
    Some((shader, module, physical))
}

fn gpu_pipeline_construct(state: &mut LuaState) -> i32 {
    if ore_context(state).is_none() {
        state.error::<()>("GPU context not initialized");
    }
    state.check_type(1, LuaType::Table);
    state.get_field(1, "vertex");
    let Some((vertex_shader, vertex_module, vertex_entry_point)) =
        resolve_stage_entry(state, state.top(), 0, "vertex")
    else {
        state.error::<()>("GPUPipeline.new: 'vertex' is required");
        unreachable!()
    };
    state.pop(1);
    state.get_field(1, "fragment");
    let explicit_fragment = !state.is_nil(-1);
    let fragment = resolve_stage_entry(state, state.top(), 1, "fragment");
    state.pop(1);

    state.get_field(1, "vertexLayout");
    state.check_type(-1, LuaType::Table);
    let mut vertex_buffers = Vec::new();
    let mut total_attributes = 0;
    for buffer_index in 1..=state.object_len(-1).min(8) {
        state.raw_get_i(-1, buffer_index);
        let table = state.top();
        let stride = optional_number_field(state, table, "stride", 0.0) as u32;
        let step_mode =
            if optional_string_field(state, table, "stepMode").as_deref() == Some("instance") {
                VertexStepMode::Instance
            } else {
                VertexStepMode::Vertex
            };
        state.get_field(table, "attributes");
        let mut attributes = Vec::new();
        for attribute_index in 1..=state.object_len(-1) {
            if total_attributes == 32 {
                break;
            }
            state.raw_get_i(-1, attribute_index);
            let attribute_table = state.top();
            let format = optional_string_field(state, attribute_table, "format")
                .map(|format| vertex_format(state, &format))
                .unwrap_or_default();
            attributes.push(VertexAttribute {
                format,
                shader_slot: optional_number_field(state, attribute_table, "slot", 0.0) as u32,
                offset: optional_number_field(state, attribute_table, "offset", 0.0) as u32,
            });
            total_attributes += 1;
            state.pop(1);
        }
        state.pop(1);
        state.pop(1);
        vertex_buffers.push(VertexBufferLayout {
            stride,
            step_mode,
            attributes,
        });
    }
    state.pop(1);

    let mut color_targets = Vec::new();
    state.get_field(1, "colorTargets");
    if state.is_table(-1) {
        let count = state.object_len(-1);
        if count > MAX_COLOR_TARGETS {
            state.error::<()>(format!(
                "GPUPipeline.new: colorTargets count {count} exceeds maximum of {MAX_COLOR_TARGETS}"
            ));
        }
        for index in 1..=count {
            state.raw_get_i(-1, index);
            let table = state.top();
            let mut target = ColorTargetState::default();
            if let Some(format) = optional_string_field(state, table, "format") {
                target.format = texture_format(state, &format);
            }
            if let Some(mask) = optional_string_field(state, table, "writeMask") {
                target.write_mask = write_mask(state, Some(&mask));
            }
            state.get_field(table, "blend");
            if state.is_table(-1) {
                target.blend_enabled = true;
                let blend_table = state.top();
                if let Some(value) = optional_string_field(state, blend_table, "srcColor") {
                    target.blend.src_color = blend_factor(state, &value);
                }
                if let Some(value) = optional_string_field(state, blend_table, "dstColor") {
                    target.blend.dst_color = blend_factor(state, &value);
                }
                if let Some(value) = optional_string_field(state, blend_table, "colorOp") {
                    target.blend.color_op = blend_op(state, &value);
                }
                if let Some(value) = optional_string_field(state, blend_table, "srcAlpha") {
                    target.blend.src_alpha = blend_factor(state, &value);
                }
                if let Some(value) = optional_string_field(state, blend_table, "dstAlpha") {
                    target.blend.dst_alpha = blend_factor(state, &value);
                }
                if let Some(value) = optional_string_field(state, blend_table, "alphaOp") {
                    target.blend.alpha_op = blend_op(state, &value);
                }
            }
            state.pop(1);
            state.pop(1);
            color_targets.push(target);
        }
    }
    state.pop(1);

    let (fragment_module, fragment_entry_point) = match fragment {
        Some((_, module, entry)) => (Some(module), Some(entry)),
        None if !color_targets.is_empty() => {
            let (module, entry) = resolve_shader_entry(state, &vertex_shader, 1, None, "fragment");
            (Some(module), Some(entry))
        }
        None => (None, None),
    };

    let mut depth_stencil = DepthStencilState::default();
    state.get_field(1, "depthStencil");
    if state.is_table(-1) {
        let table = state.top();
        if let Some(value) = optional_string_field(state, table, "format") {
            depth_stencil.format = texture_format(state, &value);
        }
        if let Some(value) = optional_string_field(state, table, "compare") {
            depth_stencil.depth_compare = compare_function(state, &value);
        }
        depth_stencil.depth_write_enabled = optional_bool_field(state, table, "write", false);
        depth_stencil.depth_bias = optional_number_field(state, table, "depthBias", 0.0) as i32;
        depth_stencil.depth_bias_slope_scale =
            optional_number_field(state, table, "depthBiasSlopeScale", 0.0) as f32;
        depth_stencil.depth_bias_clamp =
            optional_number_field(state, table, "depthBiasClamp", 0.0) as f32;
    }
    state.pop(1);
    let mut stencil_front = StencilFaceState::default();
    let mut stencil_back = StencilFaceState::default();
    state.get_field(1, "stencilFront");
    stencil_face(state, state.top(), &mut stencil_front);
    state.pop(1);
    state.get_field(1, "stencilBack");
    stencil_face(state, state.top(), &mut stencil_back);
    state.pop(1);
    let stencil_read_mask = optional_number_field(state, 1, "stencilReadMask", 255.0) as u8;
    let stencil_write_mask = optional_number_field(state, 1, "stencilWriteMask", 255.0) as u8;

    let mut automatic_layouts = Vec::new();
    let mut layouts = Vec::new();
    state.get_field(1, "bindGroupLayouts");
    let explicit_layouts = state.is_table(-1);
    if explicit_layouts {
        let count = state.object_len(-1);
        if count > MAX_BIND_GROUPS {
            state.error::<()>(format!("GPUPipeline.new: bindGroupLayouts count {count} exceeds maximum of {MAX_BIND_GROUPS}"));
        }
        for index in 1..=count {
            state.raw_get_i(-1, index);
            layouts.push(
                state
                    .try_to_rive::<ScriptedGPUBindGroupLayout>(-1)
                    .map(|value| value.layout.clone()),
            );
            state.pop(1);
        }
    }
    state.pop(1);
    if !explicit_layouts {
        let map = &vertex_shader.vertex_module().unwrap().binding_map;
        let groups = map
            .iter()
            .filter_map(|entry| (entry.group < MAX_BIND_GROUPS).then_some(entry.group))
            .collect::<BTreeSet<_>>();
        let count = groups.last().map_or(0, |group| group + 1);
        automatic_layouts.resize(count as usize, None);
        layouts.resize(count as usize, None);
        for group in groups {
            let entries = entries_from_shader(vertex_shader.vertex_module(), group, &[]);
            let layout = ore_context(state)
                .unwrap()
                .make_bind_group_layout(BindGroupLayoutDesc {
                    group_index: group,
                    entries,
                });
            automatic_layouts[group as usize] = layout.clone();
            layouts[group as usize] = layout;
        }
    }
    let cull = optional_string_field(state, 1, "cullMode")
        .map(|value| cull_mode(state, &value))
        .unwrap_or_default();
    let face_winding = optional_string_field(state, 1, "winding")
        .map(|value| winding(state, &value))
        .unwrap_or_default();
    let primitive_topology = optional_string_field(state, 1, "topology")
        .map(|value| topology(state, &value))
        .unwrap_or_default();
    let sample_count = optional_number_field(state, 1, "sampleCount", 1.0) as u32;
    let desc = PipelineDesc {
        vertex_module,
        vertex_entry_point,
        fragment_module,
        fragment_entry_point,
        vertex_buffers: vertex_buffers.clone(),
        color_targets,
        depth_stencil,
        stencil_front,
        stencil_back,
        stencil_read_mask,
        stencil_write_mask,
        bind_group_layouts: layouts,
        cull_mode: cull,
        winding: face_winding,
        topology: primitive_topology,
        sample_count,
    };
    let context = ore_context(state).unwrap();
    let pipeline = context.make_pipeline(desc).unwrap_or_else(|error| {
        if error.is_empty() {
            state.error("GPUPipeline.new: failed to create pipeline")
        } else {
            state.error(format!("GPUPipeline.new: {error}"))
        }
    });
    state.new_rive(ScriptedGPUPipeline {
        pipeline,
        sample_count,
        owned_vertex_layout_data: vertex_buffers,
        auto_bind_group_layouts: automatic_layouts,
    });
    1
}

fn gpu_pipeline_get_bind_group_layout(state: &mut LuaState) -> i32 {
    let group = state.check_unsigned(2);
    let pipeline = state.to_rive::<ScriptedGPUPipeline>(1);
    if pipeline.auto_bind_group_layouts.is_empty() {
        state.error::<()>("getBindGroupLayout: pipeline was built with explicit bindGroupLayouts; reuse the layout you supplied");
    }
    let Some(layout) = pipeline
        .auto_bind_group_layouts
        .get(group as usize)
        .and_then(Clone::clone)
    else {
        state.error::<()>(format!(
            "getBindGroupLayout: group {group} not present in shader"
        ));
        unreachable!()
    };
    state.new_rive(ScriptedGPUBindGroupLayout { layout });
    1
}

fn gpu_pipeline_namecall(state: &mut LuaState) -> i32 {
    let (method, _) = state.namecall_atom();
    match method.as_deref() {
        Some("getBindGroupLayout") => gpu_pipeline_get_bind_group_layout(state),
        Some(method) => state.error(format!("GPUPipeline: unknown method '{method}'")),
        None => state.error("GPUPipeline: no method specified"),
    }
}

fn validate_render_pass(state: &mut LuaState, pass: &ScriptedGPURenderPass) {
    if pass.finished
        || pass.pass.is_none()
        || pass.pass.as_ref().is_some_and(|pass| pass.is_finished())
    {
        state.error::<()>("render pass expired — already finished, or auto-finished by a subsequent beginRenderPass");
    }
}

fn validate_pipeline_set(state: &mut LuaState, pass: &ScriptedGPURenderPass) {
    if !pass.pipeline_set {
        state.error::<()>("setPipeline must be called before draw/setVertexBuffer/setBindGroup");
    }
}

fn gpu_render_pass_set_pipeline(state: &mut LuaState) -> i32 {
    let pipeline = state.to_rive::<ScriptedGPUPipeline>(2).clone();
    let pass = state.to_rive_mut::<ScriptedGPURenderPass>(1);
    validate_render_pass(state, pass);
    if pipeline.sample_count != pass.sample_count {
        state.error::<()>(format!("pipeline sampleCount ({}) does not match render pass sampleCount ({}) — recreate the pipeline with matching sampleCount", pipeline.sample_count, pass.sample_count));
    }
    if let Some(context) = ore_context(state) {
        context.clear_last_error();
    }
    pass.pass.as_mut().unwrap().set_pipeline(&pipeline.pipeline);
    if let Some(context) = ore_context(state) {
        if !context.last_error().is_empty() {
            state.error::<()>(format!("setPipeline: {}", context.last_error()));
        }
    }
    pass.pipeline_set = true;
    0
}

fn gpu_render_pass_set_vertex_buffer(state: &mut LuaState) -> i32 {
    let slot = state.check_unsigned(2);
    if slot > 7 {
        state.error::<()>(format!("setVertexBuffer: slot must be 0-7 (got {slot})"));
    }
    let buffer = state.to_rive::<ScriptedGPUBuffer>(3).buffer.clone();
    let pass = state.to_rive_mut::<ScriptedGPURenderPass>(1);
    validate_render_pass(state, pass);
    pass.pass.as_mut().unwrap().set_vertex_buffer(slot, &buffer);
    0
}

fn gpu_render_pass_set_index_buffer(state: &mut LuaState) -> i32 {
    let buffer = state.to_rive::<ScriptedGPUBuffer>(2).buffer.clone();
    let format = if state.string_or(3, "uint16") == "uint32" {
        IndexFormat::Uint32
    } else {
        IndexFormat::Uint16
    };
    let pass = state.to_rive_mut::<ScriptedGPURenderPass>(1);
    validate_render_pass(state, pass);
    pass.pass
        .as_mut()
        .unwrap()
        .set_index_buffer(&buffer, format);
    0
}

fn gpu_render_pass_set_bind_group(state: &mut LuaState) -> i32 {
    let group = state.check_unsigned(2);
    if group >= MAX_BIND_GROUPS {
        state.error::<()>(format!(
            "setBindGroup: groupIndex must be in [0, {MAX_BIND_GROUPS}) (got {group})"
        ));
    }
    let bind_group = state.to_rive::<ScriptedGPUBindGroup>(3).bind_group.clone();
    let mut offsets = Vec::new();
    if state.is_table(4) {
        let count = state.object_len(4);
        if count > 8 {
            state.error::<()>(format!(
                "setBindGroup: dynamicOffsets count {count} exceeds maximum of 8"
            ));
        }
        for index in 1..=count {
            state.raw_get_i(4, index);
            let offset = state.to_number(-1) as u32;
            state.pop(1);
            if offset % 256 != 0 {
                state.error::<()>(format!("setBindGroup: dynamicOffsets[{}] = {offset} is not a multiple of 256 (alignment requirement)", index - 1));
            }
            offsets.push(offset);
        }
    }
    if offsets.len() != bind_group.dynamic_offset_count() as usize {
        state.error::<()>(format!("setBindGroup: dynamicOffsets count {} does not match the BindGroup's declared dynamic UBO count {}", offsets.len(), bind_group.dynamic_offset_count()));
    }
    let pass = state.to_rive_mut::<ScriptedGPURenderPass>(1);
    validate_render_pass(state, pass);
    pass.pass
        .as_mut()
        .unwrap()
        .set_bind_group(group, &bind_group, &offsets);
    0
}

fn gpu_render_pass_set_viewport(state: &mut LuaState) -> i32 {
    let values = [
        state.check_number(2) as f32,
        state.check_number(3) as f32,
        state.check_number(4) as f32,
        state.check_number(5) as f32,
    ];
    let pass = state.to_rive_mut::<ScriptedGPURenderPass>(1);
    validate_render_pass(state, pass);
    pass.pass
        .as_mut()
        .unwrap()
        .set_viewport(values[0], values[1], values[2], values[3]);
    0
}

fn gpu_render_pass_set_scissor_rect(state: &mut LuaState) -> i32 {
    let values = [
        state.check_unsigned(2),
        state.check_unsigned(3),
        state.check_unsigned(4),
        state.check_unsigned(5),
    ];
    let pass = state.to_rive_mut::<ScriptedGPURenderPass>(1);
    validate_render_pass(state, pass);
    pass.pass
        .as_mut()
        .unwrap()
        .set_scissor_rect(values[0], values[1], values[2], values[3]);
    0
}

fn gpu_render_pass_set_stencil_reference(state: &mut LuaState) -> i32 {
    let reference = state.check_unsigned(2);
    let pass = state.to_rive_mut::<ScriptedGPURenderPass>(1);
    validate_render_pass(state, pass);
    pass.pass.as_mut().unwrap().set_stencil_reference(reference);
    0
}

fn gpu_render_pass_set_blend_color(state: &mut LuaState) -> i32 {
    let values = [
        state.check_number(2) as f32,
        state.check_number(3) as f32,
        state.check_number(4) as f32,
        state.check_number(5) as f32,
    ];
    let pass = state.to_rive_mut::<ScriptedGPURenderPass>(1);
    validate_render_pass(state, pass);
    pass.pass
        .as_mut()
        .unwrap()
        .set_blend_color(values[0], values[1], values[2], values[3]);
    0
}

fn gpu_render_pass_draw(state: &mut LuaState) -> i32 {
    let vertex_count = state.check_unsigned(2);
    let instance_count = state.number_or(3, 1.0) as u32;
    let first_vertex = state.number_or(4, 0.0) as u32;
    let first_instance = state.number_or(5, 0.0) as u32;
    if first_instance > 0 && !ore_context(state).unwrap().features().draw_base_instance {
        state.error::<()>(format!("draw: firstInstance={first_instance} requires the drawBaseInstance feature, which the active backend does not support"));
    }
    let pass = state.to_rive_mut::<ScriptedGPURenderPass>(1);
    validate_render_pass(state, pass);
    validate_pipeline_set(state, pass);
    pass.pass
        .as_mut()
        .unwrap()
        .draw(vertex_count, instance_count, first_vertex, first_instance);
    pass.draw_call_count += 1;
    0
}

fn gpu_render_pass_draw_indexed(state: &mut LuaState) -> i32 {
    let index_count = state.check_unsigned(2);
    let instance_count = state.number_or(3, 1.0) as u32;
    let first_index = state.number_or(4, 0.0) as u32;
    let base_vertex = state.integer_or(5, 0) as i32;
    let first_instance = state.number_or(6, 0.0) as u32;
    let base_instance = ore_context(state).unwrap().features().draw_base_instance;
    if base_vertex != 0 && !base_instance {
        state.error::<()>(format!("drawIndexed: baseVertex={base_vertex} requires the drawBaseInstance feature, which the active backend does not support"));
    }
    if first_instance > 0 && !base_instance {
        state.error::<()>(format!("drawIndexed: firstInstance={first_instance} requires the drawBaseInstance feature, which the active backend does not support"));
    }
    let pass = state.to_rive_mut::<ScriptedGPURenderPass>(1);
    validate_render_pass(state, pass);
    validate_pipeline_set(state, pass);
    pass.pass.as_mut().unwrap().draw_indexed(
        index_count,
        instance_count,
        first_index,
        base_vertex,
        first_instance,
    );
    pass.draw_call_count += 1;
    0
}

fn gpu_render_pass_finish(state: &mut LuaState) -> i32 {
    let pass = state.to_rive_mut::<ScriptedGPURenderPass>(1);
    validate_render_pass(state, pass);
    pass.pass.as_mut().unwrap().finish();
    pass.finished = true;
    if let (Some(pass), Some(context)) = (pass.pass.as_deref(), ore_context(state)) {
        // Same-call identity check between two live shared borrows. Neither
        // reference nor an address derived from it is retained.
        if context
            .active_render_pass()
            .is_some_and(|active| std::ptr::eq(active, pass))
        {
            context.set_active_render_pass(None);
        }
    }
    0
}

fn gpu_render_pass_namecall(state: &mut LuaState) -> i32 {
    let (name, atom) = state.namecall_atom();
    match atom {
        LuaAtoms::SetPipeline => gpu_render_pass_set_pipeline(state),
        LuaAtoms::SetVertexBuffer => gpu_render_pass_set_vertex_buffer(state),
        LuaAtoms::SetIndexBuffer => gpu_render_pass_set_index_buffer(state),
        LuaAtoms::SetBindGroup => gpu_render_pass_set_bind_group(state),
        LuaAtoms::SetViewport => gpu_render_pass_set_viewport(state),
        LuaAtoms::SetScissorRect => gpu_render_pass_set_scissor_rect(state),
        LuaAtoms::SetStencilReference => gpu_render_pass_set_stencil_reference(state),
        LuaAtoms::SetBlendColor => gpu_render_pass_set_blend_color(state),
        LuaAtoms::Draw => gpu_render_pass_draw(state),
        LuaAtoms::DrawIndexed => gpu_render_pass_draw_indexed(state),
        LuaAtoms::Finish => gpu_render_pass_finish(state),
        _ => state.error(format!(
            "{} is not a valid method of {}",
            name.unwrap_or_default(),
            ScriptedGPURenderPass::LUA_NAME
        )),
    }
}

impl Drop for ScriptedGPUCanvas {
    fn drop(&mut self) {
        if let (Some(state), Some(reference)) = (self.lua_state.as_mut(), self.image_ref.take()) {
            state.unref(reference);
        }
    }
}

impl Drop for ScriptedGPURenderPass {
    fn drop(&mut self) {
        if let (Some(context), Some(pass)) = (self.context.as_deref_mut(), self.pass.as_deref()) {
            if context
                .active_render_pass()
                .is_some_and(|active| ptr::eq(active, pass))
            {
                context.set_active_render_pass(None);
            }
        }
    }
}

impl Drop for ScriptedCanvas {
    fn drop(&mut self) {
        self.rive_renderer = None;
        if let Some(state) = self.lua_state.as_mut() {
            if let Some(reference) = self.renderer_ref.take() {
                state.unref(reference);
            }
            if let Some(reference) = self.image_ref.take() {
                state.unref(reference);
            }
        }
    }
}

fn load_op(value: &str) -> LoadOp {
    if value == "load" {
        LoadOp::Load
    } else {
        LoadOp::Clear
    }
}

fn store_op(value: &str) -> StoreOp {
    if value == "discard" {
        StoreOp::Discard
    } else {
        StoreOp::Store
    }
}

pub fn gpu_canvas_begin_render_pass(state: &mut LuaState) -> i32 {
    let canvas = state.to_rive::<ScriptedGPUCanvas>(1).clone();
    if ore_context(state).is_none() {
        state.error::<()>("GPUCanvas:beginRenderPass() requires a GPU context");
    }
    if !state
        .thread_data::<dyn ScriptingContext>()
        .canvas_drawing_phase()
    {
        state.error::<()>("GPUCanvas:beginRenderPass() called outside drawing phase");
    }
    state.check_type(2, LuaType::Table);
    let mut desc = RenderPassDesc::default();
    let mut pass_sample_count = None;
    let mut sample_count_source = String::new();
    let mut record_sample_count = |state: &mut LuaState, count: u32, label: &str| {
        if let Some(previous) = pass_sample_count {
            if count != previous {
                state.error::<()>(format!("beginRenderPass: {label} sampleCount ({count}) does not match {sample_count_source} sampleCount ({previous}). All render-pass attachments must share one sampleCount."));
            }
        } else {
            pass_sample_count = Some(count);
            sample_count_source = label.to_owned();
        }
    };
    state.get_field(2, "color");
    if state.is_table(-1) {
        for index in 1..=4 {
            state.raw_get_i(-1, index);
            if !state.is_table(-1) {
                state.pop(1);
                break;
            }
            let table = state.top();
            state.get_field(table, "view");
            let view = if state.is_nil(-1) {
                canvas.color_view.clone().unwrap_or_else(|| state.error(format!("beginRenderPass: color[{index}].view omitted but the receiving canvas has no backing texture (zero-sized). Call canvas:resize(w, h) before drawing, or pass an explicit view.")))
            } else {
                state
                    .try_to_rive::<ScriptedGPUTextureView>(-1)
                    .filter(|view| view.view.is_valid())
                    .map(|view| view.view.clone())
                    .unwrap_or_else(|| {
                        state.error(format!(
                            "beginRenderPass: color[{index}].view is not a valid GPUTextureView"
                        ))
                    })
            };
            record_sample_count(
                state,
                view.texture().unwrap().sample_count(),
                &format!("color[{index}]"),
            );
            state.pop(1);
            state.get_field(table, "resolveTarget");
            let resolve_target = if state.is_nil(-1) {
                None
            } else {
                let target = state
                    .try_to_rive::<ScriptedGPUTextureView>(-1)
                    .filter(|value| value.view.is_valid())
                    .map(|value| value.view.clone());
                if let Some(target) = target.as_ref() {
                    let source_texture = view.texture().unwrap();
                    let target_texture = target.texture().unwrap();
                    if source_texture.sample_count() == 1 {
                        state.error::<()>(format!("beginRenderPass: color[{index}].resolveTarget is meaningless when the source `view` is single-sampled — drop it, or use an MSAA texture as `view`"));
                    }
                    if target_texture.format() != source_texture.format() {
                        state.error::<()>(format!("beginRenderPass: resolveTarget format '{}' does not match MSAA attachment format '{}' — resolve requires identical formats. Use canvas.format to match your pipeline and textures.", texture_format_string(target_texture.format()), texture_format_string(source_texture.format())));
                    }
                    if target_texture.sample_count() != 1 {
                        state.error::<()>(format!("beginRenderPass: color[{index}].resolveTarget must have sampleCount=1 (got {})", target_texture.sample_count()));
                    }
                }
                target
            };
            state.pop(1);
            let attachment_load = optional_string_field(state, table, "loadOp")
                .map_or(LoadOp::Clear, |value| load_op(&value));
            let Some(attachment_store) =
                optional_string_field(state, table, "storeOp").map(|value| store_op(&value))
            else {
                state.error::<()>(format!("beginRenderPass: color[{index}].storeOp is required — use 'discard' for MSAA color (after resolve) or 'store' to keep the rendered output"));
                unreachable!()
            };
            let mut clear_color = ColorF::default();
            state.get_field(table, "clearColor");
            if state.is_table(-1) {
                let values = (1..=4)
                    .map(|component| {
                        state.raw_get_i(-1, component);
                        let value = state.to_number(-1) as f32;
                        state.pop(1);
                        value
                    })
                    .collect::<Vec<_>>();
                clear_color = ColorF {
                    r: values[0],
                    g: values[1],
                    b: values[2],
                    a: values[3],
                };
            }
            state.pop(1);
            desc.color_attachments.push(RenderPassColorAttachment {
                view,
                resolve_target,
                load_op: attachment_load,
                store_op: attachment_store,
                clear_color,
            });
            state.pop(1);
        }
    }
    state.pop(1);
    state.get_field(2, "depthStencil");
    if state.is_table(-1) {
        let table = state.top();
        state.get_field(table, "view");
        if state.is_nil(-1) {
            state.error::<()>(
                "beginRenderPass: depthStencil.view is required — pass GPUTexture:view()",
            );
        }
        let view = state
            .try_to_rive::<ScriptedGPUTextureView>(-1)
            .filter(|value| value.view.is_valid())
            .map(|value| value.view.clone())
            .unwrap_or_else(|| {
                state.error("beginRenderPass: depthStencil.view is not a valid GPUTextureView")
            });
        record_sample_count(
            state,
            view.texture().unwrap().sample_count(),
            "depthStencil",
        );
        state.pop(1);
        let depth_load_op = optional_string_field(state, table, "depthLoadOp")
            .map_or(LoadOp::Clear, |value| load_op(&value));
        let Some(depth_store_op) =
            optional_string_field(state, table, "depthStoreOp").map(|value| store_op(&value))
        else {
            state.error::<()>("beginRenderPass: depthStencil.depthStoreOp is required — use 'discard' for transient/MSAA depth or 'store' if you need to read it later");
            unreachable!()
        };
        desc.depth_stencil = Some(RenderPassDepthStencilAttachment {
            view,
            depth_load_op,
            depth_store_op,
            depth_clear_value: optional_number_field(state, table, "depthClearValue", 1.0) as f32,
        });
    }
    state.pop(1);
    if desc.color_attachments.is_empty() && desc.depth_stencil.is_none() {
        state.error::<()>("beginRenderPass: descriptor must include at least one color attachment or a depthStencil attachment");
    }
    let context = ore_context(state).unwrap();
    if context
        .active_render_pass()
        .is_some_and(|pass| !pass.is_finished())
    {
        context.active_render_pass_mut().unwrap().finish();
        context.set_active_render_pass(None);
    }
    let pass = context.begin_render_pass(desc);
    context.set_active_render_pass(Some(pass.as_ref()));
    state.new_rive(ScriptedGPURenderPass {
        pass: Some(pass),
        context: Some(context),
        finished: false,
        pipeline_set: false,
        sample_count: pass_sample_count.unwrap_or(1).max(1),
        label: String::new(),
        draw_call_count: 0,
    });
    1
}

fn gpu_canvas_resize(state: &mut LuaState) -> i32 {
    let width = state.check_unsigned(2);
    let height = state.check_unsigned(3);
    let canvas = state.to_rive_mut::<ScriptedGPUCanvas>(1);
    if canvas.render_context.is_none() {
        state.error::<()>("GPUCanvas: renderCtx not initialized");
    }
    if ore_context(state).is_none() {
        state.error::<()>("GPUCanvas: GPU context not initialized");
    }
    if width == 0 || height == 0 {
        if let (Some(lua), Some(reference)) = (canvas.lua_state.as_mut(), canvas.image_ref.take()) {
            lua.unref(reference);
        }
        canvas.canvas = None;
        canvas.color_view = None;
        return 0;
    }
    let new_canvas = canvas
        .render_context
        .as_mut()
        .unwrap()
        .make_render_canvas(width, height)
        .unwrap_or_else(|| state.error("GPUCanvas:resize() failed to create RenderCanvas"));
    let new_view = ore_context(state)
        .unwrap()
        .wrap_canvas_texture(&new_canvas)
        .unwrap_or_else(|| state.error("GPUCanvas:resize() failed to wrap canvas texture"));
    if let (Some(lua), Some(reference)) = (canvas.lua_state.as_mut(), canvas.image_ref.take()) {
        lua.unref(reference);
    }
    canvas.canvas = Some(new_canvas);
    canvas.color_view = Some(new_view);
    let image = ScriptedImage {
        image: Some(canvas.canvas.as_ref().unwrap().render_image().clone()),
        ..Default::default()
    };
    state.new_rive(image);
    canvas.image_ref = Some(state.create_ref(-1));
    state.pop(1);
    0
}

fn gpu_canvas_color_view(state: &mut LuaState) -> i32 {
    let Some(view) = state.to_rive::<ScriptedGPUCanvas>(1).color_view.clone() else {
        state.error::<()>(
            "GPUCanvas:colorView() called on a zero-sized canvas; call canvas:resize(w, h) first",
        );
        unreachable!()
    };
    state.new_rive(ScriptedGPUTextureView {
        view,
        retained_image: None,
    });
    1
}

fn gpu_canvas_index(state: &mut LuaState) -> i32 {
    let (key, atom) = state.string_atom(2);
    let canvas = state.to_rive::<ScriptedGPUCanvas>(1);
    match atom {
        LuaAtoms::Image => match canvas.image_ref {
            Some(reference) => state.push_ref(reference),
            None => state.push_nil(),
        },
        LuaAtoms::Width => {
            state.push_number(canvas.canvas.as_ref().map_or(0, RenderCanvas::width) as f64)
        }
        LuaAtoms::Height => {
            state.push_number(canvas.canvas.as_ref().map_or(0, RenderCanvas::height) as f64)
        }
        LuaAtoms::Format => state.push_string(texture_format_string(
            canvas
                .color_view
                .as_ref()
                .and_then(TextureView::texture)
                .map_or(TextureFormat::Rgba8Unorm, Texture::format),
        )),
        _ => {
            return state.error(format!(
                "'{}' is not a valid index of GPUCanvas",
                key.unwrap_or_default()
            ));
        }
    }
    1
}

fn gpu_canvas_namecall(state: &mut LuaState) -> i32 {
    let (name, atom) = state.namecall_atom();
    match atom {
        LuaAtoms::ColorView => gpu_canvas_color_view(state),
        LuaAtoms::Resize => gpu_canvas_resize(state),
        LuaAtoms::BeginRenderPass => gpu_canvas_begin_render_pass(state),
        _ => state.error(format!(
            "{} is not a valid method of {}",
            name.unwrap_or_default(),
            ScriptedGPUCanvas::LUA_NAME
        )),
    }
}

fn canvas_resize(state: &mut LuaState) -> i32 {
    let width = state.check_unsigned(2);
    let height = state.check_unsigned(3);
    let canvas = state.to_rive_mut::<ScriptedCanvas>(1);
    if canvas.render_context.is_none() {
        state.error::<()>("Canvas: renderCtx not initialized");
    }
    if canvas.canvas_state != CanvasState::Idle {
        state.error::<()>("Canvas:resize() called during an active frame");
    }
    if width == 0 || height == 0 {
        if let (Some(lua), Some(reference)) = (canvas.lua_state.as_mut(), canvas.image_ref.take()) {
            lua.unref(reference);
        }
        canvas.canvas = None;
        return 0;
    }
    let new_canvas = canvas
        .render_context
        .as_mut()
        .unwrap()
        .make_render_canvas(width, height)
        .unwrap_or_else(|| state.error("Canvas:resize() failed to create RenderCanvas"));
    if let (Some(lua), Some(reference)) = (canvas.lua_state.as_mut(), canvas.image_ref.take()) {
        lua.unref(reference);
    }
    canvas.canvas = Some(new_canvas);
    state.new_rive(ScriptedImage {
        image: Some(canvas.canvas.as_ref().unwrap().render_image().clone()),
        ..Default::default()
    });
    canvas.image_ref = Some(state.create_ref(-1));
    state.pop(1);
    0
}

fn canvas_begin_frame(state: &mut LuaState) -> i32 {
    let canvas = state.to_rive_mut::<ScriptedCanvas>(1);
    if canvas.render_context.is_none() {
        state.error::<()>("Canvas: renderCtx not initialized");
    }
    if !state
        .thread_data::<dyn ScriptingContext>()
        .canvas_drawing_phase()
    {
        state.error::<()>("Canvas:beginFrame() called outside drawing phase");
    }
    if canvas.canvas_state != CanvasState::Idle {
        state.error::<()>("Canvas:beginFrame() called during an active frame");
    }
    if canvas.canvas.is_none() {
        state.error::<()>(
            "Canvas:beginFrame() called on a zero-sized canvas; call canvas:resize(w, h) first",
        );
    }
    let target = canvas.canvas.as_ref().unwrap();
    let clear_color = if state.top() >= 2 && state.is_table(2) {
        optional_number_field(state, 2, "clearColor", 0.0) as ColorInt
    } else {
        0
    };
    canvas
        .render_context
        .as_mut()
        .unwrap()
        .begin_frame(FrameDescriptor {
            render_target_width: target.width(),
            render_target_height: target.height(),
            load_action: FrameLoadAction::Clear,
            clear_color,
        });
    canvas.rive_renderer = Some(RiveRenderer::new(canvas.render_context.as_mut().unwrap()));
    canvas.canvas_state = CanvasState::Rendering;
    state.new_rive(ScriptedRenderer::new_non_owning(
        canvas.rive_renderer.as_mut().unwrap(),
    ));
    state.push_value(-1);
    canvas.renderer_ref = Some(state.create_ref(-1));
    state.pop(1);
    1
}

fn canvas_end_frame(state: &mut LuaState) -> i32 {
    let canvas = state.to_rive_mut::<ScriptedCanvas>(1);
    if canvas.canvas_state != CanvasState::Rendering {
        state.error::<()>("Canvas:endFrame() called without beginFrame()");
    }
    if let Some(reference) = canvas.renderer_ref.take() {
        state.push_ref(reference);
        if !state.is_nil(-1) {
            if let Some(renderer) = state.try_to_rive_mut::<ScriptedRenderer>(-1) {
                renderer.end();
            }
        }
        state.pop(1);
        if let Some(lua) = canvas.lua_state.as_mut() {
            lua.unref(reference);
        }
    }
    let render_context = canvas.render_context.as_mut().unwrap();
    let command_buffer = render_context.implementation().make_command_buffer();
    render_context.flush(FlushResources {
        render_target: canvas.canvas.as_ref().unwrap().render_target().clone(),
        external_command_buffer: Some(command_buffer.clone()),
    });
    render_context
        .implementation()
        .commit_command_buffer(command_buffer);
    canvas.rive_renderer = None;
    canvas.canvas_state = CanvasState::Idle;
    0
}

fn canvas_index(state: &mut LuaState) -> i32 {
    let (key, atom) = state.string_atom(2);
    let canvas = state.to_rive::<ScriptedCanvas>(1);
    match atom {
        LuaAtoms::Image => match canvas.image_ref {
            Some(reference) => state.push_ref(reference),
            None => state.push_nil(),
        },
        LuaAtoms::Width => {
            state.push_number(canvas.canvas.as_ref().map_or(0, RenderCanvas::width) as f64)
        }
        LuaAtoms::Height => {
            state.push_number(canvas.canvas.as_ref().map_or(0, RenderCanvas::height) as f64)
        }
        _ => {
            return state.error(format!(
                "'{}' is not a valid index of Canvas",
                key.unwrap_or_default()
            ));
        }
    }
    1
}

fn canvas_namecall(state: &mut LuaState) -> i32 {
    let (name, atom) = state.namecall_atom();
    match atom {
        LuaAtoms::BeginFrame => canvas_begin_frame(state),
        LuaAtoms::EndFrame => canvas_end_frame(state),
        LuaAtoms::Resize => canvas_resize(state),
        _ => state.error(format!(
            "{} is not a valid method of {}",
            name.unwrap_or_default(),
            ScriptedCanvas::LUA_NAME
        )),
    }
}

pub fn luaopen_rive_gpu(state: &mut LuaState) -> i32 {
    state.register_rive::<ScriptedShader>();
    state.set_readonly(-1, true);
    state.pop(1);
    state.register_rive_with_constructor::<ScriptedGPUBuffer>(gpu_buffer_construct);
    state.push_function(gpu_buffer_namecall);
    state.set_field(-2, "__namecall");
    state.push_function(gpu_buffer_index);
    state.set_field(-2, "__index");
    state.register_userdata_direct_number_field::<ScriptedGPUBuffer>("size", |value| {
        value.buffer.size() as f64
    });
    state.set_readonly(-1, true);
    state.pop(1);
    state.register_rive_with_constructor::<ScriptedGPUTexture>(gpu_texture_construct);
    state.push_function(gpu_texture_namecall);
    state.set_field(-2, "__namecall");
    state.push_function(gpu_texture_index);
    state.set_field(-2, "__index");
    state.register_userdata_direct_number_field::<ScriptedGPUTexture>("width", |value| {
        value.texture.width() as f64
    });
    state.register_userdata_direct_number_field::<ScriptedGPUTexture>("height", |value| {
        value.texture.height() as f64
    });
    state.set_readonly(-1, true);
    state.pop(1);
    state.register_rive_with_constructor::<ScriptedGPUSampler>(gpu_sampler_construct);
    state.set_readonly(-1, true);
    state.pop(1);
    state.register_rive_with_constructor::<ScriptedGPUPipeline>(gpu_pipeline_construct);
    state.push_function(gpu_pipeline_namecall);
    state.set_field(-2, "__namecall");
    state.set_readonly(-1, true);
    state.pop(1);
    state.register_rive_with_constructor::<ScriptedGPUBindGroup>(gpu_bind_group_construct);
    state.set_readonly(-1, true);
    state.pop(1);
    state.register_rive_with_constructor::<ScriptedGPUBindGroupLayout>(
        gpu_bind_group_layout_construct,
    );
    state.set_readonly(-1, true);
    state.pop(1);
    state.register_rive::<ScriptedGPURenderPass>();
    state.push_function(gpu_render_pass_namecall);
    state.set_field(-2, "__namecall");
    state.set_readonly(-1, true);
    state.pop(1);
    state.register_rive::<ScriptedGPUTextureView>();
    state.push_function(gpu_texture_view_index);
    state.set_field(-2, "__index");
    state.set_readonly(-1, true);
    state.pop(1);
    state.register_rive::<ScriptedGPUCanvas>();
    state.push_function(gpu_canvas_namecall);
    state.set_field(-2, "__namecall");
    state.push_function(gpu_canvas_index);
    state.set_field(-2, "__index");
    state.register_userdata_direct_number_field::<ScriptedGPUCanvas>("width", |value| {
        value.canvas.as_ref().map_or(0, RenderCanvas::width) as f64
    });
    state.register_userdata_direct_number_field::<ScriptedGPUCanvas>("height", |value| {
        value.canvas.as_ref().map_or(0, RenderCanvas::height) as f64
    });
    state.set_readonly(-1, true);
    state.pop(1);
    state.register_rive::<ScriptedCanvas>();
    state.push_function(canvas_namecall);
    state.set_field(-2, "__namecall");
    state.push_function(canvas_index);
    state.set_field(-2, "__index");
    state.register_userdata_direct_number_field::<ScriptedCanvas>("width", |value| {
        value.canvas.as_ref().map_or(0, RenderCanvas::width) as f64
    });
    state.register_userdata_direct_number_field::<ScriptedCanvas>("height", |value| {
        value.canvas.as_ref().map_or(0, RenderCanvas::height) as f64
    });
    state.set_readonly(-1, true);
    state.pop(1);
    0
}

pub fn rive_image_view_impl(state: &mut LuaState) -> i32 {
    let image = state.to_rive_mut::<ScriptedImage>(1);
    let Some(render_image) = image.image.as_ref() else {
        state.error::<()>("Image has no backing texture");
        unreachable!()
    };
    let Some(rive_image) = render_image.downcast_ref::<RiveRenderImage>() else {
        state.error::<()>("Image is not a GPU-backed RiveRenderImage");
        unreachable!()
    };
    let Some(source_texture) = rive_image.texture() else {
        state.error::<()>("Image GPU texture not available");
        unreachable!()
    };
    let scripting_context = state.thread_data::<dyn ScriptingContext>();
    if scripting_context.ore_context().is_none() {
        state.error::<()>("GPU context not available for Image:view()");
    }
    if image.cached_ore_view.is_none() {
        let mut texture_to_wrap = source_texture.clone();
        {
            image.cached_mirror_image = get_canvas_import_mirror_gl(
                scripting_context.render_context(),
                source_texture,
                render_image.width(),
                render_image.height(),
            );
            if let Some(mirror) = image
                .cached_mirror_image
                .as_ref()
                .and_then(|image| image.downcast_ref::<RiveRenderImage>())
                .and_then(RiveRenderImage::texture)
            {
                texture_to_wrap = mirror.clone();
            }
        }
        image.cached_ore_view = scripting_context.ore_context().unwrap().wrap_rive_texture(
            &texture_to_wrap,
            render_image.width(),
            render_image.height(),
        );
        if image.cached_ore_view.is_none() {
            state.error::<()>("Image:view() not supported on this backend");
        }
    }
    state.new_rive(ScriptedGPUTextureView {
        view: image.cached_ore_view.as_ref().unwrap().clone(),
        retained_image: image.image.clone(),
    });
    1
}

impl ScriptedImage {
    pub fn lua_new(state: &mut LuaState) -> &mut Self {
        state.new_rive(Self::default());
        state.to_rive_mut(-1)
    }
}

pub fn rive_lua_close_orphan_render_pass(state: &mut LuaState) {
    let context = state.thread_data::<dyn ScriptingContext>();
    let Some(ore) = context.ore_context() else {
        return;
    };
    let Some(pass) = ore.active_render_pass_mut() else {
        return;
    };
    if pass.is_finished() {
        return;
    }
    pass.finish();
    ore.set_active_render_pass(None);
    state.push_string("GPU render pass left open at script return. Call :finish() on render passes before returning.");
    context.print_error(state);
    state.pop(1);
}
