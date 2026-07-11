//! Rectangle packing with the same skyline heuristic as Rive's C++ renderer.
//!
//! Ported from:
//! - `renderer/src/sk_rectanizer_skyline.cpp`
//! - `renderer/include/rive/renderer/sk_rectanizer_skyline.hpp`
//!
//! Those files identify their upstream Skia import as
//! `b4171f5ba83048039097bbc664eaa076190f6239`.

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct SkylineSegment {
    x: i32,
    y: i32,
    width: i32,
}

/// Packs rectangles by tracking the occupied skyline of an atlas.
///
/// This is a direct port of `rive::RectanizerSkyline`. In particular, it
/// chooses the lowest valid placement, breaking ties in favor of the narrowest
/// starting skyline segment.
pub(crate) struct Skyline {
    width: i32,
    height: i32,
    skyline: Vec<SkylineSegment>,
    area_so_far: i32,
}

/// The packed origins and occupied extent of an atlas layout.
#[derive(Debug)]
pub(crate) struct AtlasLayout {
    origins: Vec<[u32; 2]>,
    extent: [u32; 2],
}

impl AtlasLayout {
    pub(crate) fn origins(&self) -> &[[u32; 2]] {
        &self.origins
    }

    pub(crate) fn extent(&self) -> [u32; 2] {
        self.extent
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum AtlasPackError {
    InvalidDimensions,
    Full,
}

impl AtlasPackError {
    pub(crate) fn message(self) -> &'static str {
        match self {
            Self::InvalidDimensions => "atlas dimensions exceed skyline coordinate range",
            Self::Full => "atlas regions exceed the device texture limit",
        }
    }
}

/// Packs atlas regions within the device's maximum 2D texture dimension.
pub(crate) fn pack_atlas_regions(
    width: u32,
    max_dimension: u32,
    regions: &[(u32, u32)],
) -> Result<AtlasLayout, AtlasPackError> {
    if width == 0 || max_dimension == 0 || width > max_dimension {
        return Err(AtlasPackError::InvalidDimensions);
    }
    let width = i32::try_from(width).map_err(|_| AtlasPackError::InvalidDimensions)?;
    let max_dimension =
        i32::try_from(max_dimension).map_err(|_| AtlasPackError::InvalidDimensions)?;

    let mut skyline = Skyline::new(width, max_dimension);
    let mut origins = Vec::with_capacity(regions.len());
    let mut extent = [1, 1];
    for &(region_width, region_height) in regions {
        let region_width_i32 =
            i32::try_from(region_width).map_err(|_| AtlasPackError::InvalidDimensions)?;
        let region_height_i32 =
            i32::try_from(region_height).map_err(|_| AtlasPackError::InvalidDimensions)?;
        let mut x = 0;
        let mut y = 0;
        if !skyline.add_rect(region_width_i32, region_height_i32, &mut x, &mut y) {
            return Err(AtlasPackError::Full);
        }
        let x = u32::try_from(x).map_err(|_| AtlasPackError::InvalidDimensions)?;
        let y = u32::try_from(y).map_err(|_| AtlasPackError::InvalidDimensions)?;
        let right = x
            .checked_add(region_width)
            .ok_or(AtlasPackError::InvalidDimensions)?;
        let bottom = y
            .checked_add(region_height)
            .ok_or(AtlasPackError::InvalidDimensions)?;
        if right > max_dimension as u32 || bottom > max_dimension as u32 {
            return Err(AtlasPackError::Full);
        }
        extent[0] = extent[0].max(right);
        extent[1] = extent[1].max(bottom);
        origins.push([x, y]);
    }

    Ok(AtlasLayout { origins, extent })
}

impl Skyline {
    pub(crate) fn new(width: i32, height: i32) -> Self {
        let mut skyline = Self {
            width,
            height,
            skyline: Vec::new(),
            area_so_far: 0,
        };
        skyline.reset();
        skyline
    }

    #[allow(dead_code)] // Retained for parity with RectanizerSkyline's public API.
    pub(crate) fn width(&self) -> i32 {
        self.width
    }

    #[allow(dead_code)] // Retained for parity with RectanizerSkyline's public API.
    pub(crate) fn height(&self) -> i32 {
        self.height
    }

    pub(crate) fn reset(&mut self) {
        self.area_so_far = 0;
        self.skyline.clear();
        self.skyline.push(SkylineSegment {
            x: 0,
            y: 0,
            width: self.width,
        });
    }

