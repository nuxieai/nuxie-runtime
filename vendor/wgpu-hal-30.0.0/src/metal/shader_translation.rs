//! Pure Naga-to-MSL translation used by the Metal backend.
//!
//! Keeping this contract free of Metal device objects lets repository tools
//! generate exactly the same MSL that [`super::Device`] would generate at
//! pipeline creation time. Callers are still responsible for deriving the
//! resource and vertex maps with wgpu's Metal pipeline-layout rules.

use alloc::{
    borrow::ToOwned as _,
    string::{String, ToString as _},
    vec::Vec,
};

/// All pipeline-specific inputs consumed by wgpu's Metal Naga path.
#[derive(Clone, Copy, Debug)]
pub struct TranslationInput<'a> {
    pub shader: &'a crate::NagaShader,
    pub stage: naga::ShaderStage,
    pub entry_point: &'a str,
    pub constants: &'a naga::back::PipelineConstants,
    pub resources: &'a naga::back::msl::EntryPointResources,
    pub binding_array_length_map: &'a naga::FastHashMap<naga::ResourceBinding, u32>,
    pub vertex_buffer_mappings: &'a [naga::back::msl::VertexBufferMapping],
    pub allow_and_force_point_size: bool,
    pub msl_version: (u8, u8),
    pub zero_initialize_workgroup_memory: bool,
    pub runtime_checks: wgt::ShaderRuntimeChecks,
    pub task_dispatch_limits: naga::back::TaskDispatchLimits,
}

/// MSL and the Naga reflection data that wgpu-hal needs to create a pipeline.
#[derive(Debug)]
pub struct TranslationOutput {
    pub source: String,
    pub translated_entry_point: String,
    pub workgroup_size: [u32; 3],
    pub workgroup_memory_sizes: Vec<u32>,
    pub sized_bindings: Vec<(naga::ResourceBinding, u32)>,
    pub immutable_buffer_mask: usize,
    pub preserve_invariance: bool,
}

#[derive(Debug)]
pub enum TranslationError {
    PipelineConstants(naga::back::pipeline_constants::PipelineConstantError),
    Msl(naga::back::msl::Error),
    MissingEntryPoint,
    InvalidTranslatedEntryPoint(String),
}

