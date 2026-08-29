use std::{
    cell::RefCell,
    rc::{Rc, Weak},
    sync::Arc,
};

use crate::RuntimeBlobAsset;
use crate::mechanical_port::source::{
    generated::data_bind::bindable_property_asset_base::BindablePropertyAssetBase,
    text_engine::FontRef,
};
use nuxie_render_api::RenderImage;

pub trait AssetReferencer {
    fn asset_updated(&mut self);
    fn add_text_shape_dirt(&mut self);
}

pub struct ImageAsset {
    value: RefCell<Option<Rc<dyn RenderImage>>>,
    referencers: RefCell<Vec<Weak<RefCell<dyn AssetReferencer>>>>,
}

pub struct FontAsset {
    value: RefCell<Option<FontRef>>,
    referencers: RefCell<Vec<Weak<RefCell<dyn AssetReferencer>>>>,
}

pub struct BindablePropertyAsset {
    pub base: BindablePropertyAssetBase,
    file_asset: Rc<ImageAsset>,
    font_asset: Rc<FontAsset>,
    blob_asset: Option<Arc<RuntimeBlobAsset>>,
}

impl Default for BindablePropertyAsset {
    fn default() -> Self {
        Self {
            base: BindablePropertyAssetBase::default(),
            file_asset: Rc::new(ImageAsset {
                value: RefCell::new(None),
                referencers: RefCell::new(Vec::new()),
            }),
            font_asset: Rc::new(FontAsset {
                value: RefCell::new(None),
                referencers: RefCell::new(Vec::new()),
            }),
            blob_asset: None,
        }
    }
}

impl BindablePropertyAsset {
    pub const DEFAULT_VALUE: u32 = u32::MAX;

    pub fn file_asset(&self) -> Rc<ImageAsset> {
        self.file_asset.clone()
    }

    pub fn set_image_value(&self, image: Option<Rc<dyn RenderImage>>) {
        *self.file_asset.value.borrow_mut() = image;
        if let Some(image) = self.file_asset.value.borrow().as_ref() {
            let _ = image;
        }
        let referencers = self
            .file_asset
            .referencers
            .borrow()
            .iter()
            .filter_map(Weak::upgrade)
            .collect::<Vec<_>>();
        self.file_asset
            .referencers
            .borrow_mut()
            .retain(|referencer| referencer.strong_count() != 0);
        for referencer in referencers {
            referencer.borrow_mut().asset_updated();
        }
    }

    pub fn image_value(&self) -> Option<Rc<dyn RenderImage>> {
        self.file_asset.value.borrow().clone()
    }

    pub fn font_file_asset(&self) -> Rc<FontAsset> {
        self.font_asset.clone()
    }

    pub fn set_font_value(&self, font: Option<FontRef>) {
        *self.font_asset.value.borrow_mut() = font;
        let referencers = self
            .font_asset
            .referencers
            .borrow()
            .iter()
            .filter_map(Weak::upgrade)
            .collect::<Vec<_>>();
        self.font_asset
            .referencers
            .borrow_mut()
            .retain(|referencer| referencer.strong_count() != 0);
        for referencer in referencers {
            referencer.borrow_mut().add_text_shape_dirt();
        }
    }

    pub fn font_value(&self) -> Option<FontRef> {
        self.font_asset.value.borrow().clone()
    }

    pub fn blob_file_asset(&self) -> Option<Arc<RuntimeBlobAsset>> {
        self.blob_asset.clone()
    }

    pub fn set_blob_value(&mut self, blob: Option<Arc<RuntimeBlobAsset>>) {
        self.blob_asset = blob;
    }

    pub fn blob_value(&self) -> Option<Arc<RuntimeBlobAsset>> {
        self.blob_asset.clone()
    }
}

impl ImageAsset {
    pub fn add_referencer(&self, referencer: &Rc<RefCell<dyn AssetReferencer>>) {
        let referencer = Rc::downgrade(referencer);
        let mut referencers = self.referencers.borrow_mut();
        referencers.retain(|candidate| candidate.strong_count() != 0);
        if !referencers
            .iter()
            .any(|candidate| candidate.ptr_eq(&referencer))
        {
            referencers.push(referencer);
        }
    }
}

impl FontAsset {
    pub fn add_referencer(&self, referencer: &Rc<RefCell<dyn AssetReferencer>>) {
        let referencer = Rc::downgrade(referencer);
        let mut referencers = self.referencers.borrow_mut();
        referencers.retain(|candidate| candidate.strong_count() != 0);
        if !referencers
            .iter()
            .any(|candidate| candidate.ptr_eq(&referencer))
        {
            referencers.push(referencer);
        }
    }
}
