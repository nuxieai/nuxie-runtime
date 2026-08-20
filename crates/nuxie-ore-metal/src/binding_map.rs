// Mechanical translation of:
//   renderer/include/rive/renderer/ore/ore_binding_map.hpp
//   renderer/src/ore/ore_binding_map.cpp
// Upstream revision: 4ac7b32798da0482e441ef09304dc3b480ed3ee5

//! Portable ORE binding-map representation and RSTB serialization.
//!
//! `BindingMap` is populated from the RSTB sidecar at pipeline creation and
//! then consumed by backend binding/layout code. The tooling-only methods
//! mirror the editor-side construction and serialization API from upstream.

#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(non_upper_case_globals)]

/// Metal's reserved internal buffer slot (`[[buffer(30)]]`).
pub const METAL_RESERVED_BUFFER_SLOT: u32 = 30;

/// Maximum Metal buffer slot available to user bindings.
pub const METAL_MAX_USER_BUFFER_SLOT: u32 = METAL_RESERVED_BUFFER_SLOT - 1;

// Keep the C++ constant spellings used by the neighboring ORE translation.
pub const kMetalReservedBufferSlot: u32 = METAL_RESERVED_BUFFER_SLOT;
pub const kMetalMaxUserBufferSlot: u32 = METAL_MAX_USER_BUFFER_SLOT;

/// One resource kind from the frozen on-disk RSTB schema.
///
/// This is a transparent byte rather than a closed Rust enum because the
/// pinned C++ decoder accepts unknown enum bytes via `static_cast` and keeps
/// them in the entry. Keeping the byte makes forward-compatible blobs
/// lossless while the associated constants retain the upstream API shape.
#[repr(transparent)]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ResourceKind(pub u8);

impl ResourceKind {
    pub const UniformBuffer: Self = Self(0);
    pub const StorageBufferRO: Self = Self(1);
    pub const StorageBufferRW: Self = Self(2);
    pub const SampledTexture: Self = Self(3);
    pub const StorageTexture: Self = Self(4);
    pub const Sampler: Self = Self(5);
    pub const ComparisonSampler: Self = Self(6);

    pub const fn bits(self) -> u8 {
        self.0
    }
}

/// Texture view dimension from the frozen on-disk RSTB schema.
#[repr(transparent)]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct TextureViewDim(pub u8);

impl TextureViewDim {
    pub const Undefined: Self = Self(0);
    pub const D1: Self = Self(1);
    pub const D2: Self = Self(2);
    pub const D2Array: Self = Self(3);
    pub const Cube: Self = Self(4);
    pub const CubeArray: Self = Self(5);
    pub const D3: Self = Self(6);

    pub const fn bits(self) -> u8 {
        self.0
    }
}

/// Texture sample type from the frozen on-disk RSTB schema.
#[repr(transparent)]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct TextureSampleType(pub u8);

impl TextureSampleType {
    pub const Undefined: Self = Self(0);
    pub const Float: Self = Self(1);
    pub const UnfilterableFloat: Self = Self(2);
    pub const Depth: Self = Self(3);
    pub const Sint: Self = Self(4);
    pub const Uint: Self = Self(5);

    pub const fn bits(self) -> u8 {
        self.0
    }
}

/// Shader stage index for the fixed-width per-stage backend-slot fields.
#[repr(u8)]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Stage {
    VS = 0,
    FS = 1,
    CS = 2,
}

impl Stage {
    /// Rust-spelled aliases for callers that prefer descriptive names.
    pub const VERTEX: Self = Self::VS;
    pub const FRAGMENT: Self = Self::FS;
    pub const COMPUTE: Self = Self::CS;

    const fn index(self) -> usize {
        self as usize
    }

    const fn bit(self) -> u8 {
        1u8 << self as u8
    }
}

/// One row of the binding map.
///
/// The first fields and the packed `[u16; 3]` slots correspond to the current
/// 14-byte wire prefix. Texture reflection fields remain part of every entry,
/// even for non-texture kinds, with their exact upstream defaults.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct BindingMapEntry {
    pub group: u8,
    pub binding: u8,
    pub kind: ResourceKind,
    pub stageMask: u8,
    pub backendSpace: u8,
    pub backendSlot: [u16; 3],
    pub textureViewDim: TextureViewDim,
    pub textureSampleType: TextureSampleType,
    pub textureMultisampled: bool,
}

