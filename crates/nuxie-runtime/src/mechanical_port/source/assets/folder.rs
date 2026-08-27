use crate::mechanical_port::source::generated::assets::folder_base::FolderBase;

pub struct Folder {
    pub base: FolderBase,
}

impl Default for Folder {
    fn default() -> Self {
        Self {
            base: FolderBase::default(),
        }
    }
}
