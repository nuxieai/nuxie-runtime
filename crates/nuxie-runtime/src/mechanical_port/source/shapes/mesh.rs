use crate::mechanical_port::source::{
    bones::{
        skin::Skin,
        skinnable::{Skinnable, SkinnableBehavior},
    },
    component::{ComponentDirt, has_dirt},
    core::CoreHandle,
    core::binary_reader::BinaryReader,
    core_context::CoreContext,
    generated::shapes::mesh_base::MeshBase,
    math::{mat2d::Mat2D, vec2d::Vec2D},
    shapes::{
        image::Image,
        mesh_drawable::{MeshDrawable, MeshDrawableState, RuntimeRenderBufferHandle},
        mesh_vertex::MeshVertex,
        vertex::VertexBehavior,
    },
    status_code::StatusCode,
};
use nuxie_render_api::{
    BlendMode, ImageSampler, RenderBufferFlags, RenderBufferType, RenderImage, Renderer,
};
use std::{cell::RefCell, rc::Rc, sync::Arc};

#[derive(Default)]
pub struct IndexBuffer(pub Vec<u16>);

impl std::ops::Deref for Mesh {
    type Target = MeshBase;
    fn deref(&self) -> &Self::Target {
        &self.base
    }
}

impl std::ops::DerefMut for Mesh {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.base
    }
}

