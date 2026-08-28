use crate::mechanical_port::source::{
    component_dirt::ComponentDirt,
    core::CoreHandle,
    core_context::CoreContext,
    generated::draw_rules_base::{DrawRulesBase, DrawRulesBaseCallbacks},
    status_code::StatusCode,
};

#[derive(Default)]
pub struct DrawRules {
    pub base: DrawRulesBase,
    active_target: Option<CoreHandle>,
}

impl DrawRules {
    pub fn active_target(&self) -> Option<CoreHandle> {
        self.active_target.clone()
    }

    pub fn on_added_dirty(&mut self, context: &mut dyn CoreContext) -> StatusCode {
        let result = self.base.base.base.base.on_added_dirty(context);
        if result != StatusCode::Ok {
            return result;
        }
        if let Some(target) = context.resolve_handle(self.base.draw_target_id())
            && target.is_type_of(
                crate::mechanical_port::source::generated::draw_target_base::DrawTargetBase::TYPE_KEY,
            )
        {
            self.active_target = Some(target);
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
            .artboard_handle()
            .and_then(|artboard| {
                artboard
                    .with_downcast::<crate::mechanical_port::source::artboard::Artboard, _>(
                        |artboard| artboard.resolve_handle(self.base.draw_target_id()),
                    )
                    .flatten()
            })
            .filter(|target| {
                target.is_type_of(
                    crate::mechanical_port::source::generated::draw_target_base::DrawTargetBase::TYPE_KEY,
                )
            });
        if let Some(artboard) = self.base.base.base.base.artboard_handle() {
            artboard.with_mut(|artboard| {
                artboard.component_add_dirt(ComponentDirt::DRAW_ORDER, false);
            });
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

impl std::ops::Deref for DrawRules {
    type Target = DrawRulesBase;

    fn deref(&self) -> &Self::Target {
        &self.base
    }
}

impl std::ops::DerefMut for DrawRules {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.base
    }
}
