use crate::mechanical_port::source::{
    component_dirt::ComponentDirt,
    core_context::CoreContext,
    draw_target::DrawTarget,
    generated::draw_rules_base::{DrawRulesBase, DrawRulesBaseCallbacks},
    status_code::StatusCode,
};

#[derive(Default)]
pub struct DrawRules {
    pub base: DrawRulesBase,
    active_target: Option<*mut DrawTarget>,
}

impl DrawRules {
    pub fn active_target(&self) -> Option<&DrawTarget> {
        self.active_target.map(|target| unsafe { &*target })
    }

    pub fn on_added_dirty(&mut self, context: &mut dyn CoreContext) -> StatusCode {
        let result = self.base.base.base.base.on_added_dirty(context);
        if result != StatusCode::Ok {
            return result;
        }
        if let Some(target) = context
            .resolve(self.base.draw_target_id())
            .and_then(|object| object.as_draw_target_mut())
        {
            self.active_target = Some(target as *mut DrawTarget);
        }

        StatusCode::Ok
    }

    pub fn on_added_clean(&mut self, _context: &mut dyn CoreContext) -> StatusCode {
        StatusCode::Ok
    }

    pub fn draw_target_id_changed(&mut self) {
        self.active_target = self
            .base
            .base
            .base
            .base
            .artboard_mut()
            .and_then(|artboard| artboard.resolve(self.base.draw_target_id()))
            .and_then(|object| object.as_draw_target_mut())
            .map(|target| target as *mut DrawTarget);
        if let Some(artboard) = self.base.base.base.base.artboard_mut() {
            artboard.add_dirt(ComponentDirt::DRAW_ORDER);
        }
    }
}

impl DrawRulesBaseCallbacks for DrawRules {
    fn notify_property_changed(&mut self, property_key: u16) {
        self.base
            .base
            .base
            .base
            .base
            .notify_property_changed(property_key);
    }

    fn draw_target_id_changed(&mut self) {
        DrawRules::draw_target_id_changed(self);
    }
}
