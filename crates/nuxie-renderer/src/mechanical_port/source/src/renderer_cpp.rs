/*
 * Copyright 2022 Rive
 */

// #include "rive/math/mat2d.hpp"
// #include "rive/renderer.hpp"
// #include "rive/text_engine.hpp"

// Mechanical translation of the complete pinned source implementation
// src/renderer.cpp.
// Upstream source revision: 4ac7b32798da0482e441ef09304dc3b480ed3ee5

#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(non_upper_case_globals)]

use core::ffi::c_void;

// The generic renderer header is the adjacent source-shaped owner. The
// backend-neutral value types are borrowed from the render-api owner; these
// aliases preserve the pinned C++ spellings without introducing a second
// value-type implementation.
use crate::mechanical_port::source::include::rive::refcnt_hpp::RefCnt;
use crate::mechanical_port::source::include::rive::renderer_hpp::{
    RenderBuffer, RenderBufferContract, RenderBufferFlags, RenderImage, RenderPaint, RenderPath,
    RenderShader, RendererContract,
};
use crate::mechanical_port::source::include::utils::lite_rtti_hpp::{LiteRttiTypeId, CONST_ID};
use core::ops::{Index, IndexMut};
use nuxie_render_api::{Aabb as AABB, Fit, Mat2D, Vec2D, Vec2D as Alignment};

// `using Unichar = uint32_t` from the mapped text-engine dependency.
pub type Unichar = u32;
pub type GlyphID = u16;

#[derive(Clone, Copy)]
pub struct Span<'a, T> {
    data: &'a [T],
}

impl<'a, T> Span<'a, T> {
    pub const fn new(data: &'a [T]) -> Self {
        Self { data }
    }
    pub const fn len(&self) -> usize {
        self.data.len()
    }
    pub fn iter(&self) -> core::slice::Iter<'_, T> {
        self.data.iter()
    }
}

impl<T> Index<usize> for Span<'_, T> {
    type Output = T;
    fn index(&self, index: usize) -> &T {
        &self.data[index]
    }
}

#[derive(Default)]
pub struct SimpleArray<T> {
    values: Vec<T>,
}

impl<T> SimpleArray<T> {
    pub fn new(reserve: usize) -> Self {
        Self {
            values: Vec::with_capacity(reserve),
        }
    }
    pub fn add(&mut self, value: T) {
        self.values.push(value);
    }
    pub fn len(&self) -> usize {
        self.values.len()
    }
    pub fn is_empty(&self) -> bool {
        self.values.is_empty()
    }
    pub fn back(&self) -> T
    where
        T: Copy,
    {
        *self.values.last().unwrap()
    }
    pub fn iter(&self) -> core::slice::Iter<'_, T> {
        self.values.iter()
    }
    pub fn iter_mut(&mut self) -> core::slice::IterMut<'_, T> {
        self.values.iter_mut()
    }
}

impl<T> Index<usize> for SimpleArray<T> {
    type Output = T;
    fn index(&self, index: usize) -> &T {
        &self.values[index]
    }
}

impl<T> IndexMut<usize> for SimpleArray<T> {
    fn index_mut(&mut self, index: usize) -> &mut T {
        &mut self.values[index]
    }
}

pub type SimpleArrayBuilder<T> = SimpleArray<T>;

#[repr(C)]
pub struct Font {
    base: RefCnt<Font>,
    on_shape_text: for<'a> unsafe fn(
        *const Font,
        Span<'a, Unichar>,
        Span<'a, TextRun>,
        i32,
    ) -> SimpleArray<Paragraph>,
}

unsafe fn default_on_shape_text(
    _: *const Font,
    _: Span<'_, Unichar>,
    _: Span<'_, TextRun>,
    _: i32,
) -> SimpleArray<Paragraph> {
    SimpleArray::new(0)
}

impl Font {
    pub fn new(
        on_shape_text: for<'a> unsafe fn(
            *const Font,
            Span<'a, Unichar>,
            Span<'a, TextRun>,
            i32,
        ) -> SimpleArray<Paragraph>,
    ) -> Self {
        Self {
            base: RefCnt::new(),
            on_shape_text,
        }
    }

    pub fn new_with_default_shape_dispatch() -> Self {
        Self::new(default_on_shape_text)
    }
}

