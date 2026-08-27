#[cfg(feature = "rive-layout")]
use crate::mechanical_port::source::yoga::{
    YGAlign, YGDirection, YGDisplay, YGEdge, YGFlexDirection, YGFloatOptional, YGGutter, YGJustify,
    YGOverflow, YGPositionType, YGStyle, YGUnit, YGValue, YGWrap,
};
use crate::mechanical_port::source::{
    animation::keyframe_interpolator::KeyFrameInterpolator,
    core_context::{CoreContext, StatusCode},
    generated::layout::layout_component_style_base::LayoutComponentStyleBase,
    layout::{
        grid_track::GridTrack,
        layout_enums::{
            LayoutAlignmentType, LayoutAnimationStyle, LayoutScaleType, LayoutStyleInterpolation,
        },
        layout_style_applier::LayoutSyncContext,
    },
    layout_component::LayoutComponent,
};

pub struct LayoutComponentStyle {
    pub base: LayoutComponentStyleBase,
    #[cfg(feature = "rive-layout")]
    interpolator: Option<*mut KeyFrameInterpolator>,
}

impl LayoutComponentStyle {
    #[cfg(feature = "rive-layout")]
    fn apply_stack_alignment(style: &mut YGStyle, alignment: LayoutAlignmentType) {
        style.set_justify_items(match alignment {
            LayoutAlignmentType::TopLeft
            | LayoutAlignmentType::CenterLeft
            | LayoutAlignmentType::BottomLeft
            | LayoutAlignmentType::SpaceBetweenStart => YGJustify::Start,
            LayoutAlignmentType::TopCenter
            | LayoutAlignmentType::Center
            | LayoutAlignmentType::BottomCenter
            | LayoutAlignmentType::SpaceBetweenCenter => YGJustify::Center,
            LayoutAlignmentType::TopRight
            | LayoutAlignmentType::CenterRight
            | LayoutAlignmentType::BottomRight
            | LayoutAlignmentType::SpaceBetweenEnd => YGJustify::End,
        });
        style.set_align_items(match alignment {
            LayoutAlignmentType::TopLeft
            | LayoutAlignmentType::TopCenter
            | LayoutAlignmentType::TopRight
            | LayoutAlignmentType::SpaceBetweenStart
            | LayoutAlignmentType::SpaceBetweenCenter
            | LayoutAlignmentType::SpaceBetweenEnd => YGAlign::Start,
            LayoutAlignmentType::CenterLeft
            | LayoutAlignmentType::Center
            | LayoutAlignmentType::CenterRight => YGAlign::Center,
            LayoutAlignmentType::BottomLeft
            | LayoutAlignmentType::BottomCenter
            | LayoutAlignmentType::BottomRight => YGAlign::End,
        });
    }

    #[cfg(feature = "rive-layout")]
    pub fn apply_item_style(&self, style: &mut YGStyle, context: &LayoutSyncContext) {
        self.base.apply_sizing_item_style(style, context);
        style.set_position_type(self.position_type());
        style.set_aspect_ratio(YGFloatOptional::new(if self.base.aspect_ratio() > 0.0 {
            self.base.aspect_ratio()
        } else {
            f32::NAN
        }));
        let start = if context.is_ltr {
            YGEdge::Left
        } else {
            YGEdge::Right
        };
        let end = if context.is_ltr {
            YGEdge::Right
        } else {
            YGEdge::Left
        };
        let margin_unit = |unit| {
            if context.has_layout_parent {
                unit
            } else {
                YGUnit::Point
            }
        };
        style.margin_mut()[start] = YGValue::new(
            self.base.margin_left(),
            margin_unit(self.margin_left_units()),
        );
        style.margin_mut()[end] = YGValue::new(
            self.base.margin_right(),
            margin_unit(self.margin_right_units()),
        );
        style.margin_mut()[YGEdge::Top] =
            YGValue::new(self.base.margin_top(), margin_unit(self.margin_top_units()));
        style.margin_mut()[YGEdge::Bottom] = YGValue::new(
            self.base.margin_bottom(),
            margin_unit(self.margin_bottom_units()),
        );
        style.position_mut()[start] =
            YGValue::new(self.base.position_left(), self.position_left_units());
        style.position_mut()[end] =
            YGValue::new(self.base.position_right(), self.position_right_units());
        style.position_mut()[YGEdge::Top] =
            YGValue::new(self.base.position_top(), self.position_top_units());
        style.position_mut()[YGEdge::Bottom] =
            YGValue::new(self.base.position_bottom(), self.position_bottom_units());
    }

