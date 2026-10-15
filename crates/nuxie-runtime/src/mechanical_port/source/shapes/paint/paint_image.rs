use super::artboard_paint_source::{ArtboardPaintSource, ModulatedDrawTarget};
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
use crate::source::{
    advance_flags::AdvanceFlags,
    advancing_component::{AdvancingComponent, QuietState},
    artboard::{Artboard, RuntimeArtboardInstanceHandle, RuntimeArtboardInstanceWeakHandle},
    artboard_host::ArtboardHost,
    artboard_referencer::{ArtboardReferencer, ArtboardReferencerBehavior},
    component::ComponentDirt,
    data_bind::data_context::RuntimeDataContextHandle,
    data_bind_path_referencer::DataBindPathReferencer,
    file::RuntimeFileWeakHandle,
    importers::backboard_importer::BackboardImporter,
    math::vec2d::Vec2D,
};
use std::{cell::Cell, rc::Rc};

#[derive(Default)]
pub struct PaintImage {
    // Derived owned resources are destroyed before the inherited component.
    pub source: Option<ArtboardPaintSource>,
    // The unique source can be moved into an occurrence-scoped operation to
    // release this component's borrow. Host traversal still sees its child;
    // this weak pointer does not extend that child's upstream lifetime.
    detached_artboard: Option<RuntimeArtboardInstanceWeakHandle>,
    pub base: PaintImageBase,
    pub asset_referencer: FileAssetReferencer,
    pub artboard_referencer: ArtboardReferencer,
    file: Option<RuntimeFileWeakHandle>,
    inactive_sequence: Rc<Cell<u64>>,
    bind_path: DataBindPathReferencer,
}

impl PaintImage {
    pub const TYPE_KEY: u16 = PaintImageBase::TYPE_KEY;

