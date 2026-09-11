//! Flexible ordinary layout participants distribute positive main-axis space.
//! They never enter the authored DOM, source map, or baseline child summaries.
use super::{Diagnostic, Direction, Emitter, Size, Style, unsupported};
use crate::wire::Value;

#[derive(Clone, Copy)]
pub(super) enum Spacing { Normal, FlexStart, Around, Evenly }
impl Spacing {
    pub fn distributes(self) -> bool { matches!(self, Self::Around | Self::Evenly) }
    pub fn alignment(self, direction: Direction) -> u32 {
        // A collapsed flexible spacer must leave overflow at physical start,
        // including reverse flows (CSS safe-center distribution fallback).
        if self.distributes() { if direction.is_row() { 2 } else { 6 } }
        else { direction.alignment() }
    }
}
pub(super) fn computed(text: &str, parent: Spacing, source: &str) -> Result<Spacing, Diagnostic> {
    match text.trim().to_ascii_lowercase().as_str() {
        "normal" | "initial" | "unset" => Ok(Spacing::Normal),
        "flex-start" => Ok(Spacing::FlexStart),
        "space-around" => Ok(Spacing::Around),
        "space-evenly" => Ok(Spacing::Evenly),
        "inherit" => Ok(parent),
        _ => Err(unsupported(source, "justify-content admits normal, flex-start, space-around, space-evenly, inherit, initial and unset; other distributions require further ordinary-file validation")),
    }
}
pub(super) fn emit(emitter: &mut Emitter, parent_id: u32, parent: &Style, edge: bool) -> Result<(), Diagnostic> {
    if !parent.spacing.distributes() { return Ok(()); }
    // N authored children require N+1 spacers, each exactly two records. The
    // global 8192-element limit bounds this by 32768 additional records.
    let id = emitter.layout_box("", parent_id, Direction::Column, 0, parent.direction,
        [Size::Pixels(0.); 2], [Size::Pixels(0.), Size::Pixels(0.), Size::Auto, Size::Auto], false, [false; 4])?;
    let row = parent.direction.is_row();
    let weight = if edge && matches!(parent.spacing, Spacing::Around) { 0.5 } else { 1. };
    emitter.records[id as usize + 1].set(if row { "fractionalWidth" } else { "fractionalHeight" }, Value::Float(weight))?;
    emitter.records[id as usize + 2].set(if row { "layoutWidthScaleType" } else { "layoutHeightScaleType" }, Value::Uint(1))?;
    Ok(())
}
