//! Base transition-condition dispatch.
//!
//! Pinned C++ `src/animation/transition_condition.cpp` attaches each condition
//! to its containing transition. Rust performs that attachment while building
//! `RuntimeStateTransition`; this module keeps the shared condition type at the
//! matching base-owner boundary while specialized behavior lives in the
//! filename-corresponding modules.

pub(super) use super::transition_viewmodel_condition::RuntimeTransitionCondition;
