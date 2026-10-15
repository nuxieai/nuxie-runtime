use crate::mechanical_port::source::layout::layout_style_applier::{
    YGAlign, YGDirection, YGDisplay, YGFlexDirection, YGJustify, YGOverflow, YGPositionType,
    YGUnit, YGWrap, physical_edges, rive_align_content, rive_align_items, rive_direction,
    rive_flex_direction, rive_flex_wrap, rive_inset, rive_justify_content, rive_justify_items,
    rive_length, rive_margin, rive_position,
};
use crate::mechanical_port::source::{
    core::CoreHandle,
    core_context::{CoreContext, StatusCode},
    generated::layout::layout_component_style_base::LayoutComponentStyleBase,
    layout::{
        grid_track::GridTrack,
        layout_enums::{
            LayoutAlignmentType, LayoutAnimationStyle, LayoutScaleType, LayoutStyleInterpolation,
        },
        layout_style_applier::{LayoutStyleApplier, LayoutSyncContext},
    },
    layout_component::LayoutComponent,
};
use taffy::prelude::{Size, Style};

impl std::ops::Deref for LayoutComponentStyle {
    type Target = LayoutComponentStyleBase;
    fn deref(&self) -> &Self::Target {
        &self.base
    }
}

impl std::ops::DerefMut for LayoutComponentStyle {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.base
    }
}

impl LayoutComponentStyle {
    fn layout_owner(&self) -> Option<CoreHandle> {
        self.base.parent_handle().filter(|parent| {
            parent
                .is_type_of(crate::mechanical_port::source::generated::layout_component_base::LayoutComponentBase::TYPE_KEY)
        })
    }
    pub const TYPE_KEY: u16 = LayoutComponentStyleBase::TYPE_KEY;
}

#[derive(Default)]
pub struct LayoutComponentStyle {
    pub base: LayoutComponentStyleBase,
    interpolator: Option<CoreHandle>,
}

impl LayoutStyleApplier for LayoutComponentStyle {
    fn apply_base_style(&self, style: &mut Style, _context: &LayoutSyncContext) {
        self.base
            .apply_base_style_with_display(style, self.display());
    }

    fn apply_container_style(&self, style: &mut Style, context: &LayoutSyncContext) {
        LayoutComponentStyle::apply_container_style(self, style, context);
    }

    fn apply_item_style(&self, style: &mut Style, context: &LayoutSyncContext) {
        LayoutComponentStyle::apply_item_style(self, style, context);
    }
}

