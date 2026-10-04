/*
 * Copyright 2026 Rive
 */

// #include "rive/renderer/ore/ore_bind_group_layout.hpp"
// #include "rive/renderer/ore/ore_binding_map.hpp"

// #include <sstream>

// Mechanical translation of the complete pinned source implementation
// renderer/src/ore/ore_bind_group_layout.cpp.
// Upstream source revision: 4ac7b32798da0482e441ef09304dc3b480ed3ee5

#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(non_upper_case_globals)]
use super::*;
use crate::mechanical_port::source::renderer::include::rive::renderer::gpu_resource_hpp::AnyResourceHandle;
use crate::{context::ContextApi, shader_module::ShaderModule};

impl BindGroup {
    /// Protected upstream operation exposed to sibling backend crates.
    /// The descriptor's slice counts must first pass `validateBindGroupDesc`.
    pub fn recordDynamicRanges(&mut self, desc: &BindGroupDesc<'_>) {
        self.m_dynamicRanges.clear();
        let Some(layout) = desc.layout.and_then(AnyResourceHandle::bindGroupLayoutBase) else {
            return;
        };
        for ubo in &desc.ubos[..desc.uboCount as usize] {
            if !layout.hasDynamicOffset(ubo.slot) {
                continue;
            }
            if let Some(bufferSize) = ubo.buffer.and_then(AnyResourceHandle::size).map(u64::from) {
                let size = if ubo.size != 0 {
                    u64::from(ubo.size)
                } else {
                    bufferSize.saturating_sub(u64::from(ubo.offset))
                };
                self.m_dynamicRanges.push(DynamicUBORange {
                    slot: ubo.slot,
                    offset: ubo.offset,
                    size: size as u32,
                    bufferSize,
                });
            }
        }
        self.m_dynamicRanges
            .sort_unstable_by_key(|range| range.slot);
    }
}

pub fn validateSetBindGroup(
    groupIndex: u32,
    group: Option<&BindGroup>,
    offsets: Option<&[u32]>,
    offsetCount: u32,
    alignment: u32,
    mut outError: Option<&mut String>,
) -> bool {
    let mut fail = |message: String| {
        if let Some(error) = outError.as_mut() {
            **error = message;
        }
        false
    };
    if groupIndex >= kMaxBindGroups {
        return fail(format!(
            "setBindGroup: groupIndex must be in [0, {kMaxBindGroups}) (got {groupIndex})"
        ));
    }
    if offsetCount > kMaxDynamicOffsets {
        return fail(dynamic_offset_count_message(
            offsetCount,
            kMaxDynamicOffsets,
        ));
    }
    let Some(group) = group else {
        return true;
    };
    if group.layout().is_some() && group.groupIndex() != groupIndex {
        return fail(format!(
            "setBindGroup: BindGroup was made for group {}, not {groupIndex}",
            group.groupIndex()
        ));
    }
    if offsetCount != group.dynamicOffsetCount() {
        return fail(format!(
            "setBindGroup: dynamicOffsets count {offsetCount} does not match the BindGroup's declared dynamic UBO count {}",
            group.dynamicOffsetCount()
        ));
    }
    let ranges = group.dynamicRanges();
    let paired = ranges.len() == offsetCount as usize;
    for i in 0..offsetCount as usize {
        let offset = offsets.expect("dynamic offsets")[i];
        if alignment != 0 && offset % alignment != 0 {
            return fail(format!(
                "setBindGroup: dynamicOffsets[{i}] = {offset} is not a multiple of {alignment} (alignment requirement)"
            ));
        }
        if paired {
            let range = &ranges[i];
            if u64::from(offset) + u64::from(range.offset) + u64::from(range.size)
                > range.bufferSize
            {
                return fail(format!(
                    "setBindGroup: dynamicOffsets[{i}] = {offset} moves @binding({}) past the end of its {} byte buffer",
                    range.slot, range.bufferSize
                ));
            }
        }
    }
    true
}

pub fn scriptDynamicOffsetAlignment(context: Option<&dyn ContextApi>) -> u32 {
    context.filter(|context| context.featuresKnown()).map_or(
        Features::default().minUniformBufferOffsetAlignment,
        |context| context.features().minUniformBufferOffsetAlignment,
    )
}

// namespace rive::ore

#[cfg(all(test, feature = "with-rive-tools"))]
mod shader_layout_tests {
    use super::*;
    use crate::buffer::{Buffer, BufferApi, BufferUpdateError};
    use crate::gpu_resource::{GPUResource, GpuResourcePayload, ResourceHandle};

    struct TestBuffer(Buffer);
    unsafe impl GpuResourcePayload for TestBuffer {
        fn gpu_resource(&self) -> &GPUResource {
            self.0.gpu_resource()
        }
        fn gpu_resource_mut(&mut self) -> &mut GPUResource {
            self.0.gpu_resource_mut()
        }
    }
    impl BufferApi for TestBuffer {
        fn size(&self) -> u32 {
            self.0.size()
        }
        fn usage(&self) -> BufferUsage {
            self.0.usage()
        }
        fn update(&self, _: &[u8], _: u32, _: u32) -> Result<(), BufferUpdateError> {
            Ok(())
        }
    }