impl Mesh {
    pub const TYPE_KEY: u16 = MeshBase::TYPE_KEY;
}

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
    /// The CPU positions and topology consumed by draw(), in artboard space.
    /// Skinned positions already contain their world deformation.
    pub(crate) fn rendered_triangles(&self) -> Option<Vec<[Vec2D; 3]>> {
        let transform = if self.skin().is_none() {
            self.base.parent_handle()?.with(|parent| {
                parent
                    .as_world_transform_component()
                    .map(|parent| *parent.world_transform())
            })??
        } else {
            Mat2D::identity()
        };
        let positions = self
            .vertices
            .iter()
            .map(|vertex| {
                vertex
                    .with(|vertex| {
                        vertex
                            .as_vertex_behavior()
                            .map(VertexBehavior::render_translation)
                    })
                    .flatten()
                    .map(|point| transform * point)
            })
            .collect::<Option<Vec<_>>>()?;
        let indices = self.index_buffer.as_ref()?;
        if indices.0.len() % 3 != 0 {
            return None;
        }
        indices
            .0
            .chunks_exact(3)
            .map(|triangle| {
                Some([
                    *positions.get(usize::from(triangle[0]))?,
                    *positions.get(usize::from(triangle[1]))?,
                    *positions.get(usize::from(triangle[2]))?,
                ])
            })
            .collect()
    }

    pub fn clone_definition(&self) -> Self {
        let mut twin = Self::default();
        let mut base = std::mem::take(&mut twin.base.base);
        base.copy(&self.base.base, &mut twin);
        twin.base.base = base;
        twin.index_buffer = self.index_buffer.clone();
        twin.vertex_render_buffer_dirty = true;
        twin.mesh.uv_render_buffer = self.mesh.uv_render_buffer.clone();
        twin.mesh.index_render_buffer = self.mesh.index_render_buffer.clone();
        twin
    }
    pub fn mark_drawable_dirty(&mut self) {
        if let Some(skin) = self.skin() {
            skin.with_mut(|object| {
                if std::any::Any::type_id(&*object) == std::any::TypeId::of::<Skin>() {
                    object
                        .as_any_mut()
                        .downcast_mut::<Skin>()
                        .expect("Skin")
                        .add_dirt_from_mesh(self);
                } else if let Some(component) = object.as_component_mut() {
                    // Retain the established custom projection contract.
                    component.add_dirt(ComponentDirt::SKIN, false);
                }
            });
        }
        self.base.add_dirt(ComponentDirt::VERTICES, false);
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
            .with_downcast_mut::<Image, _>(|image| image.set_mesh(Some(this.clone())))
            .is_some()
            || image
                .with_downcast_mut::<crate::video::Video, _>(|video| {
                    video.set_mesh(Some(this));
                })
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
        self.base.add_dirt(ComponentDirt::VERTICES, false);
    }

    pub fn on_asset_loaded(&mut self, render_image: Option<&dyn RenderImage>) {
        let uv_transform = render_image
            .map(RenderImage::uv_transform)
            .unwrap_or(nuxie_render_api::Mat2D::IDENTITY);
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
        additiveness: f32,
    ) {
        if self.vertex_render_buffer_dirty
            && self.mesh.vertex_render_buffer.is_none()
            && !self.vertices.is_empty()
        {
            let factory = self
                .base
                .with_artboard(|artboard| artboard.factory())
                .flatten()
                .expect("Mesh renderer factory");
            self.mesh.vertex_render_buffer =
                Some(Rc::new(RefCell::new(factory.with_factory_mut(|factory| {
                    factory.make_render_buffer(
                        RenderBufferType::Vertex,
                        RenderBufferFlags::None,
                        self.vertices.len() * std::mem::size_of::<Vec2D>(),
                    )
                }))));
        }
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
                self.vertex_render_buffer_dirty = false;
            }
        }
        if self.skin().is_none() {
            if let Some(parent) = self.base.parent_handle() {
                let transform = parent
                    .with(|parent| {
                        parent.as_world_transform_component().map(|parent| {
                            nuxie_render_api::Mat2D(*parent.world_transform().values())
                        })
                    })
                    .flatten();
                if let Some(transform) = transform {
                    renderer.transform(transform);
                }
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
        renderer.draw_image_mesh_with_additiveness(
            Some(image),
            sampler,
            vertex.as_deref().map(Box::as_ref),
            uv.as_deref().map(Box::as_ref),
            index.as_deref().map(Box::as_ref),
            self.vertices.len() as u32,
            self.index_buffer.as_ref().unwrap().0.len() as u32,
            blend_mode,
            opacity,
            additiveness,
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
        additiveness: f32,
    ) {
        Mesh::draw(
            self,
            renderer,
            image,
            sampler,
            blend_mode,
            opacity,
            additiveness,
        );
    }
}

#[cfg(test)]
mod source_owner_tests {
    use super::*;
    use crate::source::{core::CoreArena, node::Node};
    use nuxie_render_api::{Factory, NullFactory, NullRenderer};

    #[derive(Clone)]
    struct TestImage(Rc<()>);
    impl RenderImage for TestImage {
        fn as_any(&self) -> &dyn std::any::Any {
            self
        }
        fn retain_image(&self) -> Rc<dyn RenderImage> {
            Rc::new(self.clone())
        }
        fn image_identity(&self) -> usize {
            Rc::as_ptr(&self.0) as usize
        }
        fn width(&self) -> u32 {
            1
        }
        fn height(&self) -> u32 {
            1
        }
    }

    #[test]
    fn empty_mesh_keeps_pending_vertex_upload_until_a_buffer_exists() {
        let arena = CoreArena::default();
        let mut mesh = Mesh::default();
        mesh.skinnable.set_skin(arena.insert(Skin::default()));
        mesh.decode_triangle_index_bytes(&[]);
        let image = TestImage(Rc::new(()));
        let mut renderer = NullRenderer::new();
        mesh.draw(
            &mut renderer,
            &image,
            ImageSampler::LINEAR_CLAMP,
            BlendMode::SrcOver,
            1.0,
            0.0,
        );
        assert!(mesh.vertex_render_buffer_dirty);
        let buffer = NullFactory::new().make_render_buffer(
            RenderBufferType::Vertex,
            RenderBufferFlags::None,
            0,
        );
        mesh.mesh.vertex_render_buffer = Some(Rc::new(RefCell::new(buffer)));
        mesh.draw(
            &mut renderer,
            &image,
            ImageSampler::LINEAR_CLAMP,
            BlendMode::SrcOver,
            1.0,
            0.0,
        );
        assert!(!mesh.vertex_render_buffer_dirty);
    }

    #[test]
    fn vertex_movement_marks_mesh_and_skin_without_recursing_into_dependents() {
        let arena = CoreArena::default();
        let mesh = arena.insert(Mesh::default());
        let skin = arena.insert(Skin::default());
        let dependent = arena.insert(Node::default());
        for owner in [&mesh, &skin, &dependent] {
            owner.with_mut(|object| {
                object
                    .as_component_mut()
                    .unwrap()
                    .set_dirt(ComponentDirt::NONE)
            });
        }
        skin.with_mut(|object| {
            object
                .as_component_mut()
                .unwrap()
                .add_dependent(dependent.clone())
        });
        mesh.with_downcast_mut::<Mesh, _>(|mesh| {
            mesh.skinnable.set_skin(skin.clone());
            mesh.base.add_dependent(dependent.clone());
            mesh.mark_drawable_dirty();
        })
        .unwrap();
        assert_eq!(
            mesh.with(|object| object.as_component().unwrap().dirt()),
            Some(ComponentDirt::VERTICES)
        );
        assert_eq!(
            skin.with(|object| object.as_component().unwrap().dirt()),
            Some(ComponentDirt::SKIN)
        );
        assert_eq!(
            dependent.with(|object| object.as_component().unwrap().dirt()),
            Some(ComponentDirt::NONE)
        );
    }
    struct MeshContext<'a> {
        arena: &'a CoreArena,
        image: CoreHandle,
    }
    impl CoreContext for MeshContext<'_> {
        fn core_arena(&self) -> &CoreArena {
            self.arena
        }
        fn resolve_handle(&self, id: u32) -> Option<CoreHandle> {
            (id == 1).then(|| self.image.clone())
        }
    }

    struct MutatesParent {
        parent: CoreHandle,
        transformed: bool,
        drew_mesh: bool,
    }
    impl Renderer for MutatesParent {
        fn save(&mut self) {}
        fn restore(&mut self) {}
        fn transform(&mut self, matrix: nuxie_render_api::Mat2D) {
            assert_eq!(matrix, nuxie_render_api::Mat2D::IDENTITY);
            self.parent
                .with_downcast_mut::<Image, _>(|image| {
                    image.base.set_origin_x_value(0.25);
                })
                .expect("Image parent must be available during renderer callback");
            self.transformed = true;
        }
        fn draw_path(
            &mut self,
            _: &dyn nuxie_render_api::RenderPath,
            _: &dyn nuxie_render_api::RenderPaint,
        ) {
        }
        fn clip_path(&mut self, _: &dyn nuxie_render_api::RenderPath) {}
        fn draw_image(
            &mut self,
            _: Option<&dyn RenderImage>,
            _: ImageSampler,
            _: BlendMode,
            _: f32,
        ) {
        }
        fn draw_image_mesh(
            &mut self,
            _: Option<&dyn RenderImage>,
            _: ImageSampler,
            _: Option<&dyn nuxie_render_api::RenderBuffer>,
            _: Option<&dyn nuxie_render_api::RenderBuffer>,
            _: Option<&dyn nuxie_render_api::RenderBuffer>,
            vertices: u32,
            indices: u32,
            _: BlendMode,
            _: f32,
        ) {
            assert!(self.transformed);
            assert_eq!((vertices, indices), (0, 0));
            self.drew_mesh = true;
        }
        fn modulate_opacity(&mut self, _: f32) {}
    }

    #[test]
    fn mesh_transform_releases_image_parent_before_renderer_callback() {
        let arena = CoreArena::default();
        let parent = arena.insert(Image::default());
        let mesh = arena.insert(Mesh::default());
        let mut context = MeshContext {
            arena: &arena,
            image: parent.clone(),
        };
        mesh.with_downcast_mut::<Mesh, _>(|mesh| {
            mesh.base.set_parent_id_value(1);
            assert_eq!(mesh.on_added_dirty(&mut context), StatusCode::Ok);
            mesh.decode_triangle_index_bytes(&[]);
        })
        .unwrap();
        let mut renderer = MutatesParent {
            parent: parent.clone(),
            transformed: false,
            drew_mesh: false,
        };
        mesh.with_downcast_mut::<Mesh, _>(|mesh| {
            mesh.draw(
                &mut renderer,
                &TestImage(Rc::new(())),
                ImageSampler::LINEAR_CLAMP,
                BlendMode::SrcOver,
                1.0,
                0.0,
            );
        })
        .unwrap();
        assert!(renderer.drew_mesh);
        assert_eq!(
            parent.with_downcast::<Image, _>(|image| image.base.origin_x()),
            Some(0.25)
        );
    }
}
