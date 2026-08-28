#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
#[repr(u32)]
pub enum AudioFormat {
    #[default]
    Unknown = 0,
    Wav,
    Flac,
    Mp3,
    Vorbis,
    Buffered,
}

impl From<nuxie_audio::AudioFormat> for AudioFormat {
    fn from(value: nuxie_audio::AudioFormat) -> Self {
        match value {
            nuxie_audio::AudioFormat::Unknown => Self::Unknown,
            nuxie_audio::AudioFormat::Wav => Self::Wav,
            nuxie_audio::AudioFormat::Flac => Self::Flac,
            nuxie_audio::AudioFormat::Mp3 => Self::Mp3,
            nuxie_audio::AudioFormat::Vorbis => Self::Vorbis,
            nuxie_audio::AudioFormat::Buffered => Self::Buffered,
        }
    }
}
