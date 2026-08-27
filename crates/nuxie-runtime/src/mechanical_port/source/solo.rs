use crate::mechanical_port::source::{
    component::Component,
    generated::solo_base::{SoloBase, SoloBaseCallbacks},
    status_code::StatusCode,
};

#[derive(Default)]
pub struct Solo {
    pub base: SoloBase,
}

fn is_solo_set_member(child: &mut Component) -> bool {
    !(child.is_constraint()
        || child.is_clipping_shape()
        || child.is_focus_data()
        || child.is_semantic_data())
}

impl Solo {
    pub fn active_component(&mut self) -> Option<&mut Component> {
        let active = self
            .base
            .base
            .base
            .base
            .base
            .artboard_mut()?
            .resolve(self.base.active_component_id())? as *mut _;
        for child in self.base.base.base.base.base.children().iter().copied() {
            if std::ptr::eq(child.cast(), active) {
                return Some(unsafe { &mut *child });
            }
        }
        None
    }

    #[cfg(feature = "rive_layout")]
    pub fn recollect_owning_layout(&mut self) {
        let mut parent = self.base.base.base.base.base.parent_mut();
        while let Some(current) = parent {
            if let Some(layout) = current.as_layout_component_mut() {
                layout.sync_layout_children();
                return;
            }
            parent = current.base.base.parent_mut();
        }
    }

    fn propagate_collapse(&mut self, collapse: bool) {
        let active = if collapse {
            None
        } else {
            self.base
                .base
                .base
                .base
                .base
                .artboard_mut()
                .and_then(|artboard| artboard.resolve(self.base.active_component_id()))
                .map(|core| core as *mut _)
        };
        for child in self.base.base.base.base.base.children().iter().copied() {
            let child_ref = unsafe { &mut *child };
            if !is_solo_set_member(child_ref) {
                child_ref.collapse(collapse);
                continue;
            }
            child_ref.collapse(active.is_none_or(|active| !std::ptr::eq(child.cast(), active)));
        }
    }

    pub fn collapse(&mut self, value: bool) -> bool {
        if !self.base.base.base.base.base.collapse(value) {
            return false;
        }
        self.propagate_collapse(value);
        true
    }

    pub fn active_component_id_changed(&mut self) {
        let collapsed = self.base.base.base.base.base.is_collapsed();
        self.propagate_collapse(collapsed);
        #[cfg(feature = "rive_layout")]
        self.recollect_owning_layout();
    }

    pub fn on_added_clean(
        &mut self,
        context: &mut dyn crate::mechanical_port::source::core_context::CoreContext,
    ) -> StatusCode {
        let code = self.base.base.base.base.base.on_added_clean(context);
        if code != StatusCode::Ok {
            return code;
        }
        let collapsed = self.base.base.base.base.base.is_collapsed();
        self.propagate_collapse(collapsed);
        #[cfg(feature = "rive_layout")]
        self.recollect_owning_layout();
        StatusCode::Ok
    }

    pub fn update_by_index(&mut self, index: usize) {
        let children = self.base.base.base.base.base.children();
        if self.base.base.base.base.base.artboard().is_none() || index >= children.len() {
            return;
        }
        let mut solo_index = 0;
        for child in children.iter().copied() {
            if !is_solo_set_member(unsafe { &mut *child }) {
                continue;
            }
            if solo_index == index {
                let id = self
                    .base
                    .base
                    .base
                    .base
                    .base
                    .artboard_mut()
                    .unwrap()
                    .id_of(child);
                self.base.set_active_component_id(id, self);
                return;
            }
            solo_index += 1;
        }
    }

    pub fn update_by_name(&mut self, name: &str) {
        if self.base.base.base.base.base.artboard().is_none() {
            return;
        }
        for child in self.base.base.base.base.base.children().iter().copied() {
            let child_ref = unsafe { &mut *child };
            if is_solo_set_member(child_ref) && child_ref.base.name() == name {
                let id = self
                    .base
                    .base
                    .base
                    .base
                    .base
                    .artboard_mut()
                    .unwrap()
                    .id_of(child);
                self.base.set_active_component_id(id, self);
                break;
            }
        }
    }

    pub fn get_active_child_index(&mut self) -> i32 {
        let Some(artboard) = self.base.base.base.base.base.artboard_mut() else {
            return -1;
        };
        let active = artboard
            .resolve(self.base.active_component_id())
            .map(|core| core as *mut _);
        let Some(active) = active else {
            return -1;
        };
        let mut index = 0;
        for child in self.base.base.base.base.base.children().iter().copied() {
            if !is_solo_set_member(unsafe { &mut *child }) {
                continue;
            }
            if std::ptr::eq(child.cast(), active) {
                return index;
            }
            index += 1;
        }
        -1
    }

    pub fn get_active_child_name(&mut self) -> String {
        self.active_component()
            .map(|component| component.base.name().to_owned())
            .unwrap_or_default()
    }
}

impl SoloBaseCallbacks for Solo {
    fn notify_property_changed(&mut self, property_key: u16) {
        self.base.base.notify_property_changed(property_key);
    }
    fn active_component_id_changed(&mut self) {
        Solo::active_component_id_changed(self);
    }
}
