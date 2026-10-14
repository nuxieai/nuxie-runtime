//! Pipeline and bind-group construction from lua_gpu.cpp.
use super::*;

pub(super) struct Pipeline {
    pub resource: AnyResourceHandle,
    pub sample_count: u32,
    pub auto_layouts: Vec<Option<Layout>>,
}
impl UserData for Pipeline {
    fn add_methods<M: UserDataMethods<Self>>(methods: &mut M) {
        methods.add_method("getBindGroupLayout",|lua,this,group:u32| {
            if this.auto_layouts.is_empty() { return Err(Error::runtime("getBindGroupLayout: pipeline was built with explicit bindGroupLayouts; reuse the layout you supplied")); }
            let layout=this.auto_layouts.get(group as usize).and_then(Clone::clone).ok_or_else(||Error::runtime(format!("getBindGroupLayout: group {group} not present in shader")))?;
            lua.create_userdata(layout)
        });
    }
}
pub(super) struct BindGroup {
    pub resource: AnyResourceHandle,
    /// Kept only when a texture follows a video, to make the group again when
    /// the video's frame moves on.
    remake: Option<BindGroupInputs>,
}
struct BindGroupInputs {
    layout: AnyResourceHandle,
    ubos: Vec<(u32, AnyResourceHandle, u32, u32)>,
    textures: Vec<(u32, AnyUserData)>,
    samplers: Vec<(u32, AnyResourceHandle)>,
}
impl BindGroup {
    /// Make the group again if a video it samples shows a new frame.
    pub(super) fn follow(lua: &Lua, group: &AnyUserData) -> Result<()> {
        let views = match &group.borrow::<BindGroup>()?.remake {
            Some(inputs) => inputs
                .textures
                .iter()
                .map(|(_, view)| view.clone())
                .collect::<Vec<_>>(),
            None => return Ok(()),
        };
        let mut moved = false;
        for view in &views {
            moved |= TextureView::follow(lua, view)?;
        }
        if !moved {
            return Ok(());
        }
        let mut this = group.borrow_mut::<BindGroup>()?;
        let inputs = this.remake.as_ref().expect("checked above");
        let textures = inputs
            .textures
            .iter()
            .map(|(slot, view)| Ok((*slot, view.borrow::<TextureView>()?.resource.clone())))
            .collect::<Result<Vec<_>>>()?;
        let resource = make_bind_group(
            lua,
            &inputs.layout,
            &inputs.ubos,
            &textures,
            &inputs.samplers,
        )?;
        this.resource = resource;
        Ok(())
    }
}
fn make_bind_group(
    lua: &Lua,
    layout: &AnyResourceHandle,
    ubos: &[(u32, AnyResourceHandle, u32, u32)],
    textures: &[(u32, AnyResourceHandle)],
    samplers: &[(u32, AnyResourceHandle)],
) -> Result<AnyResourceHandle> {
    let context = context(lua)?;
    let ubos: Vec<_> = ubos
        .iter()
        .map(|(slot, buffer, offset, size)| UBOEntry {
            slot: *slot,
            buffer: Some(buffer),
            offset: *offset,
            size: *size,
        })
        .collect();
    let textures: Vec<_> = textures
        .iter()
        .map(|(slot, view)| TexEntry {
            slot: *slot,
            view: Some(view),
        })
        .collect();
    let samplers: Vec<_> = samplers
        .iter()
        .map(|(slot, sampler)| SampEntry {
            slot: *slot,
            sampler: Some(sampler),
        })
        .collect();
    let desc = BindGroupDesc {
        layout: Some(layout),
        ubos: &ubos,
        uboCount: ubos.len() as u32,
        textures: &textures,
        textureCount: textures.len() as u32,
        samplers: &samplers,
        samplerCount: samplers.len() as u32,
        label: None,
    };
    let mut ctx = context.borrow_mut();
    ctx.clearLastError();
    let value = ctx.makeBindGroup(&desc);
    resource_result(&*ctx, value, "GPUBindGroup.new", "bind group")
}
impl UserData for BindGroup {}

