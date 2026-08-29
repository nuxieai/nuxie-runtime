#[cfg(test)]
mod property_data_type_tests {
    use crate::runtime::{
        data_bind::{data_bind::BindSource, data_values::data_type::DataType},
        viewmodel::{
            viewmodel_instance_asset_font::ViewModelInstanceAssetFont,
            viewmodel_instance_symbol_list_index::ViewModelInstanceSymbolListIndex,
        },
    };

    #[test]
    fn property_metadata_covers_pinned_asset_font_and_symbol_list_index() {
        assert_eq!(
            BindSource::data_type(&ViewModelInstanceAssetFont::default()),
            DataType::AssetFont
        );
        assert_eq!(
            BindSource::data_type(&ViewModelInstanceSymbolListIndex::default()),
            DataType::SymbolListIndex
        );
    }
}
