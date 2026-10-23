//! Upstream tests/unit_tests/renderer/ore_entry_layout_test.cpp at 8920a6b0.
use super::ore_deferred_context::DeferredOreContext;
use nuxie_ore_metal::{
    bind_group_layout::*,
    binding_map::{BindingMap, Entry, ResourceKind, TextureSampleType, TextureViewDim},
    context::ContextApi,
    draw_bindings::DrawBindings,
    gpu_resource::{AnyResourceHandle, ResourceHandle},
    rstb_entry_bindings::*,
    rstb_entry_container::RstbEntryView,
    shader_module::{ShaderModule, TextureSamplerPair},
    types::*,
};

const ALL: u8 = BindingMap::kStageVertex | BindingMap::kStageFragment | BindingMap::kStageCompute;
fn row(group: u8, binding: u8, kind: ResourceKind, stage_mask: u8) -> Entry {
    let mut e = Entry::default();
    e.group = group;
    e.binding = binding;
    e.kind = kind;
    e.stageMask = stage_mask;
    e.backendSlot[0] = binding.into();
    e.backendSlot[1] = binding.into();
    if kind == ResourceKind::SampledTexture {
        e.textureViewDim = TextureViewDim::D2;
        e.textureSampleType = TextureSampleType::Float;
    }
    if kind == ResourceKind::UniformBuffer {
        e.minBindingSize = 16;
    }
    e
}
fn uses(stage: u8, name: &str, bindings: &[u16]) -> RstbEntryBindings {
    RstbEntryBindings {
        stage,
        name: name.into(),
        bindings: bindings.to_vec(),
    }
}
fn shader(with_use: bool) -> ShaderModule {
    let mut shader = nuxie_ore_metal::new_shader_module_backend_base();
    shader
        .m_bindingMap
        .push(row(0, 0, ResourceKind::UniformBuffer, ALL));
    shader
        .m_bindingMap
        .push(row(0, 1, ResourceKind::SampledTexture, ALL));
    shader.m_bindingMap.finalize();
    shader.m_bindingMap.computeLayoutIds();
    if with_use {
        shader.m_entryBindings = vec![
            uses(0, "vs_tri", &[]),
            uses(1, "fs_flat", &[]),
            uses(1, "fs_picture", &[1]),
            uses(1, "fs_tint", &[0]),
            uses(1, "fs_both", &[0, 1]),
        ];
    }
    shader
}
fn map(shader: &ShaderModule, fragment: &str) -> BindingMap {
    bindingMapForEntries(Some(shader), Some("vs_tri"), Some(shader), Some(fragment))
}
fn layout(
    ctx: &mut DeferredOreContext,
    map: &BindingMap,
    pairs: Option<&ShaderModule>,
) -> AnyResourceHandle {
    makeBindGroupLayoutFromBindingMap(ctx, map, 0, &[], pairs, None).unwrap()
}
fn texture_view(ctx: &mut DeferredOreContext) -> AnyResourceHandle {
    let texture = ctx
        .makeTexture(&TextureDesc {
            width: 4,
            height: 4,
            format: TextureFormat::rgba8unorm,
            ..Default::default()
        })
        .unwrap();
    ctx.makeTextureView(&TextureViewDesc {
        texture: Some(&texture),
        ..Default::default()
    })
    .unwrap()
}
#[test]
fn entry_bindings_encode_and_decode() {
    let sent = [uses(0, "vs_main", &[]), uses(1, "fs_main", &[1, 0x102])];
    let bytes = encodeEntryBindings(&sent);
    let mut got = Vec::new();
    assert!(decodeEntryBindings(Some(&bytes), bytes.len(), &mut got));
    assert_eq!(got.len(), 2);
    assert_eq!(got[0].name.as_bytes(), b"vs_main");
    assert!(got[0].bindings.is_empty());
    assert_eq!(got[1].stage, 1);
    assert!(got[1].uses(1, 2));
    assert!(!got[1].uses(0, 0));
    assert!(!decodeEntryBindings(
        Some(&bytes),
        bytes.len() - 1,
        &mut got
    ));
    assert!(got.is_empty());
    assert!(!decodeEntryBindings(None, 0, &mut got));
}
#[test]
fn narrowing_keeps_rows_the_entry_points_use() {
    let shader = shader(true);
    let narrowed = map(&shader, "fs_picture");
    assert_eq!(narrowed.size(), 1);
    assert_eq!(narrowed.at(0).binding, 1);
    assert_eq!(narrowed.at(0).stageMask, ALL);
    assert_ne!(narrowed.layoutIdForGroup(0), BindingMap::kNoLayoutId);
    assert_ne!(
        narrowed.layoutIdForGroup(0),
        shader.m_bindingMap.layoutIdForGroup(0)
    );
    let other = self::shader(true);
    assert_eq!(
        map(&other, "fs_picture").layoutIdForGroup(0),
        narrowed.layoutIdForGroup(0)
    );
    let flat = map(&shader, "fs_flat");
    assert!(flat.empty());
    assert_eq!(flat.groupLayoutCount(), 0);
}
#[test]
fn keeping_every_row_keeps_module_layout() {
    let shader = shader(true);
    let both = map(&shader, "fs_both");
    assert_eq!(both.size(), 2);
    assert_eq!(
        both.layoutIdForGroup(0),
        shader.m_bindingMap.layoutIdForGroup(0)
    );
    let mut ctx = DeferredOreContext::fromReal(None);
    assert_eq!(
        layout(&mut ctx, &both, Some(&shader)).allocation_identity(),
        makeBindGroupLayoutFromShader(&mut ctx, Some(&shader), 0, &[])
            .unwrap()
            .allocation_identity()
    );
}
#[test]
fn narrowing_follows_each_stages_own_module() {
    let mut vertex = nuxie_ore_metal::new_shader_module_backend_base();
    vertex.m_bindingMap.push(row(
        0,
        0,
        ResourceKind::UniformBuffer,
        BindingMap::kStageVertex,
    ));
    vertex.m_bindingMap.finalize();
    vertex.m_entryBindings = vec![uses(0, "vs_main", &[0])];
    let mut fragment = nuxie_ore_metal::new_shader_module_backend_base();
    fragment.m_bindingMap.push(row(
        0,
        1,
        ResourceKind::SampledTexture,
        BindingMap::kStageFragment,
    ));
    fragment.m_bindingMap.finalize();
    fragment.m_entryBindings = vec![uses(1, "fs_flat", &[]), uses(1, "fs_main", &[1])];
    assert_eq!(
        bindingMapForEntries(
            Some(&vertex),
            Some("vs_main"),
            Some(&fragment),
            Some("fs_main")
        )
        .size(),
        2
    );
    let flat = bindingMapForEntries(
        Some(&vertex),
        Some("vs_main"),
        Some(&fragment),
        Some("fs_flat"),
    );
    assert_eq!(flat.size(), 1);
    assert_eq!(flat.at(0).binding, 0);
    let mut unused = nuxie_ore_metal::new_shader_module_backend_base();
    unused.m_bindingMap.push(row(
        0,
        1,
        ResourceKind::UniformBuffer,
        BindingMap::kStageVertex,
    ));
    unused.m_bindingMap.finalize();
    unused.m_entryBindings = vec![uses(0, "vs_main", &[])];
    let texture = bindingMapForEntries(
        Some(&unused),
        Some("vs_main"),
        Some(&fragment),
        Some("fs_main"),
    );
    assert_eq!(texture.size(), 1);
    assert_eq!(texture.at(0).kind, ResourceKind::SampledTexture);
    let mut ctx = DeferredOreContext::fromReal(None);
    let layout = layout(&mut ctx, &texture, None);
    let layouts = [Some(&layout)];
    let unused = ResourceHandle::new(None, unused).erase();
    let fragment = ResourceHandle::new(None, fragment).erase();
    let desc = PipelineDesc {
        vertexModule: Some(&unused),
        vertexEntryPoint: Some("vs_main"),
        fragmentModule: Some(&fragment),
        fragmentEntryPoint: Some("fs_main"),
        bindGroupLayouts: Some(&layouts),
        bindGroupLayoutCount: 1,
        ..Default::default()
    };
    let mut error = String::new();
    assert!(validatePipelineDesc(
        &desc,
        &texture,
        NativeSlotScope::perStage,
        Some(&mut error)
    ));
    assert!(error.is_empty());
}
#[test]
fn sampler_narrowed_to_depth_texture_is_non_filtering() {
    let mut shader = nuxie_ore_metal::new_shader_module_backend_base();
    let mut depth = row(0, 0, ResourceKind::SampledTexture, ALL);
    depth.textureSampleType = TextureSampleType::Depth;
    shader.m_bindingMap.push(depth);
    shader
        .m_bindingMap
        .push(row(0, 1, ResourceKind::SampledTexture, ALL));
    shader
        .m_bindingMap
        .push(row(0, 2, ResourceKind::Sampler, ALL));
    shader.m_bindingMap.finalize();
    shader.m_bindingMap.computeLayoutIds();
    shader.m_textureSamplerPairs = vec![
        TextureSamplerPair {
            texGroup: 0,
            texBinding: 0,
            sampGroup: 0,
            sampBinding: 2,
        },
        TextureSamplerPair {
            texGroup: 0,
            texBinding: 1,
            sampGroup: 0,
            sampBinding: 2,
        },
    ];
    shader.m_entryBindings = vec![
        uses(0, "vs_main", &[]),
        uses(1, "fs_depth", &[0, 2]),
        uses(1, "fs_color", &[1, 2]),
    ];
    let mut ctx = DeferredOreContext::fromReal(None);
    let mut sampler_of = |map: &BindingMap| {
        layout(&mut ctx, map, Some(&shader))
            .bindGroupLayoutBase()
            .unwrap()
            .findEntry(2)
            .unwrap()
            .samplerNonFiltering
    };
    assert!(sampler_of(&bindingMapForEntries(
        Some(&shader),
        Some("vs_main"),
        Some(&shader),
        Some("fs_depth")
    )));
    assert!(!sampler_of(&bindingMapForEntries(
        Some(&shader),
        Some("vs_main"),
        Some(&shader),
        Some("fs_color")
    )));
    assert!(!sampler_of(&shader.m_bindingMap));
}
#[test]
fn older_bakes_and_unknown_entries_stay_whole() {
    assert_eq!(map(&shader(false), "fs_flat").size(), 2);
    let shader = shader(true);
    let unknown = map(&shader, "fs_missing");
    assert_eq!(unknown.size(), 2);
    assert_eq!(
        unknown.layoutIdForGroup(0),
        shader.m_bindingMap.layoutIdForGroup(0)
    );
    let mut single = self::shader(true);
    single.m_entryBindings = vec![uses(1, "fs_picture", &[1])];
    assert!(std::ptr::eq(
        single.entryBindings(Some("main")).unwrap(),
        &single.m_entryBindings[0]
    ));
}
#[test]
fn derived_layout_binds_only_entry_point_resources() {
    let shader = shader(true);
    let mut ctx = DeferredOreContext::fromReal(None);
    let picture = layout(&mut ctx, &map(&shader, "fs_picture"), None);
    assert!(
        picture
            .bindGroupLayoutBase()
            .unwrap()
            .findEntry(0)
            .is_none()
    );
    assert!(
        picture
            .bindGroupLayoutBase()
            .unwrap()
            .findEntry(1)
            .is_some()
    );
    let view = texture_view(&mut ctx);
    let textures = [TexEntry {
        slot: 1,
        view: Some(&view),
    }];
    let mut desc = BindGroupDesc {
        layout: Some(&picture),
        textures: &textures,
        textureCount: 1,
        ..Default::default()
    };
    assert!(ctx.makeBindGroup(&desc).is_some());
    let whole = makeBindGroupLayoutFromShader(&mut ctx, Some(&shader), 0, &[]).unwrap();
    desc.layout = Some(&whole);
    ctx.clearLastError();
    assert!(ctx.makeBindGroup(&desc).is_none());
    assert!(ctx.lastError().contains("leaves this uniform buffer empty"));
    assert_eq!(
        layout(&mut ctx, &map(&shader, "fs_picture"), None).allocation_identity(),
        picture.allocation_identity()
    );
}
#[test]
fn pipeline_needs_layouts_only_for_its_entries() {
    let shader = ResourceHandle::new(None, shader(true)).erase();
    let module = shader.shaderModuleBase().unwrap();
    let mut ctx = DeferredOreContext::fromReal(None);
    let picture = layout(&mut ctx, &map(module, "fs_picture"), None);
    let whole = makeBindGroupLayoutFromShader(&mut ctx, Some(module), 0, &[]).unwrap();
    let validate = |fragment, layout: Option<&AnyResourceHandle>| {
        let layouts = [layout];
        let desc = PipelineDesc {
            vertexModule: Some(&shader),
            vertexEntryPoint: Some("vs_tri"),
            fragmentModule: Some(&shader),
            fragmentEntryPoint: Some(fragment),
            bindGroupLayouts: layout.map(|_| layouts.as_slice()),
            bindGroupLayoutCount: u32::from(layout.is_some()),
            ..Default::default()
        };
        let mut error = String::new();
        validatePipelineDesc(
            &desc,
            &map(module, fragment),
            NativeSlotScope::perStage,
            Some(&mut error),
        );
        error
    };
    assert!(validate("fs_flat", None).is_empty());
    assert!(validate("fs_picture", Some(&picture)).is_empty());
    assert!(validate("fs_picture", Some(&whole)).is_empty());
    assert!(validate("fs_tint", Some(&picture)).contains("layout has no entry for this binding"));
    assert!(!validate("fs_picture", None).is_empty());
}
#[test]
fn each_module_carries_physical_entry_names() {
    let asset = [uses(0, "vertex", &[0]), uses(1, "fs_main", &[1])];
    let mut entries = [
        RstbEntryView {
            stage: 0,
            logical: "vertex".into(),
            physical: "vertex0".into(),
            ..Default::default()
        },
        RstbEntryView {
            stage: 1,
            logical: "fs_main".into(),
            physical: "fs_main".into(),
            ..Default::default()
        },
    ];
    let mut got = Vec::new();
    let bytes = moduleEntryBindingBytes(&asset, &entries, 0, 2);
    assert!(decodeEntryBindings(Some(&bytes), bytes.len(), &mut got));
    assert_eq!(got.len(), 2);
    assert_eq!(got[0].name.as_bytes(), b"vertex0");
    assert!(got[0].uses(0, 0));
    let bytes = moduleEntryBindingBytes(&asset, &entries, 1, 1);
    assert!(decodeEntryBindings(Some(&bytes), bytes.len(), &mut got));
    assert_eq!(got.len(), 1);
    assert_eq!(got[0].name.as_bytes(), b"fs_main");
    entries[1].logical = "fs_other".into();
    assert!(moduleEntryBindingBytes(&asset, &entries, 0, 2).is_empty());
}
#[test]
fn draw_needs_groups_matching_pipeline_layouts() {
    let shader = ResourceHandle::new(None, shader(true)).erase();
    let module = shader.shaderModuleBase().unwrap();
    let mut ctx = DeferredOreContext::fromReal(None);
    let picture = layout(&mut ctx, &map(module, "fs_picture"), None);
    let layouts = [Some(&picture)];
    let desc = PipelineDesc {
        vertexModule: Some(&shader),
        vertexEntryPoint: Some("vs_tri"),
        fragmentModule: Some(&shader),
        fragmentEntryPoint: Some("fs_picture"),
        bindGroupLayouts: Some(&layouts),
        bindGroupLayoutCount: 1,
        ..Default::default()
    };
    let pipeline = ResourceHandle::new(
        None,
        nuxie_ore_metal::new_pipeline_backend_base_without_manager(&desc).unwrap(),
    )
    .erase();
    let view = texture_view(&mut ctx);
    let buffer = ctx
        .makeBuffer(&BufferDesc {
            usage: BufferUsage::uniform,
            size: 16,
            ..Default::default()
        })
        .unwrap();
    let mut group_of = |layout: &AnyResourceHandle| {
        let textures = [TexEntry {
            slot: 1,
            view: Some(&view),
        }];
        let ubos = [UBOEntry {
            slot: 0,
            buffer: Some(&buffer),
            offset: 0,
            size: 16,
        }];
        let has_ubo = layout.bindGroupLayoutBase().unwrap().findEntry(0).is_some();
        ctx.makeBindGroup(&BindGroupDesc {
            layout: Some(layout),
            textures: &textures,
            textureCount: 1,
            ubos: if has_ubo { &ubos } else { &[] },
            uboCount: u32::from(has_ubo),
            ..Default::default()
        })
        .unwrap()
    };
    let picture_group = group_of(&picture);
    let mut bindings = DrawBindings::default();
    bindings.setPipeline(Some(&pipeline));
    let mut error = String::new();
    assert!(!bindings.validate(Some(&mut error)));
    assert!(error.contains("@group(0): the pipeline binds this group"));
    let whole = makeBindGroupLayoutFromShader(&mut ctx, Some(module), 0, &[]).unwrap();
    let textures = [TexEntry {
        slot: 1,
        view: Some(&view),
    }];
    let ubos = [UBOEntry {
        slot: 0,
        buffer: Some(&buffer),
        offset: 0,
        size: 16,
    }];
    let whole_group = ctx
        .makeBindGroup(&BindGroupDesc {
            layout: Some(&whole),
            textures: &textures,
            textureCount: 1,
            ubos: &ubos,
            uboCount: 1,
            ..Default::default()
        })
        .unwrap();
    bindings.setBindGroup(0, Some(&whole_group));
    assert!(!bindings.validate(Some(&mut error)));
    assert!(error.contains("does not match the pipeline's"));
    bindings.setBindGroup(0, Some(&picture_group));
    assert!(bindings.validate(Some(&mut error)));
    let entries = picture.bindGroupLayoutBase().unwrap().entries();
    let twin = ctx
        .makeBindGroupLayout(&BindGroupLayoutDesc {
            groupIndex: 0,
            entries: Some(entries),
            entryCount: entries.len() as u32,
            ..Default::default()
        })
        .unwrap();
    assert_ne!(twin.allocation_identity(), picture.allocation_identity());
    let twin_group = ctx
        .makeBindGroup(&BindGroupDesc {
            layout: Some(&twin),
            textures: &textures,
            textureCount: 1,
            ..Default::default()
        })
        .unwrap();
    bindings.setBindGroup(0, Some(&twin_group));
    assert!(bindings.validate(Some(&mut error)));
    let flat = PipelineDesc {
        bindGroupLayouts: None,
        bindGroupLayoutCount: 0,
        ..desc
    };
    let flat = ResourceHandle::new(
        None,
        nuxie_ore_metal::new_pipeline_backend_base_without_manager(&flat).unwrap(),
    )
    .erase();
    let mut none = DrawBindings::default();
    none.setPipeline(Some(&flat));
    assert!(none.validate(Some(&mut error)));
    let old = ResourceHandle::new(None, self::shader(false)).erase();
    let old_whole =
        makeBindGroupLayoutFromShader(&mut ctx, old.shaderModuleBase(), 0, &[]).unwrap();
    let old_layouts = [Some(&old_whole)];
    let old_desc = PipelineDesc {
        vertexModule: Some(&old),
        fragmentModule: Some(&old),
        fragmentEntryPoint: Some("fs_flat"),
        bindGroupLayouts: Some(&old_layouts),
        ..desc
    };
    let old_pipeline = ResourceHandle::new(
        None,
        nuxie_ore_metal::new_pipeline_backend_base_without_manager(&old_desc).unwrap(),
    )
    .erase();
    let mut older = DrawBindings::default();
    older.setPipeline(Some(&old_pipeline));
    assert!(older.validate(Some(&mut error)));
    older.setBindGroup(0, Some(&picture_group));
    assert!(!older.validate(Some(&mut error)));
}
#[test]
fn split_files_check_slot_clashes_over_actual_bindings() {
    let file = |mask, second, used: &[u16]| {
        let mut module = nuxie_ore_metal::new_shader_module_backend_base();
        let stage = if mask == BindingMap::kStageVertex {
            0
        } else {
            1
        };
        for binding in [0, second] {
            let mut e = row(0, binding, ResourceKind::UniformBuffer, mask);
            e.backendSlot[0] = BindingMap::kAbsent;
            e.backendSlot[1] = BindingMap::kAbsent;
            e.backendSlot[stage] = if binding == 0 { 0 } else { 1 };
            module.m_bindingMap.push(e);
        }
        module.m_bindingMap.finalize();
        module.m_entryBindings = vec![uses(stage as u8, "main", used)];
        ResourceHandle::new(None, module).erase()
    };
    let vertex = file(BindingMap::kStageVertex, 1, &[0]);
    let fragment = file(BindingMap::kStageFragment, 2, &[0, 2]);
    let v = vertex.shaderModuleBase();
    let f = fragment.shaderModuleBase();
    let narrowed = bindingMapForEntries(v, Some("main"), f, Some("main"));
    let mut ctx = DeferredOreContext::fromReal(None);
    let mut validate = |map: &BindingMap| {
        let layout = layout(&mut ctx, map, None);
        let layouts = [Some(&layout)];
        let desc = PipelineDesc {
            vertexModule: Some(&vertex),
            vertexEntryPoint: Some("main"),
            fragmentModule: Some(&fragment),
            fragmentEntryPoint: Some("main"),
            bindGroupLayouts: Some(&layouts),
            bindGroupLayoutCount: 1,
            ..Default::default()
        };
        let mut error = String::new();
        validatePipelineDesc(&desc, &narrowed, NativeSlotScope::perKind, Some(&mut error));
        error
    };
    assert!(validate(&narrowed).is_empty());
    assert!(validate(&bindingMapForStages(v, f)).contains("both land on native slot 1"));
}
