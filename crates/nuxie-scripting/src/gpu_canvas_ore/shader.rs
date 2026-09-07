//! Shader entry selection and binding-map layout construction from
//! `src/lua/renderer/lua_gpu.cpp` and `ScriptedShader` in rive_lua_libs.hpp.
use super::*;
use nuxie_ore_metal::bind_group_layout::{bindingMapForStages, makeBindGroupLayoutFromBindingMap};
use nuxie_ore_metal::shader_module::ShaderModule;
use nuxie_render_api::GpuCanvasShaderStage;

#[derive(Clone)]
pub(super) struct ShaderEntry {
    pub stage: u8,
    pub logical: String,
    pub physical: String,
    pub module: AnyResourceHandle,
}

#[derive(Clone)]
pub(super) struct Shader {
    pub entries: Vec<ShaderEntry>,
}
impl UserData for Shader {}

impl Shader {
    pub fn first_of_stage(&self, stage: ShaderStage) -> Option<&ShaderEntry> {
        self.entries
            .iter()
            .find(|entry| entry.stage == stage_tag(stage))
    }
    fn vertex_module(&self) -> Option<&ShaderModule> {
        self.first_of_stage(ShaderStage::vertex)?
            .module
            .shaderModuleBase()
    }
}

fn stage_tag(stage: ShaderStage) -> u8 {
    match stage {
        ShaderStage::vertex => 0,
        ShaderStage::fragment => 1,
        _ => u8::MAX,
    }
}

/// Target decoding and authenticated artifact preparation stay in the shared
/// RSTB owner. This retains its actual ORE modules, not another plan or device.
pub(super) fn shader_from_existing(shader: crate::gpu_canvas::GpuShader) -> Result<Shader> {
    let module = shader
        .module
        .ok_or_else(|| Error::runtime("shader has no loaded module"))?;
    let mut entries = Vec::with_capacity(shader.entries.len());
    for entry in shader.entries {
        let physical_module = module
            .ore_shader_entry(entry.stage, &entry.physical_entry_point)
            .ok_or_else(|| Error::runtime("shader entry has no loaded ORE module"))?;
        let stage = match entry.stage {
            GpuCanvasShaderStage::Vertex => 0,
            GpuCanvasShaderStage::Fragment => 1,
            GpuCanvasShaderStage::Compute => 2,
        };
        entries.push(ShaderEntry {
            stage,
            logical: entry.logical_entry_point,
            physical: entry.physical_entry_point,
            module: physical_module,
        });
    }
    if entries.is_empty() {
        return Err(Error::runtime("shader has no loaded module"));
    }
    Ok(Shader { entries })
}

pub(super) fn resolve_shader_entry(
    shader: &Shader,
    stage: ShaderStage,
    requested: Option<&str>,
) -> Result<ShaderEntry> {
    let requested = requested.filter(|name| !name.is_empty());
    if let Some(entry) = shader.entries.iter().find(|entry| {
        entry.stage == stage_tag(stage) && requested.is_none_or(|name| name == entry.logical)
    }) {
        return Ok(entry.clone());
    }
    let stage_name = match stage {
        ShaderStage::vertex => "vertex",
        ShaderStage::fragment => "fragment",
        _ => "compute",
    };
    if let Some(requested) = requested {
        let available = shader
            .entries
            .iter()
            .filter(|entry| entry.stage == stage_tag(stage))
            .map(|entry| entry.logical.as_str())
            .collect::<Vec<_>>()
            .join(", ");
        return Err(Error::runtime(format!(
            "GPUPipeline.new: {stage_name} entry point '{requested}' not found (available: {})",
            if available.is_empty() {
                "<none>"
            } else {
                &available
            }
        )));
    }
    Err(Error::runtime(format!(
        "GPUPipeline.new: {stage_name} shader has no {stage_name} entry point"
    )))
}

pub(super) fn resolve_stage_entry(
    table: &Table,
    key: &str,
    stage: ShaderStage,
) -> Result<Option<(Shader, ShaderEntry)>> {
    let value = table.get::<Value>(key)?;
    if matches!(value, Value::Nil) {
        return Ok(None);
    }
    let (module, requested) = match value {
        Value::Table(desc) => (desc.get::<Value>("module")?, string(&desc, "entryPoint")?),
        value => (value, None),
    };
    let shader = match module {
        Value::UserData(value) if value.is::<Shader>() => Some(value.borrow::<Shader>()?.clone()),
        _ => None,
    }.filter(|shader| !shader.entries.is_empty()).ok_or_else(||
        Error::runtime(format!("GPUPipeline.new: '{key}' must be a Shader or {{ module = Shader, entryPoint = string? }}")))?;
    let entry = resolve_shader_entry(&shader, stage, requested.as_deref())?;
    Ok(Some((shader, entry)))
}

