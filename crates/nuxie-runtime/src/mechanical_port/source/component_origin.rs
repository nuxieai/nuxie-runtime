use crate::mechanical_port::source::{
    generated::component_origin_base::ComponentOriginBase, layout_component::LayoutComponent,
};

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
