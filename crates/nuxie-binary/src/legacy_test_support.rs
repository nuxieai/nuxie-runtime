//! Source compatibility for editor builds pinned before UNIV-1788.
//!
//! The parent module compiles this file only through the non-default
//! `test-support` feature. New baseline tests and tools use the neutral fixture
//! vocabulary instead.

use anyhow::Result;

use crate::{FixtureProperty, FixtureRecord, FixtureValue, RuntimeFile};

#[doc(hidden)]
pub type AuthoringRecord = FixtureRecord;
#[doc(hidden)]
pub type AuthoringProperty = FixtureProperty;
#[doc(hidden)]
pub type AuthoringValue = FixtureValue;

impl RuntimeFile {
    #[doc(hidden)]
    pub fn from_authoring_records(records: Vec<AuthoringRecord>) -> Result<Self> {
        Self::from_fixture_records(records)
    }
}
