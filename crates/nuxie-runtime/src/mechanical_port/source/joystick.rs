use crate::mechanical_port::source::{
    animation::{
        keyed_object::KeyedObject,
        linear_animation::{LinearAnimation, LinearAnimationArtboard},
        nested_remap_animation::NestedRemapAnimation,
    },
    artboard::Artboard,
    component::Component,
    component_dirt::ComponentDirt,
    core::{Core, CoreHandle},
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
};

pub struct Joystick {
    pub base: JoystickBase,
    world_transform: Mat2D,
    inverse_world_transform: Mat2D,
    x_animation: Option<CoreHandle>,
    y_animation: Option<CoreHandle>,
    handle_source: Option<CoreHandle>,
    dependents: Vec<CoreHandle>,
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
                .filter(|object| {
                    object
                        .with(|object| object.as_transform_component().is_some())
                        .unwrap_or(false)
                })
            else {
                return StatusCode::MissingObject;
            };
            self.handle_source = Some(handle);
        }
        StatusCode::Ok
    }

    pub fn on_added_clean(&mut self, _context: &mut dyn CoreContext) -> StatusCode {
        if let Some(artboard) = self.base.base.artboard_handle() {
            self.x_animation = artboard
                .with_downcast::<Artboard, _>(|artboard| {
                    artboard.animation_handle_at(self.base.x_id() as usize)
                })
                .flatten();
            self.y_animation = artboard
                .with_downcast::<Artboard, _>(|artboard| {
                    artboard.animation_handle_at(self.base.y_id() as usize)
                })
                .flatten();
        }
        StatusCode::Ok
    }

    pub fn build_dependencies(&mut self) {
        if let Some(handle_source) = self.handle_source.clone()
            && let Some(parent) = self.base.base.parent_handle()
            && let Some(this) =
                crate::mechanical_port::source::core::CoreObject::core(self).handle()
        {
            parent.with_mut(|parent| parent.component_add_dependent(this.clone()));
            handle_source.with_mut(|handle_source| handle_source.component_add_dependent(this));
        }
    }

    pub fn update(&mut self, value: ComponentDirt) {
        let Some(handle_source) = self.handle_source.clone() else {
            return;
        };
        if !Component::has_dirt_in(
            value,
            ComponentDirt::WORLD_TRANSFORM | ComponentDirt::TRANSFORM,
        ) {
            return;
        }
        let mut world = Mat2D::from_translate(self.base.pos_x(), self.base.pos_y());
        if let Some(parent) = self.base.base.parent_handle()
            && let Some(parent_world) = parent
                .with(|parent| {
                    parent
                        .as_world_transform_component()
                        .map(|parent| *parent.world_transform())
                })
                .flatten()
        {
            world = parent_world * world;
        }
        if self.world_transform != world {
            self.world_transform = world;
            self.inverse_world_transform = world.invert_or_identity();
        }
        let Some(handle_position) = handle_source
            .with(|handle_source| {
                handle_source
                    .as_world_transform_component()
                    .map(|handle_source| handle_source.world_translation())
            })
            .flatten()
        else {
            return;
        };
        let position = self.inverse_world_transform * handle_position;
        let bounds = Aabb::new(
            -self.base.width() * self.base.origin_x(),
            -self.base.height() * self.base.origin_y(),
            -self.base.width() * self.base.origin_x() + self.base.width(),
            -self.base.height() * self.base.origin_y() + self.base.height(),
        );
        let local = bounds.factor_from(position);
        self.set_x(local.x);
        self.set_y(local.y);
    }

    pub fn apply(&self, artboard: &mut dyn LinearAnimationArtboard) {
        if let Some(animation) = &self.x_animation {
            let x = if self.is_joystick_flagged(JoystickFlags::INVERT_X) {
                -self.base.x()
            } else {
                self.base.x()
            };
            animation.with_downcast_mut::<LinearAnimation, _>(|animation| {
                let time = (x + 1.0) / 2.0 * animation.duration_seconds();
                animation.apply(artboard, time, 1.0, None);
            });
        }
        if let Some(animation) = &self.y_animation {
            let y = if self.is_joystick_flagged(JoystickFlags::INVERT_Y) {
                -self.base.y()
            } else {
                self.base.y()
            };
            animation.with_downcast_mut::<LinearAnimation, _>(|animation| {
                let time = (y + 1.0) / 2.0 * animation.duration_seconds();
                animation.apply(artboard, time, 1.0, None);
            });
        }
        for dependent in &self.dependents {
            dependent.with_downcast_mut::<NestedRemapAnimation, _>(|dependent| {
                dependent.advance(0.0, false);
            });
        }
    }

    pub fn is_joystick_flagged(&self, flag: JoystickFlags) -> bool {
        self.base.joystick_flags() as u8 & flag.0 != 0
    }

    pub fn can_apply_before_update(&self) -> bool {
        self.handle_source.is_none()
    }

    fn add_animation_dependents(&mut self, artboard: &dyn CoreContext, animation: CoreHandle) {
        let object_ids = animation
            .with_downcast::<LinearAnimation, _>(|animation| {
                (0..animation.num_keyed_objects())
                    .filter_map(|index| animation.get_object(index))
                    .filter_map(|object| {
                        object.with_downcast::<KeyedObject, _>(|object| object.base.object_id())
                    })
                    .collect::<Vec<_>>()
            })
            .unwrap_or_default();
        for object_id in object_ids {
            let Some(dependent) = artboard.resolve_handle(object_id).filter(|object| {
                object
                    .is_type_of(crate::mechanical_port::source::generated::animation::nested_remap_animation_base::NestedRemapAnimationBase::TYPE_KEY)
            }) else {
                continue;
            };
            self.dependents.push(dependent);
        }
    }

    pub fn add_dependents(&mut self, artboard: &dyn CoreContext) {
        if let Some(animation) = self.y_animation.clone() {
            self.add_animation_dependents(artboard, animation);
        }
        if let Some(animation) = self.x_animation.clone() {
            self.add_animation_dependents(artboard, animation);
        }
    }

    fn add_component_dirt(&mut self) {
        self.base
            .base
            .with_artboard_mut(|artboard| artboard.add_dirt(ComponentDirt::COMPONENTS, false));
    }

    fn set_x(&mut self, value: f32) {
        if self.base.set_x_value(value) {
            JoystickBaseCallbacks::x_changed(self);
            JoystickBaseCallbacks::notify_property_changed(self, JoystickBase::X_PROPERTY_KEY);
        }
    }

    fn set_y(&mut self, value: f32) {
        if self.base.set_y_value(value) {
            JoystickBaseCallbacks::y_changed(self);
            JoystickBaseCallbacks::notify_property_changed(self, JoystickBase::Y_PROPERTY_KEY);
        }
    }

    fn set_width(&mut self, value: f32) {
        if self.base.set_width_value(value) {
            JoystickBaseCallbacks::width_changed(self);
            JoystickBaseCallbacks::notify_property_changed(self, JoystickBase::WIDTH_PROPERTY_KEY);
        }
    }

    fn set_height(&mut self, value: f32) {
        if self.base.set_height_value(value) {
            JoystickBaseCallbacks::height_changed(self);
            JoystickBaseCallbacks::notify_property_changed(self, JoystickBase::HEIGHT_PROPERTY_KEY);
        }
    }

    fn set_pos_x(&mut self, value: f32) {
        if self.base.set_pos_x_value(value) {
            JoystickBaseCallbacks::pos_x_changed(self);
            JoystickBaseCallbacks::notify_property_changed(self, JoystickBase::POS_X_PROPERTY_KEY);
        }
    }

    fn set_pos_y(&mut self, value: f32) {
        if self.base.set_pos_y_value(value) {
            JoystickBaseCallbacks::pos_y_changed(self);
            JoystickBaseCallbacks::notify_property_changed(self, JoystickBase::POS_Y_PROPERTY_KEY);
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
        self.set_width(size.x);
        self.set_height(size.y);
        self.set_pos_x(size.x * self.base.origin_x());
        self.set_pos_y(size.y * self.base.origin_y());
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

impl std::ops::Deref for Joystick {
    type Target = JoystickBase;

    fn deref(&self) -> &Self::Target {
        &self.base
    }
}

impl std::ops::DerefMut for Joystick {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.base
    }
}
