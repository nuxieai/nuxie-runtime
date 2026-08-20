//! RSTB v4 source-variant entry containers.
//!
//! Upstream: `rive-app/rive-runtime`
//! Revision: `4ac7b32798da0482e441ef09304dc3b480ed3ee5`
//! Source: `renderer/include/rive/renderer/ore/ore_rstb_entry_container.hpp`
//!
//! This module is a mechanical translation of the pinned upstream header.
//! Whole-module targets (WGSL=0, MSL=2, SPIR-V=5) share one source, while
//! per-entry targets (GLSL=1, HLSL=3) carry a source per entry. Strings are
//! encoded as a little-endian u16 byte length followed by bytes without a
//! NUL terminator.

// RSTB v4 source-variant containers. Every source variant (WGSL/GLSL/MSL/
// HLSL/SPIR-V) carries a self-describing entry table so the runtime resolves
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

/// Owned bytes corresponding to C++ `std::string`.
///
/// RSTB strings are length-prefixed byte strings. They are commonly UTF-8,
/// but the upstream representation and parser accept arbitrary bytes.
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

/// One owned entry record used by the RSTB producer.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct RstbEntry {
    pub stage: u8, // 0=vertex 1=fragment 2=compute
    pub logical: RstbString,
    pub physical: RstbString,
    pub source: Vec<u8>, // per-entry targets only
}

mod rstb_detail {
    use super::RstbString;

    pub(super) fn put_u16(buffer: &mut Vec<u8>, value: u16) {
        buffer.push((value & 0xFF) as u8);
        buffer.push((value >> 8) as u8);
    }

    pub(super) fn put_u32(buffer: &mut Vec<u8>, value: u32) {
        buffer.push((value & 0xFF) as u8);
        buffer.push(((value >> 8) & 0xFF) as u8);
        buffer.push(((value >> 16) & 0xFF) as u8);
        buffer.push((value >> 24) as u8);
    }

    pub(super) fn put_str(buffer: &mut Vec<u8>, value: &RstbString) {
        // This cast intentionally mirrors static_cast<uint16_t>(s.size()) in
        // the producer. The source bytes are still appended in full.
        put_u16(buffer, value.as_bytes().len() as u16);
        buffer.extend_from_slice(value.as_bytes());
    }

    pub(super) fn get_u16(input: &mut &[u8], value: &mut u16) -> bool {
        if input.len() < 2 {
            return false;
        }
        *value = u16::from_le_bytes([input[0], input[1]]);
        *input = &input[2..];
        true
    }

    pub(super) fn get_u32(input: &mut &[u8], value: &mut u32) -> bool {
        if input.len() < 4 {
            return false;
        }
        *value = u32::from_le_bytes([input[0], input[1], input[2], input[3]]);
        *input = &input[4..];
        true
    }

    pub(super) fn get_str(input: &mut &[u8], value: &mut RstbString) -> bool {
        let mut length = 0;
        if !get_u16(input, &mut length) {
            return false;
        }
        let length = usize::from(length);
        if input.len() < length {
            return false;
        }
        *value = input[..length].to_vec().into();
        *input = &input[length..];
        true
    }
}

/// Build a whole-module container.
///
/// `src` is the nullable Rust representation of the upstream raw pointer:
/// `None` is valid when `src_len == 0`. For a non-zero length, the caller must
/// provide a slice containing at least `src_len` bytes, matching the upstream
/// producer's pointer-range invariant; invalid spans fail closed with an empty
/// container.
pub fn build_whole_module_container(
    entries: &[RstbEntry],
    src: Option<&[u8]>,
    src_len: u32,
) -> Vec<u8> {
    // count is a u8. A module with >255 entry points (pathological) would
    // truncate the count and corrupt the container — fail cleanly with an
    // empty blob, which callers treat as "skip this variant".
    if entries.len() > 255 {
        return Vec::new();
    }

    let source_len = src_len as usize;
    let source = if src_len == 0 {
        None
    } else if let Some(source) = src.and_then(|source| source.get(..source_len)) {
        Some(source)
    } else {
        return Vec::new();
    };

    let mut buffer = Vec::new();
    buffer.push(entries.len() as u8);
    for entry in entries {
        buffer.push(entry.stage);
        rstb_detail::put_str(&mut buffer, &entry.logical);
        rstb_detail::put_str(&mut buffer, &entry.physical);
    }
    rstb_detail::put_u32(&mut buffer, src_len);
    // The upstream guard permits a null source when srcLen == 0. Rust's
    // borrowed slice makes the non-zero pointer-range precondition explicit.
    if let Some(source) = source {
        buffer.extend_from_slice(source);
    }
    buffer
}

