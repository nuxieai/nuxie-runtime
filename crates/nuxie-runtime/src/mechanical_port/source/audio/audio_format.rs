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