// SAFETY: Font's intrusive RefCnt base is stored at offset zero and owns the
// exact complete Font allocation used by shaping runs.
unsafe impl crate::mechanical_port::source::include::rive::refcnt_hpp::RefCntTarget for Font {
    fn r#ref(&self) {
        self.base.r#ref();
    }
    unsafe fn unref(&self) {
        unsafe { self.base.unref() };
    }
}

#[repr(C)]
pub struct TextRun {
    pub font: crate::mechanical_port::source::include::rive::refcnt_hpp::rcp<Font>,
    pub size: f32,
    pub lineHeight: f32,
    pub letterSpacing: f32,
    pub unicharCount: u32,
    pub script: u32,
    pub styleId: u16,
    pub level: u8,
}

pub struct GlyphRun {
    pub font: crate::mechanical_port::source::include::rive::refcnt_hpp::rcp<Font>,
    pub size: f32,
    pub lineHeight: f32,
    pub letterSpacing: f32,
    pub glyphs: SimpleArray<GlyphID>,
    pub textIndices: SimpleArray<u32>,
    pub advances: SimpleArray<f32>,
    pub xpos: SimpleArray<f32>,
    pub offsets: SimpleArray<Vec2D>,
    pub breaks: SimpleArray<u32>,
    pub styleId: u16,
    pub level: u8,
    pub joiners: SimpleArray<u32>,
}

pub struct Paragraph {
    pub runs: SimpleArray<GlyphRun>,
    pub level: u8,
}

// namespace rive

// Mat2D::fromScale/fromTranslate and operator* are provided by the pinned
// math header in C++. The mapped render-api Mat2D is an opaque six-float value,
// so these narrow helpers preserve the same source multiplication order while
// keeping the adaptation local to this translation unit.
#[inline(always)]
fn fromScale(sx: f32, sy: f32) -> Mat2D {
    Mat2D([sx, 0.0, 0.0, sy, 0.0, 0.0])
}

#[inline(always)]
fn fromTranslate(tx: f32, ty: f32) -> Mat2D {
    Mat2D([1.0, 0.0, 0.0, 1.0, tx, ty])
}

#[inline(always)]
fn multiply(lhs: Mat2D, rhs: Mat2D) -> Mat2D {
    let a = lhs.0;
    let b = rhs.0;
    Mat2D([
        a[0].mul_add(b[0], a[2] * b[1]),
        a[1].mul_add(b[0], a[3] * b[1]),
        a[0].mul_add(b[2], a[2] * b[3]),
        a[1].mul_add(b[2], a[3] * b[3]),
        a[0].mul_add(b[4], a[2] * b[5]) + a[4],
        a[1].mul_add(b[4], a[3] * b[5]) + a[5],
    ])
}

