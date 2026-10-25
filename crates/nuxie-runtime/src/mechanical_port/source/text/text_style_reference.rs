use super::text_style_on_demand::TextStyleOnDemand;
use crate::source::{
    animation::state_machine_instance::RuntimeStateMachineInstanceHandle, core::CoreHandle,
    generated::text::text_style_reference_base::TextStyleReferenceBase,
};
use std::{
    cell::RefCell,
    rc::{Rc, Weak},
};

#[derive(Default)]
pub struct TextStyleReference {
    pub base: TextStyleReferenceBase,
}
impl std::ops::Deref for TextStyleReference {
    type Target = TextStyleReferenceBase;
    fn deref(&self) -> &Self::Target {
        &self.base
    }
}
impl std::ops::DerefMut for TextStyleReference {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.base
    }
}
impl TextStyleReference {
    pub const TYPE_KEY: u16 = TextStyleReferenceBase::TYPE_KEY;
}

#[derive(Clone)]
pub enum TextStyleReferenceHandle {
    Authored(CoreHandle),
    OnDemand(Rc<RefCell<TextStyleOnDemand>>),
}
#[derive(Clone)]
pub enum TextStyleReferenceWeak {
    Authored(CoreHandle),
    OnDemand(Weak<RefCell<TextStyleOnDemand>>),
}
impl TextStyleReferenceWeak {
    pub fn upgrade(&self) -> Option<TextStyleReferenceHandle> {
        match self {
            Self::Authored(h) => h
                .is_alive()
                .then(|| TextStyleReferenceHandle::Authored(h.clone())),
            Self::OnDemand(h) => h.upgrade().map(TextStyleReferenceHandle::OnDemand),
        }
    }
}
impl PartialEq for TextStyleReferenceHandle {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Authored(a), Self::Authored(b)) => a == b,
            (Self::OnDemand(a), Self::OnDemand(b)) => Rc::ptr_eq(a, b),
            _ => false,
        }
    }
}
impl Eq for TextStyleReferenceHandle {}
impl std::fmt::Debug for TextStyleReferenceHandle {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Authored(h) => f.debug_tuple("Authored").field(h).finish(),
            Self::OnDemand(h) => f.debug_tuple("OnDemand").field(&Rc::as_ptr(h)).finish(),
        }
    }
}
impl TextStyleReferenceHandle {
    pub fn authored(&self) -> Option<CoreHandle> {
        if let Self::Authored(h) = self {
            Some(h.clone())
        } else {
            None
        }
    }
    pub fn downgrade(&self) -> TextStyleReferenceWeak {
        match self {
            Self::Authored(h) => TextStyleReferenceWeak::Authored(h.clone()),
            Self::OnDemand(h) => TextStyleReferenceWeak::OnDemand(Rc::downgrade(h)),
        }
    }
    pub fn resolved_paint(&self) -> Option<CoreHandle> {
        match self {
            Self::Authored(h) => h
                .with(|o| {
                    if o.as_text_style_paint().is_some() {
                        Some(h.clone())
                    } else {
                        o.as_text_style_global().and_then(|g| g.resolved_paint())
                    }
                })
                .flatten(),
            Self::OnDemand(h) => h.borrow().resolved_paint(),
        }
    }
    pub fn listens_to_runs(&self) -> bool {
        match self {
            Self::Authored(h) => h
                .with(|o| {
                    o.as_text_style_global()
                        .is_some_and(|g| g.listens_to_runs())
                })
                .unwrap_or(false),
            Self::OnDemand(h) => h.borrow().listens_to_runs(),
        }
    }
    pub fn run_state_machine(&self) -> Option<RuntimeStateMachineInstanceHandle> {
        match self {
            Self::Authored(h) => h
                .with(|o| o.as_text_style_global().and_then(|g| g.run_state_machine()))
                .flatten(),
            Self::OnDemand(h) => h.borrow().run_state_machine(),
        }
    }
    pub fn list_item(&self, value: Option<CoreHandle>) {
        match self {
            Self::Authored(h) => {
                super::text_style_global::TextStyleGlobal::list_item_occurrence(h, value)
            }
            Self::OnDemand(h) => h.borrow_mut().list_item(value),
        }
    }
}
impl From<CoreHandle> for TextStyleReferenceHandle {
    fn from(value: CoreHandle) -> Self {
        Self::Authored(value)
    }
}
