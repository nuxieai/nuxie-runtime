use crate::mechanical_port::source::layout::layout_style_applier::{
    RiveTrackLength, YGJustify, rive_grid_line, rive_grid_span, rive_justify_items,
    rive_justify_self, rive_template, rive_track,
};
use crate::mechanical_port::source::{
    container_component::ContainerComponent, generated::layout::grid_track_base::GridTrackBase,
};
use taffy::prelude::{GridPlacement, Style, TrackSizingFunction};

#[repr(transparent)]
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct GridTrackCollection(u8);
#[allow(non_upper_case_globals)]
impl GridTrackCollection {
    pub const TemplateColumns: Self = Self(0);
    pub const TemplateRows: Self = Self(1);
    pub const AutoColumns: Self = Self(2);
    pub const AutoRows: Self = Self(3);
}
impl From<u8> for GridTrackCollection {
    fn from(value: u8) -> Self {
        Self(value)
    }
}
#[repr(transparent)]
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct GridTrackSizeType(u8);
#[allow(non_upper_case_globals)]
impl GridTrackSizeType {
    pub const AutoSize: Self = Self(0);
    pub const Points: Self = Self(1);
    pub const Percent: Self = Self(2);
    pub const Fr: Self = Self(3);
}
impl From<u8> for GridTrackSizeType {
    fn from(value: u8) -> Self {
        Self(value)
    }
}

impl std::ops::Deref for GridTrack {
    type Target = GridTrackBase;
    fn deref(&self) -> &Self::Target {
        &self.base
    }
}

impl std::ops::DerefMut for GridTrack {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.base
    }
}

impl GridTrack {
    pub const TYPE_KEY: u16 = GridTrackBase::TYPE_KEY;
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
            if parent.is_type_of(
                crate::mechanical_port::source::generated::layout_component_base::LayoutComponentBase::TYPE_KEY,
            ) {
                crate::mechanical_port::source::layout_component::LayoutComponent::mark_layout_node_dirty_occurrence(&parent, false);
            }
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

    fn grid_size_length(kind: GridTrackSizeType, value: f32) -> RiveTrackLength {
        match kind {
            GridTrackSizeType::Points => RiveTrackLength::Points(value),
            GridTrackSizeType::Percent => RiveTrackLength::Percent(value),
            GridTrackSizeType::Fr => RiveTrackLength::Fr(value),
            _ => RiveTrackLength::Auto,
        }
    }
    fn grid_track_size(track: &GridTrack) -> TrackSizingFunction {
        let kind = GridTrackSizeType::from(track.base.track_type());
        if track.base.track_max_type() != 0 {
            return rive_track(
                Self::grid_size_length(kind, track.base.track_value()),
                Self::grid_size_length(
                    GridTrackSizeType::from(track.base.track_max_type() - 1),
                    track.base.track_max_value(),
                ),
            );
        }
        let value = track.base.track_value();
        match kind {
            GridTrackSizeType::Points => rive_track(
                RiveTrackLength::Points(value),
                RiveTrackLength::Points(value),
            ),
            GridTrackSizeType::Percent => rive_track(
                RiveTrackLength::Percent(value),
                RiveTrackLength::Percent(value),
            ),
            GridTrackSizeType::Fr => rive_track(RiveTrackLength::Auto, RiveTrackLength::Fr(value)),
            _ => rive_track(RiveTrackLength::Auto, RiveTrackLength::Auto),
        }
    }
    fn grid_line(cell: i32) -> GridPlacement {
        if cell == 0 {
            GridPlacement::Auto
        } else {
            rive_grid_line(if cell < 0 { cell - 1 } else { cell })
        }
    }
    fn grid_span(span: u32) -> GridPlacement {
        if span > 1 {
            rive_grid_span(span)
        } else {
            GridPlacement::Auto
        }
    }

    pub fn sync_container_style(style: &mut Style, owner: &ContainerComponent, justify_items: u32) {
        let mut lists: [Vec<TrackSizingFunction>; 4] = Default::default();
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
        style.grid_template_columns = rive_template(&lists[0]);
        style.grid_template_rows = rive_template(&lists[1]);
        style.grid_auto_columns = std::mem::take(&mut lists[2]);
        style.grid_auto_rows = std::mem::take(&mut lists[3]);
        style.justify_items = rive_justify_items(YGJustify::from(justify_items));
    }
    pub fn sync_stack_container_style(style: &mut Style, justify_items: u32) {
        let fill = rive_track(RiveTrackLength::Auto, RiveTrackLength::Fr(1.0));
        style.grid_template_columns = rive_template(&[fill]);
        style.grid_template_rows = rive_template(&[fill]);
        style.grid_auto_columns = Vec::new();
        style.grid_auto_rows = Vec::new();
        style.justify_items = rive_justify_items(YGJustify::from(justify_items));
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
        style: &mut Style,
        column: i32,
        row: i32,
        column_span: u32,
        row_span: u32,
    ) {
        style.grid_column.start = Self::grid_line(column);
        style.grid_column.end = Self::grid_span(column_span);
        style.grid_row.start = Self::grid_line(row);
        style.grid_row.end = Self::grid_span(row_span);
    }
    pub fn sync_stack_item_cell(style: &mut Style) {
        style.grid_column.start = rive_grid_line(1);
        style.grid_column.end = GridPlacement::Auto;
        style.grid_row.start = rive_grid_line(1);
        style.grid_row.end = GridPlacement::Auto;
    }
    pub fn sync_item_justify_self(
        style: &mut Style,
        value: u32,
        stack: bool,
        inline_hugs: bool,
        container: u32,
    ) {
        style.justify_self = rive_justify_self(if stack {
            YGJustify::from(value)
        } else {
            Self::resolve_item_justify_self(value, inline_hugs, container)
        });
    }
}
