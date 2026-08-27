use crate::mechanical_port::source::{
    artboard_component_list::ArtboardComponentList,
    core_context::{CoreContext, StatusCode},
    generated::artboard_list_map_rule_base::ArtboardListMapRuleBase,
};

pub struct ArtboardListMapRule {
    pub base: ArtboardListMapRuleBase,
}

impl ArtboardListMapRule {
    pub fn on_added_dirty(&mut self, context: &mut CoreContext) -> StatusCode {
        let code = self.base.on_added_dirty(context);
        if code != StatusCode::Ok {
            return code;
        }
        let Some(parent) = self.base.parent_mut().as_artboard_component_list_mut() else {
            return StatusCode::MissingObject;
        };
        parent.add_map_rule(self);
        StatusCode::Ok
    }
}