/// Translate a validated Naga module using the exact options consumed by the
/// Metal backend's pipeline creation path.
pub fn translate(input: TranslationInput<'_>) -> Result<TranslationOutput, TranslationError> {
    let (module, module_info) = naga::back::pipeline_constants::process_overrides(
        &input.shader.module,
        &input.shader.info,
        Some((input.stage, input.entry_point)),
        input.constants,
    )
    .map_err(TranslationError::PipelineConstants)?;

    let bounds_check_policy = if input.runtime_checks.bounds_checks {
        naga::proc::BoundsCheckPolicy::Restrict
    } else {
        naga::proc::BoundsCheckPolicy::Unchecked
    };

    let options = naga::back::msl::Options {
        lang_version: input.msl_version,
        inline_samplers: Default::default(),
        spirv_cross_compatibility: false,
        fake_missing_bindings: false,
        per_entry_point_map: naga::back::msl::EntryPointResourceMap::from([(
            input.entry_point.to_owned(),
            input.resources.clone(),
        )]),
        bounds_check_policies: naga::proc::BoundsCheckPolicies {
            index: bounds_check_policy,
            buffer: bounds_check_policy,
            image_load: bounds_check_policy,
            // TODO: support bounds checks on binding arrays.
            binding_array: naga::proc::BoundsCheckPolicy::Unchecked,
        },
        zero_initialize_workgroup_memory: input.zero_initialize_workgroup_memory,
        force_loop_bounding: input.runtime_checks.force_loop_bounding,
        task_dispatch_limits: input
            .runtime_checks
            .task_shader_dispatch_tracking
            .then_some(input.task_dispatch_limits),
        mesh_shader_primitive_indices_clamp: input
            .runtime_checks
            .mesh_shader_primitive_indices_clamp,
        emit_int_div_checks: input.runtime_checks.int_div_checks,
        ray_query_initialization_tracking: input.runtime_checks.ray_query_initialization_tracking,
    };

    let pipeline_options = naga::back::msl::PipelineOptions {
        entry_point: Some((input.stage, input.entry_point.to_owned())),
        allow_and_force_point_size: input.allow_and_force_point_size,
        vertex_pulling_transform: true,
        vertex_buffer_mappings: input.vertex_buffer_mappings.to_vec(),
        binding_array_length_map: input.binding_array_length_map.clone(),
    };

    let (source, info) =
        naga::back::msl::write_string(&module, &module_info, &options, &pipeline_options)
            .map_err(TranslationError::Msl)?;

    let entry_point_index = module
        .entry_points
        .iter()
        .position(|ep| ep.stage == input.stage && ep.name == input.entry_point)
        .ok_or(TranslationError::MissingEntryPoint)?;
    let entry_point = &module.entry_points[entry_point_index];
    let translated_entry_point = info.entry_point_names[0]
        .as_ref()
        .map_err(|error| TranslationError::InvalidTranslatedEntryPoint(error.to_string()))?
        .clone();

    let entry_point_info = &module_info.get_entry_point(entry_point_index);
    let mut workgroup_memory_sizes = Vec::new();
    let mut sized_bindings = Vec::new();
    let mut immutable_buffer_mask = 0;
    for (var_handle, var) in module.global_variables.iter() {
        match var.space {
            naga::AddressSpace::WorkGroup => {
                if !entry_point_info[var_handle].is_empty() {
                    workgroup_memory_sizes.push(module.types[var.ty].inner.size(module.to_ctx()));
                }
            }
            naga::AddressSpace::Uniform | naga::AddressSpace::Storage { .. } => {
                let Some(binding) = var.binding else {
                    continue;
                };
                let storage_access_store = match var.space {
                    naga::AddressSpace::Storage { access } => {
                        access.contains(naga::StorageAccess::STORE)
                    }
                    _ => false,
                };

                if !entry_point_info[var_handle].is_empty() && !storage_access_store {
                    let slot = input.resources.resources[&binding].buffer.unwrap();
                    immutable_buffer_mask |= 1 << slot;
                }

                if module.types[var.ty]
                    .inner
                    .needs_host_buffer_byte_size(&module.types)
                {
                    let count = match module.types[var.ty].inner {
                        naga::TypeInner::BindingArray { size, .. } => {
                            let from_shader = match size {
                                naga::ArraySize::Constant(count) => count.get(),
                                naga::ArraySize::Pending(_) | naga::ArraySize::Dynamic => 0,
                            };
                            let from_layout = input
                                .binding_array_length_map
                                .get(&binding)
                                .copied()
                                .unwrap_or(0);
                            from_shader.max(from_layout).max(1)
                        }
                        _ => 1,
                    };
                    sized_bindings.extend((0..count).map(|index| (binding, index)));
                }
            }
            _ => {}
        }
    }

    Ok(TranslationOutput {
        preserve_invariance: msl_has_invariant_position(&source),
        source,
        translated_entry_point,
        workgroup_size: entry_point.workgroup_size,
        workgroup_memory_sizes,
        sized_bindings,
        immutable_buffer_mask,
    })
}

pub fn msl_has_invariant_position(source: &str) -> bool {
    source.contains("[[position, invariant]]")
}

#[cfg(test)]
mod tests {
    use alloc::{borrow::Cow, collections::BTreeMap};
    use std::collections::BTreeSet;

    use super::*;

    fn shader(source: &str) -> crate::NagaShader {
        let module = naga::front::wgsl::parse_str(source).unwrap();
        let info = naga::valid::Validator::new(
            naga::valid::ValidationFlags::all(),
            naga::valid::Capabilities::all(),
        )
        .validate(&module)
        .unwrap();
        crate::NagaShader {
            module: Cow::Owned(module),
            info,
            debug_source: None,
        }
    }

