/// One authoritative shaping/layout result shared by drawing and editor reads.
///
/// Keeping caret, hit, and selection geometry on this exact path prevents the
/// editor from growing an observation-side text layout that can drift from the
/// glyphs the runtime draws.
#[derive(Debug, Clone)]
pub(crate) struct StaticShapedTextLayout {
    text: String,
    lines: Vec<StaticShapedTextLine>,
    caret_boundaries: Option<Vec<StaticCaretBoundary>>,
    local_transform: Mat2D,
    shape_world: Mat2D,
    has_geometric_modifiers: bool,
    has_non_monotone_advances: bool,
}

/// C++ `Text::{m_shape,m_lines}`: retained HarfBuzz glyph topology and line
/// breaks, before `buildRenderStyles` applies positioning, modifiers, and the
/// local render transform.
#[derive(Debug, Clone)]
pub(crate) struct StaticShapedTextTopology {
    text: String,
    resolved_runs: Vec<StaticResolvedRun>,
    contextual_glyphs: Vec<StyledTextGlyph>,
    lines: Vec<StaticTextLine>,
    font_scale: f32,
}
#[derive(Debug, Clone)]
struct StaticShapedTextLine {
    line_index: usize,
    char_start: usize,
    char_end: usize,
    soft_wrap_skipped_start: Option<usize>,
    terminal_soft_wrap_skipped_end: Option<usize>,
    start_x: f32,
    end_x: f32,
    top: f32,
    baseline: f32,
    bottom: f32,
    glyphs: Vec<StaticPositionedTextGlyph>,
}
#[derive(Debug, Clone)]
struct StaticPositionedTextGlyph {
    glyph: StyledTextGlyph,
    x: f32,
    modifier_transform: Mat2D,
    modifier_opacity: f32,
}
impl StaticShapedTextLayout {
    fn caret(&self, byte_offset: usize) -> Option<(RenderVec2D, RenderVec2D)> {
        if !self.geometry_is_finite() {
            return None;
        }
        let char_index = self.char_index_at_byte(byte_offset)?;
        self.caret_for_char_index(char_index, StaticCaretAffinity::Downstream)
            .filter(|(top, bottom)| text_point_is_finite(*top) && text_point_is_finite(*bottom))
    }

    fn caret_for_char_index(
        &self,
        char_index: usize,
        affinity: StaticCaretAffinity,
    ) -> Option<(RenderVec2D, RenderVec2D)> {
        let segment = self
            .caret_boundaries
            .as_ref()?
            .get(char_index)?
            .segment(affinity)?;
        Some((segment.top, segment.bottom))
    }

    fn hit(&self, point: RenderVec2D) -> Option<usize> {
        if !self.geometry_is_finite() || !text_point_is_finite(point) {
            return None;
        }
        let determinant = self.shape_world.determinant();
        if !determinant.is_finite() || determinant == 0.0 {
            return None;
        }
        if self.has_geometric_modifiers || self.has_non_monotone_advances {
            let mut best: Option<(f32, usize, StaticCaretAffinity)> = None;
            for boundary in self.caret_boundaries.as_ref()? {
                for affinity in [
                    StaticCaretAffinity::Upstream,
                    StaticCaretAffinity::Downstream,
                ] {
                    let Some(segment) = boundary.segment(affinity) else {
                        continue;
                    };
                    let distance =
                        point_segment_distance_squared(point, segment.top, segment.bottom);
                    if !distance.is_finite() {
                        continue;
                    }
                    if best.is_none_or(|best| {
                        distance < best.0
                            || (distance == best.0
                                && text_hit_candidate_wins_tie(
                                    boundary.byte_offset,
                                    affinity,
                                    best.1,
                                    best.2,
                                ))
                    }) {
                        best = Some((distance, boundary.byte_offset, affinity));
                    }
                }
            }
            return best.map(|(_, byte_offset, _)| byte_offset);
        }
        let inverse = self.shape_world.invert_or_identity();
        let (x, y) = inverse.transform_point(point.x, point.y);
        if !x.is_finite() || !y.is_finite() {
            return None;
        }
        let line_index = self
            .lines
            .iter()
            .position(|line| y <= line.bottom)
            .or_else(|| self.lines.len().checked_sub(1))?;
        let line = self.lines.get(line_index)?;
        let mut char_index = line.hit_char_index(x).min(self.text.chars().count());
        if char_index == line.char_end
            && let Some(next_line) = line_index
                .checked_add(1)
                .and_then(|next_line| self.lines.get(next_line))
            && next_line.soft_wrap_skipped_start == Some(char_index)
        {
            char_index = next_line.char_start;
        } else if char_index == line.char_end
            && let Some(skipped_end) = line.terminal_soft_wrap_skipped_end
        {
            char_index = skipped_end;
        }
        Some(char_byte_index(&self.text, char_index))
    }

