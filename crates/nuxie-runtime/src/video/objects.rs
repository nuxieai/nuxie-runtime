use super::playback::Playback;
use crate::source::{
    assets::{file_asset::FileAsset, image_asset::ImageAsset},
    core::{
        Core, CoreHandle, CoreObject, CoreType,
        binary_reader::BinaryReader,
        field_types::{core_callback_type::CallbackData, core_string_type::CoreStringType},
    },
    factory::RuntimeFactoryHandle,
    generated::{
        assets::drawable_asset_base::DrawableAssetBase,
        core_registry::{CoreCapabilities, CoreField, CoreRegistryObject, FileAssetCapability},
        shapes::image_base::ImageBase,
    },
    importers::import_stack::ImportStack,
    renderer::Renderer,
    shapes::image::Image,
    status_code::StatusCode,
};
use nuxie_render_api::RenderImage;
use std::{any::Any, rc::Rc, sync::Arc};

/// Encoded media is shared by instances; playback and decoded frames are not.
pub struct VideoAsset {
    base: ImageAsset,
    pub source_key: String,
    pub content_type: String,
    pub duration: f32,
    encoded: Option<Arc<[u8]>>,
}
impl Default for VideoAsset {
    fn default() -> Self {
        Self {
            base: ImageAsset::default(),
            source_key: String::new(),
            content_type: "video/mp4".into(),
            duration: 0.0,
            encoded: None,
        }
    }
}
impl VideoAsset {
    pub const TYPE_KEY: u16 = 60000;
    pub fn encoded_bytes(&self) -> Option<Arc<[u8]>> {
        self.encoded.clone()
    }
    pub fn set_encoded_bytes(&mut self, bytes: Arc<[u8]>) {
        self.encoded = Some(bytes);
    }
    fn subtype(key: u16) -> bool {
        key == Self::TYPE_KEY || DrawableAssetBase::is_type_of(key)
    }
}

/// One scene occurrence owns one player, even when its media asset is shared.
pub struct Video {
    image: Image,
    pub playback: Playback,
    pub poster_asset_id: u32,
    captions: Option<super::captions::CaptionTrack>,
    captions_valid: bool,
    poster: Option<CoreHandle>,
    showing_poster: bool,
    pub(crate) sync_clock: Option<(Option<super::sync::MediaClock>, f64)>,
    pub(crate) sync_groups: Vec<std::rc::Weak<super::sync::RegisteredSynchronizationGroup>>,
}
impl Default for Video {
    fn default() -> Self {
        Self {
            image: Image::default(),
            playback: Playback::default(),
            poster_asset_id: u32::MAX,
            captions: None,
            captions_valid: true,
            poster: None,
            showing_poster: true,
            sync_clock: None,
            sync_groups: Vec::new(),
        }
    }
}
impl Video {
    pub const TYPE_KEY: u16 = 60001;
    fn subtype(key: u16) -> bool {
        key == Self::TYPE_KEY || ImageBase::is_type_of(key)
    }
    pub fn set_captions(&mut self, track: Option<super::captions::CaptionTrack>) {
        self.captions = track;
        self.captions_valid = true;
    }
    pub(crate) fn captions_valid(&self) -> bool {
        self.captions_valid
    }
    pub fn caption_language(&self) -> &str {
        self.captions.as_ref().map_or("", |track| track.language())
    }
    pub fn caption_text(&self) -> String {
        self.captions
            .as_ref()
            .map_or_else(String::new, |track| track.text(self.playback.position()))
    }
    pub fn asset_id(&self) -> u32 {
        self.image.asset_id()
    }
    pub fn asset(&self) -> Option<CoreHandle> {
        self.image.file_asset_referencer.asset()
    }
    /// Frames must come from the same persistent renderer factory as this scene.
    /// The retained image owns any native surface until the renderer releases it.
    pub fn present(&mut self, generation: u64, image: Rc<dyn RenderImage>, pts: f64) -> bool {
        if !self.playback.accept_frame(generation, pts) {
            return false;
        }
        self.showing_poster = false;
        self.image.set_runtime_frame(Some(image));
        true
    }
    /// Apply the host's current measured decoder allocation to this occurrence.
    /// The host executes returned actions. If it must reclaim the decoder, it
    /// also replaces the source generation before closing/reopening the player.
    /// Posters use the
    /// same scene drawable and preserve authored transforms and clipping.
    pub fn apply_allocation(
        &mut self,
        allocation: super::resources::Allocation,
    ) -> Vec<super::playback::DecoderAction> {
        let denied = allocation == super::resources::Allocation::Poster;
        if denied {
            self.clear_frame();
        }
        self.playback
            .update_suspension(super::playback::SuspensionReason::Resources, denied)
    }
    pub fn has_video_frame(&self) -> bool {
        !self.showing_poster
    }
    pub fn has_poster(&self) -> bool {
        self.poster.as_ref().is_some_and(|p| {
            p.with_downcast::<ImageAsset, _>(|a| a.render_image().is_some())
                .unwrap_or(false)
        })
    }
    pub fn clear_frame(&mut self) {
        self.showing_poster = true;
        self.refresh_poster();
    }
    pub(crate) fn resolve_poster(&mut self, assets: &[CoreHandle]) -> bool {
        if self.poster_asset_id == u32::MAX {
            return true;
        }
        let Some(asset) = assets.get(self.poster_asset_id as usize) else {
            return false;
        };
        if !asset.is_type_of(ImageAsset::TYPE_KEY) {
            return false;
        }
        self.poster = Some(asset.clone());
        self.refresh_poster();
        true
    }
    fn refresh_poster(&mut self) {
        let image = self
            .poster
            .as_ref()
            .and_then(|p| p.with_downcast::<ImageAsset, _>(|a| a.render_image().cloned()))
            .flatten();
        self.image.set_runtime_frame(image);
    }
    pub(crate) fn update_transform_after_super(&mut self) {
        self.image.update_transform_after_super();
    }
    pub(crate) fn try_compose_world_transform_override(&mut self) -> bool {
        self.image.try_compose_world_transform_override()
    }
    fn draw(&mut self, renderer: &mut Renderer) {
        if self.showing_poster {
            self.refresh_poster();
        }
        self.image.draw(renderer);
    }
}

