use std::sync::Arc;

use crate::mechanical_port::source::artboard::ArtboardInstance;
use crate::mechanical_port::source::file::File;

pub struct BindableArtboard {
    file: Arc<File>,
    artboard: Box<ArtboardInstance>,
}

impl BindableArtboard {
    pub fn new(file: Arc<File>, artboard: Box<ArtboardInstance>) -> Self {
        Self { file, artboard }
    }

    pub fn artboard(&mut self) -> &mut ArtboardInstance {
        &mut self.artboard
    }
}
