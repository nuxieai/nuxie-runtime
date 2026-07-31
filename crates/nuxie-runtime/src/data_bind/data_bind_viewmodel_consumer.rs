//! Structural child dependency owner matching C++ `DataBindViewModelConsumer`.

use crate::RuntimeViewModelPointer;

pub(crate) fn changed(previous: RuntimeViewModelPointer, next: RuntimeViewModelPointer) -> bool {
    previous != next
}
