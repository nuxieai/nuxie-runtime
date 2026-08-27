use crate::mechanical_port::source::{
    animation::{linear_animation::LinearAnimation, nested_remap_animation::NestedRemapAnimation},
    artboard::Artboard,
    component_dirt::ComponentDirt,
    core::Core,
    core_context::CoreContext,
    generated::joystick_base::{JoystickBase, JoystickBaseCallbacks},
    intrinsically_sizeable::IntrinsicallySizeable,
    joystick_flags::JoystickFlags,
    layout::{
        layout_enums::{LayoutDirection, LayoutScaleType},
        layout_measure_mode::LayoutMeasureMode,
    },
    math::{aabb::Aabb, mat2d::Mat2D, vec2d::Vec2D},
    status_code::StatusCode,
    transform_component::TransformComponent,
};

pub struct Joystick {
    pub base: JoystickBase,
    world_transform: Mat2D,
    inverse_world_transform: Mat2D,
    x_animation: Option<*mut LinearAnimation>,
    y_animation: Option<*mut LinearAnimation>,
    handle_source: Option<*mut TransformComponent>,
    dependents: Vec<*mut NestedRemapAnimation>,
}

impl Default for Joystick {
    fn default() -> Self {
        Self {
            base: JoystickBase::default(),
            world_transform: Mat2D::default(),
            inverse_world_transform: Mat2D::default(),
            x_animation: None,
            y_animation: None,
            handle_source: None,
            dependents: Vec::new(),
        }
    }
}

impl Joystick {
    pub fn on_added_dirty(&mut self, context: &mut dyn CoreContext) -> StatusCode {
        let status = self.base.base.on_added_dirty(context);
        if status != StatusCode::Ok {
            return status;
        }
        if self.base.handle_source_id() != Core::EMPTY_ID {
            let Some(handle) = context
                .resolve(self.base.handle_source_id())
                .and_then(|object| object.as_transform_component_mut())
            else {
                return StatusCode::MissingObject;
            };
            self.handle_source = Some(handle as *mut TransformComponent);
        }
        StatusCode::Ok
    }

    pub fn on_added_clean(&mut self, _context: &mut dyn CoreContext) -> StatusCode {
        if let Some(artboard) = self.base.base.artboard_mut() {
            self.x_animation = artboard
                .animation(self.base.x_id())
                .map(|value| value as *mut _);
            self.y_animation = artboard
                .animation(self.base.y_id())
                .map(|value| value as *mut _);
        }
        StatusCode::Ok
    }

    pub fn build_dependencies(&mut self) {
        if let Some(handle_source) = self.handle_source
            && let Some(parent) = self.base.base.parent_mut()
        {
            let this = (&mut self.base.base) as *mut _;
            parent.base.base.add_dependent(this);
            unsafe { &mut *handle_source }
                .base
                .base
                .base
                .base
                .base
                .add_dependent(this);
        }
    }

    pub fn update(&mut self, value: ComponentDirt) {
        let Some(handle_source) = self.handle_source else {
            return;
        };
        if !value.contains(ComponentDirt::WORLD_TRANSFORM | ComponentDirt::TRANSFORM) {
            return;
        }
        let mut world = Mat2D::from_translate(self.base.pos_x(), self.base.pos_y());
        if let Some(parent) = self.base.base.parent_mut()
            && let Some(world_parent) = parent.as_world_transform_component_mut()
        {
            world = *world_parent.world_transform() * world;
        }
        if self.world_transform != world {
            self.world_transform = world;
            self.inverse_world_transform = world.invert_or_identity();
        }
        let position =
            self.inverse_world_transform * unsafe { &*handle_source }.base.base.world_translation();
        let bounds = Aabb::new(
            -self.base.width() * self.base.origin_x(),
            -self.base.height() * self.base.origin_y(),
            -self.base.width() * self.base.origin_x() + self.base.width(),
            -self.base.height() * self.base.origin_y() + self.base.height(),
        );
        let local = bounds.factor_from(position);
        self.base.set_x(local.x, self);
        self.base.set_y(local.y, self);
    }

