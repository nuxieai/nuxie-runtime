use crate::ArtboardInstance;
use nuxie_binary::{RuntimeFile, RuntimeFileAssetContents, RuntimeObject};
use nuxie_image_codec::{decoded_rgba_len, preflight_encoded_image};
use nuxie_render_api::{Factory as RenderFactory, ImageDecodeError, RenderImage};
use std::cell::RefCell;
use std::collections::{BTreeMap, BTreeSet};
use std::rc::{Rc, Weak};
use std::sync::Arc;

use super::mesh::RuntimeMeshSharedRenderBuffers;

/// File-owned C++ `ImageAsset` occurrences and their `m_RenderImage` members.
///
/// Dense global-id addressing is only the runtime arena representation. Each
/// slot is a concrete owner; artboard occurrences retain the shared list and
/// Images borrow the exact asset-owned RenderImage.
pub struct RuntimeImageAssetOwners {
    backend_context_id: u64,
    state: RefCell<RuntimeImageAssetOwnerState>,
}

/// Host-side C++ `FileAssetLoader` equivalent for an `ImageAsset` import.
///
/// Returning `true` accepts responsibility for the asset and suppresses the
/// in-band fallback decode, exactly like `FileAssetImporter::resolve`.
pub trait RuntimeImageAssetLoader {
    fn load_contents(
        &mut self,
        asset: &RuntimeObject,
        in_band: &[u8],
        factory: &mut dyn RenderFactory,
    ) -> bool;
}

impl<F> RuntimeImageAssetLoader for F
where
    F: FnMut(&RuntimeObject, &[u8], &mut dyn RenderFactory) -> bool,
{
    fn load_contents(
        &mut self,
        asset: &RuntimeObject,
        in_band: &[u8],
        factory: &mut dyn RenderFactory,
    ) -> bool {
        self(asset, in_band, factory)
    }
}

pub(super) struct RuntimeImageAssetCatalog<'a> {
    pub(super) globals: Vec<u32>,
    embedded_by_global: BTreeMap<u32, &'a [u8]>,
}

impl<'a> RuntimeImageAssetCatalog<'a> {
    pub(super) fn from_runtime(runtime: &'a RuntimeFile) -> Self {
        Self::from_entries(runtime.imported_file_assets_with_contents())
    }

    pub(super) fn from_entries(
        entries: impl IntoIterator<Item = RuntimeFileAssetContents<'a>>,
    ) -> Self {
        let mut globals = Vec::new();
        let mut embedded_by_global = BTreeMap::new();
        for entry in entries {
            if entry.asset.type_name != "ImageAsset" {
                continue;
            }
            globals.push(entry.asset.id);
            if let Some(contents) = entry.contents {
                embedded_by_global.insert(entry.asset.id, contents);
            }
        }
        Self {
            globals,
            embedded_by_global,
        }
    }

    pub(super) fn embedded_bytes(&self, asset_global: u32) -> Option<&'a [u8]> {
        self.embedded_by_global.get(&asset_global).copied()
    }
}

#[derive(Default)]
struct RuntimeImageAssetOwnerState {
    owners_by_global: Vec<Option<RuntimeImageAssetOwner>>,
    import_resolved_globals: BTreeSet<u32>,
    /// Source-artboard Mesh members retained by the file. Concrete clones
    /// copy the UV/index reference-counted handles and allocate only a fresh
    /// dynamic vertex buffer, matching `Mesh::clone`.
    source_meshes: BTreeMap<(u32, usize), Rc<RefCell<RuntimeMeshSharedRenderBuffers>>>,
    retained_decoded_bytes: usize,
    // `None` is pinned-C++ behavior. A bound is an explicit host import
    // policy applied before decode by the high-level API.
    max_retained_decoded_bytes: Option<usize>,
    referencers: Vec<Weak<RuntimeImageAssetReferencerQueue>>,
}

struct RuntimeImageAssetOwner {
    global_id: u32,
    render_image: Option<Rc<dyn RenderImage>>,
    decoded_byte_length: usize,
}

#[derive(Debug, Default)]
pub(crate) struct RuntimeImageAssetReferencerQueue {
    images: RefCell<Vec<(usize, Weak<RefCell<super::image::RuntimeImageOwner>>)>>,
    pending_locals: RefCell<BTreeSet<usize>>,
}