#[derive(Clone)]
pub(super) struct Layout {
    pub resource: AnyResourceHandle,
    pub group: u32,
}
impl UserData for Layout {}

pub(super) fn auto_layouts(
    context: &mut dyn ContextApi,
    vertex: Option<&ShaderModule>,
    fragment: Option<&ShaderModule>,
) -> Result<Vec<Option<Layout>>> {
    let binding_map = bindingMapForStages(vertex, fragment);
    let mut seen = [false; kMaxBindGroups as usize];
    let mut max_group = 0;
    for index in 0..binding_map.size() {
        let group = usize::from(binding_map.at(index).group);
        if group >= seen.len() {
            continue;
        }
        seen[group] = true;
        max_group = max_group.max(group + 1);
    }
    let mut layouts = vec![None; max_group];
    for group in 0..max_group {
        if !seen[group] {
            continue;
        }
        // Source does not diagnose allocation here: null is passed to pipeline
        // validation for an automatically reflected layout.
        layouts[group] = makeBindGroupLayoutFromBindingMap(context, &binding_map, group as u32, &[])
            .map(|resource| Layout {
                resource,
                group: group as u32,
            });
    }
    Ok(layouts)
}

pub(super) fn dynamic_ubo_bindings(desc: &Table) -> Result<Vec<u32>> {
    let mut dynamic = Vec::new();
    if let Some(table) = optional_table(desc, "dynamicUBOs")? {
        for index in 1..=table.raw_len() {
            if dynamic.len() == 16 {
                break;
            }
            if let Some(value) = table.lua().coerce_number(table.raw_get::<Value>(index)?)? {
                dynamic.push(value as u32);
            }
        }
    }
    Ok(dynamic)
}

pub(super) fn install(lua: &Lua) -> Result<()> {
    constructor(lua, "GPUBindGroupLayout", |lua, desc| {
        let context = context(lua)?;
        let group = number(&desc, "groupIndex", 0.0)? as u32;
        if group >= kMaxBindGroups {
            return Err(Error::runtime(format!(
                "GPUBindGroupLayout.new: groupIndex must be in [0, {kMaxBindGroups})"
            )));
        }
        let shader = match desc.get::<Value>("shader")? {
            Value::UserData(value) if value.is::<Shader>() => {
                Some(value.borrow::<Shader>()?.clone())
            }
            _ => None,
        }
        .filter(|shader| !shader.entries.is_empty())
        .ok_or_else(|| {
            Error::runtime("GPUBindGroupLayout.new: 'shader' must be a Shader with a loaded module")
        })?;
        let dynamic = dynamic_ubo_bindings(&desc)?;
        let fragment = match desc.get::<Value>("fragment")? {
            Value::Nil => None,
            Value::UserData(value) if value.is::<Shader>() => {
                Some(value.borrow::<Shader>()?.clone())
            }
            _ => return Err(Error::runtime("GPUBindGroupLayout.new: 'fragment' must be a Shader with a @fragment entry point")),
        };
        let fragment_module = if let Some(fragment) = &fragment {
            Some(fragment.first_of_stage(ShaderStage::fragment).ok_or_else(||
                Error::runtime("GPUBindGroupLayout.new: 'fragment' must be a Shader with a @fragment entry point"))?)
                .and_then(|entry| entry.module.shaderModuleBase())
        } else {
            shader.first_of_stage(ShaderStage::fragment)
                .or_else(|| shader.first_of_stage(ShaderStage::vertex))
                .and_then(|entry| entry.module.shaderModuleBase())
        };
        let binding_map = bindingMapForStages(shader.vertex_module(), fragment_module);
        let mut context = context.borrow_mut();
        let resource =
            makeBindGroupLayoutFromBindingMap(&mut *context, &binding_map, group, &dynamic);
        let Some(resource) = resource else {
            let error = context.lastError();
            context.clearLastError();
            return Err(Error::runtime(format!(
                "GPUBindGroupLayout.new: {}",
                if error.is_empty() { "failed" } else { &error }
            )));
        };
        lua.create_userdata(Layout { resource, group })
    })
}