/// Short name corresponding to the C++ nested `BindingMap::Entry`.
pub type Entry = BindingMapEntry;

impl Default for BindingMapEntry {
    fn default() -> Self {
        Self {
            group: 0,
            binding: 0,
            kind: ResourceKind::UniformBuffer,
            stageMask: 0,
            backendSpace: 0,
            backendSlot: [BindingMap::K_ABSENT; 3],
            textureViewDim: TextureViewDim::Undefined,
            textureSampleType: TextureSampleType::Undefined,
            textureMultisampled: false,
        }
    }
}

/// Binding map populated from an RSTB sidecar.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct BindingMap {
    entries: Vec<BindingMapEntry>,
    #[cfg(feature = "tools")]
    finalized: bool,
}

impl BindingMap {
    fn reset_for_parse(&mut self) {
        self.entries.clear();
        #[cfg(feature = "tools")]
        {
            self.finalized = false;
        }
    }

    /// Stage bitmask bit for vertex visibility.
    pub const K_STAGE_VERTEX: u32 = 1u32 << 0;
    /// Stage bitmask bit for fragment visibility.
    pub const K_STAGE_FRAGMENT: u32 = 1u32 << 1;
    /// Stage bitmask bit for compute visibility.
    pub const K_STAGE_COMPUTE: u32 = 1u32 << 2;

    /// Sentinel for a resource that is not visible to a stage.
    pub const K_ABSENT: u16 = u16::MAX;

    /// RSTB blob version byte.
    pub const K_BLOB_VERSION: u8 = 2;

    /// Binding-map allocator version byte.
    pub const K_ALLOCATOR_VERSION: u8 = 1;

    /// Fixed known wire prefix for one entry.
    pub const K_ENTRY_WIRE_SIZE: u16 = 14;

    // Exact C++ spellings used in the pinned header.
    pub const kStageVertex: u32 = Self::K_STAGE_VERTEX;
    pub const kStageFragment: u32 = Self::K_STAGE_FRAGMENT;
    pub const kStageCompute: u32 = Self::K_STAGE_COMPUTE;
    pub const kAbsent: u16 = Self::K_ABSENT;
    pub const kBlobVersion: u8 = Self::K_BLOB_VERSION;
    pub const kAllocatorVersion: u8 = Self::K_ALLOCATOR_VERSION;

    // Descriptive aliases retain the same values without introducing another
    // source of truth for the protocol constants.
    pub const STAGE_VERTEX: u32 = Self::K_STAGE_VERTEX;
    pub const STAGE_FRAGMENT: u32 = Self::K_STAGE_FRAGMENT;
    pub const STAGE_COMPUTE: u32 = Self::K_STAGE_COMPUTE;
    pub const ABSENT: u16 = Self::K_ABSENT;
    pub const BLOB_VERSION: u8 = Self::K_BLOB_VERSION;
    pub const ALLOCATOR_VERSION: u8 = Self::K_ALLOCATOR_VERSION;
    pub const ENTRY_WIRE_SIZE: u16 = Self::K_ENTRY_WIRE_SIZE;

    /// Parse a blob produced by `to_blob`.
    ///
    /// The destination is cleared before size/version/payload validation, just
    /// as upstream clears `m_entries` before its first parse guard. A failed
    /// parse therefore leaves an empty map. Rust slices cannot represent a
    /// null data pointer; `from_blob_optional` preserves that distinct C++
    /// null-input path for callers that need it.
    pub fn from_blob(data: &[u8], out: &mut Self) -> bool {
        out.reset_for_parse();

        const HEADER_SIZE: usize = 8;
        if data.len() < HEADER_SIZE {
            return false;
        }
        if data[0] != Self::K_BLOB_VERSION {
            return false;
        }
        if data[1] != Self::K_ALLOCATOR_VERSION {
            return false;
        }

        let entry_size = read_u16_le(&data[2..4]);
        let entry_count = read_u32_le(&data[4..8]);
        let entry_size = usize::from(entry_size);
        if entry_size < usize::from(Self::K_ENTRY_WIRE_SIZE) {
            return false;
        }

        let Some(payload_size) = (entry_count as usize).checked_mul(entry_size) else {
            return false;
        };
        let Some(needed) = HEADER_SIZE.checked_add(payload_size) else {
            return false;
        };
        if data.len() < needed {
            return false;
        }

        out.entries.reserve(entry_count as usize);
        let mut offset = HEADER_SIZE;
        for _ in 0..entry_count {
            let row = &data[offset..offset + usize::from(Self::K_ENTRY_WIRE_SIZE)];
            let mut entry = BindingMapEntry::default();
            entry.group = row[0];
            entry.binding = row[1];
            entry.kind = resource_kind_from_u8(row[2]);
            entry.stageMask = row[3];
            entry.backendSpace = row[4];
            entry.backendSlot[0] = read_u16_le(&row[5..7]);
            entry.backendSlot[1] = read_u16_le(&row[7..9]);
            entry.backendSlot[2] = read_u16_le(&row[9..11]);
            entry.textureViewDim = texture_view_dim_from_u8(row[11]);
            entry.textureSampleType = texture_sample_type_from_u8(row[12]);
            entry.textureMultisampled = row[13] != 0;
            out.entries.push(entry);
            // `needed` proves this addition is in-bounds for every iteration;
            // the checked parse above keeps hostile lengths from wrapping.
            offset += entry_size;
        }

        #[cfg(feature = "tools")]
        {
            // `from_blob` trusts the serialized canonical order and performs
            // no sort on the runtime path, matching upstream.
            out.finalized = true;
        }
        true
    }

