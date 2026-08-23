/*
 * Copyright 2026 Rive
 */

// #include "rive/renderer/ore/ore_binding_map.hpp"
// #include <algorithm>
// #include <cstring>

// Mechanical translation of the complete pinned source implementation
// renderer/src/ore/ore_binding_map.cpp.
// Upstream source revision: 4ac7b32798da0482e441ef09304dc3b480ed3ee5

#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(non_upper_case_globals)]
use super::*;

// BindingMap serialization (toBlob/fromBlob), sort/finalize, and lookup.
// All construction at runtime goes through `fromBlob` against an RSTB
// sidecar; the editor-side toolchain produces the blob via `toBlob`.

// namespace rive::ore

mod binding_map_detail {
    // On-disk blob layout:
    //
    //   offset 0  [u8]  blob_version        (= kBlobVersion)
    //          1  [u8]  allocator_version   (= kAllocatorVersion)
    //          2  [u16] entry_size (LE)     (grows append-only)
    //          4  [u32] entry_count (LE)
    //          8  [entry_count * entry_size] entries
    //
    // Each entry (entry_size = 14 bytes, no trailing alignment):
    //
    //          0  [u8]  group
    //          1  [u8]  binding
    //          2  [u8]  kind (ResourceKind)
    //          3  [u8]  stageMask
    //          4  [u8]  backendSpace
    //          5  [u16] backendSlot[0] (VS, LE)
    //          7  [u16] backendSlot[1] (FS, LE)
    //          9  [u16] backendSlot[2] (CS, LE)
    //         11  [u8]  textureViewDim (TextureViewDim)
    //         12  [u8]  textureSampleType (TextureSampleType)
    //         13  [u8]  textureMultisampled (0 or 1)
    //
    // Forward compat: a newer writer may emit entries larger than the current
    // reader knows about by bumping entry_size. The reader skips the trailing
    // unknown bytes per entry. New fields are always *appended* at the tail.
    // No reserved-for-future slots inside the known prefix, since entry_size
    // already gives us self-describing append-only growth. Any mismatch that
    // matters semantically (blob_version or allocator_version) is a loud
    // error.

    pub(super) const kBlobHeaderSize: usize = 8;
    pub(super) const kEntryWireSize: u16 = 14;

    pub(super) fn readU16LE(p: &[u8]) -> u16 {
        u16::from(p[0]) | (u16::from(p[1]) << 8)
    }

    pub(super) fn readU32LE(p: &[u8]) -> u32 {
        u32::from(p[0]) | (u32::from(p[1]) << 8) | (u32::from(p[2]) << 16) | (u32::from(p[3]) << 24)
    }

    // #ifdef WITH_RIVE_TOOLS
    #[cfg(feature = "with-rive-tools")]
    pub(super) fn writeU16LE(p: &mut [u8], v: u16) {
        p[0] = (v & 0xFF) as u8;
        p[1] = ((v >> 8) & 0xFF) as u8;
    }

    #[cfg(feature = "with-rive-tools")]
    pub(super) fn writeU32LE(p: &mut [u8], v: u32) {
        p[0] = (v & 0xFF) as u8;
        p[1] = ((v >> 8) & 0xFF) as u8;
        p[2] = ((v >> 16) & 0xFF) as u8;
        p[3] = ((v >> 24) & 0xFF) as u8;
    }
    // #endif
}

// } // namespace

