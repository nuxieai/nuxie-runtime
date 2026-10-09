use crate::mechanical_port::source::{
    artboard::Artboard,
    assets::{audio_asset::AudioAsset, file_asset_referencer::FileAssetReferencer},
    audio::audio_engine::AudioEngine,
    core::{CoreHandle, field_types::core_callback_type::CallbackData},
    generated::{
        assets::audio_asset_base::AudioAssetBase,
        audio_event_base::{AudioEventBase, AudioEventBaseCallbacks},
    },
    importers::import_stack::ImportStack,
    status_code::StatusCode,
};

#[derive(Default)]
pub struct AudioEvent {
    pub base: AudioEventBase,
    file_asset_referencer: FileAssetReferencer,
}

impl AudioEvent {
    pub fn play(&mut self) {
        let Some(asset) = self.file_asset_referencer.asset() else {
            return;
        };
        let Some((audio_source, asset_volume)) = asset
            .with_downcast::<AudioAsset, _>(|asset| {
                Some((asset.audio_source()?, asset.base.volume()))
            })
            .flatten()
        else {
            return;
        };
        let Some(artboard) = self.base.base.base.base.artboard_handle() else {
            return;
        };
        let Some((volume, engine, identity)) = artboard
            .with_downcast::<Artboard, _>(|artboard| {
                let volume = asset_volume * artboard.volume();
                // Source rejects silence before resolving/creating an engine.
                if volume <= 0.0 {
                    return None;
                }
                let engine = artboard
                    .audio_engine_handle()
                    .or_else(|| AudioEngine::runtime_engine(true))?;
                Some((
                    volume,
                    engine,
                    artboard.runtime_weak_handle().audio_identity()?,
                ))
            })
            .flatten()
        else {
            return;
        };
        let time = engine.time_in_frames();
        if let Some(sound) = AudioEngine::play(&engine, audio_source, time, 0, 0, Some(identity)) {
            if volume != 1.0 {
                sound.set_volume(volume);
            }
        }
    }

    pub fn trigger(&mut self, value: &mut CallbackData<'_>) {
        self.base.base.trigger(value);
        if !value
            .context()
            .expect("AudioEvent::trigger requires CallbackData context")
            .plays_audio()
        {
            self.play();
        }
    }

    pub fn import(&mut self, import_stack: &mut ImportStack) -> StatusCode {
        let Some(this) = self.core_handle() else {
            return StatusCode::MissingObject;
        };
        let result = self
            .file_asset_referencer
            .register_referencer(this, import_stack);
        if result != StatusCode::Ok {
            return result;
        }
        self.base.base.import(import_stack)
    }

    pub fn set_asset(&mut self, asset: Option<CoreHandle>) {
        if asset
            .as_ref()
            .is_some_and(|asset| asset.is_type_of(AudioAssetBase::TYPE_KEY))
        {
            let Some(this) = self.core_handle() else {
                return;
            };
            self.file_asset_referencer.set_asset(this, asset);
        }
    }

    pub fn clone_event(&self) -> Self {
        let mut cloned = Self::default();
        let mut base = std::mem::take(&mut cloned.base);
        base.copy(&self.base, &mut cloned);
        cloned.base = base;
        // The cloned owner acquires occurrence identity when inserted. Its
        // FileAssetReferencer attaches to that identity at the existing seam.
        if let Some(asset) = self.file_asset_referencer.asset() {
            cloned
                .file_asset_referencer
                .set_asset_unattached(Some(asset));
        }
        cloned
    }

    pub fn asset_id(&self) -> u32 {
        self.base.asset_id()
    }

    pub(crate) fn file_asset_referencer_mut(&mut self) -> &mut FileAssetReferencer {
        &mut self.file_asset_referencer
    }

    fn core_handle(&self) -> Option<CoreHandle> {
        self.base.base.base.base.base.base.base.base.base.handle()
    }
}

impl AudioEventBaseCallbacks for AudioEvent {
    fn notify_property_changed(&mut self, property_key: u16) {
        self.base.base.notify_property_changed(property_key);
    }
}

impl std::ops::Deref for AudioEvent {
    type Target = AudioEventBase;

    fn deref(&self) -> &Self::Target {
        &self.base
    }
}

impl std::ops::DerefMut for AudioEvent {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.base
    }
}
