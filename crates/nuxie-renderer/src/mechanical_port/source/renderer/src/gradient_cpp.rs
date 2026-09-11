/*
 * Copyright 2022 Rive
 */

// Mechanical translation of the complete pinned source implementation
// renderer/src/gradient.cpp.
// Upstream source revision: 4ac7b32798da0482e441ef09304dc3b480ed3ee5

// /*
//  * Copyright 2022 Rive
//  */
//
// #include "gradient.hpp"
//
// namespace rive::gpu
// {
// // Ensure the given gradient stops are in a format expected by PLS.
// static bool validate_gradient_stops(const ColorInt colors[], // [count]
//                                     const float stops[],     // [count]
//                                     size_t count)
// {
//     // Stops cannot be empty.
//     if (count == 0)
//     {
//         return false;
//     }
//     for (size_t i = 0; i < count; ++i)
//     {
//         // Stops must be finite, real numbers in the range [0, 1].
//         if (!(0 <= stops[i] && stops[i] <= 1))
//         {
//             return false;
//         }
//     }
//     for (size_t i = 1; i < count; ++i)
//     {
//         // Stops must be ordered.
//         if (!(stops[i - 1] <= stops[i]))
//         {
//             return false;
//         }
//     }
//     return true;
// }
//
// rcp<Gradient> Gradient::MakeLinear(float sx,
//                                    float sy,
//                                    float ex,
//                                    float ey,
//                                    const ColorInt colors[], // [count]
//                                    const float stops[],     // [count]
//                                    size_t count)
// {
//     if (!validate_gradient_stops(colors, stops, count))
//     {
//         return nullptr;
//     }
//
//     float2 start = {sx, sy};
//     float2 end = {ex, ey};
//     GradDataArray<ColorInt> newColors(colors, count);
//     GradDataArray<float> newStops(stops, count);
//
//     // If the stops don't begin and end on 0 and 1, transform the gradient so
//     // they do. This allows us to take full advantage of the gradient's range of
//     // pixels in the texture.
//     float firstStop = stops[0];
//     float lastStop = stops[count - 1];
//     if ((firstStop != 0 || lastStop != 1) &&
//         lastStop - firstStop > math::EPSILON)
//     {
//         // Tighten the endpoints to align with the mininum and maximum gradient
//         // stops.
//         float4 newEndpoints =
//             simd::precise_mix(start.xyxy,
//                               end.xyxy,
//                               float4{firstStop, firstStop, lastStop, lastStop});
//         start = newEndpoints.xy;
//         end = newEndpoints.zw;
//         newStops[0] = 0;
//         newStops[count - 1] = 1;
//         if (count > 2)
//         {
//             // Transform the stops into the range defined by the new endpoints.
//             float m = 1.f / (lastStop - firstStop);
//             float a = -firstStop * m;
//             for (size_t i = 1; i < count - 1; ++i)
//             {
//                 newStops[i] = stops[i] * m + a;
//             }
//
//             // Clamp the interior stops so they remain monotonically increasing.
//             // newStops[0] and newStops[count - 1] are already 0 and 1, so this
//             // also ensures they stay within 0..1.
//             for (size_t i = 1; i < count - 1; ++i)
//             {
//                 newStops[i] = fmaxf(newStops[i - 1], newStops[i]);
//             }
//             for (size_t i = count - 2; i != 0; --i)
//             {
//                 newStops[i] = fminf(newStops[i], newStops[i + 1]);
//             }
//         }
//         assert(validate_gradient_stops(newColors.get(), newStops.get(), count));
//     }
//
//     float2 v = end - start;
//     v *= 1.f / simd::dot(v, v); // dot(v, end - start) == 1
//     return rcp(new Gradient(gpu::PaintType::linearGradient,
//                             std::move(newColors),
//                             std::move(newStops),
//                             count,
//                             v.x,
//                             v.y,
//                             -simd::dot(v, start)));
// }
//
// rcp<Gradient> Gradient::MakeRadial(float cx,
//                                    float cy,
//                                    float radius,
//                                    const ColorInt colors[], // [count]
//                                    const float stops[],     // [count]
//                                    size_t count)
// {
//     if (!validate_gradient_stops(colors, stops, count))
//     {
//         return nullptr;
//     }
//
//     GradDataArray<ColorInt> newColors(colors, count);
//     GradDataArray<float> newStops(stops, count);
//
//     // If the stops don't end on 1, scale the gradient so they do. This allows
//     // us to take better advantage of the gradient's full range of pixels in the
//     // texture.
//     //
//     // TODO: If we want to take full advantage of the gradient texture pixels,
//     // we could add an inner radius that specifies where t=0 begins (instead of
//     // assuming it begins at the center).
//     float lastStop = stops[count - 1];
//     if (lastStop != 1 && lastStop > math::EPSILON)
//     {
//         // Update the gradient to finish on 1.
//         newStops[count - 1] = 1;
//
//         // Scale the radius to align with the final stop.
//         radius *= lastStop;
//
//         // Scale the stops into the range defined by the new radius.
//         float inverseLastStop = 1.f / lastStop;
//         for (size_t i = 0; i < count - 1; ++i)
//         {
//             newStops[i] = stops[i] * inverseLastStop;
//         }
//
//         if (count > 1)
//         {
//             // Clamp the stops so they remain monotonically increasing.
//             // newStops[count - 1] is already 1, so this also ensures they stay
//             // within 0..1.
//             newStops[0] = fmaxf(0, newStops[0]);
//             for (size_t i = 1; i < count - 1; ++i)
//             {
//                 newStops[i] = fmaxf(newStops[i - 1], newStops[i]);
//             }
//             for (size_t i = count - 2; i != -1; --i)
//             {
//                 newStops[i] = fminf(newStops[i], newStops[i + 1]);
//             }
//         }
//
//         assert(validate_gradient_stops(newColors.get(), newStops.get(), count));
//     }
//
//     return rcp(new Gradient(gpu::PaintType::radialGradient,
//                             std::move(newColors),
//                             std::move(newStops),
//                             count,
//                             cx,
//                             cy,
//                             radius));
// }
//
// bool Gradient::isOpaque() const
// {
//     if (m_isOpaque == gpu::TriState::unknown)
//     {
//         ColorInt allColors = ~0;
//         for (int i = 0; i < m_count; ++i)
//         {
//             allColors &= m_colors[i];
//         }
//         m_isOpaque = colorAlpha(allColors) == 0xff ? gpu::TriState::yes
//                                                    : gpu::TriState::no;
//     }
//     return m_isOpaque == gpu::TriState::yes;
// }
//
// rcp<Gradient> Gradient::getModulated(float opacity) const
// {
//     // Fast path: no modulation needed
//     if (opacity == 1.0f)
//     {
//         return ref_rcp(const_cast<Gradient*>(this));
//     }
//
//     // Check single-entry cache
//     if (m_lastModulatedOpacity == opacity && m_lastModulatedGradient)
//     {
//         return m_lastModulatedGradient;
//     }
//
//     // Create new modulated gradient
//     GradDataArray<ColorInt> newColors(m_count);
//     for (size_t i = 0; i < m_count; ++i)
//     {
//         newColors[i] = colorModulateOpacity(m_colors[i], opacity);
//     }
//
//     GradDataArray<float> newStops(m_stops.get(), m_count);
//
//     m_lastModulatedGradient = rcp(new Gradient(m_paintType,
//                                                std::move(newColors),
//                                                std::move(newStops),
//                                                m_count,
//                                                m_coeffs[0],
//                                                m_coeffs[1],
//                                                m_coeffs[2]));
//     m_lastModulatedOpacity = opacity;
//
//     return m_lastModulatedGradient;
// }
//
// } // namespace rive::gpu

