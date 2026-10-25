//! Four-wide CPU math from bezier_utils.cpp and draw.cpp.
use super::*;

#[derive(Default)]
pub(super) struct Convex180Chops4 {
    pub num_chops: [u8; 4],
    pub t0: [f32; 4],
    pub t1: [f32; 4],
    pub possible_cusps: [bool; 4],
}
fn min_fast(a: f32, b: f32) -> f32 {
    if b < a { b } else { a }
}
fn max_fast(a: f32, b: f32) -> f32 {
    if a < b { b } else { a }
}
fn cross_unfused(a: Vec2D, b: Vec2D) -> f32 {
    a.x * b.y - a.y * b.x
}
pub(super) fn cubics_need_no_chops(cubics: [[Vec2D; 4]; 4]) -> [bool; 4] {
    cubics.map(|p| {
        let a = subtract(p[1], p[0]);
        let b = subtract(p[2], p[1]);
        let c = subtract(p[3], p[2]);
        let mut ab = cross_unfused(a, b);
        let mut bc = cross_unfused(b, c);
        let mut ac = cross_unfused(a, c);
        let az = a.x == 0.0 && a.y == 0.0;
        let cz = c.x == 0.0 && c.y == 0.0;
        if az {
            ab = bc;
            ac = bc;
        }
        if cz {
            bc = ab;
            ac = ab;
        }
        let lo = min_fast(min_fast(ab, bc), ac);
        let hi = max_fast(max_fast(ab, bc), ac);
        let length = |p: Vec2D| p.x * p.x + p.y * p.y;
        let threshold = (1.0 / 4096.0) * max_fast(max_fast(length(a), length(b)), length(c));
        lo > threshold || hi < -threshold || (az && cz && (b.x != 0.0 || b.y != 0.0))
    })
}
pub(super) fn find_cubic_convex_180_chops4(cubics: [[Vec2D; 4]; 4]) -> Convex180Chops4 {
    const E: f32 = 1.0 / 1024.0;
    let coefficients = cubics.map(|p| {
        let c = subtract(p[1], p[0]);
        let d = subtract(p[2], p[1]);
        let e = subtract(p[3], p[0]);
        let b = subtract(d, c);
        let a = Vec2D::new(-3.0 * d.x + e.x, -3.0 * d.y + e.y);
        let aa = cross_unfused(a, b);
        let bb = -0.5 * cross_unfused(a, c);
        let cc = cross_unfused(b, c);
        let discr = bb * bb - aa * cc;
        let threshold = aa * (E / 2.0);
        (aa, bb, cc, discr, threshold * threshold)
    });
    let all_no_inflections = coefficients.iter().all(|&(_, _, _, d, t)| d < -t);
    let mut out = Convex180Chops4::default();
    for (i, (a, b, c, discr, threshold)) in coefficients.into_iter().enumerate() {
        let no_inflections = discr < -threshold;
        let root = c / b;
        let chop = (root - E).to_bits() < (1.0 - 2.0 * E).to_bits();
        if all_no_inflections {
            out.num_chops[i] = u8::from(chop);
            out.t0[i] = root;
            continue;
        }
        out.possible_cusps[i] = !no_inflections && discr <= threshold;
        let q = discr.sqrt().copysign(b) + b;
        let r0 = q / a;
        let r1 = c / q;
        let in0 = r0 > E && r0 < 1.0 - E;
        let in1 = r1 > E && r1 < 1.0 - E;
        let two = in0 && in1 && r0 != r1;
        let swap = r0 > r1;
        let t0 = if two {
            if swap { r1 } else { r0 }
        } else if in0 {
            r0
        } else {
            r1
        };
        out.t1[i] = if swap { r0 } else { r1 };
        out.num_chops[i] = if no_inflections {
            u8::from(chop)
        } else if two {
            2
        } else {
            u8::from(in0 || in1)
        };
        out.t0[i] = if no_inflections { root } else { t0 };
    }
    out
}
pub(super) fn find_cubic_tangents4(cubics: [[Vec2D; 4]; 4]) -> [[Vec2D; 2]; 4] {
    std::array::from_fn(|lane| {
        let p = cubics[lane];
        let differ01 = p[0].x != p[1].x || p[0].y != p[1].y;
        let differ12 = p[1].x != p[2].x || p[1].y != p[2].y;
        let differ23 = p[2].x != p[3].x || p[2].y != p[3].y;
        let a = if differ01 {
            p[1]
        } else if differ12 {
            p[2]
        } else {
            p[3]
        };
        let b = if differ23 {
            p[2]
        } else if differ12 {
            p[1]
        } else {
            p[0]
        };
        [
            Vec2D::new(a.x - p[0].x, a.y - p[0].y),
            Vec2D::new(p[3].x - b.x, p[3].y - b.y),
        ]
    })
}