    fn selection_rects(&self, range: std::ops::Range<usize>) -> Vec<RenderAabb> {
        if !self.geometry_is_finite() || range.is_empty() {
            return Vec::new();
        }
        let Some(start) = self.char_index_at_byte(range.start) else {
            return Vec::new();
        };
        let Some(end) = self.char_index_at_byte(range.end) else {
            return Vec::new();
        };
        if start >= end {
            return Vec::new();
        }

        let mut rects = Vec::new();
        for line in &self.lines {
            let selected_start = start.max(line.char_start);
            let selected_end = end.min(line.char_end);
            if selected_start >= selected_end {
                continue;
            }
            if let Some(rect) = line.selection_rect(
                selected_start,
                selected_end,
                self.shape_world,
                self.has_geometric_modifiers,
                self.has_non_monotone_advances,
            ) {
                rects.push(rect);
            }
        }
        if rects.iter().all(|rect| text_aabb_is_finite(*rect)) {
            rects
        } else {
            Vec::new()
        }
    }

    fn char_index_at_byte(&self, byte_offset: usize) -> Option<usize> {
        self.caret_boundaries
            .as_ref()?
            .binary_search_by_key(&byte_offset, |boundary| boundary.byte_offset)
            .ok()
    }

    fn geometry_is_finite(&self) -> bool {
        self.local_transform.0.iter().all(|value| value.is_finite())
            && self.shape_world.0.iter().all(|value| value.is_finite())
            && self
                .lines
                .iter()
                .all(StaticShapedTextLine::geometry_is_finite)
            && self.caret_boundaries.as_ref().is_some_and(|boundaries| {
                boundaries.iter().all(|boundary| {
                    [boundary.upstream, boundary.downstream]
                        .into_iter()
                        .flatten()
                        .all(|segment| {
                            text_point_is_finite(segment.top)
                                && text_point_is_finite(segment.bottom)
                        })
                })
            })
    }
}
impl StaticShapedTextLine {
    fn geometry_is_finite(&self) -> bool {
        [
            self.start_x,
            self.end_x,
            self.top,
            self.baseline,
            self.bottom,
        ]
        .into_iter()
        .all(f32::is_finite)
            && self.glyphs.iter().all(|positioned| {
                positioned.x.is_finite()
                    && positioned.glyph.advance.is_finite()
                    && positioned.modifier_opacity.is_finite()
                    && positioned
                        .modifier_transform
                        .0
                        .iter()
                        .all(|value| value.is_finite())
            })
    }

