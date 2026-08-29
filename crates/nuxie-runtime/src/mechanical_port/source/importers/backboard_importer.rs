use std::{
    any::Any,
    cell::RefCell,
    collections::{HashMap, HashSet},
    rc::Rc,
};

use crate::mechanical_port::source::{
    artboard::Artboard, assets::file_asset::FileAsset, core::CoreHandle,
    file::RuntimeFileWeakHandle, status_code::StatusCode,
    viewmodel::viewmodel_instance::ViewModelInstance,
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
    file_view_models: Option<Rc<RefCell<Vec<CoreHandle>>>>,
    file_view_model_instances: Option<Rc<RefCell<Vec<CoreHandle>>>>,
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
            file_view_models: None,
            file_view_model_instances: None,
        }
    }

    pub fn add_artboard_referencer(&mut self, artboard: CoreHandle) {
        self.artboard_referencers.push(artboard);
    }

    pub fn add_file_asset(&mut self, asset: &mut FileAsset) {
        let handle = asset
            .handle()
            .expect("imported FileAsset has an arena owner");
        self.file_assets.push(handle.clone());
        let mut ids = HashSet::new();
        let mut next_id = 1u32;
        let mut ensure_unique_id = |file_asset: &mut FileAsset| {
            let asset_id = file_asset.asset_id();
            if ids.contains(&asset_id) {
                file_asset.set_asset_id(next_id);
            } else {
                ids.insert(asset_id);
                if asset_id >= next_id {
                    next_id = asset_id.wrapping_add(1);
                }
            }
        };
        for file_asset in &self.file_assets {
            if *file_asset == handle {
                ensure_unique_id(asset);
            } else {
                file_asset
                    .with_mut(|asset| {
                        ensure_unique_id(
                            asset
                                .as_file_asset_mut()
                                .expect("BackboardImporter assets remain FileAsset-derived")
                                .file_asset_base_mut(),
                        );
                    })
                    .expect("BackboardImporter retains live FileAssets");
            }
        }
    }

    pub fn add_file_asset_referencer(&mut self, referencer: CoreHandle) {
        self.file_asset_referencers.push(referencer);
    }

    pub fn add_artboard(&mut self, artboard: &mut Artboard) {
        #[cfg(feature = "tools")]
        artboard.set_artboard_id(self.next_artboard_id as u16);
        let artboard = crate::mechanical_port::source::core::CoreObject::core(artboard)
            .handle()
            .expect("imported Artboard owner");
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

    pub fn add_interpolator(
        &mut self,
        interpolator: &mut dyn crate::mechanical_port::source::core::CoreObject,
    ) {
        // Import is already executing on this owner. Invoke the real virtual
        // initialize before retaining its handle, without reborrowing its slot.
        let initialized = interpolator.keyframe_interpolator_initialize();
        assert!(
            initialized,
            "BackboardImporter interpolators must implement KeyFrameInterpolator"
        );
        self.interpolators.push(
            interpolator
                .core()
                .handle()
                .expect("imported interpolator owner"),
        );
    }

    pub fn add_physics(&mut self, physics: CoreHandle) {
        self.physics.push(physics);
    }

    pub fn add_view_model_instance(&mut self, instance: &mut ViewModelInstance) {
        let Some(models) = self.file_view_models.as_ref() else {
            return;
        };
        let view_model = models
            .borrow()
            .get(instance.base.view_model_id() as usize)
            .cloned();
        if let Some(view_model) = view_model {
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

    pub(crate) fn add_file_view_model_instance(&mut self, instance: CoreHandle) {
        if let Some(instances) = &self.file_view_model_instances {
            instances.borrow_mut().push(instance);
        }
    }

    pub fn physics(&self) -> Vec<CoreHandle> {
        self.physics.clone()
    }

    pub fn assets(&mut self) -> &mut Vec<CoreHandle> {
        &mut self.file_assets
    }

    pub(crate) fn set_file(
        &mut self,
        file: RuntimeFileWeakHandle,
        view_models: Rc<RefCell<Vec<CoreHandle>>>,
        view_model_instances: Rc<RefCell<Vec<CoreHandle>>>,
    ) {
        self.file = Some(file);
        // These are the File's canonical lists, shared only to permit synchronous
        // registration while File::read and the imported instance are borrowed.
        self.file_view_models = Some(view_models);
        self.file_view_model_instances = Some(view_model_instances);
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
            let assigned = crate::mechanical_port::source::generated::core_registry::file_asset_referencer_set_asset_handle(referencer, self.file_assets[index].clone());
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
