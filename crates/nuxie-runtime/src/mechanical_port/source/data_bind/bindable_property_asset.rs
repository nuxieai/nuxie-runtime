use std::{cell::RefCell, rc::Rc};

use crate::mechanical_port::source::generated::data_bind::bindable_property_asset_base::BindablePropertyAssetBase;

pub trait AssetReferencer {
    fn asset_updated(&mut self);
    fn add_text_shape_dirt(&mut self);
}

pub trait RenderImage {
    fn set_delegate_to_image_asset(&self, _asset: *const ImageAsset) {}
}

pub trait Font {}
pub trait BlobAsset {}

pub struct ImageAsset {
    value: RefCell<Option<Rc<dyn RenderImage>>>,
    referencers: RefCell<Vec<*mut dyn AssetReferencer>>,
}

pub struct FontAsset {
    value: RefCell<Option<Rc<dyn Font>>>,
    referencers: RefCell<Vec<*mut dyn AssetReferencer>>,
}

pub struct BindablePropertyAsset {
    pub base: BindablePropertyAssetBase,
    file_asset: Rc<ImageAsset>,
    font_asset: Rc<FontAsset>,
    blob_asset: Option<Rc<dyn BlobAsset>>,
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
            image.set_delegate_to_image_asset(Rc::as_ptr(&self.file_asset));
        }
        for referencer in self.file_asset.referencers.borrow().iter().copied() {
            unsafe { (&mut *referencer).asset_updated() };
        }
    }

    pub fn image_value(&self) -> Option<*const dyn RenderImage> {
        self.file_asset.value.borrow().as_ref().map(Rc::as_ptr)
    }

    pub fn font_file_asset(&self) -> Rc<FontAsset> {
        self.font_asset.clone()
    }

    pub fn set_font_value(&self, font: Option<Rc<dyn Font>>) {
        *self.font_asset.value.borrow_mut() = font;
        for referencer in self.font_asset.referencers.borrow().iter().copied() {
            unsafe { (&mut *referencer).add_text_shape_dirt() };
        }
    }

    pub fn font_value(&self) -> Option<*const dyn Font> {
        self.font_asset.value.borrow().as_ref().map(Rc::as_ptr)
    }

    pub fn blob_file_asset(&self) -> Option<Rc<dyn BlobAsset>> {
        self.blob_asset.clone()
    }

    pub fn set_blob_value(&mut self, blob: Option<Rc<dyn BlobAsset>>) {
        self.blob_asset = blob;
    }

    pub fn blob_value(&self) -> Option<*const dyn BlobAsset> {
        self.blob_asset.as_ref().map(Rc::as_ptr)
    }
}

impl ImageAsset {
    pub fn add_referencer(&self, referencer: *mut dyn AssetReferencer) {
        self.referencers.borrow_mut().push(referencer);
    }
}

impl FontAsset {
    pub fn add_referencer(&self, referencer: *mut dyn AssetReferencer) {
        self.referencers.borrow_mut().push(referencer);
    }
}
