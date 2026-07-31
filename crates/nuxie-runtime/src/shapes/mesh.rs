use super::*;
use nuxie_render_api::RenderBuffer;
use std::cell::{Cell, RefCell};
use std::rc::Rc;

use super::slice_mesh::RuntimeSliceMeshOwner;
use crate::{ArtboardInstance, ComponentDirt};

/// Direct `Mesh::onAddedDirty`: validate the Image parent and install this
/// clone-owned Mesh on the Image through `Image::setMesh`.
pub(crate) fn on_added_dirty(
    images: &super::image::RuntimeImageList,
    local_id: usize,
    parent: Option<(usize, &'static str)>,
) -> Result<()> {
    let image_local = parent
        .filter(|(_, type_name)| *type_name == "Image")
        .map(|(local_id, _)| local_id)
        .context("Mesh parent must be an Image")?;
    images
        .set_mesh(
            image_local,
            super::image::RuntimeImageMeshOwner::Mesh(local_id),
        )
        .context("Mesh parent Image must retain a direct owner")?;
    Ok(())
}

/// Clone-owned Mesh and NSlicer resource members. Dense local-id slots are
/// merely the arena representation of the concrete C++ objects; backend
/// buffers live on those direct owners, never on a scene paint cache.
#[derive(Debug, Default)]
pub(crate) struct RuntimeMeshList {
    pub(super) meshes_by_local: Vec<Option<RuntimeMeshOwner>>,
    pub(super) slices_by_local: Vec<Option<RefCell<RuntimeSliceMeshOwner>>>,
    pub(super) details_by_local: Vec<Option<super::n_slicer_details::RuntimeNSlicerDetailsOwner>>,
}

impl Clone for RuntimeMeshList {
    fn clone(&self) -> Self {
        Self {
            meshes_by_local: self.meshes_by_local.clone(),
            slices_by_local: self
                .slices_by_local
                .iter()
                .map(|owner| {
                    owner
                        .as_ref()
                        .map(|owner| RefCell::new(owner.borrow().clone()))
                })
                .collect(),
            details_by_local: self.details_by_local.clone(),
        }
    }
}

impl RuntimeMeshList {
    pub(crate) fn from_graph(graph: &ArtboardGraph) -> Self {
        let mut owners = Self::default();
        if let Some(maximum) = graph.meshes.iter().map(|mesh| mesh.local_id).max() {
            owners
                .meshes_by_local
                .resize_with(maximum.saturating_add(1), || None);
            for mesh in &graph.meshes {
                let mut owner = RuntimeMeshOwner::new(mesh.local_id);
                for vertex in &mesh.vertices {
                    super::mesh_vertex::on_added_dirty(
                        mesh.local_id,
                        &mut owner.vertex_locals,
                        vertex,
                        Some(mesh.local_id),
                    )
                    .expect("validated Mesh projection must retain each MeshVertex parent");
                }
                owners.meshes_by_local[mesh.local_id] = Some(owner);
            }
        }
        if let Some(maximum) = graph
            .n_slicer_details
            .iter()
            .map(|details| details.local_id)
            .max()
        {
            owners
                .details_by_local
                .resize_with(maximum.saturating_add(1), || None);
            for details in &graph.n_slicer_details {
                owners.details_by_local[details.local_id] = Some(
                    super::n_slicer_details::RuntimeNSlicerDetailsOwner::from_graph(details)
                        .expect("validated NSlicerDetails projection must retain child parents"),
                );
            }
        }
        if let Some(maximum) = graph
            .n_slicer_details
            .iter()
            .filter(|details| details.type_name == "NSlicer")
            .map(|details| details.local_id)
            .max()
        {
            owners
                .slices_by_local
                .resize_with(maximum.saturating_add(1), || None);
            for details in graph
                .n_slicer_details
                .iter()
                .filter(|details| details.type_name == "NSlicer")
            {
                owners.slices_by_local[details.local_id] =
                    Some(RefCell::new(RuntimeSliceMeshOwner::new(details.local_id)));
            }
        }
        owners
    }

    pub(crate) fn mark_component_dirt(&self, local_id: usize, dirt: ComponentDirt) {
        if dirt.contains(ComponentDirt::VERTICES)
            && let Some(mesh) = self.meshes_by_local.get(local_id).and_then(Option::as_ref)
        {
            mesh.vertex_render_buffer_dirty.set(true);
        }
        if !(dirt & (ComponentDirt::N_SLICER | ComponentDirt::WORLD_TRANSFORM)).is_empty()
            && let Some(slice) = self.slices_by_local.get(local_id).and_then(Option::as_ref)
        {
            slice.borrow_mut().dirty = true;
        }
    }

    pub(crate) fn mesh(&self, local_id: usize) -> Option<&RuntimeMeshOwner> {
        self.meshes_by_local.get(local_id)?.as_ref()
    }

    pub(crate) fn slice(&self, local_id: usize) -> Option<&RefCell<RuntimeSliceMeshOwner>> {
        self.slices_by_local.get(local_id)?.as_ref()
    }

    pub(crate) fn details(
        &self,
        local_id: usize,
    ) -> Option<&super::n_slicer_details::RuntimeNSlicerDetailsOwner> {
        self.details_by_local.get(local_id)?.as_ref()
    }
}

pub(crate) struct RuntimeMeshSharedRenderBuffers {
    pub(crate) context_id: Option<u64>,
    pub(crate) source_vertices: Option<Box<dyn RenderBuffer>>,
    pub(crate) uv_coords: Option<Box<dyn RenderBuffer>>,
    pub(crate) indices: Option<Box<dyn RenderBuffer>>,
    pub(crate) vertex_count: u32,
    pub(crate) index_count: u32,
}

impl Default for RuntimeMeshSharedRenderBuffers {
    fn default() -> Self {
        Self {
            context_id: None,
            source_vertices: None,
            uv_coords: None,
            indices: None,
            vertex_count: 0,
            index_count: 0,
        }
    }
}

/// Concrete clone-owned `Mesh` resource state. Clones share immutable UV and
/// index buffers while keeping a fresh dirty dynamic vertex buffer.
pub(crate) struct RuntimeMeshOwner {
    pub(crate) local_id: usize,
    pub(crate) vertex_locals: Vec<usize>,
    pub(crate) shared: RefCell<Rc<RefCell<RuntimeMeshSharedRenderBuffers>>>,
    pub(crate) vertices: RefCell<Option<(u64, Box<dyn RenderBuffer>)>>,
    pub(crate) vertex_render_buffer_dirty: Cell<bool>,
    pub(crate) settled_vertex_bytes: RefCell<Option<Vec<u8>>>,
}

impl RuntimeMeshOwner {
    pub(crate) fn new(local_id: usize) -> Self {
        Self {
            local_id,
            vertex_locals: Vec::new(),
            shared: RefCell::new(Rc::new(RefCell::new(
                RuntimeMeshSharedRenderBuffers::default(),
            ))),
            vertices: RefCell::new(None),
            vertex_render_buffer_dirty: Cell::new(true),
            settled_vertex_bytes: RefCell::new(None),
        }
    }
}

impl Clone for RuntimeMeshOwner {
    fn clone(&self) -> Self {
        // Direct `Mesh::clone`: the dynamic vertex buffer is fresh and dirty;
        // UV/index retain the source-owned reference-counted buffers.
        Self {
            local_id: self.local_id,
            vertex_locals: self.vertex_locals.clone(),
            shared: RefCell::new(Rc::clone(&self.shared.borrow())),
            vertices: RefCell::new(None),
            vertex_render_buffer_dirty: Cell::new(true),
            settled_vertex_bytes: RefCell::new(None),
        }
    }
}

impl std::fmt::Debug for RuntimeMeshOwner {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("RuntimeMeshOwner")
            .field("local_id", &self.local_id)
            .field(
                "vertex_render_buffer_dirty",
                &self.vertex_render_buffer_dirty.get(),
            )
            .finish()
    }
}

