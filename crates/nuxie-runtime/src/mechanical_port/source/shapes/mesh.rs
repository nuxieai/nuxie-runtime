use crate::mechanical_port::source::{
    binary_reader::BinaryReader,
    bones::skinnable::Skinnable,
    component::{ComponentDirt, has_dirt},
    core::{Core, CoreContext, StatusCode},
    math::{mat2d::Mat2D, vec2d::Vec2D},
    renderer::{
        BlendMode, ImageSampler, RenderBuffer, RenderBufferFlags, RenderBufferType, RenderImage,
        Renderer,
    },
    shapes::{image::Image, mesh_drawable::MeshDrawable, mesh_vertex::MeshVertex},
};
use std::sync::Arc;

#[derive(Default)]
pub struct IndexBuffer(pub Vec<u16>);

pub struct Mesh {
    pub base: MeshBase,
    pub skinnable: Skinnable,
    vertex_render_buffer_dirty: bool,
    index_buffer: Option<Arc<IndexBuffer>>,
    vertices: Vec<MeshVertex>,
    vertex_render_buffer: Option<RenderBuffer>,
    uv_render_buffer: Option<RenderBuffer>,
    index_render_buffer: Option<RenderBuffer>,
}

impl Mesh {
    pub fn mark_drawable_dirty(&mut self) {
        if let Some(skin) = self.skinnable.skin_mut() {
            skin.add_dirt(ComponentDirt::SKIN);
        }
        self.base.add_dirt(ComponentDirt::VERTICES);
    }
    pub fn add_vertex(&mut self, vertex: MeshVertex) {
        self.vertices.push(vertex);
    }

    pub fn on_added_dirty(&mut self, context: &mut CoreContext) -> StatusCode {
        let result = self.base.on_added_dirty(context);
        if result != StatusCode::Ok {
            return result;
        }
        let Some(image) = self.base.parent_mut().as_image_mut() else {
            return StatusCode::MissingObject;
        };
        image.set_mesh(Some(self.clone_mesh_drawable()));
        StatusCode::Ok
    }

    pub fn on_added_clean(&mut self, context: &mut CoreContext) -> StatusCode {
        let Some(indices) = self.index_buffer.as_ref() else {
            return StatusCode::InvalidObject;
        };
        if indices
            .0
            .iter()
            .any(|index| *index as usize >= self.vertices.len())
        {
            return StatusCode::InvalidObject;
        }
        self.base.on_added_clean(context)
    }

    pub fn decode_triangle_index_bytes(&mut self, value: &[u8]) {
        let mut indices = Vec::new();
        let mut reader = BinaryReader::new(value);
        while !reader.reached_end() {
            indices.push(reader.read_var_uint_as::<u16>());
        }
        self.index_buffer = Some(Arc::new(IndexBuffer(indices)));
    }

    pub fn copy_triangle_index_bytes(&mut self, object: &MeshBase) {
        self.index_buffer = object.as_mesh().unwrap().index_buffer.clone();
    }

    pub fn mark_skin_dirty(&mut self) {
        self.base.add_dirt(ComponentDirt::VERTICES);
    }

    pub fn clone_core(&self) -> Core {
        let factory = self.base.artboard().factory();
        let mut clone = self.base.clone_mesh();
        clone.vertex_render_buffer_dirty = true;
        clone.vertex_render_buffer = factory.make_render_buffer(
            RenderBufferType::Vertex,
            RenderBufferFlags::NONE,
            self.vertices.len() * std::mem::size_of::<Vec2D>(),
        );
        clone.uv_render_buffer = self.uv_render_buffer.clone();
        clone.index_render_buffer = self.index_render_buffer.clone();
        clone.into_core()
    }

    pub fn on_asset_loaded(&mut self, render_image: Option<&RenderImage>) {
        let uv_transform = render_image
            .map(RenderImage::uv_transform)
            .unwrap_or_default();
        let factory = self.base.artboard().factory();
        self.vertex_render_buffer_dirty = true;
        self.vertex_render_buffer = factory.make_render_buffer(
            RenderBufferType::Vertex,
            RenderBufferFlags::NONE,
            self.vertices.len() * std::mem::size_of::<Vec2D>(),
        );
        self.uv_render_buffer = factory.make_render_buffer(
            RenderBufferType::Vertex,
            RenderBufferFlags::MAPPED_ONCE_AT_INITIALIZATION,
            self.vertices.len() * std::mem::size_of::<Vec2D>(),
        );
        if let Some(buffer) = self.uv_render_buffer.as_mut() {
            if let Some(mapped) = buffer.map_as_mut::<f32>() {
                for (output, vertex) in mapped.chunks_exact_mut(2).zip(&self.vertices) {
                    let uv = uv_transform * Vec2D::new(vertex.u(), vertex.v());
                    output[0] = uv.x;
                    output[1] = uv.y;
                }
                buffer.unmap();
            }
        }
        if let Some(indices) = self.index_buffer.as_ref() {
            self.index_render_buffer = factory.make_render_buffer(
                RenderBufferType::Index,
                RenderBufferFlags::MAPPED_ONCE_AT_INITIALIZATION,
                indices.0.len() * std::mem::size_of::<u16>(),
            );
            if let Some(buffer) = self.index_render_buffer.as_mut() {
                if let Some(mapped) = buffer.map_as_mut::<u16>() {
                    mapped.copy_from_slice(&indices.0);
                    buffer.unmap();
                }
            }
        }
    }

    pub fn build_dependencies(&mut self) {
        self.base.build_dependencies();
        if let Some(skin) = self.skinnable.skin_mut() {
            skin.add_dependent(&mut self.base);
        }
        self.base.parent_mut().add_dependent(&mut self.base);
    }

    pub fn update(&mut self, value: ComponentDirt) {
        if has_dirt(value, ComponentDirt::VERTICES) {
            if let Some(skin) = self.skinnable.skin_mut() {
                skin.deform(self.vertices.iter_mut().map(MeshVertex::as_vertex_mut));
            }
            self.vertex_render_buffer_dirty = true;
        }
        self.base.update(value);
    }

    pub fn draw(
        &mut self,
        renderer: &mut Renderer,
        image: &RenderImage,
        sampler: ImageSampler,
        blend_mode: BlendMode,
        opacity: f32,
    ) {
        if self.vertex_render_buffer_dirty {
            if let Some(buffer) = self.vertex_render_buffer.as_mut() {
                if let Some(mapped) = buffer.map_as_mut::<Vec2D>() {
                    for (output, vertex) in mapped.iter_mut().zip(&self.vertices) {
                        *output = vertex.render_translation();
                    }
                    buffer.unmap();
                }
            }
            self.vertex_render_buffer_dirty = false;
        }
        if self.skinnable.skin().is_none() {
            renderer.transform(
                self.base
                    .parent()
                    .as_world_transform_component()
                    .unwrap()
                    .world_transform(),
            );
        }
        renderer.draw_image_mesh(
            image,
            sampler,
            self.vertex_render_buffer.as_ref(),
            self.uv_render_buffer.as_ref(),
            self.index_render_buffer.as_ref(),
            self.vertices.len() as u32,
            self.index_buffer.as_ref().unwrap().0.len() as u32,
            blend_mode,
            opacity,
        );
    }
}
