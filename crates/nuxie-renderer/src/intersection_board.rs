//! Rectangle draw-group assignment based on tiled intersection tracking.
//!
//! C++ source parity:
//! - `/Users/levi/dev/oss/rive-runtime/renderer/src/intersection_board.hpp`
//! - `/Users/levi/dev/oss/rive-runtime/renderer/src/intersection_board.cpp`
//! - `/Users/levi/dev/oss/rive-runtime/tests/unit_tests/renderer/intersection_board_test.cpp`
//!
//! The implementation is deliberately scalar for now. It preserves the C++
//! module's grouping, strict-overlap, baseline, and eight-lane result contract;
//! SIMD storage can be introduced later without changing this interface.

/// Whether a rectangle may share a draw group with rectangles it overlaps.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum GroupingType {
    Disjoint,
    OverlapAllowed,
}

/// An axis-aligned rectangle with exclusive right and bottom edges.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Rect {
    pub left: i32,
    pub top: i32,
    pub right: i32,
    pub bottom: i32,
}

impl Rect {
    pub const fn new(left: i32, top: i32, right: i32, bottom: i32) -> Self {
        Self {
            left,
            top,
            right,
            bottom,
        }
    }

    fn is_non_empty(self) -> bool {
        self.left < self.right && self.top < self.bottom
    }

    fn intersects(self, other: Self) -> bool {
        self.left < other.right
            && self.top < other.bottom
            && self.right > other.left
            && self.bottom > other.top
    }

    fn clamp_to(self, width: i32, height: i32) -> Self {
        Self::new(
            self.left.clamp(0, width),
            self.top.clamp(0, height),
            self.right.clamp(0, width),
            self.bottom.clamp(0, height),
        )
    }
}

/// Per-lane intermediate result used while accumulating results from tiles.
///
/// This mirrors `IntersectionTile::FindResult` in the C++ source. Consumers
/// reduce `max_group_indices` to find the highest group and combine overlap
/// bits only from lanes that match that maximum.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct FindResult {
    pub max_group_indices: [i16; 8],
    pub overlap_bits: [u16; 8],
}

#[derive(Clone, Copy, Debug)]
struct StoredRect {
    rect: Rect,
    group_index: i16,
}

/// One 255 by 255 region of an [`IntersectionBoard`].
#[derive(Clone, Debug, Default)]
pub struct IntersectionTile {
    top_left: (i32, i32),
    baseline_group_index: i16,
    baseline_overlap_bits: u16,
    max_group_index: i16,
    overlap_bits_for_max_group: u16,
    rectangles: Vec<StoredRect>,
    // `None` is the C++ disjoint-mode invariant: no overlap storage exists.
    overlap_bits: Option<Vec<u16>>,
}

impl IntersectionTile {
    pub const TILE_DIM: i32 = 255;
    const CHUNK_SIZE: usize = 8;

    pub fn reset(
        &mut self,
        left: i32,
        top: i32,
        baseline_group_index: i16,
        baseline_overlap_bits: u16,
    ) {
        assert!(baseline_group_index >= 0);
        self.top_left = (left, top);
        self.baseline_group_index = baseline_group_index;
        self.baseline_overlap_bits = baseline_overlap_bits;
        self.max_group_index = baseline_group_index;
        self.overlap_bits_for_max_group = baseline_overlap_bits;
        self.rectangles.clear();
        self.overlap_bits = None;
    }

