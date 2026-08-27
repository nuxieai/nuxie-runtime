use crate::mechanical_port::source::{
    refcnt::Rcp,
    renderer::{RenderBuffer, RenderImage, Renderer},
    shapes::paint::{blend_mode::BlendMode, image_sampler::ImageSampler},
};

#[repr(u8)]
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum MeshType {
    Vertex = 0,
    NSlice = 1,
}
pub struct IndexBuffer {
    pub indices: Vec<u16>,
}

pub struct MeshDrawableState {
    pub index_render_buffer: Option<Rcp<RenderBuffer>>,
    pub vertex_render_buffer: Option<Rcp<RenderBuffer>>,
    pub uv_render_buffer: Option<Rcp<RenderBuffer>>,
}

pub trait MeshDrawable {
    fn mesh_state(&mut self) -> &mut MeshDrawableState;
    fn mesh_type(&self) -> MeshType {
        MeshType::Vertex
    }
    fn on_asset_loaded(&mut self, image: &mut RenderImage);
    fn draw(
        &mut self,
        renderer: &mut Renderer,
        image: &RenderImage,
        sampler: ImageSampler,
        blend_mode: BlendMode,
        opacity: f32,
    );
}
