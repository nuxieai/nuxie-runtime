//! Direct translation of ore_webgpu_parity_test.cpp at upstream 1d33ee5d.
use super::ore_deferred_context::DeferredOreContext;
use nuxie_ore_metal::{
    bind_group_layout::{
        NativeSlotScope, makeBindGroupLayoutFromBindingMap, validatePipelineDesc,
        validateStageLimits,
    },
    binding_map::{
        BindingMap, Entry, ResourceKind, TextureSampleType, TextureSamplerPair, TextureViewDim,
    },
    context::{ContextApi, ReplayCaps},
    gpu_resource::{AnyResourceHandle, ResourceHandle},
    types::*,
};

fn entry(binding: u32, kind: BindingKind) -> BindGroupLayoutEntry {
    BindGroupLayoutEntry {
        binding,
        kind,
        ..Default::default()
    }
}
fn texture(
    binding: u32,
    sample: SampleType,
    dimension: TextureViewDimension,
) -> BindGroupLayoutEntry {
    BindGroupLayoutEntry {
        textureSampleType: sample,
        textureViewDim: dimension,
        ..entry(binding, BindingKind::sampledTexture)
    }
}
fn float_texture(binding: u32) -> BindGroupLayoutEntry {
    texture(
        binding,
        SampleType::floatFilterable,
        TextureViewDimension::texture2D,
    )
}
fn layout_of(ctx: &mut DeferredOreContext, entries: &[BindGroupLayoutEntry]) -> AnyResourceHandle {
    ctx.makeBindGroupLayout(&BindGroupLayoutDesc {
        entries: Some(entries),
        entryCount: entries.len() as u32,
        ..Default::default()
    })
    .unwrap()
}
fn view_of(
    ctx: &mut DeferredOreContext,
    format: TextureFormat,
    sample_count: u32,
    aspect: TextureAspect,
) -> AnyResourceHandle {
    let tex = ctx
        .makeTexture(&TextureDesc {
            width: 4,
            height: 4,
            format,
            sampleCount: sample_count,
            ..Default::default()
        })
        .unwrap();
    ctx.makeTextureView(&TextureViewDesc {
        texture: Some(&tex),
        aspect,
        ..Default::default()
    })
    .unwrap()
}
fn texture_error(
    ctx: &mut DeferredOreContext,
    layout_entry: BindGroupLayoutEntry,
    view: &AnyResourceHandle,
) -> String {
    let layout = layout_of(ctx, &[layout_entry]);
    let textures = [TexEntry {
        slot: layout_entry.binding,
        view: Some(view),
    }];
    let desc = BindGroupDesc {
        layout: Some(&layout),
        textures: &textures,
        textureCount: 1,
        ..Default::default()
    };
    ctx.clearLastError();
    if ctx.makeBindGroup(&desc).is_some() {
        String::new()
    } else {
        ctx.lastError()
    }
}
fn sampler_error(
    ctx: &mut DeferredOreContext,
    layout_entry: BindGroupLayoutEntry,
    sampler_desc: &SamplerDesc<'_>,
) -> String {
    let layout = layout_of(ctx, &[layout_entry]);
    let sampler = ctx.makeSampler(sampler_desc).unwrap();
    let samplers = [SampEntry {
        slot: layout_entry.binding,
        sampler: Some(&sampler),
    }];
    let desc = BindGroupDesc {
        layout: Some(&layout),
        samplers: &samplers,
        samplerCount: 1,
        ..Default::default()
    };
    ctx.clearLastError();
    if ctx.makeBindGroup(&desc).is_some() {
        String::new()
    } else {
        ctx.lastError()
    }
}

