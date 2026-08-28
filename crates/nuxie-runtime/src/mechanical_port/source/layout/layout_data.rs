use crate::mechanical_port::source::{
    core::CoreHandle,
    layout::layout_style_applier::{LayoutStyleApplier, LayoutSyncContext, YGStyle},
    layout_component::Layout,
};

#[derive(Default)]
pub struct LayoutData {
    #[cfg(feature = "tools")]
    pub children: Vec<CoreHandle>,
    pub style: YGStyle,
    pub solved_layout: Layout,
    pub has_new_layout: bool,
    pub dirty: bool,
    pub appliers: Option<Box<Vec<CoreHandle>>>,
}

impl LayoutData {
    pub fn add_applier(&mut self, applier: CoreHandle) {
        let appliers = self.appliers.get_or_insert_with(|| Box::new(Vec::new()));
        if !appliers.contains(&applier) {
            appliers.push(applier);
        }
    }
    pub fn apply_layout_styles(&self, style: &mut YGStyle, context: &LayoutSyncContext) {
        let Some(appliers) = &self.appliers else {
            return;
        };
        if appliers.is_empty() {
            return;
        }
        for applier in appliers {
            applier.with(|applier| {
                if let Some(applier) = applier.as_layout_style_applier() {
                    applier.apply_base_style(style, context);
                }
            });
        }
        for applier in appliers {
            applier.with(|applier| {
                if let Some(applier) = applier.as_layout_style_applier() {
                    applier.apply_container_style(style, context);
                }
            });
        }
        for applier in appliers {
            applier.with(|applier| {
                if let Some(applier) = applier.as_layout_style_applier() {
                    applier.apply_item_style(style, context);
                }
            });
        }
    }
    #[cfg(feature = "tools")]
    pub fn clear_children(&mut self) {
        self.children.clear();
    }
}

#[cfg(feature = "tools")]
impl Drop for LayoutData {
    fn drop(&mut self) {
        self.clear_children();
    }
}

pub type LayoutDataRef = std::rc::Rc<std::cell::RefCell<LayoutData>>;
