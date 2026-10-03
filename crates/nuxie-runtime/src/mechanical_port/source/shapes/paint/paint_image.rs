use crate::mechanical_port::source::{
    assets::{file_asset_referencer::FileAssetReferencer, image_asset::ImageAsset},
    core::CoreHandle,
    core_context::CoreContext,
    generated::{
        assets::image_asset_base::ImageAssetBase,
        shapes::paint::{paint_image_base::PaintImageBase, shape_paint_base::ShapePaintBase},
    },
    importers::import_stack::ImportStack,
    math::{aabb::Aabb, mat2d::Mat2D},
    shapes::paint::image_sampler::{ImageFilter, ImageSampler, ImageWrap},
    status_code::StatusCode,
};

#[derive(Default)]
pub struct PaintImage {
    pub base: PaintImageBase,
    pub asset_referencer: FileAssetReferencer,
}

impl PaintImage {
    pub fn import(&mut self, stack: &mut ImportStack) -> StatusCode {
        if let Some(this) = self.base.base.handle() {
            self.asset_referencer.register_referencer(this, stack);
        }
        self.base.base.import(stack)
    }
    pub fn validate(&mut self, context: &mut dyn CoreContext) -> bool {
        if !self.base.base.validate(context) {
            return false;
        }
        context
            .resolve(self.base.base.parent_id())
            .expect("validated PaintImage parent")
            .is_type_of(ShapePaintBase::TYPE_KEY)
    }
    pub fn clone_definition(&self) -> Self {
        let mut twin = Self::default();
        let mut base = std::mem::take(&mut twin.base);
        base.copy(&self.base, &mut twin);
        twin.base = base;
        if let Some(asset) = self.asset_referencer.asset() {
            twin.asset_referencer.set_asset_unattached(Some(asset));
        }
        twin
    }
    pub fn asset_id(&self) -> u32 {
        self.base.image_asset_id()
    }
    fn invalidate_paint(&self) {
        if let Some(parent) = self.base.base.parent_handle() {
            crate::mechanical_port::source::shapes::paint::effects_container::invalidate_rendering_handle(
                &parent,
            );
        }
    }
    pub fn set_asset(&mut self, asset: Option<CoreHandle>) {
        if let Some(asset) = asset.filter(|asset| asset.is_type_of(ImageAssetBase::TYPE_KEY)) {
            self.asset_referencer.set_asset(
                self.base.base.handle().expect("live PaintImage owner"),
                Some(asset),
            );
            self.invalidate_paint();
        }
    }
    pub fn asset_updated(&mut self) {
        self.invalidate_paint();
    }
    pub fn image_asset(&self) -> Option<CoreHandle> {
        self.asset_referencer.asset()
    }
    pub fn image_sampler(&self) -> ImageSampler {
        fn filter(value: u8) -> ImageFilter {
            match value {
                1 => ImageFilter::Nearest,
                _ => ImageFilter::Bilinear,
            }
        }
        fn wrap(value: u8) -> ImageWrap {
            match value {
                1 => ImageWrap::Repeat,
                2 => ImageWrap::Mirror,
                _ => ImageWrap::Clamp,
            }
        }
        let mut sampler = ImageSampler::linear_clamp();
        if let Some(asset) = self.image_asset() {
            asset.with_downcast::<ImageAsset, _>(|asset| {
                sampler.filter = filter(asset.base.sampler_filter());
                sampler.wrap_x = wrap(asset.base.sampler_wrap_x());
                sampler.wrap_y = wrap(asset.base.sampler_wrap_y());
            });
        }
        if self.base.image_sampler_filter() != 0 {
            sampler.filter = filter(self.base.image_sampler_filter() - 1);
        }
        if self.base.image_sampler_wrap_x() != 0 {
            sampler.wrap_x = wrap(self.base.image_sampler_wrap_x() - 1);
        }
        if self.base.image_sampler_wrap_y() != 0 {
            sampler.wrap_y = wrap(self.base.image_sampler_wrap_y() - 1);
        }
        sampler
    }
    pub fn apply_to(&self, paint: &mut dyn nuxie_render_api::RenderPaint, bounds: &Aabb) -> bool {
        let Some(image) = self.image_asset().and_then(|asset| {
            asset
                .with_downcast::<ImageAsset, _>(|asset| asset.render_image().cloned())
                .flatten()
        }) else {
            return false;
        };
        let original = self.base.image_size_mode() == 1;
        let base_w = if original {
            image.width() as f32
        } else {
            bounds.width()
        };
        let base_h = if original {
            image.height() as f32
        } else {
            bounds.height()
        };
        let transform = Mat2D::from_translate(
            bounds.left() + bounds.width() * 0.5,
            bounds.top() + bounds.height() * 0.5,
        ) * Mat2D::from_rotation(self.base.image_rotation())
            * Mat2D::from_translate(
                self.base.image_offset_x() * base_w,
                self.base.image_offset_y() * base_h,
            )
            * Mat2D::from_scale(
                base_w * self.base.image_scale_x(),
                base_h * self.base.image_scale_y(),
            )
            * Mat2D::from_translate(-0.5, -0.5);
        paint.modulated_image(
            Some(image.as_ref()),
            self.image_sampler().into(),
            nuxie_render_api::Mat2D(*transform.values()),
        );
        true
    }
}