    #[test]
    fn validate_bind_group_desc_rejects_ubo_shorter_than_block() {
        let ubo_entry = |binding, minimum| BindGroupLayoutEntry {
            binding,
            kind: BindingKind::uniformBuffer,
            minBindingSize: minimum,
            ..Default::default()
        };
        let layout = |entries| {
            let mut layout = BindGroupLayout::new();
            layout.m_groupIndex = 0;
            layout.m_entries = entries;
            ResourceHandle::new(None, layout).erase()
        };
        let sized = layout(vec![ubo_entry(0, 208), ubo_entry(1, 192)]);
        let buffer = |size| {
            ResourceHandle::new_buffer(None, TestBuffer(Buffer::new(size, BufferUsage::uniform)))
                .erase()
        };
        let camera = buffer(208);
        let model = buffer(160);
        let models = buffer(1024);
        let mut ubos = [
            UBOEntry {
                slot: 0,
                buffer: Some(&camera),
                ..Default::default()
            },
            UBOEntry {
                slot: 1,
                buffer: Some(&model),
                ..Default::default()
            },
        ];
        let check = |layout, ubos: &[UBOEntry<'_>], error: &mut String| {
            validateBindGroupDesc(
                &BindGroupDesc {
                    layout: Some(layout),
                    ubos,
                    uboCount: ubos.len() as u32,
                    textures: &[],
                    textureCount: 0,
                    samplers: &[],
                    samplerCount: 0,
                    label: None,
                },
                Some(error),
            )
        };
        let mut error = String::new();
        assert!(!check(&sized, &ubos, &mut error));
        assert!(error.contains("@binding(1)"));
        assert!(error.contains("160"));
        assert!(error.contains("192"));
        ubos[1].buffer = Some(&models);
        assert!(check(&sized, &ubos, &mut error));
        ubos[1].offset = 256;
        ubos[1].size = 192;
        assert!(check(&sized, &ubos, &mut error));
        ubos[1].size = 160;
        assert!(!check(&sized, &ubos, &mut error));
        assert!(error.contains("160"));
        ubos[1].offset = 1000;
        ubos[1].size = 192;
        assert!(!check(&sized, &ubos, &mut error));
        assert!(error.contains("exceeds"));
        let unsized_layout = layout(vec![ubo_entry(1, 0)]);
        ubos[1].offset = 0;
        ubos[1].size = 0;
        ubos[1].buffer = Some(&model);
        assert!(check(&unsized_layout, &ubos[1..], &mut error));
    }

    #[test]
    fn shared_population_preserves_fields_and_reflects_binding_metadata() {
        let mut shader = ShaderModule::new();
        shader.m_bindingMap.push(&BindingMapEntry {
            group: 1,
            binding: 7,
            kind: ResourceKind::UniformBuffer,
            stageMask: 7,
            backendSlot: [4, BindingMap::kAbsent, 9],
            minBindingSize: 192,
            ..Default::default()
        });
        shader.m_bindingMap.push(&BindingMapEntry {
            group: 0,
            binding: 9,
            ..Default::default()
        });
        shader.m_bindingMap.push(&BindingMapEntry {
            group: 1,
            binding: 8,
            kind: ResourceKind::SampledTexture,
            textureViewDim: TextureViewDim::CubeArray,
            textureSampleType: TextureSampleType::Uint,
            textureMultisampled: true,
            ..Default::default()
        });
        let mut entries = [BindGroupLayoutEntry {
            minBindingSize: 123,
            nativeSlotCS: 456,
            ..Default::default()
        }; 2];
        assert_eq!(
            populateBindGroupLayoutEntriesFromShader(&mut entries, Some(&shader), 1, &[7, 8]),
            2
        );
        assert_eq!(entries[0].binding, 7);
        assert_eq!(entries[0].visibility.mask, 7);
        assert!(entries[0].hasDynamicOffset);
        assert_eq!(entries[0].nativeSlotVS, 4);
        assert_eq!(
            entries[0].nativeSlotFS,
            BindGroupLayoutEntry::kNativeSlotAbsent
        );
        assert_eq!(entries[0].minBindingSize, 192);
        assert_eq!(entries[0].nativeSlotCS, 456);
        assert_eq!(entries[1].binding, 8);
        assert!(!entries[1].hasDynamicOffset);
        assert!(entries[1].textureViewDim == TextureViewDimension::cubeArray);
        assert!(entries[1].textureSampleType == SampleType::uint);
        assert!(entries[1].textureMultisampled);
        assert_eq!(
            populateBindGroupLayoutEntriesFromShader(&mut entries[..1], Some(&shader), 1, &[]),
            2
        );
        assert!(!entries[0].hasDynamicOffset);
        assert_eq!(
            populateBindGroupLayoutEntriesFromShader(&mut entries, None, 1, &[]),
            0
        );
        assert_eq!(entries[0].binding, 7);
        assert_eq!(
            populateBindGroupLayoutEntriesFromShader(&mut [], Some(&shader), 1, &[]),
            2
        );
    }
}

