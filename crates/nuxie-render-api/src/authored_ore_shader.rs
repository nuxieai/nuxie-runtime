//! Authored RSTB artifact-to-ORE module bridge used by immediate and deferred factories.
//! Target decoding and trust checks remain in the existing shader-asset owner.
use crate::{
    GpuCanvasError, GpuCanvasShaderArtifact, GpuCanvasShaderBinding, GpuCanvasShaderEntry,
    GpuCanvasShaderEntrySelection, GpuCanvasShaderProfile, GpuCanvasShaderStage,
    GpuCanvasShaderTextureSamplerPair, RenderGpuCanvasShader,
};
use nuxie_ore_metal::{
    context::{ContextApi, ShaderTarget},
    gpu_resource::AnyResourceHandle,
    types::{ShaderLanguage, ShaderModuleDesc, ShaderStage},
};
use std::{any::Any, rc::Rc};

fn rejected(message: impl Into<String>) -> GpuCanvasError {
    GpuCanvasError::new(format!("exact GPU-canvas: {}", message.into()))
}

/// Upstream moduleEntryBindingBytes: narrow and rename logical entries to the
/// physical names used by a backend. Incomplete/older metadata stays absent.
pub fn module_entry_bindings(bytes: &[u8], entries: &[GpuCanvasShaderEntry]) -> Vec<u8> {
    use nuxie_ore_metal::{
        rstb_entry_bindings::{decodeEntryBindings, moduleEntryBindingBytes},
        rstb_entry_container::RstbEntryView,
    };
    let mut asset = Vec::new();
    decodeEntryBindings(Some(bytes), bytes.len(), &mut asset);
    let views: Vec<_> = entries
        .iter()
        .map(|entry| RstbEntryView {
            stage: match entry.stage {
                GpuCanvasShaderStage::Vertex => 0,
                GpuCanvasShaderStage::Fragment => 1,
                GpuCanvasShaderStage::Compute => 2,
            },
            logical: entry.logical_entry_point.as_str().into(),
            physical: entry.physical_entry_point.as_str().into(),
            ..Default::default()
        })
        .collect();
    moduleEntryBindingBytes(&asset, &views, 0, views.len() as u32)
}

#[cfg(test)]
mod entry_binding_tests {
    use super::*;
    use nuxie_ore_metal::rstb_entry_bindings::{
        RstbEntryBindings, decodeEntryBindings, encodeEntryBindings,
    };

    #[test]
    fn metadata_is_narrowed_and_renamed_per_physical_module() {
        let asset = encodeEntryBindings(&[
            RstbEntryBindings {
                stage: 0,
                name: "vertex".into(),
                bindings: vec![1],
            },
            RstbEntryBindings {
                stage: 1,
                name: "fragment".into(),
                bindings: vec![0x202],
            },
        ]);
        let selected = GpuCanvasShaderEntry {
            stage: GpuCanvasShaderStage::Fragment,
            logical_entry_point: "fragment".into(),
            physical_entry_point: "main".into(),
        };
        let bytes = module_entry_bindings(&asset, &[selected.clone()]);
        let mut decoded = Vec::new();
        assert!(decodeEntryBindings(Some(&bytes), bytes.len(), &mut decoded));
        assert_eq!(
            decoded,
            vec![RstbEntryBindings {
                stage: 1,
                name: "main".into(),
                bindings: vec![0x202]
            }]
        );
        let missing = GpuCanvasShaderEntry {
            logical_entry_point: "missing".into(),
            ..selected
        };
        assert!(module_entry_bindings(&asset, &[missing]).is_empty());
        assert!(module_entry_bindings(&[1, 1], &[]).is_empty());
    }
}
fn context_error(context: &dyn ContextApi, fallback: &str) -> String {
    let error = context.lastError();
    if error.is_empty() {
        fallback.to_owned()
    } else {
        error
    }
}
pub fn profile_for_target(target: ShaderTarget) -> GpuCanvasShaderProfile {
    match target {
        ShaderTarget::wgsl => GpuCanvasShaderProfile::WebGpu,
        ShaderTarget::glsl => GpuCanvasShaderProfile::WebGl2,
        ShaderTarget::msl => GpuCanvasShaderProfile::TrustedAppleMetal,
        ShaderTarget::spirv => GpuCanvasShaderProfile::TrustedVulkanSpirV,
        ShaderTarget::hlsl => panic!("HLSL is not a supported product backend"),
    }
}

