//! Complete mechanical declaration translation of
//! `renderer/src/ore/gl/ore_buffer_gl.hpp`.

#![allow(non_snake_case)]

use nuxie_ore_metal::buffer::{Buffer, BufferApi, BufferUpdateError};
use nuxie_ore_metal::gpu_resource::{GPUResource, GpuResourcePayload};
use nuxie_ore_metal::types::BufferUsage;
use std::mem::ManuallyDrop;
use std::ops::{Deref, DerefMut};

use super::gles3_decl::GLExecutionStamp;

pub(crate) const PINNED_SOURCE: &str = include_str!("source/renderer_src_ore_gl_ore_buffer_gl.hpp");

#[repr(C)]
pub(crate) struct BufferGL {
    pub(crate) base: ManuallyDrop<Buffer>,
    pub(crate) m_glBuffer: u32,
    pub(crate) m_glTarget: u32,
    /// Rust execution/lifetime sidecar after the complete source prefix.
    pub(crate) rust_execution: GLExecutionStamp,
}

impl BufferGL {
    pub(crate) fn new(size: u32, usage: BufferUsage, execution: GLExecutionStamp) -> Self {
        Self {
            base: ManuallyDrop::new(nuxie_ore_metal::new_buffer_backend_base_without_manager(
                size, usage,
            )),
            m_glBuffer: 0,
            m_glTarget: 0,
            rust_execution: execution,
        }
    }

    pub(crate) fn executionStamp(&self) -> &GLExecutionStamp {
        &self.rust_execution
    }
}

impl Deref for BufferGL {
    type Target = Buffer;
    fn deref(&self) -> &Self::Target {
        &self.base
    }
}

impl DerefMut for BufferGL {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.base
    }
}

unsafe impl GpuResourcePayload for BufferGL {
    fn gpu_resource(&self) -> &GPUResource {
        self.base.gpu_resource()
    }
    fn gpu_resource_mut(&mut self) -> &mut GPUResource {
        self.base.gpu_resource_mut()
    }
}

impl BufferApi for BufferGL {
    fn size(&self) -> u32 {
        self.base.size()
    }
    fn usage(&self) -> BufferUsage {
        self.base.usage()
    }
    fn update(&self, data: &[u8], size: u32, offset: u32) -> Result<(), BufferUpdateError> {
        super::ore_buffer_gl_impl::update(self, data, size, offset)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::mem::offset_of;

    #[test]
    fn complete_header_denominator_and_layout_are_frozen() {
        assert_eq!(PINNED_SOURCE.lines().count(), 22);
        assert_eq!(offset_of!(BufferGL, base), 0);
        assert_eq!(
            offset_of!(BufferGL, m_glBuffer),
            std::mem::size_of::<Buffer>()
        );
        assert!(offset_of!(BufferGL, rust_execution) > offset_of!(BufferGL, m_glTarget));
        assert!(std::mem::size_of::<BufferGL>() > std::mem::size_of::<Buffer>() + 8);
    }
}
