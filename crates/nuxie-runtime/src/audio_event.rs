use std::cell::{Cell, RefCell};
use std::collections::BTreeMap;
use std::rc::Rc;
use std::sync::Arc;

use nuxie_audio::{AudioArtboardId, AudioEngine, AudioSound};
use nuxie_binary::RuntimeFile;

use crate::{InstanceSlot, RuntimeAudioAssetOwners};

#[derive(Debug, Clone, Copy)]
struct RuntimeAudioEventAsset {
    global_id: u32,
    volume: f32,
}

#[derive(Debug)]
struct RuntimeAudioEventPlaybackInner {
    artboard_id: AudioArtboardId,
    engine: RefCell<Option<AudioEngine>>,
    volume: Cell<f32>,
    assets: RefCell<Arc<RuntimeAudioAssetOwners>>,
    event_assets: BTreeMap<usize, RuntimeAudioEventAsset>,
}

/// Clone-safe owner for one concrete Artboard's `AudioEvent::play` state.
///
/// State-machine occurrences retain this owner so engine and volume changes
/// made after their construction remain visible at the playback unwind seam.
#[derive(Debug, Clone)]
pub(crate) struct RuntimeAudioEventPlayback {
    inner: Rc<RuntimeAudioEventPlaybackInner>,
}

impl RuntimeAudioEventPlayback {
    pub(crate) fn empty(artboard_id: AudioArtboardId) -> Self {
        Self {
            inner: Rc::new(RuntimeAudioEventPlaybackInner {
                artboard_id,
                engine: RefCell::new(None),
                volume: Cell::new(1.0),
                assets: RefCell::new(Arc::new(RuntimeAudioAssetOwners::default())),
                event_assets: BTreeMap::new(),
            }),
        }
    }

    pub(crate) fn new(
        artboard_id: AudioArtboardId,
        file: &RuntimeFile,
        slots: &[InstanceSlot],
        assets: Arc<RuntimeAudioAssetOwners>,
    ) -> Self {
        let event_assets = slots
            .iter()
            .filter_map(|slot| {
                let event = file.object(slot.source_global_id as usize)?;
                if event.type_name != "AudioEvent" {
                    return None;
                }
                // AudioEvent.assetId is the dense File::assets ordinal, not
                // the semantic FileAsset.assetId value.
                let asset_index = usize::try_from(event.uint_property("assetId")?).ok()?;
                let asset = file.file_asset(asset_index)?;
                (asset.type_name == "AudioAsset").then_some((
                    slot.local_id,
                    RuntimeAudioEventAsset {
                        global_id: asset.id,
                        volume: asset.double_property("volume").unwrap_or(1.0),
                    },
                ))
            })
            .collect();
        Self {
            inner: Rc::new(RuntimeAudioEventPlaybackInner {
                artboard_id,
                engine: RefCell::new(None),
                volume: Cell::new(1.0),
                assets: RefCell::new(assets),
                event_assets,
            }),
        }
    }

    pub(crate) fn cold_clone(&self, artboard_id: AudioArtboardId) -> Self {
        Self {
            inner: Rc::new(RuntimeAudioEventPlaybackInner {
                artboard_id,
                engine: RefCell::new(self.engine()),
                volume: Cell::new(self.volume()),
                assets: RefCell::new(self.inner.assets.borrow().clone()),
                event_assets: self.inner.event_assets.clone(),
            }),
        }
    }

    pub(crate) fn replace_with_transient_view_of(&mut self, source: &Self) {
        self.inner = Rc::clone(&source.inner);
    }

    pub(crate) fn engine(&self) -> Option<AudioEngine> {
        self.inner.engine.borrow().clone()
    }

    pub(crate) fn set_engine(&self, engine: Option<AudioEngine>) {
        *self.inner.engine.borrow_mut() = engine;
    }

    pub(crate) fn volume(&self) -> f32 {
        self.inner.volume.get()
    }

    pub(crate) fn set_volume(&self, volume: f32) {
        self.inner.volume.set(volume);
    }

    pub(crate) fn set_assets(&self, assets: Arc<RuntimeAudioAssetOwners>) {
        *self.inner.assets.borrow_mut() = assets;
    }

    pub(crate) fn inherit_configuration_from(&self, source: &Self) {
        self.set_engine(source.engine());
        self.set_volume(source.volume());
        self.set_assets(source.inner.assets.borrow().clone());
    }

    /// Pinned `AudioEvent::play`: resolve the retained AudioAsset, multiply
    /// asset and Artboard volumes, schedule at the current PCM frame, and tag
    /// the sound for Artboard-scoped teardown. Report delay is intentionally
    /// absent from this calculation, matching the C++ event path.
    pub(crate) fn play(&self, event_local_id: usize) -> Option<AudioSound> {
        let event_asset = self.inner.event_assets.get(&event_local_id)?;
        let volume = event_asset.volume * self.volume();
        if volume <= 0.0 {
            return None;
        }
        let source = self.inner.assets.borrow().get(event_asset.global_id)?;
        let engine = self.engine().unwrap_or_else(AudioEngine::runtime_engine);
        let sound = engine.play(
            source,
            engine.time_in_frames(),
            0,
            0,
            Some(self.inner.artboard_id),
        )?;
        if volume != 1.0 {
            sound.set_volume(volume);
        }
        Some(sound)
    }

    pub(crate) fn stop_artboard(&self) {
        // Pinned external-engine teardown consults only Artboard::m_audioEngine.
        // A sound played through RuntimeEngine fallback therefore remains
        // runtime-owned when an engine-less Artboard is destroyed.
        if let Some(engine) = self.engine() {
            engine.stop_artboard(self.inner.artboard_id);
        }
    }
}
