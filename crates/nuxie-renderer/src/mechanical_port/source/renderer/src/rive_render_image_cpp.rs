/*
 * Copyright 2022 Rive
 */

// #include "rive/renderer/rive_render_image.hpp"

// Mechanical translation of the complete pinned source implementation
// renderer/src/rive_render_image.cpp.
// Upstream source revision: 4ac7b32798da0482e441ef09304dc3b480ed3ee5

#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(non_upper_case_globals)]

use core::sync::atomic::{AtomicU32, Ordering};

use crate::mechanical_port::source::include::rive::refcnt_hpp::RefCnt;
use crate::mechanical_port::source::renderer::include::rive::renderer::texture_hpp::{
    null_native_handle, Texture,
};

// namespace rive::gpu
// {

// Texture::Texture(uint32_t width, uint32_t height) :
//     m_width(width), m_height(height)
impl Texture {
    pub fn new(width: u32, height: u32) -> Self {
        // The implicit RefCnt<Texture> base constructor runs before the
        // source member initializers.
        let base = RefCnt::new();
        let m_width = width;
        let m_height = height;

        // static std::atomic_uint32_t textureResourceHashCounter = 0;
        static textureResourceHashCounter: AtomicU32 = AtomicU32::new(0);

        // m_textureResourceHash = ++textureResourceHashCounter;
        // `fetch_add` is the source atomic pre-increment's sequentially
        // consistent operation; `wrapping_add` preserves uint32_t overflow.
        let m_textureResourceHash = textureResourceHashCounter
            .fetch_add(1, Ordering::SeqCst)
            .wrapping_add(1);

        Self {
            base,
            destroy_complete: |ptr| unsafe { drop(Box::from_raw(ptr)) },
            m_width,
            m_height,
            // uint32_t pre-increment wraps modulo 2^32. Zero is therefore an
            // authored, observable value after wraparound and must not panic.
            m_textureResourceHash,
            m_nativeHandle: null_native_handle,
        }
    }
}

// } // namespace rive::gpu