#[test]
fn a_bind_group_fills_every_layout_entry_exactly_once() {
    let mut ctx = DeferredOreContext::new(ReplayCaps::default());
    let layout = layout_of(
        &mut ctx,
        &[
            entry(0, BindingKind::uniformBuffer),
            float_texture(1),
            entry(2, BindingKind::sampler),
        ],
    );
    let buffer = ctx
        .makeBuffer(&BufferDesc::uninitialized(BufferUsage::uniform, 256))
        .unwrap();
    let ubos = [UBOEntry {
        slot: 0,
        buffer: Some(&buffer),
        offset: 0,
        size: 0,
    }];
    let mut desc = BindGroupDesc {
        layout: Some(&layout),
        ubos: &ubos,
        uboCount: 1,
        ..Default::default()
    };
    assert!(ctx.makeBindGroup(&desc).is_none());
    assert!(ctx.lastError().contains("@group(0) @binding(1)"));
    assert!(ctx.lastError().contains("leaves this texture empty"));
    let view = view_of(&mut ctx, TextureFormat::rgba8unorm, 1, TextureAspect::all);
    let sampler = ctx.makeSampler(&SamplerDesc::default()).unwrap();
    let textures = [TexEntry {
        slot: 1,
        view: Some(&view),
    }];
    let samplers = [
        SampEntry {
            slot: 2,
            sampler: Some(&sampler),
        },
        SampEntry {
            slot: 2,
            sampler: Some(&sampler),
        },
    ];
    desc.textures = &textures;
    desc.textureCount = 1;
    desc.samplers = &samplers;
    desc.samplerCount = 2;
    assert!(ctx.makeBindGroup(&desc).is_none());
    assert!(ctx.lastError().contains("more than once"));
    desc.samplerCount = 1;
    ctx.clearLastError();
    assert!(ctx.makeBindGroup(&desc).is_some());
}

#[test]
fn an_entry_the_layout_lacks_is_refused() {
    let mut ctx = DeferredOreContext::new(ReplayCaps::default());
    let layout = layout_of(&mut ctx, &[entry(0, BindingKind::sampler)]);
    let sampler = ctx.makeSampler(&SamplerDesc::default()).unwrap();
    let samplers = [
        SampEntry {
            slot: 0,
            sampler: Some(&sampler),
        },
        SampEntry {
            slot: 1,
            sampler: Some(&sampler),
        },
    ];
    let desc = BindGroupDesc {
        layout: Some(&layout),
        samplers: &samplers,
        samplerCount: 2,
        ..Default::default()
    };
    assert!(ctx.makeBindGroup(&desc).is_none());
    assert!(
        ctx.lastError()
            .contains("@binding(1): the layout has no sampler")
    );
}

#[test]
fn uniform_buffers_need_uniform_usage_and_256_byte_offsets() {
    let mut ctx = DeferredOreContext::new(ReplayCaps::default());
    let layout = layout_of(&mut ctx, &[entry(0, BindingKind::uniformBuffer)]);
    let mut bind = |usage, offset| {
        let buffer = ctx
            .makeBuffer(&BufferDesc::uninitialized(usage, 1024))
            .unwrap();
        let ubos = [UBOEntry {
            slot: 0,
            buffer: Some(&buffer),
            offset,
            size: 64,
        }];
        let desc = BindGroupDesc {
            layout: Some(&layout),
            ubos: &ubos,
            uboCount: 1,
            ..Default::default()
        };
        ctx.clearLastError();
        if ctx.makeBindGroup(&desc).is_some() {
            String::new()
        } else {
            ctx.lastError()
        }
    };
    assert!(bind(BufferUsage::uniform, 256).is_empty());
    assert!(bind(BufferUsage::uniform, 64).contains("not a multiple of 256"));
    assert!(bind(BufferUsage::vertex, 0).contains("without uniform usage"));
}

#[test]
fn samplers_fill_only_bindings_of_their_own_type() {
    let mut ctx = DeferredOreContext::new(ReplayCaps::default());
    let linear = SamplerDesc {
        minFilter: Filter::linear,
        magFilter: Filter::linear,
        ..Default::default()
    };
    let compare = SamplerDesc {
        compare: CompareFunction::less,
        ..Default::default()
    };
    let linear_compare = SamplerDesc {
        minFilter: Filter::linear,
        magFilter: Filter::linear,
        compare: CompareFunction::less,
        ..Default::default()
    };
    let non_filtering = BindGroupLayoutEntry {
        samplerNonFiltering: true,
        ..entry(0, BindingKind::sampler)
    };
    assert!(sampler_error(&mut ctx, non_filtering, &linear).contains("give it nearest filters"));
    assert!(sampler_error(&mut ctx, non_filtering, &SamplerDesc::default()).is_empty());
    let filtering = entry(0, BindingKind::sampler);
    assert!(sampler_error(&mut ctx, filtering, &linear).is_empty());
    assert!(sampler_error(&mut ctx, filtering, &compare).contains("binds a comparison sampler"));
    let comparison = entry(0, BindingKind::comparisonSampler);
    assert!(
        sampler_error(&mut ctx, comparison, &SamplerDesc::default())
            .contains("has no compare function")
    );
    assert!(sampler_error(&mut ctx, comparison, &linear_compare).is_empty());
}

