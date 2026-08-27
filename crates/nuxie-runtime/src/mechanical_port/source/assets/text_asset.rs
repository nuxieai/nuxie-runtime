use crate::mechanical_port::source::generated::assets::text_asset_base::TextAssetBase;

pub struct TextAsset {
    pub base: TextAssetBase,
    #[cfg(feature = "rive_scripting")]
    verified: bool,
}

impl Default for TextAsset {
    fn default() -> Self {
        Self {
            base: TextAssetBase::default(),
            #[cfg(feature = "rive_scripting")]
            verified: false,
        }
    }
}

impl TextAsset {
    #[cfg(feature = "rive_scripting")]
    pub fn verified(&self) -> bool {
        self.verified
    }

    #[cfg(feature = "rive_scripting")]
    pub(crate) fn set_verified(&mut self, verified: bool) {
        self.verified = verified;
    }
}