// Delegate inherited property behavior to the existing Image owners. Extension
// values are decoded below, outside the mechanical upstream registry.
macro_rules! inherited_fields {
    ($owner:ty, $field:ident) => {
        impl CoreRegistryObject for $owner {
            fn as_registry_any(&self) -> &dyn Any {
                self
            }
            fn as_registry_any_mut(&mut self) -> &mut dyn Any {
                self
            }
            fn is_type_of(&self, key: u16) -> bool {
                Self::subtype(key)
            }
            fn set_uint(&mut self, f: CoreField, v: u32) {
                self.$field.set_uint(f, v);
            }
            fn set_string(&mut self, f: CoreField, v: String) {
                self.$field.set_string(f, v);
            }
            fn set_color(&mut self, f: CoreField, v: i32) {
                self.$field.set_color(f, v);
            }
            fn set_bool(&mut self, f: CoreField, v: bool) {
                self.$field.set_bool(f, v);
            }
            fn set_double(&mut self, f: CoreField, v: f32) {
                self.$field.set_double(f, v);
            }
            fn set_int(&mut self, f: CoreField, v: i32) {
                self.$field.set_int(f, v);
            }
            fn set_callback(&mut self, f: CoreField, v: CallbackData<'_>) {
                self.$field.set_callback(f, v);
            }
            fn get_uint(&mut self, f: CoreField) -> u32 {
                self.$field.get_uint(f)
            }
            fn get_string(&mut self, f: CoreField) -> String {
                self.$field.get_string(f)
            }
            fn get_color(&mut self, f: CoreField) -> i32 {
                self.$field.get_color(f)
            }
            fn get_bool(&mut self, f: CoreField) -> bool {
                self.$field.get_bool(f)
            }
            fn get_double(&mut self, f: CoreField) -> f32 {
                self.$field.get_double(f)
            }
            fn get_int(&mut self, f: CoreField) -> i32 {
                self.$field.get_int(f)
            }
        }
        impl CoreType for $owner {
            const TYPE_KEY: u16 = <$owner>::TYPE_KEY;
        }
    };
}
inherited_fields!(VideoAsset, base);
inherited_fields!(Video, image);

