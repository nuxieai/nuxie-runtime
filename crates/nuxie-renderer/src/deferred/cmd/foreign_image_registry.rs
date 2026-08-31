//! renderer/cmd/foreign_image_registry.hpp at e949498e.
use super::handle_flags::{HANDLE_FOREIGN_FLAG, HANDLE_FOREIGN_MASK};
use nuxie_render_api::RenderImage;
use std::{collections::HashMap, rc::Rc};

#[derive(Default)]
pub struct ForeignImageRegistry {
    ids: HashMap<usize, u32>,
    images: Vec<Rc<dyn RenderImage>>,
}
impl ForeignImageRegistry {
    pub fn image_draw_id(&mut self, image: &dyn RenderImage) -> u32 {
        let key = image.image_identity();
        let id = *self.ids.entry(key).or_insert_with(|| {
            let id = self.images.len();
            assert!(id <= HANDLE_FOREIGN_MASK as usize);
            self.images.push(image.retain_image());
            id as u32
        });
        HANDLE_FOREIGN_FLAG | id
    }
    pub fn image_at(&self, id: u32) -> Option<&Rc<dyn RenderImage>> {
        self.images.get(id as usize)
    }
    pub fn images(&self) -> &[Rc<dyn RenderImage>] {
        &self.images
    }
    pub fn reset(&mut self) {
        self.ids.clear();
        self.images.clear();
    }
}
