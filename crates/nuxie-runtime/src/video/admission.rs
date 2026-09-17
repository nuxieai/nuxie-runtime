//! Import admission runs before invoking loaders or decoders. Tooling can
//! import video definitions without playback; presentation hosts must supply
//! the capabilities of the decoder AND renderer they actually initialized.
use super::{Video, VideoAsset};
use crate::source::{
    core::CoreHandle,
    file::{ImportAdmission, ImportAdmissionRef},
};
use std::{cell::Cell, rc::Rc};

pub struct VideoAdmission {
    pub playback_available: bool,
    pub max_embedded_bytes: usize,
    rejected: Cell<bool>,
    parent: Option<ImportAdmissionRef>,
}
impl VideoAdmission {
    pub fn new(
        playback_available: bool,
        max_embedded_bytes: usize,
        parent: Option<ImportAdmissionRef>,
    ) -> Rc<Self> {
        Rc::new(Self {
            playback_available,
            max_embedded_bytes,
            rejected: Cell::new(false),
            parent,
        })
    }
    fn accept(&self, accepted: bool) -> bool {
        if !accepted {
            self.rejected.set(true);
        }
        accepted
    }
}
impl ImportAdmission for VideoAdmission {
    fn admit_object(&self, object: &CoreHandle) -> bool {
        self.accept(
            (self.playback_available
                || !(object.is_type_of(Video::TYPE_KEY)
                    || object.is_type_of(VideoAsset::TYPE_KEY)))
                && self.parent.as_ref().is_none_or(|p| p.admit_object(object)),
        )
    }
    fn admit_asset_bytes(&self, asset: &CoreHandle, bytes: &[u8]) -> bool {
        self.accept(
            (!asset.is_type_of(VideoAsset::TYPE_KEY) || bytes.len() <= self.max_embedded_bytes)
                && self
                    .parent
                    .as_ref()
                    .is_none_or(|p| p.admit_asset_bytes(asset, bytes)),
        )
    }
    fn admit_loaded_asset(&self, asset: &CoreHandle) -> bool {
        self.accept(
            self.parent
                .as_ref()
                .is_none_or(|p| p.admit_loaded_asset(asset)),
        )
    }
    fn is_rejected(&self) -> bool {
        self.rejected.get() || self.parent.as_ref().is_some_and(|p| p.is_rejected())
    }
}
