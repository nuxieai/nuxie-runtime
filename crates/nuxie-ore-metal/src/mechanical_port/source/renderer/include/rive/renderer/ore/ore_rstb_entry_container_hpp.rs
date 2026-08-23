// #pragma once
//
// #include <cstdint>
// #include <string>
// #include <vector>

// Mechanical translation of the complete pinned source header
// renderer/include/rive/renderer/ore/ore_rstb_entry_container.hpp.
// Upstream source revision: 4ac7b32798da0482e441ef09304dc3b480ed3ee5

#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(non_upper_case_globals)]

// RSTB v4 source-variant containers. Every source variant (WGSL/GLSL/MSL/HLSL/
// SPIR-V) carries a self-describing entry table so the runtime resolves
// @vertex/@fragment by name (WebGPU-style) instead of assuming vs_main/fs_main.
//
//   logical  - the WGSL entry-point name a script matches against.
//   physical - the name the driver looks up: == logical for WGSL/SPIR-V (naga
//              writes them verbatim), the naga-emitted name for MSL (its Namer
//              sanitizes MSL keywords), "main" for GLSL, the SPIRV-Cross
//              cleansed name for HLSL.
//
// Two shapes. Whole-module targets (WGSL=0, MSL=2, SPIR-V=5) share one source:
//   u8 count, count*{u8 stage, str logical, str physical}, u32 srcLen, src
// Per-entry targets (GLSL=1, HLSL=3) carry a source per entry:
//   u8 count, count*{u8 stage, str logical, str physical, u32 srcLen, src}
// where str is u16 len + bytes (no NUL). Order is naga declaration order, so
// the first vertex/fragment entry is the WebGPU "no entryPoint" default.
//
// This header is the single source of truth shared by the producer
// (scripting_workspace, GM bake) and consumer (lua_gpu, GM helper).

// namespace rive
// namespace ore

// ---------------------------------------------------------------------------
// Encode (producer)
// ---------------------------------------------------------------------------

/// The source `std::string` is a byte string, rather than necessarily UTF-8.
/// Keeping its bytes in an owned wrapper preserves embedded NULs and all
/// source byte values while retaining the source field shape.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct RstbString(Vec<u8>);

impl RstbString {
    pub fn as_bytes(&self) -> &[u8] {
        &self.0
    }
}

impl From<&str> for RstbString {
    fn from(value: &str) -> Self {
        Self(value.as_bytes().to_vec())
    }
}

impl From<String> for RstbString {
    fn from(value: String) -> Self {
        Self(value.into_bytes())
    }
}

impl From<Vec<u8>> for RstbString {
    fn from(value: Vec<u8>) -> Self {
        Self(value)
    }
}

