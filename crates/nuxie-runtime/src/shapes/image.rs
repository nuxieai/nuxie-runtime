use super::*;

/// Concrete Image members retained by one Artboard occurrence. This owns the
/// C++ `Image::{m_layoutWidth,m_layoutHeight,m_layoutOffset*,m_layoutScale*}`
/// state and is cloned cold like `Image::clone` before layout control reruns.
#[derive(Debug, Default)]
pub(crate) struct RuntimeImageList {
    by_local: Vec<Option<Rc<RefCell<RuntimeImageOwner>>>>,
}

impl RuntimeImageList {
    pub(crate) fn from_graph(runtime: &RuntimeFile, graph: &ArtboardGraph) -> Self {
        let separate = runtime_layout_image_uses_separate_fit_scale(
            runtime.header.major_version,
            runtime.header.minor_version,
        );
        let image_locals = graph
            .components
            .iter()
            .filter(|component| component.type_name == "Image")
            .map(|component| component.local_id)
            .collect::<Vec<_>>();
        let mut by_local = Vec::new();
        if let Some(maximum) = image_locals.iter().copied().max() {
            by_local.resize_with(maximum + 1, || None);
            for local_id in image_locals {
                let image_object = graph
                    .components
                    .iter()
                    .find(|component| component.local_id == local_id)
                    .and_then(|component| runtime.object(component.global_id as usize));
                by_local[local_id] = Some(Rc::new(RefCell::new(RuntimeImageOwner::new(
                    separate,
                    image_object,
                ))));
            }
        }
        let owners = Self { by_local };
        owners
            .register_mesh_children(graph)
            .expect("validated Image children must register their Mesh owners");
        owners
    }

    fn owner(&self, local_id: usize) -> Option<&Rc<RefCell<RuntimeImageOwner>>> {
        self.by_local.get(local_id)?.as_ref()
    }

    pub(crate) fn public_scale(&self, local_id: usize, property_key: u16) -> Option<f32> {
        self.owner(local_id)?.borrow().public_scale(property_key)
    }

    pub(crate) fn has_public_scale(&self, local_id: usize, property_key: u16) -> bool {
        self.owner(local_id)
            .is_some_and(|owner| owner.borrow().has_public_scale(property_key))
    }

    pub(crate) fn mark_public_scale_written(&self, local_id: usize, property_key: u16) -> bool {
        self.owner(local_id)
            .is_some_and(|owner| owner.borrow_mut().mark_public_scale_written(property_key))
    }

    /// Direct `Image::controlSize`: retain the solved dimensions on the Image
    /// occurrence. The caller then invokes `updateImageScale` at the same
    /// mutation boundary.
    pub(crate) fn control_size(&self, local_id: usize, width: f32, height: f32) -> Option<bool> {
        self.owner(local_id)?
            .borrow_mut()
            .control_size(width, height)
    }

    pub(crate) fn set_asset(
        &self,
        local_id: usize,
        asset_global: Option<u32>,
        dimensions: Option<(u32, u32)>,
    ) -> bool {
        self.owner(local_id).is_some_and(|owner| {
            owner
                .borrow_mut()
                .set_asset(asset_global, dimensions.map(|(w, h)| (w as f32, h as f32)))
        })
    }

    /// Direct `Image::setMesh`: retain the clone-local non-owning mesh link
    /// and immediately rerun layout fit at the child registration boundary.
    pub(crate) fn set_mesh(&self, local_id: usize, mesh: RuntimeImageMeshOwner) -> Option<bool> {
        Some(self.owner(local_id)?.borrow_mut().set_mesh(mesh))
    }

    pub(crate) fn mesh(&self, local_id: usize) -> Option<RuntimeImageMeshOwner> {
        self.owner(local_id)?.borrow().mesh
    }

    fn register_mesh_children(&self, graph: &ArtboardGraph) -> Result<()> {
        for component in &graph.components {
            let parent = graph
                .components
                .iter()
                .find(|candidate| candidate.children.contains(&component.local_id))
                .map(|candidate| (candidate.local_id, candidate.type_name));
            if graph
                .meshes
                .iter()
                .any(|mesh| mesh.local_id == component.local_id)
            {
                super::mesh::on_added_dirty(self, component.local_id, parent)?;
            } else if graph.n_slicer_details.iter().any(|details| {
                details.local_id == component.local_id && details.type_name == "NSlicer"
            }) {
                super::n_slicer::on_added_dirty(self, component.local_id, parent)?;
            }
        }
        Ok(())
    }