/// Build a per-entry container.
pub fn build_per_entry_container(entries: &[RstbEntry]) -> Vec<u8> {
    if entries.len() > 255 {
        return Vec::new();
    }

    let mut buffer = Vec::new();
    buffer.push(entries.len() as u8);
    for entry in entries {
        buffer.push(entry.stage);
        rstb_detail::put_str(&mut buffer, &entry.logical);
        rstb_detail::put_str(&mut buffer, &entry.physical);
        rstb_detail::put_u32(&mut buffer, entry.source.len() as u32);
        buffer.extend_from_slice(&entry.source);
    }
    buffer
}

/// One decoded RSTB entry. The strings are copied; `source` borrows the input
/// blob and cannot outlive it. Whole-module entries have no per-entry source.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RstbEntryView<'a> {
    pub stage: u8,
    pub logical: RstbString,
    pub physical: RstbString,
    pub source: Option<&'a [u8]>, // per-entry targets; None for whole-module
    pub source_size: u32,
}

impl Default for RstbEntryView<'_> {
    fn default() -> Self {
        Self {
            stage: 0,
            logical: RstbString::default(),
            physical: RstbString::default(),
            source: None,
            source_size: 0,
        }
    }
}

/// Parse a whole-module container.
///
/// The output vector is cleared only after the null/empty input checks, then
/// receives each complete entry in order. Consequently malformed input may
/// leave a successfully decoded prefix in `out`, matching the upstream
/// parser. `source` is assigned only after the complete container validates.
pub fn parse_whole_module_container<'a>(
    blob: &'a [u8],
    size: u32,
    out: &mut Vec<RstbEntryView<'a>>,
    source: &mut Option<&'a [u8]>,
    source_len: &mut u32,
) -> bool {
    // Reject empty input before taking any range. A Rust slice is never null;
    // an empty slice is the equivalent observable rejection for size == 0.
    if blob.is_empty() || size == 0 {
        return false;
    }
    let size = size as usize;
    if size > blob.len() {
        return false;
    }
    let mut input = &blob[..size];
    let count = input[0];
    input = &input[1..];
    out.clear();
    out.reserve(usize::from(count));
    for _ in 0..count {
        if input.is_empty() {
            return false;
        }
        let stage = input[0];
        input = &input[1..];
        if stage > 2 {
            return false;
        }
        let mut entry = RstbEntryView {
            stage,
            ..RstbEntryView::default()
        };
        if !rstb_detail::get_str(&mut input, &mut entry.logical)
            || !rstb_detail::get_str(&mut input, &mut entry.physical)
        {
            return false;
        }
        out.push(entry);
    }
    let mut length = 0;
    if !rstb_detail::get_u32(&mut input, &mut length) {
        return false;
    }
    let source_size = length as usize;
    if input.len() < source_size {
        return false;
    }
    *source = Some(&input[..source_size]);
    *source_len = length;
    true
}

