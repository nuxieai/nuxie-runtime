use super::*;
use crate::ArtboardInstance;

/// Direct `NSlicer::onAddedDirty`: validate the Image parent and install the
/// NSlicer's uniquely owned SliceMesh through `Image::setMesh`.
pub(crate) fn on_added_dirty(
    images: &super::image::RuntimeImageList,
    local_id: usize,
    parent: Option<(usize, &'static str)>,
) -> Result<()> {
    let image_local = parent
        .filter(|(_, type_name)| *type_name == "Image")
        .map(|(local_id, _)| local_id)
        .context("NSlicer parent must be an Image")?;
    images
        .set_mesh(
            image_local,
            super::image::RuntimeImageMeshOwner::SliceMesh(local_id),
        )
        .context("NSlicer parent Image must retain a direct owner")?;
    Ok(())
}

/// Direct `NSlicer::validate` parent contract.
pub(crate) fn image_parent(instance: &ArtboardInstance, local_id: usize) -> Option<usize> {
    instance
        .component_parent_local(local_id)
        .filter(|parent_local| {
            instance
                .component(*parent_local)
                .is_some_and(|component| component.type_name == "Image")
        })
}

#[allow(clippy::too_many_arguments)]
pub(super) fn runtime_prepare_slice_meshes(
    runtime: &RuntimeFile,
    instance: &ArtboardInstance,
    graph: &ArtboardGraph,
    layout_bounds: Option<&BTreeMap<usize, RuntimeLayoutBounds>>,
    factory: &mut dyn RenderFactory,
    path_cache: &mut RuntimeArtboardPathState,
) -> Result<()> {
    let Some(backend_context_id) = instance.runtime_image_backend_context_id() else {
        return Ok(());
    };
    for details in graph.n_slicer_details.iter().filter(|details| {
        super::n_slicer_details::is_details(details.type_name) && details.type_name == "NSlicer"
    }) {
        let Some(image_local) = runtime_nslicer_image_local(instance, details) else {
            continue;
        };
        let authored_image_asset_global = instance
            .runtime_drawables
            .iter()
            .find(|drawable| {
                drawable.type_name == "Image" && drawable.local_id == Some(image_local)
            })
            .and_then(|drawable| drawable.resolved_image_asset_global);
        let resolved_image_asset_global =
            instance.resolved_image_asset_global(Some(image_local), authored_image_asset_global);
        let Some(image) = resolved_image_asset_global
            .and_then(|asset_global| instance.runtime_render_image(asset_global))
        else {
            continue;
        };
        let Some(owner) = instance.runtime_meshes.slice(details.local_id) else {
            continue;
        };
        let Some(registered_details) = instance.runtime_meshes.details(details.local_id) else {
            continue;
        };
        let needs_update = {
            let owner = owner.borrow();
            owner.dirty
                || owner.context_id != Some(backend_context_id)
                || owner.settled_update.is_none()
        };
        if !needs_update {
            continue;
        }
        let layout_state = path_cache.image_layout_world_transform_with_bounds(
            runtime,
            instance,
            graph,
            image_local,
            layout_bounds,
        )?;
        let render_scale_x = layout_state
            .map(|state| state.render_scale_x)
            .unwrap_or_else(|| {
                instance
                    .transform_property(image_local, TransformProperty::ScaleX)
                    .unwrap_or(1.0)
            });
        let render_scale_y = layout_state
            .map(|state| state.render_scale_y)
            .unwrap_or_else(|| {
                instance
                    .transform_property(image_local, TransformProperty::ScaleY)
                    .unwrap_or(1.0)
            });
        let mut owner = owner.borrow_mut();
        if owner.dirty || owner.settled_update.is_none() {
            let geometry = super::slice_mesh::runtime_slice_mesh_geometry(
                runtime,
                instance,
                registered_details,
                image.width() as f32,
                image.height() as f32,
                render_scale_x.abs(),
                render_scale_y.abs(),
            );
            owner.settled_update = Some(super::slice_mesh::runtime_slice_mesh_update(
                geometry,
                image.uv_transform(),
            ));
            owner.dirty = false;
        }
        super::slice_mesh::runtime_update_slice_mesh_render_buffers(
            factory,
            &mut owner,
            backend_context_id,
        );
    }
    Ok(())
}

pub(super) fn runtime_nslicer_image_local(
    instance: &ArtboardInstance,
    details: &NSlicerDetailsNode,
) -> Option<usize> {
    instance
        .component(details.local_id)
        .filter(|component| component.type_name == "NSlicer")?;
    image_parent(instance, details.local_id)
}
