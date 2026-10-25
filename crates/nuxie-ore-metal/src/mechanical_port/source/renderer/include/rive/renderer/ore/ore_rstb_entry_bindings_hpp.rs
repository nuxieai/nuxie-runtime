//! RSTB section 2, translated from upstream e406d1f4.
#![allow(non_snake_case, non_upper_case_globals)]
use super::ore_binding_map_hpp::TextureSamplerPair;
use super::ore_rstb_entry_container_hpp::{RstbEntryView, RstbString};
pub const kRstbEntryBindingsTag: u8 = 2;
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct RstbEntryBindings {
    pub stage: u8,
    pub name: RstbString,
    pub bindings: Vec<u16>,
    pub pairs: Vec<TextureSamplerPair>,
    pub pairsKnown: bool,
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
    if entries.iter().any(|entry| !entry.pairsKnown || entry.pairs.len() > 255) {
        return out;
    }
    let bindings_end = out.len();
    out.push(entries.len() as u8);
    for entry in entries {
        out.push(entry.pairs.len() as u8);
        for pair in &entry.pairs {
            out.extend_from_slice(&[
                pair.textureGroup, pair.textureBinding, pair.samplerGroup, pair.samplerBinding,
            ]);
        }
    }
    if out.len() > 0xffff {
        out.truncate(bindings_end);
    }
    out
}
/// Decode the optional per-entry pairs as an all-or-none extension.
pub fn decodeEntryPairs(data: &[u8], entries: &mut [RstbEntryBindings]) {
    let Some((&count, mut remaining)) = data.split_first() else { return };
    if usize::from(count) != entries.len() {
        return;
    }
    let mut pairs = Vec::with_capacity(entries.len());
    for _ in 0..entries.len() {
        let Some((&count, rest)) = remaining.split_first() else { return };
        let size = usize::from(count) * 4;
        if rest.len() < size {
            return;
        }
        pairs.push(rest[..size].chunks_exact(4).map(|p| TextureSamplerPair {
            textureGroup: p[0], textureBinding: p[1], samplerGroup: p[2], samplerBinding: p[3],
        }).collect());
        remaining = &rest[size..];
    }
    for (entry, pairs) in entries.iter_mut().zip(pairs) {
        entry.pairs = pairs;
        entry.pairsKnown = true;
    }
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
            ..RstbEntryBindings::default()
        });
    }
    if out.len() != count as usize {
        out.clear();
        return false;
    }
    decodeEntryPairs(remaining, out);
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