/// Direct port of `Mesh::markDrawableDirty`, including the retained Skin
/// owner's notification before publishing vertex dirt on the Mesh.
pub(crate) fn mark_vertices_dirty(instance: &mut ArtboardInstance, mesh_local: usize) -> bool {
    let Some(mesh) = instance.component_handle(mesh_local) else {
        return false;
    };
    if let Some(skin) = instance.runtime_skinnable_skin_local(mesh) {
        instance.add_dirt(skin, ComponentDirt::SKIN, false);
    }
    instance.add_dirt(mesh_local, ComponentDirt::VERTICES, false)
}

pub(super) fn runtime_draw_mesh_image(
    runtime: &RuntimeFile,
    instance: &ArtboardInstance,
    graph: &ArtboardGraph,
    mesh_component: ComponentHandle,
    image_local: usize,
    image_object: Option<&RuntimeObject>,
    _resolved_image_asset_global: Option<u32>,
    mesh: &MeshGeometryNode,
    owner: &RuntimeMeshOwner,
    layout_bounds: Option<&BTreeMap<usize, RuntimeLayoutBounds>>,
    image: &dyn RenderImage,
    backend_context_id: u64,
    factory: &mut dyn RenderFactory,
    path_cache: &mut RuntimeArtboardPathState,
    renderer: &mut dyn Renderer,
) -> Result<()> {
    // C++ `Mesh::draw` reads the Mesh's retained `Skinnable::m_Skin`
    // pointer, never the parent Image's Component state. The Image supplies
    // draw ownership while the distinct Mesh occurrence decides whether its
    // vertices are already in artboard space (`mesh.cpp:175-207`; FLR-11,
    // RF-30).
    let weighted_context = instance
        .runtime_skinnable_handle_has_skin(mesh_component)
        .then_some(WeightedPathContext { instance });
    if weighted_context.is_none() {
        let world = path_cache
            .image_world_transform_with_bounds(
                runtime,
                instance,
                graph,
                image_local,
                layout_bounds,
            )?
            .unwrap_or_else(|| {
                path_cache.component_world_transform_with_bounds(
                    instance,
                    graph,
                    image_local,
                    layout_bounds,
                )
            });
        renderer.transform(runtime_render_mat(world));
    }

    runtime_realize_mesh_owner(
        runtime,
        instance,
        mesh,
        weighted_context.as_ref(),
        image,
        backend_context_id,
        factory,
        owner,
    )?;
    let vertices = owner.vertices.borrow();
    let Some((_, vertices)) = vertices.as_ref() else {
        return Ok(());
    };
    let shared_handle = Rc::clone(&owner.shared.borrow());
    let shared = shared_handle.borrow();
    let (Some(uv_coords), Some(indices)) = (shared.uv_coords.as_deref(), shared.indices.as_deref())
    else {
        return Ok(());
    };

    let blend_mode_key = runtime_draw_property_key_for_name("Drawable", "blendModeValue")
        .context("missing Drawable.blendModeValue")?;
    let blend_mode_value = instance
        .uint_property(image_local, blend_mode_key)
        .or_else(|| {
            image_object.and_then(|object| {
                runtime_object_explicit_uint_property_by_key(object, blend_mode_key)
            })
        })
        .unwrap_or(3);
    let opacity = instance
        .component(image_local)
        .map(|component| component.transform.render_opacity)
        .unwrap_or(1.0);
    renderer.draw_image_mesh(
        Some(image),
        RenderImageSampler::LINEAR_CLAMP,
        Some(vertices.as_ref()),
        Some(uv_coords),
        Some(indices),
        shared.vertex_count,
        shared.index_count,
        runtime_blend_mode(u32::try_from(blend_mode_value).unwrap_or(3))?,
        opacity,
    );
    Ok(())
}

