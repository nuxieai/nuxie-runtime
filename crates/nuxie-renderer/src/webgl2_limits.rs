const RGBA_BYTES_PER_PIXEL: usize = 4;

pub(crate) const MAX_WEBGL2_FRAME_INTERMEDIATE_RGBA_BYTES: usize = 64 * 1024 * 1024;

#[derive(Default)]
pub(crate) struct WebGl2FrameIntermediateBudget {
    retained_bytes: usize,
}

impl WebGl2FrameIntermediateBudget {
    pub(crate) fn try_reserve_rgba_images(
        &mut self,
        width: usize,
        height: usize,
        image_count: usize,
    ) -> Option<usize> {
        let reserved_bytes = width
            .checked_mul(height)?
            .checked_mul(RGBA_BYTES_PER_PIXEL)?
            .checked_mul(image_count)?;
        let retained_bytes = self.retained_bytes.checked_add(reserved_bytes)?;
        if retained_bytes > MAX_WEBGL2_FRAME_INTERMEDIATE_RGBA_BYTES {
            return None;
        }
        self.retained_bytes = retained_bytes;
        Some(reserved_bytes)
    }

    pub(crate) fn release(&mut self, reserved_bytes: usize) {
        debug_assert!(reserved_bytes <= self.retained_bytes);
        self.retained_bytes = self.retained_bytes.saturating_sub(reserved_bytes);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reserves_the_exact_frame_intermediate_boundary_atomically() {
        let mut budget = WebGl2FrameIntermediateBudget::default();

        let reserved = budget
            .try_reserve_rgba_images(4_096, 2_048, 2)
            .expect("two 32 MiB images fit the 64 MiB budget exactly");

        assert_eq!(reserved, MAX_WEBGL2_FRAME_INTERMEDIATE_RGBA_BYTES);
        assert!(budget.try_reserve_rgba_images(1, 1, 1).is_none());
        assert_eq!(
            budget.retained_bytes,
            MAX_WEBGL2_FRAME_INTERMEDIATE_RGBA_BYTES
        );

        budget.release(reserved);
        assert_eq!(budget.retained_bytes, 0);
    }

    #[test]
    fn rejects_an_over_budget_pair_before_the_caller_allocates_either_image() {
        let mut budget = WebGl2FrameIntermediateBudget::default();
        let mut allocation_count = 0;

        let reservation = budget.try_reserve_rgba_images(4_097, 2_048, 2);
        if reservation.is_some() {
            allocation_count += 1;
        }

        assert!(reservation.is_none());
        assert_eq!(allocation_count, 0);
        assert_eq!(budget.retained_bytes, 0);
    }

    #[test]
    fn rejects_dimension_arithmetic_overflow_without_changing_the_budget() {
        let mut budget = WebGl2FrameIntermediateBudget::default();

        assert!(budget
            .try_reserve_rgba_images(usize::MAX, usize::MAX, 2)
            .is_none());
        assert_eq!(budget.retained_bytes, 0);
    }
}