fn clamp_source(x: f32, lo: f32, hi: f32) -> f32 {
    x.max(lo).min(hi)
}
pub(super) fn cubic_segment_counts4(
    cubics: [[Vec2D; 4]; 4],
    precision: f32,
    transform: Mat2D,
) -> [u32; 4] {
    let [xx, yx, xy, yy, _, _] = transform.0;
    let pow4 = cubics.map(|p| {
        let second = |a: Vec2D, b: Vec2D, c: Vec2D| {
            let x = -2.0 * b.x + a.x + c.x;
            let y = -2.0 * b.y + a.y + c.y;
            let tx = xx * x + xy * y;
            let ty = yy * y + yx * x;
            tx * tx + ty * ty
        };
        max_fast(second(p[0], p[1], p[2]), second(p[1], p[2], p[3]))
            * ((9.0 / 16.0) * (precision * precision))
    });
    #[cfg(any(
        all(
            any(target_arch = "x86", target_arch = "x86_64"),
            target_feature = "sse"
        ),
        all(
            any(target_arch = "arm", target_arch = "aarch64"),
            target_feature = "neon"
        )
    ))]
    let n = inverse_sqrt_fast(inverse_sqrt_fast(pow4));
    #[cfg(not(any(
        all(
            any(target_arch = "x86", target_arch = "x86_64"),
            target_feature = "sse"
        ),
        all(
            any(target_arch = "arm", target_arch = "aarch64"),
            target_feature = "neon"
        )
    )))]
    let n = pow4.map(|v| v.sqrt().sqrt());
    n.map(|v| clamp_source(v.ceil(), 1.0, MAX_PARAMETRIC_SEGMENTS as f32) as u32)
}
pub(crate) fn process_pieces(
    batch: &cubic_batch::CubicBatch,
    piece_count: usize,
    precision: f32,
    matrix: Mat2D,
    counts: &mut [u32],
) -> u32 {
    let mut total = [0u32; 4];
    for first in (0..piece_count).step_by(4) {
        let cubics = std::array::from_fn(|lane| batch.cubic(first + lane));
        let n = cubic_segment_counts4(cubics, precision, matrix);
        counts[first..first + 4].copy_from_slice(&n);
        for lane in 0..4 {
            if first + lane < piece_count {
                total[lane] += n[lane];
            }
        }
    }
    total.into_iter().sum()
}
pub(super) fn count_polar_segments(pairs: &[[Vec2D; 2]], per_radian: f32, out: &mut [u32]) {
    for first in (0..pairs.len()).step_by(4) {
        let p: [[Vec2D; 2]; 4] = std::array::from_fn(|lane| {
            pairs
                .get(first + lane)
                .copied()
                .unwrap_or([Vec2D::new(0.0, 1.0); 2])
        });
        let numer = p.map(|[a, b]| a.x * b.x + a.y * b.y);
        let denom = p.map(|[a, b]| (a.x * a.x + a.y * a.y) * (b.x * b.x + b.y * b.y));
        #[cfg(any(
            all(
                any(target_arch = "x86", target_arch = "x86_64"),
                target_feature = "sse"
            ),
            all(
                any(target_arch = "arm", target_arch = "aarch64"),
                target_feature = "neon"
            )
        ))]
        let cosine = {
            let inv = inverse_sqrt_fast(denom);
            std::array::from_fn::<_, 4, _>(|i| numer[i] * inv[i])
        };
        #[cfg(not(any(
            all(
                any(target_arch = "x86", target_arch = "x86_64"),
                target_feature = "sse"
            ),
            all(
                any(target_arch = "arm", target_arch = "aarch64"),
                target_feature = "neon"
            )
        )))]
        let cosine = std::array::from_fn::<_, 4, _>(|i| numer[i] / denom[i].sqrt());
        for lane in 0..4 {
            if first + lane < pairs.len() {
                out[first + lane] = clamp_source(
                    (fast_acos(clamp_source(cosine[lane], -1.0, 1.0)) * per_radian).ceil(),
                    1.0,
                    crate::gpu::MAX_POLAR_SEGMENTS as f32,
                ) as u32;
            }
        }
    }
}