impl RuntimeImageAssetReferencerQueue {
    pub(super) fn replace_images(
        &self,
        images: impl IntoIterator<Item = (usize, Weak<RefCell<super::image::RuntimeImageOwner>>)>,
    ) {
        *self.images.borrow_mut() = images.into_iter().collect();
    }

    fn publish(&self, global_id: u32, width: u32, height: u32) {
        self.images.borrow_mut().retain(|(local_id, image)| {
            let Some(image) = image.upgrade() else {
                return false;
            };
            // `ImageAsset::renderImage` invokes `Image::assetUpdated` inline.
            // Settle the direct Image owner before returning from replacement;
            // only dependency-graph dirt publication waits for the next
            // mutable Artboard update boundary.
            if image.borrow_mut().asset_updated(global_id, width, height) {
                self.pending_locals.borrow_mut().insert(*local_id);
            }
            true
        });
    }

    fn take(&self) -> BTreeSet<usize> {
        std::mem::take(&mut *self.pending_locals.borrow_mut())
    }

    #[cfg(test)]
    pub(crate) fn is_pending(&self) -> bool {
        !self.pending_locals.borrow().is_empty()
    }

    pub(crate) fn clear(&self) {
        self.pending_locals.borrow_mut().clear();
        self.images.borrow_mut().clear();
    }
}

impl Default for RuntimeImageAssetOwners {
    fn default() -> Self {
        Self {
            backend_context_id: super::next_render_backend_context_id(),
            state: RefCell::new(RuntimeImageAssetOwnerState::default()),
        }
    }
}

impl std::fmt::Debug for RuntimeImageAssetOwners {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let state = self.state.borrow();
        formatter
            .debug_struct("RuntimeImageAssetOwners")
            .field(
                "materialized",
                &state.owners_by_global.iter().flatten().count(),
            )
            .field("retained_decoded_bytes", &state.retained_decoded_bytes)
            .finish()
    }
}

impl RuntimeImageAssetOwners {
    /// Constructs an owner list with the caller's aggregate decoded-byte
    /// admission policy. Compressed-byte limits cannot substitute for this
    /// post-header, pre-decode reservation.
    pub fn with_max_retained_decoded_bytes(max_retained_decoded_bytes: Option<usize>) -> Self {
        Self {
            backend_context_id: super::next_render_backend_context_id(),
            state: RefCell::new(RuntimeImageAssetOwnerState {
                max_retained_decoded_bytes,
                ..RuntimeImageAssetOwnerState::default()
            }),
        }
    }

    pub fn get(&self, global_id: u32) -> Option<Rc<dyn RenderImage>> {
        self.state
            .borrow()
            .owners_by_global
            .get(global_id as usize)
            .and_then(|owner| owner.as_ref())
            .and_then(|owner| owner.render_image.as_ref())
            .map(Rc::clone)
    }

    pub(crate) fn mark_import_resolved(&self, global_id: u32) {
        self.state
            .borrow_mut()
            .import_resolved_globals
            .insert(global_id);
    }

    fn is_import_resolved(&self, global_id: u32) -> bool {
        self.state
            .borrow()
            .import_resolved_globals
            .contains(&global_id)
    }

    /// Decode and atomically replace one concrete ImageAsset owner.
    pub fn decode(
        &self,
        global_id: u32,
        bytes: &[u8],
        factory: &mut dyn RenderFactory,
    ) -> Result<(), ImageDecodeError> {
        let dimensions = preflight_encoded_image(bytes).ok_or(ImageDecodeError)?;
        let decoded_byte_length =
            decoded_rgba_len(dimensions.width, dimensions.height).ok_or(ImageDecodeError)?;
        let previous_retained_decoded_bytes = self
            .try_reserve_replacement_decoded_bytes(global_id, decoded_byte_length)
            .ok_or(ImageDecodeError)?;
        let image = match factory.decode_image(bytes) {
            Ok(image) => image,
            Err(error) => {
                self.cancel_decoded_byte_reservation(previous_retained_decoded_bytes);
                return Err(error);
            }
        };
        self.insert_reserved(global_id, image, decoded_byte_length);
        Ok(())
    }

