//! Direct Rust owner for pinned C++ `src/data_bind/converters/data_converter_group_item.cpp`.

use crate::data_bind_graph::RuntimeDataBindGraphConverter;

/// Import-time owner for one non-null group converter occurrence.
///
/// Consuming the item transfers its uniquely cloned converter into the group,
/// matching C++ `DataConverterGroupItem::clone`/`ownsConverter`.
#[derive(Debug, Clone)]
pub(crate) struct RuntimeDataConverterGroupItem {
    converter: RuntimeDataBindGraphConverter,
}

impl RuntimeDataConverterGroupItem {
    pub(crate) fn import(converter: RuntimeDataBindGraphConverter) -> Self {
        Self { converter }
    }

    pub(crate) fn into_owned_converter(self) -> RuntimeDataBindGraphConverter {
        self.converter
    }
}