#![allow(dead_code)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]

use super::gradient_hpp::{GradDataArray, Gradient};
use crate::mechanical_port::source::include::rive::refcnt_hpp::{rcp, ref_rcp};
use crate::mechanical_port::source::renderer::include::rive::renderer::gpu_hpp as gpu;
use nuxie_render_api::ColorInt;

const GRADIENT_EPSILON: f32 = 1.0 / 4096.0;

fn validate_gradient_stops(stops: &[f32]) -> bool {
    if stops.is_empty() {
        return false;
    }
    if stops.iter().any(|stop| !(0.0 <= *stop && *stop <= 1.0)) {
        return false;
    }
    stops.windows(2).all(|pair| pair[0] <= pair[1])
}

#[inline]
fn precise_mix(a: f32, b: f32, t: f32) -> f32 {
    // Exact scalar operation order of simd::precise_mix: unlike the common
    // `a + (b - a) * t` spelling, this preserves both authored endpoint terms.
    a * (1.0 - t) + b * t
}

#[inline]
fn color_modulate_opacity(value: ColorInt, opacity: f32) -> ColorInt {
    // colorModulateOpacity(value, opacity) =
    // colorWithAlpha(value, opacityToAlpha(colorOpacity(value) * opacity)).
    // Multiply first, then apply the source min/max clamp. Rust f32::min/max
    // match the source std::min/std::max NaN selection here (NaN becomes 1).
    let source_opacity = ((value >> 24) & 0xff) as f32 / 255.0;
    let product = source_opacity * opacity;
    let clamped = product.min(1.0).max(0.0);
    let alpha = (255.0 * clamped).round() as u32;
    (value & 0x00ff_ffff) | (alpha << 24)
}

