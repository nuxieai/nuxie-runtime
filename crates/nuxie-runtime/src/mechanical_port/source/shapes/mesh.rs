use crate::mechanical_port::source::{
    binary_reader::BinaryReader,
    bones::{
        skin::Skin,
        skinnable::{Skinnable, SkinnableBehavior},
    },
    component::{ComponentDirt, has_dirt},
    core::{CoreContext, CoreHandle, StatusCode},
    generated::shapes::mesh_base::MeshBase,
    math::{mat2d::Mat2D, vec2d::Vec2D},
    shapes::{
        image::Image,
        mesh_drawable::{MeshDrawable, MeshDrawableState, RuntimeRenderBufferHandle},
        mesh_vertex::MeshVertex,
        vertex::VertexBehavior,
    },
};
use nuxie_render_api::{
    BlendMode, ImageSampler, RenderBufferFlags, RenderBufferType, RenderImage, Renderer,
};
use std::{cell::RefCell, rc::Rc, sync::Arc};

#[derive(Default)]
pub struct IndexBuffer(pub Vec<u16>);

pub struct Mesh {
    pub base: MeshBase,
    pub skinnable: Skinnable,
    vertex_render_buffer_dirty: bool,
    index_buffer: Option<Arc<IndexBuffer>>,
    vertices: Vec<CoreHandle>,
    mesh: MeshDrawableState,
}

impl Default for Mesh {
    fn default() -> Self {
        Self {
            base: MeshBase::default(),
            skinnable: Skinnable::default(),
            vertex_render_buffer_dirty: true,
            index_buffer: None,
            vertices: Vec::new(),
            mesh: MeshDrawableState::default(),
        }
    }
}

impl SkinnableBehavior for Mesh {
    fn skinnable(&self) -> &Skinnable {
        &self.skinnable
    }

    fn skinnable_mut(&mut self) -> &mut Skinnable {
        &mut self.skinnable
    }

    fn mark_skin_dirty(&mut self) {
        Mesh::mark_skin_dirty(self);
    }
}

impl Mesh {
    pub fn clone_definition(&self) -> Self {
        let factory = self
            .base
            .with_artboard(|artboard| artboard.factory())
            .flatten()
            .expect("Mesh renderer factory");
        let mut twin = Self::default();
        let mut base = std::mem::take(&mut twin.base.base);
        base.copy(&self.base.base, &mut twin);
        twin.base.base = base;
        twin.index_buffer = self.index_buffer.clone();
        twin.vertex_render_buffer_dirty = true;
        twin.mesh.vertex_render_buffer =
            Some(Rc::new(RefCell::new(factory.with_factory_mut(|factory| {
                factory.make_render_buffer(
                    RenderBufferType::Vertex,
                    RenderBufferFlags::None,
                    self.vertices.len() * std::mem::size_of::<Vec2D>(),
                )
            }))));
        twin.mesh.uv_render_buffer = self.mesh.uv_render_buffer.clone();
        twin.mesh.index_render_buffer = self.mesh.index_render_buffer.clone();
        twin
    }
    pub fn mark_drawable_dirty(&mut self) {
        if let Some(skin) = self.skin() {
            skin.with_mut(|skin| {
                if let Some(skin) = skin.as_component_mut() {
                    skin.add_dirt(ComponentDirt::SKIN, true);
                }
            });
        }
        self.base.add_dirt(ComponentDirt::VERTICES, true);
    }
    pub fn add_vertex(&mut self, vertex: CoreHandle) {
        self.vertices.push(vertex);
    }

    pub fn on_added_dirty(&mut self, context: &mut dyn CoreContext) -> StatusCode {
        let result = self.base.on_added_dirty(context);
        if result != StatusCode::Ok {
            return result;
        }
        let (Some(image), Some(this)) = (self.base.parent_handle(), self.base.handle()) else {
            return StatusCode::MissingObject;
        };
        let installed = image
            .with_downcast_mut::<Image, _>(|image| image.set_mesh(Some(this)))
            .is_some();
        if !installed {
            return StatusCode::MissingObject;
        }
        StatusCode::Ok
    }

    pub fn on_added_clean(&mut self, context: &mut dyn CoreContext) -> StatusCode {
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
        self.index_buffer = object
            .handle()
            .and_then(|owner| owner.with_downcast::<Mesh, _>(|owner| owner.index_buffer.clone()))
            .flatten();
    }

    pub fn mark_skin_dirty(&mut self) {
        self.base.add_dirt(ComponentDirt::VERTICES);
    }

