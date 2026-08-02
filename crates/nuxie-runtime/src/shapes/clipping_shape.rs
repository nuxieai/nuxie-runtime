use nuxie_render_api::Mat2D as RenderMat2D;

use crate::{
    ArtboardInstance, ComponentDirt,
    draw::{
        RuntimeShapePaintPathKind, runtime_draw_property_key_for_name, runtime_fill_rule_for_value,
        runtime_shape_paint_path_kind_slot,
    },
    properties::property_key_for_name,
};

impl ArtboardInstance {
    /// Direct port of pinned C++ `ClippingShape::update`
    /// (`src/shapes/clipping_shape.cpp:151-173`). The dependency node rewinds
    /// its occurrence-owned world path only under Path/WorldTransform/NSlicer
    /// dirt; `emptyClipCount` and draw read the retained pointer thereafter.
    pub(crate) fn update_runtime_clipping_shape_owner(&self, local_id: usize, dirt: ComponentDirt) {
        if (dirt & (ComponentDirt::PATH | ComponentDirt::WORLD_TRANSFORM | ComponentDirt::N_SLICER))
            .is_empty()
        {
            return;
        }
        let Some(owner) = self.runtime_clipping_shapes.get(local_id) else {
            return;
        };

        let fill_rule = runtime_draw_property_key_for_name("ClippingShape", "fillRule")
            .and_then(|key| self.uint_property(local_id, key))
            .unwrap_or(owner.authored_fill_rule);
        owner.fill_rule.set(runtime_fill_rule_for_value(fill_rule));

        let mut path = owner.path.borrow_mut();
        path.rewind();
        let mut has_path = false;
        for shape_local in &owner.shape_locals {
            let Some(shape) = self.runtime_shapes.get(*shape_local) else {
                continue;
            };
            let world_path = shape.paint_paths
                [runtime_shape_paint_path_kind_slot(RuntimeShapePaintPathKind::World)]
            .retained
            .borrow();
            let Some(world_path) = world_path.as_ref() else {
                continue;
            };
            if world_path.raw_path.verbs().is_empty() {
                continue;
            }
            path.add_path(world_path.raw_path.as_ref(), RenderMat2D::IDENTITY);
            has_path = true;
        }
        owner.has_path.set(has_path);
    }
}

pub(crate) fn bool_property_changed(
    artboard: &mut ArtboardInstance,
    _local_id: usize,
    property_key: u16,
) -> Option<bool> {
    if property_key_for_name("ClippingShape", "isVisible") != Some(property_key) {
        return None;
    }
    Some(artboard.add_dirt(0, ComponentDirt::CLIPPING, false))
}
