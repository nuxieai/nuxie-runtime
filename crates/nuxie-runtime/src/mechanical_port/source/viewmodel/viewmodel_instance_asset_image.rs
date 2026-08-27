use crate::mechanical_port::source::{
    assets::image_asset::ImageAsset, component_dirt::ComponentDirt,
    data_bind::data_values::data_value_integer::DataValueInteger,
    generated::viewmodel::viewmodel_instance_asset_image_base::ViewModelInstanceAssetImageBase,
    refcnt::RiveRc, renderer::RenderImageRef,
};

pub struct ViewModelInstanceAssetImage {
    pub base: ViewModelInstanceAssetImageBase,
    image_asset: RiveRc<ImageAsset>,
}

impl Default for ViewModelInstanceAssetImage {
    fn default() -> Self {
        Self::new()
    }
}

impl ViewModelInstanceAssetImage {
    pub fn new() -> Self {
        Self {
            base: ViewModelInstanceAssetImageBase::default(),
            image_asset: RiveRc::new(ImageAsset::default()),
        }
    }

    pub fn property_value_changed(&mut self) {
        self.base.add_dirt(ComponentDirt::BINDINGS);
        #[cfg(feature = "rive_tools")]
        if let Some(callback) = self.base.changed_callback() {
            callback(self, self.base.property_value());
        }
        self.base.on_value_changed();
    }

    pub fn set_value(&mut self, image: Option<RenderImageRef>) {
        if self.image_asset.render_image().map(RenderImageRef::as_ptr)
            == image.as_ref().map(RenderImageRef::as_ptr)
        {
            self.base.set_property_value(u32::MAX);
            return;
        }
        #[cfg(feature = "rive_tools")]
        let already_sentinel = self.base.property_value() == u32::MAX;
        self.image_asset.set_render_image_direct(image);
        #[cfg(feature = "rive_tools")]
        if !already_sentinel {
            self.base.set_property_value(u32::MAX);
        } else if let Some(callback) = self.base.changed_callback() {
            callback(self, self.base.property_value());
        }
        #[cfg(not(feature = "rive_tools"))]
        self.base.set_property_value(u32::MAX);
        self.base.add_dirt(ComponentDirt::BINDINGS);
        self.base.on_value_changed();
    }

    pub fn asset(&self) -> RiveRc<ImageAsset> {
        self.image_asset.clone()
    }

    pub fn apply_value(&mut self, data_value: &DataValueInteger) {
        if let Some(asset_value) = data_value.as_asset_image() {
            let image = asset_value.image_value();
            self.set_value(image.clone());
            if image.is_some() {
                return;
            }
        }
        self.base.set_property_value(data_value.value());
    }

    pub fn clone_value(&self) -> Box<Self> {
        let mut cloned = Box::new(Self::new());
        cloned.base.copy_from(&self.base);
        for asset in self.base.assets() {
            cloned.base.add_asset(asset.clone());
        }
        cloned
    }
}
