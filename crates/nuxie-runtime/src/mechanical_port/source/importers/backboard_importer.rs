use std::{
    any::Any,
    collections::{HashMap, HashSet},
};

use crate::mechanical_port::source::{
    artboard::Artboard, core::CoreHandle, file::RuntimeFileWeakHandle, status_code::StatusCode,
};

use super::import_stack::ImportStackObject;

pub struct BackboardImporter {
    backboard: CoreHandle,
    artboard_lookup: HashMap<i32, CoreHandle>,
    artboard_referencers: Vec<CoreHandle>,
    file_assets: Vec<CoreHandle>,
    file_asset_referencers: Vec<CoreHandle>,
    data_converters: Vec<CoreHandle>,
    data_converter_referencers: Vec<CoreHandle>,
    data_converter_group_item_referencers: Vec<CoreHandle>,
    interpolators: Vec<CoreHandle>,
    physics: Vec<CoreHandle>,
    next_artboard_id: i32,
    file: Option<RuntimeFileWeakHandle>,
}

impl BackboardImporter {
    pub fn new(backboard: CoreHandle) -> Self {
        Self {
            backboard,
            artboard_lookup: HashMap::new(),
            artboard_referencers: Vec::new(),
            file_assets: Vec::new(),
            file_asset_referencers: Vec::new(),
            data_converters: Vec::new(),
            data_converter_referencers: Vec::new(),
            data_converter_group_item_referencers: Vec::new(),
            interpolators: Vec::new(),
            physics: Vec::new(),
            next_artboard_id: 0,
            file: None,
        }
    }

    pub fn add_artboard_referencer(&mut self, artboard: CoreHandle) {
        self.artboard_referencers.push(artboard);
    }

    pub fn add_file_asset(&mut self, asset: CoreHandle) {
        self.file_assets.push(asset);
        let mut ids = HashSet::new();
        let mut next_id = 1u32;
        for file_asset in &self.file_assets {
            let asset_id = file_asset
                .with(|asset| {
                    asset
                        .as_file_asset()
                        .expect("BackboardImporter assets remain FileAsset-derived")
                        .file_asset_base()
                        .asset_id()
                })
                .expect("BackboardImporter retains live FileAssets");
            if ids.contains(&asset_id) {
                file_asset
                    .with_mut(|asset| {
                        asset
                            .as_file_asset_mut()
                            .expect("BackboardImporter assets remain FileAsset-derived")
                            .file_asset_base_mut()
                            .set_asset_id(next_id);
                    })
                    .expect("BackboardImporter retains live FileAssets");
            } else {
                ids.insert(asset_id);
                if asset_id >= next_id {
                    next_id = asset_id.wrapping_add(1);
                }
            }
        }
    }

    pub fn add_file_asset_referencer(&mut self, referencer: CoreHandle) {
        self.file_asset_referencers.push(referencer);
    }

    pub fn add_artboard(&mut self, artboard: CoreHandle) {
        #[cfg(feature = "tools")]
        artboard
            .with_downcast_mut::<Artboard, _>(|artboard| {
                artboard.set_artboard_id(self.next_artboard_id as u16)
            })
            .expect("BackboardImporter retains a live Artboard");
        self.artboard_lookup.insert(self.next_artboard_id, artboard);
        self.next_artboard_id += 1;
    }

    pub fn add_missing_artboard(&mut self) {
        self.next_artboard_id += 1;
    }

    pub fn add_data_converter(&mut self, converter: CoreHandle) {
        self.data_converters.push(converter);
    }

    pub fn add_data_converter_referencer(&mut self, data_bind: CoreHandle) {
        self.data_converter_referencers.push(data_bind);
    }

    pub fn add_data_converter_group_item_referencer(&mut self, item: CoreHandle) {
        self.data_converter_group_item_referencers.push(item);
    }

    pub fn add_interpolator(&mut self, interpolator: CoreHandle) {
        let initialized = interpolator
            .with_mut(|interpolator| interpolator.keyframe_interpolator_initialize())
            .expect("BackboardImporter retains a live interpolator");
        assert!(
            initialized,
            "BackboardImporter interpolators must implement KeyFrameInterpolator"
        );
        self.interpolators.push(interpolator);
    }

    pub fn add_physics(&mut self, physics: CoreHandle) {
        self.physics.push(physics);
    }