impl BindGroupLayout {
    // const BindGroupLayoutEntry* BindGroupLayout::findEntry(uint32_t binding) const
    pub fn findEntry(&self, binding: u32) -> Option<&BindGroupLayoutEntry> {
        for e in &self.m_entries {
            if e.binding == binding {
                return Some(e);
            }
        }
        None
    }

    // bool BindGroupLayout::hasDynamicOffset(uint32_t binding) const
    pub fn hasDynamicOffset(&self, binding: u32) -> bool {
        let e = self.findEntry(binding);
        if let Some(e) = e {
            e.kind == BindingKind::uniformBuffer && e.hasDynamicOffset
        } else {
            false
        }
    }
}

// Map binding-map types to layout-entry types.
fn sameBakedLayout(a: &BindingMap, b: &BindingMap) -> bool {
    a.groupLayoutCount() != 0
        && a.groupLayoutCount() == b.groupLayoutCount()
        && (0..a.groupLayoutCount()).all(|i| {
            let ga = a.groupLayoutAt(i);
            let gb = b.groupLayoutAt(i);
            ga.group == gb.group
                && ga.layoutId == gb.layoutId
                && ga.layoutId != BindingMap::kNoLayoutId
        })
}

pub fn bindingMapForStages(
    vertex: Option<&ShaderModule>,
    fragment: Option<&ShaderModule>,
) -> BindingMap {
    let Some(vertex) = vertex else {
        return fragment
            .map(|module| module.m_bindingMap.clone())
            .unwrap_or_default();
    };
    let Some(fragment) = fragment else {
        return vertex.m_bindingMap.clone();
    };
    if std::ptr::eq(vertex, fragment)
        || sameBakedLayout(&vertex.m_bindingMap, &fragment.m_bindingMap)
    {
        return vertex.m_bindingMap.clone();
    }
    let mut merged = vertex.m_bindingMap.clone();
    merged.replaceStage(&fragment.m_bindingMap, Stage::FS);
    merged
}

pub fn populateBindGroupLayoutEntriesFromShader(
    entries: &mut [BindGroupLayoutEntry],
    shader: Option<&ShaderModule>,
    groupIndex: u32,
    dynamicUBOBindings: &[u32],
) -> u32 {
    shader.map_or(0, |shader| {
        populateBindGroupLayoutEntries(
            entries,
            &shader.m_bindingMap,
            groupIndex,
            dynamicUBOBindings,
        )
    })
}

pub fn makeBindGroupLayoutFromShader(
    ctx: &mut dyn ContextApi,
    shader: Option<&ShaderModule>,
    groupIndex: u32,
    dynamicUBOBindings: &[u32],
) -> Option<AnyResourceHandle> {
    let empty = BindingMap::default();
    makeBindGroupLayoutFromBindingMap(
        ctx,
        shader.map(|shader| &shader.m_bindingMap).unwrap_or(&empty),
        groupIndex,
        dynamicUBOBindings,
        shader,
        None,
    )
}

fn bindingKindFromResource(kind: ResourceKind) -> BindingKind {
    match kind {
        ResourceKind::UniformBuffer => BindingKind::uniformBuffer,
        ResourceKind::StorageBufferRO => BindingKind::storageBufferRO,
        ResourceKind::StorageBufferRW => BindingKind::storageBufferRW,
        ResourceKind::SampledTexture => BindingKind::sampledTexture,
        ResourceKind::StorageTexture => BindingKind::storageTexture,
        ResourceKind::Sampler => BindingKind::sampler,
        ResourceKind::ComparisonSampler => BindingKind::comparisonSampler,
        _ => BindingKind::uniformBuffer,
    }
}

fn viewDimFromBindingMap(dimension: TextureViewDim) -> TextureViewDimension {
    match dimension {
        TextureViewDim::Cube => TextureViewDimension::cube,
        TextureViewDim::CubeArray => TextureViewDimension::cubeArray,
        TextureViewDim::D3 => TextureViewDimension::texture3D,
        TextureViewDim::D2Array => TextureViewDimension::array2D,
        TextureViewDim::D1 | TextureViewDim::D2 | TextureViewDim::Undefined => {
            TextureViewDimension::texture2D
        }
        _ => TextureViewDimension::texture2D,
    }
}

fn sampleTypeFromBindingMap(sample: TextureSampleType) -> SampleType {
    match sample {
        TextureSampleType::UnfilterableFloat => SampleType::floatUnfilterable,
        TextureSampleType::Depth => SampleType::depth,
        TextureSampleType::Sint => SampleType::sint,
        TextureSampleType::Uint => SampleType::uint,
        TextureSampleType::Float | TextureSampleType::Undefined => SampleType::floatFilterable,
        _ => SampleType::floatFilterable,
    }
}

