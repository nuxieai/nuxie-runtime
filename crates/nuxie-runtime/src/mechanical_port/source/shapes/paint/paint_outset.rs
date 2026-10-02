//! Runtime half of upstream shapes/paint/paint_outset.{hpp,cpp}.
use crate::mechanical_port::source::{
    core::CoreObject,
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
    let mut out = ShapePaintOutset::default();
    let Some(paint) = paint else { return out };
    let Some(behavior) = paint.as_shape_paint_behavior() else {
        return out;
    };
    if !behavior.should_draw() {
        return out;
    }
    let stroke = paint
        .as_registry_any()
        .downcast_ref::<Stroke>()
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
    let mut reach = PaintReach::default();
    let Some(paints) = paints else { return reach };
    let world_scale = world.find_max_scale();
    for paint in paints.shape_paints() {
        let po = paint
            .with(|paint| shape_paint_outset(Some(paint)))
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