    pub fn add_view_model_instance(&mut self, instance: CoreHandle) {
        let Some(file) = self.file.as_ref() else {
            return;
        };
        let view_model_id = instance
            .with(|instance| {
                instance
                    .as_view_model_instance()
                    .map(|instance| instance.base.view_model_id())
            })
            .flatten()
            .expect("BackboardImporter instances remain ViewModelInstance-derived");
        if let Some(view_model) = file
            .with_file(|file| file.view_model_handle(view_model_id as usize))
            .flatten()
        {
            view_model
                .with_mut(|view_model| {
                    view_model
                        .as_view_model_mut()
                        .expect("File view model handles remain ViewModel-derived")
                        .add_instance(instance);
                })
                .expect("File retains live ViewModels");
        }
    }

    pub fn physics(&self) -> Vec<CoreHandle> {
        self.physics.clone()
    }

    pub fn assets(&mut self) -> &mut Vec<CoreHandle> {
        &mut self.file_assets
    }

    pub fn set_file(&mut self, file: Option<RuntimeFileWeakHandle>) {
        self.file = file;
    }

    pub fn file(&self) -> Option<RuntimeFileWeakHandle> {
        self.file.clone()
    }

    pub fn backboard(&self) -> CoreHandle {
        self.backboard.clone()
    }
}

impl ImportStackObject for BackboardImporter {
    fn resolve(&mut self) -> StatusCode {
        for referencer in &self.artboard_referencers {
            let referenced_artboard_id = referencer
                .with(|referencer| referencer.artboard_referencer_referenced_artboard_id())
                .expect("BackboardImporter retains live ArtboardReferencers")
                .expect("registered ArtboardReferencers expose their referenced artboard id");
            if let Some(artboard) = self.artboard_lookup.get(&referenced_artboard_id) {
                let assigned = referencer
                    .with_mut(|referencer| {
                        referencer.artboard_referencer_set_referenced_artboard(artboard.clone())
                    })
                    .expect("BackboardImporter retains live ArtboardReferencers");
                assert!(
                    assigned,
                    "an ArtboardReferencer exposing an id must accept its resolved Artboard"
                );
            }
        }

        for referencer in &self.file_asset_referencers {
            let index = referencer
                .with(|referencer| referencer.file_asset_referencer_asset_id())
                .expect("BackboardImporter retains live FileAssetReferencers")
                .expect("registered FileAssetReferencers expose their asset id")
                as usize;
            if index >= self.file_assets.len() {
                continue;
            }
            let assigned = referencer
                .with_mut(|referencer| {
                    referencer.file_asset_referencer_set_asset(self.file_assets[index].clone())
                })
                .expect("BackboardImporter retains live FileAssetReferencers");
            assert!(
                assigned,
                "a FileAssetReferencer exposing an asset id must accept its resolved FileAsset"
            );
        }

        for converter in &self.data_converters {
            let Some(interpolator_id) = converter
                .with(|converter| converter.data_converter_interpolator_id())
                .expect("BackboardImporter retains live DataConverters")
            else {
                continue;
            };
            let index = interpolator_id as usize;
            if index != usize::MAX && index < self.interpolators.len() {
                let assigned = converter
                    .with_mut(|converter| {
                        converter.data_converter_set_interpolator(self.interpolators[index].clone())
                    })
                    .expect("BackboardImporter retains live DataConverters");
                assert!(
                    assigned,
                    "a converter exposing an interpolator id must accept its interpolator"
                );
            }
        }

        for referencer in &self.data_converter_group_item_referencers {
            let index = referencer
                .with(|referencer| referencer.data_converter_group_item_converter_id())
                .expect("BackboardImporter retains live DataConverterGroupItems")
                .expect("registered DataConverterGroupItems expose their converter id")
                as usize;
            if index >= self.data_converters.len() {
                continue;
            }
            let assigned = referencer
                .with_mut(|referencer| {
                    referencer.data_converter_group_item_set_converter(
                        self.data_converters[index].clone(),
                    )
                })
                .expect("BackboardImporter retains live DataConverterGroupItems");
            assert!(
                assigned,
                "a DataConverterGroupItem exposing an id must accept its converter"
            );
        }

        for referencer in &self.data_converter_referencers {
            let index = referencer
                .with(|referencer| referencer.data_bind_converter_id())
                .expect("BackboardImporter retains live DataBinds")
                .expect("registered DataBinds expose their converter id")
                as usize;
            if index >= self.data_converters.len() {
                continue;
            }
            let converter = self.data_converters[index]
                .clone_occurrence()
                .expect("registered DataConverters must support pinned Core cloning");
            let assigned = referencer
                .with_mut(|referencer| referencer.data_bind_set_converter(converter))
                .expect("BackboardImporter retains live DataBinds");
            assert!(
                assigned,
                "a DataBind exposing a converter id must accept its cloned converter"
            );
        }
        StatusCode::Ok
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}
