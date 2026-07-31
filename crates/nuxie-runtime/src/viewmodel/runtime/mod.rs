// Runtime ViewModel facade coordinator.
//
// As with the authored ViewModel family, each C++ implementation has one
// direct Rust correspondence file while the implementations share a private
// namespace. Public consumers continue to enter through `view_model.rs`.

use super::*;
pub use super::{RuntimeBindableArtboard, RuntimeViewModelImage};

include!("viewmodel_instance_value_runtime.rs");
include!("viewmodel_instance_number_runtime.rs");
include!("viewmodel_instance_string_runtime.rs");
include!("viewmodel_instance_boolean_runtime.rs");
include!("viewmodel_instance_color_runtime.rs");
include!("viewmodel_instance_enum_runtime.rs");
include!("viewmodel_instance_trigger_runtime.rs");
include!("viewmodel_instance_list_index_runtime.rs");
include!("viewmodel_instance_asset_image_runtime.rs");
include!("viewmodel_instance_asset_font_runtime.rs");
include!("viewmodel_instance_artboard_runtime.rs");
include!("viewmodel_instance_list_runtime.rs");
include!("viewmodel_instance_runtime.rs");
include!("viewmodel_runtime.rs");