unsafe fn checked_inputs<'a>(
    colors: *const ColorInt,
    stops: *const f32,
    count: usize,
) -> Option<(&'a [ColorInt], &'a [f32])> {
    if count == 0 || colors.is_null() || stops.is_null() {
        return None;
    }
    // SAFETY: these are the source `[count]` arrays supplied by Factory. The
    // source has the same precondition; no pointer is retained after copying.
    unsafe {
        Some((
            core::slice::from_raw_parts(colors, count),
            core::slice::from_raw_parts(stops, count),
        ))
    }
}

impl Gradient {
    /// Source `Gradient::MakeLinear` with the original pointer/count surface.
    pub unsafe fn MakeLinear(
        sx: f32,
        sy: f32,
        ex: f32,
        ey: f32,
        colors: *const ColorInt,
        stops: *const f32,
        count: usize,
    ) -> rcp<Gradient> {
        let Some((colors, stops)) = (unsafe { checked_inputs(colors, stops, count) }) else {
            return rcp::new();
        };
        if !validate_gradient_stops(stops) {
            return rcp::new();
        }

        let mut start = [sx, sy];
        let mut end = [ex, ey];
        let new_colors = GradDataArray::from_slice(colors);
        let mut new_stops = GradDataArray::from_slice(stops);

        let first_stop = stops[0];
        let last_stop = stops[count - 1];
        if (first_stop != 0.0 || last_stop != 1.0) && last_stop - first_stop > GRADIENT_EPSILON {
            // `simd::precise_mix(start.xyxy, end.xyxy, ...)` is these two
            // endpoint mixes in source declaration order.
            // The SIMD source mixes both endpoints against the original
            // vectors; retain those original values until both are computed.
            let original_start = [sx, sy];
            let original_end = [ex, ey];
            start = [
                precise_mix(original_start[0], original_end[0], first_stop),
                precise_mix(original_start[1], original_end[1], first_stop),
            ];
            end = [
                precise_mix(original_start[0], original_end[0], last_stop),
                precise_mix(original_start[1], original_end[1], last_stop),
            ];
            // SAFETY: validate_gradient_stops rejected empty input and count
            // bounds every transformed read/write below; both arrays contain
            // exactly count initialized source elements.
            unsafe {
                new_stops.write(0, 0.0);
            }
            unsafe {
                new_stops.write(count - 1, 1.0);
            }
            if count > 2 {
                let m = 1.0 / (last_stop - first_stop);
                let a = -first_stop * m;
                for index in 1..count - 1 {
                    unsafe {
                        new_stops.write(index, stops[index] * m + a);
                    }
                }
                for index in 1..count - 1 {
                    let previous = unsafe { new_stops.read(index - 1) };
                    let current = unsafe { new_stops.read(index) };
                    unsafe {
                        new_stops.write(index, previous.max(current));
                    }
                }
                for index in (1..count - 1).rev() {
                    let current = unsafe { new_stops.read(index) };
                    let next = unsafe { new_stops.read(index + 1) };
                    unsafe {
                        new_stops.write(index, current.min(next));
                    }
                }
            }
            debug_assert!(validate_gradient_stops(unsafe {
                new_stops.as_slice(count)
            }));
        }

        let dx = end[0] - start[0];
        let dy = end[1] - start[1];
        let inverse_length_squared = 1.0 / (dx * dx + dy * dy);
        let vx = dx * inverse_length_squared;
        let vy = dy * inverse_length_squared;
        unsafe {
            Gradient::make_rcp(
                gpu::PaintType::linearGradient,
                new_colors,
                new_stops,
                count,
                vx,
                vy,
                -(vx * start[0] + vy * start[1]),
            )
        }
    }

