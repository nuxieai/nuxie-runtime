use crate::mechanical_port::source::{
    component::ComponentDirt,
    core_context::{CoreContext, StatusCode},
    draw_target_placement::DrawTargetPlacement,
    drawable::Drawable,
    generated::draw_target_base::DrawTargetBase,
};

pub struct DrawTarget {
    pub base: DrawTargetBase,
    drawable: Option<*mut Drawable>,
    pub(crate) first: Option<*mut Drawable>,
    pub(crate) last: Option<*mut Drawable>,
}

impl DrawTarget {
    pub fn drawable(&self) -> Option<&Drawable> {
        self.drawable.map(|drawable| unsafe { &*drawable })
    }

    pub fn on_added_dirty(&mut self, context: &mut CoreContext) -> StatusCode {
        let code = self.base.on_added_dirty(context);
        if code != StatusCode::Ok {
            return code;
        }
        let Some(core_object) = context.resolve(self.base.drawable_id()) else {
            return StatusCode::MissingObject;
        };
        let Some(drawable) = core_object.as_drawable_mut() else {
            return StatusCode::MissingObject;
        };
        self.drawable = Some(drawable as *mut Drawable);
        StatusCode::Ok
    }

    pub fn on_added_clean(&mut self, _context: &mut CoreContext) -> StatusCode {
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
        self.base
            .artboard_mut()
            .add_dirt(ComponentDirt::DRAW_ORDER, false);
    }
}