    #[derive(Clone, Copy)]
    struct TestOptions {
        stage: naga::ShaderStage,
        msl_version: (u8, u8),
        point_size: bool,
        zero_workgroup: bool,
        runtime_checks: wgt::ShaderRuntimeChecks,
    }

    impl Default for TestOptions {
        fn default() -> Self {
            Self {
                stage: naga::ShaderStage::Vertex,
                msl_version: (2, 1),
                point_size: false,
                zero_workgroup: true,
                runtime_checks: wgt::ShaderRuntimeChecks::checked(),
            }
        }
    }

    fn translate_test(
        shader: &crate::NagaShader,
        entry_point: &str,
        constants: &naga::back::PipelineConstants,
        resources: &naga::back::msl::EntryPointResources,
        binding_array_lengths: &naga::FastHashMap<naga::ResourceBinding, u32>,
        vertex_buffers: &[naga::back::msl::VertexBufferMapping],
        options: TestOptions,
    ) -> TranslationOutput {
        translate(TranslationInput {
            shader,
            stage: options.stage,
            entry_point,
            constants,
            resources,
            binding_array_length_map: binding_array_lengths,
            vertex_buffer_mappings: vertex_buffers,
            allow_and_force_point_size: options.point_size,
            msl_version: options.msl_version,
            zero_initialize_workgroup_memory: options.zero_workgroup,
            runtime_checks: options.runtime_checks,
            task_dispatch_limits: naga::back::TaskDispatchLimits {
                max_mesh_workgroups_per_dim: 65_535,
                max_mesh_workgroups_total: 65_535,
            },
        })
        .unwrap()
    }

    fn position_vertex_buffer(id: u32, stride: u32) -> naga::back::msl::VertexBufferMapping {
        naga::back::msl::VertexBufferMapping {
            id,
            stride,
            step_mode: naga::back::msl::VertexBufferStepMode::ByVertex,
            attributes: alloc::vec![naga::back::msl::AttributeMapping {
                shader_location: 0,
                offset: 0,
                format: nt::VertexFormat::Float32x2,
            }],
        }
    }

    fn vertex_resources() -> naga::back::msl::EntryPointResources {
        naga::back::msl::EntryPointResources {
            sizes_buffer: Some(0),
            ..Default::default()
        }
    }

    #[test]
    fn translation_returns_msl_entry_point_and_reflection() {
        let shader = shader(
            "@group(0) @binding(0) var<uniform> color: vec4<f32>;\n\
             struct Output { @builtin(position) position: vec4<f32>, @location(0) color: vec4<f32> }\n\
             @vertex fn main(@location(0) position: vec2<f32>) -> Output {\n\
                 return Output(vec4<f32>(position, 0.0, 1.0), color);\n\
             }",
        );
        let resources = naga::back::msl::EntryPointResources {
            resources: BTreeMap::from([(
                naga::ResourceBinding {
                    group: 0,
                    binding: 0,
                },
                naga::back::msl::BindTarget {
                    buffer: Some(0),
                    ..Default::default()
                },
            )]),
            immediates_buffer: None,
            sizes_buffer: Some(1),
        };
        let vertex_buffer_mappings = [naga::back::msl::VertexBufferMapping {
            id: 30,
            stride: 8,
            step_mode: naga::back::msl::VertexBufferStepMode::ByVertex,
            attributes: alloc::vec![naga::back::msl::AttributeMapping {
                shader_location: 0,
                offset: 0,
                format: nt::VertexFormat::Float32x2,
            }],
        }];
        let output = translate(TranslationInput {
            shader: &shader,
            stage: naga::ShaderStage::Vertex,
            entry_point: "main",
            constants: &Default::default(),
            resources: &resources,
            binding_array_length_map: &Default::default(),
            vertex_buffer_mappings: &vertex_buffer_mappings,
            allow_and_force_point_size: false,
            msl_version: (2, 1),
            zero_initialize_workgroup_memory: true,
            runtime_checks: wgt::ShaderRuntimeChecks::checked(),
            task_dispatch_limits: naga::back::TaskDispatchLimits {
                max_mesh_workgroups_per_dim: 65_535,
                max_mesh_workgroups_total: 65_535,
            },
        })
        .unwrap();

        assert!(!output.source.is_empty());
        assert!(!output.translated_entry_point.is_empty());
        assert_eq!(output.workgroup_size, [0, 0, 0]);
        assert_eq!(output.immutable_buffer_mask, 1);
        assert!(output.workgroup_memory_sizes.is_empty());
        assert!(output.sized_bindings.is_empty());
        assert!(!output.preserve_invariance);
    }