impl From<&[u8]> for RstbString {
    fn from(value: &[u8]) -> Self {
        Self(value.to_vec())
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct RstbEntry {
    pub stage: u8, // 0=vertex 1=fragment 2=compute
    pub logical: RstbString,
    pub physical: RstbString,
    pub source: Vec<u8>, // per-entry targets only
}

struct rstb_detail;

impl rstb_detail {
    fn putU16(b: &mut Vec<u8>, v: u16) {
        b.push((v & 0xFF) as u8);
        b.push(((v >> 8) & 0xFF) as u8);
    }

    fn putU32(b: &mut Vec<u8>, v: u32) {
        b.push((v & 0xFF) as u8);
        b.push(((v >> 8) & 0xFF) as u8);
        b.push(((v >> 16) & 0xFF) as u8);
        b.push(((v >> 24) & 0xFF) as u8);
    }

    fn putStr(b: &mut Vec<u8>, s: &RstbString) {
        // Preserve the authored static_cast<uint16_t>: the wire length wraps,
        // but the producer still appends every source byte.
        Self::putU16(b, s.as_bytes().len() as u16);
        b.extend_from_slice(s.as_bytes());
    }
}

pub fn buildWholeModuleContainer(
    entries: &[RstbEntry],
    src: Option<&[u8]>,
    srcLen: u32,
) -> Vec<u8> {
    // count is a u8. A module with >255 entry points (pathological) would
    // truncate the count and corrupt the container — fail cleanly with an empty
    // blob, which callers treat as "skip this variant".
    if entries.len() > 255 {
        return Vec::new();
    }
    let mut b = Vec::new();
    b.push(entries.len() as u8);
    for e in entries {
        b.push(e.stage);
        rstb_detail::putStr(&mut b, &e.logical);
        rstb_detail::putStr(&mut b, &e.physical);
    }
    rstb_detail::putU32(&mut b, srcLen);
    // Guard the raw-pointer range: src may be null when srcLen == 0, and
    // `src + 0` on a null pointer is undefined behavior (trips UBSan).
    if srcLen > 0 {
        let Some(src) = src else {
            return Vec::new();
        };
        let srcLen = srcLen as usize;
        let Some(src) = src.get(..srcLen) else {
            return Vec::new();
        };
        b.extend_from_slice(src);
    }
    b
}

pub fn buildPerEntryContainer(entries: &[RstbEntry]) -> Vec<u8> {
    if entries.len() > 255 {
        return Vec::new();
    }
    let mut b = Vec::new();
    b.push(entries.len() as u8);
    for e in entries {
        b.push(e.stage);
        rstb_detail::putStr(&mut b, &e.logical);
        rstb_detail::putStr(&mut b, &e.physical);
        // Preserve static_cast<uint32_t>(e.source.size()): only the encoded
        // wire length truncates; all bytes are appended below.
        rstb_detail::putU32(&mut b, e.source.len() as u32);
        b.extend_from_slice(&e.source);
    }
    b
}

// ---------------------------------------------------------------------------
// Decode (consumer). Bounded readers — an adversarial blob returns false, never
// reads out of bounds. String fields are copied; source is a span into `blob`.
// ---------------------------------------------------------------------------

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RstbEntryView<'a> {
    pub stage: u8,
    pub logical: RstbString,
    pub physical: RstbString,
    pub source: Option<&'a [u8]>, // per-entry targets; null for whole-module
    pub sourceSize: u32,
}

impl Default for RstbEntryView<'_> {
    fn default() -> Self {
        Self {
            stage: 0,
            logical: RstbString::default(),
            physical: RstbString::default(),
            source: None,
            sourceSize: 0,
        }
    }
}

impl rstb_detail {
    fn getU16(p: &mut &[u8], v: &mut u16) -> bool {
        if p.len() < 2 {
            return false;
        }
        *v = u16::from_le_bytes([p[0], p[1]]);
        *p = &p[2..];
        true
    }

    fn getU32(p: &mut &[u8], v: &mut u32) -> bool {
        if p.len() < 4 {
            return false;
        }
        *v = u32::from_le_bytes([p[0], p[1], p[2], p[3]]);
        *p = &p[4..];
        true
    }

    fn getStr(p: &mut &[u8], s: &mut RstbString) -> bool {
        let mut n = 0;
        if !Self::getU16(p, &mut n) {
            return false;
        }
        let n = usize::from(n);
        if p.len() < n {
            return false;
        }
        *s = RstbString::from(&p[..n]);
        *p = &p[n..];
        true
    }
}

