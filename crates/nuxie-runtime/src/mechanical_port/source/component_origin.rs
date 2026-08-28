use crate::mechanical_port::source::{
    generated::component_origin_base::{ComponentOriginBase, ComponentOriginBaseCallbacks},
    layout_component::LayoutComponent,
};

#[derive(Default)]
pub struct ComponentOrigin {
    pub base: ComponentOriginBase,
}

impl ComponentOrigin {
    fn reapply(&mut self) {
        let Some(owner) = self.base.parent_mut() else {
            return;
        };
        if let Some(nested) = owner.as_nested_artboard_mut() {
            if let Some(instance) = nested.artboard_instance_mut() {
                instance.set_origin_x(self.base.origin_x());
                instance.set_origin_y(self.base.origin_y());
            }
            return;
        }
        if owner.is::<LayoutComponent>() && !owner.is_artboard() {
            owner
                .as_layout_component_mut()
                .unwrap()
                .mark_world_transform_dirty();
        }
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
