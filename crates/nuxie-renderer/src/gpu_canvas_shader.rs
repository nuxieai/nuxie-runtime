//! Physical authored-shader creation at the renderer factory boundary.
//!
//! Pinned C++ creates the exact selected backend module before it publishes a
//! `ScriptedShader` occurrence (`src/lua/renderer/lua_gpu.cpp:519-656`). A
//! rejected factory result therefore never enters Lua. Native wgpu can close
//! that validation scope synchronously; browser WebGPU exposes the same result
//! through an asynchronous `popErrorScope()` promise.

use std::collections::BTreeMap;
use std::sync::atomic::Ordering;
use std::sync::{Arc, Weak};

#[cfg(target_arch = "wasm32")]
use nuxie_render_api::GpuCanvasShaderLoad;
use nuxie_render_api::{GpuCanvasError, GpuCanvasShader, RenderGpuCanvasShader};

use super::gpu_canvas::{
    imported_resource_requirements, parse_authored_wgsl, ImportedResourceRequirement,
    ImportedUniformRequirement, ParsedAuthoredWgsl,
};
use super::Context;

pub(super) struct WgpuGpuCanvasShader {
    pub(super) occurrence_id: u64,
    pub(super) owner: Weak<Context>,
    pub(super) shader: GpuCanvasShader,
    pub(super) parsed: ParsedAuthoredWgsl,
    pub(super) uniform_requirements: BTreeMap<(u32, u32), ImportedUniformRequirement>,
    pub(super) resource_requirements: BTreeMap<(u32, u32), ImportedResourceRequirement>,
    pub(super) module: wgpu::ShaderModule,
}

impl RenderGpuCanvasShader for WgpuGpuCanvasShader {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

struct PreparedShader {
    owner: Arc<Context>,
    shader: GpuCanvasShader,
    parsed: ParsedAuthoredWgsl,
    uniform_requirements: BTreeMap<(u32, u32), ImportedUniformRequirement>,
    resource_requirements: BTreeMap<(u32, u32), ImportedResourceRequirement>,
}

struct UnvalidatedShader {
    prepared: PreparedShader,
    module: wgpu::ShaderModule,
}

impl PreparedShader {
    fn new(owner: Arc<Context>, shader: GpuCanvasShader) -> Result<Self, GpuCanvasError> {
        let parsed = parse_authored_wgsl(&shader.source)?;
        let resource_requirements =
            imported_resource_requirements(&shader, &parsed.module, &parsed.info)?;
        let uniform_requirements = resource_requirements
            .iter()
            .filter_map(|(&binding, requirement)| match requirement {
                ImportedResourceRequirement::Uniform(requirement) => Some((binding, *requirement)),
                _ => None,
            })
            .collect();
        Ok(Self {
            owner,
            shader,
            parsed,
            uniform_requirements,
            resource_requirements,
        })
    }

    fn create(self) -> UnvalidatedShader {
        let module = self
            .owner
            .device
            .create_shader_module(wgpu::ShaderModuleDescriptor {
                label: Some("nuxie-imported-gpu-canvas"),
                source: wgpu::ShaderSource::Wgsl(self.shader.source.clone().into()),
            });
        UnvalidatedShader {
            prepared: self,
            module,
        }
    }
}

impl UnvalidatedShader {
    fn publish(self) -> Arc<dyn RenderGpuCanvasShader> {
        Arc::new(WgpuGpuCanvasShader {
            occurrence_id: self
                .prepared
                .owner
                .next_gpu_canvas_shader_occurrence_id
                .fetch_add(1, Ordering::Relaxed),
            owner: Arc::downgrade(&self.prepared.owner),
            shader: self.prepared.shader,
            parsed: self.prepared.parsed,
            uniform_requirements: self.prepared.uniform_requirements,
            resource_requirements: self.prepared.resource_requirements,
            module: self.module,
        })
    }
}

fn validation_error(error: wgpu::Error) -> GpuCanvasError {
    GpuCanvasError::new(format!(
        "wgpu rejected imported GPU-canvas shader module: {error}"
    ))
}

#[cfg(not(target_arch = "wasm32"))]
pub(super) fn load_ready(
    owner: &Arc<Context>,
    shader: &GpuCanvasShader,
) -> Result<Arc<dyn RenderGpuCanvasShader>, GpuCanvasError> {
    let prepared = PreparedShader::new(Arc::clone(owner), shader.clone())?;
    let scope = owner.device.push_error_scope(wgpu::ErrorFilter::Validation);
    let unvalidated = prepared.create();
    if let Some(error) = pollster::block_on(scope.pop()) {
        return Err(validation_error(error));
    }
    Ok(unvalidated.publish())
}

#[cfg(target_arch = "wasm32")]
pub(super) fn load_awaitable(owner: Arc<Context>, shader: GpuCanvasShader) -> GpuCanvasShaderLoad {
    GpuCanvasShaderLoad::Pending(Box::pin(async move {
        let prepared = PreparedShader::new(Arc::clone(&owner), shader)?;
        let scope = owner.device.push_error_scope(wgpu::ErrorFilter::Validation);
        let unvalidated = prepared.create();
        if let Some(error) = scope.pop().await {
            return Err(validation_error(error));
        }
        Ok(unvalidated.publish())
    }))
}