// Mat2D rive::computeAlignment(Fit fit,
//                              Alignment alignment,
//                              const AABB& frame,
//                              const AABB& content,
//                              const float scaleFactor)
pub fn computeAlignment(
    fit: Fit,
    alignment: Alignment,
    frame: &AABB,
    content: &AABB,
    scaleFactor: f32,
) -> Mat2D {
    // float contentWidth = content.width();
    let contentWidth = content.width();
    // float contentHeight = content.height();
    let contentHeight = content.height();
    // float x = -content.left() - contentWidth * 0.5f -
    //           (alignment.x() * contentWidth * 0.5f);
    let x = -content.min_x - contentWidth * 0.5 - (alignment.x * contentWidth * 0.5);
    // float y = -content.top() - contentHeight * 0.5f -
    //           (alignment.y() * contentHeight * 0.5f);
    let y = -content.min_y - contentHeight * 0.5 - (alignment.y * contentHeight * 0.5);

    // float scaleX = 1.0f, scaleY = 1.0f;
    let (mut scaleX, mut scaleY) = (1.0_f32, 1.0_f32);

    // switch (fit)
    match fit {
        // case Fit::fill:
        Fit::Fill => {
            // scaleX = frame.width() / contentWidth;
            scaleX = frame.width() / contentWidth;
            // scaleY = frame.height() / contentHeight;
            scaleY = frame.height() / contentHeight;
        }
        // case Fit::contain:
        Fit::Contain => {
            // float minScale = std::fmin(frame.width() / contentWidth,
            //                            frame.height() / contentHeight);
            let minScale = (frame.width() / contentWidth).min(frame.height() / contentHeight);
            // scaleX = scaleY = minScale;
            scaleX = minScale;
            scaleY = minScale;
        }
        // case Fit::cover:
        Fit::Cover => {
            // float maxScale = std::fmax(frame.width() / contentWidth,
            //                            frame.height() / contentHeight);
            let maxScale = (frame.width() / contentWidth).max(frame.height() / contentHeight);
            // scaleX = scaleY = maxScale;
            scaleX = maxScale;
            scaleY = maxScale;
        }
        // case Fit::fitHeight:
        Fit::FitHeight => {
            // float minScale = frame.height() / contentHeight;
            let minScale = frame.height() / contentHeight;
            // scaleX = scaleY = minScale;
            scaleX = minScale;
            scaleY = minScale;
        }
        // case Fit::fitWidth:
        Fit::FitWidth => {
            // float minScale = frame.width() / contentWidth;
            let minScale = frame.width() / contentWidth;
            // scaleX = scaleY = minScale;
            scaleX = minScale;
            scaleY = minScale;
        }
        // case Fit::layout:
        Fit::Layout => {
            // scaleX = scaleY = scaleFactor;
            scaleX = scaleFactor;
            scaleY = scaleFactor;
        }
        // case Fit::none:
        Fit::None => {
            // scaleX = scaleY = 1.0f;
            scaleX = 1.0;
            scaleY = 1.0;
        }
        // case Fit::scaleDown:
        Fit::ScaleDown => {
            // float minScale = std::fmin(frame.width() / contentWidth,
            //                            frame.height() / contentHeight);
            let minScale = (frame.width() / contentWidth).min(frame.height() / contentHeight);
            // scaleX = scaleY = minScale < 1.0f ? minScale : 1.0f;
            let scale = if minScale < 1.0 { minScale } else { 1.0 };
            scaleX = scale;
            scaleY = scale;
        }
    }

    // Mat2D translation;
    let mut translation = Mat2D::IDENTITY;
    // translation[4] = frame.left() + frame.width() * 0.5f +
    //                   (alignment.x() * frame.width() * 0.5f);
    translation.0[4] = frame.min_x + frame.width() * 0.5 + (alignment.x * frame.width() * 0.5);
    // translation[5] = frame.top() + frame.height() * 0.5f +
    //                   (alignment.y() * frame.height() * 0.5f);
    translation.0[5] = frame.min_y + frame.height() * 0.5 + (alignment.y * frame.height() * 0.5);

    // return translation * Mat2D::fromScale(scaleX, scaleY) *
    //        Mat2D::fromTranslate(x, y);
    multiply(
        multiply(translation, fromScale(scaleX, scaleY)),
        fromTranslate(x, y),
    )
}

// void Renderer::translate(float tx, float ty)
// void Renderer::scale(float sx, float sy)
// void Renderer::rotate(float radians)
// The C++ methods dispatch through Renderer::transform. Rust has no base-class
// virtual subobject, so the header's RendererContract is extended here while
// retaining the exact helper order and transform matrices.
pub trait RendererHelpers: RendererContract {
    fn translate(&mut self, tx: f32, ty: f32) {
        // this->transform(Mat2D(1, 0, 0, 1, tx, ty));
        RendererContract::transform(self, &Mat2D([1.0, 0.0, 0.0, 1.0, tx, ty]));
    }

    fn scale(&mut self, sx: f32, sy: f32) {
        // this->transform(Mat2D(sx, 0, 0, sy, 0, 0));
        RendererContract::transform(self, &Mat2D([sx, 0.0, 0.0, sy, 0.0, 0.0]));
    }

    fn rotate(&mut self, radians: f32) {
        // const float s = std::sin(radians);
        let s = radians.sin();
        // const float c = std::cos(radians);
        let c = radians.cos();
        // this->transform(Mat2D(c, s, -s, c, 0, 0));
        RendererContract::transform(self, &Mat2D([c, s, -s, c, 0.0, 0.0]));
    }
}

impl<T: RendererContract + ?Sized> RendererHelpers for T {}