pub struct ExactGpuCanvasShaderOccurrence {
    pub profile: GpuCanvasShaderProfile,
    pub artifact: GpuCanvasShaderArtifact,
    pub modules: Vec<AnyResourceHandle>,
    pub execution_anchor: Rc<dyn Any>,
}

impl ExactGpuCanvasShaderOccurrence {
    pub fn compile(
        context: &mut dyn ContextApi,
        profile: GpuCanvasShaderProfile,
        artifact: &GpuCanvasShaderArtifact,
        execution_anchor: Rc<dyn Any>,
    ) -> Result<Self, GpuCanvasError> {
        context.clearLastError();
        let modules = match (profile, artifact) {
            (GpuCanvasShaderProfile::WebGpu, GpuCanvasShaderArtifact::WebGpu(shader)) => {
                vec![make_shader_module(
                    context,
                    shader.source.as_bytes(),
                    ShaderLanguage::wgsl,
                    ShaderStage::autoDetect,
                    &shader.binding_map_bytes,
                    &module_entry_bindings(&shader.entry_binding_bytes, &shader.entries),
                    None,
                    shader.shader_asset_id,
                    &shader.texture_sampler_pairs,
                    "trusted WGSL GPU-canvas module",
                )?]
            }
            (GpuCanvasShaderProfile::WebGl2, GpuCanvasShaderArtifact::WebGl2(shader)) => shader
                .entries
                .iter()
                .zip(&shader.sources)
                .map(|(entry, source)| {
                    let (stage, fixup) = match entry.stage {
                        GpuCanvasShaderStage::Vertex => {
                            (ShaderStage::vertex, shader.vertex_gl_fixup_bytes.as_ref())
                        }
                        GpuCanvasShaderStage::Fragment => (
                            ShaderStage::fragment,
                            shader.fragment_gl_fixup_bytes.as_ref(),
                        ),
                        GpuCanvasShaderStage::Compute => {
                            return Err(rejected("WebGL2 cannot compile compute entries"));
                        }
                    };
                    make_shader_module(
                        context,
                        source.as_bytes(),
                        ShaderLanguage::glsl,
                        stage,
                        &shader.binding_map_bytes,
                        &module_entry_bindings(
                            &shader.entry_binding_bytes,
                            std::slice::from_ref(entry),
                        ),
                        Some(fixup),
                        shader.shader_asset_id,
                        &shader.texture_sampler_pairs,
                        "trusted GLSL GPU-canvas entry",
                    )
                })
                .collect::<Result<Vec<_>, _>>()?,
            (
                GpuCanvasShaderProfile::TrustedAppleMetal,
                GpuCanvasShaderArtifact::TrustedAppleMetal(shader),
            ) => vec![make_shader_module(
                context,
                shader.source().as_bytes(),
                ShaderLanguage::glsl,
                ShaderStage::autoDetect,
                shader.binding_map_bytes(),
                &module_entry_bindings(shader.entry_binding_bytes(), shader.entries()),
                None,
                shader.shader_asset_id(),
                shader.texture_sampler_pairs(),
                "trusted MSL GPU-canvas module",
            )?],
            (
                GpuCanvasShaderProfile::TrustedVulkanSpirV,
                GpuCanvasShaderArtifact::TrustedVulkanSpirV(shader),
            ) => vec![make_shader_module(
                context,
                shader.code(),
                ShaderLanguage::glsl,
                ShaderStage::autoDetect,
                shader.binding_map_bytes(),
                &module_entry_bindings(shader.entry_binding_bytes(), shader.entries()),
                None,
                shader.shader_asset_id(),
                shader.texture_sampler_pairs(),
                "trusted SPIR-V GPU-canvas module",
            )?],
            (GpuCanvasShaderProfile::WebGpu, _) => {
                return Err(rejected("WebGPU factory requires the WebGPU RSTB target"));
            }
            (GpuCanvasShaderProfile::WebGl2, _) => {
                return Err(rejected("WebGL2 factory requires the WebGL2 RSTB target"));
            }
            (GpuCanvasShaderProfile::TrustedVulkanSpirV, _) => {
                return Err(rejected("Vulkan factory requires the SPIR-V RSTB target"));
            }
            (GpuCanvasShaderProfile::TrustedAppleMetal, _) => {
                return Err(rejected("Metal factory requires the MSL RSTB target"));
            }
        };
        Ok(Self {
            profile,
            artifact: artifact.clone(),
            modules,
            execution_anchor,
        })
    }

