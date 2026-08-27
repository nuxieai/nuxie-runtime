use crate::mechanical_port::source::generated::assets::export_audio_base::ExportAudioBase;

pub struct ExportAudio {
    pub base: ExportAudioBase,
}

impl Default for ExportAudio {
    fn default() -> Self {
        Self {
            base: ExportAudioBase::default(),
        }
    }
}