    #[cfg(feature = "rive-layout")]
    pub fn apply_container_style(&self, style: &mut YGStyle, context: &LayoutSyncContext) {
        let start = if context.is_ltr {
            YGEdge::Left
        } else {
            YGEdge::Right
        };
        let end = if context.is_ltr {
            YGEdge::Right
        } else {
            YGEdge::Left
        };
        style.set_flex_direction(self.flex_direction());
        style.set_flex_wrap(self.flex_wrap());
        style.set_direction(self.direction());
        style.gap_mut()[YGGutter::Column] =
            YGValue::new(self.base.gap_horizontal(), self.gap_horizontal_units());
        style.gap_mut()[YGGutter::Row] =
            YGValue::new(self.base.gap_vertical(), self.gap_vertical_units());
        style.border_mut()[start] = YGValue::new(self.base.border_left(), self.border_left_units());
        style.border_mut()[end] = YGValue::new(self.base.border_right(), self.border_right_units());
        style.border_mut()[YGEdge::Top] =
            YGValue::new(self.base.border_top(), self.border_top_units());
        style.border_mut()[YGEdge::Bottom] =
            YGValue::new(self.base.border_bottom(), self.border_bottom_units());
        style.padding_mut()[start] =
            YGValue::new(self.base.padding_left(), self.padding_left_units());
        style.padding_mut()[end] =
            YGValue::new(self.base.padding_right(), self.padding_right_units());
        style.padding_mut()[YGEdge::Top] =
            YGValue::new(self.base.padding_top(), self.padding_top_units());
        style.padding_mut()[YGEdge::Bottom] =
            YGValue::new(self.base.padding_bottom(), self.padding_bottom_units());
        if self.is_stack() {
            GridTrack::sync_stack_container_style(style, self.base.justify_items_value());
            Self::apply_stack_alignment(style, self.alignment_type());
            return;
        }
        let row = matches!(
            self.flex_direction(),
            YGFlexDirection::Row | YGFlexDirection::RowReverse
        );
        match self.alignment_type() {
            LayoutAlignmentType::TopLeft
            | LayoutAlignmentType::TopCenter
            | LayoutAlignmentType::TopRight => {
                if row {
                    style.set_align_items(YGAlign::FlexStart);
                    style.set_align_content(YGAlign::FlexStart);
                } else {
                    style.set_justify_content(YGJustify::FlexStart);
                }
            }
            LayoutAlignmentType::CenterLeft
            | LayoutAlignmentType::Center
            | LayoutAlignmentType::CenterRight => {
                if row {
                    style.set_align_items(YGAlign::Center);
                    style.set_align_content(YGAlign::Center);
                } else {
                    style.set_justify_content(YGJustify::Center);
                }
            }
            LayoutAlignmentType::BottomLeft
            | LayoutAlignmentType::BottomCenter
            | LayoutAlignmentType::BottomRight => {
                if row {
                    style.set_align_items(YGAlign::FlexEnd);
                    style.set_align_content(YGAlign::FlexEnd);
                } else {
                    style.set_justify_content(YGJustify::FlexEnd);
                }
            }
            _ => {}
        }
        match self.alignment_type() {
            LayoutAlignmentType::TopLeft
            | LayoutAlignmentType::CenterLeft
            | LayoutAlignmentType::BottomLeft => {
                if row {
                    style.set_justify_content(YGJustify::FlexStart);
                } else {
                    style.set_align_items(YGAlign::FlexStart);
                    style.set_align_content(YGAlign::FlexStart);
                }
            }
            LayoutAlignmentType::TopCenter
            | LayoutAlignmentType::Center
            | LayoutAlignmentType::BottomCenter => {
                if row {
                    style.set_justify_content(YGJustify::Center);
                } else {
                    style.set_align_items(YGAlign::Center);
                    style.set_align_content(YGAlign::Center);
                }
            }
            LayoutAlignmentType::TopRight
            | LayoutAlignmentType::CenterRight
            | LayoutAlignmentType::BottomRight => {
                if row {
                    style.set_justify_content(YGJustify::FlexEnd);
                } else {
                    style.set_align_items(YGAlign::FlexEnd);
                    style.set_align_content(YGAlign::FlexEnd);
                }
            }
            LayoutAlignmentType::SpaceBetweenStart => {
                style.set_align_items(YGAlign::FlexStart);
                style.set_align_content(YGAlign::FlexStart);
                style.set_justify_content(YGJustify::SpaceBetween);
            }
            LayoutAlignmentType::SpaceBetweenCenter => {
                style.set_align_items(YGAlign::Center);
                style.set_align_content(YGAlign::Center);
                style.set_justify_content(YGJustify::SpaceBetween);
            }
            LayoutAlignmentType::SpaceBetweenEnd => {
                style.set_align_items(YGAlign::FlexEnd);
                style.set_align_content(YGAlign::FlexEnd);
                style.set_justify_content(YGJustify::SpaceBetween);
            }
        }
    }

