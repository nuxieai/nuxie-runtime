//! Native Metal adaptation for a caller-owned Core Animation drawable.
//!
//! This seam keeps caller-owned presentation separate from the exact Metal
//! renderer's command and resource ownership.

use objc2::runtime::ProtocolObject;
use objc2_quartz_core::CAMetalDrawable;

use crate::{NativeMetalDrawableFrame, NativeMetalFactory, RendererError};

impl NativeMetalFactory {
    /// Begins a native Metal frame from one caller-acquired drawable. The
    /// texture is derived from that same object so rendering and presentation
    /// cannot accidentally target different native owners. The frame retains
    /// that drawable through replay and presentation.
    pub fn begin_drawable_frame(
        &self,
        drawable: &ProtocolObject<dyn CAMetalDrawable>,
        clear_color: u32,
    ) -> Result<NativeMetalDrawableFrame, RendererError> {
        self.begin_drawable_frame_parts(drawable.as_ref(), drawable.texture(), clear_color)
    }
}