    /// Imports one runtime `ImageAsset` through a host loader. This is the
    /// production-facing path used to exercise C++ loader accept/reject
    /// semantics without constructing a renderer cache first.
    pub fn load_asset_with_loader(
        &self,
        runtime: &RuntimeFile,
        asset_global: u32,
        factory: &mut dyn RenderFactory,
        loader: &mut dyn RuntimeImageAssetLoader,
    ) -> Result<(), ImageDecodeError> {
        let catalog = RuntimeImageAssetCatalog::from_runtime(runtime);
        predecode_render_image_with_loader(
            runtime,
            &catalog,
            asset_global,
            None,
            factory,
            self,
            Some(loader),
        )
    }

    pub(crate) fn backend_context_id(&self) -> u64 {
        self.backend_context_id
    }

    pub(crate) fn register_referencer(&self, referencer: &Rc<RuntimeImageAssetReferencerQueue>) {
        let mut state = self.state.borrow_mut();
        state
            .referencers
            .retain(|candidate| candidate.strong_count() != 0);
        if state
            .referencers
            .iter()
            .any(|candidate| candidate.ptr_eq(&Rc::downgrade(referencer)))
        {
            return;
        }
        state.referencers.push(Rc::downgrade(referencer));
    }

    pub fn insert(&self, global_id: u32, image: Box<dyn RenderImage>) {
        let Some(decoded_byte_length) = decoded_rgba_len(image.width(), image.height()) else {
            return;
        };
        if self
            .try_reserve_replacement_decoded_bytes(global_id, decoded_byte_length)
            .is_none()
        {
            return;
        }
        self.insert_reserved(global_id, image, decoded_byte_length);
    }

    pub(crate) fn try_reserve_replacement_decoded_bytes(
        &self,
        global_id: u32,
        decoded_byte_length: usize,
    ) -> Option<usize> {
        let mut state = self.state.borrow_mut();
        let replaced_decoded_byte_length = state
            .owners_by_global
            .get(global_id as usize)
            .and_then(|owner| owner.as_ref())
            .map_or(0, |owner| owner.decoded_byte_length);
        let next_retained_decoded_bytes = replacement_decoded_bytes(
            state.retained_decoded_bytes,
            replaced_decoded_byte_length,
            decoded_byte_length,
        )?;
        if state
            .max_retained_decoded_bytes
            .is_some_and(|budget| next_retained_decoded_bytes > budget)
        {
            return None;
        }
        let previous_retained_decoded_bytes = state.retained_decoded_bytes;
        state.retained_decoded_bytes = next_retained_decoded_bytes;
        Some(previous_retained_decoded_bytes)
    }

    pub(crate) fn cancel_decoded_byte_reservation(&self, previous_retained_decoded_bytes: usize) {
        self.state.borrow_mut().retained_decoded_bytes = previous_retained_decoded_bytes;
    }

    pub(crate) fn insert_reserved(
        &self,
        global_id: u32,
        image: Box<dyn RenderImage>,
        decoded_byte_length: usize,
    ) {
        let width = image.width();
        let height = image.height();
        let mut state = self.state.borrow_mut();
        let slot = global_id as usize;
        if state.owners_by_global.len() <= slot {
            state.owners_by_global.resize_with(slot + 1, || None);
        }
        state.owners_by_global[slot] = Some(RuntimeImageAssetOwner {
            global_id,
            render_image: Some(Rc::from(image)),
            decoded_byte_length,
        });
        state.referencers.retain(|referencer| {
            let Some(referencer) = referencer.upgrade() else {
                return false;
            };
            // Direct `ImageAsset::renderImage`: publish to every registered
            // FileAssetReferencer synchronously with owner replacement.
            referencer.publish(global_id, width, height);
            true
        });
    }

    pub(crate) fn dimensions(&self) -> std::vec::IntoIter<(u32, u32, u32, usize)> {
        self.state
            .borrow()
            .owners_by_global
            .iter()
            .filter_map(|owner| {
                let owner = owner.as_ref()?;
                let image = owner.render_image.as_ref()?;
                Some((
                    owner.global_id,
                    image.width(),
                    image.height(),
                    render_image_identity(image.as_ref()),
                ))
            })
            .collect::<Vec<_>>()
            .into_iter()
    }

