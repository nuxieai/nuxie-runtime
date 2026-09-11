//! First-baseline measurement and ordinary-file constraints. Keep only scalar
//! summaries; retaining sibling Styles would duplicate large variable maps.
use super::{Diagnostic, Direction, Emitter, Size, Style, MAX_SIZE, unsupported};
use crate::wire::{Record, Value};

#[derive(Clone, Copy)]
enum Ascent { Fixed(f32), PercentFloor { percent: f32, floor: f32 } }
#[derive(Clone, Copy)]
pub(super) struct Metric { ascent: Ascent, descent: f32, origin: f32 }
pub(super) struct Child {
    pub object_id: u32,
    pub index: usize,
    pub order: i32,
    pub metric: Option<Metric>,
    pub participates: bool,
}

fn fixed_height(style: &Style) -> Option<f32> {
    let Size::Pixels(height) = style.height else { return None };
    let Size::Pixels(minimum) = style.min_height else { return None };
    let maximum = match style.max_height { Size::Auto => MAX_SIZE, Size::Pixels(v) => v, _ => return None };
    Some(height.min(maximum).max(minimum))
}

pub(super) fn summarize(style: &Style, children: &[Child]) -> Option<Metric> {
    if children.is_empty() {
        if let Some(height) = fixed_height(style) {
            return Some(Metric { ascent: Ascent::Fixed(height), descent: 0., origin: if height == 0. { 0. } else { 1. } });
        }
        if let (Size::Percent(percent), Size::Pixels(floor), Size::Auto) = (style.height, style.min_height, style.max_height) {
            return Some(Metric { ascent: Ascent::PercentFloor { percent, floor }, descent: 0., origin: 1. });
        }
        return None;
    }
    // In the admitted zero-padding/margin profile, the first order-modified
    // child of a column begins at y=0. Its baseline propagates through a fixed
    // box. Other topology and responsive nested anchors need separate proof.
    if !matches!(style.direction, Direction::Column) { return None; }
    let height = fixed_height(style)?;
    let first = children.iter().min_by_key(|child| (child.order, child.index))?;
    let Ascent::Fixed(ascent) = first.metric?.ascent else { return None };
    if ascent > height { return None; }
    Some(Metric { ascent: Ascent::Fixed(ascent), descent: height - ascent,
        origin: if height == 0. { 0. } else { ascent / height } })
}

pub(super) fn emit(emitter: &mut Emitter, parent_id: u32, parent: &Style, children: &[Child], source: &str) -> Result<(), Diagnostic> {
    if !children.iter().any(|child| child.participates) { return Ok(()); }
    if !parent.direction.is_row() {
        return Err(unsupported(source, "First baseline in column containers requires further ordinary-file validation; current lowering admits row and row-reverse"));
    }
    let mut percent = 0.0_f32;
    let mut floor = 0.0_f32;
    let mut descent = 0.0_f32;
    let mut responsive = false;
    for child in children.iter().filter(|child| child.participates) {
        let metric = child.metric.ok_or_else(|| unsupported(source, "Unresolved first-baseline metric"))?;
        match metric.ascent {
            Ascent::Fixed(value) => floor = floor.max(value),
            Ascent::PercentFloor { percent: value, floor: minimum } => {
                percent = percent.max(value); floor = floor.max(minimum); responsive |= value != 0.;
            }
        }
        descent = descent.max(metric.descent);
    }
    if responsive && descent != 0. {
        return Err(unsupported(source, "Responsive baseline ascent plus nonzero nested descent requires an additional ordinary-file expression composition"));
    }
    let extent = floor + descent;
    if !extent.is_finite() || extent > MAX_SIZE {
        return Err(unsupported(source, "Computed baseline measurement extent exceeds the finite 1000000px compiler limit"));
    }
    let (height, minimum, origin) = if responsive {
        (Size::Percent(percent), Size::Pixels(floor), 1.)
    } else {
        (Size::Pixels(extent), Size::Pixels(0.), if extent == 0. { 0. } else { floor / extent })
    };
    // The zero-width participant adds the full ascent+descent to automatic
    // parent sizing while preserving authored horizontal slots. Percentage
    // heights keep the original parent as their containing block.
    let helper = emitter.layout_box("", parent_id, Direction::Column, 0, parent.direction,
        [Size::Pixels(0.), height], [Size::Pixels(0.), minimum, Size::Auto, Size::Auto], false, [false; 4])?;
    let landmark = emitter.records.len() as u32 - 1;
    let mut node = Record::new("Node"); node.set("parentId", Value::Uint(0))?; emitter.records.push(node);
    let mut target = Record::new("TransformConstraint");
    target.set("parentId", Value::Uint(landmark))?; target.set("targetId", Value::Uint(helper))?;
    target.set("originY", Value::Float(origin))?; emitter.records.push(target);
    for child in children.iter().filter(|child| child.participates) {
        let metric = child.metric.expect("participant checked above");
        let mut anchor = Record::new("ComponentOrigin");
        anchor.set("parentId", Value::Uint(child.object_id))?;
        anchor.set("originX", Value::Float(0.))?; anchor.set("originY", Value::Float(metric.origin))?;
        emitter.records.push(anchor);
        let mut follow = Record::new("TranslationConstraint");
        follow.set("parentId", Value::Uint(child.object_id))?; follow.set("targetId", Value::Uint(landmark))?;
        follow.set("doesCopy", Value::Bool(false))?; follow.set("doesCopyY", Value::Bool(true))?;
        emitter.records.push(follow);
    }
    Ok(())
}