    pub fn add_rectangle(
        &mut self,
        grouping_type: GroupingType,
        ltrb: Rect,
        group_index: i16,
        current_rectangle_overlap_bits: u16,
    ) {
        assert!(ltrb.is_non_empty());
        assert!(group_index >= 0);

        match grouping_type {
            GroupingType::OverlapAllowed => {
                if self.overlap_bits.is_none() {
                    self.overlap_bits = Some(vec![0; self.rectangles.len()]);
                }
            }
            GroupingType::Disjoint => assert_eq!(current_rectangle_overlap_bits, 0),
        }

        debug_assert!(self.addition_preserves_grouping(grouping_type, ltrb, group_index,));

        let local = self.local_rect(ltrb);
        assert!(local.left < Self::TILE_DIM);
        assert!(local.top < Self::TILE_DIM);
        assert!(local.right > 0);
        assert!(local.bottom > 0);

        if grouping_type == GroupingType::OverlapAllowed
            && group_index == self.baseline_group_index
            && (current_rectangle_overlap_bits | self.baseline_overlap_bits)
                == self.baseline_overlap_bits
        {
            return;
        }

        if self.covers_tile(local) {
            match grouping_type {
                GroupingType::OverlapAllowed => {
                    assert!(group_index >= self.max_group_index);
                    if group_index == self.max_group_index {
                        self.update_baseline_to_max_group_index(current_rectangle_overlap_bits);
                        debug_assert!(self.invariants_hold());
                        return;
                    }
                }
                GroupingType::Disjoint => assert!(group_index > self.max_group_index),
            }

            self.reset(
                self.top_left.0,
                self.top_left.1,
                group_index,
                current_rectangle_overlap_bits,
            );
            debug_assert!(self.invariants_hold());
            return;
        }

        self.rectangles.push(StoredRect {
            rect: local,
            group_index,
        });
        match grouping_type {
            GroupingType::OverlapAllowed => {
                let overlap_bits = self
                    .overlap_bits
                    .as_mut()
                    .expect("overlap mode allocates overlap storage");
                overlap_bits.push(current_rectangle_overlap_bits);
                if group_index > self.max_group_index {
                    self.max_group_index = group_index;
                    self.overlap_bits_for_max_group = current_rectangle_overlap_bits;
                } else if group_index == self.max_group_index {
                    self.overlap_bits_for_max_group |= current_rectangle_overlap_bits;
                }
            }
            GroupingType::Disjoint => {
                self.max_group_index = self.max_group_index.max(group_index);
            }
        }
        debug_assert!(self.invariants_hold());
    }

    pub fn find_max_intersecting_group_index(
        &self,
        grouping_type: GroupingType,
        ltrb: Rect,
        mut running: FindResult,
    ) -> FindResult {
        assert!(ltrb.is_non_empty());
        assert!(running.max_group_indices.iter().all(|index| *index >= 0));
        assert!(ltrb.left < self.top_left.0 + Self::TILE_DIM);
        assert!(ltrb.top < self.top_left.1 + Self::TILE_DIM);
        assert!(ltrb.right > self.top_left.0);
        assert!(ltrb.bottom > self.top_left.1);

        let local = self.local_rect(ltrb);
        if self.covers_tile(local) {
            match grouping_type {
                GroupingType::OverlapAllowed => {
                    let current_max = running.max_group_indices[0];
                    if current_max < self.max_group_index {
                        running.max_group_indices[0] = self.max_group_index;
                        running.overlap_bits[0] = self.overlap_bits_for_max_group;
                    } else if current_max == self.max_group_index {
                        running.overlap_bits[0] |= self.overlap_bits_for_max_group;
                    }
                }
                GroupingType::Disjoint => {
                    running.max_group_indices[0] =
                        running.max_group_indices[0].max(self.max_group_index);
                }
            }
            return running;
        }

        for (index, stored) in self.rectangles.iter().enumerate() {
            let lane = index % Self::CHUNK_SIZE;
            let masked_group_index = if stored.rect.intersects(local) {
                stored.group_index
            } else {
                0
            };

            if grouping_type == GroupingType::OverlapAllowed {
                if masked_group_index > running.max_group_indices[lane] {
                    running.overlap_bits[lane] = 0;
                }
                if running.max_group_indices[lane] <= masked_group_index {
                    // Like the C++ SIMD loop, this can retain irrelevant bits
                    // in lanes with no intersection. Board-level reduction
                    // filters them by the final maximum group index.
                    running.overlap_bits[lane] |= self.overlap_bits_at(index);
                }
            }
            running.max_group_indices[lane] =
                running.max_group_indices[lane].max(masked_group_index);
        }

        match grouping_type {
            GroupingType::OverlapAllowed => {
                if running.max_group_indices[0] < self.baseline_group_index {
                    running.max_group_indices[0] = self.baseline_group_index;
                    running.overlap_bits[0] = self.baseline_overlap_bits;
                } else if running.max_group_indices[0] == self.baseline_group_index {
                    running.overlap_bits[0] |= self.baseline_overlap_bits;
                }
            }
            GroupingType::Disjoint => {
                running.max_group_indices[0] =
                    running.max_group_indices[0].max(self.baseline_group_index);
            }
        }
        running
    }

