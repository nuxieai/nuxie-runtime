//! include/rive/data_bind/host_data_context_consumer.hpp and paired source.
use crate::source::{
    core::CoreHandle,
    data_bind::data_context::RuntimeDataContextHandle,
    text::{text::Text, text_style_global::TextStyleGlobal},
};

/// Retains the consumer identity, releasing its payload borrow before callbacks.
#[derive(Clone)]
pub struct HostDataContextConsumer {
    owner: CoreHandle,
    global: bool,
}
impl HostDataContextConsumer {
    pub fn from(owner: &CoreHandle) -> Option<Self> {
        let global = match owner.core_type()? {
            TextStyleGlobal::TYPE_KEY => true,
            Text::TYPE_KEY => false,
            _ => return None,
        };
        Some(Self { owner: owner.clone(), global })
    }
    pub fn host_data_context(&self, value: Option<RuntimeDataContextHandle>) {
        if self.global {
            TextStyleGlobal::host_data_context_occurrence(&self.owner, value);
        } else {
            Text::host_data_context_occurrence(&self.owner, value);
        }
    }
    pub fn relink_host_data_context(&self) {
        if self.global {
            TextStyleGlobal::relink_host_data_context_occurrence(&self.owner);
        } else {
            Text::relink_host_data_context_occurrence(&self.owner);
        }
    }
    pub fn update_hosted_data_binds(&self) {
        if self.global {
            TextStyleGlobal::update_hosted_data_binds_occurrence(&self.owner);
        } else {
            Text::update_hosted_data_binds_occurrence(&self.owner);
        }
    }
}