#[test]
fn a_view_fills_only_entries_its_format_can_be_read_as() {
    let mut ctx = DeferredOreContext::new(ReplayCaps::default());
    let r32 = view_of(&mut ctx, TextureFormat::r32float, 1, TextureAspect::all);
    assert!(
        texture_error(&mut ctx, float_texture(0), &r32)
            .contains("cannot read this view's format as a float texture")
    );
    assert!(
        texture_error(
            &mut ctx,
            texture(
                0,
                SampleType::floatUnfilterable,
                TextureViewDimension::texture2D
            ),
            &r32
        )
        .is_empty()
    );
    let depth = view_of(&mut ctx, TextureFormat::depth32float, 1, TextureAspect::all);
    assert!(
        texture_error(
            &mut ctx,
            texture(0, SampleType::depth, TextureViewDimension::texture2D),
            &depth
        )
        .is_empty()
    );
    assert!(
        texture_error(
            &mut ctx,
            texture(
                0,
                SampleType::floatUnfilterable,
                TextureViewDimension::texture2D
            ),
            &depth
        )
        .is_empty()
    );
    assert!(texture_error(&mut ctx, float_texture(0), &depth).contains("as a float texture"));
    let color = view_of(&mut ctx, TextureFormat::rgba8unorm, 1, TextureAspect::all);
    assert!(
        texture_error(
            &mut ctx,
            texture(0, SampleType::depth, TextureViewDimension::texture2D),
            &color
        )
        .contains("as a depth texture")
    );
    let both = view_of(
        &mut ctx,
        TextureFormat::depth24plusStencil8,
        1,
        TextureAspect::all,
    );
    assert!(
        texture_error(
            &mut ctx,
            texture(0, SampleType::depth, TextureViewDimension::texture2D),
            &both
        )
        .contains("both depth and stencil")
    );
    let depth_only = view_of(
        &mut ctx,
        TextureFormat::depth24plusStencil8,
        1,
        TextureAspect::depthOnly,
    );
    assert!(
        texture_error(
            &mut ctx,
            texture(0, SampleType::depth, TextureViewDimension::texture2D),
            &depth_only
        )
        .is_empty()
    );
}

#[test]
fn a_view_matches_the_declared_dimension_and_sample_count() {
    let mut ctx = DeferredOreContext::new(ReplayCaps::default());
    let flat = view_of(&mut ctx, TextureFormat::rgba8unorm, 1, TextureAspect::all);
    assert!(
        texture_error(
            &mut ctx,
            texture(
                0,
                SampleType::floatFilterable,
                TextureViewDimension::array2D
            ),
            &flat
        )
        .contains("binds a 2d view where the shader declares a 2d array")
    );
    let msaa = view_of(&mut ctx, TextureFormat::rgba8unorm, 4, TextureAspect::all);
    assert!(texture_error(&mut ctx, float_texture(0), &msaa).contains("binds a multisampled view"));
}

#[test]
fn pipeline_layouts_stay_within_webgpus_per_stage_limits() {
    let mut samplers: Vec<_> = (0..17).map(|i| entry(i, BindingKind::sampler)).collect();
    let mut ctx = DeferredOreContext::new(ReplayCaps::default());
    let wide = layout_of(&mut ctx, &samplers);
    let mut error = String::new();
    assert!(!validateStageLimits(
        Some(&[Some(&wide)]),
        1,
        Some(&mut error)
    ));
    assert!(error.contains("vertex stage 17 samplers"));
    samplers.pop();
    let full = layout_of(&mut ctx, &samplers);
    assert!(validateStageLimits(
        Some(&[Some(&full)]),
        1,
        Some(&mut error)
    ));
    let dynamic: Vec<_> = (0..9)
        .map(|i| BindGroupLayoutEntry {
            hasDynamicOffset: true,
            ..entry(i, BindingKind::uniformBuffer)
        })
        .collect();
    let offsets = layout_of(&mut ctx, &dynamic);
    assert!(!validateStageLimits(
        Some(&[Some(&offsets)]),
        1,
        Some(&mut error)
    ));
    assert!(error.contains("9 dynamic uniform buffers"));
}