    /// Adds a rectangle and writes its origin on success.
    ///
    /// As in the C++ implementation, requests larger than the atlas leave
    /// `x` and `y` untouched. Requests that fit the atlas but cannot fit its
    /// remaining skyline set both to zero on failure.
    pub(crate) fn add_rect(&mut self, width: i32, height: i32, x: &mut i32, y: &mut i32) -> bool {
        if width < 0 || height < 0 || width > self.width || height > self.height {
            return false;
        }

        let mut best_width = self.width.wrapping_add(1);
        let mut best_x = 0;
        let mut best_y = self.height.wrapping_add(1);
        let mut best_index = None;
        for skyline_index in 0..self.skyline.len() {
            if let Some(candidate_y) = self.rectangle_fits(skyline_index, width, height) {
                if candidate_y < best_y
                    || (candidate_y == best_y && self.skyline[skyline_index].width < best_width)
                {
                    best_index = Some(skyline_index);
                    best_width = self.skyline[skyline_index].width;
                    best_x = self.skyline[skyline_index].x;
                    best_y = candidate_y;
                }
            }
        }

        if let Some(best_index) = best_index {
            self.add_skyline_level(best_index, best_x, best_y, width, height);
            *x = best_x;
            *y = best_y;
            self.area_so_far = self.area_so_far.wrapping_add(width.wrapping_mul(height));
            true
        } else {
            *x = 0;
            *y = 0;
            false
        }
    }

    #[allow(dead_code)] // Used by future atlas callers; matches the C++ header contract.
    pub(crate) fn add_padded_rect(
        &mut self,
        width: i32,
        height: i32,
        padding: i16,
        x: &mut i32,
        y: &mut i32,
    ) -> bool {
        let padding = i32::from(padding);
        if self.add_rect(
            width.wrapping_add(padding.wrapping_mul(2)),
            height.wrapping_add(padding.wrapping_mul(2)),
            x,
            y,
        ) {
            *x = x.wrapping_add(padding);
            *y = y.wrapping_add(padding);
            true
        } else {
            false
        }
    }

    #[allow(dead_code)] // Retained for parity with RectanizerSkyline's public API.
    pub(crate) fn is_empty(&self) -> bool {
        self.area_so_far == 0
    }

    #[allow(dead_code)] // Retained for parity with RectanizerSkyline's public API.
    pub(crate) fn percent_full(&self) -> f32 {
        self.area_so_far as f32 / (self.width as f32 * self.height as f32)
    }

    fn rectangle_fits(&self, skyline_index: usize, width: i32, height: i32) -> Option<i32> {
        let x = self.skyline[skyline_index].x;
        if x.wrapping_add(width) > self.width {
            return None;
        }

        let mut width_left = width;
        let mut index = skyline_index;
        let mut y = self.skyline[skyline_index].y;
        while width_left > 0 {
            y = y.max(self.skyline[index].y);
            if y.wrapping_add(height) > self.height {
                return None;
            }
            width_left = width_left.wrapping_sub(self.skyline[index].width);
            index += 1;
            debug_assert!(index < self.skyline.len() || width_left <= 0);
        }

        Some(y)
    }

