//! Shared logical tessellation relocation used by both renderer roots.

use crate::gpu;

pub(crate) fn relocate_tessellation_logically(
    spans: &mut Vec<gpu::TessVertexSpan>,
    base_instance: &mut u32,
    contours: &mut [gpu::ContourData],
    next_base_instance: u32,
    segment_span: u32,
) {
    let mut span_scratch = Vec::new();
    relocate_tessellation_logically_with_scratch(
        spans,
        base_instance,
        contours,
        next_base_instance,
        segment_span,
        &mut span_scratch,
    );
}

pub(crate) fn relocate_tessellation_logically_with_scratch(
    spans: &mut Vec<gpu::TessVertexSpan>,
    base_instance: &mut u32,
    contours: &mut [gpu::ContourData],
    next_base_instance: u32,
    segment_span: u32,
    span_scratch: &mut Vec<gpu::TessVertexSpan>,
) {
    #[derive(PartialEq, Eq)]
    struct SourceSpanKey {
        points: [[u32; 2]; 4],
        join_tangent: [u32; 2],
        logical_x0: i64,
        logical_x1: i64,
        reflection_location: Option<u32>,
        segment_counts: u32,
        contour_id_with_flags: u32,
    }

    let texture_width = gpu::TESS_TEXTURE_WIDTH as u32;
    let old_base = base_instance
        .checked_mul(segment_span)
        .expect("tessellation source location overflow");
    let new_base = next_base_instance
        .checked_mul(segment_span)
        .expect("tessellation destination location overflow");
    let relocation = new_base
        .checked_sub(old_base)
        .expect("compact tessellation layout must move forward");
    let mut source = std::mem::take(spans);
    span_scratch.clear();
    span_scratch.reserve(source.len());
    let mut previous_key = None;
    for mut span in source.drain(..) {
        let (source_x0, source_x1) = span.x_range();
        debug_assert!(span.y >= 0.0 && span.y.fract() == 0.0);
        let reverse_only = span.reflection_y.is_nan() && source_x1 < source_x0;
        let (source_logical_local_x0, source_logical_local_x1) = if reverse_only {
            (source_x1, source_x0)
        } else {
            debug_assert!(source_x1 >= source_x0);
            (source_x0, source_x1)
        };
        let vertex_count = u32::try_from(source_logical_local_x1 - source_logical_local_x0)
            .expect("tessellation span width must be non-negative");
        let vertex_count_i32 =
            i32::try_from(vertex_count).expect("tessellation span width fits i32");
        let source_logical_x0 = (span.y as i64)
            .checked_mul(i64::from(texture_width))
            .and_then(|row| row.checked_add(i64::from(source_logical_local_x0)))
            .expect("tessellation source span location overflow");
        let source_logical_x1 = source_logical_x0
            .checked_add(i64::from(vertex_count))
            .expect("tessellation source span end overflow");
        let source_reflection_location = span.reflection_y.is_finite().then(|| {
            debug_assert!(span.reflection_y >= 0.0 && span.reflection_y.fract() == 0.0);
            let source_reflection_x0 = span.reflection_x0_x1 as i16 as i32;
            (span.reflection_y as u32)
                .wrapping_mul(texture_width)
                .wrapping_add_signed(source_reflection_x0)
        });
        let key = SourceSpanKey {
            points: span.points.map(|point| point.map(f32::to_bits)),
            join_tangent: span.join_tangent.map(f32::to_bits),
            logical_x0: source_logical_x0,
            logical_x1: source_logical_x1,
            reflection_location: source_reflection_location,
            segment_counts: span.segment_counts,
            contour_id_with_flags: span.contour_id_with_flags,
        };
        if previous_key.as_ref() == Some(&key) {
            continue;
        }
        previous_key = Some(key);

        let logical_x0 = u32::try_from(source_logical_x0)
            .expect("tessellation span start must be non-negative")
            .checked_add(relocation)
            .expect("tessellation span relocation overflow");
        let mut y = logical_x0 / texture_width;
        let mut x0 =
            i32::try_from(logical_x0 % texture_width).expect("tessellation span x must fit i32");
        let mut x1 = x0
            .checked_add(vertex_count_i32)
            .expect("tessellation span end overflow");

        if let Some(source_reflection_location) = source_reflection_location {
            let source_reflection_x0 = span.reflection_x0_x1 as i16 as i32;
            let source_reflection_x1 = (span.reflection_x0_x1 >> 16) as i16 as i32;
            debug_assert!(source_reflection_x0 >= source_reflection_x1);
            debug_assert_eq!(
                source_reflection_x0 - source_reflection_x1,
                vertex_count_i32
            );
            let reflection_location = source_reflection_location.wrapping_add(relocation);
            let reflection_last = reflection_location.wrapping_sub(1);
            let mut reflection_y = reflection_last / texture_width;
            let mut reflection_x0 = i32::try_from(reflection_last % texture_width + 1)
                .expect("tessellation reflection x must fit i32");
            let mut reflection_x1 = reflection_x0 - vertex_count_i32;
            loop {
                span.y = y as f32;
                span.set_ranges(x0, x1, reflection_x0, reflection_x1, reflection_y as f32);
                span_scratch.push(span);
                if x1 <= gpu::TESS_TEXTURE_WIDTH && reflection_x1 >= 0 {
                    break;
                }
                y += 1;
                x0 -= gpu::TESS_TEXTURE_WIDTH;
                x1 -= gpu::TESS_TEXTURE_WIDTH;
                reflection_y = reflection_y.wrapping_sub(1);
                reflection_x0 += gpu::TESS_TEXTURE_WIDTH;
                reflection_x1 += gpu::TESS_TEXTURE_WIDTH;
            }
        } else if reverse_only {
            // Reverse-only spans are authored right-to-left. Reconstruct them
            // from the relocated high endpoint, retaining the source row-wrap
            // behavior of push_reverse_tessellation_spans.
            let reverse_location = logical_x0
                .checked_add(u32::try_from(vertex_count_i32).expect("reverse span width"))
                .expect("reverse tessellation span end overflow");
            let reverse_last = reverse_location
                .checked_sub(1)
                .expect("reverse tessellation span underflow");
            let mut reverse_y = reverse_last / texture_width;
            let mut reverse_x0 = i32::try_from(reverse_last % texture_width + 1)
                .expect("reverse tessellation span x must fit i32");
            let mut reverse_x1 = reverse_x0 - vertex_count_i32;
            loop {
                span.y = reverse_y as f32;
                span.set_ranges(reverse_x0, reverse_x1, -1, -1, f32::NAN);
                span_scratch.push(span);
                if reverse_x1 >= 0 {
                    break;
                }
                reverse_y = reverse_y
                    .checked_sub(1)
                    .expect("reverse tessellation row underflow");
                reverse_x0 = reverse_x0
                    .checked_add(texture_width as i32)
                    .expect("reverse tessellation x overflow");
                reverse_x1 = reverse_x1
                    .checked_add(texture_width as i32)
                    .expect("reverse tessellation x overflow");
            }
        } else {
            loop {
                span.y = y as f32;
                span.set_ranges(x0, x1, -1, -1, f32::NAN);
                span_scratch.push(span);
                if x1 <= gpu::TESS_TEXTURE_WIDTH {
                    break;
                }
                y += 1;
                x0 -= gpu::TESS_TEXTURE_WIDTH;
                x1 -= gpu::TESS_TEXTURE_WIDTH;
            }
        }
    }
    *base_instance = next_base_instance;
    for contour in contours {
        contour.vertex_index0 = contour
            .vertex_index0
            .checked_add(relocation)
            .expect("MSAA midpoint contour relocation overflow");
    }
    std::mem::swap(spans, span_scratch);
    *span_scratch = source;
}
