//! Physical authored-shader creation at the renderer factory boundary.
//!
//! Pinned C++ creates the exact selected backend module before it publishes a
//! `ScriptedShader` occurrence (`src/lua/renderer/lua_gpu.cpp:519-656`). A
//! rejected factory result therefore never enters Lua. Native wgpu can close
//! that validation scope synchronously; browser WebGPU exposes the same result
//! through an asynchronous `popErrorScope()` promise.

#[cfg(all(
    feature = "apple-authored-msl",
    any(target_os = "ios", target_os = "macos")
))]
use std::borrow::Cow;
use std::collections::BTreeMap;
use std::sync::atomic::Ordering;
use std::sync::{Arc, Weak};

#[cfg(target_arch = "wasm32")]
use nuxie_render_api::GpuCanvasShaderLoad;
use nuxie_render_api::{
    GpuCanvasError, GpuCanvasShader, GpuCanvasShaderArtifact, GpuCanvasShaderProfile,
    RenderGpuCanvasShader,
};

#[cfg(all(
    feature = "apple-authored-msl",
    any(target_os = "ios", target_os = "macos")
))]
use super::gpu_canvas::imported_supplemental_entry_reflections;
use super::gpu_canvas::{
    imported_resource_requirements, imported_wgsl_entry_reflections, parse_authored_wgsl,
    ImportedEntryPointReflection, ImportedResourceRequirement, ImportedUniformRequirement,
};
use super::Context;

pub(super) struct WgpuGpuCanvasShader {
    pub(super) occurrence_id: u64,
    pub(super) template: Arc<WgpuGpuCanvasShaderTemplate>,
}

pub(super) struct WgpuGpuCanvasShaderTemplate {
    pub(super) owner: Weak<Context>,
    pub(super) shader: GpuCanvasShader,
    pub(super) entry_reflections: BTreeMap<(u8, String), ImportedEntryPointReflection>,
    pub(super) uniform_requirements: BTreeMap<(u32, u32), ImportedUniformRequirement>,
    pub(super) resource_requirements: BTreeMap<(u32, u32), ImportedResourceRequirement>,
    pub(super) module: wgpu::ShaderModule,
}

impl std::ops::Deref for WgpuGpuCanvasShader {
    type Target = WgpuGpuCanvasShaderTemplate;

    fn deref(&self) -> &Self::Target {
        &self.template
    }
}

impl RenderGpuCanvasShader for WgpuGpuCanvasShader {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

struct PreparedShader {
    owner: Arc<Context>,
    shader: GpuCanvasShader,
    entry_reflections: BTreeMap<(u8, String), ImportedEntryPointReflection>,
    uniform_requirements: BTreeMap<(u32, u32), ImportedUniformRequirement>,
    resource_requirements: BTreeMap<(u32, u32), ImportedResourceRequirement>,
}

struct UnvalidatedShader {
    prepared: PreparedShader,
    module: wgpu::ShaderModule,
}

impl PreparedShader {
    fn new(owner: Arc<Context>, artifact: GpuCanvasShaderArtifact) -> Result<Self, GpuCanvasError> {
        let (shader, entry_reflections, resource_requirements) =
            match (owner.gpu_canvas_shader_profile, artifact) {
                (GpuCanvasShaderProfile::WebGpu, GpuCanvasShaderArtifact::WebGpu(shader)) => {
                    let parsed = parse_authored_wgsl(&shader.source)?;
                    let entry_reflections = imported_wgsl_entry_reflections(&shader, &parsed)?;
                    let resource_requirements =
                        imported_resource_requirements(&shader, &parsed.module, &parsed.info)?;
                    (shader, entry_reflections, resource_requirements)
                }
                #[cfg(all(
                    feature = "apple-authored-msl",
                    any(target_os = "ios", target_os = "macos")
                ))]
                (
                    GpuCanvasShaderProfile::TrustedAppleMetal,
                    GpuCanvasShaderArtifact::TrustedAppleMetal(shader),
                ) => {
                    let entry_reflections = imported_supplemental_entry_reflections(
                        shader.entries(),
                        shader.entry_reflection(),
                    )?;
                    let resource_requirements =
                        super::gpu_canvas::imported_apple_metal_resource_requirements(&shader)?;
                    (
                        GpuCanvasShader {
                            source: shader.source().to_owned(),
                            entries: shader.entries().to_vec(),
                            bindings: shader.bindings().to_vec(),
                        },
                        entry_reflections,
                        resource_requirements,
                    )
                }
                (profile, _) => {
                    return Err(GpuCanvasError::new(format!(
                        "authored shader artifact does not match factory profile {profile:?}"
                    )));
                }
            };
        let uniform_requirements = resource_requirements
            .iter()
            .filter_map(|(&binding, requirement)| match requirement {
                ImportedResourceRequirement::Uniform(requirement) => Some((binding, *requirement)),
                _ => None,
            })
            .collect();
        validate_compute_workgroups(&entry_reflections, &owner.device.limits())?;
        Ok(Self {
            owner,
            shader,
            entry_reflections,
            uniform_requirements,
            resource_requirements,
        })
    }

    fn create(self) -> UnvalidatedShader {
        let module = create_module(
            &self.owner,
            &self.shader,
            &self.entry_reflections,
            "nuxie-imported-gpu-canvas",
        );
        UnvalidatedShader {
            prepared: self,
            module,
        }
    }
}