    /// C++-spelled entry point retained for source-corresponding callers.
    /// `size` selects the prefix of the supplied borrowed buffer.
    pub fn fromBlob(data: &[u8], size: usize, out: &mut Self) -> bool {
        let Some(data) = data.get(..size) else {
            out.reset_for_parse();
            return false;
        };
        Self::from_blob(data, out)
    }

    /// Preserve the C++ `nullptr` data/output guards when a caller needs them.
    pub fn from_blob_optional(data: Option<&[u8]>, out: Option<&mut Self>) -> bool {
        let (Some(data), Some(out)) = (data, out) else {
            return false;
        };
        Self::from_blob(data, out)
    }

    /// C++-null-aware spelling of the optional-input adapter.
    pub fn fromBlobOptional(data: Option<&[u8]>, size: usize, out: Option<&mut Self>) -> bool {
        let (Some(data), Some(out)) = (data, out) else {
            return false;
        };
        let Some(data) = data.get(..size) else {
            out.reset_for_parse();
            return false;
        };
        Self::from_blob(data, out)
    }

    /// Look up one stage's backend slot.
    ///
    /// Sampler and comparison-sampler kinds intentionally collapse to the
    /// same runtime bind-API category. Stage visibility is checked before the
    /// per-stage slot is returned.
    pub fn lookup(&self, group: u32, binding: u32, kind: ResourceKind, stage: Stage) -> u16 {
        let Some(entry) = self.find_entry(group, binding) else {
            return Self::K_ABSENT;
        };
        let kind_matches =
            entry.kind == kind || (is_sampler_kind(kind) && is_sampler_kind(entry.kind));
        if !kind_matches {
            return Self::K_ABSENT;
        }
        if entry.stageMask & stage.bit() == 0 {
            return Self::K_ABSENT;
        }
        entry.backendSlot[stage.index()]
    }

    /// Number of entries in the map.
    pub fn size(&self) -> usize {
        self.entries.len()
    }

    /// Whether this map has no entries.
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// C++-spelled emptiness accessor.
    pub fn empty(&self) -> bool {
        self.is_empty()
    }

    /// Runtime iteration accessor.
    pub fn at(&self, index: usize) -> &BindingMapEntry {
        &self.entries[index]
    }

    /// Tooling construction: append an entry and invalidate finalization.
    #[cfg(feature = "tools")]
    pub fn push(&mut self, entry: BindingMapEntry) {
        self.entries.push(entry);
        self.finalized = false;
    }

    /// Tooling construction: append a borrowed entry using C++ copy semantics.
    #[cfg(feature = "tools")]
    pub fn push_ref(&mut self, entry: &BindingMapEntry) {
        self.push(*entry);
    }

    /// Value-taking alias used by the translated tests and producer code.
    #[cfg(feature = "tools")]
    pub fn push_entry(&mut self, entry: BindingMapEntry) {
        self.push(entry);
    }