    fn local_rect(&self, rect: Rect) -> Rect {
        Rect::new(
            (rect.left - self.top_left.0).clamp(0, Self::TILE_DIM),
            (rect.top - self.top_left.1).clamp(0, Self::TILE_DIM),
            (rect.right - self.top_left.0).clamp(0, Self::TILE_DIM),
            (rect.bottom - self.top_left.1).clamp(0, Self::TILE_DIM),
        )
    }

    fn covers_tile(&self, rect: Rect) -> bool {
        rect == Rect::new(0, 0, Self::TILE_DIM, Self::TILE_DIM)
    }

    fn overlap_bits_at(&self, index: usize) -> u16 {
        self.overlap_bits
            .as_ref()
            .and_then(|bits| bits.get(index))
            .copied()
            .unwrap_or(0)
    }

    fn addition_preserves_grouping(
        &self,
        grouping_type: GroupingType,
        ltrb: Rect,
        group_index: i16,
    ) -> bool {
        let max_group = self
            .find_max_intersecting_group_index(grouping_type, ltrb, FindResult::default())
            .max_group_indices
            .into_iter()
            .max()
            .unwrap_or(0);
        match grouping_type {
            GroupingType::OverlapAllowed => {
                group_index >= max_group && group_index >= self.baseline_group_index
            }
            GroupingType::Disjoint => {
                group_index > max_group && group_index > self.baseline_group_index
            }
        }
    }

    fn update_baseline_to_max_group_index(&mut self, additional_baseline_overlap_bits: u16) {
        let existing_max_bits = self.overlap_bits_for_max_group;
        if (additional_baseline_overlap_bits | existing_max_bits)
            == additional_baseline_overlap_bits
            || self.rectangles.is_empty()
        {
            self.reset(
                self.top_left.0,
                self.top_left.1,
                self.max_group_index,
                existing_max_bits | additional_baseline_overlap_bits,
            );
            return;
        }

        self.overlap_bits_for_max_group |= additional_baseline_overlap_bits;
        if self.max_group_index == self.baseline_group_index {
            if self.overlap_bits_for_max_group == self.baseline_overlap_bits {
                return;
            }
            self.baseline_overlap_bits |= additional_baseline_overlap_bits;
        } else {
            self.baseline_group_index = self.max_group_index;
            self.baseline_overlap_bits = additional_baseline_overlap_bits;
        }

        let overlaps = self
            .overlap_bits
            .as_mut()
            .expect("baseline updates require overlap storage");
        let mut keep = self
            .rectangles
            .iter()
            .zip(overlaps.iter())
            .map(|(stored, overlap)| {
                stored.group_index >= self.baseline_group_index
                    && (self.baseline_overlap_bits | overlap) != self.baseline_overlap_bits
            })
            .collect::<Vec<_>>();

        let Some(mut first) = keep.iter().position(|keep| !keep) else {
            return;
        };
        let mut last = keep.iter().rposition(|keep| *keep).unwrap_or(first);
        while first < last {
            self.rectangles[first] = self.rectangles[last];
            overlaps[first] = overlaps[last];
            keep[last] = false;

            first += 1;
            while first < self.rectangles.len() && keep[first] {
                first += 1;
            }
            while last > first && !keep[last] {
                last -= 1;
            }
        }
        self.rectangles.truncate(first);
        overlaps.truncate(first);
    }