// RenderBuffer::RenderBuffer(RenderBufferType type,
//                             RenderBufferFlags flags,
//                             size_t sizeInBytes)
impl RenderBuffer {
    /// Constructs the flattened abstract base for a concrete offset-zero
    /// backend owner.
    ///
    /// # Safety
    /// `Owner` must be the complete allocation containing this RenderBuffer at
    /// offset zero. The installed virtual slots cast the base pointer back to
    /// that exact live owner.
    pub unsafe fn new_for_owner<Owner: RenderBufferContract + LiteRttiTypeId>(
        m_type: crate::mechanical_port::source::include::rive::renderer_hpp::RenderBufferType,
        m_flags: RenderBufferFlags,
        m_sizeInBytes: usize,
    ) -> Self {
        unsafe fn destroy_complete<Owner>(ptr: *mut RenderBuffer) {
            unsafe { drop(Box::from_raw(ptr.cast::<Owner>())) };
        }
        unsafe fn on_map<Owner: RenderBufferContract>(ptr: *mut RenderBuffer) -> *mut c_void {
            unsafe { RenderBufferContract::onMap(&mut *ptr.cast::<Owner>()) }
        }
        unsafe fn on_unmap<Owner: RenderBufferContract>(ptr: *mut RenderBuffer) {
            unsafe { RenderBufferContract::onUnmap(&mut *ptr.cast::<Owner>()) };
        }
        Self {
            base: RefCnt::new(),
            destroy_complete: destroy_complete::<Owner>,
            on_map: on_map::<Owner>,
            on_unmap: on_unmap::<Owner>,
            m_liteTypeId: Owner::LITE_RTTI_TYPE_ID,
            m_type,
            m_flags,
            m_sizeInBytes,
            m_dirty: false,
            #[cfg(debug_assertions)]
            m_mapCount: 0,
            #[cfg(debug_assertions)]
            m_unmapCount: 0,
        }
    }

    // RenderBuffer::~RenderBuffer()
    // The empty C++ destructor remains an explicit Rust Drop owner boundary.
}

impl Drop for RenderBuffer {
    fn drop(&mut self) {}
}

impl RenderBuffer {
    // void* RenderBuffer::map()
    //
    pub fn map(&mut self) -> *mut c_void {
        #[cfg(debug_assertions)]
        {
            assert!(
                self.m_mapCount == 0
                    || (self.m_flags as u8 & RenderBufferFlags::mappedOnceAtInitialization as u8)
                        == 0
            );
            assert!(self.m_mapCount == self.m_unmapCount);
            // RIVE_DEBUG_CODE(++m_mapCount;)
            self.m_mapCount += 1;
        }
        // m_dirty = true;
        self.m_dirty = true;
        // return onMap();
        unsafe { (self.on_map)(self) }
    }

    // void RenderBuffer::unmap()
    pub fn unmap(&mut self) {
        #[cfg(debug_assertions)]
        {
            // assert(m_unmapCount + 1 == m_mapCount);
            assert!(self.m_unmapCount + 1 == self.m_mapCount);
            // RIVE_DEBUG_CODE(++m_unmapCount;)
            self.m_unmapCount += 1;
        }
        // onUnmap();
        unsafe { (self.on_unmap)(self) };
    }
}

// RenderShader::RenderShader()
// RenderShader::~RenderShader()
impl RenderShader {
    pub fn new() -> Self {
        Self {
            base: RefCnt::new(),
            destroy_complete: |ptr| unsafe { drop(Box::from_raw(ptr)) },
            m_liteTypeId: CONST_ID("RenderShader"),
        }
    }

    /// # Safety
    /// `Owner` must be the complete offset-zero derived allocation.
    pub unsafe fn new_for_owner<Owner: LiteRttiTypeId>() -> Self {
        unsafe fn destroy_complete<Owner>(ptr: *mut RenderShader) {
            unsafe { drop(Box::from_raw(ptr.cast::<Owner>())) };
        }
        Self {
            base: RefCnt::new(),
            destroy_complete: destroy_complete::<Owner>,
            m_liteTypeId: Owner::LITE_RTTI_TYPE_ID,
        }
    }
}

impl Drop for RenderShader {
    fn drop(&mut self) {}
}

