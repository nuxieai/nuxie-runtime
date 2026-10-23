//! RSTB section 2, translated from upstream 8920a6b.
#![allow(non_snake_case, non_upper_case_globals)]
use super::ore_rstb_entry_container_hpp::{RstbEntryView, RstbString};
pub const kRstbEntryBindingsTag: u8 = 2;
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct RstbEntryBindings {
    pub stage: u8,
    pub name: RstbString,
    pub bindings: Vec<u16>,
}
impl RstbEntryBindings {
    pub fn uses(&self, group: u32, binding: u32) -> bool {
        self.bindings.contains(&((group << 8 | binding) as u16))
    }
}
pub fn encodeEntryBindings(entries: &[RstbEntryBindings]) -> Vec<u8> {
    if entries.is_empty() || entries.len() > 255 {
        return Vec::new();
    }
    let mut out = vec![entries.len() as u8];
    for entry in entries {
        let name = entry.name.as_bytes();
        if entry.bindings.len() > 255 || name.len() > 65535 {
            return Vec::new();
        }
        out.push(entry.stage);
        out.extend_from_slice(&(name.len() as u16).to_le_bytes());
        out.extend_from_slice(name);
        out.push(entry.bindings.len() as u8);
        for binding in &entry.bindings {
            out.extend_from_slice(&binding.to_be_bytes());
        }
    }
    out
}
pub fn decodeEntryBindings(
    data: Option<&[u8]>,
    size: usize,
    out: &mut Vec<RstbEntryBindings>,
) -> bool {
    out.clear();
    let Some(data) = data.and_then(|bytes| bytes.get(..size)) else {
        return false;
    };
    let Some((&count, mut remaining)) = data.split_first() else {
        return false;
    };
    out.reserve(count as usize);
    for _ in 0..count {
        if remaining.len() < 3 {
            break;
        }
        let stage = remaining[0];
        let length = u16::from_le_bytes([remaining[1], remaining[2]]) as usize;
        remaining = &remaining[3..];
        if remaining.len() < length + 1 {
            break;
        }
        let name = remaining[..length].into();
        let n = remaining[length] as usize;
        remaining = &remaining[length + 1..];
        if remaining.len() < n * 2 {
            break;
        }
        let bindings = remaining[..n * 2]
            .chunks_exact(2)
            .map(|b| u16::from_be_bytes([b[0], b[1]]))
            .collect();
        remaining = &remaining[n * 2..];
        out.push(RstbEntryBindings {
            stage,
            name,
            bindings,
        });
    }
    if out.len() != count as usize {
        out.clear();
        return false;
    }
    true
}
/// ore_shader_asset_modules.hpp: rename complete logical metadata to physical names.
pub fn moduleEntryBindingBytes(
    asset: &[RstbEntryBindings],
    entries: &[RstbEntryView<'_>],
    first: u32,
    count: u32,
) -> Vec<u8> {
    let mut module = Vec::new();
    for entry in entries.iter().skip(first as usize).take(count as usize) {
        let Some(found) = asset
            .iter()
            .find(|e| e.stage == entry.stage && e.name == entry.logical)
        else {
            return Vec::new();
        };
        let mut named = found.clone();
        named.name = entry.physical.clone();
        module.push(named);
    }
    encodeEntryBindings(&module)
}