    /// Serialize the canonical entry vector to the RSTB sidecar format.
    #[cfg(feature = "tools")]
    pub fn to_blob(&self) -> Vec<u8> {
        const HEADER_SIZE: usize = 8;
        let entry_size = usize::from(Self::K_ENTRY_WIRE_SIZE);
        let mut blob = vec![0u8; HEADER_SIZE + self.entries.len() * entry_size];
        blob[0] = Self::K_BLOB_VERSION;
        blob[1] = Self::K_ALLOCATOR_VERSION;
        write_u16_le(&mut blob[2..4], Self::K_ENTRY_WIRE_SIZE);
        write_u32_le(&mut blob[4..8], self.entries.len() as u32);

        let mut offset = HEADER_SIZE;
        for entry in &self.entries {
            let row = &mut blob[offset..offset + entry_size];
            row[0] = entry.group;
            row[1] = entry.binding;
            row[2] = entry.kind.0;
            row[3] = entry.stageMask;
            row[4] = entry.backendSpace;
            write_u16_le(&mut row[5..7], entry.backendSlot[0]);
            write_u16_le(&mut row[7..9], entry.backendSlot[1]);
            write_u16_le(&mut row[9..11], entry.backendSlot[2]);
            row[11] = entry.textureViewDim.0;
            row[12] = entry.textureSampleType.0;
            row[13] = u8::from(entry.textureMultisampled);
            offset += entry_size;
        }
        blob
    }

    /// C++-spelled tooling entry point retained for source-corresponding
    /// callers.
    #[cfg(feature = "tools")]
    pub fn toBlob(&self) -> Vec<u8> {
        self.to_blob()
    }

    /// Sort entries into canonical `(group, binding)` order.
    #[cfg(feature = "tools")]
    pub fn finalize(&mut self) {
        self.entries.sort_unstable_by(|a, b| {
            a.group
                .cmp(&b.group)
                .then_with(|| a.binding.cmp(&b.binding))
        });
        self.finalized = true;
    }

    /// Whether tooling construction has been finalized.
    #[cfg(feature = "tools")]
    pub fn is_finalized(&self) -> bool {
        self.finalized
    }

    /// Tooling-only accessor for the backing vector.
    #[cfg(feature = "tools")]
    pub fn entries(&self) -> &Vec<BindingMapEntry> {
        &self.entries
    }

    /// Tooling-only full-entry lookup.
    #[cfg(feature = "tools")]
    pub fn lookup_entry(&self, group: u32, binding: u32) -> Option<&BindingMapEntry> {
        self.find_entry(group, binding)
    }

    /// C++-spelled tooling full-entry lookup.
    #[cfg(feature = "tools")]
    pub fn lookupEntry(&self, group: u32, binding: u32) -> Option<&BindingMapEntry> {
        self.lookup_entry(group, binding)
    }

    #[cfg(feature = "tools")]
    pub fn isFinalized(&self) -> bool {
        self.is_finalized()
    }

    fn find_entry(&self, group: u32, binding: u32) -> Option<&BindingMapEntry> {
        #[cfg(feature = "tools")]
        debug_assert!(self.finalized, "BindingMap::lookup before finalize");

        // Mirror `std::lower_bound`, which returns the first equivalent row
        // when malformed/tooling input contains duplicate keys. Rust's
        // `binary_search_by` may return any equivalent index instead.
        let mut first = 0;
        let mut last = self.entries.len();
        while first < last {
            let middle = first + (last - first) / 2;
            let key = (
                self.entries[middle].group as u32,
                self.entries[middle].binding as u32,
            );
            if key < (group, binding) {
                first = middle + 1;
            } else {
                last = middle;
            }
        }
        if first < self.entries.len()
            && self.entries[first].group as u32 == group
            && self.entries[first].binding as u32 == binding
        {
            Some(&self.entries[first])
        } else {
            None
        }
    }
}

/// Shared helper for backend bind-group construction.
pub fn lookup_backend_slot(
    map: &BindingMap,
    group: u32,
    binding: u32,
    kind: ResourceKind,
    stage: Stage,
) -> u16 {
    let slot = map.lookup(group, binding, kind, stage);
    debug_assert_ne!(
        slot,
        BindingMap::K_ABSENT,
        "BindingMap lookup failed for (group, binding, kind, stage)"
    );
    slot
}

/// C++-spelled helper retained for source-corresponding callers.
pub fn lookupBackendSlot(
    map: &BindingMap,
    group: u32,
    binding: u32,
    kind: ResourceKind,
    stage: Stage,
) -> u16 {
    lookup_backend_slot(map, group, binding, kind, stage)
}

fn is_sampler_kind(kind: ResourceKind) -> bool {
    matches!(
        kind,
        ResourceKind::Sampler | ResourceKind::ComparisonSampler
    )
}

