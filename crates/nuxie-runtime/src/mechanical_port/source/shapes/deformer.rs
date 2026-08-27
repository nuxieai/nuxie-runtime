use crate::mechanical_port::source::{
    component::Component,
    layout::n_sliced_node::NSlicedNode,
    math::{mat2d::Mat2D, raw_path::RawPath, vec2d::Vec2D},
};

pub trait Deformer {
    fn as_component(&self) -> &Component;
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

pub fn render_path_deformer_from(component: &Component) -> Option<&dyn RenderPathDeformer> {
    match component.core_type() {
        NSlicedNode::TYPE_KEY => component.as_n_sliced_node().map(|value| value as _),
        _ => None,
    }
}

pub fn point_deformer_from(component: &Component) -> Option<&dyn PointDeformer> {
    match component.core_type() {
        NSlicedNode::TYPE_KEY => component.as_n_sliced_node().map(|value| value as _),
        _ => None,
    }
}