pub(super) fn runtime_realize_mesh_owner(
    runtime: &RuntimeFile,
    instance: &ArtboardInstance,
    mesh: &MeshGeometryNode,
    weighted_context: Option<&WeightedPathContext<'_>>,
    image: &dyn RenderImage,
    backend_context_id: u64,
    factory: &mut dyn RenderFactory,
    owner: &RuntimeMeshOwner,
) -> Result<()> {
    {
        let shared_handle = Rc::clone(&owner.shared.borrow());
        let mut shared = shared_handle.borrow_mut();
        if shared.context_id != Some(backend_context_id) {
            shared.context_id = Some(backend_context_id);
            shared.source_vertices = None;
            shared.uv_coords = None;
            shared.indices = None;
            shared.vertex_count = 0;
            shared.index_count = 0;
        }
        if shared.source_vertices.is_none()
            || shared.uv_coords.is_none()
            || shared.indices.is_none()
        {
            runtime_realize_mesh_shared_buffers(runtime, mesh, image, factory, &mut shared)?;
        }
    }

    let vertex_byte_length = mesh.vertices.len().saturating_mul(8);
    let mut vertices = owner.vertices.borrow_mut();
    if vertices.as_ref().is_some_and(|(context_id, buffer)| {
        *context_id != backend_context_id || buffer.size_in_bytes() != vertex_byte_length
    }) {
        *vertices = None;
        owner.vertex_render_buffer_dirty.set(true);
    }
    if vertices.is_none() {
        *vertices = Some((
            backend_context_id,
            factory.make_render_buffer(
                RenderBufferType::Vertex,
                RenderBufferFlags::None,
                vertex_byte_length,
            ),
        ));
        owner.vertex_render_buffer_dirty.set(true);
    }
    if owner.vertex_render_buffer_dirty.get()
        && let Some((_, buffer)) = vertices.as_mut()
    {
        let bytes = if let Some(bytes) = owner.settled_vertex_bytes.borrow().as_ref() {
            bytes.clone()
        } else {
            runtime_mesh_vertex_buffer_bytes(instance, mesh, weighted_context)?
        };
        write_render_buffer_bytes(buffer.as_mut(), &bytes);
        *owner.settled_vertex_bytes.borrow_mut() = Some(bytes);
        owner.vertex_render_buffer_dirty.set(false);
    }
    Ok(())
}