fn validate_compute_workgroups(
    reflections: &BTreeMap<(u8, String), ImportedEntryPointReflection>,
    limits: &wgpu::Limits,
) -> Result<(), GpuCanvasError> {
    for ((stage, entry), reflection) in reflections {
        if *stage != nuxie_render_api::GpuCanvasShaderStage::Compute as u8 {
            continue;
        }
        let [x, y, z] = reflection.workgroup_size;
        let invocations = x
            .checked_mul(y)
            .and_then(|value| value.checked_mul(z))
            .ok_or_else(|| GpuCanvasError::new("compute workgroup invocation count overflowed"))?;
        if x > limits.max_compute_workgroup_size_x
            || y > limits.max_compute_workgroup_size_y
            || z > limits.max_compute_workgroup_size_z
            || invocations > limits.max_compute_invocations_per_workgroup
        {
            return Err(GpuCanvasError::new(format!(
                "compute entry '{entry}' workgroup [{x}, {y}, {z}] exceeds device limits [{}, {}, {}] / {} invocations",
                limits.max_compute_workgroup_size_x,
                limits.max_compute_workgroup_size_y,
                limits.max_compute_workgroup_size_z,
                limits.max_compute_invocations_per_workgroup,
            )));
        }
    }
    Ok(())
}

impl UnvalidatedShader {
    fn publish(self) -> Arc<dyn RenderGpuCanvasShader> {
        let owner = Arc::clone(&self.prepared.owner);
        let template = Arc::new(WgpuGpuCanvasShaderTemplate {
            owner: Arc::downgrade(&owner),
            shader: self.prepared.shader,
            entry_reflections: self.prepared.entry_reflections,
            uniform_requirements: self.prepared.uniform_requirements,
            resource_requirements: self.prepared.resource_requirements,
            module: self.module,
        });
        Arc::new(WgpuGpuCanvasShader {
            occurrence_id: owner
                .next_gpu_canvas_shader_occurrence_id
                .fetch_add(1, Ordering::Relaxed),
            template,
        })
    }
}

fn create_module(
    owner: &Context,
    shader: &GpuCanvasShader,
    entry_reflections: &BTreeMap<(u8, String), ImportedEntryPointReflection>,
    label: &'static str,
) -> wgpu::ShaderModule {
    #[cfg(not(all(
        feature = "apple-authored-msl",
        any(target_os = "ios", target_os = "macos")
    )))]
    let _ = entry_reflections;
    match owner.gpu_canvas_shader_profile {
        GpuCanvasShaderProfile::WebGpu => {
            owner
                .device
                .create_shader_module(wgpu::ShaderModuleDescriptor {
                    label: Some(label),
                    source: wgpu::ShaderSource::Wgsl(shader.source.clone().into()),
                })
        }
        #[cfg(all(
            feature = "apple-authored-msl",
            any(target_os = "ios", target_os = "macos")
        ))]
        GpuCanvasShaderProfile::TrustedAppleMetal => {
            let entries = shader
                .entries
                .iter()
                .map(|entry| {
                    let workgroup_size = entry_reflections
                        .get(&(entry.stage as u8, entry.physical_entry_point.clone()))
                        .expect("validated supplemental entry reflection")
                        .workgroup_size;
                    wgpu::PassthroughShaderEntryPoint {
                        name: Cow::Borrowed(entry.physical_entry_point.as_str()),
                        workgroup_size: (workgroup_size[0], workgroup_size[1], workgroup_size[2]),
                    }
                })
                .collect::<Vec<_>>();
            // SAFETY: `TrustedAppleMetal` can only be constructed from exact
            // bytes admitted as valid output of the trusted MSL exporter.
            // Before reaching this boundary, target-10 slots and supplemental
            // entry/resource reflection are checked against the same allocator
            // used by wgpu-hal production layouts.
            unsafe {
                owner.device.create_shader_module_passthrough(
                    wgpu::ShaderModuleDescriptorPassthrough {
                        label: Some(label),
                        entry_points: Cow::Borrowed(&entries),
                        msl: Some(Cow::Borrowed(&shader.source)),
                        ..Default::default()
                    },
                )
            }
        }
        #[cfg(not(all(
            feature = "apple-authored-msl",
            any(target_os = "ios", target_os = "macos")
        )))]
        GpuCanvasShaderProfile::TrustedAppleMetal => {
            unreachable!("trusted Apple Metal profile has no constructor on this build")
        }
    }
}