// RenderPaint::RenderPaint()
// RenderPaint::~RenderPaint()
impl RenderPaint {
    pub fn new() -> Self {
        Self {
            base: RefCnt::new(),
            destroy_complete: |ptr| unsafe { drop(Box::from_raw(ptr)) },
            m_liteTypeId: CONST_ID("RenderPaint"),
        }
    }

    /// # Safety
    /// `Owner` must be the complete offset-zero derived allocation.
    pub unsafe fn new_for_owner<Owner: LiteRttiTypeId>() -> Self {
        unsafe fn destroy_complete<Owner>(ptr: *mut RenderPaint) {
            unsafe { drop(Box::from_raw(ptr.cast::<Owner>())) };
        }
        Self {
            base: RefCnt::new(),
            destroy_complete: destroy_complete::<Owner>,
            m_liteTypeId: Owner::LITE_RTTI_TYPE_ID,
        }
    }
}

impl Drop for RenderPaint {
    fn drop(&mut self) {}
}

// RenderImage::RenderImage(const Mat2D& uvTransform)
impl RenderImage {
    pub fn new_with_uv_transform(uvTransform: &Mat2D) -> Self {
        Self {
            base: RefCnt::new(),
            destroy_complete: |ptr| unsafe { drop(Box::from_raw(ptr)) },
            m_liteTypeId: CONST_ID("RenderImage"),
            m__width: 0,
            m__height: 0,
            m_uv_transform: *uvTransform,
            #[cfg(target_os = "emscripten")]
            m_delegate: None,
        }
    }

    // RenderImage::RenderImage()
    pub fn new() -> Self {
        Self {
            base: RefCnt::new(),
            destroy_complete: |ptr| unsafe { drop(Box::from_raw(ptr)) },
            m_liteTypeId: CONST_ID("RenderImage"),
            m__width: 0,
            m__height: 0,
            m_uv_transform: Mat2D::IDENTITY,
            #[cfg(target_os = "emscripten")]
            m_delegate: None,
        }
    }
}

// RenderImage::~RenderImage()
impl Drop for RenderImage {
    fn drop(&mut self) {}
}

// RenderPath::~RenderPath()
impl Drop for RenderPath {
    fn drop(&mut self) {}
}

// bool rive::isWhiteSpace(Unichar c)
pub fn isWhiteSpace(c: Unichar) -> bool {
    // 0x2028 is a Line separator.
    // 0x200B is a Zero width space.
    c <= b' ' as u32 || c == 0x2028 || c == 0x200B
}

// Font::shapeText(Span<const Unichar> text,
//                 Span<const TextRun> runs,
//                 int textDirectionFlag) const
//
// text_engine.hpp is a generic dependency owner rather than a campaign source
// file. This trait is the Rust spelling of Font's virtual onShapeText seam;
// every source branch and post-shape mutation remains in this implementation.
pub trait FontShapeTextContract {
    fn onShapeText(
        &self,
        text: Span<'_, Unichar>,
        runs: Span<'_, TextRun>,
        textDirectionFlag: i32,
    ) -> SimpleArray<Paragraph>;