/// Fill entries from the shader's binding map, preserving caller-owned fields
/// not written upstream. The slice length represents `maxEntries`.
/// Returns the required count, which may exceed the provided slice length.
/// `dynamicUBOBindings` contains WGSL binding values within `groupIndex`.
pub fn populateBindGroupLayoutEntries(
    entries: &mut [BindGroupLayoutEntry],
    bindingMap: &BindingMap,
    groupIndex: u32,
    dynamicUBOBindings: &[u32],
) -> u32 {
    let mut n = 0;
    for i in 0..bindingMap.size() {
        let e = bindingMap.at(i);
        if u32::from(e.group) != groupIndex {
            continue;
        }
        let index = n;
        n += 1;
        let Some(out) = entries.get_mut(index) else {
            continue;
        };
        out.binding = u32::from(e.binding);
        out.kind = bindingKindFromResource(e.kind);
        let mut visibility = 0;
        for (source, destination) in [
            (BindingMap::kStageVertex, StageVisibility::kVertex),
            (BindingMap::kStageFragment, StageVisibility::kFragment),
            (BindingMap::kStageCompute, StageVisibility::kCompute),
        ] {
            if u32::from(e.stageMask) & source != 0 {
                visibility |= destination;
            }
        }
        out.visibility.mask = visibility;
        out.hasDynamicOffset =
            out.kind == BindingKind::uniformBuffer && dynamicUBOBindings.contains(&out.binding);
        out.textureViewDim = viewDimFromBindingMap(e.textureViewDim);
        out.textureSampleType = sampleTypeFromBindingMap(e.textureSampleType);
        out.textureMultisampled = e.textureMultisampled;
        out.minBindingSize = e.minBindingSize;
        let native_slot = |slot| {
            if slot == BindingMap::kAbsent {
                BindGroupLayoutEntry::kNativeSlotAbsent
            } else {
                u32::from(slot)
            }
        };
        out.nativeSlotVS = native_slot(e.backendSlot[0]);
        out.nativeSlotFS = native_slot(e.backendSlot[1]);
    }
    n as u32
}

fn samplerPairsDepthOnly(
    bm: &BindingMap,
    vertexPairSource: Option<&ShaderModule>,
    fragmentPairSource: Option<&ShaderModule>,
    groupIndex: u32,
    binding: u32,
) -> bool {
    let fragment = fragmentPairSource
        .filter(|fragment| !vertexPairSource.is_some_and(|vertex| std::ptr::eq(vertex, *fragment)));
    let mut paired = false;
    for module in [vertexPairSource, fragment].into_iter().flatten() {
        for p in &module.m_textureSamplerPairs {
            if u32::from(p.samplerGroup) != groupIndex || u32::from(p.samplerBinding) != binding {
                continue;
            }
            let tex = bm.lookupEntry(u32::from(p.textureGroup), u32::from(p.textureBinding));
            if !tex.is_some_and(|tex| tex.textureSampleType == TextureSampleType::Depth) {
                return false;
            }
            paired = true;
        }
    }
    paired
}

fn collectDepthOnlySamplers(
    bm: &BindingMap,
    vertexPairSource: Option<&ShaderModule>,
    fragmentPairSource: Option<&ShaderModule>,
    groupIndex: u32,
) -> Vec<u32> {
    let mut bindings = Vec::new();
    for i in 0..bm.size() {
        let e = bm.at(i);
        if u32::from(e.group) == groupIndex
            && e.kind == ResourceKind::Sampler
            && samplerPairsDepthOnly(
                bm,
                vertexPairSource,
                fragmentPairSource,
                groupIndex,
                u32::from(e.binding),
            )
        {
            bindings.push(u32::from(e.binding));
        }
    }
    bindings
}

const kNonFilteringLayoutIdSalt: u64 = 0x9e3779b97f4a7c15;
fn saltLayoutId(mut layoutId: u64, nonFiltering: &[u32]) -> u64 {
    for &binding in nonFiltering {
        layoutId = (layoutId ^ kNonFilteringLayoutIdSalt.wrapping_add(u64::from(binding)))
            .wrapping_mul(0x100000001b3);
    }
    layoutId
}

/// Intern by baked identity plus depth-only sampler flags. Pair sources must
/// include every module whose binding map was merged.
pub fn makeBindGroupLayoutFromBindingMap(
    ctx: &mut dyn ContextApi,
    bindingMap: &BindingMap,
    groupIndex: u32,
    dynamicUBOBindings: &[u32],
    vertexPairSource: Option<&ShaderModule>,
    fragmentPairSource: Option<&ShaderModule>,
) -> Option<AnyResourceHandle> {
    let nonFiltering = if vertexPairSource.is_some() || fragmentPairSource.is_some() {
        collectDepthOnlySamplers(bindingMap, vertexPairSource, fragmentPairSource, groupIndex)
    } else {
        Vec::new()
    };
    let mut layoutId = if dynamicUBOBindings.is_empty() {
        bindingMap.layoutIdForGroup(groupIndex)
    } else {
        BindingMap::kNoLayoutId
    };
    if layoutId != BindingMap::kNoLayoutId && !nonFiltering.is_empty() {
        layoutId = saltLayoutId(layoutId, &nonFiltering);
    }
    if layoutId != BindingMap::kNoLayoutId {
        if let Some(hit) = ctx.findInternedBindGroupLayout(layoutId) {
            return Some(hit);
        }
    }
    let mut entries = [BindGroupLayoutEntry::default(); 16];
    let n =
        populateBindGroupLayoutEntries(&mut entries, bindingMap, groupIndex, dynamicUBOBindings);
    let mut spilled = Vec::new();
    let entries = if n > entries.len() as u32 {
        spilled.resize(n as usize, BindGroupLayoutEntry::default());
        populateBindGroupLayoutEntries(&mut spilled, bindingMap, groupIndex, dynamicUBOBindings);
        spilled.as_mut_slice()
    } else {
        entries.as_mut_slice()
    };
    if !nonFiltering.is_empty() {
        for e in entries.iter_mut().take(n as usize) {
            e.samplerNonFiltering =
                e.kind == BindingKind::sampler && nonFiltering.contains(&e.binding);
        }
    }
    let layout = ctx.makeBindGroupLayout(&BindGroupLayoutDesc {
        groupIndex,
        entries: Some(entries),
        entryCount: n,
        ..BindGroupLayoutDesc::default()
    });
    if layoutId != BindingMap::kNoLayoutId {
        if let Some(layout) = &layout {
            ctx.internBindGroupLayout(layoutId, layout.clone());
        }
    }
    layout
}

