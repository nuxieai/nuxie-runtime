//! Where a decoded video frame's picture sits in its buffer and how it is
//! displayed: the crop, the quarter turns and the display size, plus the map
//! a GPU conversion samples through. Decoders that hand over their own
//! buffers (Android hardware buffers) describe frames this way.

/// Where the picture sits in a decoded video buffer and how it is displayed.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct VideoFrameGeometry {
    /// The picture's left, top, right and bottom edges in buffer pixels, all
    /// zero for the whole buffer. Decoders may pad the buffer beyond it.
    pub crop: [u32; 4],
    /// Clockwise quarter turns from buffer to display: 0, 1, 2 or 3.
    pub quarter_turns: u32,
    /// Displayed width and height after the turn, both zero for the turned
    /// crop. Video with non-square pixels displays wider or taller than its
    /// crop, and the frame is stretched to this size.
    pub display: [u32; 2],
}

impl VideoFrameGeometry {
    /// The size the frame displays at in a buffer of `buffer_size`, or `None`
    /// when the geometry does not fit it.
    pub fn display_size(&self, buffer_size: (u32, u32)) -> Option<(u32, u32)> {
        FrameRegion::resolve(*self, buffer_size)
            .ok()
            .map(|region| region.display)
    }
}

/// A frame's geometry with the whole-buffer and turned-crop defaults applied.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct FrameRegion {
    /// Left, top, right and bottom edges in buffer pixels.
    pub crop: [u32; 4],
    /// Clockwise quarter turns from buffer to display: 0, 1, 2 or 3.
    pub quarter_turns: u32,
    /// Width and height of the converted frame.
    pub display: (u32, u32),
}

impl FrameRegion {
    /// Resolves `geometry` for a buffer of `buffer_size`, or explains why it
    /// does not fit.
    pub(crate) fn resolve(
        geometry: VideoFrameGeometry,
        buffer_size: (u32, u32),
    ) -> Result<Self, String> {
        let VideoFrameGeometry {
            crop,
            quarter_turns,
            display,
        } = geometry;
        let crop = if crop == [0; 4] {
            [0, 0, buffer_size.0, buffer_size.1]
        } else {
            crop
        };
        let [left, top, right, bottom] = crop;
        if quarter_turns > 3
            || left >= right
            || top >= bottom
            || right > buffer_size.0
            || bottom > buffer_size.1
        {
            return Err(format!(
                "crop {crop:?} turned {quarter_turns} times does not fit a {}x{} buffer",
                buffer_size.0, buffer_size.1
            ));
        }
        let display = match display {
            [0, 0] if quarter_turns % 2 == 1 => (bottom - top, right - left),
            [0, 0] => (right - left, bottom - top),
            [width, height] if width > 0 && height > 0 => (width, height),
            _ => return Err(format!("display size {display:?} is half set")),
        };
        Ok(Self {
            crop,
            quarter_turns,
            display,
        })
    }

    /// Display width and height of the converted frame.
    pub(crate) fn display_extent(&self) -> (u32, u32) {
        self.display
    }