fn resource_kind_from_u8(value: u8) -> ResourceKind {
    ResourceKind(value)
}

fn texture_view_dim_from_u8(value: u8) -> TextureViewDim {
    TextureViewDim(value)
}

fn texture_sample_type_from_u8(value: u8) -> TextureSampleType {
    TextureSampleType(value)
}

fn read_u16_le(bytes: &[u8]) -> u16 {
    u16::from(bytes[0]) | (u16::from(bytes[1]) << 8)
}

fn read_u32_le(bytes: &[u8]) -> u32 {
    u32::from(bytes[0])
        | (u32::from(bytes[1]) << 8)
        | (u32::from(bytes[2]) << 16)
        | (u32::from(bytes[3]) << 24)
}

#[cfg(feature = "tools")]
fn write_u16_le(bytes: &mut [u8], value: u16) {
    bytes[0] = (value & 0xff) as u8;
    bytes[1] = (value >> 8) as u8;
}

#[cfg(feature = "tools")]
fn write_u32_le(bytes: &mut [u8], value: u32) {
    bytes[0] = (value & 0xff) as u8;
    bytes[1] = ((value >> 8) & 0xff) as u8;
    bytes[2] = ((value >> 16) & 0xff) as u8;
    bytes[3] = ((value >> 24) & 0xff) as u8;
}

#[cfg(all(test, feature = "tools"))]
mod tests {
    use super::*;

    fn make_entry(
        group: u8,
        binding: u8,
        kind: ResourceKind,
        slot_vs: u16,
        slot_fs: u16,
        slot_cs: u16,
        stageMask: u8,
        backendSpace: u8,
    ) -> BindingMapEntry {
        BindingMapEntry {
            group,
            binding,
            kind,
            stageMask,
            backendSpace,
            backendSlot: [slot_vs, slot_fs, slot_cs],
            ..BindingMapEntry::default()
        }
    }

    #[test]
    fn binding_map_lookup_after_finalize() {
        let mut map = BindingMap::default();
        map.push_entry(make_entry(
            1,
            2,
            ResourceKind::SampledTexture,
            5,
            5,
            BindingMap::K_ABSENT,
            (BindingMap::K_STAGE_VERTEX | BindingMap::K_STAGE_FRAGMENT) as u8,
            0,
        ));
        map.push_entry(make_entry(
            0,
            1,
            ResourceKind::UniformBuffer,
            1,
            1,
            BindingMap::K_ABSENT,
            (BindingMap::K_STAGE_VERTEX | BindingMap::K_STAGE_FRAGMENT) as u8,
            0,
        ));
        map.push_entry(make_entry(
            0,
            0,
            ResourceKind::UniformBuffer,
            0,
            0,
            BindingMap::K_ABSENT,
            (BindingMap::K_STAGE_VERTEX | BindingMap::K_STAGE_FRAGMENT) as u8,
            0,
        ));
        map.finalize();

        assert_eq!(map.size(), 3);
        assert_eq!(map.at(0).group, 0);
        assert_eq!(map.at(0).binding, 0);
        assert_eq!(map.at(1).group, 0);
        assert_eq!(map.at(1).binding, 1);
        assert_eq!(map.at(2).group, 1);
        assert_eq!(map.at(2).binding, 2);
        assert_eq!(map.lookup(0, 0, ResourceKind::UniformBuffer, Stage::VS), 0);
        assert_eq!(map.lookup(0, 1, ResourceKind::UniformBuffer, Stage::VS), 1);
        assert_eq!(map.lookup(1, 2, ResourceKind::SampledTexture, Stage::VS), 5);
        assert_eq!(
            map.lookup(9, 9, ResourceKind::UniformBuffer, Stage::VS),
            BindingMap::K_ABSENT
        );
        assert_eq!(
            map.lookup(0, 0, ResourceKind::Sampler, Stage::VS),
            BindingMap::K_ABSENT
        );
    }

    #[test]
    fn binding_map_stage_visibility() {
        let mut map = BindingMap::default();
        map.push_entry(make_entry(
            0,
            0,
            ResourceKind::UniformBuffer,
            3,
            BindingMap::K_ABSENT,
            BindingMap::K_ABSENT,
            BindingMap::K_STAGE_VERTEX as u8,
            0,
        ));
        map.finalize();
        assert_eq!(map.lookup(0, 0, ResourceKind::UniformBuffer, Stage::VS), 3);
        assert_eq!(
            map.lookup(0, 0, ResourceKind::UniformBuffer, Stage::FS),
            BindingMap::K_ABSENT
        );
        assert_eq!(
            map.lookup(0, 0, ResourceKind::UniformBuffer, Stage::CS),
            BindingMap::K_ABSENT
        );
    }