pub fn parseWholeModuleContainer<'a>(
    blob: Option<&'a [u8]>,
    size: u32,
    out: &mut Vec<RstbEntryView<'a>>,
    src: Option<&mut Option<&'a [u8]>>,
    srcLen: Option<&mut u32>,
) -> bool {
    // Reject null/empty input before any pointer arithmetic (blob + size on a
    // null pointer is undefined behavior, even when size == 0).
    let (Some(blob), Some(src), Some(srcLen)) = (blob, src, srcLen) else {
        return false;
    };
    if size == 0 || blob.is_empty() {
        return false;
    }
    let size = size as usize;
    let Some(blob) = blob.get(..size) else {
        return false;
    };
    let mut p = blob;
    let count = p[0];
    p = &p[1..];
    out.clear();
    out.reserve(usize::from(count));
    for _ in 0..count {
        if p.is_empty() {
            return false;
        }
        let mut e = RstbEntryView::default();
        e.stage = p[0];
        p = &p[1..];
        if e.stage > 2 {
            // only 0=vertex, 1=fragment, 2=compute are defined
            return false;
        }
        if !rstb_detail::getStr(&mut p, &mut e.logical)
            || !rstb_detail::getStr(&mut p, &mut e.physical)
        {
            return false;
        }
        out.push(e);
    }
    let mut n = 0;
    if !rstb_detail::getU32(&mut p, &mut n) {
        return false;
    }
    let Some(source) = p.get(..n as usize) else {
        return false;
    };
    *src = Some(source);
    *srcLen = n;
    true
}

pub fn parsePerEntryContainer<'a>(
    blob: Option<&'a [u8]>,
    size: u32,
    out: &mut Vec<RstbEntryView<'a>>,
) -> bool {
    let Some(blob) = blob else {
        return false;
    };
    if size == 0 || blob.is_empty() {
        return false;
    }
    let size = size as usize;
    let Some(blob) = blob.get(..size) else {
        return false;
    };
    let mut p = blob;
    let count = p[0];
    p = &p[1..];
    out.clear();
    out.reserve(usize::from(count));
    for _ in 0..count {
        if p.is_empty() {
            return false;
        }
        let mut e = RstbEntryView::default();
        e.stage = p[0];
        p = &p[1..];
        if e.stage > 2 {
            // only 0=vertex, 1=fragment, 2=compute are defined
            return false;
        }
        if !rstb_detail::getStr(&mut p, &mut e.logical)
            || !rstb_detail::getStr(&mut p, &mut e.physical)
        {
            return false;
        }
        let mut n = 0;
        if !rstb_detail::getU32(&mut p, &mut n) {
            return false;
        }
        let Some(source) = p.get(..n as usize) else {
            return false;
        };
        e.source = Some(source);
        e.sourceSize = n;
        p = &p[n as usize..];
        out.push(e);
    }
    true
}

// namespace ore
// namespace rive
#[cfg(test)]
mod tests {
    use super::*;

    fn span_string(source: Option<&[u8]>, length: u32) -> String {
        let source = source.expect("source span");
        String::from_utf8(source[..length as usize].to_vec()).expect("UTF-8 test source")
    }

    #[test]
    fn rstb_whole_module_container_round_trip() {
        // Two vertex entries + one fragment entry; physical differs from
        // logical (the MSL-rename case).
        let entries = vec![
            RstbEntry {
                stage: 0,
                logical: "vertex".into(),
                physical: "vertex_1".into(),
                source: Vec::new(),
            },
            RstbEntry {
                stage: 0,
                logical: "instanced".into(),
                physical: "instanced".into(),
                source: Vec::new(),
            },
            RstbEntry {
                stage: 1,
                logical: "fragment".into(),
                physical: "fragment_1".into(),
                source: Vec::new(),
            },
        ];
        let source = b"// shared MSL/SPIRV/WGSL source\n";
        let blob = buildWholeModuleContainer(&entries, Some(source), source.len() as u32);

        let mut out = Vec::new();
        let mut out_source = None;
        let mut out_source_len = 0;
        assert!(parseWholeModuleContainer(
            Some(&blob),
            blob.len() as u32,
            &mut out,
            Some(&mut out_source),
            Some(&mut out_source_len),
        ));
        assert_eq!(out.len(), 3);
        assert_eq!(out[0].stage, 0);
        assert_eq!(out[0].logical.as_bytes(), b"vertex");
        assert_eq!(out[0].physical.as_bytes(), b"vertex_1");
        assert_eq!(out[1].logical.as_bytes(), b"instanced");
        assert_eq!(out[1].physical.as_bytes(), b"instanced");
        assert_eq!(out[2].stage, 1);
        assert_eq!(out[2].logical.as_bytes(), b"fragment");
        assert_eq!(out[2].physical.as_bytes(), b"fragment_1");
        assert_eq!(
            span_string(out_source, out_source_len),
            String::from_utf8_lossy(source).into_owned()
        );
        // Whole-module entries do not carry per-entry source.
        assert!(out[0].source.is_none());
    }

