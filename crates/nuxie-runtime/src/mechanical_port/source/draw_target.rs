use crate::mechanical_port::source::{
    component_dirt::ComponentDirt,
    core::CoreHandle,
    core_context::{CoreContext, StatusCode},
    draw_target_placement::DrawTargetPlacement,
    drawable::RuntimeDrawableOccurrence,
    generated::draw_target_base::{DrawTargetBase, DrawTargetBaseCallbacks},
};

#[derive(Default)]
pub struct DrawTarget {
    pub base: DrawTargetBase,
    drawable: Option<CoreHandle>,
    pub(crate) first: Option<RuntimeDrawableOccurrence>,
    pub(crate) last: Option<RuntimeDrawableOccurrence>,
}

impl DrawTargetBaseCallbacks for DrawTarget {
    fn notify_property_changed(&mut self, property_key: u16) {
        self.base
            .base
            .base
            .base
            .notify_property_changed(property_key);
    }

    fn placement_value_changed(&mut self) {
        DrawTarget::placement_value_changed(self);
    }
}

impl DrawTarget {
    pub fn drawable(&self) -> Option<CoreHandle> {
        self.drawable.clone()
    }

    pub fn on_added_dirty(&mut self, context: &mut dyn CoreContext) -> StatusCode {
        let code = self.base.on_added_dirty(context);
        if code != StatusCode::Ok {
            return code;
        }
        let Some(drawable) = context.resolve_handle(self.base.drawable_id()) else {
            return StatusCode::MissingObject;
        };
        if !drawable
            .with(|drawable| drawable.as_drawable().is_some())
            .unwrap_or(false)
        {
            return StatusCode::MissingObject;
        }
        self.drawable = Some(drawable);
        StatusCode::Ok
    }

    pub fn on_added_clean(&mut self, _context: &mut dyn CoreContext) -> StatusCode {
        StatusCode::Ok
    }

    pub fn placement(&self) -> DrawTargetPlacement {
        match self.base.placement_value() {
            0 => DrawTargetPlacement::Before,
            1 => DrawTargetPlacement::After,
            value => panic!("invalid draw target placement {value}"),
        }
    }

    pub fn placement_value_changed(&mut self) {
        if let Some(artboard) = self.base.base.artboard_handle() {
            artboard.with_mut(|artboard| {
                artboard.component_add_dirt(ComponentDirt::DRAW_ORDER, false);
            });
        }
    }

    pub fn first(&self) -> Option<RuntimeDrawableOccurrence> {
        self.first.clone()
    }

    pub fn set_first(&mut self, value: Option<RuntimeDrawableOccurrence>) {
        self.first = value;
    }

    pub fn last(&self) -> Option<RuntimeDrawableOccurrence> {
        self.last.clone()
    }

    pub fn set_last(&mut self, value: Option<RuntimeDrawableOccurrence>) {
        self.last = value;
    }
}

impl std::ops::Deref for DrawTarget {
    type Target = DrawTargetBase;

    fn deref(&self) -> &Self::Target {
        &self.base
    }
}

impl std::ops::DerefMut for DrawTarget {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.base
    }
}