    pub(crate) fn insert_source_mesh(
        &self,
        graph_global_id: u32,
        local_id: usize,
        source: Rc<RefCell<RuntimeMeshSharedRenderBuffers>>,
    ) {
        self.state
            .borrow_mut()
            .source_meshes
            .insert((graph_global_id, local_id), source);
    }

    pub(crate) fn source_mesh(
        &self,
        graph_global_id: u32,
        local_id: usize,
    ) -> Option<Rc<RefCell<RuntimeMeshSharedRenderBuffers>>> {
        self.state
            .borrow()
            .source_meshes
            .get(&(graph_global_id, local_id))
            .map(Rc::clone)
    }

    #[cfg(test)]
    pub(crate) fn retained_decoded_bytes(&self) -> usize {
        self.state.borrow().retained_decoded_bytes
    }
}

pub(super) fn predecode_render_image(
    runtime: &RuntimeFile,
    image_assets: &RuntimeImageAssetCatalog<'_>,
    asset_global: u32,
    external_images: Option<&BTreeMap<u32, Arc<[u8]>>>,
    factory: &mut dyn RenderFactory,
    images: &RuntimeImageAssetOwners,
) -> Result<(), ImageDecodeError> {
    if images.is_import_resolved(asset_global) {
        return Ok(());
    }
    predecode_render_image_with_loader(
        runtime,
        image_assets,
        asset_global,
        external_images,
        factory,
        images,
        None,
    )
}

pub(super) fn predecode_render_image_with_loader(
    runtime: &RuntimeFile,
    image_assets: &RuntimeImageAssetCatalog<'_>,
    asset_global: u32,
    external_images: Option<&BTreeMap<u32, Arc<[u8]>>>,
    factory: &mut dyn RenderFactory,
    images: &RuntimeImageAssetOwners,
    mut loader: Option<&mut dyn RuntimeImageAssetLoader>,
) -> Result<(), ImageDecodeError> {
    let is_import = loader.is_some();
    let embedded = image_assets.embedded_bytes(asset_global);
    if let (Some(loader), Some(asset)) = (loader.as_mut(), runtime.object(asset_global as usize))
        && loader.load_contents(asset, embedded.unwrap_or_default(), factory)
    {
        // Direct `FileAssetImporter::resolve`: a true result transfers loading
        // responsibility to the host and suppresses in-band fallback decode.
        images.mark_import_resolved(asset_global);
        return Ok(());
    }
    let bytes = embedded.or_else(|| {
        let semantic_id = runtime
            .object(asset_global as usize)?
            .uint_property("assetId")?;
        let semantic_id = u32::try_from(semantic_id).ok()?;
        external_images?.get(&semantic_id).map(AsRef::as_ref)
    });
    if let Some(bytes) = bytes {
        images.decode(asset_global, bytes, factory)?;
    }
    if is_import {
        images.mark_import_resolved(asset_global);
    }
    Ok(())
}

impl ArtboardInstance {
    /// Settles C++ `ImageAsset::renderImage -> Image::assetUpdated`: every
    /// referencing Image has already recomputed its direct scale state inline;
    /// publish the queued exact WorldTransform dirt into the dependency graph.
    pub(crate) fn settle_runtime_image_asset_updates(&mut self) -> bool {
        self.runtime_image_asset_referencer
            .take()
            .into_iter()
            .fold(false, |changed, local_id| {
                self.add_dirt(local_id, crate::ComponentDirt::WORLD_TRANSFORM, true) | changed
            })
    }
}

pub(crate) fn render_image_identity(image: &dyn RenderImage) -> usize {
    (image as *const dyn RenderImage as *const ()) as usize
}

/// Computes the decoded-byte total for replacing one retained ImageAsset.
/// Reservation happens before factory decode and is rolled back on failure.
fn replacement_decoded_bytes(
    retained: usize,
    replaced: usize,
    replacement: usize,
) -> Option<usize> {
    retained.checked_sub(replaced)?.checked_add(replacement)
}