    pub(crate) fn apply_double_property(&self, local_id: usize, property_key: u16, value: f32) {
        if let Some(owner) = self.owner(local_id) {
            owner
                .borrow_mut()
                .apply_double_property(property_key, value);
        }
    }

    pub(crate) fn apply_uint_property(&self, local_id: usize, property_key: u16, value: u64) {
        if let Some(owner) = self.owner(local_id) {
            owner.borrow_mut().apply_uint_property(property_key, value);
        }
    }

    pub(crate) fn register_asset_referencers(
        &self,
        queue: &crate::draw::image_asset::RuntimeImageAssetReferencerQueue,
    ) {
        queue.replace_images(
            self.by_local
                .iter()
                .enumerate()
                .filter_map(|(local_id, owner)| Some((local_id, Rc::downgrade(owner.as_ref()?)))),
        );
    }
}

impl Clone for RuntimeImageList {
    fn clone(&self) -> Self {
        Self {
            by_local: self
                .by_local
                .iter()
                .map(|owner| {
                    owner.as_ref().map(|owner| {
                        let owner = owner.borrow();
                        let mut cloned = RuntimeImageOwner::new(owner.layout_scale_separate, None);
                        cloned.mesh = owner.mesh;
                        cloned.fit = owner.fit;
                        cloned.origin_x = owner.origin_x;
                        cloned.origin_y = owner.origin_y;
                        cloned.alignment_x = owner.alignment_x;
                        cloned.alignment_y = owner.alignment_y;
                        Rc::new(RefCell::new(cloned))
                    })
                })
                .collect(),
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub(super) struct RuntimeImageOwner {
    asset_global: Option<u32>,
    image_width: Option<f32>,
    image_height: Option<f32>,
    layout_width: f32,
    layout_height: f32,
    layout_offset_x: f32,
    layout_offset_y: f32,
    layout_scale_x: f32,
    layout_scale_y: f32,
    layout_scale_separate: bool,
    mesh: Option<RuntimeImageMeshOwner>,
    has_layout_fit: bool,
    fit: u64,
    origin_x: f32,
    origin_y: f32,
    alignment_x: f32,
    alignment_y: f32,
    user_scale_x: bool,
    user_scale_y: bool,
}

impl RuntimeImageOwner {
    fn new(layout_scale_separate: bool, image_object: Option<&RuntimeObject>) -> Self {
        Self {
            asset_global: None,
            image_width: None,
            image_height: None,
            layout_width: f32::NAN,
            layout_height: f32::NAN,
            layout_offset_x: 0.0,
            layout_offset_y: 0.0,
            layout_scale_x: 1.0,
            layout_scale_y: 1.0,
            layout_scale_separate,
            mesh: None,
            has_layout_fit: false,
            fit: image_object
                .and_then(|object| object.uint_property("fit"))
                .unwrap_or(0),
            origin_x: image_object
                .and_then(|object| object.double_property("originX"))
                .unwrap_or(0.5),
            origin_y: image_object
                .and_then(|object| object.double_property("originY"))
                .unwrap_or(0.5),
            alignment_x: image_object
                .and_then(|object| object.double_property("alignmentX"))
                .unwrap_or(0.0),
            alignment_y: image_object
                .and_then(|object| object.double_property("alignmentY"))
                .unwrap_or(0.0),
            user_scale_x: false,
            user_scale_y: false,
        }
    }

    fn control_size(&mut self, width: f32, height: f32) -> Option<bool> {
        if self.layout_width == width && self.layout_height == height {
            return None;
        }
        self.layout_width = width;
        self.layout_height = height;
        Some(self.update_image_scale_from_members())
    }

    fn set_asset(&mut self, asset_global: Option<u32>, dimensions: Option<(f32, f32)>) -> bool {
        let current_dimensions = self.image_width.zip(self.image_height);
        if self.asset_global == asset_global && current_dimensions == dimensions {
            return false;
        }
        self.asset_global = asset_global;
        (self.image_width, self.image_height) = dimensions
            .map(|(width, height)| (Some(width), Some(height)))
            .unwrap_or((None, None));
        self.update_image_scale_from_members()
    }

    fn set_mesh(&mut self, mesh: RuntimeImageMeshOwner) -> bool {
        if self.mesh == Some(mesh) {
            return false;
        }
        self.mesh = Some(mesh);
        self.update_image_scale_from_members()
    }

    pub(super) fn asset_updated(&mut self, global_id: u32, width: u32, height: u32) -> bool {
        if self.asset_global != Some(global_id) {
            return false;
        }
        self.image_width = Some(width as f32);
        self.image_height = Some(height as f32);
        self.update_image_scale_from_members();
        true
    }

    fn apply_double_property(&mut self, property_key: u16, value: f32) {
        if property_key_for_name("Image", "originX") == Some(property_key) {
            self.origin_x = value;
        } else if property_key_for_name("Image", "originY") == Some(property_key) {
            self.origin_y = value;
        } else if property_key_for_name("Image", "alignmentX") == Some(property_key) {
            self.alignment_x = value;
        } else if property_key_for_name("Image", "alignmentY") == Some(property_key) {
            self.alignment_y = value;
        }
    }

    fn apply_uint_property(&mut self, property_key: u16, value: u64) {
        if property_key_for_name("Image", "fit") == Some(property_key) {
            self.fit = value;
        }
    }

    fn update_image_scale_from_members(&mut self) -> bool {
        let fit = match (self.image_width, self.image_height) {
            (Some(image_width), Some(image_height))
                if !self.layout_width.is_nan() && !self.layout_height.is_nan() =>
            {
                Some(runtime_image_layout_fit_values(
                    image_width,
                    image_height,
                    self.layout_width,
                    self.layout_height,
                    matches!(self.mesh, Some(RuntimeImageMeshOwner::Mesh(_))),
                    self.fit,
                    self.origin_x,
                    self.origin_y,
                    self.alignment_x,
                    self.alignment_y,
                ))
            }
            _ => None,
        };
        self.update_image_scale(fit)
    }

    fn update_image_scale(&mut self, fit: Option<RuntimeImageLayoutFit>) -> bool {
        let Some(fit) = fit else {
            let changed = self.layout_offset_x != 0.0 || self.layout_offset_y != 0.0;
            self.layout_offset_x = 0.0;
            self.layout_offset_y = 0.0;
            return changed;
        };
        let changed = self.layout_offset_x != fit.offset_x
            || self.layout_offset_y != fit.offset_y
            || self.layout_scale_x != fit.scale_x
            || self.layout_scale_y != fit.scale_y
            || !self.has_layout_fit;
        self.layout_offset_x = fit.offset_x;
        self.layout_offset_y = fit.offset_y;
        self.layout_scale_x = fit.scale_x;
        self.layout_scale_y = fit.scale_y;
        self.has_layout_fit = true;
        if !self.layout_scale_separate {
            self.user_scale_x = false;
            self.user_scale_y = false;
        }
        changed
    }

    fn public_scale(&self, property_key: u16) -> Option<f32> {
        if self.layout_scale_separate || !self.has_layout_fit {
            return None;
        }
        match (
            image_scale_axis(property_key)?,
            self.user_scale_x,
            self.user_scale_y,
        ) {
            (true, false, _) => Some(self.layout_scale_x),
            (false, _, false) => Some(self.layout_scale_y),
            _ => None,
        }
    }

    fn has_public_scale(&self, property_key: u16) -> bool {
        !self.layout_scale_separate
            && self.has_layout_fit
            && image_scale_axis(property_key).is_some()
    }

    fn mark_public_scale_written(&mut self, property_key: u16) -> bool {
        if !self.has_public_scale(property_key) {
            return false;
        }
        if image_scale_axis(property_key) == Some(true) {
            let changed = !self.user_scale_x;
            self.user_scale_x = true;
            changed
        } else {
            let changed = !self.user_scale_y;
            self.user_scale_y = true;
            changed
        }
    }
}

fn image_scale_axis(property_key: u16) -> Option<bool> {
    static SCALE_KEYS: OnceLock<(Option<u16>, Option<u16>)> = OnceLock::new();
    let (scale_x, scale_y) = *SCALE_KEYS.get_or_init(|| {
        (
            property_key_for_name("Node", "scaleX"),
            property_key_for_name("Node", "scaleY"),
        )
    });
    if scale_x == Some(property_key) {
        Some(true)
    } else if scale_y == Some(property_key) {
        Some(false)
    } else {
        None
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum RuntimeImageMeshOwner {
    Mesh(usize),
    SliceMesh(usize),
}

use crate::Mat2D;

/// Direct port of Image's two layout-scale storage modes. Older files bake
/// fit scale into the local transform; current files retain it separately.
pub(crate) fn apply_layout_fit(
    mut base_local_transform: Mat2D,
    scale_x: f32,
    scale_y: f32,
    offset_x: f32,
    offset_y: f32,
    layout_scale_separate: bool,
) -> Mat2D {
    if layout_scale_separate {
        base_local_transform.scale_by_values(scale_x, scale_y);
        base_local_transform.0[4] += offset_x;
        base_local_transform.0[5] += offset_y;
        base_local_transform
    } else {
        let mut components = base_local_transform.decompose();
        components.scale_x = scale_x;
        components.scale_y = scale_y;
        components.x += offset_x;
        components.y += offset_y;
        Mat2D::compose(components)
    }
}

pub(super) fn runtime_layout_image_uses_separate_fit_scale(
    major_version: u64,
    minor_version: u64,
) -> bool {
    major_version > 7 || (major_version == 7 && minor_version >= 2)
}

#[derive(Debug, Clone, Copy)]
pub(super) struct RuntimeImageLayoutFit {
    scale_x: f32,
    scale_y: f32,
    offset_x: f32,
    offset_y: f32,
}

#[derive(Debug, Clone, Copy)]
pub(super) struct RuntimeImageLayoutTransform {
    pub(super) local_transform: Mat2D,
    pub(super) render_scale_x: f32,
    pub(super) render_scale_y: f32,
}

#[derive(Debug, Clone, Copy)]
pub(super) struct RuntimeImageLayoutWorldTransform {
    pub(super) world_transform: Mat2D,
    pub(super) render_scale_x: f32,
    pub(super) render_scale_y: f32,
}

pub(super) fn runtime_image_layout_local_transform(
    instance: &ArtboardInstance,
    local_id: usize,
    base_local_transform: Mat2D,
    layout_scale_separate: bool,
) -> Result<RuntimeImageLayoutTransform> {
    // `Image::draw` and bounds queries only read callback-settled owner state.
    // `controlSize`, `setMesh`, and `assetUpdated` are the sole
    // `updateImageScale` mutation boundaries in pinned C++.
    let authored_scale_x = property_key_for_name("Node", "scaleX")
        .and_then(|property_key| instance.objects.double_property(local_id, property_key))
        .unwrap_or(1.0);
    let authored_scale_y = property_key_for_name("Node", "scaleY")
        .and_then(|property_key| instance.objects.double_property(local_id, property_key))
        .unwrap_or(1.0);
    let owner = instance
        .runtime_images
        .owner(local_id)
        .context("Image occurrence is missing its direct owner")?;
    let owner = owner.borrow();
    debug_assert_eq!(owner.layout_scale_separate, layout_scale_separate);

    if owner.layout_scale_separate {
        return Ok(RuntimeImageLayoutTransform {
            local_transform: apply_layout_fit(
                base_local_transform,
                owner.layout_scale_x,
                owner.layout_scale_y,
                owner.layout_offset_x,
                owner.layout_offset_y,
                true,
            ),
            render_scale_x: authored_scale_x * owner.layout_scale_x,
            render_scale_y: authored_scale_y * owner.layout_scale_y,
        });
    }

    let public_scale_x = if owner.user_scale_x {
        authored_scale_x
    } else {
        owner.layout_scale_x
    };
    let public_scale_y = if owner.user_scale_y {
        authored_scale_y
    } else {
        owner.layout_scale_y
    };
    Ok(RuntimeImageLayoutTransform {
        local_transform: apply_layout_fit(
            base_local_transform,
            public_scale_x,
            public_scale_y,
            owner.layout_offset_x,
            owner.layout_offset_y,
            false,
        ),
        render_scale_x: public_scale_x,
        render_scale_y: public_scale_y,
    })
}

#[allow(clippy::too_many_arguments)]
fn runtime_image_layout_fit_values(
    image_width: f32,
    image_height: f32,
    layout_width: f32,
    layout_height: f32,
    has_vertex_mesh: bool,
    fit: u64,
    origin_x: f32,
    origin_y: f32,
    alignment_x: f32,
    alignment_y: f32,
) -> RuntimeImageLayoutFit {
    let width_scale = layout_width / image_width;
    let height_scale = layout_height / image_height;
    let (mut scale_x, mut scale_y) = match fit {
        1 => {
            let scale = width_scale.min(height_scale);
            (scale, scale)
        }
        2 => {
            let scale = width_scale.max(height_scale);
            (scale, scale)
        }
        3 => (width_scale, width_scale),
        4 => (height_scale, height_scale),
        5 => (1.0, 1.0),
        6 => {
            let scale = width_scale.min(height_scale);
            let scale = if scale < 1.0 { scale } else { 1.0 };
            (scale, scale)
        }
        0 | 7 => (width_scale, height_scale),
        _ => (width_scale, height_scale),
    };

    if fit != 5 && fit != 6 && (!scale_x.is_finite() || !scale_y.is_finite()) {
        scale_x = f32::NAN;
        scale_y = f32::NAN;
    }

    let (mut offset_x, mut offset_y) = (0.0, 0.0);
    if fit != 0 {
        let (bounds_origin_x, bounds_origin_y) = if has_vertex_mesh {
            (0.5, 0.5)
        } else {
            (origin_x, origin_y)
        };
        let bounds_left = -image_width * bounds_origin_x;
        let bounds_top = -image_height * bounds_origin_y;
        let x_align = (alignment_x + 1.0) * 0.5;
        let y_align = (alignment_y + 1.0) * 0.5;
        let scaled_left = bounds_left * scale_x;
        let scaled_top = bounds_top * scale_y;
        let width_remainder = layout_width - image_width * scale_x;
        let height_remainder = layout_height - image_height * scale_y;
        offset_x = -scaled_left + width_remainder * x_align;
        offset_y = -scaled_top + height_remainder * y_align;
    }

    RuntimeImageLayoutFit {
        scale_x,
        scale_y,
        offset_x,
        offset_y,
    }
}

#[allow(clippy::too_many_arguments)]
pub(super) fn runtime_draw_live_image(
    runtime: &RuntimeFile,
    instance: &ArtboardInstance,
    graph: &ArtboardGraph,
    drawable: &RuntimeDrawable,
    layout_bounds: Option<&BTreeMap<usize, RuntimeLayoutBounds>>,
    factory: &mut dyn RenderFactory,
    path_cache: &mut RuntimeArtboardPathState,
    renderer: &mut dyn Renderer,
) -> Result<()> {
    let local_id = drawable.local_id.context("live image missing local id")?;
    let resolved_image_asset_global =
        instance.resolved_image_asset_global(Some(local_id), drawable.resolved_image_asset_global);
    runtime_draw_image_with_owner(
        runtime,
        instance,
        graph,
        local_id,
        drawable.global_id,
        resolved_image_asset_global,
        drawable.needs_save_operation,
        layout_bounds,
        factory,
        path_cache,
        renderer,
    )
}

#[allow(clippy::too_many_arguments)]
pub(super) fn runtime_draw_image_with_owner(
    runtime: &RuntimeFile,
    instance: &ArtboardInstance,
    graph: &ArtboardGraph,
    local_id: usize,
    image_global_id: Option<u32>,
    resolved_image_asset_global: Option<u32>,
    needs_save_operation: bool,
    layout_bounds: Option<&BTreeMap<usize, RuntimeLayoutBounds>>,
    factory: &mut dyn RenderFactory,
    path_cache: &mut RuntimeArtboardPathState,
    renderer: &mut dyn Renderer,
) -> Result<()> {
    // Direct port of C++ `Image::draw`; the live path reads the retained
    // Image-to-Mesh pointer installed by the child owner's onAddedDirty.
    let image_mesh = instance.runtime_images.mesh(local_id);
    let image_object = image_global_id.and_then(|global_id| runtime.object(global_id as usize));
    let Some(image) = instance
        .image_render_overrides
        .get(&local_id)
        .and_then(|image| image.render_image())
        .or_else(|| {
            resolved_image_asset_global
                .and_then(|asset_global| instance.runtime_render_image(asset_global))
        })
    else {
        // C++ `Image::draw` returns before saving when the asset has no
        // decoded RenderImage, e.g. hosted images with no loader.
        return Ok(());
    };

    if needs_save_operation {
        renderer.save();
    }

    if let Some(RuntimeImageMeshOwner::SliceMesh(slice_local)) = image_mesh {
        let details = graph
            .n_slicer_details
            .iter()
            .find(|details| details.local_id == slice_local)
            .with_context(|| format!("missing slice mesh owner for local {slice_local}"))?;
        if let Some(owner) = instance.runtime_meshes.slice(details.local_id) {
            let owner = owner.borrow();
            slice_mesh::runtime_draw_slice_mesh_image(
                runtime,
                instance,
                graph,
                local_id,
                image_object,
                resolved_image_asset_global,
                &owner,
                layout_bounds,
                image.as_ref(),
                path_cache,
                renderer,
            )?;
        }
        if needs_save_operation {
            renderer.restore();
        }
        return Ok(());
    }

    if let Some(RuntimeImageMeshOwner::Mesh(mesh_local)) = image_mesh {
        let mesh = graph
            .meshes
            .iter()
            .find(|mesh| mesh.local_id == mesh_local)
            .with_context(|| format!("missing mesh owner for local {mesh_local}"))?;
        let mesh_component = instance
            .component_handle(mesh.local_id)
            .with_context(|| format!("live Mesh local {} has no Component owner", mesh.local_id))?;
        let owner = instance
            .runtime_meshes
            .mesh(mesh.local_id)
            .with_context(|| format!("missing mesh owner for local {}", mesh.local_id))?;
        let backend_context_id = instance
            .runtime_image_backend_context_id()
            .context("mesh image owner is missing its file backend context")?;
        mesh::runtime_draw_mesh_image(
            runtime,
            instance,
            graph,
            mesh_component,
            local_id,
            image_object,
            resolved_image_asset_global,
            mesh,
            owner,
            layout_bounds,
            image.as_ref(),
            backend_context_id,
            factory,
            path_cache,
            renderer,
        )?;
        if needs_save_operation {
            renderer.restore();
        }
        return Ok(());
    }

    let origin_x_key =
        runtime_draw_property_key_for_name("Image", "originX").context("missing Image.originX")?;
    let origin_y_key =
        runtime_draw_property_key_for_name("Image", "originY").context("missing Image.originY")?;
    let origin_x = instance
        .double_property(local_id, origin_x_key)
        .or_else(|| {
            image_object.and_then(|object| {
                runtime_object_explicit_double_property_by_key(object, origin_x_key)
            })
        })
        .unwrap_or(0.5);
    let origin_y = instance
        .double_property(local_id, origin_y_key)
        .or_else(|| {
            image_object.and_then(|object| {
                runtime_object_explicit_double_property_by_key(object, origin_y_key)
            })
        })
        .unwrap_or(0.5);
    let world = path_cache
        .image_world_transform_with_bounds(runtime, instance, graph, local_id, layout_bounds)?
        .unwrap_or_else(|| {
            path_cache.component_world_transform_with_bounds(
                instance,
                graph,
                local_id,
                layout_bounds,
            )
        });
    renderer.transform(runtime_render_mat(world));

    renderer.transform(RenderMat2D([
        1.0,
        0.0,
        0.0,
        1.0,
        -(image.width() as f32 * origin_x),
        -(image.height() as f32 * origin_y),
    ]));

    let blend_mode_key = runtime_draw_property_key_for_name("Drawable", "blendModeValue")
        .context("missing Drawable.blendModeValue")?;
    let blend_mode_value = instance
        .uint_property(local_id, blend_mode_key)
        .or_else(|| {
            image_object.and_then(|object| {
                runtime_object_explicit_uint_property_by_key(object, blend_mode_key)
            })
        })
        .unwrap_or(3);
    let opacity = instance
        .component(local_id)
        .map(|component| component.transform.render_opacity)
        .unwrap_or(1.0);
    renderer.draw_image(
        Some(image.as_ref()),
        RenderImageSampler::LINEAR_CLAMP,
        runtime_blend_mode(u32::try_from(blend_mode_value).unwrap_or(3))?,
        opacity,
    );

    if needs_save_operation {
        renderer.restore();
    }
    Ok(())
}

impl ArtboardInstance {
    /// Direct `LayoutComponent::propagateSizeToChildren -> Image::controlSize`.
    /// Scale/offset changes publish local Transform dirt immediately; merely
    /// retaining a new control size does not dirty an unchanged transform.
    pub(crate) fn control_runtime_layout_images(
        &mut self,
        graph: &ArtboardGraph,
        layout_bounds: &BTreeMap<usize, RuntimeLayoutBounds>,
    ) -> bool {
        let controls = graph
            .components
            .iter()
            .filter(|component| component.type_name == "Image")
            .filter_map(|component| {
                let parent_local = component.parent_local?;
                graph.components.iter().find(|parent| {
                    parent.local_id == parent_local && parent.type_name == "LayoutComponent"
                })?;
                let bounds = layout_bounds.get(&parent_local)?;
                Some((component.local_id, bounds.width, bounds.height))
            })
            .collect::<Vec<_>>();
        controls
            .into_iter()
            .fold(false, |changed, (local_id, width, height)| {
                let transform_changed = self
                    .runtime_images
                    .control_size(local_id, width, height)
                    .unwrap_or(false);
                (transform_changed && self.add_dirt(local_id, ComponentDirt::TRANSFORM, true))
                    | changed
            })
    }
}

impl RuntimeArtboardPathState {
    pub(super) fn image_layout_world_transform_with_bounds(
        &mut self,
        runtime: &RuntimeFile,
        instance: &ArtboardInstance,
        graph: &ArtboardGraph,
        local_id: usize,
        layout_bounds: Option<&BTreeMap<usize, RuntimeLayoutBounds>>,
    ) -> Result<Option<image::RuntimeImageLayoutWorldTransform>> {
        let Some(layout_bounds) = layout_bounds else {
            return Ok(None);
        };
        let Some(component) = instance.component(local_id) else {
            return Ok(None);
        };
        let Some(parent_local) = instance.component_parent_local(local_id) else {
            return Ok(None);
        };
        if !instance
            .component(parent_local)
            .is_some_and(|parent| parent.type_name == "LayoutComponent")
        {
            return Ok(None);
        }
        let Some(_) = layout_bounds.get(&parent_local) else {
            return Ok(None);
        };

        let layout_scale_separate = image::runtime_layout_image_uses_separate_fit_scale(
            runtime.header.major_version,
            runtime.header.minor_version,
        );
        let state = image::runtime_image_layout_local_transform(
            instance,
            local_id,
            component.transform.local_transform,
            layout_scale_separate,
        )?;

        let parent_world = self.component_world_transform_with_bounds(
            instance,
            graph,
            parent_local,
            Some(layout_bounds),
        );
        Ok(Some(image::RuntimeImageLayoutWorldTransform {
            world_transform: parent_world.multiply(state.local_transform),
            render_scale_x: state.render_scale_x,
            render_scale_y: state.render_scale_y,
        }))
    }

    pub(super) fn image_world_transform_with_bounds(
        &mut self,
        runtime: &RuntimeFile,
        instance: &ArtboardInstance,
        graph: &ArtboardGraph,
        local_id: usize,
        layout_bounds: Option<&BTreeMap<usize, RuntimeLayoutBounds>>,
    ) -> Result<Option<Mat2D>> {
        Ok(self
            .image_layout_world_transform_with_bounds(
                runtime,
                instance,
                graph,
                local_id,
                layout_bounds,
            )?
            .map(|state| state.world_transform))
    }
}