    fn write_caret_boundaries(
        &self,
        shape_world: Mat2D,
        boundaries: &mut [StaticCaretBoundary],
        work: &mut StaticCaretBuildWork,
    ) {
        let clusters = self.positioned_clusters(work);
        if clusters.iter().any(|cluster| cluster.rtl) {
            for cluster in &clusters {
                let character_count = cluster.char_end.saturating_sub(cluster.char_start);
                for char_index in cluster.char_start..=cluster.char_end {
                    work.boundary_visits = work.boundary_visits.saturating_add(1);
                    let logical_ratio = if character_count == 0 {
                        0.0
                    } else {
                        (char_index - cluster.char_start) as f32 / character_count as f32
                    };
                    let ratio = if cluster.rtl {
                        1.0 - logical_ratio
                    } else {
                        logical_ratio
                    };
                    let x = cluster.start_x + (cluster.end_x - cluster.start_x) * ratio;
                    let glyph = self.glyphs.get(cluster.last_glyph);
                    let segment = self.caret_segment(x, glyph, shape_world);
                    if let Some(boundary) = boundaries.get_mut(char_index) {
                        boundary.upstream.get_or_insert(segment);
                        boundary.downstream = Some(segment);
                    }
                }
            }
            return;
        }
        let mut x_cluster = 0;
        let mut previous_cluster = None;
        let mut previous_cursor = 0;
        let mut next_cluster = 0;

        for char_index in self.char_start..=self.char_end {
            work.boundary_visits = work.boundary_visits.saturating_add(1);
            while clusters
                .get(x_cluster)
                .is_some_and(|cluster| cluster.char_end < char_index)
            {
                x_cluster = x_cluster.saturating_add(1);
            }
            while clusters
                .get(previous_cursor)
                .is_some_and(|cluster| cluster.char_end <= char_index)
            {
                previous_cluster = Some(previous_cursor);
                previous_cursor = previous_cursor.saturating_add(1);
            }
            while clusters
                .get(next_cluster)
                .is_some_and(|cluster| cluster.char_start < char_index)
            {
                next_cluster = next_cluster.saturating_add(1);
            }

            let current = clusters.get(x_cluster).copied();
            let containing = current
                .filter(|cluster| cluster.char_start < char_index && char_index < cluster.char_end);
            let x = if char_index <= self.char_start {
                self.start_x
            } else if let Some(cluster) = current {
                if char_index <= cluster.char_start {
                    cluster.start_x
                } else {
                    cluster.end_x
                }
            } else {
                self.end_x
            };

            let upstream_glyph = containing
                .map(|cluster| cluster.last_glyph)
                .or_else(|| {
                    previous_cluster
                        .and_then(|index| clusters.get(index))
                        .map(|cluster| cluster.last_glyph)
                })
                .or_else(|| clusters.first().map(|cluster| cluster.first_glyph));
            let downstream_glyph = containing
                .map(|cluster| cluster.last_glyph)
                .or_else(|| {
                    clusters
                        .get(next_cluster)
                        .map(|cluster| cluster.first_glyph)
                })
                .or_else(|| clusters.last().map(|cluster| cluster.last_glyph));
            let upstream = self.caret_segment(
                x,
                upstream_glyph.and_then(|index| self.glyphs.get(index)),
                shape_world,
            );
            let downstream = self.caret_segment(
                x,
                downstream_glyph.and_then(|index| self.glyphs.get(index)),
                shape_world,
            );

            if let Some(boundary) = boundaries.get_mut(char_index) {
                boundary.upstream.get_or_insert(upstream);
                boundary.downstream = Some(downstream);
            }
        }
    }

    fn positioned_clusters(
        &self,
        work: &mut StaticCaretBuildWork,
    ) -> Vec<StaticPositionedTextCluster> {
        let mut clusters = Vec::new();
        let mut glyph_index = 0;
        while let Some(first) = self.glyphs.get(glyph_index) {
            work.glyph_visits = work.glyph_visits.saturating_add(1);
            let char_start = first.glyph.char_index;
            let char_end = char_start.saturating_add(first.glyph.char_len);
            let first_glyph = glyph_index;
            let mut last_glyph = glyph_index;
            let mut end_x = first.x + first.glyph.advance;
            glyph_index = glyph_index.saturating_add(1);
            while let Some(next) = self.glyphs.get(glyph_index) {
                let next_end = next.glyph.char_index.saturating_add(next.glyph.char_len);
                if next.glyph.char_index != char_start || next_end != char_end {
                    break;
                }
                work.glyph_visits = work.glyph_visits.saturating_add(1);
                last_glyph = glyph_index;
                // A cluster's caret follows its final logical cursor. Its
                // visual extrema are retained separately by selection bounds.
                end_x = next.x + next.glyph.advance;
                glyph_index = glyph_index.saturating_add(1);
            }
            clusters.push(StaticPositionedTextCluster {
                char_start,
                char_end,
                start_x: first.x,
                end_x,
                first_glyph,
                last_glyph,
                rtl: first.glyph.rtl,
            });
        }
        clusters
    }