    fn invariants_hold(&self) -> bool {
        self.baseline_group_index >= 0
            && self.max_group_index >= self.baseline_group_index
            && self
                .overlap_bits
                .as_ref()
                .is_none_or(|bits| bits.len() == self.rectangles.len())
            && self.rectangles.iter().all(|stored| {
                stored.group_index >= self.baseline_group_index
                    && stored.group_index <= self.max_group_index
            })
    }
}

/// Tiled collection of rectangles that assigns non-conflicting draw groups.
#[derive(Clone, Debug)]
pub struct IntersectionBoard {
    grouping_type: GroupingType,
    viewport_size: (i32, i32),
    cols: i32,
    rows: i32,
    tiles: Vec<IntersectionTile>,
}

impl IntersectionBoard {
    pub const TILE_DIM: i32 = IntersectionTile::TILE_DIM;

    pub fn new(grouping_type: GroupingType) -> Self {
        Self {
            grouping_type,
            viewport_size: (0, 0),
            cols: 0,
            rows: 0,
            tiles: Vec::new(),
        }
    }

    pub fn grouping_type(&self) -> GroupingType {
        self.grouping_type
    }

    pub fn resize_and_reset(&mut self, viewport_width: u32, viewport_height: u32) {
        let width = i32::try_from(viewport_width).expect("viewport width exceeds i32");
        let height = i32::try_from(viewport_height).expect("viewport height exceeds i32");
        self.viewport_size = (width, height);
        self.cols = (width + Self::TILE_DIM - 1) / Self::TILE_DIM;
        self.rows = (height + Self::TILE_DIM - 1) / Self::TILE_DIM;
        let tile_count = usize::try_from(self.cols * self.rows).expect("tile count is negative");
        if self.tiles.len() < tile_count {
            self.tiles
                .resize_with(tile_count, IntersectionTile::default);
        }
        for y in 0..self.rows {
            for x in 0..self.cols {
                self.tiles[usize::try_from(y * self.cols + x).expect("tile index is negative")]
                    .reset(x * Self::TILE_DIM, y * Self::TILE_DIM, 0, 0);
            }
        }
    }

    pub fn add_rectangle(&mut self, ltrb: Rect, layer_count: i16) -> i16 {
        self.add_rectangle_with_overlap(ltrb, 0, 0, layer_count)
    }

    pub fn add_rectangle_with_overlap(
        &mut self,
        ltrb: Rect,
        current_rectangle_overlap_bits: u16,
        disallowed_overlap_bits_mask: u16,
        layer_count: i16,
    ) -> i16 {
        assert!(layer_count > 0);
        if self.grouping_type == GroupingType::OverlapAllowed {
            assert_eq!(layer_count, 1);
        }

        let (width, height) = self.viewport_size;
        if ltrb.left >= width
            || ltrb.top >= height
            || ltrb.right <= 0
            || ltrb.bottom <= 0
            || !ltrb.is_non_empty()
        {
            return 0;
        }
        let ltrb = ltrb.clamp_to(width, height);
        let min_x = ltrb.left / Self::TILE_DIM;
        let min_y = ltrb.top / Self::TILE_DIM;
        let max_x = (ltrb.right - 1) / Self::TILE_DIM;
        let max_y = (ltrb.bottom - 1) / Self::TILE_DIM;
        assert!(min_x <= max_x && min_y <= max_y);

        let mut results = FindResult::default();
        for y in min_y..=max_y {
            for x in min_x..=max_x {
                results = self.tiles[self.tile_index(x, y)].find_max_intersecting_group_index(
                    self.grouping_type,
                    ltrb,
                    results,
                );
            }
        }

        let mut bottom_group_index = results.max_group_indices.into_iter().max().unwrap_or(0);
        assert!(bottom_group_index <= i16::MAX - layer_count);
        match self.grouping_type {
            GroupingType::OverlapAllowed => {
                let mut all_overlap_bits = 0;
                for lane in 0..IntersectionTile::CHUNK_SIZE {
                    if results.max_group_indices[lane] == bottom_group_index {
                        all_overlap_bits |= results.overlap_bits[lane];
                    }
                }
                if all_overlap_bits & disallowed_overlap_bits_mask != 0 {
                    bottom_group_index += 1;
                } else {
                    bottom_group_index = bottom_group_index.max(1);
                }
            }
            GroupingType::Disjoint => bottom_group_index += 1,
        }

        let top_group_index = bottom_group_index + layer_count - 1;
        for y in min_y..=max_y {
            for x in min_x..=max_x {
                let tile_index = self.tile_index(x, y);
                self.tiles[tile_index].add_rectangle(
                    self.grouping_type,
                    ltrb,
                    top_group_index,
                    current_rectangle_overlap_bits,
                );
            }
        }
        bottom_group_index
    }

