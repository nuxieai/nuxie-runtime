use std::{cell::RefCell, rc::Rc};
pub trait RenderImage {}
pub trait Font {}
pub trait BlobAsset {}
pub struct ImageAsset {
    value: RefCell<Option<Rc<dyn RenderImage>>>,
}
pub struct FontAsset {
    value: RefCell<Option<Rc<dyn Font>>>,
}
pub struct BindablePropertyAsset {
    file_asset: Rc<ImageAsset>,
    font_asset: Rc<FontAsset>,
    blob_asset: Option<Rc<dyn BlobAsset>>,
}
impl Default for BindablePropertyAsset {
    fn default() -> Self {
        Self {
            file_asset: Rc::new(ImageAsset {
                value: RefCell::new(None),
            }),
            font_asset: Rc::new(FontAsset {
                value: RefCell::new(None),
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
        *self.file_asset.value.borrow_mut() = image
    }
    pub fn image_value(&self) -> Option<Rc<dyn RenderImage>> {
        self.file_asset.value.borrow().clone()
    }
    pub fn font_file_asset(&self) -> Rc<FontAsset> {
        self.font_asset.clone()
    }
    pub fn set_font_value(&self, font: Option<Rc<dyn Font>>) {
        *self.font_asset.value.borrow_mut() = font
    }
    pub fn font_value(&self) -> Option<Rc<dyn Font>> {
        self.font_asset.value.borrow().clone()
    }
    pub fn blob_file_asset(&self) -> Option<Rc<dyn BlobAsset>> {
        self.blob_asset.clone()
    }
    pub fn set_blob_value(&mut self, blob: Option<Rc<dyn BlobAsset>>) {
        self.blob_asset = blob
    }
    pub fn blob_value(&self) -> Option<&dyn BlobAsset> {
        self.blob_asset.as_deref()
    }
}