    #[test]
    fn rstb_per_entry_container_round_trip() {
        let entries = vec![
            RstbEntry {
                stage: 0,
                logical: "vs_main".into(),
                physical: "main".into(),
                source: b"void main() { /* vs */ }".to_vec(),
            },
            RstbEntry {
                stage: 1,
                logical: "fs_main".into(),
                physical: "main".into(),
                source: b"void main() { /* fs longer source */ }".to_vec(),
            },
        ];
        let blob = buildPerEntryContainer(&entries);
        let mut out = Vec::new();
        assert!(parsePerEntryContainer(
            Some(&blob),
            blob.len() as u32,
            &mut out,
        ));
        assert_eq!(out.len(), 2);
        assert_eq!(out[0].logical.as_bytes(), b"vs_main");
        assert_eq!(out[0].physical.as_bytes(), b"main");
        assert_eq!(
            span_string(out[0].source, out[0].sourceSize),
            "void main() { /* vs */ }"
        );
        assert_eq!(out[1].logical.as_bytes(), b"fs_main");
        assert_eq!(out[1].physical.as_bytes(), b"main");
        assert_eq!(
            span_string(out[1].source, out[1].sourceSize),
            "void main() { /* fs longer source */ }"
        );
    }

    #[test]
    fn rstb_container_empty_entries() {
        // Zero entries (e.g. a not-yet-populated module) must round-trip
        // cleanly.
        let blob = buildWholeModuleContainer(&[], Some(b"src"), 3);
        let mut out = Vec::new();
        let mut out_source = None;
        let mut out_source_len = 0;
        assert!(parseWholeModuleContainer(
            Some(&blob),
            blob.len() as u32,
            &mut out,
            Some(&mut out_source),
            Some(&mut out_source_len),
        ));
        assert!(out.is_empty());
        assert_eq!(span_string(out_source, out_source_len), "src");
    }

    #[test]
    fn rstb_container_rejects_truncation() {
        let entries = vec![RstbEntry {
            stage: 0,
            logical: "vs_main".into(),
            physical: "vs_main".into(),
            source: Vec::new(),
        }];
        let source = b"source bytes";
        let blob = buildWholeModuleContainer(&entries, Some(source), source.len() as u32);

        // Every truncated prefix must be rejected by the bounded readers,
        // never read out of bounds.
        for size in 0..blob.len() {
            let mut out = Vec::new();
            let mut out_source = None;
            let mut out_source_len = 0;
            assert!(!parseWholeModuleContainer(
                Some(&blob),
                size as u32,
                &mut out,
                Some(&mut out_source),
                Some(&mut out_source_len),
            ));
        }
        // The full blob parses.
        let mut out = Vec::new();
        let mut out_source = None;
        let mut out_source_len = 0;
        assert!(parseWholeModuleContainer(
            Some(&blob),
            blob.len() as u32,
            &mut out,
            Some(&mut out_source),
            Some(&mut out_source_len),
        ));
    }

    #[test]
    fn rstb_container_empty_source_accepts_null_source() {
        let blob = buildWholeModuleContainer(&[], None, 0);
        assert_eq!(blob, vec![0, 0, 0, 0, 0]);
        let mut out = Vec::new();
        let mut out_source = None;
        let mut out_source_len = 0;
        assert!(parseWholeModuleContainer(
            Some(&blob),
            blob.len() as u32,
            &mut out,
            Some(&mut out_source),
            Some(&mut out_source_len),
        ));
        assert_eq!(out_source_len, 0);
        assert_eq!(out_source.map(|source| source.len()), Some(0));
    }