pub(super) fn runtime_realize_mesh_shared_buffers(
    runtime: &RuntimeFile,
    mesh: &MeshGeometryNode,
    image: &dyn RenderImage,
    factory: &mut dyn RenderFactory,
    shared: &mut RuntimeMeshSharedRenderBuffers,
) -> Result<()> {
    let mesh_object = runtime
        .object(mesh.global_id as usize)
        .with_context(|| format!("missing mesh global {}", mesh.global_id))?;
    let indices = mesh_object
        .mesh_triangle_indices()
        .context("mesh missing triangle indices")?;
    let u_key =
        runtime_draw_property_key_for_name("MeshVertex", "u").context("missing MeshVertex.u")?;
    let v_key =
        runtime_draw_property_key_for_name("MeshVertex", "v").context("missing MeshVertex.v")?;
    let source_vertices = factory.make_render_buffer(
        RenderBufferType::Vertex,
        RenderBufferFlags::None,
        mesh.vertices.len().saturating_mul(8),
    );
    let mut uv_bytes = Vec::with_capacity(mesh.vertices.len() * 8);
    for vertex in &mesh.vertices {
        let vertex_object = runtime
            .object(vertex.global_id as usize)
            .with_context(|| format!("missing mesh vertex global {}", vertex.global_id))?;
        let uv = image.uv_transform().transform_point(RenderVec2D::new(
            runtime_object_explicit_double_property_by_key(vertex_object, u_key).unwrap_or(0.0),
            runtime_object_explicit_double_property_by_key(vertex_object, v_key).unwrap_or(0.0),
        ));
        push_f32_pair_bytes(&mut uv_bytes, uv.x, uv.y);
    }
    let mut uv_coords = factory.make_render_buffer(
        RenderBufferType::Vertex,
        RenderBufferFlags::MappedOnceAtInitialization,
        uv_bytes.len(),
    );
    write_render_buffer_bytes(uv_coords.as_mut(), &uv_bytes);

    let mut index_bytes = Vec::with_capacity(indices.len() * 2);
    for index in indices {
        index_bytes.extend_from_slice(&index.to_le_bytes());
    }
    let mut index_buffer = factory.make_render_buffer(
        RenderBufferType::Index,
        RenderBufferFlags::MappedOnceAtInitialization,
        index_bytes.len(),
    );
    write_render_buffer_bytes(index_buffer.as_mut(), &index_bytes);

    shared.source_vertices = Some(source_vertices);
    shared.uv_coords = Some(uv_coords);
    shared.indices = Some(index_buffer);
    shared.vertex_count = u32::try_from(mesh.vertices.len()).unwrap_or(u32::MAX);
    shared.index_count = u32::try_from(index_bytes.len() / 2).unwrap_or(u32::MAX);
    Ok(())
}

