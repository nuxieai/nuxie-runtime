//! Runtime half of upstream shapes/paint/paint_outset.{hpp,cpp}.
use crate::mechanical_port::source::{
    core::{CoreHandle, CoreObject},
    math::{mat2d::Mat2D, vec2d::Vec2D},
    shapes::{
        paint::{feather::Feather, stroke::Stroke, stroke_cap::StrokeCap, stroke_join::StrokeJoin},
        shape_paint_container::ShapePaintContainer,
    },
    transform_space::TransformSpace,
};
pub use nuxie_render_api::paint_outset::*;

pub struct ShapePaintOutset {
    pub outset: f32,
    pub feather_offset: Vec2D,
    pub feather_offset_in_world: bool,
    pub path_is_local: bool,
    pub trustworthy: bool,
}
impl Default for ShapePaintOutset {
    fn default() -> Self {
        Self {
            outset: 0.0,
            feather_offset: Vec2D::new(0.0, 0.0),
            feather_offset_in_world: false,
            path_is_local: true,
            trustworthy: true,
        }
    }
}
pub struct PaintReach {
    pub world_outset: f32,
    pub trustworthy: bool,
}
impl Default for PaintReach {
    fn default() -> Self {
        Self {
            world_outset: 0.0,
            trustworthy: true,
        }
    }
}

pub fn shape_paint_outset(paint: Option<&dyn CoreObject>) -> ShapePaintOutset {
    shape_paint_outset_drawn(paint, false)
}
/// Nuxie addition: as [`shape_paint_outset`], for a Stroke a CSS BoxBorder
/// draws as a filled ring when `stroke_fills` (docs/CSS_LAYOUT.md, 4.2):
/// its thickness, and the visibility a thickness gives a Stroke, do not
/// apply.
pub fn shape_paint_outset_drawn(
    paint: Option<&dyn CoreObject>,
    stroke_fills: bool,
) -> ShapePaintOutset {
    let mut out = ShapePaintOutset::default();
    let Some(paint) = paint else { return out };
    let Some(behavior) = paint.as_shape_paint_behavior() else {
        return out;
    };
    let draws = if stroke_fills {
        behavior.shape_paint().base.is_visible() && behavior.shape_paint().mutator_is_visible()
    } else {
        behavior.should_draw()
    };
    if !draws {
        return out;
    }
    let stroke = paint
        .as_registry_any()
        .downcast_ref::<Stroke>()
        .filter(|_| !stroke_fills)
        .map(|stroke| {
            out.path_is_local = stroke.base.transform_affects_stroke();
            nuxie_render_api::StrokeParams {
                thickness: stroke.base.thickness(),
                join: StrokeJoin::from(u32::from(stroke.base.join())).into(),
                cap: StrokeCap::from(u32::from(stroke.base.cap())).into(),
                position: stroke.stroke_position(),
            }
        });
    let shape_paint = behavior.shape_paint();
    let mut strength = 0.0;
    if let Some(feather) = shape_paint.feather() {
        feather
            .with(|object| {
                let feather = object
                    .as_registry_any()
                    .downcast_ref::<Feather>()
                    .expect("paint feather");
                strength = feather.base.strength();
                out.feather_offset = Vec2D::new(feather.base.offset_x(), feather.base.offset_y());
                out.feather_offset_in_world = feather.space() == TransformSpace::World;
            })
            .expect("live paint feather");
    }
    out.outset = paint_bounds_outset(stroke.as_ref(), strength);
    out.trustworthy = !shape_paint.effects_container.has_effects();
    out
}
pub fn shape_paints_world_reach(paints: Option<&ShapePaintContainer>, world: &Mat2D) -> PaintReach {
    let Some(paints) = paints else {
        return PaintReach::default();
    };
    shape_paint_handles_world_reach(
        paints.shape_paints().iter().map(|paint| (paint, false)),
        world,
    )
}
/// Nuxie addition: the reach of chosen paints, each with whether it is a
/// Stroke drawn as a filled ring (see [`shape_paint_outset_drawn`]), as
/// [`shape_paints_world_reach`] measures a whole container. A CSS box
/// measures each of its paths with only the paint that draws it.
pub fn shape_paint_handles_world_reach<'a>(
    paints: impl IntoIterator<Item = (&'a CoreHandle, bool)>,
    world: &Mat2D,
) -> PaintReach {
    let mut reach = PaintReach::default();
    let world_scale = world.find_max_scale();
    for (paint, stroke_fills) in paints {
        let po = paint
            .with(|paint| shape_paint_outset_drawn(Some(paint), stroke_fills))
            .expect("live shape paint");
        reach.trustworthy &= po.trustworthy;
        let mut distance = if po.path_is_local {
            po.outset * world_scale
        } else {
            po.outset
        };
        let offset = if po.feather_offset_in_world {
            po.feather_offset
        } else {
            Vec2D::transform_dir(po.feather_offset, world)
        };
        distance += offset.length();
        if distance.is_finite() && reach.world_outset < distance {
            reach.world_outset = distance;
        }
    }
    reach
}
