//! Translation of include/rive/bitmap_cache.hpp and src/bitmap_cache.cpp at
//! upstream a4dbc3ff.

use crate::mechanical_port::source::generated::{
    bitmap_cache_base::{BitmapCacheBase, BitmapCacheBaseCallbacks},
    component_base::ComponentBaseCallbacks,
};
use nuxie_render_api::RenderCanvasHandle;

/// The offscreen render state an Artboard caches itself into. Upstream makes
/// Artboard a friend so it reads and writes these fields directly; here they
/// are crate-visible for the same owner.
pub struct BitmapCache {
    pub base: BitmapCacheBase,
    pub(crate) canvas: Option<RenderCanvasHandle>,
    pub(crate) width_px: u32,
    pub(crate) height_px: u32,
    pub(crate) raster_scale: f32,
    pub(crate) dirty: bool,
}

impl Default for BitmapCache {
    fn default() -> Self {
        Self {
            base: BitmapCacheBase::default(),
            canvas: None,
            width_px: 0,
            height_px: 0,
            raster_scale: 1.0,
            dirty: true,
        }
    }
}

impl BitmapCache {
    pub const TYPE_KEY: u16 = BitmapCacheBase::TYPE_KEY;

    pub fn cache_enabled(&self) -> bool {
        self.base.cache_enabled()
    }

    pub fn resolution(&self) -> f32 {
        self.base.resolution()
    }

    pub(crate) fn invalidate(&mut self) {
        // The cached raster is no longer trustworthy; rebuild it on the next
        // draw.
        self.dirty = true;
        // ...but only if there *is* a next draw. `dirty` is private to this
        // object, so nothing outside it can tell the artboard changed -- and a
        // host that skips rendering while Artboard::did_change() is false
        // would then never submit the frame that rebuilds the cache, leaving
        // the stale raster on screen for good. None while the property is
        // being deserialized, before the object is parented.
        if let Some(owner) = self.base.base.artboard_handle() {
            if let Some(dirty) = owner.artboard_dirty_handle() {
                dirty.changed();
            }
        }
    }

    pub fn resolution_changed(&mut self) {
        // A new resolution means the cached raster is the wrong size.
        self.invalidate();
    }

    pub fn cache_flags_changed(&mut self) {
        // A flag change can alter how the raster is produced, so the existing
        // one is no longer trustworthy. (`dither` is reserved and not read
        // anywhere yet, so today only cacheEnabled actually changes the
        // output.)
        self.invalidate();
        if !self.cache_enabled() {
            // Freeing the texture is the point of a disable switch. Safe to
            // drop ours mid-frame: the canvas is reference counted and a frame
            // being replayed holds its own reference.
            self.canvas = None;
            self.width_px = 0;
            self.height_px = 0;
        }
    }
}

impl BitmapCacheBaseCallbacks for BitmapCache {
    fn notify_property_changed(&mut self, property_key: u16) {
        self.base
            .base
            .base
            .base
            .notify_property_changed(property_key);
    }

    fn resolution_changed(&mut self) {
        BitmapCache::resolution_changed(self);
    }

    fn cache_flags_changed(&mut self) {
        BitmapCache::cache_flags_changed(self);
    }
}

impl ComponentBaseCallbacks for BitmapCache {
    fn notify_property_changed(&mut self, property_key: u16) {
        self.base
            .base
            .base
            .base
            .notify_property_changed(property_key);
    }
}

impl std::ops::Deref for BitmapCache {
    type Target = BitmapCacheBase;

    fn deref(&self) -> &Self::Target {
        &self.base
    }
}

impl std::ops::DerefMut for BitmapCache {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.base
    }
}