    fn add_skyline_level(&mut self, skyline_index: usize, x: i32, y: i32, width: i32, height: i32) {
        self.skyline.insert(
            skyline_index,
            SkylineSegment {
                x,
                y: y.wrapping_add(height),
                width,
            },
        );

        let index = skyline_index + 1;
        while index < self.skyline.len() {
            let previous = self.skyline[index - 1];
            let segment = &mut self.skyline[index];
            if segment.x < previous.x.wrapping_add(previous.width) {
                let shrink = previous
                    .x
                    .wrapping_add(previous.width)
                    .wrapping_sub(segment.x);
                segment.x = segment.x.wrapping_add(shrink);
                segment.width = segment.width.wrapping_sub(shrink);

                if segment.width <= 0 {
                    self.skyline.remove(index);
                } else {
                    break;
                }
            } else {
                break;
            }
        }

        let mut index = 0;
        while index + 1 < self.skyline.len() {
            if self.skyline[index].y == self.skyline[index + 1].y {
                let next_width = self.skyline[index + 1].width;
                self.skyline[index].width = self.skyline[index].width.wrapping_add(next_width);
                self.skyline.remove(index + 1);
            } else {
                index += 1;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{pack_atlas_regions, AtlasPackError, Skyline};

    fn add_rect(skyline: &mut Skyline, width: i32, height: i32) -> Option<(i32, i32)> {
        let mut x = -1;
        let mut y = -1;
        skyline
            .add_rect(width, height, &mut x, &mut y)
            .then_some((x, y))
    }

    #[test]
    fn packs_at_the_lowest_position_then_the_narrowest_skyline_segment() {
        let mut skyline = Skyline::new(10, 10);

        assert_eq!(add_rect(&mut skyline, 6, 4), Some((0, 0)));
        assert_eq!(add_rect(&mut skyline, 4, 6), Some((6, 0)));
        assert_eq!(add_rect(&mut skyline, 4, 4), Some((0, 4)));
        assert_eq!(add_rect(&mut skyline, 6, 2), Some((4, 6)));
        assert_eq!(add_rect(&mut skyline, 10, 2), Some((0, 8)));
    }

    #[test]
    fn matches_cpp_placement_oracle_trace() {
        // Emitted by the two C++ source files named in this module's docs.
        let expected = [
            (6, 4, true, 0, 0),
            (4, 6, true, 6, 0),
            (4, 4, true, 0, 4),
            (6, 2, true, 4, 6),
            (10, 2, true, 0, 8),
            (1, 1, false, 0, 0),
        ];
        let mut skyline = Skyline::new(10, 10);

        for (width, height, placed, expected_x, expected_y) in expected {
            let mut x = -1;
            let mut y = -1;
            assert_eq!(
                skyline.add_rect(width, height, &mut x, &mut y),
                placed,
                "{width}x{height}",
            );
            assert_eq!((x, y), (expected_x, expected_y), "{width}x{height}");
        }
    }

    #[test]
    fn padded_rect_returns_the_content_origin_and_counts_the_padding() {
        let mut skyline = Skyline::new(10, 10);
        let mut x = -1;
        let mut y = -1;

        assert!(skyline.add_padded_rect(4, 3, 2, &mut x, &mut y));

        assert_eq!((x, y), (2, 2));
        assert!(!skyline.is_empty());
        assert_eq!(skyline.percent_full(), 0.56);
    }

    #[test]
    fn padded_oversize_rect_preserves_output_coordinates() {
        let mut skyline = Skyline::new(7, 7);
        let mut x = 17;
        let mut y = 29;

        assert!(!skyline.add_padded_rect(4, 3, 2, &mut x, &mut y));

        assert_eq!((x, y), (17, 29));
        assert!(skyline.is_empty());
    }

    #[test]
    fn oversize_rectangles_fail_without_changing_output_coordinates() {
        let mut skyline = Skyline::new(4, 4);
        let mut x = 17;
        let mut y = 29;

        assert!(!skyline.add_rect(5, 1, &mut x, &mut y));
        assert_eq!((x, y), (17, 29));
        assert!(!skyline.add_rect(-1, 1, &mut x, &mut y));
        assert_eq!((x, y), (17, 29));
        assert!(skyline.is_empty());
    }

    #[test]
    fn exhausted_atlas_fails_at_the_origin_without_consuming_space() {
        let mut skyline = Skyline::new(4, 4);

        assert_eq!(add_rect(&mut skyline, 4, 2), Some((0, 0)));
        let mut x = 17;
        let mut y = 29;
        assert!(!skyline.add_rect(4, 3, &mut x, &mut y));
        assert_eq!((x, y), (0, 0));
        assert_eq!(add_rect(&mut skyline, 4, 2), Some((0, 2)));
        assert_eq!(skyline.percent_full(), 1.0);
    }

    #[test]
    fn reset_restores_the_initial_skyline() {
        let mut skyline = Skyline::new(7, 5);
        assert_eq!((skyline.width(), skyline.height()), (7, 5));
        assert_eq!(add_rect(&mut skyline, 3, 5), Some((0, 0)));

        skyline.reset();

        assert!(skyline.is_empty());
        assert_eq!(add_rect(&mut skyline, 7, 5), Some((0, 0)));
    }

    #[test]
    fn atlas_layout_uses_the_occupied_extent_instead_of_vertical_capacity() {
        let layout = pack_atlas_regions(1920, 2048, &[(50, 100); 30]).unwrap();

        assert_eq!(layout.extent(), [1500, 100]);
        assert!(layout.extent()[1] <= 2048);
        assert!(layout.origins().iter().all(|&[_, y]| y == 0));
    }

    #[test]
    fn many_regions_do_not_overflow_a_temporary_summed_height() {
        let layout = pack_atlas_regions(1920, 2048, &[(50, 100); 328]).unwrap();

        assert_eq!(layout.extent(), [1900, 900]);
        assert_eq!(layout.origins().last(), Some(&[1150, 800]));
    }

    #[test]
    fn atlas_layout_fails_before_exceeding_the_device_texture_limit() {
        let result = pack_atlas_regions(1920, 2048, &[(1920, 100); 21]);

        assert!(matches!(result, Err(AtlasPackError::Full)));
    }

    #[test]
    fn atlas_layout_rejects_dimension_overflow() {
        let result = pack_atlas_regions(1920, 2048, &[(50, u32::MAX), (50, 1)]);

        assert!(matches!(result, Err(AtlasPackError::InvalidDimensions)));
    }
}