impl CoreObject for VideoAsset {
    fn core(&self) -> &Core {
        CoreObject::core(&self.base)
    }
    fn core_mut(&mut self) -> &mut Core {
        CoreObject::core_mut(&mut self.base)
    }
    fn core_type(&self) -> u16 {
        Self::TYPE_KEY
    }
    fn is_type_of(&self, key: u16) -> bool {
        Self::subtype(key)
    }
    fn type_predicate(&self) -> fn(u16) -> bool {
        Self::subtype
    }
    fn deserialize(&mut self, key: u16, reader: &mut BinaryReader<'_>) -> bool {
        match key {
            60000 => self.source_key = CoreStringType::deserialize(reader),
            60001 => self.content_type = CoreStringType::deserialize(reader),
            60002 => self.duration = reader.read_float32(),
            _ => return self.base.deserialize(key, reader),
        }
        true
    }
}
impl FileAssetCapability for VideoAsset {
    fn file_asset_base(&self) -> &FileAsset {
        self.base.base.file_asset()
    }
    fn file_asset_base_mut(&mut self) -> &mut FileAsset {
        self.base.base.file_asset_mut()
    }
    fn file_asset_decode(&mut self, data: &mut Vec<u8>, _: &RuntimeFactoryHandle) -> bool {
        self.encoded = Some(Arc::from(std::mem::take(data)));
        true
    }
    fn file_extension(&self) -> &'static str {
        "mp4"
    }
    fn adds_to_backboard(&self) -> bool {
        true
    }
}
impl CoreCapabilities for VideoAsset {
    fn lifecycle_import(&mut self, stack: &mut ImportStack) -> Option<StatusCode> {
        Some(self.file_asset_base_mut().import(true, stack))
    }
    fn as_file_asset(&self) -> Option<&dyn FileAssetCapability> {
        Some(self)
    }
    fn as_file_asset_mut(&mut self) -> Option<&mut dyn FileAssetCapability> {
        Some(self)
    }
}
impl CoreObject for Video {
    fn core(&self) -> &Core {
        CoreObject::core(&self.image)
    }
    fn core_mut(&mut self) -> &mut Core {
        CoreObject::core_mut(&mut self.image)
    }
    fn core_type(&self) -> u16 {
        Self::TYPE_KEY
    }
    fn is_type_of(&self, key: u16) -> bool {
        Self::subtype(key)
    }
    fn type_predicate(&self) -> fn(u16) -> bool {
        Self::subtype
    }
    fn clone_boxed(&self) -> Option<Box<dyn CoreObject>> {
        Some(Box::new(Self {
            image: self.image.clone_definition(),
            playback: Playback::new(self.playback.settings()),
            poster_asset_id: self.poster_asset_id,
            captions: self.captions.clone(),
            captions_valid: self.captions_valid,
            poster: self.poster.clone(),
            showing_poster: true,
            sync_clock: None,
            sync_groups: Vec::new(),
        }))
    }
    fn deserialize(&mut self, key: u16, reader: &mut BinaryReader<'_>) -> bool {
        let mut settings = self.playback.settings();
        match key {
            60003 => settings.autoplay = reader.read_var_uint_as::<u32>() != 0,
            60004 => settings.looping = reader.read_var_uint_as::<u32>() != 0,
            60005 => settings.muted = reader.read_var_uint_as::<u32>() != 0,
            60006 => settings.volume = reader.read_float32(),
            60007 => settings.rate = reader.read_float32(),
            60008 => settings.priority = reader.read_var_uint_as::<u32>(),
            60009 => self.poster_asset_id = reader.read_var_uint_as::<u32>(),
            60010 => settings.audio_policy = reader.read_var_uint_as::<u32>(),
            60011 => settings.readiness = reader.read_var_uint_as::<u32>(),
            60012 => settings.reentry = reader.read_var_uint_as::<u32>(),
            60014 => settings.loop_start = f64::from(reader.read_float32()),
            60015 => settings.loop_end = f64::from(reader.read_float32()),
            60013 => {
                let text = CoreStringType::deserialize(reader);
                if text.is_empty() {
                    self.set_captions(None);
                } else {
                    match super::captions::CaptionTrack::from_json(&text) {
                        Ok(track) => self.set_captions(Some(track)),
                        Err(_) => self.captions_valid = false,
                    }
                }
            }
            _ => return self.image.deserialize(key, reader),
        }
        self.playback = Playback::new(settings);
        true
    }
}

