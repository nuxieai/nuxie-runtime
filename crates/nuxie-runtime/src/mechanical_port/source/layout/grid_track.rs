use crate::mechanical_port::source::layout::layout_style_applier::{
    GridTrackList, YGGridLine, YGGridTrackSize, YGJustify, YGStyle, YGStyleSizeLength,
};
use crate::mechanical_port::source::{
    component::ContainerComponent, generated::layout::grid_track_base::GridTrackBase,
};

#[repr(u8)]
pub enum GridTrackCollection {
    TemplateColumns,
    TemplateRows,
    AutoColumns,
    AutoRows,
}
#[repr(u8)]
#[derive(Clone, Copy)]
pub enum GridTrackSizeType {
    AutoSize,
    Points,
    Percent,
    Fr,
}

#[derive(Default)]
pub struct GridTrack {
    pub base: GridTrackBase,
}
impl GridTrack {
    pub fn grid_collection(&self) -> GridTrackCollection {
        GridTrackCollection::from(self.base.collection())
    }
    fn mark_layout_dirty(&mut self) {
        if let Some(parent) = self.base.parent_handle() {
            parent.with_mut(|parent| {
                if let Some(layout) = parent.as_layout_component_mut() {
                    layout.mark_layout_node_dirty(false);
                }
            });
        }
    }
    pub fn collection_changed(&mut self) {
        self.mark_layout_dirty();
    }
    pub fn track_type_changed(&mut self) {
        self.mark_layout_dirty();
    }
    pub fn track_value_changed(&mut self) {
        self.mark_layout_dirty();
    }
    pub fn track_max_type_changed(&mut self) {
        self.mark_layout_dirty();
    }
    pub fn track_max_value_changed(&mut self) {
        self.mark_layout_dirty();
    }

    fn grid_size_length(kind: GridTrackSizeType, value: f32) -> YGStyleSizeLength {
        match kind {
            GridTrackSizeType::Points => YGStyleSizeLength::points(value),
            GridTrackSizeType::Percent => YGStyleSizeLength::percent(value),
            GridTrackSizeType::Fr => YGStyleSizeLength::stretch(value),
            GridTrackSizeType::AutoSize => YGStyleSizeLength::auto(),
        }
    }
    fn grid_track_size(track: &GridTrack) -> YGGridTrackSize {
        let kind = GridTrackSizeType::from(track.base.track_type());
        if track.base.track_max_type() != 0 {
            return YGGridTrackSize::minmax(
                Self::grid_size_length(kind, track.base.track_value()),
                Self::grid_size_length(
                    GridTrackSizeType::from(track.base.track_max_type() - 1),
                    track.base.track_max_value(),
                ),
            );
        }
        match kind {
            GridTrackSizeType::Points => YGGridTrackSize::length(track.base.track_value()),
            GridTrackSizeType::Percent => YGGridTrackSize::percent(track.base.track_value()),
            GridTrackSizeType::Fr => YGGridTrackSize::fr(track.base.track_value()),
            GridTrackSizeType::AutoSize => YGGridTrackSize::auto(),
        }
    }
    fn grid_line(cell: i32) -> YGGridLine {
        if cell == 0 {
            YGGridLine::auto()
        } else {
            YGGridLine::from_integer(if cell < 0 { cell - 1 } else { cell })
        }
    }
    fn grid_span(span: u32) -> YGGridLine {
        if span > 1 {
            YGGridLine::span(span as i32)
        } else {
            YGGridLine::auto()
        }
    }

    pub fn sync_container_style(
        style: &mut YGStyle,
        owner: &ContainerComponent,
        justify_items: u32,
    ) {
        let mut lists: [GridTrackList; 4] = Default::default();
        for child in owner.children() {
            let Some((collection, track_size)) = child.with_downcast::<GridTrack, _>(|track| {
                (track.base.collection(), Self::grid_track_size(track))
            }) else {
                continue;
            };
            if collection > 3 {
                continue;
            }
            lists[collection as usize].push(track_size);
        }
        style.set_grid_template_columns(std::mem::take(&mut lists[0]));
        style.set_grid_template_rows(std::mem::take(&mut lists[1]));
        style.set_grid_auto_columns(std::mem::take(&mut lists[2]));
        style.set_grid_auto_rows(std::mem::take(&mut lists[3]));
        style.set_justify_items(YGJustify::from(justify_items));
    }
    pub fn sync_stack_container_style(style: &mut YGStyle, justify_items: u32) {
        style.set_grid_template_columns(vec![YGGridTrackSize::fr(1.0)]);
        style.set_grid_template_rows(vec![YGGridTrackSize::fr(1.0)]);
        style.set_grid_auto_columns(Vec::new());
        style.set_grid_auto_rows(Vec::new());
        style.set_justify_items(YGJustify::from(justify_items));
    }
    fn resolve_item_justify_self(value: u32, inline_hugs: bool, container: u32) -> YGJustify {
        if !inline_hugs {
            return YGJustify::from(value);
        }
        let effective = if value == YGJustify::Auto as u32 {
            container
        } else {
            value
        };
        if effective == YGJustify::Stretch as u32 {
            YGJustify::FlexStart
        } else {
            YGJustify::from(value)
        }
    }
    pub fn sync_item_lines(
        style: &mut YGStyle,
        column: i32,
        row: i32,
        column_span: u32,
        row_span: u32,
    ) {
        style.set_grid_column_start(Self::grid_line(column));
        style.set_grid_column_end(Self::grid_span(column_span));
        style.set_grid_row_start(Self::grid_line(row));
        style.set_grid_row_end(Self::grid_span(row_span));
    }
    pub fn sync_stack_item_cell(style: &mut YGStyle) {
        style.set_grid_column_start(YGGridLine::from_integer(1));
        style.set_grid_column_end(YGGridLine::auto());
        style.set_grid_row_start(YGGridLine::from_integer(1));
        style.set_grid_row_end(YGGridLine::auto());
    }
    pub fn sync_item_justify_self(
        style: &mut YGStyle,
        value: u32,
        stack: bool,
        inline_hugs: bool,
        container: u32,
    ) {
        if stack {
            style.set_justify_self(YGJustify::from(value));
            return;
        }
        style.set_justify_self(Self::resolve_item_justify_self(
            value,
            inline_hugs,
            container,
        ));
    }
}