// Map ore::BindingKind (public layout API) ↔ ore::ResourceKind (binding-map
// internal). Kept private to this TU.
#[allow(unreachable_patterns)] // Preserve the source defensive default for future enum values.
fn kindsMatch(layoutKind: BindingKind, shaderKind: ResourceKind) -> bool {
    match layoutKind {
        BindingKind::uniformBuffer => shaderKind == ResourceKind::UniformBuffer,
        BindingKind::storageBufferRO => shaderKind == ResourceKind::StorageBufferRO,
        BindingKind::storageBufferRW => shaderKind == ResourceKind::StorageBufferRW,
        BindingKind::sampledTexture => shaderKind == ResourceKind::SampledTexture,
        BindingKind::storageTexture => shaderKind == ResourceKind::StorageTexture,
        BindingKind::sampler => {
            // Sampler / ComparisonSampler are interchangeable on the
            // bind-API side — matches the BindingMap::lookup collapse
            // (ore_binding_map.hpp:201-208). Layout is allowed to declare
            // either; runtime treats them as one bind-time category.
            shaderKind == ResourceKind::Sampler || shaderKind == ResourceKind::ComparisonSampler
        }
        BindingKind::comparisonSampler => {
            shaderKind == ResourceKind::Sampler || shaderKind == ResourceKind::ComparisonSampler
        }
        _ => false,
    }
}

#[allow(unreachable_patterns)] // Preserve the source diagnostic fallback for future enum values.
fn kindName(k: BindingKind) -> &'static str {
    match k {
        BindingKind::uniformBuffer => "uniformBuffer",
        BindingKind::storageBufferRO => "storageBufferRO",
        BindingKind::storageBufferRW => "storageBufferRW",
        BindingKind::sampledTexture => "sampledTexture",
        BindingKind::storageTexture => "storageTexture",
        BindingKind::sampler => "sampler",
        BindingKind::comparisonSampler => "comparisonSampler",
        _ => "?",
    }
}

fn shaderKindName(k: ResourceKind) -> &'static str {
    match k {
        ResourceKind::UniformBuffer => "uniformBuffer",
        ResourceKind::StorageBufferRO => "storageBufferRO",
        ResourceKind::StorageBufferRW => "storageBufferRW",
        ResourceKind::SampledTexture => "sampledTexture",
        ResourceKind::StorageTexture => "storageTexture",
        ResourceKind::Sampler => "sampler",
        ResourceKind::ComparisonSampler => "comparisonSampler",
        ResourceKind(_) => "?",
    }
}

