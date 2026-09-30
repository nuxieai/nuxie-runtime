//! Shared literal port of upstream tests/include/riv_bytes.hpp.
#![allow(dead_code)]
use nuxie_runtime::{
    File,
    source::generated::{
        artboard_base::ArtboardBase, backboard_base::BackboardBase, component_base::ComponentBase,
        layout_component_base::LayoutComponentBase,
    },
};
use std::collections::BTreeMap;

#[derive(Default)]
pub struct RivBytes {
    objects: Vec<u8>,
    toc: BTreeMap<u16, u32>,
}
fn uint(out: &mut Vec<u8>, mut value: u64) {
    while value >= 128 {
        out.push((value as u8 & 127) | 128);
        value >>= 7;
    }
    out.push(value as u8);
}
impl RivBytes {
    pub fn object(&mut self, key: u16) {
        uint(&mut self.objects, key.into());
    }
    pub fn end(&mut self) {
        uint(&mut self.objects, 0);
    }
    pub fn prop_uint(&mut self, key: u16, value: u64) {
        self.toc.insert(key, 0);
        self.prop_uint_outside_toc(key, value);
    }
    pub fn prop_uint_outside_toc(&mut self, key: u16, value: u64) {
        uint(&mut self.objects, key.into());
        uint(&mut self.objects, value);
    }
    pub fn prop_bool(&mut self, key: u16, value: bool) {
        self.toc.insert(key, 0);
        uint(&mut self.objects, key.into());
        self.objects.push(u8::from(value));
    }
    pub fn prop_string(&mut self, key: u16, value: &str) {
        self.toc.insert(key, 1);
        uint(&mut self.objects, key.into());
        uint(&mut self.objects, value.len() as u64);
        self.objects.extend_from_slice(value.as_bytes());
    }
    pub fn prop_float(&mut self, key: u16, value: f32) {
        self.toc.insert(key, 2);
        uint(&mut self.objects, key.into());
        self.objects.extend_from_slice(&value.to_le_bytes());
    }
    pub fn bytes(&self) -> Vec<u8> {
        let mut out = b"RIVE".to_vec();
        uint(&mut out, File::MAJOR_VERSION as u64);
        uint(&mut out, File::MINOR_VERSION as u64);
        uint(&mut out, 0);
        for &key in self.toc.keys() {
            uint(&mut out, key.into());
        }
        uint(&mut out, 0);
        let mut packed = 0u32;
        let mut bit = 0;
        for &field in self.toc.values() {
            packed |= field << bit;
            bit += 2;
            if bit == 8 {
                out.extend_from_slice(&packed.to_le_bytes());
                packed = 0;
                bit = 0;
            }
        }
        if bit != 0 {
            out.extend_from_slice(&packed.to_le_bytes());
        }
        out.extend_from_slice(&self.objects);
        out
    }
}
pub fn write_artboard(riv: &mut RivBytes) {
    riv.object(BackboardBase::TYPE_KEY);
    riv.end();
    write_artboard_object(riv);
}
pub fn write_artboard_object(riv: &mut RivBytes) {
    riv.object(ArtboardBase::TYPE_KEY);
    riv.prop_string(ComponentBase::NAME_PROPERTY_KEY, "A");
    riv.prop_float(LayoutComponentBase::WIDTH_PROPERTY_KEY, 100.0);
    riv.prop_float(LayoutComponentBase::HEIGHT_PROPERTY_KEY, 100.0);
    riv.end();
}