    #[cfg(feature = "rive-layout")]
    pub fn interpolator(&mut self) -> Option<&mut KeyFrameInterpolator> {
        self.interpolator.map(|pointer| unsafe { &mut *pointer })
    }
    pub fn interpolation(&self) -> LayoutStyleInterpolation {
        LayoutStyleInterpolation::from(self.base.interpolation_type())
    }
    pub fn animation_style(&self) -> LayoutAnimationStyle {
        LayoutAnimationStyle::from(self.base.animation_style_type())
    }
    pub fn alignment_type(&self) -> LayoutAlignmentType {
        LayoutAlignmentType::from(self.base.layout_alignment_type())
    }
    pub fn width_scale_type(&self) -> LayoutScaleType {
        LayoutScaleType::from(self.base.layout_width_scale_type())
    }
    pub fn height_scale_type(&self) -> LayoutScaleType {
        LayoutScaleType::from(self.base.layout_height_scale_type())
    }
    pub fn is_stack(&self) -> bool {
        self.base.layout_type_value() == 2
    }
    pub fn is_grid(&self) -> bool {
        self.base.layout_type_value() != 0
    }

    #[cfg(feature = "rive-layout")]
    pub fn display(&self) -> YGDisplay {
        if YGDisplay::from(self.base.display_value()) == YGDisplay::None {
            YGDisplay::None
        } else if self.base.layout_type_value() == 0 {
            YGDisplay::Flex
        } else {
            YGDisplay::Grid
        }
    }
    #[cfg(feature = "rive-layout")]
    pub fn position_type(&self) -> YGPositionType {
        YGPositionType::from(self.base.position_type_value())
    }
    #[cfg(feature = "rive-layout")]
    pub fn flex_direction(&self) -> YGFlexDirection {
        YGFlexDirection::from(self.base.flex_direction_value())
    }
    #[cfg(feature = "rive-layout")]
    pub fn direction(&self) -> YGDirection {
        YGDirection::from(self.base.direction_value())
    }
    #[cfg(feature = "rive-layout")]
    pub fn flex_wrap(&self) -> YGWrap {
        YGWrap::from(self.base.flex_wrap_value())
    }
    #[cfg(feature = "rive-layout")]
    pub fn overflow(&self) -> YGOverflow {
        YGOverflow::from(self.base.overflow_value())
    }
    pub fn intrinsically_sized(&self) -> bool {
        self.base.intrinsically_sized_value() == 1
    }