    /// Checked CSS construction. The interpolation mode is fixed before the
    /// fresh intrusive allocation is shared or used as a cache key.
    pub fn make_premultiplied_linear(
        sx: f32, sy: f32, ex: f32, ey: f32,
        colors: &[ColorInt], stops: &[f32],
    ) -> Option<rcp<Gradient>> {
        if colors.len() != stops.len() || colors.len() < 2
            || colors.len() > super::gradient_hpp::CssGradientStopTable::MAX_STOPS
            || ![sx, sy, ex, ey].iter().all(|v| v.is_finite())
            || !stops.iter().all(|v| v.is_finite() && (0.0..=1.0).contains(v))
            || stops.windows(2).any(|v| v[0] > v[1])
        {
            return None;
        }
        // SAFETY: paired live slices have the validated count. MakeLinear
        // returns a fresh, unshared allocation; no cached key can observe this
        // one-time initialization of its interpolation mode.
        let gradient = unsafe {
            Self::MakeLinear(sx, sy, ex, ey, colors.as_ptr(), stops.as_ptr(), colors.len())
        };
        if gradient.get().is_null() { return None; }
        // Preserve the ordinary source calculation when it is usable. Very
        // short or long finite lines can under/overflow its squared f32 length;
        // recover their representable coefficients with f64 intermediates.
        let value = unsafe { &mut *gradient.get() };
        if !value.m_coeffs.iter().all(|v| v.is_finite())
            || (value.m_coeffs[0] == 0.0 && value.m_coeffs[1] == 0.0) {
            let first = stops[0];
            let last = stops[stops.len() - 1];
            let (start, end) = if (first != 0.0 || last != 1.0)
                && last - first > GRADIENT_EPSILON {
                ([precise_mix(sx, ex, first), precise_mix(sy, ey, first)],
                 [precise_mix(sx, ex, last), precise_mix(sy, ey, last)])
            } else { ([sx, sy], [ex, ey]) };
            let dx = end[0] as f64 - start[0] as f64;
            let dy = end[1] as f64 - start[1] as f64;
            let length_squared = dx * dx + dy * dy;
            if !length_squared.is_finite() || length_squared == 0.0 { return None; }
            let vx = dx / length_squared;
            let vy = dy / length_squared;
            let coefficients = [vx as f32, vy as f32,
                (-(vx * start[0] as f64 + vy * start[1] as f64)) as f32];
            if !coefficients.iter().all(|v| v.is_finite()) { return None; }
            value.m_coeffs = coefficients;
        }
        value.premultiplied_interpolation = true;
        Some(gradient)
    }