    pub fn on_asset_loaded(&mut self, render_image: Option<&dyn RenderImage>) {
        let uv_transform = render_image
            .map(RenderImage::uv_transform)
            .unwrap_or_default();
        let Some(factory) = self
            .base
            .with_artboard(|artboard| artboard.factory())
            .flatten()
        else {
            return;
        };
        self.vertex_render_buffer_dirty = true;
        self.mesh.vertex_render_buffer =
            Some(Rc::new(RefCell::new(factory.with_factory_mut(|factory| {
                factory.make_render_buffer(
                    RenderBufferType::Vertex,
                    RenderBufferFlags::None,
                    self.vertices.len() * std::mem::size_of::<Vec2D>(),
                )
            }))));
        self.mesh.uv_render_buffer =
            Some(Rc::new(RefCell::new(factory.with_factory_mut(|factory| {
                factory.make_render_buffer(
                    RenderBufferType::Vertex,
                    RenderBufferFlags::MappedOnceAtInitialization,
                    self.vertices.len() * std::mem::size_of::<Vec2D>(),
                )
            }))));
        if let Some(buffer) = self.mesh.uv_render_buffer.as_ref() {
            let mut buffer = buffer.borrow_mut();
            {
                let mapped = buffer.map_mut();
                for (output, vertex) in mapped.chunks_exact_mut(8).zip(&self.vertices) {
                    if let Some(uv) = vertex
                        .with(|vertex| {
                            vertex.as_mesh_vertex().map(|vertex| {
                                let uv = uv_transform.transform_point(
                                    nuxie_render_api::Vec2D::new(vertex.base.u(), vertex.base.v()),
                                );
                                Vec2D::new(uv.x, uv.y)
                            })
                        })
                        .flatten()
                    {
                        output[..4].copy_from_slice(&uv.x.to_ne_bytes());
                        output[4..].copy_from_slice(&uv.y.to_ne_bytes());
                    }
                }
            }
            buffer.unmap();
        }
        if let Some(indices) = self.index_buffer.as_ref() {
            self.mesh.index_render_buffer =
                Some(Rc::new(RefCell::new(factory.with_factory_mut(|factory| {
                    factory.make_render_buffer(
                        RenderBufferType::Index,
                        RenderBufferFlags::MappedOnceAtInitialization,
                        indices.0.len() * std::mem::size_of::<u16>(),
                    )
                }))));
            if let Some(buffer) = self.mesh.index_render_buffer.as_ref() {
                let mut buffer = buffer.borrow_mut();
                {
                    let mapped = buffer.map_mut();
                    for (output, index) in mapped.chunks_exact_mut(2).zip(&indices.0) {
                        output.copy_from_slice(&index.to_ne_bytes());
                    }
                }
                buffer.unmap();
            }
        }
    }

    pub fn build_dependencies(&mut self) {
        self.base.build_dependencies();
        let Some(this) = self.base.handle() else {
            return;
        };
        if let Some(skin) = self.skin() {
            skin.with_mut(|skin| {
                if let Some(skin) = skin.as_component_mut() {
                    skin.add_dependent(this.clone());
                }
            });
        }
        if let Some(parent) = self.base.parent_handle() {
            parent.with_mut(|parent| {
                if let Some(parent) = parent.as_component_mut() {
                    parent.add_dependent(this);
                }
            });
        }
    }

    pub fn update(&mut self, value: ComponentDirt) {
        if has_dirt(value, ComponentDirt::VERTICES) {
            if let Some(skin) = self.skin() {
                skin.with_downcast::<Skin, _>(|skin| skin.deform(&self.vertices));
            }
            self.vertex_render_buffer_dirty = true;
        }
        self.base.update(value);
    }

    pub fn draw(
        &mut self,
        renderer: &mut dyn Renderer,
        image: &dyn RenderImage,
        sampler: ImageSampler,
        blend_mode: BlendMode,
        opacity: f32,
    ) {
        if self.vertex_render_buffer_dirty {
            if let Some(buffer) = self.mesh.vertex_render_buffer.as_ref() {
                let mut buffer = buffer.borrow_mut();
                {
                    let mapped = buffer.map_mut();
                    for (output, vertex) in mapped.chunks_exact_mut(8).zip(&self.vertices) {
                        if let Some(position) = vertex
                            .with(|vertex| {
                                vertex
                                    .as_vertex_behavior()
                                    .map(VertexBehavior::render_translation)
                            })
                            .flatten()
                        {
                            output[..4].copy_from_slice(&position.x.to_ne_bytes());
                            output[4..].copy_from_slice(&position.y.to_ne_bytes());
                        }
                    }
                }
                buffer.unmap();
            }
            self.vertex_render_buffer_dirty = false;
        }
        if self.skin().is_none() {
            if let Some(parent) = self.base.parent_handle() {
                parent.with(|parent| {
                    if let Some(parent) = parent.as_world_transform_component() {
                        renderer
                            .transform(nuxie_render_api::Mat2D(*parent.world_transform().values()));
                    }
                });
            }
        }
        let vertex = self
            .mesh
            .vertex_render_buffer
            .as_ref()
            .map(|buffer| buffer.borrow());
        let uv = self
            .mesh
            .uv_render_buffer
            .as_ref()
            .map(|buffer| buffer.borrow());
        let index = self
            .mesh
            .index_render_buffer
            .as_ref()
            .map(|buffer| buffer.borrow());
        renderer.draw_image_mesh(
            Some(image),
            sampler,
            vertex.as_deref().map(Box::as_ref),
            uv.as_deref().map(Box::as_ref),
            index.as_deref().map(Box::as_ref),
            self.vertices.len() as u32,
            self.index_buffer.as_ref().unwrap().0.len() as u32,
            blend_mode,
            opacity,
        );
    }
}

impl MeshDrawable for Mesh {
    fn mesh_state(&mut self) -> &mut MeshDrawableState {
        &mut self.mesh
    }

    fn on_asset_loaded(&mut self, image: &dyn RenderImage) {
        Mesh::on_asset_loaded(self, Some(image));
    }

    fn draw(
        &mut self,
        renderer: &mut dyn Renderer,
        image: &dyn RenderImage,
        sampler: ImageSampler,
        blend_mode: BlendMode,
        opacity: f32,
    ) {
        Mesh::draw(self, renderer, image, sampler, blend_mode, opacity);
    }
}