    #[cfg(feature = "rive-layout")]
    pub fn width_units(&self) -> YGUnit {
        YGUnit::from(self.base.width_units_value())
    }
    #[cfg(feature = "rive-layout")]
    pub fn height_units(&self) -> YGUnit {
        YGUnit::from(self.base.height_units_value())
    }
    #[cfg(feature = "rive-layout")]
    pub fn border_left_units(&self) -> YGUnit {
        YGUnit::from(self.base.border_left_units_value())
    }
    #[cfg(feature = "rive-layout")]
    pub fn border_right_units(&self) -> YGUnit {
        YGUnit::from(self.base.border_right_units_value())
    }
    #[cfg(feature = "rive-layout")]
    pub fn border_top_units(&self) -> YGUnit {
        YGUnit::from(self.base.border_top_units_value())
    }
    #[cfg(feature = "rive-layout")]
    pub fn border_bottom_units(&self) -> YGUnit {
        YGUnit::from(self.base.border_bottom_units_value())
    }
    #[cfg(feature = "rive-layout")]
    pub fn margin_left_units(&self) -> YGUnit {
        YGUnit::from(self.base.margin_left_units_value())
    }
    #[cfg(feature = "rive-layout")]
    pub fn margin_right_units(&self) -> YGUnit {
        YGUnit::from(self.base.margin_right_units_value())
    }
    #[cfg(feature = "rive-layout")]
    pub fn margin_top_units(&self) -> YGUnit {
        YGUnit::from(self.base.margin_top_units_value())
    }
    #[cfg(feature = "rive-layout")]
    pub fn margin_bottom_units(&self) -> YGUnit {
        YGUnit::from(self.base.margin_bottom_units_value())
    }
    #[cfg(feature = "rive-layout")]
    pub fn padding_left_units(&self) -> YGUnit {
        YGUnit::from(self.base.padding_left_units_value())
    }
    #[cfg(feature = "rive-layout")]
    pub fn padding_right_units(&self) -> YGUnit {
        YGUnit::from(self.base.padding_right_units_value())
    }
    #[cfg(feature = "rive-layout")]
    pub fn padding_top_units(&self) -> YGUnit {
        YGUnit::from(self.base.padding_top_units_value())
    }
    #[cfg(feature = "rive-layout")]
    pub fn padding_bottom_units(&self) -> YGUnit {
        YGUnit::from(self.base.padding_bottom_units_value())
    }
    #[cfg(feature = "rive-layout")]
    pub fn position_left_units(&self) -> YGUnit {
        YGUnit::from(self.base.position_left_units_value())
    }
    #[cfg(feature = "rive-layout")]
    pub fn position_right_units(&self) -> YGUnit {
        YGUnit::from(self.base.position_right_units_value())
    }
    #[cfg(feature = "rive-layout")]
    pub fn position_top_units(&self) -> YGUnit {
        YGUnit::from(self.base.position_top_units_value())
    }
    #[cfg(feature = "rive-layout")]
    pub fn position_bottom_units(&self) -> YGUnit {
        YGUnit::from(self.base.position_bottom_units_value())
    }
    #[cfg(feature = "rive-layout")]
    pub fn gap_horizontal_units(&self) -> YGUnit {
        YGUnit::from(self.base.gap_horizontal_units_value())
    }
    #[cfg(feature = "rive-layout")]
    pub fn gap_vertical_units(&self) -> YGUnit {
        YGUnit::from(self.base.gap_vertical_units_value())
    }
    #[cfg(feature = "rive-layout")]
    pub fn flex_basis_units(&self) -> YGUnit {
        YGUnit::from(self.base.flex_basis_units_value())
    }

