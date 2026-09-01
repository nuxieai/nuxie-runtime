use crate::mechanical_port::source::{
    core_context::CoreContext,
    generated::component_origin_base::{ComponentOriginBase, ComponentOriginBaseCallbacks},
    layout_component::LayoutComponent,
    status_code::StatusCode,
};

#[derive(Default)]
pub struct ComponentOrigin {
    pub base: ComponentOriginBase,
}

impl ComponentOrigin {
    pub fn on_added_dirty(&mut self, context: &mut dyn CoreContext) -> StatusCode {
        let code = self.base.base.on_added_dirty(context);
        if code != StatusCode::Ok {
            return code;
        }
        let Some(owner) = self.base.parent_handle() else {
            return StatusCode::Ok;
        };
        owner.with_mut(|owner| {
            if let Some(layout) = owner.as_layout_component_mut() {
                layout.mark_has_component_origin();
            }
        });
        StatusCode::Ok
    }

    fn reapply(&mut self) {
        let Some(owner) = self.base.parent_handle() else {
            return;
        };
        if owner.is_type_of(crate::mechanical_port::source::generated::nested_artboard_base::NestedArtboardBase::TYPE_KEY) {
            // A child origin setter dirties this same NestedArtboard host.
            // Only retain its instance handle, not the host's Core borrow.
            let instance = owner.with(|owner| owner.as_nested_artboard().unwrap().artboard_instance_handle(0)).flatten();
            if let Some(instance) = instance {
                instance.with_artboard_mut(|instance| instance.set_origin_x(self.base.origin_x()));
                instance.with_artboard_mut(|instance| instance.set_origin_y(self.base.origin_y()));
            }
            return;
        }
        owner.with_mut(|owner| {
            if owner.as_artboard().is_none() {
                if let Some(layout) = owner.as_layout_component_mut() {
                    layout.mark_world_transform_dirty();
                }
            }
        });
    }

    pub fn origin_x_changed(&mut self) {
        self.reapply();
    }

    pub fn origin_y_changed(&mut self) {
        self.reapply();
    }
}

impl ComponentOriginBaseCallbacks for ComponentOrigin {
    fn notify_property_changed(&mut self, property_key: u16) {
        self.base.base.base.notify_property_changed(property_key);
    }

    fn origin_x_changed(&mut self) {
        ComponentOrigin::origin_x_changed(self);
    }

    fn origin_y_changed(&mut self) {
        ComponentOrigin::origin_y_changed(self);
    }
}

impl std::ops::Deref for ComponentOrigin {
    type Target = ComponentOriginBase;

    fn deref(&self) -> &Self::Target {
        &self.base
    }
}

impl std::ops::DerefMut for ComponentOrigin {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.base
    }
}