    #[test]
    fn rstb_container_rejects_invalid_stage() {
        let blob = [1, 3];
        let mut out = Vec::new();
        assert!(!parsePerEntryContainer(
            Some(&blob),
            blob.len() as u32,
            &mut out
        ));
        assert!(out.is_empty());
    }

    #[test]
    fn rstb_container_malformed_parse_leaves_partial_output() {
        let entries = vec![
            RstbEntry {
                stage: 0,
                logical: "vs".into(),
                physical: "vs".into(),
                source: b"one".to_vec(),
            },
            RstbEntry {
                stage: 1,
                logical: "fs".into(),
                physical: "fs".into(),
                source: b"two".to_vec(),
            },
        ];
        let blob = buildPerEntryContainer(&entries);
        let mut out = Vec::new();
        assert!(!parsePerEntryContainer(
            Some(&blob[..blob.len() - 1]),
            (blob.len() - 1) as u32,
            &mut out,
        ));
        assert_eq!(out.len(), 1);
        assert_eq!(out[0].logical.as_bytes(), b"vs");
    }

    #[test]
    fn rstb_whole_module_malformed_parse_leaves_partial_output() {
        let entries = vec![
            RstbEntry {
                stage: 0,
                logical: "vs".into(),
                physical: "vs".into(),
                source: Vec::new(),
            },
            RstbEntry {
                stage: 1,
                logical: "fs".into(),
                physical: "fs".into(),
                source: Vec::new(),
            },
        ];
        let blob = buildWholeModuleContainer(&entries, Some(b"source"), 6);
        let mut out = Vec::new();
        let mut out_source = None;
        let mut out_source_len = 0;
        assert!(!parseWholeModuleContainer(
            Some(&blob[..blob.len() - 1]),
            (blob.len() - 1) as u32,
            &mut out,
            Some(&mut out_source),
            Some(&mut out_source_len),
        ));
        assert_eq!(out.len(), 2);
        assert!(out_source.is_none());
        assert_eq!(out_source_len, 0);
    }

    #[test]
    fn rstb_container_more_than_255_entries_returns_empty() {
        let entries = vec![RstbEntry::default(); 256];
        assert!(buildWholeModuleContainer(&entries, None, 0).is_empty());
        assert!(buildPerEntryContainer(&entries).is_empty());
    }

    #[test]
    fn rstb_container_accepts_non_utf8_names() {
        let entries = [RstbEntry {
            stage: 0,
            logical: vec![b'v', 0xff].into(),
            physical: vec![0x80, b'p'].into(),
            source: Vec::new(),
        }];
        let blob = buildPerEntryContainer(&entries);
        let mut out = Vec::new();
        assert!(parsePerEntryContainer(
            Some(&blob),
            blob.len() as u32,
            &mut out,
        ));
        assert_eq!(out[0].logical.as_bytes(), &[b'v', 0xff]);
        assert_eq!(out[0].physical.as_bytes(), &[0x80, b'p']);
    }

    #[test]
    fn rstb_whole_module_builder_fails_closed_for_invalid_source_span() {
        assert!(buildWholeModuleContainer(&[], None, 1).is_empty());
        assert!(buildWholeModuleContainer(&[], Some(b"x"), 2).is_empty());
    }

    #[test]
    fn rstb_container_empty_input_is_rejected() {
        let mut out = Vec::new();
        let mut out_source = None;
        let mut out_source_len = 0;
        assert!(!parseWholeModuleContainer(
            Some(&[]),
            0,
            &mut out,
            Some(&mut out_source),
            Some(&mut out_source_len),
        ));
        assert!(!parsePerEntryContainer(Some(&[]), 0, &mut out));
    }
}