pub fn validateLayoutsAgainstBindingMap(
    bindingMap: &BindingMap,
    layouts: Option<&[Option<&AnyResourceHandle>]>,
    layoutCount: u32,
    mut outError: Option<&mut String>,
) -> bool {
    let mut fail = |msg: String| {
        if let Some(error) = outError.as_mut() {
            **error = msg;
        }
        false
    };

    // Every binding the shader references must have a corresponding layout
    // entry. Unused layout entries (declared but not referenced by the shader)
    // are allowed — Dawn permits this and it lets pipelines reuse a more
    // permissive layout than the shader strictly needs.
    for i in 0..bindingMap.size() {
        let shaderEntry: &Entry = bindingMap.at(i);
        let group: u32 = shaderEntry.group.into();
        let binding: u32 = shaderEntry.binding.into();

        if group >= layoutCount
            || layouts.is_none()
            || layouts
                .and_then(|allLayouts| allLayouts.get(group as usize))
                .and_then(|layout| *layout)
                .is_none()
        {
            let msg = format!(
                "@group({group}) @binding({binding}): shader declares {} but PipelineDesc::bindGroupLayouts has no entry for group {group}",
                shaderKindName(shaderEntry.kind)
            );
            return fail(msg);
        }

        let layoutHandle = layouts
            .and_then(|allLayouts| allLayouts.get(group as usize))
            .and_then(|layout| *layout)
            .expect("layout was checked above");
        let Some(layout) = layoutHandle.downcast_ref::<BindGroupLayout>() else {
            return fail(format!(
                "PipelineDesc::bindGroupLayouts[{group}] is not a BindGroupLayout"
            ));
        };
        if layout.groupIndex() != group {
            let msg = format!(
                "PipelineDesc::bindGroupLayouts[{group}]->groupIndex == {}, expected {group} (positional index must match layout's groupIndex)",
                layout.groupIndex()
            );
            return fail(msg);
        }

        let layoutEntry: &BindGroupLayoutEntry = match layout.findEntry(binding) {
            Some(entry) => entry,
            None => {
                let msg = format!(
                    "@group({group}) @binding({binding}): layout has no entry for this binding (shader expects {})",
                    shaderKindName(shaderEntry.kind)
                );
                return fail(msg);
            }
        };

        if !kindsMatch(layoutEntry.kind, shaderEntry.kind) {
            let msg = format!(
                "@group({group}) @binding({binding}): layout declares {} but shader declares {}",
                kindName(layoutEntry.kind),
                shaderKindName(shaderEntry.kind)
            );
            return fail(msg);
        }

        // Visibility narrower than the shader's stageMask is rejected.
        // Layout broader than shader is fine (allowed by WebGPU spec).
        let shaderStageMask: u8 = shaderEntry.stageMask;
        let layoutVisibility: u8 = layoutEntry.visibility.mask;
        if (shaderStageMask & !layoutVisibility) != 0 {
            let msg = format!(
                "@group({group}) @binding({binding}): layout visibility 0x{:x} missing stages required by shader (stageMask=0x{:x})",
                layoutVisibility, shaderStageMask
            );
            return fail(msg);
        }

        // Texture dimension/sampleType compatibility (texture kinds only).
        if layoutEntry.kind == BindingKind::sampledTexture
            || layoutEntry.kind == BindingKind::storageTexture
        {
            // Map TextureViewDim (binding-map) ↔ TextureViewDimension
            // (public layout API). The binding-map enum has D1/D2/D2Array/
            // Cube/CubeArray/D3; the public enum has texture2D/cube/
            // texture3D/array2D/cubeArray.
            #[allow(unreachable_patterns)] // Source retains a defensive fallback for enum growth.
            let dimsMatch = |a: TextureViewDimension, b: TextureViewDim| match a {
                TextureViewDimension::texture2D => b == TextureViewDim::D2,
                TextureViewDimension::cube => b == TextureViewDim::Cube,
                TextureViewDimension::texture3D => b == TextureViewDim::D3,
                TextureViewDimension::array2D => b == TextureViewDim::D2Array,
                TextureViewDimension::cubeArray => b == TextureViewDim::CubeArray,
                _ => false,
            };

            // Shader's textureViewDim is Undefined for non-texture kinds
            // and may also be Undefined for textures the shader compiler
            // didn't reflect a dim for. Skip the check when shader side
            // is Undefined.
            if shaderEntry.textureViewDim != TextureViewDim::Undefined
                && !dimsMatch(layoutEntry.textureViewDim, shaderEntry.textureViewDim)
            {
                let msg =
                    format!("@group({group}) @binding({binding}): texture view dimension mismatch");
                return fail(msg);
            }
        }
    }
    true
}

/// Rust derived-class integration spelling of
/// `validateLayoutsAgainstBindingMap`.
///
/// A C++ `BindGroupLayoutVulkan*` converts implicitly to its
/// `BindGroupLayout*` base. Rust resource handles retain the concrete payload,
/// so sibling backend crates perform the downcast and pass these exact base
/// references through this seam. The validation body and diagnostics remain
/// identical to the pinned source function above.
#[doc(hidden)]
pub fn validateLayoutBasesAgainstBindingMap(
    bindingMap: &BindingMap,
    layouts: Option<&[Option<&BindGroupLayout>]>,
    layoutCount: u32,
    mut outError: Option<&mut String>,
) -> bool {
    let mut fail = |msg: String| {
        if let Some(error) = outError.as_mut() {
            **error = msg;
        }
        false
    };

    for i in 0..bindingMap.size() {
        let shaderEntry: &Entry = bindingMap.at(i);
        let group: u32 = shaderEntry.group.into();
        let binding: u32 = shaderEntry.binding.into();

        if group >= layoutCount
            || layouts.is_none()
            || layouts
                .and_then(|allLayouts| allLayouts.get(group as usize))
                .and_then(|layout| *layout)
                .is_none()
        {
            return fail(format!(
                "@group({group}) @binding({binding}): shader declares {} but PipelineDesc::bindGroupLayouts has no entry for group {group}",
                shaderKindName(shaderEntry.kind)
            ));
        }

        let layout = layouts
            .and_then(|allLayouts| allLayouts.get(group as usize))
            .and_then(|layout| *layout)
            .expect("layout was checked above");
        if layout.groupIndex() != group {
            return fail(format!(
                "PipelineDesc::bindGroupLayouts[{group}]->groupIndex == {}, expected {group} (positional index must match layout's groupIndex)",
                layout.groupIndex()
            ));
        }

        let layoutEntry: &BindGroupLayoutEntry = match layout.findEntry(binding) {
            Some(entry) => entry,
            None => {
                return fail(format!(
                    "@group({group}) @binding({binding}): layout has no entry for this binding (shader expects {})",
                    shaderKindName(shaderEntry.kind)
                ));
            }
        };

        if !kindsMatch(layoutEntry.kind, shaderEntry.kind) {
            return fail(format!(
                "@group({group}) @binding({binding}): layout declares {} but shader declares {}",
                kindName(layoutEntry.kind),
                shaderKindName(shaderEntry.kind)
            ));
        }

        let shaderStageMask: u8 = shaderEntry.stageMask;
        let layoutVisibility: u8 = layoutEntry.visibility.mask;
        if (shaderStageMask & !layoutVisibility) != 0 {
            return fail(format!(
                "@group({group}) @binding({binding}): layout visibility 0x{:x} missing stages required by shader (stageMask=0x{:x})",
                layoutVisibility, shaderStageMask
            ));
        }

        if matches!(
            layoutEntry.kind,
            BindingKind::sampledTexture | BindingKind::storageTexture
        ) {
            let dimsMatch = |a: TextureViewDimension, b: TextureViewDim| match a {
                TextureViewDimension::texture2D => b == TextureViewDim::D2,
                TextureViewDimension::cube => b == TextureViewDim::Cube,
                TextureViewDimension::texture3D => b == TextureViewDim::D3,
                TextureViewDimension::array2D => b == TextureViewDim::D2Array,
                TextureViewDimension::cubeArray => b == TextureViewDim::CubeArray,
            };
            if shaderEntry.textureViewDim != TextureViewDim::Undefined
                && !dimsMatch(layoutEntry.textureViewDim, shaderEntry.textureViewDim)
            {
                return fail(format!(
                    "@group({group}) @binding({binding}): texture view dimension mismatch"
                ));
            }
        }
    }
    true
}

