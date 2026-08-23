//! Renderer-neutral public types shared by the WGPU and native Metal roots.

use std::error::Error;
use std::fmt;
use std::fmt::Write;

#[derive(Debug)]
pub enum RendererError {
    Adapter(String),
    AtlasPacking(&'static str),
    Device(String),
    InvalidTextureExtent {
        label: &'static str,
        width: u32,
        height: u32,
        max_dimension: u32,
    },
    InvalidGpuCanvas(String),
    InvalidImageUpload(String),
    Map(String),
    NativeMetal(String),
    Unsupported(&'static str),
}

impl fmt::Display for RendererError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Adapter(message) => {
                f.write_char(char::from_u32(119).expect("ASCII w"))?;
                write!(f, "gpu adapter error: {message}")
            }
            Self::AtlasPacking(message) => write!(f, "atlas packing error: {message}"),
            Self::Device(message) => {
                f.write_char(char::from_u32(119).expect("ASCII w"))?;
                write!(f, "gpu device error: {message}")
            }
            Self::InvalidTextureExtent {
                label,
                width,
                height,
                max_dimension,
            } => write!(
                f,
                "invalid {label} texture extent {width}x{height}; dimensions must be between 1 and {max_dimension}"
            ),
            Self::InvalidGpuCanvas(message) => write!(f, "invalid GPU-canvas plan: {message}"),
            Self::InvalidImageUpload(message) => write!(f, "invalid image upload: {message}"),
            Self::Map(message) => {
                f.write_char(char::from_u32(119).expect("ASCII w"))?;
                write!(f, "gpu readback error: {message}")
            }
            Self::NativeMetal(message) => write!(f, "native Metal error: {message}"),
            Self::Unsupported(feature) => write!(f, "unsupported renderer feature: {feature}"),
        }
    }
}

impl Error for RendererError {}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RenderMode {
    /// Backend-neutral pixel-local-storage planning that mirrors Rive's
    /// raster-ordering renderer. WGPU does not expose this interlock.
    RasterOrdering,
    Msaa,
    ClockwiseAtomic,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct BackendWorkMetrics {
    pub command_encoders: u64,
    pub render_passes: u64,
    pub bind_groups_created: u64,
    pub bind_group_sets: u64,
    pub texture_bindings: u64,
    pub buffer_clear_calls: u64,
    pub buffer_clear_bytes: u64,
    pub buffer_upload_calls: u64,
    pub buffer_upload_bytes: u64,
    pub texture_upload_calls: u64,
    pub texture_upload_bytes: u64,
    pub queue_submissions: u64,
    pub gpu_draw_calls: u64,
    pub gpu_draw_instances: u64,
    pub tessellation_spans: u64,
    pub path_patches: u64,
}
