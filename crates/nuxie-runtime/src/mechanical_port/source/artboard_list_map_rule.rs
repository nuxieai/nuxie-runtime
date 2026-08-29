use crate::mechanical_port::source::{
    artboard_component_list::ArtboardComponentList,
    core_context::{CoreContext, StatusCode},
    generated::artboard_list_map_rule_base::{
        ArtboardListMapRuleBase, ArtboardListMapRuleBaseCallbacks,
    },
};

#[derive(Default)]
pub struct ArtboardListMapRule {
    pub base: ArtboardListMapRuleBase,
}

impl ArtboardListMapRuleBaseCallbacks for ArtboardListMapRule {
    fn notify_property_changed(&mut self, property_key: u16) {
        self.base
            .base
            .base
            .base
            .notify_property_changed(property_key);
    }
}

impl ArtboardListMapRule {
    pub fn on_added_dirty(&mut self, context: &mut dyn CoreContext) -> StatusCode {
        let code = self.base.on_added_dirty(context);
        if code != StatusCode::Ok {
            return code;
        }
        let Some(parent) = self.base.parent_handle() else {
            return StatusCode::MissingObject;
        };
        parent
            .with_downcast_mut::<ArtboardComponentList, _>(|parent| parent.add_map_rule(self))
            .map_or(StatusCode::MissingObject, |_| StatusCode::Ok)
    }
}

impl std::ops::Deref for ArtboardListMapRule {
    type Target = ArtboardListMapRuleBase;

    fn deref(&self) -> &Self::Target {
        &self.base
    }
}

impl std::ops::DerefMut for ArtboardListMapRule {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.base
    }
}