fn tables(table: &Table, key: &str, max: usize) -> Result<Vec<Table>> {
    let Some(values) = optional_table(table, key)? else {
        return Ok(Vec::new());
    };
    (1..=values.raw_len().min(max))
        .map(|index| values.raw_get::<Table>(index))
        .collect()
}
fn write_mask(value: &str) -> Result<ColorWriteMask> {
    if value == "all" || value == "rgba" {
        return Ok(ColorWriteMask::all);
    }
    if value.is_empty() || value == "none" {
        return Ok(ColorWriteMask::none);
    }
    let mut bits = 0;
    for byte in value.bytes() {
        bits |= match byte.to_ascii_lowercase() {
            b'r' => ColorWriteMask::red.0,
            b'g' => ColorWriteMask::green.0,
            b'b' => ColorWriteMask::blue.0,
            b'a' => ColorWriteMask::alpha.0,
            _ => {
                return Err(Error::runtime(format!(
                    "invalid ColorWriteMask: '{value}' (expected r/g/b/a chars or 'all'/'none')"
                )));
            }
        };
    }
    Ok(ColorWriteMask(bits))
}
fn stencil(table: Option<Table>) -> Result<StencilFaceState> {
    let mut face = StencilFaceState::default();
    if let Some(table) = table {
        if let Some(value) = string(&table, "compare")? {
            face.compare = compare(&value)?;
        }
        if let Some(value) = string(&table, "failOp")? {
            face.failOp = stencil_op(&value)?;
        }
        if let Some(value) = string(&table, "depthFailOp")? {
            face.depthFailOp = stencil_op(&value)?;
        }
        if let Some(value) = string(&table, "passOp")? {
            face.passOp = stencil_op(&value)?;
        }
    }
    Ok(face)
}
pub(super) fn install(lua: &Lua) -> Result<()> {
    constructor(lua, "GPUBindGroup", |lua, table| {
        let layout_data: AnyUserData = table.get("layout")?;
        let layout = layout_data.borrow::<Layout>()?;
        let mut ubos = Vec::new();
        for entry in tables(&table, "ubos", 8)? {
            let slot = number(&entry, "slot", 0.0)? as u32;
            if slot > 7 {
                return Err(Error::runtime(format!(
                    "GPUBindGroup.new: UBO slot must be 0-7 (got {slot})"
                )));
            }
            let buffer: AnyUserData = entry.get("buffer")?;
            ubos.push((
                slot,
                buffer.borrow::<Buffer>()?.resource.clone(),
                number(&entry, "offset", 0.0)? as u32,
                number(&entry, "size", 0.0)? as u32,
            ));
        }
        let mut textures = Vec::new();
        let mut views = Vec::new();
        for entry in tables(&table, "textures", 8)? {
            let slot = number(&entry, "slot", 0.0)? as u32;
            if slot > 7 {
                return Err(Error::runtime(format!(
                    "GPUBindGroup.new: texture slot must be 0-7 (got {slot})"
                )));
            }
            let view: AnyUserData = entry.get("view")?;
            let (resource, follows) = {
                let view = view.borrow::<TextureView>()?;
                (view.resource.clone(), view.follows.is_some())
            };
            textures.push((slot, resource));
            views.push((slot, view, follows));
        }
        let mut samplers = Vec::new();
        for entry in tables(&table, "samplers", 8)? {
            let slot = number(&entry, "slot", 0.0)? as u32;
            if slot > 7 {
                return Err(Error::runtime(format!(
                    "GPUBindGroup.new: sampler slot must be 0-7 (got {slot})"
                )));
            }
            let sampler: AnyUserData = entry.get("sampler")?;
            samplers.push((slot, sampler.borrow::<Sampler>()?.resource.clone()));
        }
        let resource = make_bind_group(lua, &layout.resource, &ubos, &textures, &samplers)?;
        let remake = views
            .iter()
            .any(|(_, _, follows)| *follows)
            .then(|| BindGroupInputs {
                layout: layout.resource.clone(),
                ubos,
                textures: views
                    .into_iter()
                    .map(|(slot, view, _)| (slot, view))
                    .collect(),
                samplers,
            });
        lua.create_userdata(BindGroup { resource, remake })
    })?;
    constructor(lua, "GPUPipeline", |lua, table| {
        let context = context(lua)?;
        let (vertex_shader, vertex) =
            resolve_stage_entry(&table, "vertex", ShaderStage::vertex)?
                .ok_or_else(|| Error::runtime("GPUPipeline.new: 'vertex' is required"))?;
        let mut fragment =
            resolve_stage_entry(&table, "fragment", ShaderStage::fragment)?.map(|(_, entry)| entry);
        let layouts: Table = table.get("vertexLayout")?;
        let mut owned_attributes = Vec::new();
        let mut strides = Vec::new();
        let mut total_attributes = 0;
        // The C++ fixed arrays have eight layouts and 32 total attributes.
        // Reject overflow before constructing Rust spans, rather than reproducing an out-of-bounds read.
        if layouts.raw_len() > 8 {
            return Err(Error::runtime(
                nuxie_ore_metal::script_guards::vertex_layout_count_message(
                    layouts.raw_len() as u32,
                    8,
                ),
            ));
        }
        for index in 1..=layouts.raw_len() {
            let layout: Table = layouts.raw_get(index)?;
            strides.push((
                number(&layout, "stride", 0.0)? as u32,
                if string(&layout, "stepMode")?.as_deref() == Some("instance") {
                    VertexStepMode::instance
                } else {
                    VertexStepMode::vertex
                },
            ));
            let attributes_value = layout.get::<Value>("attributes")?;
            // The source reads lua_objlen before checking its fixed array
            // budget. This includes raw string/buffer/userdata byte lengths,
            // even though a later lua_rawgeti requires a table.
            // SAFETY: exec_raw roots argument 1; lua_objlen accepts every Lua
            // value and the closure pushes exactly one integer result.
            let attribute_count: i32 = unsafe {
                lua.exec_raw(attributes_value.clone(), |state| {
                    let count = luaur_vm::functions::lua_objlen::lua_objlen(state, 1);
                    luaur_vm::functions::lua_pushinteger::lua_pushinteger(state, count)
                })?
            };
            let attribute_count = attribute_count.max(0) as usize;
            if total_attributes + attribute_count > 32 {
                return Err(Error::runtime(
                    "GPUPipeline.new: more than 32 vertex attributes",
                ));
            }
            let attributes = match attributes_value {
                Value::Table(attributes) => Some(attributes),
                _ if attribute_count > 0 => {
                    return Err(Error::runtime("vertex attributes must be a table"));
                }
                _ => None,
            };
            let mut owned = Vec::new();
            for index in 1..=attribute_count {
                let attribute: Table = attributes
                    .as_ref()
                    .expect("attribute table")
                    .raw_get(index)?;
                total_attributes += 1;
                let mut attr = VertexAttribute::default();
                if let Some(value) = string(&attribute, "format")? {
                    attr.format = vertex_format(&value)?;
                }
                attr.shaderSlot = number(&attribute, "slot", 0.0)? as u32;
                attr.offset = number(&attribute, "offset", 0.0)? as u32;
                owned.push(attr);
            }
            owned_attributes.push(owned);
        }
        let vertex_buffers: Vec<_> = owned_attributes
            .iter()
            .zip(&strides)
            .map(|(attributes, (stride, step))| VertexBufferLayout {
                stride: *stride,
                stepMode: *step,
                attributes: Some(attributes),
                attributeCount: attributes.len() as u32,
            })
            .collect();
        let mut desc = PipelineDesc {
            colorCount: 0,
            vertexBuffers: Some(&vertex_buffers),
            vertexBufferCount: vertex_buffers.len() as u32,
            ..PipelineDesc::default()
        };
        if let Some(colors) = optional_table(&table, "colorTargets")? {
            if colors.raw_len() > 4 {
                return Err(Error::runtime(format!(
                    "GPUPipeline.new: colorTargets count {} exceeds maximum of 4",
                    colors.raw_len()
                )));
            }
            desc.colorCount = colors.raw_len() as u32;
            for index in 1..=colors.raw_len() {
                let color: Table = colors.raw_get(index)?;
                let target = &mut desc.colorTargets[index - 1];
                if let Some(value) = string(&color, "format")? {
                    target.format = texture_format(&value)?;
                }
                if let Some(value) = string(&color, "writeMask")? {
                    target.writeMask = write_mask(&value)?;
                }
                if let Some(blend) = optional_table(&color, "blend")? {
                    target.blendEnabled = true;
                    if let Some(value) = string(&blend, "srcColor")? {
                        target.blend.srcColor = blend_factor(&value)?;
                    }
                    if let Some(value) = string(&blend, "dstColor")? {
                        target.blend.dstColor = blend_factor(&value)?;
                    }
                    if let Some(value) = string(&blend, "colorOp")? {
                        target.blend.colorOp = blend_op(&value)?;
                    }
                    if let Some(value) = string(&blend, "srcAlpha")? {
                        target.blend.srcAlpha = blend_factor(&value)?;
                    }
                    if let Some(value) = string(&blend, "dstAlpha")? {
                        target.blend.dstAlpha = blend_factor(&value)?;
                    }
                    if let Some(value) = string(&blend, "alphaOp")? {
                        target.blend.alphaOp = blend_op(&value)?;
                    }
                }
            }
        }
        if fragment.is_none() && desc.colorCount > 0 {
            fragment = Some(resolve_shader_entry(
                &vertex_shader,
                ShaderStage::fragment,
                None,
            )?);
        }
        desc.vertexModule = Some(&vertex.module);
        desc.vertexEntryPoint = Some(&vertex.physical);
        desc.fragmentModule = fragment.as_ref().map(|entry| &entry.module);
        desc.fragmentEntryPoint = fragment.as_ref().map(|entry| entry.physical.as_str());
        if let Some(depth) = optional_table(&table, "depthStencil")? {
            if let Some(value) = string(&depth, "format")? {
                desc.depthStencil.format = texture_format(&value)?;
            }
            if let Some(value) = string(&depth, "compare")? {
                desc.depthStencil.depthCompare = compare(&value)?;
            }
            desc.depthStencil.depthWriteEnabled = boolean(&depth, "write", false)?;
            desc.depthStencil.depthBias = number(&depth, "depthBias", 0.0)? as i32;
            desc.depthStencil.depthBiasSlopeScale =
                number(&depth, "depthBiasSlopeScale", 0.0)? as f32;
            desc.depthStencil.depthBiasClamp = number(&depth, "depthBiasClamp", 0.0)? as f32;
        }
        desc.stencilFront = stencil(optional_table(&table, "stencilFront")?)?;
        desc.stencilBack = stencil(optional_table(&table, "stencilBack")?)?;
        desc.stencilReadMask = number(&table, "stencilReadMask", 255.0)? as u8;
        desc.stencilWriteMask = number(&table, "stencilWriteMask", 255.0)? as u8;
        let (layouts, auto) = if let Some(layouts) = optional_table(&table, "bindGroupLayouts")? {
            if layouts.raw_len() > kMaxBindGroups as usize {
                return Err(Error::runtime(
                    "GPUPipeline.new: bindGroupLayouts count exceeds maximum",
                ));
            }
            let mut values = Vec::with_capacity(layouts.raw_len());
            for index in 1..=layouts.raw_len() {
                let data: AnyUserData = layouts.raw_get(index)?;
                values.push(Some(data.borrow::<Layout>()?.clone()));
            }
            (values, false)
        } else {
            (
                auto_layouts(
                    &mut *context.borrow_mut(),
                    desc.vertexModule
                        .and_then(|module| module.shaderModuleBase()),
                    desc.fragmentModule
                        .and_then(|module| module.shaderModuleBase()),
                )?,
                true,
            )
        };
        let layout_refs: Vec<_> = layouts
            .iter()
            .map(|layout| layout.as_ref().map(|layout| &layout.resource))
            .collect();
        desc.bindGroupLayouts = Some(&layout_refs);
        desc.bindGroupLayoutCount = layout_refs.len() as u32;
        if let Some(value) = string(&table, "cullMode")? {
            desc.cullMode = cull_mode(&value)?;
        }
        if let Some(value) = string(&table, "winding")? {
            desc.winding = winding(&value)?;
        }
        if let Some(value) = string(&table, "topology")? {
            desc.topology = topology(&value)?;
        }
        desc.sampleCount = number(&table, "sampleCount", 1.0)? as u32;
        let mut error = String::new();
        let mut ctx = context.borrow_mut();
        check_guard(|error| {
            nuxie_ore_metal::script_guards::validateVertexLayout(Some(&*ctx), &desc, Some(error))
        })?;
        let resource = ctx.makePipeline(&desc, Some(&mut error)).ok_or_else(|| {
            Error::runtime(if error.is_empty() {
                "GPUPipeline.new: failed to create pipeline".to_owned()
            } else {
                format!("GPUPipeline.new: {error}")
            })
        })?;
        lua.create_userdata(Pipeline {
            resource,
            sample_count: desc.sampleCount,
            auto_layouts: if auto { layouts } else { Vec::new() },
        })
    })
}
