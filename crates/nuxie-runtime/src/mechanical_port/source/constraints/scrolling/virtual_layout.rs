//! Upstream constraints/scrolling/virtual_layout.hpp and virtual_layout.cpp.
use super::super::super::layout::layout_enums::{LayoutCrossAlign, LayoutMainDistribute};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct VirtualWindow {
    pub start: i32,
    pub end: i32,
    pub visible_start: i32,
    pub visible_end: i32,
}
impl Default for VirtualWindow {
    fn default() -> Self {
        Self {
            start: 0,
            end: -1,
            visible_start: 0,
            visible_end: -1,
        }
    }
}
impl VirtualWindow {
    pub fn is_empty(&self) -> bool {
        self.end < self.start
    }
    pub fn is_visible(&self, line: i32) -> bool {
        line >= self.visible_start && line <= self.visible_end
    }
}
#[repr(u8)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum VirtualTrackSizing {
    #[default]
    AutoSize,
    Points,
    Percent,
    Fr,
}
#[derive(Clone, Copy, Debug, Default)]
pub struct VirtualGridTrack {
    pub min_sizing: VirtualTrackSizing,
    pub min_value: f32,
    pub max_sizing: VirtualTrackSizing,
    pub max_value: f32,
}
impl VirtualGridTrack {
    pub fn points(value: f32) -> Self {
        Self {
            min_sizing: VirtualTrackSizing::Points,
            min_value: value,
            max_sizing: VirtualTrackSizing::Points,
            max_value: value,
        }
    }
    pub fn auto_size() -> Self {
        Self::default()
    }
    pub fn percent(value: f32) -> Self {
        Self {
            min_sizing: VirtualTrackSizing::Percent,
            min_value: value,
            max_sizing: VirtualTrackSizing::Percent,
            max_value: value,
        }
    }
    pub fn fr(value: f32) -> Self {
        Self {
            max_sizing: VirtualTrackSizing::Fr,
            max_value: value,
            ..Self::default()
        }
    }
}
#[derive(Clone, Debug)]
pub struct VirtualGridAxis {
    pub templates: Vec<VirtualGridTrack>,
    pub autos: Vec<VirtualGridTrack>,
    pub space: f32,
    pub resize: bool,
}
impl Default for VirtualGridAxis {
    fn default() -> Self {
        Self {
            templates: Vec::new(),
            autos: Vec::new(),
            space: -1.0,
            resize: false,
        }
    }
}
#[derive(Clone, Debug)]
pub struct VirtualLayout {
    item_count: i32,
    line_count: i32,
    gap: f32,
    running: f32,
    trailing: f32,
    line_open: bool,
    line_flow_used: f32,
    segment_start: Vec<i32>,
    wraps: bool,
    hugs_lines: bool,
    flow_from_layout: bool,
    line_used: Vec<f32>,
    flow_gap: f32,
    flow_start: f32,
    flow_extent: f32,
    justify: LayoutMainDistribute,
    align: LayoutCrossAlign,
    grid: bool,
    column_count: i32,
    column_gap: f32,
    column_starts: Vec<f32>,
    column_widths: Vec<f32>,
    column_sizes: Vec<f32>,
    row_sizes: Vec<f32>,
    rows: VirtualGridAxis,
    columns: VirtualGridAxis,
    column_track_sizes: Vec<f32>,
    natural_extent: f32,
    item_extent: Vec<f32>,
    item_flow_extent: Vec<f32>,
    item_flow_offset: Vec<f32>,
    item_line_offset: Vec<f32>,
    line_start: Vec<f32>,
    line_extent: Vec<f32>,
    trailing_max: Vec<f32>,
    start_suffix_min: Vec<f32>,
    line_first_item: Vec<i32>,
}
impl Default for VirtualLayout {
    fn default() -> Self {
        Self {
            item_count: 0,
            line_count: 0,
            gap: 0.0,
            running: 0.0,
            trailing: 0.0,
            line_open: false,
            line_flow_used: 0.0,
            segment_start: Vec::new(),
            wraps: false,
            hugs_lines: false,
            flow_from_layout: true,
            line_used: Vec::new(),
            flow_gap: 0.0,
            flow_start: 0.0,
            flow_extent: 0.0,
            justify: LayoutMainDistribute::Start,
            align: LayoutCrossAlign::Start,
            grid: false,
            column_count: 1,
            column_gap: 0.0,
            column_starts: Vec::new(),
            column_widths: Vec::new(),
            column_sizes: Vec::new(),
            row_sizes: Vec::new(),
            rows: VirtualGridAxis::default(),
            columns: VirtualGridAxis::default(),
            column_track_sizes: Vec::new(),
            natural_extent: -1.0,
            item_extent: Vec::new(),
            item_flow_extent: Vec::new(),
            item_flow_offset: Vec::new(),
            item_line_offset: Vec::new(),
            line_start: Vec::new(),
            line_extent: Vec::new(),
            trailing_max: Vec::new(),
            start_suffix_min: Vec::new(),
            line_first_item: vec![0],
        }
    }
}
// std::min/max retain the first operand for unordered floating-point pairs.
fn min(a: f32, b: f32) -> f32 {
    if b < a { b } else { a }
}
fn max(a: f32, b: f32) -> f32 {
    if a < b { b } else { a }
}
#[cfg(test)]
#[path = "scroll_window_sweep_test.rs"]
mod scroll_window_sweep_test;
#[cfg(test)]
#[path = "virtual_layout_grid_test.rs"]
mod virtual_layout_grid_test;
#[cfg(test)]
#[path = "virtual_layout_wrap_test.rs"]
mod virtual_layout_wrap_test;
impl VirtualLayout {
    pub fn item_count(&self) -> i32 {
        self.item_count
    }
    pub fn line_count(&self) -> i32 {
        self.line_count
    }
    pub fn gap(&self) -> f32 {
        self.gap
    }
    pub fn extent(&self) -> f32 {
        if self.natural_extent >= 0.0 {
            self.natural_extent
        } else if self.line_count == 0 {
            0.0
        } else {
            let last = self.line_count as usize - 1;
            self.line_start[last] + self.line_extent[last]
        }
    }
    pub fn cycle_extent(&self) -> f32 {
        self.extent() + self.gap
    }
    pub fn begin_linear(&mut self, gap: f32) {
        self.item_count = 0;
        self.line_count = 0;
        self.natural_extent = -1.0;
        self.gap = gap;
        self.running = 0.0;
        self.trailing = f32::NEG_INFINITY;
        self.line_open = false;
        self.wraps = false;
        self.grid = false;
        self.hugs_lines = false;
        self.flow_from_layout = true;
        self.line_used.clear();
        self.line_start.clear();
        self.line_extent.clear();
        self.trailing_max.clear();
        self.line_first_item.clear();
        self.segment_start.clear();
        self.item_extent.clear();
        self.item_flow_extent.clear();
        self.item_flow_offset.clear();
        self.item_line_offset.clear();
    }
    pub fn begin_wrap(
        &mut self,
        line_gap: f32,
        flow_gap: f32,
        flow_start: f32,
        flow_extent: f32,
        justify: LayoutMainDistribute,
        align: LayoutCrossAlign,
        hugs_lines: bool,
    ) {
        self.begin_linear(line_gap);
        self.wraps = true;
        self.hugs_lines = hugs_lines;
        self.flow_gap = flow_gap;
        self.flow_start = flow_start;
        self.flow_extent = flow_extent;
        self.justify = justify;
        self.align = align;
    }
    pub fn begin_grid(
        &mut self,
        row_gap: f32,
        column_count: i32,
        align: LayoutCrossAlign,
        column_gap: f32,
    ) {
        self.column_gap = column_gap;
        self.begin_linear(row_gap);
        self.wraps = true;
        self.grid = true;
        self.column_count = 1.max(column_count);
        self.column_widths.clear();
        self.column_widths.resize(self.column_count as usize, 0.0);
        self.column_starts.clear();
        for axis in [&mut self.rows, &mut self.columns] {
            axis.templates.clear();
            axis.autos.clear();
            axis.space = -1.0;
            axis.resize = false;
        }
        self.column_track_sizes.clear();
        self.align = align;
    }
    pub fn grid_column_starts(&mut self) -> &mut Vec<f32> {
        &mut self.column_starts
    }
    pub fn grid_rows(&mut self) -> &mut VirtualGridAxis {
        &mut self.rows
    }
    pub fn grid_columns(&mut self) -> &mut VirtualGridAxis {
        &mut self.columns
    }
    pub fn grid_column_sizes(&mut self) -> &mut Vec<f32> {
        &mut self.column_sizes
    }
    pub fn grid_row_sizes(&mut self) -> &mut Vec<f32> {
        &mut self.row_sizes
    }
    pub fn wraps(&self) -> bool {
        self.wraps
    }
    pub fn flow_from_layout(&self) -> bool {
        self.flow_from_layout
    }
    pub fn set_flow_from_layout(&mut self, value: bool) {
        self.flow_from_layout = value;
    }
    pub fn is_grid(&self) -> bool {
        self.grid
    }
    pub fn column_count(&self) -> i32 {
        if self.grid { self.column_count } else { 1 }
    }
    pub fn grid_column_size(&self, column: i32) -> f32 {
        if column < self.column_track_sizes.len() as i32 {
            self.column_track_sizes[column as usize]
        } else {
            0.0
        }
    }
    fn track_at(axis: &VirtualGridAxis, index: usize) -> VirtualGridTrack {
        if index < axis.templates.len() {
            axis.templates[index]
        } else if axis.autos.is_empty() {
            VirtualGridTrack::default()
        } else {
            axis.autos[(index - axis.templates.len()) % axis.autos.len()]
        }
    }
    fn size_axis(axis: &VirtualGridAxis, content: &[f32], gap: f32, sizes: &mut Vec<f32>) -> f32 {
        Self::size_tracks(content, axis.space, axis, gap, sizes);
        let n = content.len();
        let mut sizes_again = axis.space < 0.0 && axis.resize;
        let mut index = 0;
        while index < n && axis.space < 0.0 && !sizes_again {
            let track = Self::track_at(axis, index);
            sizes_again = track.min_sizing == VirtualTrackSizing::Percent
                || track.max_sizing == VirtualTrackSizing::Percent;
            index += 1;
        }
        if !sizes_again {
            return -1.0;
        }
        let mut natural = if n > 1 { gap * (n - 1) as f32 } else { 0.0 };
        for &size in sizes.iter() {
            natural += size;
        }
        Self::size_tracks(content, natural, axis, gap, sizes);
        natural
    }
    fn size_grid_rows(&mut self) {
        while self.line_count < self.rows.templates.len() as i32 {
            self.push_line(0.0, 0.0);
        }
        let n = self.line_count as usize;
        let content = self.line_extent[..n].to_vec();
        let mut sizes = Vec::new();
        let natural = Self::size_axis(&self.rows, &content, self.gap, &mut sizes);
        if natural >= 0.0 {
            self.natural_extent = natural;
        }
        self.running = 0.0;
        self.trailing = f32::NEG_INFINITY;
        for line in 0..n {
            let extent = sizes[line];
            self.line_start[line] = self.running;
            self.line_extent[line] = extent;
            let last = if line + 1 < n {
                self.line_first_item[line + 1]
            } else {
                self.item_count
            };
            for item in self.line_first_item[line] as usize..last as usize {
                let slack = extent - self.item_extent[item];
                self.item_line_offset[item] = if self.align == LayoutCrossAlign::Center {
                    slack / 2.0
                } else {
                    0.0
                };
            }
            self.trailing = max(self.trailing, self.running + extent);
            self.trailing_max[line] = self.trailing;
            self.running += extent + self.gap;
        }
    }
    fn size_tracks(
        content: &[f32],
        space: f32,
        axis: &VirtualGridAxis,
        gap: f32,
        size: &mut Vec<f32>,
    ) {
        let n = content.len();
        size.clear();
        size.resize(n, 0.0);
        let mut limit = vec![0.0; n];
        let mut flex = vec![0.0; n];
        let resolve = |sizing: VirtualTrackSizing, value: f32, auto_size: f32| match sizing {
            VirtualTrackSizing::Points => value,
            VirtualTrackSizing::Percent if space >= 0.0 => value / 100.0 * space,
            _ => auto_size,
        };
        let mut any_flex = false;
        for row in 0..n {
            let track = Self::track_at(axis, row);
            size[row] = resolve(track.min_sizing, track.min_value, content[row]);
            if track.max_sizing == VirtualTrackSizing::Fr {
                flex[row] = track.max_value;
                limit[row] = size[row];
                any_flex = any_flex || flex[row] > 0.0;
            } else {
                limit[row] = max(
                    size[row],
                    resolve(track.max_sizing, track.max_value, content[row]),
                );
            }
        }
        let gaps = if n > 1 { gap * (n - 1) as f32 } else { 0.0 };
        if space >= 0.0 {
            let mut room = space - gaps;
            for &s in size.iter() {
                room -= s;
            }
            while room > 1e-6 {
                let mut growing = 0;
                for row in 0..n {
                    if limit[row] > size[row] {
                        growing += 1;
                    }
                }
                if growing == 0 {
                    break;
                }
                let share = room / growing as f32;
                for row in 0..n {
                    if limit[row] > size[row] {
                        let grow = min(share, limit[row] - size[row]);
                        size[row] += grow;
                        room -= grow;
                    }
                }
            }
        }
        if !any_flex {
            return;
        }
        let mut fr_size = 0.0;
        if space < 0.0 {
            for row in 0..n {
                if flex[row] > 0.0 {
                    let factor = max(flex[row], 1.0);
                    let items = content[row] / factor;
                    fr_size = max(
                        fr_size,
                        max(
                            size[row] / factor,
                            if flex[row] * items < size[row] {
                                content[row] - size[row]
                            } else {
                                items
                            },
                        ),
                    );
                }
            }
        } else {
            let mut fixed = vec![false; n];
            let mut settled = false;
            while !settled {
                let mut left = space - gaps;
                let mut factors = 0.0;
                for row in 0..n {
                    if flex[row] > 0.0 && !fixed[row] {
                        factors += flex[row];
                    } else {
                        left -= size[row];
                    }
                }
                fr_size = max(0.0, left) / max(factors, 1.0);
                settled = true;
                for row in 0..n {
                    if flex[row] > 0.0 && !fixed[row] && flex[row] * fr_size < size[row] {
                        fixed[row] = true;
                        settled = false;
                    }
                }
            }
        }
        for row in 0..n {
            if flex[row] > 0.0 {
                size[row] = max(size[row], flex[row] * fr_size);
            }
        }
    }
    pub fn begin_segment(&mut self) {
        self.segment_start.push(self.item_count);
    }
    fn push_line(&mut self, start: f32, extent: f32) {
        self.line_start.push(start);
        self.line_extent.push(extent);
        self.line_first_item.push(self.item_count);
        self.trailing_max.push(0.0);
        if self.wraps {
            self.line_used.push(0.0);
        }
        self.line_count += 1;
    }
    fn open_line(&mut self) {
        self.push_line(self.running, 0.0);
        self.line_open = true;
        self.line_flow_used = 0.0;
    }
    fn close_line(&mut self) {
        let line = self.line_count as usize - 1;
        let first = self.line_first_item[line] as usize;
        if self.grid {
            for item in first..self.item_count as usize {
                let column = item - first;
                self.column_widths[column] =
                    max(self.column_widths[column], self.item_flow_extent[item]);
                self.item_flow_offset[item] = if column < self.column_starts.len() {
                    self.column_starts[column]
                } else {
                    0.0
                };
            }
            self.line_open = false;
            return;
        }
        let line_extent = self.line_extent[line];
        self.line_used[line] = self.line_flow_used;
        for item in first..self.item_count as usize {
            let slack = line_extent - self.item_extent[item];
            self.item_line_offset[item] = match self.align {
                LayoutCrossAlign::Start => 0.0,
                LayoutCrossAlign::Center => slack / 2.0,
                LayoutCrossAlign::End => slack,
            };
        }
        if !self.hugs_lines {
            self.place_line(line, self.flow_extent);
        }
        self.trailing = max(self.trailing, self.line_start[line] + line_extent);
        self.trailing_max[line] = self.trailing;
        self.running = self.line_start[line] + line_extent + self.gap;
        self.line_open = false;
    }
    fn place_line(&mut self, line: usize, extent: f32) {
        let first = self.line_first_item[line];
        let last = if line + 1 < self.line_first_item.len() && line + 1 < self.line_count as usize {
            self.line_first_item[line + 1]
        } else {
            self.item_count
        };
        let count = last - first;
        let free_space = extent - self.line_used[line];
        let mut lead = 0.0;
        let mut between = self.flow_gap;
        match self.justify {
            LayoutMainDistribute::Start => (),
            LayoutMainDistribute::Center => lead = free_space / 2.0,
            LayoutMainDistribute::End => lead = free_space,
            LayoutMainDistribute::SpaceBetween => {
                if count > 1 {
                    between += max(free_space, 0.0) / (count - 1) as f32;
                }
            }
        }
        let mut flow = self.flow_start + lead;
        for item in first as usize..last as usize {
            self.item_flow_offset[item] = flow;
            flow += self.item_flow_extent[item] + between;
        }
    }
    pub fn add_item(&mut self, extent: f32, flow_extent: f32) {
        if !self.wraps {
            self.push_line(self.running, extent);
            self.trailing = max(self.trailing, self.running + extent);
            *self.trailing_max.last_mut().unwrap() = self.trailing;
            self.running += extent + self.gap;
            self.item_count += 1;
            return;
        }
        if self.line_open
            && if self.grid {
                self.item_count - self.line_first_item[self.line_count as usize - 1]
                    >= self.column_count
            } else {
                self.line_flow_used + self.flow_gap + flow_extent > self.flow_extent
            }
        {
            self.close_line();
        }
        if !self.line_open {
            self.open_line();
            self.line_flow_used = flow_extent;
        } else {
            self.line_flow_used += self.flow_gap + flow_extent;
        }
        let line = self.line_count as usize - 1;
        self.line_extent[line] = max(self.line_extent[line], extent);
        self.item_extent.push(extent);
        self.item_flow_extent.push(flow_extent);
        self.item_flow_offset.push(0.0);
        self.item_line_offset.push(0.0);
        self.item_count += 1;
    }
    pub fn end(&mut self) {
        if self.line_open {
            self.close_line();
        }
        if self.grid {
            Self::size_axis(
                &self.columns,
                &self.column_widths,
                self.column_gap,
                &mut self.column_track_sizes,
            );
            self.size_grid_rows();
        }
        self.line_first_item.push(self.item_count);
        self.start_suffix_min.resize(self.line_count as usize, 0.0);
        let mut lowest = f32::INFINITY;
        for line in (0..self.line_count as usize).rev() {
            lowest = min(lowest, self.line_start[line]);
            self.start_suffix_min[line] = lowest;
        }
        if self.wraps && !self.grid && self.hugs_lines {
            let mut widest = 0.0;
            for line in 0..self.line_count as usize {
                widest = max(widest, self.line_used[line]);
            }
            for line in 0..self.line_count as usize {
                self.place_line(line, widest);
            }
        }
    }
    pub fn item_flow_offset(&self, item: i32) -> f32 {
        if self.wraps {
            self.item_flow_offset[item as usize]
        } else {
            0.0
        }
    }
    pub fn item_line_offset(&self, item: i32) -> f32 {
        if self.wraps {
            self.item_line_offset[item as usize]
        } else {
            0.0
        }
    }
    pub fn build_linear(&mut self, count: i32, mut extent_at: impl FnMut(i32) -> f32, gap: f32) {
        self.begin_linear(gap);
        self.begin_segment();
        for i in 0..count {
            self.add_item(extent_at(i), 0.0);
        }
        self.end();
    }
    pub fn segment_count(&self) -> i32 {
        self.segment_start.len() as i32
    }
    pub fn segment_start(&self, segment: i32) -> i32 {
        self.segment_start[segment as usize]
    }
    pub fn segment_of(&self, item: i32) -> i32 {
        self.segment_start.partition_point(|&start| start <= item) as i32 - 1
    }
    fn wrap_index(index: i32, count: i32) -> i32 {
        let wrapped = index % count;
        if wrapped < 0 {
            wrapped + count
        } else {
            wrapped
        }
    }
    pub fn wrap_line(&self, line: i32) -> i32 {
        Self::wrap_index(line, self.line_count)
    }
    pub fn wrap_column(&self, column: i32) -> i32 {
        Self::wrap_index(column, self.column_count())
    }
    pub fn line_start(&self, line: i32) -> f32 {
        let wrapped = self.wrap_line(line);
        let cycle = (line - wrapped) / self.line_count;
        cycle as f32 * self.cycle_extent() + self.line_start[wrapped as usize]
    }
    pub fn line_extent(&self, line: i32) -> f32 {
        self.line_extent[self.wrap_line(line) as usize]
    }
    pub fn line_first_item(&self, line: i32) -> i32 {
        self.line_first_item[self.wrap_line(line) as usize]
    }
    pub fn line_last_item(&self, line: i32) -> i32 {
        self.line_first_item[self.wrap_line(line) as usize + 1] - 1
    }
    pub fn line_of_item(&self, item: i32) -> i32 {
        let mut lo = 0;
        let mut hi = self.line_count - 1;
        while lo < hi {
            let mid = (lo + hi + 1) >> 1;
            if self.line_first_item[mid as usize] <= item {
                lo = mid;
            } else {
                hi = mid - 1;
            }
        }
        lo
    }
    fn last_start_before(&self, value: f32) -> i32 {
        let mut lo = 0;
        let mut hi = self.line_count;
        while lo < hi {
            let mid = (lo + hi) >> 1;
            if self.start_suffix_min[mid as usize] >= value {
                hi = mid;
            } else {
                lo = mid + 1;
            }
        }
        lo - 1
    }
    fn first_trailing_past(&self, value: f32, inclusive: bool) -> i32 {
        let mut lo = 0;
        let mut hi = self.line_count;
        while lo < hi {
            let mid = (lo + hi) >> 1;
            let edge = self.trailing_max[mid as usize];
            if edge > value || (inclusive && edge == value) {
                hi = mid;
            } else {
                lo = mid + 1;
            }
        }
        lo
    }
    pub fn window(&self, offset: f32, viewport: f32, infinite: bool, buffer: i32) -> VirtualWindow {
        let n = self.line_count;
        if n == 0 {
            return VirtualWindow::default();
        }
        let mut start = self.first_trailing_past(offset, false);
        if start == n {
            start = if infinite { n } else { n - 1 };
        }
        let limit = offset + viewport;
        let end = if !infinite {
            start.max(self.last_start_before(limit))
        } else {
            let mut latest = -1;
            let cycle = self.cycle_extent();
            for c in 0..=2 {
                let found = self.last_start_before(limit - c as f32 * cycle);
                if found >= 0 {
                    latest = c * n + found;
                }
            }
            start.max(latest).min(start + n - 1)
        };
        let mut window = VirtualWindow {
            visible_start: start,
            visible_end: end,
            ..Default::default()
        };
        Self::buffer_window(&mut window, n, buffer, infinite);
        window
    }
    fn buffer_window(window: &mut VirtualWindow, count: i32, buffer: i32, cycles: bool) {
        let start = window.visible_start;
        let end = window.visible_end;
        window.start = start;
        window.end = end;
        if buffer <= 0 {
            return;
        }
        let extra = 0.max(count - (end - start + 1));
        let before = 0.max(buffer.min(if cycles { extra } else { start }));
        let after = 0.max(buffer.min(if cycles {
            extra - before
        } else {
            count - 1 - end
        }));
        window.start = start - before;
        window.end = end + after;
    }
    pub fn column_cycle_extent(&self) -> f32 {
        let count = self.column_count() as usize;
        if !self.grid || self.column_starts.len() < count + 1 {
            return 0.0;
        }
        self.column_starts[count] - self.column_starts[0] + self.column_gap
    }
    pub fn column_start(&self, column: i32) -> f32 {
        let wrapped = self.wrap_column(column);
        if wrapped >= self.column_starts.len() as i32 {
            return 0.0;
        }
        let cycle = (column - wrapped) / self.column_count();
        cycle as f32 * self.column_cycle_extent() + self.column_starts[wrapped as usize]
    }
    pub fn column_window(
        &self,
        offset: f32,
        viewport: f32,
        buffer: i32,
        infinite: bool,
    ) -> VirtualWindow {
        let count = self.column_count();
        let mut window = VirtualWindow {
            start: 0,
            visible_start: 0,
            end: count - 1,
            visible_end: count - 1,
        };
        if !self.grid || (self.column_starts.len() as i32) < count + 1 {
            return window;
        }
        let column_end = |column: i32| {
            if column < count - 1 {
                self.column_starts[column as usize + 1] - self.column_gap
            } else {
                self.column_starts[count as usize]
            }
        };
        let cycle = if infinite {
            self.column_cycle_extent()
        } else {
            0.0
        };
        if cycle <= 0.0 {
            let mut start = 0;
            while start < count - 1 && column_end(start) <= offset {
                start += 1;
            }
            let mut end = start;
            while end < count - 1 && self.column_starts[end as usize + 1] < offset + viewport {
                end += 1;
            }
            window.visible_start = start;
            window.visible_end = end;
            Self::buffer_window(&mut window, count, buffer, false);
            return window;
        }
        let k = ((offset - self.column_starts[0]) / cycle).floor() as i32;
        let local = offset - k as f32 * cycle;
        let mut start = 0;
        while start < count && column_end(start) <= local {
            start += 1;
        }
        start += k * count;
        let mut end = start;
        while end < start + count - 1 && self.column_start(end + 1) < offset + viewport {
            end += 1;
        }
        window.visible_start = start;
        window.visible_end = end;
        Self::buffer_window(&mut window, count, buffer, true);
        window
    }
}