    /// Rows `s` and `t` of the affine map from a target point (u, v), top-left
    /// origin in [0, 1], to normalized source coordinates.
    #[cfg_attr(not(target_os = "android"), allow(dead_code))]
    pub(crate) fn source_transform(&self, buffer_width: u32, buffer_height: u32) -> [[f32; 4]; 2] {
        // Turning the buffer clockwise to display it means a display point
        // reads from the buffer point turned back counterclockwise.
        let (s, t): ([f32; 3], [f32; 3]) = match self.quarter_turns {
            0 => ([1.0, 0.0, 0.0], [0.0, 1.0, 0.0]),
            1 => ([0.0, 1.0, 0.0], [-1.0, 0.0, 1.0]),
            2 => ([-1.0, 0.0, 1.0], [0.0, -1.0, 1.0]),
            _ => ([0.0, -1.0, 1.0], [1.0, 0.0, 0.0]),
        };
        let [left, top, right, bottom] = self.crop.map(|edge| edge as f32);
        let (width, height) = (buffer_width as f32, buffer_height as f32);
        let crop = |row: [f32; 3], origin: f32, extent: f32, size: f32| {
            let scale = extent / size;
            [
                row[0] * scale,
                row[1] * scale,
                row[2] * scale + origin / size,
                0.0,
            ]
        };
        [
            crop(s, left, right - left, width),
            crop(t, top, bottom - top, height),
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::{FrameRegion, VideoFrameGeometry};

    fn region(crop: [u32; 4], quarter_turns: u32, buffer: (u32, u32)) -> FrameRegion {
        let geometry = VideoFrameGeometry {
            crop,
            quarter_turns,
            display: [0, 0],
        };
        FrameRegion::resolve(geometry, buffer).unwrap()
    }

    fn map(region: FrameRegion, buffer: (u32, u32), (u, v): (f32, f32)) -> (f32, f32) {
        let [s, t] = region.source_transform(buffer.0, buffer.1);
        (s[0] * u + s[1] * v + s[2], t[0] * u + t[1] * v + t[2])
    }

    fn close(actual: (f32, f32), expected: (f32, f32)) -> bool {
        (actual.0 - expected.0).abs() < 1e-6 && (actual.1 - expected.1).abs() < 1e-6
    }

    #[test]
    fn an_unturned_frame_maps_display_corners_to_its_crop() {
        let region = region([0, 0, 1920, 1080], 0, (1920, 1088));
        assert_eq!(region.display_extent(), (1920, 1080));
        // A 1088-row buffer pads the picture below row 1080.
        assert!(close(map(region, (1920, 1088), (0.0, 0.0)), (0.0, 0.0)));
        assert!(close(
            map(region, (1920, 1088), (1.0, 1.0)),
            (1.0, 1080.0 / 1088.0)
        ));
    }

    #[test]
    fn turned_frames_read_the_buffer_corner_that_lands_top_left() {
        let crop = [8, 4, 72, 36];
        let buffer = (80, 40);
        let corner = |edge_u: u32, edge_v: u32| {
            (
                edge_u as f32 / buffer.0 as f32,
                edge_v as f32 / buffer.1 as f32,
            )
        };
        // Display top-left shows the buffer's top-left, bottom-left,
        // bottom-right and top-right corner for 0 to 3 clockwise turns.
        let expected = [corner(8, 4), corner(8, 36), corner(72, 36), corner(72, 4)];
        for (quarter_turns, expected) in expected.into_iter().enumerate() {
            let region = region(crop, quarter_turns as u32, buffer);
            assert!(
                close(map(region, buffer, (0.0, 0.0)), expected),
                "{quarter_turns}"
            );
        }
        let turned = region(crop, 1, buffer);
        assert_eq!(turned.display_extent(), (32, 64));
        // After one clockwise turn the buffer's top-left lands top-right.
        assert!(close(map(turned, buffer, (1.0, 0.0)), corner(8, 4)));
    }

    #[test]
    fn frames_with_non_square_pixels_stretch_their_crop_to_the_display_size() {
        // 64x32 stored pixels that are twice as wide as they are tall.
        let geometry = VideoFrameGeometry {
            crop: [0; 4],
            quarter_turns: 0,
            display: [128, 32],
        };
        let wide = FrameRegion::resolve(geometry, (64, 32)).unwrap();
        assert_eq!(wide.crop, [0, 0, 64, 32]);
        assert_eq!(wide.display_extent(), (128, 32));
        // The whole display still reads the whole crop.
        assert!(close(map(wide, (64, 32), (1.0, 1.0)), (1.0, 1.0)));
    }

    #[test]
    fn geometry_that_does_not_fit_the_buffer_is_rejected() {
        let resolve = |crop, quarter_turns, display| {
            let geometry = VideoFrameGeometry {
                crop,
                quarter_turns,
                display,
            };
            FrameRegion::resolve(geometry, (64, 32))
        };
        assert!(resolve([0, 0, 65, 32], 0, [0, 0]).is_err());
        assert!(resolve([8, 0, 8, 32], 0, [0, 0]).is_err());
        assert!(resolve([0; 4], 4, [0, 0]).is_err());
        assert!(resolve([0; 4], 0, [128, 0]).is_err());
        assert!(resolve([0; 4], 0, [0, 0]).is_ok());
    }
}