pub fn validateColorRequiresFragment(
    colorCount: u32,
    hasFragmentModule: bool,
    mut outError: Option<&mut String>,
) -> bool {
    if colorCount > 0 && !hasFragmentModule {
        if let Some(error) = outError.as_mut() {
            **error = "pipeline declares color outputs but has no fragment shader; supply `fragment`, or omit `colorTargets` for a depth-only pipeline".to_owned();
        }
        return false;
    }
    true
}

// } // namespace rive::ore
#[derive(Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum NativeSlotScope {
    perStage,
    perGroup,
    perKind,
}

pub fn validateSplitStageSlots(
    stagesCompiledApart: bool,
    mergedMap: &BindingMap,
    scope: NativeSlotScope,
    mut outError: Option<&mut String>,
) -> bool {
    if !stagesCompiledApart || scope == NativeSlotScope::perStage {
        return true;
    }
    let mut claimed: Vec<(u32, u16, u8, u8)> = Vec::with_capacity(mergedMap.size());
    for i in 0..mergedMap.size() {
        let e = mergedMap.at(i);
        let vs = e.backendSlot[0];
        let fs = e.backendSlot[1];
        if vs != BindingMap::kAbsent && fs != BindingMap::kAbsent && vs != fs {
            if let Some(error) = outError.as_mut() {
                **error = format!(
                    "@group({}) @binding({}): the vertex module put it on native slot {vs} and the fragment module on {fs}, and this backend shares one slot namespace between the stages. Declare the same bindings in both files, or compile both stages from one shader",
                    e.group, e.binding
                );
            }
            return false;
        }
        let slot = if vs != BindingMap::kAbsent { vs } else { fs };
        if slot == BindingMap::kAbsent {
            continue;
        }
        let scope_key = if scope == NativeSlotScope::perGroup {
            u32::from(e.group)
        } else {
            match e.kind {
                ResourceKind::UniformBuffer
                | ResourceKind::StorageBufferRO
                | ResourceKind::StorageBufferRW => 0,
                ResourceKind::SampledTexture | ResourceKind::StorageTexture => 1,
                ResourceKind::Sampler | ResourceKind::ComparisonSampler => 2,
                _ => 0,
            }
        };
        for &(key, previous_slot, group, binding) in &claimed {
            if key != scope_key || previous_slot != slot {
                continue;
            }
            if let Some(error) = outError.as_mut() {
                **error = format!(
                    "@group({}) @binding({}) and @group({group}) @binding({binding}) both land on native slot {slot}: the vertex and fragment modules were compiled apart, and this backend shares one slot namespace between them. Declare the same bindings in both, or compile both stages from one shader",
                    e.group, e.binding
                );
            }
            return false;
        }
        claimed.push((scope_key, slot, e.group, e.binding));
    }
    true
}