    #[test]
    fn binding_map_fs_only_entry_hides_vs_cs() {
        let mut map = BindingMap::default();
        map.push_entry(make_entry(
            2,
            0,
            ResourceKind::SampledTexture,
            BindingMap::K_ABSENT,
            4,
            BindingMap::K_ABSENT,
            BindingMap::K_STAGE_FRAGMENT as u8,
            0,
        ));
        map.finalize();
        assert_eq!(map.lookup(2, 0, ResourceKind::SampledTexture, Stage::FS), 4);
        assert_eq!(
            map.lookup(2, 0, ResourceKind::SampledTexture, Stage::VS),
            BindingMap::K_ABSENT
        );
        assert_eq!(
            map.lookup(2, 0, ResourceKind::SampledTexture, Stage::CS),
            BindingMap::K_ABSENT
        );
    }

    #[test]
    fn binding_map_empty_stage_mask_hides_every_stage() {
        let mut map = BindingMap::default();
        map.push_entry(make_entry(0, 0, ResourceKind::UniformBuffer, 9, 9, 9, 0, 0));
        map.finalize();
        for stage in [Stage::VS, Stage::FS, Stage::CS] {
            assert_eq!(
                map.lookup(0, 0, ResourceKind::UniformBuffer, stage),
                BindingMap::K_ABSENT
            );
        }
    }

    #[test]
    fn binding_map_sampler_comparison_sampler_kind_collapse() {
        let mut map = BindingMap::default();
        map.push_entry(make_entry(
            0,
            0,
            ResourceKind::ComparisonSampler,
            7,
            7,
            BindingMap::K_ABSENT,
            (BindingMap::K_STAGE_VERTEX | BindingMap::K_STAGE_FRAGMENT) as u8,
            0,
        ));
        map.finalize();
        assert_eq!(
            map.lookup(0, 0, ResourceKind::ComparisonSampler, Stage::FS),
            7
        );
        assert_eq!(map.lookup(0, 0, ResourceKind::Sampler, Stage::FS), 7);
        assert_eq!(
            map.lookup(0, 0, ResourceKind::SampledTexture, Stage::FS),
            BindingMap::K_ABSENT
        );
        assert_eq!(
            map.lookup(0, 0, ResourceKind::UniformBuffer, Stage::FS),
            BindingMap::K_ABSENT
        );

        let mut reverse = BindingMap::default();
        reverse.push_entry(make_entry(
            0,
            0,
            ResourceKind::Sampler,
            2,
            2,
            BindingMap::K_ABSENT,
            (BindingMap::K_STAGE_VERTEX | BindingMap::K_STAGE_FRAGMENT) as u8,
            0,
        ));
        reverse.finalize();
        assert_eq!(reverse.lookup(0, 0, ResourceKind::Sampler, Stage::FS), 2);
        assert_eq!(
            reverse.lookup(0, 0, ResourceKind::ComparisonSampler, Stage::FS),
            2
        );
    }

    #[test]
    fn binding_map_per_stage_slots_can_disagree() {
        let mut map = BindingMap::default();
        map.push_entry(make_entry(
            0,
            0,
            ResourceKind::SampledTexture,
            1,
            4,
            9,
            (BindingMap::K_STAGE_VERTEX
                | BindingMap::K_STAGE_FRAGMENT
                | BindingMap::K_STAGE_COMPUTE) as u8,
            0,
        ));
        map.finalize();
        assert_eq!(map.lookup(0, 0, ResourceKind::SampledTexture, Stage::VS), 1);
        assert_eq!(map.lookup(0, 0, ResourceKind::SampledTexture, Stage::FS), 4);
        assert_eq!(map.lookup(0, 0, ResourceKind::SampledTexture, Stage::CS), 9);
    }

