// Authored ViewModel owner coordinator.
//
// The direct files intentionally share one private namespace while FL-D2 and
// FL-D3 still consume their crate-visible adapters through `view_model.rs`.
// Each implementation body lives in its one-to-one C++ correspondence file.

use super::*;

pub mod runtime;

include!("data_enum.rs");
include!("data_enum_value.rs");
include!("property_symbol_dependent.rs");
include!("viewmodel.rs");
include!("viewmodel_instance.rs");
include!("viewmodel_instance_artboard.rs");
include!("viewmodel_instance_asset.rs");
include!("viewmodel_instance_asset_blob.rs");
include!("viewmodel_instance_asset_font.rs");
include!("viewmodel_instance_asset_image.rs");
include!("viewmodel_instance_boolean.rs");
include!("viewmodel_instance_color.rs");
include!("viewmodel_instance_enum.rs");
include!("viewmodel_instance_list.rs");
include!("viewmodel_instance_list_item.rs");
include!("viewmodel_instance_number.rs");
include!("viewmodel_instance_string.rs");
include!("viewmodel_instance_symbol_list_index.rs");
include!("viewmodel_instance_trigger.rs");
include!("viewmodel_instance_value.rs");
include!("viewmodel_instance_viewmodel.rs");
include!("viewmodel_property.rs");
include!("viewmodel_property_enum.rs");
include!("viewmodel_property_enum_system.rs");