    /// A CSS image tile repeats in local x/y before projection onto the line.
    pub fn make_tiled_premultiplied_linear(
        sx: f32, sy: f32, ex: f32, ey: f32, tile: [f32; 4],
        colors: &[ColorInt], stops: &[f32],
    ) -> Option<rcp<Gradient>> {
        if !nuxie_render_api::css_gradient_tile_is_representable(tile) {
            return None;
        }
        let gradient = Self::make_premultiplied_linear(sx, sy, ex, ey, colors, stops)?;
        let [cx, cy, cz] = unsafe { (*gradient.get()).m_coeffs };
        // These are the exact projection lanes used by set_css_gradient_tile.
        // Reject unrepresentable paint before it can reach the draw assertion.
        if ![cx * tile[2], cy * tile[3], cx * tile[0] + cy * tile[1] + cz]
            .iter().all(|v| v.is_finite()) { return None; }
        // Fresh owner: set immutable tile metadata before publishing or caching.
        unsafe { (*gradient.get()).css_tile = Some(tile); }
        Some(gradient)
    }

    /// Source `Gradient::MakeRadial` with the original pointer/count surface.
    pub unsafe fn MakeRadial(
        cx: f32,
        cy: f32,
        mut radius: f32,
        colors: *const ColorInt,
        stops: *const f32,
        count: usize,
    ) -> rcp<Gradient> {
        let Some((colors, stops)) = (unsafe { checked_inputs(colors, stops, count) }) else {
            return rcp::new();
        };
        if !validate_gradient_stops(stops) {
            return rcp::new();
        }

        let new_colors = GradDataArray::from_slice(colors);
        let mut new_stops = GradDataArray::from_slice(stops);
        let last_stop = stops[count - 1];
        if last_stop != 1.0 && last_stop > GRADIENT_EPSILON {
            unsafe {
                new_stops.write(count - 1, 1.0);
            }
            radius *= last_stop;
            let inverse_last_stop = 1.0 / last_stop;
            for index in 0..count - 1 {
                unsafe {
                    new_stops.write(index, stops[index] * inverse_last_stop);
                }
            }
            if count > 1 {
                let first = unsafe { new_stops.read(0) };
                unsafe {
                    new_stops.write(0, 0.0f32.max(first));
                }
                for index in 1..count - 1 {
                    let previous = unsafe { new_stops.read(index - 1) };
                    let current = unsafe { new_stops.read(index) };
                    unsafe {
                        new_stops.write(index, previous.max(current));
                    }
                }
                for index in (0..count - 1).rev() {
                    let current = unsafe { new_stops.read(index) };
                    let next = unsafe { new_stops.read(index + 1) };
                    unsafe {
                        new_stops.write(index, current.min(next));
                    }
                }
            }
            debug_assert!(validate_gradient_stops(unsafe {
                new_stops.as_slice(count)
            }));
        }

        unsafe {
            Gradient::make_rcp(
                gpu::PaintType::radialGradient,
                new_colors,
                new_stops,
                count,
                cx,
                cy,
                radius,
            )
        }
    }

    pub fn isOpaque(&self) -> bool {
        let mut cached = unsafe { *self.m_isOpaque.get() };
        if cached == gpu::TriState::unknown {
            let mut all_colors = !0u32;
            for color in unsafe { self.m_colors.as_slice(self.m_count) }.iter() {
                all_colors &= *color;
            }
            let computed = if (all_colors >> 24) == 0xff {
                gpu::TriState::yes
            } else {
                gpu::TriState::no
            };
            unsafe {
                *self.m_isOpaque.get() = computed;
            }
            cached = computed;
        }
        cached == gpu::TriState::yes
    }