    fn caret_segment(
        &self,
        x: f32,
        glyph: Option<&StaticPositionedTextGlyph>,
        shape_world: Mat2D,
    ) -> StaticCaretSegment {
        let map = |y| {
            let (x, y) = glyph
                .map(|glyph| glyph.modified_point(x, y, self.baseline))
                .unwrap_or((x, y));
            let (x, y) = shape_world.transform_point(x, y);
            RenderVec2D::new(x, y)
        };
        StaticCaretSegment {
            top: map(self.top),
            bottom: map(self.bottom),
        }
    }

    fn caret_points(
        &self,
        char_index: usize,
        shape_world: Mat2D,
        affinity: StaticCaretAffinity,
    ) -> (RenderVec2D, RenderVec2D) {
        let x = self.caret_x(char_index);
        let glyph = self.caret_glyph(char_index, affinity);
        let segment = self.caret_segment(x, glyph, shape_world);
        (segment.top, segment.bottom)
    }

    fn selection_rect(
        &self,
        selected_start: usize,
        selected_end: usize,
        shape_world: Mat2D,
        has_geometric_modifiers: bool,
        has_non_monotone_advances: bool,
    ) -> Option<RenderAabb> {
        // A selected visual segment starts downstream and ends upstream.
        // Combining clusters and ligatures still remain indivisible because
        // both affinities snap internal source boundaries to the cluster end.
        let start_x = self.caret_x(selected_start);
        let end_x = self.caret_x(selected_end);
        if !has_geometric_modifiers && !has_non_monotone_advances {
            return (start_x != end_x).then(|| {
                transformed_text_rect(
                    shape_world,
                    start_x.min(end_x),
                    self.top,
                    start_x.max(end_x),
                    self.bottom,
                )
            });
        }

        let start_caret =
            self.caret_points(selected_start, shape_world, StaticCaretAffinity::Downstream);
        let end_caret = self.caret_points(selected_end, shape_world, StaticCaretAffinity::Upstream);
        let mut bounds = None;
        let mut has_selected_glyph_extent = false;
        extend_text_bounds(&mut bounds, start_caret.0);
        extend_text_bounds(&mut bounds, start_caret.1);
        extend_text_bounds(&mut bounds, end_caret.0);
        extend_text_bounds(&mut bounds, end_caret.1);
        for positioned in &self.glyphs {
            let glyph_start = positioned.glyph.char_index;
            if glyph_start < selected_start || glyph_start >= selected_end {
                continue;
            }
            let [top_left, top_right, bottom_right, bottom_left] = [
                (positioned.x, self.top),
                (positioned.x + positioned.glyph.advance, self.top),
                (positioned.x + positioned.glyph.advance, self.bottom),
                (positioned.x, self.bottom),
            ]
            .map(|(x, y)| {
                let (x, y) = positioned.modified_point(x, y, self.baseline);
                let (x, y) = shape_world.transform_point(x, y);
                RenderVec2D::new(x, y)
            });
            has_selected_glyph_extent |= top_left != top_right || bottom_right != bottom_left;
            for point in [top_left, top_right, bottom_right, bottom_left] {
                extend_text_bounds(&mut bounds, point);
            }
        }
        (start_caret != end_caret || has_selected_glyph_extent)
            .then_some(bounds)
            .flatten()
    }

