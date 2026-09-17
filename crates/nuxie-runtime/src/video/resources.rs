//! Deterministic admission under a host-measured decoder budget. Availability
//! is supplied by the host; renderer selection never implies decode support.
#[derive(Clone, Copy, Debug, Default)]
pub struct DecoderBudget {
    /// Total admitted players across all implementation classes.
    pub max_players: usize,
    /// Platform-selected decoders can use hardware or software internally.
    pub managed_players: usize,
    pub managed_pixels_per_second: u64,
    pub hardware_players: usize,
    pub software_pixels_per_second: u64,
}
#[derive(Clone, Copy, Debug)]
pub struct DecoderRequest {
    pub id: u64,
    pub priority: u32,
    pub visible: bool,
    pub hardware_supported: bool,
    pub managed_supported: bool,
    pub software_supported: bool,
    /// Width * height * frame rate, rounded up by the host.
    pub pixels_per_second: u64,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Allocation {
    Hardware,
    Software,
    Poster,
    PlatformManaged,
}
/// Higher priorities win; stable occurrence IDs break ties. Invisible videos
/// consume no slots. Managed and software decoding require positive bounded
/// pixel rates; every admitted implementation also consumes the total limit.
pub fn allocate(requests: &[DecoderRequest], mut budget: DecoderBudget) -> Vec<(u64, Allocation)> {
    let mut sorted: Vec<_> = requests.iter().collect();
    sorted.sort_by_key(|r| (std::cmp::Reverse(r.priority), r.id));
    sorted
        .into_iter()
        .map(|r| {
            let allocation = if !r.visible || budget.max_players == 0 {
                Allocation::Poster
            } else if r.hardware_supported && budget.hardware_players > 0 {
                budget.hardware_players -= 1;
                Allocation::Hardware
            } else if r.managed_supported
                && budget.managed_players > 0
                && r.pixels_per_second > 0
                && r.pixels_per_second <= budget.managed_pixels_per_second
            {
                budget.managed_players -= 1;
                budget.managed_pixels_per_second -= r.pixels_per_second;
                Allocation::PlatformManaged
            } else if r.software_supported
                && r.pixels_per_second > 0
                && r.pixels_per_second <= budget.software_pixels_per_second
            {
                budget.software_pixels_per_second -= r.pixels_per_second;
                Allocation::Software
            } else {
                Allocation::Poster
            };
            if allocation != Allocation::Poster {
                budget.max_players -= 1;
            }
            (r.id, allocation)
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    fn managed(id: u64, priority: u32, pixels: u64) -> DecoderRequest {
        DecoderRequest {
            id,
            priority,
            visible: true,
            hardware_supported: false,
            software_supported: false,
            managed_supported: true,
            pixels_per_second: pixels,
        }
    }
    #[test]
    fn managed_admission_bounds_both_slots_and_work_without_claiming_hardware() {
        let requests = [managed(1, 0, 100), managed(2, 10, 100), managed(3, 5, 100)];
        for (slots, pixels, expected) in [
            (
                2,
                100,
                vec![
                    (2, Allocation::PlatformManaged),
                    (3, Allocation::Poster),
                    (1, Allocation::Poster),
                ],
            ),
            (
                1,
                300,
                vec![
                    (2, Allocation::PlatformManaged),
                    (3, Allocation::Poster),
                    (1, Allocation::Poster),
                ],
            ),
            (
                2,
                200,
                vec![
                    (2, Allocation::PlatformManaged),
                    (3, Allocation::PlatformManaged),
                    (1, Allocation::Poster),
                ],
            ),
        ] {
            assert_eq!(
                allocate(
                    &requests,
                    DecoderBudget {
                        max_players: 3,
                        managed_players: slots,
                        managed_pixels_per_second: pixels,
                        ..Default::default()
                    }
                ),
                expected
            );
        }
        assert_eq!(
            allocate(
                &[managed(1, 0, 0)],
                DecoderBudget {
                    max_players: 1,
                    managed_players: 1,
                    managed_pixels_per_second: u64::MAX,
                    ..Default::default()
                }
            ),
            vec![(1, Allocation::Poster)]
        );
    }
    #[test]
    fn total_limit_and_software_fallback_are_shared_across_classes() {
        let mut first = managed(1, 10, 100);
        first.hardware_supported = true;
        let mut second = managed(2, 5, 100);
        second.software_supported = true;
        let mut third = managed(3, 0, 100);
        third.software_supported = true;
        let budget = DecoderBudget {
            max_players: 2,
            hardware_players: 1,
            software_pixels_per_second: 300,
            managed_players: 0,
            managed_pixels_per_second: 300,
        };
        assert_eq!(
            allocate(&[first, second, third], budget),
            vec![
                (1, Allocation::Hardware),
                (2, Allocation::Software),
                (3, Allocation::Poster)
            ]
        );
        assert!(
            allocate(&[first, second, third], DecoderBudget::default())
                .iter()
                .all(|(_, mode)| *mode == Allocation::Poster)
        );
    }
}