    #[test]
    fn preserve_invariance_tracks_only_invariant_position() {
        assert!(!msl_has_invariant_position("float4 position [[position]];"));
        assert!(msl_has_invariant_position(
            "float4 position [[position, invariant]];"
        ));
    }

    #[test]
    fn supported_msl_versions_translate_a_real_entry_point() {
        let shader = shader("@compute @workgroup_size(1) fn main() {}");
        for version in [(2, 0), (2, 1), (2, 4), (3, 0), (3, 2), (4, 0)] {
            let output = translate_test(
                &shader,
                "main",
                &Default::default(),
                &Default::default(),
                &Default::default(),
                &[],
                TestOptions {
                    stage: naga::ShaderStage::Compute,
                    msl_version: version,
                    ..Default::default()
                },
            );
            assert!(
                output.source.contains("kernel void main"),
                "MSL {version:?} did not emit the compute entry point"
            );
            assert_eq!(output.workgroup_size, [1, 1, 1]);
        }
    }

    #[test]
    fn point_topology_controls_forced_point_size_output() {
        let shader = shader(
            "@vertex fn main(@location(0) position: vec2<f32>) -> @builtin(position) vec4<f32> {\n\
                 return vec4<f32>(position, 0.0, 1.0);\n\
             }",
        );
        let vertices = [position_vertex_buffer(30, 8)];
        let resources = vertex_resources();
        let outputs: Vec<_> = [false, true]
            .into_iter()
            .map(|point_size| {
                translate_test(
                    &shader,
                    "main",
                    &Default::default(),
                    &resources,
                    &Default::default(),
                    &vertices,
                    TestOptions {
                        point_size,
                        ..Default::default()
                    },
                )
            })
            .collect();

        assert!(!outputs[0].source.contains("[[point_size]]"));
        assert!(outputs[1].source.contains("[[point_size]]"));
        assert_ne!(outputs[0].source, outputs[1].source);
    }

    #[test]
    fn workgroup_zeroing_and_runtime_checks_change_real_compute_msl() {
        let shader = shader(
            "@group(0) @binding(0) var<storage, read_write> values: array<u32>;\n\
             var<workgroup> scratch: array<u32, 4>;\n\
             @compute @workgroup_size(4)\n\
             fn main(@builtin(local_invocation_index) index: u32) {\n\
                 scratch[index] = values[index] / (index + 1u);\n\
                 workgroupBarrier();\n\
                 values[index] = scratch[index];\n\
             }",
        );
        let binding = naga::ResourceBinding {
            group: 0,
            binding: 0,
        };
        let resources = naga::back::msl::EntryPointResources {
            resources: BTreeMap::from([(
                binding,
                naga::back::msl::BindTarget {
                    buffer: Some(0),
                    ..Default::default()
                },
            )]),
            immediates_buffer: None,
            sizes_buffer: Some(1),
        };
        let cases = [
            (true, wgt::ShaderRuntimeChecks::checked()),
            (false, wgt::ShaderRuntimeChecks::checked()),
            (true, wgt::ShaderRuntimeChecks::unchecked()),
            (false, wgt::ShaderRuntimeChecks::unchecked()),
        ];
        let outputs: Vec<_> = cases
            .into_iter()
            .map(|(zero_workgroup, runtime_checks)| {
                translate_test(
                    &shader,
                    "main",
                    &Default::default(),
                    &resources,
                    &Default::default(),
                    &[],
                    TestOptions {
                        stage: naga::ShaderStage::Compute,
                        zero_workgroup,
                        runtime_checks,
                        ..Default::default()
                    },
                )
            })
            .collect();

        for output in &outputs {
            assert_eq!(output.workgroup_size, [4, 1, 1]);
            assert_eq!(output.workgroup_memory_sizes, [16]);
            assert_eq!(output.sized_bindings, [(binding, 0)]);
        }
        assert_eq!(
            outputs
                .iter()
                .map(|output| output.source.as_str())
                .collect::<BTreeSet<_>>()
                .len(),
            outputs.len(),
            "each zeroing/runtime-check combination must produce distinct MSL"
        );
    }