pub fn validateStagesAgree(
    vertexMap: &BindingMap,
    fragmentMap: &BindingMap,
    mut outError: Option<&mut String>,
) -> bool {
    for f in 0..fragmentMap.size() {
        let fs = fragmentMap.at(f);
        for v in 0..vertexMap.size() {
            let vs = vertexMap.at(v);
            if vs.group != fs.group || vs.binding != fs.binding {
                continue;
            }
            let both_samplers = matches!(
                vs.kind,
                ResourceKind::Sampler | ResourceKind::ComparisonSampler
            ) && matches!(
                fs.kind,
                ResourceKind::Sampler | ResourceKind::ComparisonSampler
            );
            let mismatch = if vs.kind != fs.kind && !both_samplers {
                Some("kind")
            } else if vs.textureViewDim != TextureViewDim::Undefined
                && fs.textureViewDim != TextureViewDim::Undefined
                && vs.textureViewDim != fs.textureViewDim
            {
                Some("texture dimension")
            } else if vs.textureSampleType != TextureSampleType::Undefined
                && fs.textureSampleType != TextureSampleType::Undefined
                && vs.textureSampleType != fs.textureSampleType
            {
                Some("texture sample type")
            } else if vs.minBindingSize != 0
                && fs.minBindingSize != 0
                && vs.minBindingSize != fs.minBindingSize
            {
                Some("uniform block size")
            } else {
                None
            };
            if let Some(what) = mismatch {
                if let Some(error) = outError.as_mut() {
                    **error = format!(
                        "@group({}) @binding({}): the vertex and fragment files declare it with a different {what}. A pipeline carries one declaration per binding, so the stage that loses reads what the other one bound",
                        fs.group, fs.binding
                    );
                }
                return false;
            }
            break;
        }
    }
    true
}

fn validatePipelineDescWithBases(
    desc: &PipelineDesc<'_>,
    mergedMap: &BindingMap,
    scope: NativeSlotScope,
    vertex: Option<&ShaderModule>,
    fragment: Option<&ShaderModule>,
    layouts: Option<&[Option<&BindGroupLayout>]>,
    mut outError: Option<&mut String>,
) -> bool {
    let stages_compiled_apart = match (desc.vertexModule, desc.fragmentModule) {
        (Some(v), Some(f)) => !v.ptr_eq(f),
        _ => false,
    };
    if stages_compiled_apart {
        if let (Some(v), Some(f)) = (vertex, fragment) {
            if !validateStagesAgree(&v.m_bindingMap, &f.m_bindingMap, outError.as_deref_mut()) {
                return false;
            }
        }
    }
    validateLayoutBasesAgainstBindingMap(
        mergedMap,
        layouts,
        desc.bindGroupLayoutCount,
        outError.as_deref_mut(),
    ) && validateColorRequiresFragment(
        desc.colorCount,
        desc.fragmentModule.is_some(),
        outError.as_deref_mut(),
    ) && validateSplitStageSlots(stages_compiled_apart, mergedMap, scope, outError)
}

pub fn validatePipelineDesc(
    desc: &PipelineDesc<'_>,
    mergedMap: &BindingMap,
    scope: NativeSlotScope,
    outError: Option<&mut String>,
) -> bool {
    let layouts = desc.bindGroupLayouts.map(|layouts| {
        layouts
            .iter()
            .map(|layout| layout.and_then(AnyResourceHandle::bindGroupLayoutBase))
            .collect::<Vec<_>>()
    });
    validatePipelineDescWithBases(
        desc,
        mergedMap,
        scope,
        desc.vertexModule
            .and_then(AnyResourceHandle::shaderModuleBase),
        desc.fragmentModule
            .and_then(AnyResourceHandle::shaderModuleBase),
        layouts.as_deref(),
        outError,
    )
}

/// Validate UBO ranges before a backend touches native objects. Null layouts
/// and missing bindings/buffers remain the backend's own errors to name.
pub fn validateBindGroupDesc(desc: &BindGroupDesc<'_>, mut outError: Option<&mut String>) -> bool {
    // Rust's pointer/count adaptation carries a bounded slice. Reject an
    // invalid count before recording dynamic ranges or touching native objects.
    if desc.uboCount as usize > desc.ubos.len() {
        if let Some(error) = outError.as_mut() {
            **error = format!(
                "uboCount {} exceeds the supplied UBO entry count {}",
                desc.uboCount,
                desc.ubos.len()
            );
        }
        return false;
    }
    let Some(layout) = desc.layout.and_then(AnyResourceHandle::bindGroupLayoutBase) else {
        return true;
    };
    for ubo in desc.ubos.iter().take(desc.uboCount as usize) {
        let Some(entry) = layout.findEntry(ubo.slot) else {
            continue;
        };
        if entry.kind != BindingKind::uniformBuffer {
            continue;
        }
        let Some(bufferSize) = ubo.buffer.and_then(AnyResourceHandle::size).map(u64::from) else {
            continue;
        };
        if u64::from(ubo.offset) > bufferSize
            || u64::from(ubo.offset) + u64::from(ubo.size) > bufferSize
        {
            if let Some(error) = outError.as_mut() {
                **error = format!(
                    "@group({}) @binding({}): offset {} + size {} exceeds the {} byte buffer",
                    layout.groupIndex(),
                    ubo.slot,
                    ubo.offset,
                    ubo.size,
                    bufferSize
                );
            }
            return false;
        }
        let bound = if ubo.size != 0 {
            u64::from(ubo.size)
        } else {
            bufferSize - u64::from(ubo.offset)
        };
        if bound < u64::from(entry.minBindingSize) {
            if let Some(error) = outError.as_mut() {
                **error = format!(
                    "@group({}) @binding({}): binds {} bytes but the shader's uniform block needs {}",
                    layout.groupIndex(),
                    ubo.slot,
                    bound,
                    entry.minBindingSize
                );
            }
            return false;
        }
    }
    true
}