    fn shapeText(
        &self,
        text: Span<'_, Unichar>,
        runs: Span<'_, TextRun>,
        textDirectionFlag: i32,
    ) -> SimpleArray<Paragraph> {
        // #ifdef DEBUG
        #[cfg(debug_assertions)]
        {
            let mut count = 0usize;
            for tr in runs.iter() {
                // assert(tr.unicharCount > 0);
                assert!(tr.unicharCount > 0);
                // count += tr.unicharCount;
                count += tr.unicharCount as usize;
            }
            // assert(count <= text.size());
            assert!(count <= text.len());
        }
        // #endif

        // SimpleArray<Paragraph> paragraphs =
        //     onShapeText(text, runs, textDirectionFlag);
        let mut paragraphs = self.onShapeText(text, runs, textDirectionFlag);
        // bool wantWhiteSpace = false;
        let mut wantWhiteSpace = false;
        // GlyphRun* lastRun = nullptr;
        let mut lastRun: *mut GlyphRun = core::ptr::null_mut();
        // size_t reserveSize = text.size() / 4;
        let reserveSize = text.len() / 4;
        // SimpleArrayBuilder<uint32_t> breakBuilder(reserveSize);
        let mut breakBuilder = SimpleArrayBuilder::<u32>::new(reserveSize);
        // SimpleArrayBuilder<uint32_t> joinerBuilder(reserveSize);
        let mut joinerBuilder = SimpleArrayBuilder::<u32>::new(reserveSize);
        // for (const Paragraph& para : paragraphs)
        for para in paragraphs.iter_mut() {
            // for (GlyphRun& gr : para.runs)
            for gr in para.runs.iter_mut() {
                if !lastRun.is_null() {
                    // lastRun->breaks = std::move(breakBuilder);
                    unsafe {
                        (*lastRun).breaks = breakBuilder;
                        (*lastRun).joiners = joinerBuilder;
                    }
                    // Reset the builder.
                    // breakBuilder = SimpleArrayBuilder<uint32_t>(reserveSize);
                    breakBuilder = SimpleArrayBuilder::<u32>::new(reserveSize);
                    // joinerBuilder = SimpleArrayBuilder<uint32_t>(reserveSize);
                    joinerBuilder = SimpleArrayBuilder::<u32>::new(reserveSize);
                }
                // uint32_t glyphIndex = 0;
                let mut glyphIndex = 0u32;
                // for (uint32_t offset : gr.textIndices)
                for offset in gr.textIndices.iter().copied() {
                    // Unichar unicode = text[offset];
                    let unicode = text[offset as usize];
                    if unicode == '\n' as u32 || unicode == 0x2028 {
                        // breakBuilder.add(glyphIndex);
                        breakBuilder.add(glyphIndex);
                        // breakBuilder.add(glyphIndex);
                        breakBuilder.add(glyphIndex);
                    }
                    if unicode == 0x2060 {
                        // joinerBuilder.add(offset);
                        joinerBuilder.add(offset);
                    }
                    if wantWhiteSpace == isWhiteSpace(unicode) {
                        // breakBuilder.add(glyphIndex);
                        breakBuilder.add(glyphIndex);
                        // wantWhiteSpace = !wantWhiteSpace;
                        wantWhiteSpace = !wantWhiteSpace;
                    }
                    // glyphIndex++;
                    glyphIndex += 1;
                }

                // lastRun = &gr;
                lastRun = gr as *mut GlyphRun;
            }
        }
        if !lastRun.is_null() {
            if wantWhiteSpace {
                // breakBuilder.add((uint32_t)lastRun->glyphs.size());
                unsafe {
                    breakBuilder.add((*lastRun).glyphs.len() as u32);
                }
            } else {
                // Consume the rest of the run.
                // breakBuilder.add(breakBuilder.empty() ? 0 : breakBuilder.back());
                breakBuilder.add(if breakBuilder.is_empty() {
                    0
                } else {
                    breakBuilder.back()
                });
                // breakBuilder.add((uint32_t)lastRun->glyphs.size());
                unsafe {
                    breakBuilder.add((*lastRun).glyphs.len() as u32);
                }
            }
            unsafe {
                // lastRun->breaks = std::move(breakBuilder);
                (*lastRun).breaks = breakBuilder;
                // lastRun->joiners = std::move(joinerBuilder);
                (*lastRun).joiners = joinerBuilder;
            }
        }

        // #ifdef DEBUG
        #[cfg(debug_assertions)]
        for para in paragraphs.iter() {
            for gr in para.runs.iter() {
                // assert(gr.glyphs.size() > 0);
                assert!(gr.glyphs.len() > 0);
                // assert(gr.glyphs.size() == gr.textIndices.size());
                assert!(gr.glyphs.len() == gr.textIndices.len());
                // assert(gr.glyphs.size() + 1 == gr.xpos.size());
                assert!(gr.glyphs.len() + 1 == gr.xpos.len());
            }
        }
        // #endif
        // return paragraphs;
        paragraphs
    }
}

impl FontShapeTextContract for Font {
    fn onShapeText(
        &self,
        text: Span<'_, Unichar>,
        runs: Span<'_, TextRun>,
        textDirectionFlag: i32,
    ) -> SimpleArray<Paragraph> {
        unsafe { (self.on_shape_text)(self, text, runs, textDirectionFlag) }
    }
}

// } // namespace rive