pub(super) fn publish_occurrence(
    owner: &Arc<Context>,
    prepared: &Arc<dyn RenderGpuCanvasShader>,
) -> Result<Arc<dyn RenderGpuCanvasShader>, GpuCanvasError> {
    let prepared = prepared
        .as_any()
        .downcast_ref::<WgpuGpuCanvasShader>()
        .ok_or_else(|| GpuCanvasError::new("foreign shader backend"))?;
    let prepared_owner = prepared
        .owner
        .upgrade()
        .ok_or_else(|| GpuCanvasError::new("prepared shader device is unavailable"))?;
    if !Arc::ptr_eq(owner, &prepared_owner) {
        return Err(GpuCanvasError::new(
            "prepared shader belongs to a different device",
        ));
    }
    // Browser WebGPU validates this exact authored source asynchronously
    // before Lua starts. The synchronous lookup can therefore create the
    // fresh physical module that C++ creates in `Context::shader` without
    // opening another promise-backed error scope. Keep parsed interface data
    // shared by value, but never share the module-owning template itself.
    let module = create_module(
        owner,
        &prepared.shader,
        &prepared.entry_reflections,
        "nuxie-imported-gpu-canvas-occurrence",
    );
    Ok(Arc::new(WgpuGpuCanvasShader {
        occurrence_id: owner
            .next_gpu_canvas_shader_occurrence_id
            .fetch_add(1, Ordering::Relaxed),
        template: Arc::new(WgpuGpuCanvasShaderTemplate {
            owner: Arc::downgrade(owner),
            shader: prepared.shader.clone(),
            entry_reflections: prepared.entry_reflections.clone(),
            uniform_requirements: prepared.uniform_requirements.clone(),
            resource_requirements: prepared.resource_requirements.clone(),
            module,
        }),
    }))
}

fn validation_error(error: wgpu::Error) -> GpuCanvasError {
    GpuCanvasError::new(format!(
        "wgpu rejected imported GPU-canvas shader module: {error}"
    ))
}

#[cfg(not(target_arch = "wasm32"))]
pub(super) fn load_ready(
    owner: &Arc<Context>,
    shader: &GpuCanvasShaderArtifact,
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
pub(super) fn load_awaitable(
    owner: Arc<Context>,
    shader: GpuCanvasShaderArtifact,
) -> GpuCanvasShaderLoad {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[cfg(all(
        feature = "apple-authored-msl",
        any(target_os = "ios", target_os = "macos")
    ))]
    #[test]
    fn compute_workgroups_are_checked_against_every_device_limit() {
        use nuxie_render_api::{
            GpuCanvasShaderEntry, GpuCanvasShaderEntryReflection, GpuCanvasShaderStage,
        };

        let limits = wgpu::Limits::downlevel_defaults();
        let reflections = |size| {
            imported_supplemental_entry_reflections(
                &[GpuCanvasShaderEntry {
                    stage: GpuCanvasShaderStage::Compute,
                    logical_entry_point: "main".into(),
                    physical_entry_point: "compute_main".into(),
                }],
                &[GpuCanvasShaderEntryReflection {
                    stage: GpuCanvasShaderStage::Compute,
                    logical_entry_point: "main".into(),
                    physical_entry_point: "compute_main".into(),
                    workgroup_size: size,
                    inputs: Vec::new(),
                    outputs: Vec::new(),
                }],
            )
            .unwrap()
        };

        validate_compute_workgroups(&reflections([1, 1, 1]), &limits).unwrap();
        let error = validate_compute_workgroups(
            &reflections([limits.max_compute_workgroup_size_x + 1, 1, 1]),
            &limits,
        )
        .unwrap_err();
        assert!(error.to_string().contains("exceeds device limits"));

        let error = validate_compute_workgroups(
            &reflections([limits.max_compute_invocations_per_workgroup, 2, 1]),
            &limits,
        )
        .unwrap_err();
        assert!(error.to_string().contains("exceeds device limits"));
    }
}