impl LayoutComponentStyle {
    pub fn set_position_left(&mut self, value: f32) {
        if self.base.set_position_left_value(value) {
            self.position_left_changed();
            crate::mechanical_port::source::core::Core::notify_property_changed(
                self,
                LayoutComponentStyleBase::POSITION_LEFT_PROPERTY_KEY,
            );
        }
    }
    pub fn set_position_right(&mut self, value: f32) {
        if self.base.set_position_right_value(value) {
            self.position_right_changed();
            crate::mechanical_port::source::core::Core::notify_property_changed(
                self,
                LayoutComponentStyleBase::POSITION_RIGHT_PROPERTY_KEY,
            );
        }
    }
    pub fn set_position_top(&mut self, value: f32) {
        if self.base.set_position_top_value(value) {
            self.position_top_changed();
            crate::mechanical_port::source::core::Core::notify_property_changed(
                self,
                LayoutComponentStyleBase::POSITION_TOP_PROPERTY_KEY,
            );
        }
    }
    pub fn set_position_bottom(&mut self, value: f32) {
        if self.base.set_position_bottom_value(value) {
            self.position_bottom_changed();
            crate::mechanical_port::source::core::Core::notify_property_changed(
                self,
                LayoutComponentStyleBase::POSITION_BOTTOM_PROPERTY_KEY,
            );
        }
    }
    pub fn set_position_left_units_value(&mut self, value: u8) {
        if self.base.set_position_left_units_value_value(value) {
            self.position_left_units_value_changed();
            crate::mechanical_port::source::core::Core::notify_property_changed(
                self,
                LayoutComponentStyleBase::POSITION_LEFT_UNITS_VALUE_PROPERTY_KEY,
            );
        }
    }
    pub fn set_position_top_units_value(&mut self, value: u8) {
        if self.base.set_position_top_units_value_value(value) {
            self.position_top_units_value_changed();
            crate::mechanical_port::source::core::Core::notify_property_changed(
                self,
                LayoutComponentStyleBase::POSITION_TOP_UNITS_VALUE_PROPERTY_KEY,
            );
        }
    }
    pub fn set_position_right_units_value(&mut self, value: u8) {
        if self.base.set_position_right_units_value_value(value) {
            self.position_right_units_value_changed();
            crate::mechanical_port::source::core::Core::notify_property_changed(
                self,
                LayoutComponentStyleBase::POSITION_RIGHT_UNITS_VALUE_PROPERTY_KEY,
            );
        }
    }
    pub fn set_position_bottom_units_value(&mut self, value: u8) {
        if self.base.set_position_bottom_units_value_value(value) {
            self.position_bottom_units_value_changed();
            crate::mechanical_port::source::core::Core::notify_property_changed(
                self,
                LayoutComponentStyleBase::POSITION_BOTTOM_UNITS_VALUE_PROPERTY_KEY,
            );
        }
    }
    pub fn set_intrinsically_sized_value(&mut self, value: bool) {
        if self.base.set_intrinsically_sized_value_value(value) {
            self.intrinsically_sized_value_changed();
            crate::mechanical_port::source::core::Core::notify_property_changed(
                self,
                LayoutComponentStyleBase::INTRINSICALLY_SIZED_VALUE_PROPERTY_KEY,
            );
        }
    }
    fn with_layout_mut<R>(&mut self, f: impl FnOnce(&mut LayoutComponent) -> R) -> Option<R> {
        self.base
            .parent_handle()?
            .with_mut(|parent| parent.as_layout_component_mut().map(f))
            .flatten()
    }
    fn apply_stack_alignment(style: &mut Style, alignment: LayoutAlignmentType) {
        style.justify_items = rive_justify_items(match alignment {
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
        style.align_items = rive_align_items(match alignment {
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

    pub fn apply_item_style(&self, style: &mut Style, context: &LayoutSyncContext) {
        self.base.apply_sizing_item_style(style, context);
        style.position = rive_position(self.position_type());
        style.aspect_ratio = (self.base.aspect_ratio() > 0.0).then_some(self.base.aspect_ratio());
        // A layout root's margins are always lengths.
        let margin_unit = |unit| {
            if context.has_layout_parent {
                unit
            } else {
                YGUnit::Point
            }
        };
        style.margin = physical_edges(
            context.is_ltr,
            rive_margin(
                self.base.margin_left(),
                margin_unit(self.margin_left_units()),
            ),
            rive_margin(
                self.base.margin_right(),
                margin_unit(self.margin_right_units()),
            ),
            rive_margin(self.base.margin_top(), margin_unit(self.margin_top_units())),
            rive_margin(
                self.base.margin_bottom(),
                margin_unit(self.margin_bottom_units()),
            ),
        );
        style.inset = physical_edges(
            context.is_ltr,
            rive_inset(self.base.position_left(), self.position_left_units()),
            rive_inset(self.base.position_right(), self.position_right_units()),
            rive_inset(self.base.position_top(), self.position_top_units()),
            rive_inset(self.base.position_bottom(), self.position_bottom_units()),
        );
    }

    pub fn apply_container_style(&self, style: &mut Style, context: &LayoutSyncContext) {
        style.flex_direction = rive_flex_direction(self.flex_direction());
        style.flex_wrap = rive_flex_wrap(self.flex_wrap());
        // CSS inherits direction; the context carries this node's resolved one.
        style.direction = rive_direction(context.is_ltr);
        style.gap = Size {
            width: rive_length(self.base.gap_horizontal(), self.gap_horizontal_units()),
            height: rive_length(self.base.gap_vertical(), self.gap_vertical_units()),
        };
        style.border = physical_edges(
            context.is_ltr,
            rive_length(self.base.border_left(), self.border_left_units()),
            rive_length(self.base.border_right(), self.border_right_units()),
            rive_length(self.base.border_top(), self.border_top_units()),
            rive_length(self.base.border_bottom(), self.border_bottom_units()),
        );
        style.padding = physical_edges(
            context.is_ltr,
            rive_length(self.base.padding_left(), self.padding_left_units()),
            rive_length(self.base.padding_right(), self.padding_right_units()),
            rive_length(self.base.padding_top(), self.padding_top_units()),
            rive_length(self.base.padding_bottom(), self.padding_bottom_units()),
        );
        if self.is_stack() {
            GridTrack::sync_stack_container_style(
                style,
                u32::from(self.base.justify_items_value()),
            );
            Self::apply_stack_alignment(style, self.alignment_type());
            return;
        }
        let row = matches!(
            self.flex_direction(),
            YGFlexDirection::Row | YGFlexDirection::RowReverse
        );
        use crate::source::layout::layout_enums::{
            LayoutCrossAlign, LayoutMainDistribute, container_alignment,
        };
        let alignment = container_alignment(self.alignment_type(), row);
        style.justify_content = rive_justify_content(match alignment.main {
            LayoutMainDistribute::Start => YGJustify::FlexStart,
            LayoutMainDistribute::Center => YGJustify::Center,
            LayoutMainDistribute::End => YGJustify::FlexEnd,
            LayoutMainDistribute::SpaceBetween => YGJustify::SpaceBetween,
        });
        let cross = match alignment.cross {
            LayoutCrossAlign::Start => YGAlign::FlexStart,
            LayoutCrossAlign::Center => YGAlign::Center,
            LayoutCrossAlign::End => YGAlign::FlexEnd,
        };
        style.align_items = rive_align_items(cross);
        style.align_content = rive_align_content(cross);
    }

    pub fn interpolator(&self) -> Option<CoreHandle> {
        self.interpolator.clone()
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

    pub fn display(&self) -> YGDisplay {
        if YGDisplay::from(self.base.display_value()) == YGDisplay::None {
            YGDisplay::None
        } else if self.base.layout_type_value() == 0 {
            YGDisplay::Flex
        } else {
            YGDisplay::Grid
        }
    }
    pub fn position_type(&self) -> YGPositionType {
        YGPositionType::from(self.base.position_type_value())
    }
    pub fn flex_direction(&self) -> YGFlexDirection {
        YGFlexDirection::from(self.base.flex_direction_value())
    }
    pub fn direction(&self) -> YGDirection {
        YGDirection::from(self.base.direction_value())
    }
    pub fn flex_wrap(&self) -> YGWrap {
        YGWrap::from(self.base.flex_wrap_value())
    }
    pub fn overflow(&self) -> YGOverflow {
        YGOverflow::from(self.base.overflow_value())
    }
    pub fn intrinsically_sized(&self) -> bool {
        self.base.intrinsically_sized_value()
    }

    pub fn width_units(&self) -> YGUnit {
        YGUnit::from(self.base.width_units_value())
    }
    pub fn height_units(&self) -> YGUnit {
        YGUnit::from(self.base.height_units_value())
    }
    pub fn border_left_units(&self) -> YGUnit {
        YGUnit::from(self.base.border_left_units_value())
    }
    pub fn border_right_units(&self) -> YGUnit {
        YGUnit::from(self.base.border_right_units_value())
    }
    pub fn border_top_units(&self) -> YGUnit {
        YGUnit::from(self.base.border_top_units_value())
    }
    pub fn border_bottom_units(&self) -> YGUnit {
        YGUnit::from(self.base.border_bottom_units_value())
    }
    pub fn margin_left_units(&self) -> YGUnit {
        YGUnit::from(self.base.margin_left_units_value())
    }
    pub fn margin_right_units(&self) -> YGUnit {
        YGUnit::from(self.base.margin_right_units_value())
    }
    pub fn margin_top_units(&self) -> YGUnit {
        YGUnit::from(self.base.margin_top_units_value())
    }
    pub fn margin_bottom_units(&self) -> YGUnit {
        YGUnit::from(self.base.margin_bottom_units_value())
    }
    pub fn padding_left_units(&self) -> YGUnit {
        YGUnit::from(self.base.padding_left_units_value())
    }
    pub fn padding_right_units(&self) -> YGUnit {
        YGUnit::from(self.base.padding_right_units_value())
    }
    pub fn padding_top_units(&self) -> YGUnit {
        YGUnit::from(self.base.padding_top_units_value())
    }
    pub fn padding_bottom_units(&self) -> YGUnit {
        YGUnit::from(self.base.padding_bottom_units_value())
    }
    pub fn position_left_units(&self) -> YGUnit {
        YGUnit::from(self.base.position_left_units_value())
    }
    pub fn position_right_units(&self) -> YGUnit {
        YGUnit::from(self.base.position_right_units_value())
    }
    pub fn position_top_units(&self) -> YGUnit {
        YGUnit::from(self.base.position_top_units_value())
    }
    pub fn position_bottom_units(&self) -> YGUnit {
        YGUnit::from(self.base.position_bottom_units_value())
    }
    pub fn gap_horizontal_units(&self) -> YGUnit {
        YGUnit::from(self.base.gap_horizontal_units_value())
    }
    pub fn gap_vertical_units(&self) -> YGUnit {
        YGUnit::from(self.base.gap_vertical_units_value())
    }
    pub fn flex_basis_units(&self) -> YGUnit {
        YGUnit::from(self.base.flex_basis_units_value())
    }

    pub fn mark_layout_node_dirty(&mut self) {
        if let Some(parent) = self.layout_owner() {
            LayoutComponent::mark_layout_node_dirty_occurrence(&parent, false);
        }
    }
    pub fn mark_layout_style_dirty(&mut self) {
        if let Some(parent) = self.layout_owner() {
            LayoutComponent::mark_layout_style_dirty_occurrence(&parent);
        }
    }
    pub fn scale_type_changed(&mut self) {
        if let Some(parent) = self.layout_owner() {
            LayoutComponent::scale_type_changed_from_style(&parent, self);
        }
    }
    pub fn mark_render_path_dirty(&mut self) {
        if let Some(parent) = self.layout_owner() {
            parent.with_mut(|parent| {
                parent.component_add_dirt(
                    crate::mechanical_port::source::component_dirt::ComponentDirt::PATH,
                    false,
                )
            });
        }
    }
    pub fn display_changed(&mut self) {
        if let Some(parent) = self.layout_owner() {
            LayoutComponent::display_changed_from_style(&parent, self);
        }
    }
    pub fn position_type_value_changed(&mut self) {
        if let Some(parent) = self.layout_owner() {
            LayoutComponent::position_type_changed_from_style(&parent, self);
        }
    }
    pub fn flex_direction_value_changed(&mut self) {
        if let Some(parent) = self.layout_owner() {
            LayoutComponent::flow_style_changed_from_style(&parent, self);
        }
    }
    pub fn direction_value_changed(&mut self) {
        if let Some(parent) = self.layout_owner() {
            LayoutComponent::direction_changed_occurrence(&parent);
        }
    }
    pub fn on_added_dirty(&mut self, context: &mut dyn CoreContext) -> StatusCode {
        let code = self.base.on_added_dirty(context);
        if code != StatusCode::Ok {
            return code;
        }
        self.interpolator = context
            .resolve(self.base.interpolator_id())
            .filter(|object| {
                object.is_type_of(
                    crate::mechanical_port::source::generated::animation::keyframe_interpolator_base::KeyFrameInterpolatorBase::TYPE_KEY,
                )
            });
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
    pub fn hug_unbounded_changed(&mut self) {
        self.mark_layout_node_dirty();
    }
    pub fn display_value_changed(&mut self) {
        self.display_changed();
    }
    pub fn layout_type_value_changed(&mut self) {
        if let Some(parent) = self.layout_owner() {
            LayoutComponent::flow_style_changed_from_style(&parent, self);
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
        self.with_layout_mut(LayoutComponent::mark_position_left_changed);
        self.mark_layout_node_dirty();
    }
    pub fn position_right_changed(&mut self) {
        self.mark_layout_node_dirty();
    }
    pub fn position_top_changed(&mut self) {
        self.with_layout_mut(LayoutComponent::mark_position_top_changed);
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
    pub fn link_corner_radius_changed(&mut self) {
        self.mark_render_path_dirty();
    }
    pub fn corner_radius_tl_changed(&mut self) {
        self.mark_render_path_dirty();
    }
    pub fn corner_radius_tr_changed(&mut self) {
        self.mark_render_path_dirty();
    }
    pub fn corner_radius_bl_changed(&mut self) {
        self.mark_render_path_dirty();
    }
    pub fn corner_radius_br_changed(&mut self) {
        self.mark_render_path_dirty();
    }
}