pub(super) fn runtime_mesh_vertex_buffer_bytes(
    instance: &ArtboardInstance,
    mesh: &MeshGeometryNode,
    weighted_context: Option<&WeightedPathContext<'_>>,
) -> Result<Vec<u8>> {
    let mut bytes = Vec::with_capacity(mesh.vertices.len() * 8);
    for vertex in &mesh.vertices {
        let (x, y) = runtime_mesh_vertex_render_translation(instance, vertex, weighted_context)?;
        push_f32_pair_bytes(&mut bytes, x, y);
    }
    Ok(bytes)
}

pub(super) fn runtime_mesh_vertex_render_translation(
    instance: &ArtboardInstance,
    vertex: &MeshVertexNode,
    weighted_context: Option<&WeightedPathContext<'_>>,
) -> Result<(f32, f32)> {
    let x_key = runtime_draw_property_key_for_name("Vertex", "x").context("missing Vertex.x")?;
    let y_key = runtime_draw_property_key_for_name("Vertex", "y").context("missing Vertex.y")?;
    let x = instance
        .double_property(vertex.local_id, x_key)
        .unwrap_or(0.0);
    let y = instance
        .double_property(vertex.local_id, y_key)
        .unwrap_or(0.0);
    if let Some(weighted_context) = weighted_context
        && vertex.weight_local.is_some()
    {
        return weighted_context
            .translation(vertex.local_id)
            .context("mesh Weight occurrence has no settled translation");
    }
    Ok((x, y))
}

pub(super) fn preallocate_file_source_mesh_owners(
    runtime: &RuntimeFile,
    artboards: &[ArtboardGraph],
    factory: &mut dyn RenderFactory,
    image_assets: &RuntimeImageAssetOwners,
    backend_context_id: u64,
) {
    // Direct port of `Mesh::onAssetLoaded` before any Artboard clone is
    // created (`src/shapes/mesh.cpp:101-150`). Source-artboard buffers are
    // file-owned; each later occurrence shares UV/index from these owners.
    for graph in artboards {
        for mesh in &graph.meshes {
            let Some(image) = runtime_source_mesh_image(runtime, graph, mesh, image_assets) else {
                continue;
            };
            let source = Rc::new(RefCell::new(RuntimeMeshSharedRenderBuffers {
                context_id: Some(backend_context_id),
                ..RuntimeMeshSharedRenderBuffers::default()
            }));
            if mesh::runtime_realize_mesh_shared_buffers(
                runtime,
                mesh,
                image.as_ref(),
                factory,
                &mut source.borrow_mut(),
            )
            .is_ok()
            {
                image_assets.insert_source_mesh(graph.global_id, mesh.local_id, source);
            }
        }
    }
}

fn runtime_source_mesh_image(
    runtime: &RuntimeFile,
    graph: &ArtboardGraph,
    mesh: &MeshGeometryNode,
    image_assets: &RuntimeImageAssetOwners,
) -> Option<Rc<dyn RenderImage>> {
    let mesh_object = runtime.object(mesh.global_id as usize)?;
    let parent_id_key = runtime_draw_property_key_for_name("Component", "parentId")
        .or_else(|| runtime_draw_property_key_for_name("WorldTransformComponent", "parentId"))?;
    let image_local = usize::try_from(runtime_object_uint_property_by_key(
        mesh_object,
        parent_id_key,
    )?)
    .ok()?;
    let image_global = graph
        .local_objects
        .iter()
        .find(|object| object.local_id == image_local)?
        .global_id;
    let image_object = runtime.object(image_global as usize)?;
    let image_asset = runtime.resolved_file_asset_for_referencer(image_object)?;
    image_assets.get(image_asset.id)
}