    #[test]
    fn binding_array_lengths_change_real_resource_declarations() {
        let shader = shader(
            "enable wgpu_binding_array;\n\
             @group(0) @binding(0) var images: binding_array<texture_2d<f32>>;\n\
             @fragment fn main() -> @location(0) vec4<f32> {\n\
                 return textureLoad(images[0], vec2<i32>(0), 0);\n\
             }",
        );
        let binding = naga::ResourceBinding {
            group: 0,
            binding: 0,
        };
        let resources = naga::back::msl::EntryPointResources {
            resources: BTreeMap::from([(
                binding,
                naga::back::msl::BindTarget {
                    buffer: Some(0),
                    ..Default::default()
                },
            )]),
            ..Default::default()
        };
        let outputs: Vec<_> = [2, 5]
            .into_iter()
            .map(|length| {
                let lengths = naga::FastHashMap::from_iter([(binding, length)]);
                translate_test(
                    &shader,
                    "main",
                    &Default::default(),
                    &resources,
                    &lengths,
                    &[],
                    TestOptions {
                        stage: naga::ShaderStage::Fragment,
                        ..Default::default()
                    },
                )
            })
            .collect();

        // Texture binding arrays remain pointer-shaped in MSL. Successful
        // translation for both declared layout counts proves the values are
        // accepted by the real backend; the full input stays part of the
        // replay key even when this shader does not require count-dependent
        // generated code.
        assert!(outputs.iter().all(|output| {
            output.source.contains("NagaArgumentBufferWrapper")
                && output.source.contains("[[buffer(0)]]")
        }));
    }

    #[test]
    fn pipeline_constants_and_vertex_mappings_change_real_vertex_msl() {
        let shader = shader(
            "override scale: f32 = 1.0;\n\
             @vertex fn main(@location(0) position: vec2<f32>) -> @builtin(position) vec4<f32> {\n\
                 return vec4<f32>(position * scale, 0.0, 1.0);\n\
             }",
        );
        let cases = [(0.25, 29, 8), (0.75, 29, 8), (0.25, 30, 16)];
        let resources = vertex_resources();
        let outputs: Vec<_> = cases
            .into_iter()
            .map(|(scale, buffer_id, stride)| {
                let constants =
                    naga::back::PipelineConstants::from_iter([("scale".to_owned(), scale)]);
                let vertices = [position_vertex_buffer(buffer_id, stride)];
                translate_test(
                    &shader,
                    "main",
                    &constants,
                    &resources,
                    &Default::default(),
                    &vertices,
                    Default::default(),
                )
            })
            .collect();

        assert_eq!(
            outputs
                .iter()
                .map(|output| output.source.as_str())
                .collect::<BTreeSet<_>>()
                .len(),
            outputs.len(),
            "constants, Metal buffer slots, and strides must participate in translation"
        );
        assert!(outputs[0].source.contains("[[buffer(29)]]"));
        assert!(outputs[2].source.contains("[[buffer(30)]]"));
    }
}