// Deliberately unavailable without the hardware estimate, as in upstream.
#[cfg(all(
    any(target_arch = "x86", target_arch = "x86_64"),
    target_feature = "sse"
))]
fn inverse_sqrt_fast(x: [f32; 4]) -> [f32; 4] {
    #[cfg(target_arch = "x86")]
    use core::arch::x86::{_mm_loadu_ps, _mm_rsqrt_ps, _mm_storeu_ps};
    #[cfg(target_arch = "x86_64")]
    use core::arch::x86_64::{_mm_loadu_ps, _mm_rsqrt_ps, _mm_storeu_ps};
    let mut result = [0.0; 4];
    // SAFETY: SSE is enabled and both arrays contain four accessible lanes.
    unsafe {
        _mm_storeu_ps(result.as_mut_ptr(), _mm_rsqrt_ps(_mm_loadu_ps(x.as_ptr())));
    }
    result
}
#[cfg(all(target_arch = "aarch64", target_feature = "neon"))]
fn inverse_sqrt_fast(x: [f32; 4]) -> [f32; 4] {
    use core::arch::aarch64::{vld1q_f32, vmulq_f32, vrsqrteq_f32, vrsqrtsq_f32, vst1q_f32};
    let mut result = [0.0; 4];
    // SAFETY: NEON is enabled; load/store address complete four-lane arrays.
    unsafe {
        let xv = vld1q_f32(x.as_ptr());
        let y = vrsqrteq_f32(xv);
        let y = vmulq_f32(y, vrsqrtsq_f32(xv, vmulq_f32(y, y)));
        vst1q_f32(result.as_mut_ptr(), y);
    }
    result
}
#[cfg(all(target_arch = "arm", target_feature = "neon"))]
fn inverse_sqrt_fast(x: [f32; 4]) -> [f32; 4] {
    let mut result = [0.0; 4];
    // ARM32 stdarch NEON intrinsics are not stable. These are exactly the
    // upstream estimate and one Newton step, not a scalar 1/sqrt substitute.
    // SAFETY: NEON is enabled; pointers refer to complete four-lane arrays;
    // all vector registers modified by this sequence are declared clobbered.
    unsafe {
        core::arch::asm!(
            "vld1.32 {{q0}}, [{input}]",
            "vrsqrte.f32 q1, q0",
            "vmul.f32 q2, q1, q1",
            "vrsqrts.f32 q0, q0, q2",
            "vmul.f32 q1, q1, q0",
            "vst1.32 {{q1}}, [{output}]",
            input = in(reg) x.as_ptr(),
            output = in(reg) result.as_mut_ptr(),
            out("q0") _, out("q1") _, out("q2") _,
            options(nostack),
        );
    }
    result
}