    pub fn import(&mut self, stack: &mut ImportStack) -> StatusCode {
        if let Some(this) = self.base.base.handle() {
            self.asset_referencer.register_referencer(this, stack);
        }
        let code = self.base.base.import(stack);
        if code != StatusCode::Ok {
            return code;
        }
        if let Some(importer) = stack.latest::<BackboardImporter>(
            crate::source::generated::backboard_base::BackboardBase::TYPE_KEY,
        ) {
            importer.add_artboard_referencer(self.base.base.handle().expect("live PaintImage"));
        }
        StatusCode::Ok
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
        twin.artboard_referencer
            .set_referenced_artboard(self.source_artboard());
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
    pub fn texture_transform(
        &self,
        bounds: &Aabb,
        original_width: f32,
        original_height: f32,
    ) -> Mat2D {
        let original = self.base.image_size_mode() == 1;
        let base_w = if original {
            original_width
        } else {
            bounds.width()
        };
        let base_h = if original {
            original_height
        } else {
            bounds.height()
        };
        Mat2D::from_translate(
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
            * Mat2D::from_translate(-0.5, -0.5)
    }
    pub fn apply_to(
        &mut self,
        renderer: &mut dyn nuxie_render_api::Renderer,
        paint: &mut dyn nuxie_render_api::RenderPaint,
        bounds: &Aabb,
        target: &mut ModulatedDrawTarget,
    ) -> bool {
        target.renderer = None;
        if self.source_artboard().is_some() {
            let Some(source) = self.source.as_ref() else {
                return false;
            };
            let box_ = source.bounds();
            let transform = self.texture_transform(bounds, box_.width(), box_.height());
            let sampler = self.image_sampler();
            let host = self
                .base
                .base
                .artboard_handle()
                .and_then(|a| a.with_downcast::<Artboard, _>(|a| a.factory()).flatten())
                .and_then(|f| f.with_factory_mut(|f| f.canvas_content_host()));
            return self
                .source
                .as_mut()
                .unwrap()
                .apply_to(renderer, host, paint, sampler, transform, target);
        }
        let Some(image) = self.image_asset().and_then(|asset| {
            asset
                .with_downcast::<ImageAsset, _>(|a| a.render_image().cloned())
                .flatten()
        }) else {
            return false;
        };
        let transform = self.texture_transform(bounds, image.width() as f32, image.height() as f32);
        paint.modulated_image(
            Some(image.as_ref()),
            self.image_sampler().into(),
            nuxie_render_api::Mat2D(*transform.values()),
        );
        true
    }
    pub fn source_artboard(&self) -> Option<CoreHandle> {
        self.artboard_referencer.referenced_artboard()
    }
    pub fn apply_to_occurrence(
        this: &CoreHandle,
        renderer: &mut dyn nuxie_render_api::Renderer,
        paint: &mut dyn nuxie_render_api::RenderPaint,
        bounds: &Aabb,
        target: &mut ModulatedDrawTarget,
    ) -> bool {
        target.renderer = None;
        let referenced = this
            .with_downcast::<Self, _>(|p| p.source_artboard().is_some())
            .unwrap_or(false);
        if !referenced {
            return this
                .with_downcast_mut::<Self, _>(|p| p.apply_to(renderer, paint, bounds, target))
                .unwrap_or(false);
        }
        let parameters = this
            .with_downcast::<Self, _>(|p| {
                let source = p.source.as_ref()?;
                let box_ = source.bounds();
                Some((
                    p.texture_transform(bounds, box_.width(), box_.height()),
                    p.image_sampler(),
                    p.base.base.artboard_handle(),
                ))
            })
            .flatten();
        let Some((transform, sampler, owner)) = parameters else {
            return false;
        };
        let host = owner
            .and_then(|a| a.with_downcast::<Artboard, _>(|a| a.factory()).flatten())
            .and_then(|f| f.with_factory_mut(|f| f.canvas_content_host()));
        Self::with_source_occurrence(this, |source| {
            source.apply_to(renderer, host, paint, sampler, transform, target)
        })
        .unwrap_or(false)
    }
    pub fn artboard_source(&self) -> Option<&ArtboardPaintSource> {
        self.source.as_ref()
    }
    pub fn on_added_clean_occurrence(
        this: &CoreHandle,
        context: &mut dyn CoreContext,
    ) -> StatusCode {
        let (code, source, owner) = this
            .with_downcast_mut::<Self, _>(|p| {
                (
                    p.base.base.on_added_clean(context),
                    p.source_artboard(),
                    p.base.base.artboard_handle(),
                )
            })
            .unwrap();
        if code != StatusCode::Ok {
            return code;
        }
        if let (Some(source), Some(owner)) = (source, owner) {
            let (is_instance, factory) = owner
                .with_downcast::<Artboard, _>(|a| (a.is_instance(), a.factory()))
                .unwrap();
            if is_instance {
                let source = ArtboardPaintSource::make(Some(&source), factory, Some(this.clone()));
                let has_source = source.is_some();
                this.with_downcast_mut::<Self, _>(|p| p.source = source);
                if has_source {
                    this.with_downcast_mut::<Self, _>(|p| {
                        p.base.base.add_dirt(ComponentDirt::COMPONENTS, false)
                    });
                    if let Some(data) = owner
                        .with_downcast::<Artboard, _>(|a| a.data_context())
                        .flatten()
                    {
                        Self::with_source_occurrence(this, |source| {
                            source.data_context(Some(data))
                        });
                    }
                }
            }
        }
        StatusCode::Ok
    }
    pub fn build_dependencies(&mut self) {
        self.base.base.build_dependencies();
        if let Some(owner) = self.base.base.artboard_handle() {
            owner.with_downcast_mut::<Artboard, _>(|a| {
                a.add_dependent(self.base.base.handle().unwrap())
            });
        }
    }
    pub fn update(&mut self, value: ComponentDirt) {
        self.base.base.update(value);
        if value.contains(ComponentDirt::COMPONENTS) {
            if let Some(source) = self.source.as_mut() {
                source.update();
            }
        }
    }
    pub fn update_occurrence(this: &CoreHandle, value: ComponentDirt) {
        this.with_downcast_mut::<Self, _>(|p| p.base.base.update(value));
        if value.contains(ComponentDirt::COMPONENTS) {
            Self::with_source_occurrence(this, |source| source.update());
        }
    }
    pub fn update_data_binds_occurrence(this: &CoreHandle) {
        if this
            .with_downcast::<Self, _>(|p| p.base.base.is_collapsed())
            .unwrap_or(true)
        {
            return;
        }
        Self::with_source_occurrence(this, |source| source.update_data_binds());
    }
    pub fn internal_data_context_occurrence(this: &CoreHandle, context: RuntimeDataContextHandle) {
        Self::with_source_occurrence(this, |source| source.data_context(Some(context)));
    }
    pub fn relink_data_context_occurrence(this: &CoreHandle, instance: Option<CoreHandle>) {
        Self::with_source_occurrence(this, |source| source.relink_data_context(instance));
    }
    pub fn clear_data_context_occurrence(this: &CoreHandle) {
        Self::with_source_occurrence(this, |source| source.clear_data_context());
    }
    pub fn unbind_occurrence(this: &CoreHandle) {
        Self::with_source_occurrence(this, |source| source.unbind());
    }
    fn with_source_occurrence<R>(
        this: &CoreHandle,
        operation: impl FnOnce(&mut ArtboardPaintSource) -> R,
    ) -> Option<R> {
        let source = this
            .with_downcast_mut::<Self, _>(|p| {
                let source = p.source.take()?;
                p.detached_artboard = Some(source.artboard().downgrade());
                Some(source)
            })
            .flatten()?;
        // Restore on both normal return and unwind, so host identity isn't
        // left detached by a Rust host callback. A deleted owner stays deleted.
        struct SourceScope {
            owner: CoreHandle,
            source: Option<ArtboardPaintSource>,
        }
        impl Drop for SourceScope {
            fn drop(&mut self) {
                self.owner.with_downcast_mut::<PaintImage, _>(|p| {
                    p.source = self.source.take();
                    p.detached_artboard = None;
                });
            }
        }
        let mut scope = SourceScope {
            owner: this.clone(),
            source: Some(source),
        };
        Some(operation(scope.source.as_mut().unwrap()))
    }
    pub fn advance_component_occurrence(
        this: &CoreHandle,
        seconds: f32,
        flags: AdvanceFlags,
    ) -> bool {
        if this
            .with_downcast::<Self, _>(|p| p.base.base.is_collapsed())
            .unwrap_or(true)
        {
            return false;
        }
        let Some((keep, needs)) = Self::with_source_occurrence(this, |source| {
            let keep = source.advance(seconds, flags);
            (keep, source.needs_update())
        }) else {
            return false;
        };
        this.with_downcast_mut::<Self, _>(|p| {
            if needs {
                p.base.base.add_dirt(ComponentDirt::COMPONENTS, false);
            }
        });
        keep
    }
    pub fn quiet_state(&self) -> QuietState {
        if self.source.is_some() || self.detached_artboard.is_some() {
            QuietState::Never
        } else {
            QuietState::Quiet
        }
    }
}
impl ArtboardReferencerBehavior for PaintImage {
    fn nesting_artboard(&self) -> Option<CoreHandle> {
        self.base.base.artboard_handle()
    }
    fn artboard_referencer(&self) -> &ArtboardReferencer {
        &self.artboard_referencer
    }
    fn artboard_referencer_mut(&mut self) -> &mut ArtboardReferencer {
        &mut self.artboard_referencer
    }
    fn referenced_artboard_id(&self) -> i32 {
        self.base.artboard_id() as i32
    }
}
impl crate::source::artboard_referencer::CoreArtboardReferencer for PaintImage {
    fn core(&mut self) -> &mut crate::source::core::Core {
        &mut self.base.base.base
    }
    fn core_type(&self) -> u16 {
        PaintImageBase::TYPE_KEY
    }
}
impl AdvancingComponent for PaintImage {
    fn advance_component(&mut self, seconds: f32, flags: AdvanceFlags) -> bool {
        if self.base.base.is_collapsed() {
            return false;
        }
        let Some(source) = &mut self.source else {
            return false;
        };
        let keep = source.advance(seconds, flags);
        if source.needs_update() {
            self.base.base.add_dirt(ComponentDirt::COMPONENTS, false);
        }
        keep
    }
    fn quiet_state(&self) -> QuietState {
        self.quiet_state()
    }
}
impl ArtboardHost for PaintImage {
    fn inactive_change_sequence_state(&self) -> Rc<Cell<u64>> {
        self.inactive_sequence.clone()
    }
    fn data_bind_path_referencer(&self) -> &DataBindPathReferencer {
        &self.bind_path
    }
    fn artboard_count(&self) -> usize {
        usize::from(self.source.is_some() || self.detached_artboard.is_some())
    }
    fn artboard_instance(&self, index: i32) -> Option<RuntimeArtboardInstanceHandle> {
        if index == 0 {
            self.source
                .as_ref()
                .map(|s| s.artboard())
                .or_else(|| self.detached_artboard.as_ref().and_then(|a| a.upgrade()))
        } else {
            None
        }
    }
    fn internal_data_context(&mut self, context: RuntimeDataContextHandle) {
        if let Some(s) = &mut self.source {
            s.data_context(Some(context));
        }
    }
    fn bind_view_model_instance(&mut self, _: CoreHandle, parent: RuntimeDataContextHandle) {
        self.internal_data_context(parent);
    }
    fn relink_data_context(&mut self, instance: Option<CoreHandle>) {
        if let Some(s) = &mut self.source {
            s.relink_data_context(instance);
        }
    }
    fn clear_data_context(&mut self) {
        if let Some(s) = &mut self.source {
            s.clear_data_context();
        }
    }
    fn unbind(&mut self) {
        if let Some(s) = &mut self.source {
            s.unbind();
        }
    }
    fn update_data_binds(&mut self) {
        if !self.base.base.is_collapsed() {
            if let Some(s) = &mut self.source {
                s.update_data_binds();
            }
        }
    }
    fn mark_hosting_layout_dirty(&mut self, _: RuntimeArtboardInstanceWeakHandle) {}
    fn parent_artboard(&self) -> Option<CoreHandle> {
        self.base.base.artboard_handle()
    }
    fn hit_test_host(&mut self, _: &Vec2D, _: bool, _: RuntimeArtboardInstanceWeakHandle) -> bool {
        false
    }
    fn host_transform_point(&self, p: &Vec2D, _: RuntimeArtboardInstanceWeakHandle) -> Vec2D {
        *p
    }
    fn world_transform_for_artboard(&self, _: RuntimeArtboardInstanceWeakHandle) -> Mat2D {
        Mat2D::default()
    }
    fn mark_host_transform_dirty(&mut self) {}
    fn set_file(&mut self, file: Option<RuntimeFileWeakHandle>) {
        self.file = file;
    }
    fn file(&self) -> Option<RuntimeFileWeakHandle> {
        self.file.clone()
    }
    fn host_component(&self) -> Option<CoreHandle> {
        None
    }
    fn type_(&self) -> i32 {
        PaintImageBase::TYPE_KEY as i32
    }
}