    pub fn getModulated(&self, opacity: f32) -> rcp<Gradient> {
        if opacity == 1.0 {
            // SAFETY: source `ref_rcp(const_cast<Gradient*>(this))` retains
            // this exact intrusive owner and returns a new smart pointer.
            return unsafe { ref_rcp(self as *const Self as *mut Self) };
        }

        // The source cache is mutable but deliberately not synchronized; the
        // render owner is frame-thread confined. Keep the mutation narrow and
        // avoid manufacturing Send/Sync on the complete Gradient owner.
        let cached_gradient = unsafe { &mut *self.m_lastModulatedGradient.get() };
        let cached_opacity = unsafe { *self.m_lastModulatedOpacity.get() };
        if cached_opacity == opacity && !cached_gradient.get().is_null() {
            return cached_gradient.clone();
        }

        // Source count-allocation constructor intentionally leaves slots
        // uninitialized until this loop fills every color.
        let mut new_colors = unsafe { GradDataArray::<ColorInt>::uninitialized(self.m_count) };
        // SAFETY: m_count is the validated Gradient array count; every write
        // below is within that allocation and fills each slot exactly once.
        for index in 0..self.m_count {
            let color = unsafe { self.m_colors.read(index) };
            unsafe {
                new_colors.write(index, color_modulate_opacity(color, opacity));
            }
        }
        let new_stops = GradDataArray::from_slice(unsafe { self.m_stops.as_slice(self.m_count) });
        let paint_type = self.m_paintType;
        let count = self.m_count;
        let coeffs = self.m_coeffs;
        let gradient = unsafe {
            Gradient::make_rcp(
                paint_type, new_colors, new_stops, count, coeffs[0], coeffs[1], coeffs[2],
            )
        };
        // The new owner is unshared until it enters the modulation cache.
        unsafe { (*gradient.get()).premultiplied_interpolation = self.premultiplied_interpolation;
            (*gradient.get()).css_tile = self.css_tile; }
        *cached_gradient = gradient;
        unsafe {
            *self.m_lastModulatedOpacity.get() = opacity;
        }
        cached_gradient.clone()
    }
}

#[cfg(test)]
mod css_interpolation_tests {
    use super::*;
    use crate::mechanical_port::source::renderer::include::rive::renderer::render_context_hpp::GradientContentKey;
    use std::collections::HashMap;

    #[test]
    fn checked_css_coefficients_recover_finite_lines_and_reject_unrepresentable_paint() {
        let colors = [0xffff0000, 0xff0000ff];
        let stops = [0., 1.];
        for end in [1e-30, 1e30] {
            let gradient = Gradient::make_tiled_premultiplied_linear(
                0., 0., end, 0., [0., 0., 10., 10.], &colors, &stops).unwrap();
            let coefficients = unsafe { (*gradient.get()).m_coeffs };
            assert!(coefficients.iter().all(|v| v.is_finite()));
            assert!((coefficients[0] as f64 * end as f64 - 1.0).abs() < 1e-6);
        }
        assert!(Gradient::make_tiled_premultiplied_linear(
            0., 0., 0., 0., [0., 0., 10., 10.], &colors, &stops).is_none());
        assert!(Gradient::make_tiled_premultiplied_linear(
            0., 0., 0.001, 0., [0., 0., 3e38, 10.], &colors, &stops).is_none());
    }

    #[test]
    fn tiled_gradient_metadata_survives_shared_ownership_and_modulation() {
        let colors = [0xffff0000, 0xff0000ff];
        let stops = [0., 1.];
        let tiles = [[0., 0., 40., 30.], [-11., 3., 27., 19.]];
        let gradients = tiles.map(|tile| Gradient::make_tiled_premultiplied_linear(
            0., 0., 40., 30., tile, &colors, &stops).unwrap());
        for opacity in [1., 0.5, 0., 0.5] {
            let mut cache = HashMap::new();
            for (original, tile) in gradients.iter().zip(tiles) {
                let shared = original.clone();
                assert_eq!(unsafe { &*shared.get() }.tile(), Some(tile));
                let modulated = unsafe { &*shared.get() }.getModulated(opacity);
                assert_eq!(unsafe { &*modulated.get() }.tile(), Some(tile));
                assert!(unsafe { &*modulated.get() }.interpolates_premultiplied());
                assert_eq!(unsafe { &*original.get() }.tile(), Some(tile));
                assert_eq!(unsafe { &*original.get() }.colors_slice(), colors);
                cache.insert(unsafe { GradientContentKey::new(modulated) }, tile);
            }
            // The atlas stores only stops. Drawing retains each gradient's tile;
            // different tile transforms should not duplicate identical stop rows.
            assert_eq!(cache.len(), 1);
        }
    }