    #[test]
    fn binding_map_to_blob_from_blob_round_trip() {
        let mut original = BindingMap::default();
        original.push_entry(make_entry(
            0,
            0,
            ResourceKind::UniformBuffer,
            0,
            0,
            BindingMap::K_ABSENT,
            (BindingMap::K_STAGE_VERTEX | BindingMap::K_STAGE_FRAGMENT) as u8,
            0,
        ));
        original.push_entry(make_entry(
            0,
            7,
            ResourceKind::UniformBuffer,
            1,
            1,
            BindingMap::K_ABSENT,
            (BindingMap::K_STAGE_VERTEX | BindingMap::K_STAGE_FRAGMENT) as u8,
            0,
        ));
        original.push_entry(make_entry(
            2,
            0,
            ResourceKind::SampledTexture,
            5,
            5,
            BindingMap::K_ABSENT,
            BindingMap::K_STAGE_FRAGMENT as u8,
            2,
        ));
        original.finalize();

        let blob = original.to_blob();
        assert_eq!(blob.len(), 50);
        assert_eq!(blob[0], BindingMap::K_BLOB_VERSION);
        assert_eq!(blob[1], BindingMap::K_ALLOCATOR_VERSION);
        assert_eq!(&blob[2..4], &[14, 0]);
        assert_eq!(&blob[4..8], &[3, 0, 0, 0]);

        let mut restored = BindingMap::default();
        assert!(BindingMap::from_blob(&blob, &mut restored));
        assert_eq!(restored.size(), 3);
        for index in 0..original.size() {
            assert_eq!(original.at(index), restored.at(index));
        }
    }

    #[test]
    fn binding_map_rejects_bad_blob_version() {
        let mut source = BindingMap::default();
        source.push_entry(make_entry(
            0,
            0,
            ResourceKind::UniformBuffer,
            0,
            0,
            BindingMap::K_ABSENT,
            (BindingMap::K_STAGE_VERTEX | BindingMap::K_STAGE_FRAGMENT) as u8,
            0,
        ));
        source.finalize();
        let blob = source.to_blob();
        let mut out = BindingMap::default();
        assert!(!BindingMap::from_blob_optional(None, Some(&mut out)));
        assert!(!BindingMap::from_blob(&blob[..4], &mut out));

        let mut bad_blob_version = blob.clone();
        bad_blob_version[0] = BindingMap::K_BLOB_VERSION + 1;
        assert!(!BindingMap::from_blob(&bad_blob_version, &mut out));

        let mut bad_allocator_version = blob;
        bad_allocator_version[1] = BindingMap::K_ALLOCATOR_VERSION + 1;
        assert!(!BindingMap::from_blob(&bad_allocator_version, &mut out));
    }

    #[test]
    fn binding_map_rejects_truncated_blob() {
        let mut source = BindingMap::default();
        source.push_entry(make_entry(
            0,
            0,
            ResourceKind::UniformBuffer,
            0,
            0,
            BindingMap::K_ABSENT,
            (BindingMap::K_STAGE_VERTEX | BindingMap::K_STAGE_FRAGMENT) as u8,
            0,
        ));
        source.push_entry(make_entry(
            1,
            0,
            ResourceKind::SampledTexture,
            0,
            0,
            BindingMap::K_ABSENT,
            (BindingMap::K_STAGE_VERTEX | BindingMap::K_STAGE_FRAGMENT) as u8,
            0,
        ));
        source.finalize();
        let blob = source.to_blob();
        let mut out = BindingMap::default();
        assert!(BindingMap::from_blob(&blob, &mut out));
        assert!(!BindingMap::from_blob(&blob[..blob.len() - 1], &mut out));
        assert!(out.is_empty());
    }

    #[test]
    fn binding_map_forward_compat_larger_entry_size_parses() {
        let mut source = BindingMap::default();
        source.push_entry(make_entry(
            0,
            0,
            ResourceKind::UniformBuffer,
            0,
            0,
            BindingMap::K_ABSENT,
            (BindingMap::K_STAGE_VERTEX | BindingMap::K_STAGE_FRAGMENT) as u8,
            0,
        ));
        source.finalize();
        let blob = source.to_blob();
        let current_entry_size = u16::from_le_bytes([blob[2], blob[3]]);
        const EXTRA_TRAILING: u16 = 4;
        let future_entry_size = current_entry_size + EXTRA_TRAILING;
        let mut future_blob = vec![0u8; 8 + usize::from(future_entry_size)];
        future_blob[0] = BindingMap::K_BLOB_VERSION;
        future_blob[1] = BindingMap::K_ALLOCATOR_VERSION;
        future_blob[2..4].copy_from_slice(&future_entry_size.to_le_bytes());
        future_blob[4] = 1;
        future_blob[8..8 + usize::from(current_entry_size)]
            .copy_from_slice(&blob[8..8 + usize::from(current_entry_size)]);
        future_blob[8 + usize::from(current_entry_size)..].fill(0xff);

        let mut out = BindingMap::default();
        assert!(BindingMap::from_blob(&future_blob, &mut out));
        assert_eq!(out.size(), 1);
        assert_eq!(out.at(0).group, 0);
        assert_eq!(out.at(0).binding, 0);
        assert_eq!(out.at(0).kind, ResourceKind::UniformBuffer);
    }