impl CoreCapabilities for Video {
    fn file_asset_referencer_asset_id(&self) -> Option<u32> {
        Some(self.asset_id())
    }
    fn file_asset_referencer_set_asset(&mut self, asset: CoreHandle) -> bool {
        if !asset.is_type_of(VideoAsset::TYPE_KEY) {
            return false;
        }
        let Some(owner) = CoreObject::core(&self.image).handle() else {
            return false;
        };
        if let Some((width, height)) =
            asset.with_downcast::<VideoAsset, _>(|a| (a.base.base.width(), a.base.base.height()))
        {
            self.image.set_runtime_size(width, height);
        }
        self.image
            .file_asset_referencer
            .set_asset(owner, Some(asset));
        true
    }
    fn as_file_asset_referencer_mut(
        &mut self,
    ) -> Option<&mut crate::source::assets::file_asset_referencer::FileAssetReferencer> {
        Some(&mut self.image.file_asset_referencer)
    }
    fn file_asset_referencer_asset_updated(&mut self) -> bool {
        true
    }
    fn drawable_draw(&mut self, renderer: &mut Renderer) -> bool {
        self.draw(renderer);
        true
    }
    fn drawable_will_draw(&self) -> bool {
        (self.image.render_image().is_some()
            || self.poster.as_ref().is_some_and(|p| {
                p.with_downcast::<ImageAsset, _>(|a| a.render_image().is_some())
                    .unwrap_or(false)
            }))
            && self.image.base.will_draw()
            && self.image.base.render_opacity() != 0.0
    }
    fn as_intrinsically_sizeable_mut(
        &mut self,
    ) -> Option<&mut dyn crate::source::intrinsically_sizeable::IntrinsicallySizeable> {
        Some(&mut self.image)
    }
    fn layout_provider_handle(&self) -> Option<CoreHandle> {
        self.image.layout_participant()
    }
    fn semantic_provider_local_bounds(&self) -> Option<crate::source::math::aabb::Aabb> {
        Some(self.image.local_bounds())
    }
    fn drawable_hit_test(
        &mut self,
        info: &mut crate::source::hit_info::HitInfo,
        transform: &crate::source::math::mat2d::Mat2D,
    ) -> Option<CoreHandle> {
        self.image.hit_test(info, *transform).and_then(Core::handle)
    }
    fn as_node(&self) -> Option<&crate::source::node::Node> {
        self.image.as_node()
    }
    fn as_node_mut(&mut self) -> Option<&mut crate::source::node::Node> {
        self.image.as_node_mut()
    }
    fn as_drawable(&self) -> Option<&crate::source::drawable::Drawable> {
        self.image.as_drawable()
    }
    fn as_drawable_mut(&mut self) -> Option<&mut crate::source::drawable::Drawable> {
        self.image.as_drawable_mut()
    }
    fn as_transform_component(
        &self,
    ) -> Option<&crate::source::transform_component::TransformComponent> {
        self.image.as_transform_component()
    }
    fn as_transform_component_mut(
        &mut self,
    ) -> Option<&mut crate::source::transform_component::TransformComponent> {
        self.image.as_transform_component_mut()
    }
    fn as_component(&self) -> Option<&crate::source::component::Component> {
        self.image.as_component()
    }
    fn as_component_mut(&mut self) -> Option<&mut crate::source::component::Component> {
        self.image.as_component_mut()
    }
    fn as_container_component(
        &self,
    ) -> Option<&crate::source::container_component::ContainerComponent> {
        self.image.as_container_component()
    }
    fn as_container_component_mut(
        &mut self,
    ) -> Option<&mut crate::source::container_component::ContainerComponent> {
        self.image.as_container_component_mut()
    }
    fn component_build_dependencies(&mut self) -> bool {
        self.image.component_build_dependencies()
    }
    fn lifecycle_validate(
        &mut self,
        context: &mut dyn crate::source::core_context::CoreContext,
    ) -> Option<bool> {
        Some(
            self.playback.settings().valid()
                && self.captions_valid
                && self.image.lifecycle_validate(context).unwrap_or(true),
        )
    }
    fn lifecycle_on_added_dirty(
        &mut self,
        context: &mut dyn crate::source::core_context::CoreContext,
    ) -> Option<StatusCode> {
        self.image.lifecycle_on_added_dirty(context)
    }
    fn lifecycle_on_added_clean(
        &mut self,
        context: &mut dyn crate::source::core_context::CoreContext,
    ) -> Option<StatusCode> {
        self.image.lifecycle_on_added_clean(context)
    }
    fn lifecycle_import(&mut self, stack: &mut ImportStack) -> Option<StatusCode> {
        Some(self.image.import(stack))
    }
}
