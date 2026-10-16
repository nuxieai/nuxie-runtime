use crate::mechanical_port::source::{
    core::CoreHandle,
    layout::{
        layout_style_applier::{LayoutSyncContext, rive_default_style},
        layout_tree::LayoutNodeLink,
    },
    layout_component::Layout,
};
use taffy::prelude::Style;

pub struct LayoutData {
    #[cfg(feature = "tools")]
    pub children: Vec<CoreHandle>,
    /// The node's CSS style, written by the style appliers. The owning layout
    /// root's Taffy tree receives a copy only when it changes.
    style: Style,
    pub solved_layout: Layout,
    pub has_new_layout: bool,
    /// Queued for the owning layout tree since its last solve.
    queued: bool,
    /// The solved layout may change: the tree must discard cached results.
    invalidated: bool,
    /// `style` was resynced and may differ from the owning tree's copy.
    style_synced: bool,
    pub appliers: Option<Box<Vec<CoreHandle>>>,
    node: Option<LayoutNodeLink>,
}

impl Default for LayoutData {
    fn default() -> Self {
        let style = rive_default_style();
        Self {
            #[cfg(feature = "tools")]
            children: Vec::new(),
            style,
            // Positions are zero and dimensions undefined until the first solve.
            solved_layout: Layout::new(0.0, 0.0, f32::NAN, f32::NAN),
            has_new_layout: true,
            queued: false,
            invalidated: false,
            style_synced: false,
            appliers: None,
            node: None,
        }
    }
}

/// What a layout tree must do for a queued node.
pub(crate) struct LayoutDataUpdate<'a> {
    /// The resynced style, to compare against the tree's copy.
    pub style: Option<&'a Style>,
    pub invalidated: bool,
}

impl LayoutData {
    pub fn style(&self) -> &Style {
        &self.style
    }
    /// Take the style out for the appliers; [`LayoutData::set_style`] returns it.
    pub fn take_style(&mut self) -> Style {
        std::mem::take(&mut self.style)
    }
    /// Store a freshly synced style. The node is always revisited by the next
    /// solve; its tree discards cached results only if the style changed.
    pub fn set_style(&mut self, style: Style) {
        self.style = style;
        self.style_synced = true;
        self.enqueue();
    }
    /// The node's measured content or placement changed.
    pub fn mark_dirty(&mut self) {
        self.invalidated = true;
        self.enqueue();
    }
    /// Whether the node changed since its layout tree last solved.
    pub fn is_dirty(&self) -> bool {
        self.queued
    }
    #[cfg(test)]
    pub(crate) fn clear_dirty_for_test(&mut self) {
        self.queued = false;
        self.invalidated = false;
        self.style_synced = false;
    }
    fn enqueue(&mut self) {
        if self.queued {
            return;
        }
        self.queued = true;
        if let Some(node) = &self.node {
            node.enqueue();
        }
    }
    pub(crate) fn node(&self) -> Option<&LayoutNodeLink> {
        self.node.as_ref()
    }
    /// Attach to a tree node; the tree receives the current style itself.
    pub(crate) fn link(&mut self, node: LayoutNodeLink) -> Style {
        self.node = Some(node);
        self.queued = false;
        self.invalidated = false;
        self.style_synced = false;
        self.style.clone()
    }
    pub(crate) fn unlink(&mut self, node: &LayoutNodeLink) {
        if self.node.as_ref() == Some(node) {
            self.node = None;
        }
    }
    /// Consume the queued state for `node`'s tree.
    pub(crate) fn take_update(&mut self, node: &LayoutNodeLink) -> Option<LayoutDataUpdate<'_>> {
        if !self.queued || self.node.as_ref() != Some(node) {
            return None;
        }
        self.queued = false;
        let synced = std::mem::take(&mut self.style_synced);
        Some(LayoutDataUpdate {
            style: synced.then_some(&self.style),
            invalidated: std::mem::take(&mut self.invalidated),
        })
    }
    pub fn add_applier(&mut self, applier: CoreHandle) {
        let appliers = self.appliers.get_or_insert_with(|| Box::new(Vec::new()));
        if !appliers.contains(&applier) {
            appliers.push(applier);
        }
    }
    pub fn apply_layout_styles(&self, style: &mut Style, context: &LayoutSyncContext) {
        let Some(appliers) = &self.appliers else {
            return;
        };
        if appliers.is_empty() {
            return;
        }
        for applier in appliers.iter() {
            applier.with(|applier| {
                if let Some(applier) = applier.as_layout_style_applier() {
                    applier.apply_base_style(style, context);
                }
            });
        }
        for applier in appliers.iter() {
            applier.with(|applier| {
                if let Some(applier) = applier.as_layout_style_applier() {
                    applier.apply_container_style(style, context);
                }
            });
        }
        for applier in appliers.iter() {
            applier.with(|applier| {
                if let Some(applier) = applier.as_layout_style_applier() {
                    applier.apply_item_style(style, context);
                }
            });
        }
        for applier in appliers.iter() {
            applier.with(|applier| {
                if let Some(applier) = applier.as_layout_style_applier() {
                    applier.apply_placement_style(style, context);
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