    #[test]
    fn binding_map_forward_compat_smaller_entry_size_rejected() {
        let mut blob = vec![0u8; 8];
        blob[0] = BindingMap::K_BLOB_VERSION;
        blob[1] = BindingMap::K_ALLOCATOR_VERSION;
        blob[2..4].copy_from_slice(&10u16.to_le_bytes());
        let mut out = BindingMap::default();
        assert!(!BindingMap::from_blob(&blob, &mut out));
    }

    #[test]
    fn binding_map_accepts_unknown_enum_bytes_and_nonzero_bool() {
        let mut blob = vec![0u8; 8 + usize::from(BindingMap::K_ENTRY_WIRE_SIZE)];
        blob[0] = BindingMap::K_BLOB_VERSION;
        blob[1] = BindingMap::K_ALLOCATOR_VERSION;
        blob[2..4].copy_from_slice(&BindingMap::K_ENTRY_WIRE_SIZE.to_le_bytes());
        blob[4] = 1;
        blob[8 + 2] = 0xfa;
        blob[8 + 11] = 0xfb;
        blob[8 + 12] = 0xfc;
        blob[8 + 13] = 0x7f;

        let mut out = BindingMap::default();
        assert!(BindingMap::from_blob(&blob, &mut out));
        assert_eq!(out.at(0).kind.0, 0xfa);
        assert_eq!(out.at(0).textureViewDim.0, 0xfb);
        assert_eq!(out.at(0).textureSampleType.0, 0xfc);
        assert!(out.at(0).textureMultisampled);
        let serialized = out.to_blob();
        assert_eq!(serialized[8 + 2], 0xfa);
        assert_eq!(serialized[8 + 11], 0xfb);
        assert_eq!(serialized[8 + 12], 0xfc);
        assert_eq!(serialized[8 + 13], 1);
    }

    #[test]
    fn oversized_size_adapters_reset_output_before_rejecting() {
        let mut populated = BindingMap::default();
        populated.push_entry(make_entry(
            0,
            0,
            ResourceKind::UniformBuffer,
            0,
            0,
            BindingMap::K_ABSENT,
            BindingMap::K_STAGE_VERTEX as u8,
            0,
        ));
        populated.finalize();
        let blob = populated.to_blob();

        let mut out = populated.clone();
        assert!(!BindingMap::fromBlob(&blob, blob.len() + 1, &mut out));
        assert!(out.is_empty());
        assert!(!out.is_finalized());

        out = populated;
        assert!(!BindingMap::fromBlobOptional(
            Some(&blob),
            blob.len() + 1,
            Some(&mut out),
        ));
        assert!(out.is_empty());
        assert!(!out.is_finalized());
    }

    #[test]
    fn resource_kind_numeric_values_are_frozen() {
        assert_eq!(ResourceKind::UniformBuffer.0, 0);
        assert_eq!(ResourceKind::StorageBufferRO.0, 1);
        assert_eq!(ResourceKind::StorageBufferRW.0, 2);
        assert_eq!(ResourceKind::SampledTexture.0, 3);
        assert_eq!(ResourceKind::StorageTexture.0, 4);
        assert_eq!(ResourceKind::Sampler.0, 5);
        assert_eq!(ResourceKind::ComparisonSampler.0, 6);
    }

    #[test]
    fn lookup_backend_slot_helper() {
        let mut map = BindingMap::default();
        map.push_entry(make_entry(
            0,
            0,
            ResourceKind::UniformBuffer,
            7,
            7,
            BindingMap::K_ABSENT,
            (BindingMap::K_STAGE_VERTEX | BindingMap::K_STAGE_FRAGMENT) as u8,
            0,
        ));
        map.finalize();
        assert_eq!(
            lookup_backend_slot(&map, 0, 0, ResourceKind::UniformBuffer, Stage::VS),
            7
        );
        assert_eq!(
            lookup_backend_slot(&map, 0, 0, ResourceKind::UniformBuffer, Stage::FS),
            7
        );
    }
}