    fn caret_glyph(
        &self,
        char_index: usize,
        affinity: StaticCaretAffinity,
    ) -> Option<&StaticPositionedTextGlyph> {
        if let Some(containing) = self.glyphs.iter().find(|positioned| {
            let start = positioned.glyph.char_index;
            let end = start.saturating_add(positioned.glyph.char_len);
            start < char_index && char_index < end
        }) {
            let start = containing.glyph.char_index;
            let end = start.saturating_add(containing.glyph.char_len);
            return self.glyphs.iter().rev().find(|positioned| {
                positioned.glyph.char_index == start
                    && positioned
                        .glyph
                        .char_index
                        .saturating_add(positioned.glyph.char_len)
                        == end
            });
        }
        match affinity {
            StaticCaretAffinity::Upstream => self
                .glyphs
                .iter()
                .rev()
                .find(|positioned| {
                    positioned
                        .glyph
                        .char_index
                        .saturating_add(positioned.glyph.char_len)
                        <= char_index
                })
                .or_else(|| self.glyphs.first()),
            StaticCaretAffinity::Downstream => self
                .glyphs
                .iter()
                .find(|positioned| positioned.glyph.char_index >= char_index)
                .or_else(|| self.glyphs.last()),
        }
    }

    fn caret_x(&self, char_index: usize) -> f32 {
        if char_index <= self.char_start {
            return self.start_x;
        }
        let mut glyphs = self.glyphs.iter().peekable();
        while let Some(positioned) = glyphs.next() {
            let glyph_start = positioned.glyph.char_index;
            let glyph_end = glyph_start.saturating_add(positioned.glyph.char_len);
            let cluster_x = positioned.x;
            let mut cluster_end_x = positioned.x + positioned.glyph.advance;
            while glyphs.peek().is_some_and(|next| {
                next.glyph.char_index == glyph_start
                    && next.glyph.char_index.saturating_add(next.glyph.char_len) == glyph_end
            }) {
                if let Some(next) = glyphs.next() {
                    // Preserve the final logical cursor even when glyphs in
                    // one cluster backtrack. Selection tracks visual bounds.
                    cluster_end_x = next.x + next.glyph.advance;
                }
            }
            if char_index < glyph_start {
                return cluster_x;
            }
            if char_index <= glyph_end && glyph_start < glyph_end {
                return if char_index == glyph_start {
                    cluster_x
                } else {
                    cluster_end_x
                };
            }
        }
        self.end_x
    }

    fn hit_char_index(&self, x: f32) -> usize {
        if x <= self.start_x {
            return self.char_start;
        }
        let mut glyphs = self.glyphs.iter().peekable();
        while let Some(positioned) = glyphs.next() {
            let glyph_start = positioned.glyph.char_index;
            let glyph_end = glyph_start.saturating_add(positioned.glyph.char_len);
            let cluster_x = positioned.x;
            let mut cluster_end_x = positioned.x + positioned.glyph.advance;
            while glyphs.peek().is_some_and(|next| {
                next.glyph.char_index == glyph_start
                    && next.glyph.char_index.saturating_add(next.glyph.char_len) == glyph_end
            }) {
                if let Some(next) = glyphs.next() {
                    cluster_end_x = cluster_end_x.max(next.x + next.glyph.advance);
                }
            }
            if x <= cluster_end_x {
                let midpoint = cluster_x + (cluster_end_x - cluster_x) / 2.0;
                return if x < midpoint { glyph_start } else { glyph_end };
            }
        }
        self.char_end
    }
}
impl StaticPositionedTextGlyph {
    fn modified_point(&self, x: f32, y: f32, baseline: f32) -> (f32, f32) {
        let center_x = self.x + self.glyph.advance * 0.5;
        let (x, y) = self
            .modifier_transform
            .transform_point(x - center_x, y - baseline);
        (center_x + x, baseline + y)
    }
}