    pub fn apply(&self, artboard: &mut Artboard) {
        if let Some(animation) = self.x_animation {
            let animation = unsafe { &mut *animation };
            let x = if self.is_joystick_flagged(JoystickFlags::INVERT_X) {
                -self.base.x()
            } else {
                self.base.x()
            };
            animation.apply(artboard, (x + 1.0) / 2.0 * animation.duration_seconds());
        }
        if let Some(animation) = self.y_animation {
            let animation = unsafe { &mut *animation };
            let y = if self.is_joystick_flagged(JoystickFlags::INVERT_Y) {
                -self.base.y()
            } else {
                self.base.y()
            };
            animation.apply(artboard, (y + 1.0) / 2.0 * animation.duration_seconds());
        }
        for dependent in self.dependents.iter().copied() {
            unsafe { &mut *dependent }.advance(0.0, false);
        }
    }

    pub fn is_joystick_flagged(&self, flag: JoystickFlags) -> bool {
        self.base.joystick_flags() as u8 & flag.0 != 0
    }

    pub fn can_apply_before_update(&self) -> bool {
        self.handle_source.is_none()
    }

    fn add_animation_dependents(
        &mut self,
        artboard: &mut Artboard,
        animation: *mut LinearAnimation,
    ) {
        let animation = unsafe { &mut *animation };
        for index in 0..animation.num_keyed_objects() {
            let object = animation.get_object(index);
            if let Some(dependent) = artboard
                .resolve(object.object_id())
                .and_then(|object| object.as_nested_remap_animation_mut())
            {
                self.dependents.push(dependent as *mut NestedRemapAnimation);
            }
        }
    }

    pub fn add_dependents(&mut self, artboard: &mut Artboard) {
        if let Some(animation) = self.y_animation {
            self.add_animation_dependents(artboard, animation);
        }
        if let Some(animation) = self.x_animation {
            self.add_animation_dependents(artboard, animation);
        }
    }

    fn add_component_dirt(&mut self) {
        if let Some(artboard) = self.base.base.artboard_mut() {
            artboard.add_dirt(ComponentDirt::COMPONENTS);
        }
    }
}

impl IntrinsicallySizeable for Joystick {
    fn measure_layout(
        &mut self,
        width: f32,
        width_mode: LayoutMeasureMode,
        height: f32,
        height_mode: LayoutMeasureMode,
    ) -> Vec2D {
        Vec2D::new(
            (if width_mode == LayoutMeasureMode::Undefined {
                f32::MAX
            } else {
                width
            })
            .min(self.base.width()),
            (if height_mode == LayoutMeasureMode::Undefined {
                f32::MAX
            } else {
                height
            })
            .min(self.base.height()),
        )
    }

    fn control_size(
        &mut self,
        size: Vec2D,
        _width_scale_type: LayoutScaleType,
        _height_scale_type: LayoutScaleType,
        _direction: LayoutDirection,
    ) {
        self.base.set_width(size.x, self);
        self.base.set_height(size.y, self);
        self.base.set_pos_x(size.x * self.base.origin_x(), self);
        self.base.set_pos_y(size.y * self.base.origin_y(), self);
    }

    fn should_propagate_size_to_children(&self) -> bool {
        false
    }
}

impl JoystickBaseCallbacks for Joystick {
    fn notify_property_changed(&mut self, property_key: u16) {
        self.base
            .base
            .base
            .base
            .notify_property_changed(property_key);
    }
    fn x_changed(&mut self) {
        self.add_component_dirt();
    }
    fn y_changed(&mut self) {
        self.add_component_dirt();
    }
    fn pos_x_changed(&mut self) {
        self.add_component_dirt();
    }
    fn pos_y_changed(&mut self) {
        self.add_component_dirt();
    }
    fn width_changed(&mut self) {
        self.add_component_dirt();
    }
    fn height_changed(&mut self) {
        self.add_component_dirt();
    }
}