    pub fn entries(&self) -> &[GpuCanvasShaderEntry] {
        self.artifact.entries()
    }

    pub fn bindings(&self) -> &[GpuCanvasShaderBinding] {
        self.artifact.bindings()
    }

    pub fn module_for(
        &self,
        stage: GpuCanvasShaderStage,
        selection: Option<&GpuCanvasShaderEntrySelection>,
    ) -> Result<(&AnyResourceHandle, &GpuCanvasShaderEntry), GpuCanvasError> {
        let index = self
            .entries()
            .iter()
            .position(|entry| {
                entry.stage == stage
                    && selection.is_none_or(|selection| {
                        entry.logical_entry_point == selection.logical_entry_point
                            && entry.physical_entry_point == selection.physical_entry_point
                    })
            })
            .ok_or_else(|| rejected(format!("selected {stage:?} entry is stale or absent")))?;
        let module_index = if matches!(
            self.profile,
            GpuCanvasShaderProfile::WebGpu
                | GpuCanvasShaderProfile::TrustedVulkanSpirV
                | GpuCanvasShaderProfile::TrustedAppleMetal
        ) {
            0
        } else {
            index
        };
        let module = self
            .modules
            .get(module_index)
            .ok_or_else(|| rejected("physical shader module is absent"))?;
        Ok((module, &self.entries()[index]))
    }
}

impl RenderGpuCanvasShader for ExactGpuCanvasShaderOccurrence {
    fn ore_shader_entry(
        &self,
        stage: GpuCanvasShaderStage,
        physical_entry: &str,
    ) -> Option<AnyResourceHandle> {
        let entry = self
            .entries()
            .iter()
            .find(|entry| entry.stage == stage && entry.physical_entry_point == physical_entry)?;
        let selection = GpuCanvasShaderEntrySelection {
            logical_entry_point: entry.logical_entry_point.clone(),
            physical_entry_point: entry.physical_entry_point.clone(),
        };
        self.module_for(stage, Some(&selection))
            .ok()
            .map(|(module, _)| module.clone())
    }
    fn as_any(&self) -> &dyn Any {
        self
    }
}

fn make_shader_module(
    context: &mut dyn ContextApi,
    source: &[u8],
    language: ShaderLanguage,
    stage: ShaderStage,
    binding_map: &[u8],
    entry_bindings: &[u8],
    gl_fixup: Option<&[u8]>,
    shader_asset_id: u32,
    texture_sampler_pairs: &[GpuCanvasShaderTextureSamplerPair],
    label: &'static str,
) -> Result<AnyResourceHandle, GpuCanvasError> {
    let code_size = u32::try_from(source.len()).map_err(|_| rejected("shader exceeds u32"))?;
    let binding_map_size =
        u32::try_from(binding_map.len()).map_err(|_| rejected("binding map exceeds u32"))?;
    let gl_fixup_size = u32::try_from(gl_fixup.map_or(0, <[u8]>::len))
        .map_err(|_| rejected("GL fixup exceeds u32"))?;
    // Texture-sampler pairs travel in the descriptor so a recording context
    // captures them before deferred replay rebuilds the module.
    let pair_bytes: Vec<u8> = texture_sampler_pairs
        .iter()
        .flat_map(|pair| {
            [
                pair.texture_group,
                pair.texture_binding,
                pair.sampler_group,
                pair.sampler_binding,
            ]
        })
        .collect();
    let pair_size =
        u32::try_from(pair_bytes.len()).map_err(|_| rejected("shader pairs exceed u32"))?;
    context
        .makeShaderModule(&ShaderModuleDesc {
            code: Some(source),
            codeSize: code_size,
            language,
            stage,
            bindingMapBytes: Some(binding_map),
            bindingMapSize: binding_map_size,
            entryBindingBytes: (!entry_bindings.is_empty()).then_some(entry_bindings),
            entryBindingSize: u32::try_from(entry_bindings.len())
                .map_err(|_| rejected("entry bindings exceed u32"))?,
            texSamplerPairBytes: (!pair_bytes.is_empty()).then_some(pair_bytes.as_slice()),
            texSamplerPairSize: pair_size,
            glFixupBytes: gl_fixup,
            glFixupSize: gl_fixup_size,
            shaderAssetId: shader_asset_id,
            label: Some(label),
            ..ShaderModuleDesc::default()
        })
        .ok_or_else(|| rejected(context_error(context, "compile trusted shader module")))
}
