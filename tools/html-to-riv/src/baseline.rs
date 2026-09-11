//! First/last-baseline measurement and ordinary-file constraints. Keep only scalar
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
    pub last_metric: Option<Metric>,
    pub used_height: Option<f32>,
    pub participates: bool,
    pub last: bool,
}

fn bounded_height(style: &Style, intrinsic: Option<f32>) -> Option<f32> {
    let height = match style.height { Size::Pixels(height) => height, Size::Auto => intrinsic?, _ => return None };
    let Size::Pixels(minimum) = style.min_height else { return None };
    let maximum = match style.max_height { Size::Auto => MAX_SIZE, Size::Pixels(v) => v, _ => return None };
    Some(height.min(maximum).max(minimum))
}

pub(super) fn summarize(style: &Style, children: &[Child]) -> Option<Metric> {
    if children.is_empty() {
        if let Some(height) = bounded_height(style, Some(0.)) {
            return Some(Metric { ascent: Ascent::Fixed(height), descent: 0., origin: if height == 0. { 0. } else { 1. } });
        }
        if let (Size::Percent(percent), Size::Pixels(floor), Size::Auto) = (style.height, style.min_height, style.max_height) {
            return Some(Metric { ascent: Ascent::PercentFloor { percent, floor }, descent: 0., origin: 1. });
        }
        return None;
    }
    // In the admitted zero-padding/margin profile, the first order-modified
    // child of a column begins at y=0. Its baseline propagates through a box
    // with fixed or provable intrinsic height. Other topology and responsive
    // nested anchors need separate proof.
    if !matches!(style.direction, Direction::Column) || style.spacing.distributes() { return None; }
    let intrinsic = if matches!(style.height, Size::Auto) {
        let mut sum = 0.0_f32;
        for child in children {
            let metric = child.metric?;
            let Ascent::Fixed(ascent) = metric.ascent else { return None };
            sum += ascent + metric.descent;
            // Guard the intrinsic expression before clamping: it must remain
            // finite and bounded even when an authored maximum is smaller.
            if !sum.is_finite() || sum > MAX_SIZE { return None; }
        }
        Some(sum)
    } else { None };
    let height = bounded_height(style, intrinsic)?;
    let first = children.iter().min_by_key(|child| (child.order, child.index))?;
    let Ascent::Fixed(ascent) = first.metric?.ascent else { return None };
    if ascent > height { return None; }
    Some(Metric { ascent: Ascent::Fixed(ascent), descent: height - ascent,
        origin: if height == 0. { 0. } else { ascent / height } })
}

// Sort references only: no Style or variable environment is retained. Summing
// in physical column order matches layout's sequence of floating additions.
fn ordered(children: &[Child]) -> Vec<&Child> {
    let mut ordered: Vec<_> = children.iter().collect();
    ordered.sort_by_key(|child| (child.order, child.index));
    ordered
}

// Height is independent of whether a descendant has a usable baseline. A
// fixed preceding sibling can contribute its height even when its own
// baseline topology remains unresolved.
pub(super) fn used_height(style: &Style, children: &[Child]) -> Option<f32> {
    let intrinsic = if children.is_empty() { Some(0.) }
    else if matches!(style.direction, Direction::Column) {
        ordered(children).into_iter().try_fold(0.0_f32, |sum, child| {
            let sum = sum + child.used_height?;
            (sum.is_finite() && sum <= MAX_SIZE).then_some(sum)
        })
    } else { None };
    bounded_height(style, intrinsic)
}

pub(super) fn summarize_last(style: &Style, children: &[Child], height: Option<f32>) -> Option<Metric> {
    let height = height?;
    let ascent = if children.is_empty() { height } else {
        if !matches!(style.direction, Direction::Column) || style.spacing.distributes() { return None; }
        let last = children.iter().max_by_key(|child| (child.order, child.index))?;
        let Ascent::Fixed(last_ascent) = last.last_metric?.ascent else { return None };
        let preceding = ordered(children).into_iter().filter(|child| child.index != last.index).try_fold(0.0_f32, |sum, child| {
            let sum = sum + child.used_height?;
            (sum.is_finite() && sum <= MAX_SIZE).then_some(sum)
        })?;
        preceding + last_ascent
    };
    if !ascent.is_finite() || ascent > MAX_SIZE || ascent > height { return None; }
    Some(Metric { ascent: Ascent::Fixed(ascent), descent: height - ascent,
        origin: if height == 0. { 0. } else { ascent / height } })
}

pub(super) fn emit(emitter: &mut Emitter, parent_id: u32, parent: &Style, children: &[Child], source: &str) -> Result<(), Diagnostic> {
    if !children.iter().any(|child| child.participates) { return Ok(()); }
    if !parent.direction.is_row() {
        return Err(unsupported(source, "Baseline alignment in column containers requires further ordinary-file validation; current lowering admits row and row-reverse"));
    }
    emit_group(emitter, parent_id, parent, children, source, false)?;
    emit_group(emitter, parent_id, parent, children, source, true)
}

fn emit_group(emitter: &mut Emitter, parent_id: u32, parent: &Style, children: &[Child], source: &str, last: bool) -> Result<(), Diagnostic> {
    let participants = || children.iter().filter(|child| child.participates && child.last == last);
    if participants().next().is_none() { return Ok(()); }
    let metric_of = |child: &Child| if last { child.last_metric } else { child.metric };
    let mut percent = 0.0_f32;
    let mut floor = 0.0_f32;
    let mut descent = 0.0_f32;
    let mut responsive = false;
    for child in participants() {
        let metric = metric_of(child).ok_or_else(|| unsupported(source, "Unresolved baseline metric"))?;
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
    let mut landmark = emitter.records.len() as u32 - 1;
    let mut node = Record::new("Node"); node.set("parentId", Value::Uint(0))?; emitter.records.push(node);
    let mut target = Record::new("TransformConstraint");
    target.set("parentId", Value::Uint(landmark))?; target.set("targetId", Value::Uint(if last { parent_id } else { helper }))?;
    target.set("originY", Value::Float(if last { 1. } else { origin }))?; emitter.records.push(target);
    if last {
        // The parent-bottom landmark minus the group's descent preserves
        // unsafe end overflow, including a group taller than a fixed parent.
        let bottom = landmark;
        landmark = emitter.records.len() as u32 - 1;
        let mut node = Record::new("Node"); node.set("parentId", Value::Uint(0))?;
        node.set("y", Value::Float(-descent))?; emitter.records.push(node);
        let mut offset = Record::new("TranslationConstraint");
        offset.set("parentId", Value::Uint(landmark))?; offset.set("targetId", Value::Uint(bottom))?;
        offset.set("doesCopy", Value::Bool(false))?; offset.set("doesCopyY", Value::Bool(true))?;
        offset.set("offset", Value::Bool(true))?; emitter.records.push(offset);
    }
    for child in participants() {
        let metric = metric_of(child).expect("participant checked above");
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
