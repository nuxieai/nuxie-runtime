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
            let layout=this.auto_layouts.get(group as usize).and_then(Clone::clone).ok_or_else(||Error::runtime(format!("getBindGroupLayout: group {group} is not bound by the pipeline's entry points")))?;
            lua.create_userdata(layout)
        });
    }
}
pub(super) struct BindGroup {
    pub resource: AnyResourceHandle,
    /// Kept only when a texture follows a video, to make the group again when
    /// the video's frame moves on.
    remake: Option<BindGroupInputs>,
    /// The charges of the buffers and views the group holds, which it keeps
    /// alive past their own userdata.
    _charges: Vec<GpuCharge>,
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
        let Some(inputs) = this.remake.as_ref() else {
            return Ok(());
        };
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

fn tables(table: &Table, loan: &mut Loan<'_>, key: &str, max: usize) -> Result<Vec<Table>> {
    let Some(values) = optional_table(table, loan, key)? else {
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
fn stencil(table: Option<Table>, loan: &mut Loan<'_>) -> Result<StencilFaceState> {
    let mut face = StencilFaceState::default();
    if let Some(table) = table {
        if let Some(value) = string(&table, loan, "compare")? {
            face.compare = compare(&value)?;
        }
        if let Some(value) = string(&table, loan, "failOp")? {
            face.failOp = stencil_op(&value)?;
        }
        if let Some(value) = string(&table, loan, "depthFailOp")? {
            face.depthFailOp = stencil_op(&value)?;
        }
        if let Some(value) = string(&table, loan, "passOp")? {
            face.passOp = stencil_op(&value)?;
        }
    }
    Ok(face)
}
pub(super) fn install(lua: &Lua) -> Result<()> {
    constructor(lua, "GPUBindGroup", |lua, loan, table| {
        let layout_data: AnyUserData = table.get_with(reborrow_loan(loan), "layout")?;
        let layout = layout_data.borrow::<Layout>()?;
        let mut ubos = Vec::new();
        let mut charges = Vec::new();
        for entry in tables(&table, loan, "ubos", 8)? {
            let slot = number(&entry, loan, "slot", 0.0)? as u32;
            if slot > 7 {
                return Err(Error::runtime(format!(
                    "GPUBindGroup.new: UBO slot must be 0-7 (got {slot})"
                )));
            }
            let buffer: AnyUserData = entry.get_with(reborrow_loan(loan), "buffer")?;
            let resource = {
                let buffer = buffer.borrow::<Buffer>()?;
                charges.push(Rc::clone(&buffer.charge));
                buffer.resource.clone()
            };
            ubos.push((
                slot,
                resource,
                number(&entry, loan, "offset", 0.0)? as u32,
                number(&entry, loan, "size", 0.0)? as u32,
            ));
        }
        let mut textures = Vec::new();
        let mut views = Vec::new();
        for entry in tables(&table, loan, "textures", 8)? {
            let slot = number(&entry, loan, "slot", 0.0)? as u32;
            if slot > 7 {
                return Err(Error::runtime(format!(
                    "GPUBindGroup.new: texture slot must be 0-7 (got {slot})"
                )));
            }
            let view: AnyUserData = entry.get_with(reborrow_loan(loan), "view")?;
            let (resource, follows) = {
                let view = view.borrow::<TextureView>()?;
                charges.extend(view.charge.clone());
                (view.resource.clone(), view.follows.is_some())
            };
            textures.push((slot, resource));
            views.push((slot, view, follows));
        }
        let mut samplers = Vec::new();
        for entry in tables(&table, loan, "samplers", 8)? {
            let slot = number(&entry, loan, "slot", 0.0)? as u32;
            if slot > 7 {
                return Err(Error::runtime(format!(
                    "GPUBindGroup.new: sampler slot must be 0-7 (got {slot})"
                )));
            }
            let sampler: AnyUserData = entry.get_with(reborrow_loan(loan), "sampler")?;
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
        lua.create_userdata(BindGroup {
            resource,
            remake,
            _charges: charges,
        })
    })?;
    constructor(lua, "GPUPipeline", |lua, loan, table| {
        let context = context(lua)?;
        let (vertex_shader, vertex) =
            resolve_stage_entry(&table, loan, "vertex", ShaderStage::vertex)?
                .ok_or_else(|| Error::runtime("GPUPipeline.new: 'vertex' is required"))?;
        let mut fragment =
            resolve_stage_entry(&table, loan, "fragment", ShaderStage::fragment)?.map(|(_, entry)| entry);
        let layouts: Table = table.get_with(reborrow_loan(loan), "vertexLayout")?;
        let mut owned_attributes = Vec::new();
        let mut strides = Vec::new();
        let mut total_attributes: usize = 0;
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
                number(&layout, loan, "stride", 0.0)? as u32,
                if string(&layout, loan, "stepMode")?.as_deref() == Some("instance") {
                    VertexStepMode::instance
                } else {
                    VertexStepMode::vertex
                },
            ));
            let attributes_value = layout.get_with::<Value>(reborrow_loan(loan), "attributes")?;
            // The source reads lua_objlen before checking its fixed array
            // budget. This includes raw string/buffer/userdata byte lengths,
            // even though a later lua_rawgeti requires a table.
            // Seam row R32: `lua_objlen` of any value.
            let attribute_count = lua.objlen(&attributes_value)?;
            let Some(layout_total) = total_attributes
                .checked_add(attribute_count)
                .filter(|total| *total <= 32)
            else {
                return Err(Error::runtime(
                    "GPUPipeline.new: more than 32 vertex attributes",
                ));
            };
            let attributes = match attributes_value {
                Value::Table(attributes) => Some(attributes),
                _ if attribute_count > 0 => {
                    return Err(Error::runtime("vertex attributes must be a table"));
                }
                _ => None,
            };
            let mut owned = Vec::new();
            // `attributes` is `None` only when `attribute_count` is zero.
            if let Some(attributes) = &attributes {
                for index in 1..=attribute_count {
                    let attribute: Table = attributes.raw_get(index)?;
                    let mut attr = VertexAttribute::default();
                    if let Some(value) = string(&attribute, loan, "format")? {
                        attr.format = vertex_format(&value)?;
                    }
                    attr.shaderSlot = number(&attribute, loan, "slot", 0.0)? as u32;
                    attr.offset = number(&attribute, loan, "offset", 0.0)? as u32;
                    owned.push(attr);
                }
            }
            total_attributes = layout_total;
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
        if let Some(colors) = optional_table(&table, loan, "colorTargets")? {
            if colors.raw_len() > 4 {
                return Err(Error::runtime(format!(
                    "GPUPipeline.new: colorTargets count {} exceeds maximum of 4",
                    colors.raw_len()
                )));
            }
            desc.colorCount = colors.raw_len() as u32;
            // At most four entries (checked above), one per target slot.
            for (index, target) in (1..=colors.raw_len()).zip(desc.colorTargets.iter_mut()) {
                let color: Table = colors.raw_get(index)?;
                if let Some(value) = string(&color, loan, "format")? {
                    target.format = texture_format(&value)?;
                }
                if let Some(value) = string(&color, loan, "writeMask")? {
                    target.writeMask = write_mask(&value)?;
                }
                if let Some(blend) = optional_table(&color, loan, "blend")? {
                    target.blendEnabled = true;
                    if let Some(value) = string(&blend, loan, "srcColor")? {
                        target.blend.srcColor = blend_factor(&value)?;
                    }
                    if let Some(value) = string(&blend, loan, "dstColor")? {
                        target.blend.dstColor = blend_factor(&value)?;
                    }
                    if let Some(value) = string(&blend, loan, "colorOp")? {
                        target.blend.colorOp = blend_op(&value)?;
                    }
                    if let Some(value) = string(&blend, loan, "srcAlpha")? {
                        target.blend.srcAlpha = blend_factor(&value)?;
                    }
                    if let Some(value) = string(&blend, loan, "dstAlpha")? {
                        target.blend.dstAlpha = blend_factor(&value)?;
                    }
                    if let Some(value) = string(&blend, loan, "alphaOp")? {
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
        if let Some(depth) = optional_table(&table, loan, "depthStencil")? {
            if let Some(value) = string(&depth, loan, "format")? {
                desc.depthStencil.format = texture_format(&value)?;
            }
            if let Some(value) = string(&depth, loan, "compare")? {
                desc.depthStencil.depthCompare = compare(&value)?;
            }
            desc.depthStencil.depthWriteEnabled = boolean(&depth, loan, "write", false)?;
            desc.depthStencil.depthBias = number(&depth, loan, "depthBias", 0.0)? as i32;
            desc.depthStencil.depthBiasSlopeScale =
                number(&depth, loan, "depthBiasSlopeScale", 0.0)? as f32;
            desc.depthStencil.depthBiasClamp = number(&depth, loan, "depthBiasClamp", 0.0)? as f32;
        }
        desc.stencilFront = stencil(optional_table(&table, loan, "stencilFront")?, loan)?;
        desc.stencilBack = stencil(optional_table(&table, loan, "stencilBack")?, loan)?;
        desc.stencilReadMask = number(&table, loan, "stencilReadMask", 255.0)? as u8;
        desc.stencilWriteMask = number(&table, loan, "stencilWriteMask", 255.0)? as u8;
        let (layouts, auto) = if let Some(layouts) = optional_table(&table, loan, "bindGroupLayouts")? {
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
                    desc.vertexEntryPoint,
                    desc.fragmentModule
                        .and_then(|module| module.shaderModuleBase()),
                    desc.fragmentEntryPoint,
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
        if let Some(value) = string(&table, loan, "cullMode")? {
            desc.cullMode = cull_mode(&value)?;
        }
        if let Some(value) = string(&table, loan, "winding")? {
            desc.winding = winding(&value)?;
        }
        if let Some(value) = string(&table, loan, "topology")? {
            desc.topology = topology(&value)?;
        }
        desc.sampleCount = number(&table, loan, "sampleCount", 1.0)? as u32;
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
        // Release the context before allocating in the VM: a collection step there
        // can drop other GPU userdata, which must find the context free.
        drop(ctx);
        lua.create_userdata(Pipeline {
            resource,
            sample_count: desc.sampleCount,
            auto_layouts: if auto { layouts } else { Vec::new() },
        })
    })
}