#[cfg(feature = "with-rive-tools")]
#[test]
fn float_textures_no_sampler_reads_become_unfilterable() {
    let mut bm = BindingMap::default();
    for (binding, kind) in [
        (0, ResourceKind::SampledTexture),
        (1, ResourceKind::SampledTexture),
        (2, ResourceKind::Sampler),
    ] {
        let mut e = Entry {
            group: 0,
            binding,
            kind,
            stageMask: BindingMap::kStageFragment as u8,
            ..Default::default()
        };
        e.backendSlot[1] = u16::from(binding);
        if kind == ResourceKind::SampledTexture {
            e.textureViewDim = TextureViewDim::D2;
            e.textureSampleType = TextureSampleType::Float;
        }
        bm.push(&e);
    }
    bm.finalize();
    bm.computeLayoutIds();
    let mut sampled_first = nuxie_ore_metal::new_shader_module_backend_base();
    sampled_first.m_textureSamplerPairs = vec![TextureSamplerPair {
        textureGroup: 0,
        textureBinding: 0,
        samplerGroup: 0,
        samplerBinding: 2,
    }];
    let sample_type_of = |layout: &AnyResourceHandle, binding| {
        layout
            .bindGroupLayoutBase()
            .unwrap()
            .findEntry(binding)
            .unwrap()
            .textureSampleType
    };
    let mut ctx = DeferredOreContext::new(ReplayCaps::default());
    let inferred =
        makeBindGroupLayoutFromBindingMap(&mut ctx, &bm, 0, &[], None, Some(&sampled_first))
            .unwrap();
    assert!(sample_type_of(&inferred, 0) == SampleType::floatFilterable);
    assert!(sample_type_of(&inferred, 1) == SampleType::floatUnfilterable);
    let unknown = makeBindGroupLayoutFromBindingMap(&mut ctx, &bm, 0, &[], None, None).unwrap();
    assert!(sample_type_of(&unknown, 1) == SampleType::floatFilterable);
    assert!(!unknown.ptr_eq(&inferred));
}

#[test]
fn a_pipeline_cannot_sample_an_unfilterable_texture_with_filtering() {
    let mut shader = nuxie_ore_metal::new_shader_module_backend_base();
    shader.m_textureSamplerPairs = vec![TextureSamplerPair {
        textureGroup: 0,
        textureBinding: 1,
        samplerGroup: 0,
        samplerBinding: 2,
    }];
    let shader = ResourceHandle::new(None, shader).erase();
    let mut ctx = DeferredOreContext::new(ReplayCaps::default());
    let mut validate = |sample, non_filtering| {
        let samp = BindGroupLayoutEntry {
            samplerNonFiltering: non_filtering,
            ..entry(2, BindingKind::sampler)
        };
        let layout = layout_of(
            &mut ctx,
            &[texture(1, sample, TextureViewDimension::texture2D), samp],
        );
        let layouts = [Some(&layout)];
        let desc = PipelineDesc {
            vertexModule: Some(&shader),
            fragmentModule: Some(&shader),
            bindGroupLayouts: Some(&layouts),
            bindGroupLayoutCount: 1,
            ..Default::default()
        };
        let mut error = String::new();
        validatePipelineDesc(
            &desc,
            &BindingMap::default(),
            NativeSlotScope::perStage,
            Some(&mut error),
        );
        error
    };
    assert!(
        validate(SampleType::floatUnfilterable, false)
            .contains("@binding(1): the layout declares this texture unfilterable")
    );
    assert!(validate(SampleType::floatUnfilterable, true).is_empty());
    assert!(validate(SampleType::floatFilterable, false).is_empty());
}