    #[test]
    fn tiled_gradient_constructor_checks_all_tile_coordinates_and_stop_inputs() {
        let colors = [0xffff0000, 0xff0000ff];
        for component in 0..4 {
            for invalid in [f32::NAN, f32::INFINITY, f32::NEG_INFINITY] {
                let mut tile = [0., 0., 40., 30.]; tile[component] = invalid;
                assert!(Gradient::make_tiled_premultiplied_linear(
                    0., 0., 40., 30., tile, &colors, &[0., 1.]).is_none());
            }
        }
        for component in [2, 3] {
            for invalid in [0., -1., 1e-40] {
                let mut tile = [0., 0., 40., 30.]; tile[component] = invalid;
                assert!(Gradient::make_tiled_premultiplied_linear(
                    0., 0., 40., 30., tile, &colors, &[0., 1.]).is_none());
            }
        }
        assert!(Gradient::make_tiled_premultiplied_linear(
            0., 0., 40., 30., [f32::MAX, 0., 0.5, 30.], &colors, &[0., 1.]).is_none());
        // Small values are supported when their inverse remains representable.
        assert!(Gradient::make_tiled_premultiplied_linear(
            0., 0., 40., 30., [0., 0., 1e-30, 30.], &colors, &[0., 1.]).is_some());
        assert!(Gradient::make_tiled_premultiplied_linear(
            0., 0., 40., 30., [-10., -20., 40., 30.], &colors, &[0., 1.]).is_some());
        assert!(Gradient::make_tiled_premultiplied_linear(
            0., 0., 40., 30., [0., 0., 40., 30.], &colors, &[1., 0.]).is_none());
    }

    #[test]
    fn interpolation_mode_survives_modulation_and_separates_cache_entries() {
        let colors = [0xffff0000, 0x000000ff];
        let stops = [0.0, 1.0];
        let straight = unsafe {
            Gradient::MakeLinear(0.0, 0.0, 100.0, 0.0, colors.as_ptr(), stops.as_ptr(), 2)
        };
        let css = Gradient::make_premultiplied_linear(0.0, 0.0, 100.0, 0.0, &colors, &stops).unwrap();
        assert!(!unsafe { &*straight.get() }.interpolates_premultiplied());
        assert!(unsafe { &*css.get() }.interpolates_premultiplied());
        for opacity in [1.0, 0.5, 0.0, 0.5] {
            let a = unsafe { &*straight.get() }.getModulated(opacity);
            let b = unsafe { &*css.get() }.getModulated(opacity);
            assert!(!unsafe { &*a.get() }.interpolates_premultiplied());
            assert!(unsafe { &*b.get() }.interpolates_premultiplied());
            assert_eq!(unsafe { &*a.get() }.colors_slice(), unsafe { &*b.get() }.colors_slice());
            let mut cache = HashMap::new();
            cache.insert(unsafe { GradientContentKey::new(a) }, "straight");
            cache.insert(unsafe { GradientContentKey::new(b.clone()) }, "css");
            assert_eq!(cache.len(), 2);
            assert_eq!(cache.get(&unsafe { GradientContentKey::new(b) }), Some(&"css"));
        }
    }

    #[test]
    fn checked_css_gradient_rejects_invalid_inputs() {
        let colors = [0xffff0000, 0x000000ff];
        for stops in [vec![], vec![0.0], vec![0.5, 0.2], vec![-0.1, 1.0], vec![0.0, f32::NAN]] {
            assert!(Gradient::make_premultiplied_linear(0.0, 0.0, 100.0, 0.0, &colors, &stops).is_none());
        }
        assert!(Gradient::make_premultiplied_linear(f32::INFINITY, 0.0, 100.0, 0.0, &colors, &[0.0, 1.0]).is_none());
    }
}
