use super::slice_mesh::RuntimeSliceMeshOwner;
use super::*;
use crate::ArtboardInstance;
use std::cell::RefCell;

// The backend-late realization belongs to SliceMesh. Re-export the live
// bridge here because the protected draw owner still calls it through the
// historical NSlicer module boundary.
pub(super) use super::slice_mesh::runtime_prepare_slice_meshes;

/// Direct `NSlicer::NSlicer`: each occurrence uniquely owns one SliceMesh.
pub(crate) fn new_slice_mesh(local_id: usize) -> RefCell<RuntimeSliceMeshOwner> {
    RefCell::new(RuntimeSliceMeshOwner::new(local_id))
}

/// Direct `NSlicer::image`: resolve the live parent and downcast it to Image.
pub(crate) fn image_parent(instance: &ArtboardInstance, local_id: usize) -> Option<usize> {
    instance
        .component_parent_local(local_id)
        .filter(|parent_local| {
            instance
                .component(*parent_local)
                .is_some_and(|component| component.type_name == "Image")
        })
}

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

/// Direct `NSlicer::axisChanged`.
pub(crate) fn axis_changed(instance: &mut ArtboardInstance, local_id: usize) -> bool {
    instance.add_dirt(local_id, ComponentDirt::N_SLICER, false)
}

/// Direct `NSlicer::update` dirt gate. SliceMesh realization is retained on
/// the occurrence and performed by `runtime_prepare_slice_meshes` once Rust's
/// factory-late render image and backend context are available.
pub(crate) fn update(owner: Option<&RefCell<RuntimeSliceMeshOwner>>, value: ComponentDirt) {
    if !(value & (ComponentDirt::N_SLICER | ComponentDirt::WORLD_TRANSFORM)).is_empty()
        && let Some(owner) = owner
    {
        owner.borrow_mut().dirty = true;
    }
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
