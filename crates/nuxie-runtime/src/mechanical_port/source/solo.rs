use crate::mechanical_port::source::{
    component::Component,
    core::CoreHandle,
    generated::{
        constraints::constraint_base::ConstraintBase,
        focus_data_base::FocusDataBase,
        semantic::semantic_data_base::SemanticDataBase,
        shapes::clipping_shape_base::ClippingShapeBase,
        solo_base::{SoloBase, SoloBaseCallbacks},
    },
    status_code::StatusCode,
};

#[derive(Default)]
pub struct Solo {
    pub base: SoloBase,
}

fn is_solo_set_member(child: &CoreHandle) -> bool {
    !(child.is_type_of(ConstraintBase::TYPE_KEY)
        || child.is_type_of(ClippingShapeBase::TYPE_KEY)
        || child.is_type_of(FocusDataBase::TYPE_KEY)
        || child.is_type_of(SemanticDataBase::TYPE_KEY))
}

impl Solo {
    pub fn active_component(&self) -> Option<CoreHandle> {
        let active = self
            .base
            .base
            .base
            .base
            .base
            .artboard_handle()?
            .with_downcast::<crate::mechanical_port::source::artboard::Artboard, _>(|artboard| {
                artboard.resolve_handle(self.base.active_component_id())
            })
            .flatten()?;
        self.base
            .base
            .base
            .base
            .base
            .children()
            .iter()
            .find(|child| *child == &active)
            .cloned()
    }

    pub fn recollect_owning_layout(&mut self) {
        let mut parent = self.base.base.base.base.base.parent_handle();
        while let Some(current) = parent {
            let synced = current
                .with_mut(|current| {
                    current.as_layout_component_mut().map(|layout| {
                        layout.sync_layout_children();
                    })
                })
                .flatten()
                .is_some();
            if synced {
                return;
            }
            parent = current
                .with(|current| current.as_component().and_then(Component::parent_handle))
                .flatten();
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
                .artboard_handle()
                .and_then(|artboard| {
                    artboard
                        .with_downcast::<crate::mechanical_port::source::artboard::Artboard, _>(
                            |artboard| artboard.resolve_handle(self.base.active_component_id()),
                        )
                        .flatten()
                })
        };
        let children = self.base.base.base.base.base.children().to_vec();
        for child in children {
            if !is_solo_set_member(&child) {
                child.with_mut(|child| {
                    if let Some(child) = child.as_component_mut() {
                        child.collapse(collapse);
                    }
                });
                continue;
            }
            let child_is_active = active.as_ref() == Some(&child);
            child.with_mut(|child| {
                if let Some(child) = child.as_component_mut() {
                    child.collapse(!child_is_active);
                }
            });
        }
    }

    fn set_active_component_id(&mut self, value: u32) {
        if !self.base.set_active_component_id_value(value) {
            return;
        }
        self.active_component_id_changed();
        self.notify_property_changed(SoloBase::ACTIVE_COMPONENT_ID_PROPERTY_KEY);
    }

    pub fn collapse(&mut self, value: bool) -> bool {
        if !self.base.base.base.base.base.collapse(value) {
            return false;
        }
        self.collapse_after_component(value);
        true
    }

    pub(crate) fn collapse_after_component(&mut self, value: bool) {
        self.propagate_collapse(value);
    }

    pub fn active_component_id_changed(&mut self) {
        let collapsed = self.base.base.base.base.base.is_collapsed();
        self.propagate_collapse(collapsed);
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
        self.recollect_owning_layout();
        StatusCode::Ok
    }

    pub fn update_by_index(&mut self, index: usize) {
        let children = self.base.base.base.base.base.children().to_vec();
        let Some(artboard) = self.base.base.base.base.base.artboard_handle() else {
            return;
        };
        if index >= children.len() {
            return;
        }
        let mut solo_index = 0;
        for child in children {
            if !is_solo_set_member(&child) {
                continue;
            }
            if solo_index == index {
                let id = artboard
                    .with_downcast::<crate::mechanical_port::source::artboard::Artboard, _>(
                        |artboard| artboard.id_of(&child),
                    )
                    .unwrap_or(0);
                self.set_active_component_id(id);
                return;
            }
            solo_index += 1;
        }
    }

    pub fn update_by_name(&mut self, name: &str) {
        let Some(artboard) = self.base.base.base.base.base.artboard_handle() else {
            return;
        };
        let children = self.base.base.base.base.base.children().to_vec();
        for child in children {
            let name_matches = child
                .with(|child| {
                    child
                        .as_component()
                        .is_some_and(|component| component.base.name() == name)
                })
                .unwrap_or(false);
            if is_solo_set_member(&child) && name_matches {
                let id = artboard
                    .with_downcast::<crate::mechanical_port::source::artboard::Artboard, _>(
                        |artboard| artboard.id_of(&child),
                    )
                    .unwrap_or(0);
                self.set_active_component_id(id);
                break;
            }
        }
    }

    pub fn get_active_child_index(&mut self) -> i32 {
        let Some(artboard) = self.base.base.base.base.base.artboard_handle() else {
            return -1;
        };
        let active = artboard
            .with_downcast::<crate::mechanical_port::source::artboard::Artboard, _>(|artboard| {
                artboard.resolve_handle(self.base.active_component_id())
            })
            .flatten();
        let Some(active) = active else {
            return -1;
        };
        let mut index = 0;
        for child in self.base.base.base.base.base.children() {
            if !is_solo_set_member(child) {
                continue;
            }
            if child == &active {
                return index;
            }
            index += 1;
        }
        -1
    }

    pub fn get_active_child_name(&self) -> String {
        self.base
            .base
            .base
            .base
            .base
            .artboard_handle()
            .and_then(|artboard| {
                artboard
                    .with_downcast::<crate::mechanical_port::source::artboard::Artboard, _>(
                        |artboard| artboard.resolve_handle(self.base.active_component_id()),
                    )
                    .flatten()
            })
            .and_then(|active| {
                active
                    .with(|active| {
                        active
                            .as_component()
                            .map(|component| component.base.name().to_owned())
                    })
                    .flatten()
            })
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

impl std::ops::Deref for Solo {
    type Target = SoloBase;

    fn deref(&self) -> &Self::Target {
        &self.base
    }
}

impl std::ops::DerefMut for Solo {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.base
    }
}
