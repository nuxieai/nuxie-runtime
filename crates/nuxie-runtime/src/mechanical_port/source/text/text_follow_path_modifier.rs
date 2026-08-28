use super::text_modifier_group::TransformGlyphArg;
use crate::mechanical_port::source::{
    component_dirt::ComponentDirt,
    core_context::CoreContext,
    generated::text::text_follow_path_modifier_base::TextFollowPathModifierBase,
    math::{
        mat2d::Mat2D, path_measure::PathMeasure, raw_path::RawPath,
        transform_components::TransformComponents, vec2d::Vec2D,
    },
    status_code::StatusCode,
};
pub struct TextFollowPathModifier {
    pub base: TextFollowPathModifierBase,
    world_path: RawPath,
    local_path: RawPath,
    path_measure: PathMeasure,
}
impl TextFollowPathModifier {
    pub fn build_dependencies(&mut self) {
        let Some(dependent) = self.base.handle() else {
            return;
        };
        if let Some(target) = self.base.target() {
            target.with_mut(|target| {
                if let Some(shape) = target.as_shape_mut() {
                    shape.path_composer_mut().add_dependent(dependent.clone());
                } else if let Some(path) = target.as_path_mut() {
                    path.add_dependent(dependent.clone());
                }
            });
        }
        if let Some(text) = self.base.text_component() {
            self.base.add_dependent(text);
        }
    }
    pub fn on_added_clean(&mut self, context: &mut dyn CoreContext) -> StatusCode {
        if let Some(target) = self.base.target() {
            target.with_mut(|target| {
                if let Some(shape) = target.as_shape_mut() {
                    shape.add_follow_path_flag();
                } else if let Some(path) = target.as_path_mut() {
                    path.add_follow_path_flag();
                }
            });
        }
        self.base.on_added_clean(context)
    }
    pub fn update(&mut self, _value: ComponentDirt) {
        self.world_path.rewind();
        if let Some(target) = self.base.target() {
            target.with(|target| {
                if let Some(shape) = target.as_shape() {
                    for path in shape.paths() {
                        path.with(|path| {
                            if let Some(path) = path.as_path() {
                                self.world_path
                                    .add_path(path.raw_path(), Some(path.path_transform()));
                            }
                        });
                    }
                } else if let Some(path) = target.as_path() {
                    self.world_path
                        .add_path(path.raw_path(), Some(path.path_transform()));
                }
            });
        }
    }
    pub fn modifier_shape_dirty(&mut self) {
        if let Some(text) = self.base.text_component() {
            text.with_mut(|text| {
                if let Some(text) = text.as_text_mut() {
                    text.modifier_shape_dirty();
                }
            });
        }
    }
    pub fn radial_changed(&mut self) {
        self.modifier_shape_dirty();
    }
    pub fn orient_changed(&mut self) {
        self.modifier_shape_dirty();
    }
    pub fn start_changed(&mut self) {
        self.modifier_shape_dirty();
    }
    pub fn end_changed(&mut self) {
        self.modifier_shape_dirty();
    }
    pub fn offset_changed(&mut self) {
        self.modifier_shape_dirty();
    }
    pub fn strength_changed(&mut self) {
        self.modifier_shape_dirty();
    }
    pub fn reset(&mut self, inverse_text: &Mat2D) {
        if self.base.target().is_none() {
            self.path_measure = PathMeasure::default();
            return;
        }
        self.local_path.rewind();
        self.local_path
            .add_path(&self.world_path, Some(inverse_text));
        self.path_measure = PathMeasure::new(&self.local_path, 0.1);
    }
    pub fn transform_glyph(
        &self,
        current: TransformComponents,
        arg: &TransformGlyphArg,
    ) -> TransformComponents {
        let length = self.path_measure.length();
        if length == 0.0 {
            return current;
        }
        let position_on_path = arg.origin_position + arg.offset;
        let mut start = self.base.start().min(self.base.end()).clamp(0.0, 1.0);
        let mut end = self.base.start().max(self.base.end()).clamp(0.0, 1.0);
        let can_wrap = self.local_path.is_closed() && end - start == 1.0;
        let valid_length = (end - start) * length;
        let offset = ((self.base.offset() % 1.0) + 1.0) % 1.0;
        start += offset;
        end += offset;
        let (position, tangent) = if (!can_wrap && position_on_path.x < 0.0) || start == end {
            let result = self.path_measure.at_percentage(start);
            let tangent = result.tangent.normalized();
            (result.position - tangent * (-position_on_path.x), tangent)
        } else if !can_wrap && position_on_path.x > valid_length {
            let result = self.path_measure.at_percentage(end);
            let tangent = result.tangent.normalized();
            (
                result.position + tangent * (position_on_path.x - valid_length),
                tangent,
            )
        } else {
            let result = self
                .path_measure
                .at_percentage(start + position_on_path.x / length);
            (result.position, result.tangent.normalized())
        };
        let line = arg.line_index_in_paragraph;
        let last_baseline = if line > 0 {
            arg.paragraph_lines[(line - 1) as usize].baseline
        } else {
            0.0
        };
        let current_baseline = arg
            .paragraph_lines
            .get(line as usize)
            .map_or(0.0, |l| l.baseline);
        let translation = if self.base.radial() {
            let spacing = position_on_path.y - current_baseline;
            let perpendicular = Vec2D::new(-tangent.y, tangent.x);
            position + perpendicular * spacing
        } else {
            Vec2D::new(
                position.x,
                position_on_path.y - current_baseline + position.y + last_baseline,
            )
        };
        let rotation = if self.base.orient() {
            tangent.y.atan2(tangent.x)
        } else {
            0.0
        };
        let t = self.base.strength().clamp(0.0, 1.0);
        let ti = 1.0 - t;
        TransformComponents::from_xy_rotation(
            translation.x * t + current.x() * ti,
            translation.y * t + current.y() * ti,
            rotation * t + current.rotation() * ti,
        )
    }
}
