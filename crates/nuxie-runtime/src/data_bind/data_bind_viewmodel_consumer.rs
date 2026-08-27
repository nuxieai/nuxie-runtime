//! Direct owner for C++ `DataBindViewModelConsumer`.

/// Mechanical translation of `DataBindViewModelConsumer::from`.
///
/// Pinned C++ switches on the concrete `coreType`, so this deliberately does
/// not use schema inheritance and recognizes only `ViewModelInstanceViewModel`.
pub(crate) fn from(type_name: Option<&str>) -> bool {
    type_name == Some("ViewModelInstanceViewModel")
}

// The primary header's pure-virtual `updateViewModel` contract is represented
// by the concrete `ViewModelInstanceViewModel` target-application owner.