/// Parse a per-entry container.
///
/// As in the upstream parser, `out` is cleared after the initial input check
/// and can contain a decoded prefix when a later entry is malformed. Each
/// source is a borrowed span into `blob`, including a non-null empty span for
/// a zero-length source.
pub fn parse_per_entry_container<'a>(
    blob: &'a [u8],
    size: u32,
    out: &mut Vec<RstbEntryView<'a>>,
) -> bool {
    if blob.is_empty() || size == 0 {
        return false;
    }
    let size = size as usize;
    if size > blob.len() {
        return false;
    }
    let mut input = &blob[..size];
    let count = input[0];
    input = &input[1..];
    out.clear();
    out.reserve(usize::from(count));
    for _ in 0..count {
        if input.is_empty() {
            return false;
        }
        let stage = input[0];
        input = &input[1..];
        if stage > 2 {
            return false;
        }
        let mut entry = RstbEntryView {
            stage,
            ..RstbEntryView::default()
        };
        if !rstb_detail::get_str(&mut input, &mut entry.logical)
            || !rstb_detail::get_str(&mut input, &mut entry.physical)
        {
            return false;
        }
        let mut length = 0;
        if !rstb_detail::get_u32(&mut input, &mut length) {
            return false;
        }
        let source_size = length as usize;
        if input.len() < source_size {
            return false;
        }
        entry.source = Some(&input[..source_size]);
        entry.source_size = length;
        input = &input[source_size..];
        out.push(entry);
    }
    true
}

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
        let blob = build_whole_module_container(&entries, Some(source), source.len() as u32);

        let mut out = Vec::new();
        let mut out_source = None;
        let mut out_source_len = 0;
        assert!(parse_whole_module_container(
            &blob,
            blob.len() as u32,
            &mut out,
            &mut out_source,
            &mut out_source_len,
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
        let blob = build_per_entry_container(&entries);
        let mut out = Vec::new();
        assert!(parse_per_entry_container(
            &blob,
            blob.len() as u32,
            &mut out,
        ));
        assert_eq!(out.len(), 2);
        assert_eq!(out[0].logical.as_bytes(), b"vs_main");
        assert_eq!(out[0].physical.as_bytes(), b"main");
        assert_eq!(
            span_string(out[0].source, out[0].source_size),
            "void main() { /* vs */ }"
        );
        assert_eq!(out[1].logical.as_bytes(), b"fs_main");
        assert_eq!(out[1].physical.as_bytes(), b"main");
        assert_eq!(
            span_string(out[1].source, out[1].source_size),
            "void main() { /* fs longer source */ }"
        );
    }

    #[test]
    fn rstb_container_empty_entries() {
        // Zero entries (e.g. a not-yet-populated module) must round-trip
        // cleanly.
        let blob = build_whole_module_container(&[], Some(b"src"), 3);
        let mut out = Vec::new();
        let mut out_source = None;
        let mut out_source_len = 0;
        assert!(parse_whole_module_container(
            &blob,
            blob.len() as u32,
            &mut out,
            &mut out_source,
            &mut out_source_len,
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
        let blob = build_whole_module_container(&entries, Some(source), source.len() as u32);

        // Every truncated prefix must be rejected by the bounded readers,
        // never read out of bounds.
        for size in 0..blob.len() {
            let mut out = Vec::new();
            let mut out_source = None;
            let mut out_source_len = 0;
            assert!(!parse_whole_module_container(
                &blob,
                size as u32,
                &mut out,
                &mut out_source,
                &mut out_source_len,
            ));
        }
        // The full blob parses.
        let mut out = Vec::new();
        let mut out_source = None;
        let mut out_source_len = 0;
        assert!(parse_whole_module_container(
            &blob,
            blob.len() as u32,
            &mut out,
            &mut out_source,
            &mut out_source_len,
        ));
    }

    #[test]
    fn rstb_container_empty_source_accepts_null_source() {
        let blob = build_whole_module_container(&[], None, 0);
        assert_eq!(blob, vec![0, 0, 0, 0, 0]);
        let mut out = Vec::new();
        let mut out_source = None;
        let mut out_source_len = 0;
        assert!(parse_whole_module_container(
            &blob,
            blob.len() as u32,
            &mut out,
            &mut out_source,
            &mut out_source_len,
        ));
        assert_eq!(out_source_len, 0);
        assert_eq!(out_source.map(|source| source.len()), Some(0));
    }

    #[test]
    fn rstb_container_rejects_invalid_stage() {
        let blob = [1, 3];
        let mut out = Vec::new();
        assert!(!parse_per_entry_container(
            &blob,
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
        let blob = build_per_entry_container(&entries);
        let mut out = Vec::new();
        assert!(!parse_per_entry_container(
            &blob[..blob.len() - 1],
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
        let blob = build_whole_module_container(&entries, Some(b"source"), 6);
        let mut out = Vec::new();
        let mut out_source = None;
        let mut out_source_len = 0;
        assert!(!parse_whole_module_container(
            &blob[..blob.len() - 1],
            (blob.len() - 1) as u32,
            &mut out,
            &mut out_source,
            &mut out_source_len,
        ));
        assert_eq!(out.len(), 2);
        assert!(out_source.is_none());
        assert_eq!(out_source_len, 0);
    }

    #[test]
    fn rstb_container_more_than_255_entries_returns_empty() {
        let entries = vec![RstbEntry::default(); 256];
        assert!(build_whole_module_container(&entries, None, 0).is_empty());
        assert!(build_per_entry_container(&entries).is_empty());
    }

    #[test]
    fn rstb_container_accepts_non_utf8_names() {
        let entries = [RstbEntry {
            stage: 0,
            logical: vec![b'v', 0xff].into(),
            physical: vec![0x80, b'p'].into(),
            source: Vec::new(),
        }];
        let blob = build_per_entry_container(&entries);
        let mut out = Vec::new();
        assert!(parse_per_entry_container(
            &blob,
            blob.len() as u32,
            &mut out,
        ));
        assert_eq!(out[0].logical.as_bytes(), &[b'v', 0xff]);
        assert_eq!(out[0].physical.as_bytes(), &[0x80, b'p']);
    }

    #[test]
    fn rstb_whole_module_builder_fails_closed_for_invalid_source_span() {
        assert!(build_whole_module_container(&[], None, 1).is_empty());
        assert!(build_whole_module_container(&[], Some(b"x"), 2).is_empty());
    }

    #[test]
    fn rstb_container_empty_input_is_rejected() {
        let mut out = Vec::new();
        let mut out_source = None;
        let mut out_source_len = 0;
        assert!(!parse_whole_module_container(
            &[],
            0,
            &mut out,
            &mut out_source,
            &mut out_source_len,
        ));
        assert!(!parse_per_entry_container(&[], 0, &mut out));
    }
}