    fn tile_index(&self, x: i32, y: i32) -> usize {
        usize::try_from(y * self.cols + x).expect("tile index is negative")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Independent oracle based on
    // `/Users/levi/dev/oss/rive-runtime/tests/common/intersection_board_reference_impl.hpp`.
    // It intentionally has no tiles, baselines, or compaction state.
    struct ReferenceBoard {
        grouping_type: GroupingType,
        viewport_size: (i32, i32),
        rectangles: Vec<(Rect, i16, u16)>,
    }

    impl ReferenceBoard {
        fn new(grouping_type: GroupingType, width: i32, height: i32) -> Self {
            Self {
                grouping_type,
                viewport_size: (width, height),
                rectangles: Vec::new(),
            }
        }

        fn add(&mut self, ltrb: Rect, overlap_bits: u16, disallowed_bits: u16, layers: i16) -> i16 {
            let (width, height) = self.viewport_size;
            if ltrb.left >= width
                || ltrb.top >= height
                || ltrb.right <= 0
                || ltrb.bottom <= 0
                || !ltrb.is_non_empty()
            {
                return 0;
            }
            let ltrb = ltrb.clamp_to(width, height);
            let mut max_group = 0;
            let mut max_overlap_bits = 0;
            for (existing, group, existing_overlap_bits) in &self.rectangles {
                if existing.intersects(ltrb) {
                    if *group > max_group {
                        max_group = *group;
                        max_overlap_bits = *existing_overlap_bits;
                    } else if *group == max_group {
                        max_overlap_bits |= *existing_overlap_bits;
                    }
                }
            }
            let bottom_group = match self.grouping_type {
                GroupingType::Disjoint => max_group + 1,
                GroupingType::OverlapAllowed if max_overlap_bits & disallowed_bits != 0 => {
                    max_group + 1
                }
                GroupingType::OverlapAllowed => max_group.max(1),
            };
            self.rectangles
                .push((ltrb, bottom_group + layers - 1, overlap_bits));
            bottom_group
        }
    }

    struct Lcg(u64);

    impl Lcg {
        fn next_u32(&mut self) -> u32 {
            self.0 = self.0.wrapping_mul(6364136223846793005).wrapping_add(1);
            (self.0 >> 32) as u32
        }

        fn range(&mut self, min: i32, max: i32) -> i32 {
            assert!(min <= max);
            min + (self.next_u32() % u32::try_from(max - min + 1).unwrap()) as i32
        }
    }

    fn max_and_relevant_overlap(result: FindResult) -> (i16, u16) {
        let max_group = result.max_group_indices.into_iter().max().unwrap();
        let overlap_bits = result
            .max_group_indices
            .into_iter()
            .zip(result.overlap_bits)
            .filter_map(|(group, overlap)| (group == max_group).then_some(overlap))
            .fold(0, |bits, overlap| bits | overlap);
        (max_group, overlap_bits)
    }

    #[test]
    fn touching_edges_do_not_intersect() {
        let mut tile = IntersectionTile::default();
        tile.reset(0, 0, 0, 0);
        tile.add_rectangle(GroupingType::Disjoint, Rect::new(0, 0, 10, 10), 1, 0);
        assert_eq!(
            max_and_relevant_overlap(tile.find_max_intersecting_group_index(
                GroupingType::Disjoint,
                Rect::new(10, 0, 20, 10),
                FindResult::default(),
            )),
            (0, 0),
        );

        let mut board = IntersectionBoard::new(GroupingType::Disjoint);
        board.resize_and_reset(510, 255);
        assert_eq!(board.add_rectangle(Rect::new(0, 0, 255, 255), 1), 1);
        assert_eq!(board.add_rectangle(Rect::new(255, 0, 510, 255), 1), 1);
        assert_eq!(board.add_rectangle(Rect::new(254, 0, 256, 255), 1), 2);
    }

    #[test]
    fn assigns_contiguous_layers_across_tile_boundaries() {
        let mut board = IntersectionBoard::new(GroupingType::Disjoint);
        board.resize_and_reset(800, 600);
        assert_eq!(board.grouping_type(), GroupingType::Disjoint);
        assert_eq!(board.add_rectangle(Rect::new(254, 254, 256, 256), 7), 1);
        assert_eq!(board.add_rectangle(Rect::new(254, 254, 255, 255), 1), 8);
        assert_eq!(board.add_rectangle(Rect::new(255, 0, 510, 255), 2), 8);
        assert_eq!(board.add_rectangle(Rect::new(255, 255, 256, 510), 3), 8);
        assert_eq!(board.add_rectangle(Rect::new(0, 255, 255, 256), 4), 8);
        assert_eq!(board.add_rectangle(Rect::new(0, 254, 800, 255), 1), 10);
        assert_eq!(board.add_rectangle(Rect::new(0, 255, 800, 256), 1), 12);
        assert_eq!(board.add_rectangle(Rect::new(0, 0, 800, 600), 1), 13);
    }

    #[test]
    fn discards_empty_and_offscreen_rectangles() {
        let mut board = IntersectionBoard::new(GroupingType::Disjoint);
        board.resize_and_reset(1000, 1000);
        assert_eq!(board.add_rectangle(Rect::new(0, 0, 0, 1), 1), 0);
        assert_eq!(board.add_rectangle(Rect::new(0, 0, 1, 0), 1), 0);
        assert_eq!(board.add_rectangle(Rect::new(1000, 999, 1001, 1001), 1), 0);
        assert_eq!(board.add_rectangle(Rect::new(999, 1000, 1001, 1001), 1), 0);
        assert_eq!(board.add_rectangle(Rect::new(8, 8, 8, 9), 1), 0);
        assert_eq!(board.add_rectangle(Rect::new(8, 8, 9, 8), 1), 0);
    }

    #[test]
    fn overlap_baseline_compacts_only_subsumed_rectangles() {
        let mut tile = IntersectionTile::default();
        tile.reset(0, 0, 0, 0);
        let full_tile = Rect::new(0, 0, IntersectionTile::TILE_DIM, IntersectionTile::TILE_DIM);
        tile.add_rectangle(GroupingType::OverlapAllowed, full_tile, 1, 0x1001);
        assert_eq!(tile.rectangles.len(), 0);

        tile.add_rectangle(
            GroupingType::OverlapAllowed,
            Rect::new(0, 0, 20, 20),
            1,
            0x0010,
        );
        tile.add_rectangle(
            GroupingType::OverlapAllowed,
            Rect::new(20, 20, 40, 40),
            1,
            0x1040,
        );
        tile.add_rectangle(
            GroupingType::OverlapAllowed,
            Rect::new(40, 40, 60, 60),
            1,
            0x0021,
        );
        tile.add_rectangle(
            GroupingType::OverlapAllowed,
            Rect::new(60, 60, 80, 80),
            1,
            0x2181,
        );
        assert_eq!(tile.rectangles.len(), 4);

        tile.add_rectangle(GroupingType::OverlapAllowed, full_tile, 1, 0x0030);
        assert_eq!(tile.rectangles.len(), 2);
        assert_eq!(tile.baseline_overlap_bits, 0x1031);
        let result = tile.find_max_intersecting_group_index(
            GroupingType::OverlapAllowed,
            Rect::new(50, 50, 70, 70),
            FindResult::default(),
        );
        assert_eq!(max_and_relevant_overlap(result), (1, 0x31b1));

        tile.add_rectangle(GroupingType::OverlapAllowed, full_tile, 1, 0x21c0);
        assert_eq!(tile.rectangles.len(), 0);
        assert_eq!(tile.baseline_overlap_bits, 0x31f1);
        assert!(tile.invariants_hold());
    }

    #[test]
    fn overlap_grouping_respects_disallowed_bits() {
        let mut board = IntersectionBoard::new(GroupingType::OverlapAllowed);
        board.resize_and_reset(100, 100);
        assert_eq!(
            board.add_rectangle_with_overlap(Rect::new(0, 0, 100, 100), 1, 0, 1),
            1
        );
        assert_eq!(
            board.add_rectangle_with_overlap(Rect::new(0, 0, 100, 100), 2, 0, 1),
            1
        );
        assert_eq!(
            board.add_rectangle_with_overlap(Rect::new(0, 0, 100, 100), 4, 0, 1),
            1
        );
        assert_eq!(
            board.add_rectangle_with_overlap(Rect::new(0, 0, 100, 100), 8, 0, 1),
            1
        );
        assert_eq!(
            board.add_rectangle_with_overlap(Rect::new(0, 0, 100, 100), 16, 1, 1),
            2
        );
        assert_eq!(
            board.add_rectangle_with_overlap(Rect::new(0, 0, 100, 100), 8, 1, 1),
            2
        );
    }

    #[test]
    fn property_disjoint_board_matches_reference_model() {
        for seed in 0..16 {
            let mut random = Lcg(seed + 1);
            let mut board = IntersectionBoard::new(GroupingType::Disjoint);
            board.resize_and_reset(800, 600);
            let mut reference = ReferenceBoard::new(GroupingType::Disjoint, 800, 600);
            for _ in 0..250 {
                let width = random.range(1, 700);
                let height = random.range(1, 700);
                let rect = Rect::new(
                    random.range(-width + 1, 799),
                    random.range(-height + 1, 599),
                    0,
                    0,
                );
                let rect = Rect::new(rect.left, rect.top, rect.left + width, rect.top + height);
                let layers = random.range(1, 4) as i16;
                assert_eq!(
                    board.add_rectangle(rect, layers),
                    reference.add(rect, 0, 0, layers),
                    "seed {seed}, rect {rect:?}",
                );
            }
        }
    }

    #[test]
    fn property_overlap_board_matches_reference_model() {
        for seed in 0..16 {
            let mut random = Lcg(seed + 1000);
            let mut board = IntersectionBoard::new(GroupingType::OverlapAllowed);
            board.resize_and_reset(800, 600);
            let mut reference = ReferenceBoard::new(GroupingType::OverlapAllowed, 800, 600);
            for _ in 0..250 {
                let width = random.range(1, 700);
                let height = random.range(1, 700);
                let left = random.range(-width + 1, 799);
                let top = random.range(-height + 1, 599);
                let rect = Rect::new(left, top, left + width, top + height);
                let overlap_bits =
                    (1_u16 << (random.next_u32() % 16)) | (1_u16 << (random.next_u32() % 16));
                let disallowed_bits =
                    (1_u16 << (random.next_u32() % 16)) | (1_u16 << (random.next_u32() % 16));
                assert_eq!(
                    board.add_rectangle_with_overlap(rect, overlap_bits, disallowed_bits, 1),
                    reference.add(rect, overlap_bits, disallowed_bits, 1),
                    "seed {seed}, rect {rect:?}, overlap {overlap_bits:#06x}, disallowed {disallowed_bits:#06x}",
                );
            }
        }
    }
}
