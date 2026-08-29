use crate::mechanical_port::source::{
    component::Component,
    core::CoreObject,
    generated::core_registry::CoreCapabilities,
    layout::n_sliced_node::NSlicedNode,
    math::{mat2d::Mat2D, raw_path::RawPath, vec2d::Vec2D},
};

pub trait Deformer {
    fn as_component(&mut self) -> &mut Component;
}

pub trait RenderPathDeformer: Deformer {
    fn deform_local_render_path(
        &self,
        path: &mut RawPath,
        world_transform: Mat2D,
        inverse_world: Mat2D,
    );
    fn deform_world_render_path(&self, path: &mut RawPath);
}

pub trait PointDeformer: Deformer {
    fn deform_local_point(
        &self,
        point: Vec2D,
        world_transform: Mat2D,
        inverse_world: Mat2D,
    ) -> Vec2D;
    fn deform_world_point(&self, point: Vec2D) -> Vec2D;
}

pub fn render_path_deformer_from(component: &dyn CoreObject) -> Option<&dyn RenderPathDeformer> {
    component
        .as_any()
        .downcast_ref::<NSlicedNode>()
        .map(|value| value as _)
}

pub fn point_deformer_from(component: &dyn CoreObject) -> Option<&dyn PointDeformer> {
    component
        .as_any()
        .downcast_ref::<NSlicedNode>()
        .map(|value| value as _)
}

impl Deformer for NSlicedNode {
    fn as_component(&mut self) -> &mut Component {
        CoreCapabilities::as_component_mut(self).expect("NSlicedNode inherits Component")
    }
}

impl RenderPathDeformer for NSlicedNode {
    fn deform_local_render_path(&self, path: &mut RawPath, world: Mat2D, inverse_world: Mat2D) {
        NSlicedNode::deform_local_render_path(self, path, &world, &inverse_world);
    }

    fn deform_world_render_path(&self, path: &mut RawPath) {
        NSlicedNode::deform_world_render_path(self, path);
    }
}

impl PointDeformer for NSlicedNode {
    fn deform_local_point(&self, point: Vec2D, world: Mat2D, inverse_world: Mat2D) -> Vec2D {
        NSlicedNode::deform_local_point(self, point, &world, &inverse_world)
    }

    fn deform_world_point(&self, point: Vec2D) -> Vec2D {
        NSlicedNode::deform_world_point(self, point)
    }
}