    pub fn mark_layout_node_dirty(&mut self) {
        #[cfg(feature = "rive-layout")]
        if let Some(layout) = self.base.parent_mut().as_mut::<LayoutComponent>() {
            layout.mark_layout_node_dirty(false);
        }
    }
    pub fn mark_layout_style_dirty(&mut self) {
        #[cfg(feature = "rive-layout")]
        if let Some(layout) = self.base.parent_mut().as_mut::<LayoutComponent>() {
            layout.mark_layout_style_dirty();
        }
    }
    pub fn scale_type_changed(&mut self) {
        #[cfg(feature = "rive-layout")]
        if let Some(layout) = self.base.parent_mut().as_mut::<LayoutComponent>() {
            layout.scale_type_changed();
        }
    }
    pub fn display_changed(&mut self) {
        #[cfg(feature = "rive-layout")]
        if let Some(layout) = self.base.parent_mut().as_mut::<LayoutComponent>() {
            layout.display_changed();
        }
    }
    pub fn position_type_value_changed(&mut self) {
        #[cfg(feature = "rive-layout")]
        if let Some(layout) = self.base.parent_mut().as_mut::<LayoutComponent>() {
            layout.position_type_changed();
        }
    }
    pub fn flex_direction_value_changed(&mut self) {
        #[cfg(feature = "rive-layout")]
        if let Some(layout) = self.base.parent_mut().as_mut::<LayoutComponent>() {
            layout.flex_direction_changed();
        }
    }
    pub fn direction_value_changed(&mut self) {
        #[cfg(feature = "rive-layout")]
        if let Some(layout) = self.base.parent_mut().as_mut::<LayoutComponent>() {
            layout.direction_changed();
        }
    }
    #[cfg(feature = "rive-layout")]
    pub fn on_added_dirty(&mut self, context: &mut CoreContext) -> StatusCode {
        let code = self.base.on_added_dirty(context);
        if code != StatusCode::Ok {
            return code;
        }
        if let Some(interpolator) = context
            .resolve_mut(self.base.interpolator_id())
            .and_then(|object| object.as_mut::<KeyFrameInterpolator>())
        {
            self.interpolator = Some(interpolator);
        }
        StatusCode::Ok
    }

