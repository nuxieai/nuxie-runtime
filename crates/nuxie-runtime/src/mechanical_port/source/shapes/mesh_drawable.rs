use crate::source::math::{mat2d::Mat2D, vec2d::Vec2D};
use std::{cell::RefCell, rc::Rc};

use nuxie_render_api::{BlendMode, ImageSampler, RenderBuffer, RenderImage, Renderer};

pub type RuntimeRenderBufferHandle = Rc<RefCell<Box<dyn RenderBuffer>>>;

#[repr(u8)]
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum MeshType {
    Vertex = 0,
    NSlice = 1,
}
pub struct IndexBuffer {
    pub indices: Vec<u16>,
}

#[derive(Default)]
pub struct MeshDrawableState {
    pub index_render_buffer: Option<RuntimeRenderBufferHandle>,
    pub vertex_render_buffer: Option<RuntimeRenderBufferHandle>,
    pub uv_render_buffer: Option<RuntimeRenderBufferHandle>,
}

pub trait MeshDrawable {
    fn hit_test(&self, position: Vec2D) -> bool;
    fn uv_at(
        &self,
        position: Vec2D,
        uv: &mut Vec2D,
        inside: &mut bool,
        to_uv: Option<&mut Mat2D>,
    ) -> bool;
    fn position_at_uv(&self, uv: Vec2D, position: &mut Vec2D) -> bool;
    fn mesh_state(&mut self) -> &mut MeshDrawableState;
    fn mesh_type(&self) -> MeshType {
        MeshType::Vertex
    }
    fn on_asset_loaded(&mut self, image: &dyn RenderImage);
    fn uv_transform_changed(&mut self, transform: nuxie_render_api::Mat2D);
    fn draw(
        &mut self,
        renderer: &mut dyn Renderer,
        image: &dyn RenderImage,
        sampler: ImageSampler,
        blend_mode: BlendMode,
        opacity: f32,
        additiveness: f32,
    );
}