// Runtime API: fromBlob trusts its input to be sorted (toBlob iterates
// m_entries in canonical order). No sort on the hot path.
//
// `Option<&[u8]>` and `Option<&mut BindingMap>` retain the two nullable C++
// pointer arguments and their combined null guard. The `size` argument still
// selects the authored byte range; the sibling translated header owns the
// corresponding value and field declarations.
impl BindingMap {
    pub fn fromBlob(data: Option<&[u8]>, size: usize, out: Option<&mut BindingMap>) -> bool {
        let (Some(data), Some(out)) = (data, out) else {
            return false;
        };
        out.m_entries.clear();
        // #ifdef WITH_RIVE_TOOLS
        #[cfg(feature = "with-rive-tools")]
        {
            out.m_finalized = false;
        }
        // #endif

        if size < binding_map_detail::kBlobHeaderSize {
            return false;
        }
        let Some(data) = data.get(..size) else {
            return false;
        };

        let blobVer: u8 = data[0];
        let allocVer: u8 = data[1];
        if blobVer != Self::kBlobVersion {
            return false; // Never silent-fallback on a malformed blob.
        }
        if allocVer != Self::kAllocatorVersion {
            return false;
        }

        let entrySize: u16 = binding_map_detail::readU16LE(&data[2..]);
        let entryCount: u32 = binding_map_detail::readU32LE(&data[4..]);

        // Reject writers that emit fewer fields than the reader needs.
        // Larger entry_size is fine — trailing unknown bytes are skipped.
        if entrySize < binding_map_detail::kEntryWireSize {
            return false;
        }

        let needed: usize =
            binding_map_detail::kBlobHeaderSize + (entryCount as usize) * (entrySize as usize);
        if size < needed {
            return false;
        }

        out.m_entries.reserve(entryCount as usize);
        let mut p: &[u8] = &data[binding_map_detail::kBlobHeaderSize..];
        for _i in 0..entryCount {
            let mut e: Entry = Entry::default();
            e.group = p[0];
            e.binding = p[1];
            e.kind = ResourceKind(p[2]);
            e.stageMask = p[3];
            e.backendSpace = p[4];
            e.backendSlot[0] = binding_map_detail::readU16LE(&p[5..]);
            e.backendSlot[1] = binding_map_detail::readU16LE(&p[7..]);
            e.backendSlot[2] = binding_map_detail::readU16LE(&p[9..]);
            e.textureViewDim = TextureViewDim(p[11]);
            e.textureSampleType = TextureSampleType(p[12]);
            e.textureMultisampled = p[13] != 0;
            // bytes [kEntryWireSize..entrySize] are future-version fields — skip.
            out.m_entries.push(e);
            p = &p[entrySize as usize..];
        }
        // #ifdef WITH_RIVE_TOOLS
        // Flip the finalized flag so tooling-build lookups satisfy their assert.
        // The blob is already sorted by construction; no std::sort call.
        #[cfg(feature = "with-rive-tools")]
        {
            out.m_finalized = true;
        }
        // #endif
        true
    }

    // #ifdef WITH_RIVE_TOOLS
    #[cfg(feature = "with-rive-tools")]
    pub fn toBlob(&self) -> Vec<u8> {
        let mut blob: Vec<u8> = vec![
            0;
            binding_map_detail::kBlobHeaderSize
                + self.m_entries.len()
                    * (binding_map_detail::kEntryWireSize as usize)
        ];
        blob[0] = Self::kBlobVersion;
        blob[1] = Self::kAllocatorVersion;
        binding_map_detail::writeU16LE(&mut blob[2..], binding_map_detail::kEntryWireSize);
        binding_map_detail::writeU32LE(&mut blob[4..], self.m_entries.len() as u32);

        let mut p: &mut [u8] = &mut blob[binding_map_detail::kBlobHeaderSize..];
        for e in &self.m_entries {
            p[0] = e.group;
            p[1] = e.binding;
            p[2] = e.kind.0;
            p[3] = e.stageMask;
            p[4] = e.backendSpace;
            binding_map_detail::writeU16LE(&mut p[5..], e.backendSlot[0]);
            binding_map_detail::writeU16LE(&mut p[7..], e.backendSlot[1]);
            binding_map_detail::writeU16LE(&mut p[9..], e.backendSlot[2]);
            p[11] = e.textureViewDim.0;
            p[12] = e.textureSampleType.0;
            p[13] = if e.textureMultisampled { 1u8 } else { 0u8 };
            p = &mut p[binding_map_detail::kEntryWireSize as usize..];
        }
        blob
    }

    #[cfg(feature = "with-rive-tools")]
    pub fn finalize(&mut self) {
        self.m_entries.sort_unstable_by(|a, b| {
            if a.group != b.group {
                return a.group.cmp(&b.group);
            }
            a.binding.cmp(&b.binding)
        });
        self.m_finalized = true;
    }
    // #endif // WITH_RIVE_TOOLS
}

// } // namespace rive::ore
