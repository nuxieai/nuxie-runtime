// Mechanical translation of:
//   renderer/include/rive/renderer/ore/ore_buffer.hpp
// Upstream source revision: 4ac7b32798da0482e441ef09304dc3b480ed3ee5
//
// Copyright 2025 Rive

use std::fmt;

use crate::types::BufferUsage;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BufferUpdateError {
    UnsupportedPlatform,
    SizeOverflow,
    OutOfBounds {
        offset: u32,
        size: u32,
        buffer_size: u32,
    },
}

impl fmt::Display for BufferUpdateError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnsupportedPlatform => {
                formatter.write_str("Metal buffer updates require an Apple target")
            }
            Self::SizeOverflow => formatter.write_str("buffer update size exceeds u32"),
            Self::OutOfBounds {
                offset,
                size,
                buffer_size,
            } => write!(
                formatter,
                "buffer update range {offset}..{} exceeds buffer size {buffer_size}",
                offset.saturating_add(*size)
            ),
        }
    }
}

impl std::error::Error for BufferUpdateError {}

/// Backend-neutral immutable state shared by concrete ORE buffers.
pub struct BufferBase {
    size: u32,
    usage: BufferUsage,
}

impl BufferBase {
    pub fn new(size: u32, usage: BufferUsage) -> Self {
        Self { size, usage }
    }

    pub fn size(&self) -> u32 {
        self.size
    }

    pub fn usage(&self) -> BufferUsage {
        self.usage
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn buffer_base_preserves_size_and_usage() {
        let buffer = BufferBase::new(4096, BufferUsage::uniform);
        assert_eq!(buffer.size(), 4096);
        assert_eq!(buffer.usage(), BufferUsage::uniform);
    }
}
