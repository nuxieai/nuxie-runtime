use std::rc::Rc;

use crate::mechanical_port::source::{
    factory::RuntimeFactoryHandle, generated::assets::image_asset_base::ImageAssetBase,
    renderer::RenderImageRef,
};

pub struct ImageAsset {
    pub base: ImageAssetBase,
    render_image: Option<RenderImageRef>,
    #[cfg(any(test, feature = "tools"))]
    pub decoded_byte_size: usize,
}

impl Default for ImageAsset {
    fn default() -> Self {
        Self {
            base: ImageAssetBase::default(),
            render_image: None,
            #[cfg(any(test, feature = "tools"))]
            decoded_byte_size: 0,
        }
    }
}

impl ImageAsset {
    pub(crate) fn restore_host_image(&mut self, image: Option<RenderImageRef>) {
        self.render_image = image;
    }
    pub fn set_render_image_occurrence(
        owner: &crate::mechanical_port::source::core::CoreHandle,
        image: Option<RenderImageRef>,
    ) {
        let referencers = owner
            .with_downcast_mut::<Self, _>(|owner| {
                owner.render_image = image;
                owner.base.file_asset().file_asset_referencers().to_vec()
            })
            .expect("retained ImageAsset");
        for referencer in referencers {
            referencer
                .with_mut(|referencer| referencer.file_asset_referencer_asset_updated())
                .filter(|updated| *updated)
                .expect("retained ImageAsset referencer");
        }
    }
    #[cfg(all(target_arch = "wasm32", target_os = "unknown"))]
    pub fn decoded_async(&mut self) {
        self.notify_referencers();
    }

    pub fn decode(&mut self, data: &[u8], factory: &RuntimeFactoryHandle) -> bool {
        #[cfg(any(test, feature = "tools"))]
        {
            self.decoded_byte_size = data.len();
        }
        let mut render_image = factory.with_factory_mut(|factory| factory.decode_image(data).ok());
        #[cfg(all(target_arch = "wasm32", target_os = "unknown"))]
        if let (Some(render_image), Some(this)) = (
            render_image.as_mut(),
            self.base.file_asset().base.base.base.base.handle(),
        ) {
            render_image.set_decoded_async_callback(Some(Rc::new(move || {
                let _ = this.with_downcast_mut::<ImageAsset, _>(ImageAsset::decoded_async);
            })));
        }
        let render_image = render_image.map(Rc::from);
        self.set_render_image(render_image);
        self.render_image.is_some()
    }

    pub fn render_image(&self) -> Option<&RenderImageRef> {
        self.render_image.as_ref()
    }

    pub fn set_render_image(&mut self, render_image: Option<RenderImageRef>) {
        self.render_image = render_image;
        self.notify_referencers();
    }

    fn notify_referencers(&mut self) {
        let referencers = self.base.file_asset().file_asset_referencers().to_vec();
        for referencer in referencers {
            referencer
                .with_mut(|referencer| referencer.file_asset_referencer_asset_updated())
                .filter(|updated| *updated)
                .expect("a retained FileAssetReferencer must remain live");
        }
    }

    pub fn file_extension(&self) -> &'static str {
        "png"
    }
}