    pub fn interpolation_time_changed(&mut self) {
        self.mark_layout_style_dirty();
    }
    pub fn layout_alignment_type_changed(&mut self) {
        self.mark_layout_node_dirty();
    }
    pub fn layout_width_scale_type_changed(&mut self) {
        self.scale_type_changed();
    }
    pub fn layout_height_scale_type_changed(&mut self) {
        self.scale_type_changed();
    }
    pub fn display_value_changed(&mut self) {
        self.display_changed();
    }
    pub fn layout_type_value_changed(&mut self) {
        #[cfg(feature = "rive-layout")]
        if let Some(layout) = self.base.parent_mut().as_mut::<LayoutComponent>() {
            layout.layout_type_changed();
        }
    }
    pub fn justify_items_value_changed(&mut self) {
        self.mark_layout_node_dirty();
    }
    pub fn justify_self_value_changed(&mut self) {
        self.mark_layout_node_dirty();
    }
    pub fn overflow_value_changed(&mut self) {
        self.mark_layout_node_dirty();
    }
    pub fn intrinsically_sized_value_changed(&mut self) {
        self.mark_layout_node_dirty();
    }
    pub fn flex_wrap_value_changed(&mut self) {
        self.mark_layout_node_dirty();
    }
    pub fn flex_basis_changed(&mut self) {
        self.mark_layout_node_dirty();
    }
    pub fn aspect_ratio_changed(&mut self) {
        self.mark_layout_node_dirty();
    }
    pub fn gap_horizontal_changed(&mut self) {
        self.mark_layout_node_dirty();
    }
    pub fn gap_vertical_changed(&mut self) {
        self.mark_layout_node_dirty();
    }
    pub fn max_width_changed(&mut self) {
        self.mark_layout_node_dirty();
    }
    pub fn max_height_changed(&mut self) {
        self.mark_layout_node_dirty();
    }
    pub fn min_width_changed(&mut self) {
        self.mark_layout_node_dirty();
    }
    pub fn min_height_changed(&mut self) {
        self.mark_layout_node_dirty();
    }
    pub fn border_left_changed(&mut self) {
        self.mark_layout_node_dirty();
    }
    pub fn border_right_changed(&mut self) {
        self.mark_layout_node_dirty();
    }
    pub fn border_top_changed(&mut self) {
        self.mark_layout_node_dirty();
    }
    pub fn border_bottom_changed(&mut self) {
        self.mark_layout_node_dirty();
    }
    pub fn margin_left_changed(&mut self) {
        self.mark_layout_node_dirty();
    }
    pub fn margin_right_changed(&mut self) {
        self.mark_layout_node_dirty();
    }
    pub fn margin_top_changed(&mut self) {
        self.mark_layout_node_dirty();
    }
    pub fn margin_bottom_changed(&mut self) {
        self.mark_layout_node_dirty();
    }
    pub fn padding_left_changed(&mut self) {
        self.mark_layout_node_dirty();
    }
    pub fn padding_right_changed(&mut self) {
        self.mark_layout_node_dirty();
    }
    pub fn padding_top_changed(&mut self) {
        self.mark_layout_node_dirty();
    }
    pub fn padding_bottom_changed(&mut self) {
        self.mark_layout_node_dirty();
    }
    pub fn position_left_changed(&mut self) {
        if let Some(layout) = self.base.parent_mut().as_mut::<LayoutComponent>() {
            layout.mark_position_left_changed();
        }
        self.mark_layout_node_dirty();
    }
    pub fn position_right_changed(&mut self) {
        self.mark_layout_node_dirty();
    }
    pub fn position_top_changed(&mut self) {
        if let Some(layout) = self.base.parent_mut().as_mut::<LayoutComponent>() {
            layout.mark_position_top_changed();
        }
        self.mark_layout_node_dirty();
    }
    pub fn position_bottom_changed(&mut self) {
        self.mark_layout_node_dirty();
    }
    pub fn width_units_value_changed(&mut self) {
        self.mark_layout_node_dirty();
    }
    pub fn height_units_value_changed(&mut self) {
        self.mark_layout_node_dirty();
    }
    pub fn gap_horizontal_units_value_changed(&mut self) {
        self.mark_layout_node_dirty();
    }
    pub fn gap_vertical_units_value_changed(&mut self) {
        self.mark_layout_node_dirty();
    }
    pub fn max_width_units_value_changed(&mut self) {
        self.mark_layout_node_dirty();
    }
    pub fn max_height_units_value_changed(&mut self) {
        self.mark_layout_node_dirty();
    }
    pub fn min_width_units_value_changed(&mut self) {
        self.mark_layout_node_dirty();
    }
    pub fn min_height_units_value_changed(&mut self) {
        self.mark_layout_node_dirty();
    }
    pub fn border_left_units_value_changed(&mut self) {
        self.mark_layout_node_dirty();
    }
    pub fn border_right_units_value_changed(&mut self) {
        self.mark_layout_node_dirty();
    }
    pub fn border_top_units_value_changed(&mut self) {
        self.mark_layout_node_dirty();
    }
    pub fn border_bottom_units_value_changed(&mut self) {
        self.mark_layout_node_dirty();
    }
    pub fn margin_left_units_value_changed(&mut self) {
        self.mark_layout_node_dirty();
    }
    pub fn margin_right_units_value_changed(&mut self) {
        self.mark_layout_node_dirty();
    }
    pub fn margin_top_units_value_changed(&mut self) {
        self.mark_layout_node_dirty();
    }
    pub fn margin_bottom_units_value_changed(&mut self) {
        self.mark_layout_node_dirty();
    }
    pub fn padding_left_units_value_changed(&mut self) {
        self.mark_layout_node_dirty();
    }
    pub fn padding_right_units_value_changed(&mut self) {
        self.mark_layout_node_dirty();
    }
    pub fn padding_top_units_value_changed(&mut self) {
        self.mark_layout_node_dirty();
    }
    pub fn padding_bottom_units_value_changed(&mut self) {
        self.mark_layout_node_dirty();
    }
    pub fn position_left_units_value_changed(&mut self) {
        self.mark_layout_node_dirty();
    }
    pub fn position_right_units_value_changed(&mut self) {
        self.mark_layout_node_dirty();
    }
    pub fn position_top_units_value_changed(&mut self) {
        self.mark_layout_node_dirty();
    }
    pub fn position_bottom_units_value_changed(&mut self) {
        self.mark_layout_node_dirty();
    }
    pub fn corner_radius_tl_changed(&mut self) {
        self.mark_layout_style_dirty();
    }
    pub fn corner_radius_tr_changed(&mut self) {
        self.mark_layout_style_dirty();
    }
    pub fn corner_radius_bl_changed(&mut self) {
        self.mark_layout_style_dirty();
    }
    pub fn corner_radius_br_changed(&mut self) {
        self.mark_layout_style_dirty();
    }
}
