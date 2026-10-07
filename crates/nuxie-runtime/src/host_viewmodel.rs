//! Public host adapters over the translated runtime's retained owners.
use crate::mechanical_port::source::{
    core::CoreHandle,
    data_bind::data_context::{DataContext, RuntimeDataContextHandle},
    file::RuntimeFileHandle,
    viewmodel::viewmodel_instance::ViewModelInstance,
};
pub use crate::view_model_cell::{
    RuntimeBlobAsset, RuntimeBlobAssetValue, RuntimeFontAssetValue, RuntimeViewModelChangeCapture,
    RuntimeViewModelChangeLimitExceeded, RuntimeViewModelChangeValue,
};
use std::{
    cell::{Ref, RefCell, RefMut},
    collections::{BTreeMap, BTreeSet},
    rc::Rc,
    sync::Arc,
};
mod context;
mod instance;
pub(crate) use instance::identity as view_model_identity;
mod runtime;
mod source_handles;
mod transactions;
mod value_policy;
mod value_rules;
pub use context::*;
pub use instance::*;
pub use runtime::*;
pub use source_handles::*;
pub use transactions::*;
pub(crate) use transactions::{
    capture_native_change, capture_native_list_change, capture_native_view_model_change,
    capture_unchanged_native_write,
};
pub(crate) use value_policy::capture_initial_policy_owner;
pub use value_policy::*;
pub use value_rules::*;
