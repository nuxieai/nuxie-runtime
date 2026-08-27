use crate::mechanical_port::source::{
    core::CoreHandle, core_context::CoreContext, factory::Factory,
    generated::assets::image_asset_base::ImageAssetBase, renderer::RenderImageRef,
};

pub struct ImageAsset {
    pub base: ImageAssetBase,
    render_image: Option<RenderImageRef>,
    #[cfg(feature = "testing")]
    pub decoded_byte_size: usize,
}

impl Default for ImageAsset {
    fn default() -> Self {
        Self {
            base: ImageAssetBase::default(),
            render_image: None,
            #[cfg(feature = "testing")]
            decoded_byte_size: 0,
        }
    }
}

impl Drop for ImageAsset {
    fn drop(&mut self) {
        #[cfg(feature = "emscripten")]
        if let Some(render_image) = self.render_image.as_mut() {
            render_image.set_delegate(None);
        }
    }
}

impl ImageAsset {
    #[cfg(feature = "emscripten")]
    pub fn decoded_async(&mut self, context: &mut CoreContext) {
        self.notify_referencers(context);
    }

    pub fn decode(
        &mut self,
        this: CoreHandle,
        data: &[u8],
        factory: &mut Factory,
        context: &mut CoreContext,
    ) -> bool {
        #[cfg(feature = "testing")]
        {
            self.decoded_byte_size = data.len();
        }
        let render_image = factory.decode_image(data);
        self.set_render_image(this, render_image, context);
        self.render_image.is_some()
    }

    pub fn render_image(&self) -> Option<&RenderImageRef> {
        self.render_image.as_ref()
    }

    pub fn set_render_image(
        &mut self,
        this: CoreHandle,
        render_image: Option<RenderImageRef>,
        context: &mut CoreContext,
    ) {
        self.render_image = render_image;
        #[cfg(feature = "emscripten")]
        if let Some(render_image) = self.render_image.as_mut() {
            render_image.set_delegate(Some(this));
        }
        self.notify_referencers(context);
    }

    fn notify_referencers(&mut self, context: &mut CoreContext) {
        let referencers = self.base.file_asset().file_asset_referencers().to_vec();
        for referencer in referencers {
            context
                .file_asset_referencer_mut(referencer)
                .expect("a retained FileAssetReferencer must remain live")
                .asset_updated();
        }
    }

    pub fn file_extension(&self) -> &'static str {
        "png"
    }
}
