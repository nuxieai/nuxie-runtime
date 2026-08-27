use std::{
    any::Any,
    collections::{HashMap, HashSet},
    ptr::NonNull,
};

use crate::mechanical_port::source::{
    animation::keyframe_interpolator::KeyFrameInterpolator,
    artboard::Artboard,
    artboard_referencer::ArtboardReferencer,
    assets::{file_asset::FileAsset, file_asset_referencer::FileAssetReferencer},
    backboard::Backboard,
    constraints::scrolling::scroll_physics::ScrollPhysics,
    data_bind::{
        converters::{
            data_converter::DataConverter, data_converter_group_item::DataConverterGroupItem,
        },
        data_bind::DataBind,
    },
    file::File,
    refcnt::RiveRc,
    status_code::StatusCode,
    viewmodel::viewmodel_instance::ViewModelInstance,
};

use super::import_stack::ImportStackObject;

pub struct BackboardImporter {
    backboard: NonNull<Backboard>,
    artboard_lookup: HashMap<i32, NonNull<Artboard>>,
    artboard_referencers: Vec<NonNull<ArtboardReferencer>>,
    file_assets: Vec<RiveRc<FileAsset>>,
    file_asset_referencers: Vec<NonNull<FileAssetReferencer>>,
    data_converters: Vec<NonNull<DataConverter>>,
    data_converter_referencers: Vec<NonNull<DataBind>>,
    data_converter_group_item_referencers: Vec<NonNull<DataConverterGroupItem>>,
    interpolators: Vec<NonNull<KeyFrameInterpolator>>,
    physics: Vec<NonNull<ScrollPhysics>>,
    next_artboard_id: i32,
    file: Option<NonNull<File>>,
}

impl BackboardImporter {
    pub fn new(backboard: NonNull<Backboard>) -> Self {
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

    pub fn add_artboard_referencer(&mut self, artboard: NonNull<ArtboardReferencer>) {
        self.artboard_referencers.push(artboard);
    }

    pub fn add_file_asset(&mut self, asset: RiveRc<FileAsset>) {
        self.file_assets.push(asset);
        let mut ids = HashSet::new();
        let mut next_id = 1u32;
        for file_asset in &mut self.file_assets {
            if ids.contains(&file_asset.asset_id()) {
                file_asset.set_asset_id(next_id);
            } else {
                ids.insert(file_asset.asset_id());
                if file_asset.asset_id() >= next_id {
                    next_id = file_asset.asset_id() + 1;
                }
            }
        }
    }

    pub fn add_file_asset_referencer(&mut self, referencer: NonNull<FileAssetReferencer>) {
        self.file_asset_referencers.push(referencer);
    }

    pub fn add_artboard(&mut self, mut artboard: NonNull<Artboard>) {
        #[cfg(feature = "rive_tools")]
        unsafe {
            artboard
                .as_mut()
                .set_artboard_id(self.next_artboard_id as u16)
        };
        self.artboard_lookup.insert(self.next_artboard_id, artboard);
        self.next_artboard_id += 1;
    }

    pub fn add_missing_artboard(&mut self) {
        self.next_artboard_id += 1;
    }

    pub fn add_data_converter(&mut self, converter: NonNull<DataConverter>) {
        self.data_converters.push(converter);
    }

    pub fn add_data_converter_referencer(&mut self, data_bind: NonNull<DataBind>) {
        self.data_converter_referencers.push(data_bind);
    }

    pub fn add_data_converter_group_item_referencer(
        &mut self,
        item: NonNull<DataConverterGroupItem>,
    ) {
        self.data_converter_group_item_referencers.push(item);
    }

    pub fn add_interpolator(&mut self, mut interpolator: NonNull<KeyFrameInterpolator>) {
        unsafe { interpolator.as_mut().initialize() };
        self.interpolators.push(interpolator);
    }

    pub fn add_physics(&mut self, physics: NonNull<ScrollPhysics>) {
        self.physics.push(physics);
    }

    pub fn add_view_model_instance(&mut self, instance: NonNull<ViewModelInstance>) {
        let Some(mut file) = self.file else {
            return;
        };
        let instance_ref = unsafe { instance.as_ref() };
        if let Some(view_model) = unsafe {
            file.as_mut()
                .view_model(instance_ref.view_model_id() as usize)
        } {
            unsafe { view_model.as_mut().add_instance(instance) };
        }
    }

    pub fn physics(&self) -> Vec<NonNull<ScrollPhysics>> {
        self.physics.clone()
    }

    pub fn assets(&mut self) -> &mut Vec<RiveRc<FileAsset>> {
        &mut self.file_assets
    }

    pub fn set_file(&mut self, file: Option<NonNull<File>>) {
        self.file = file;
    }

    pub fn file(&self) -> Option<NonNull<File>> {
        self.file
    }

    pub fn backboard(&self) -> NonNull<Backboard> {
        self.backboard
    }
}

impl ImportStackObject for BackboardImporter {
    fn resolve(&mut self) -> StatusCode {
        for referencer in self.artboard_referencers.iter_mut() {
            let referencer = unsafe { referencer.as_mut() };
            if let Some(artboard) = self
                .artboard_lookup
                .get(&referencer.referenced_artboard_id())
                .copied()
            {
                referencer.set_referenced_artboard(artboard);
            }
        }

        for referencer in self.file_asset_referencers.iter_mut() {
            let referencer = unsafe { referencer.as_mut() };
            let index = referencer.asset_id() as usize;
            if index >= self.file_assets.len() {
                continue;
            }
            referencer.set_asset(Some(self.file_assets[index].clone()));
        }

        for converter in self.data_converters.iter_mut() {
            let converter = unsafe { converter.as_mut() };
            if let Some(range_mapper) = converter.as_range_mapper_mut() {
                let id = range_mapper.interpolator_id() as usize;
                if id != usize::MAX && id < self.interpolators.len() {
                    range_mapper.set_interpolator(self.interpolators[id]);
                }
            } else if let Some(interpolator_converter) = converter.as_interpolator_mut() {
                let id = interpolator_converter.interpolator_id() as usize;
                if id != usize::MAX && id < self.interpolators.len() {
                    interpolator_converter.set_interpolator(self.interpolators[id]);
                }
            }
        }

        for referencer in self.data_converter_group_item_referencers.iter_mut() {
            let referencer = unsafe { referencer.as_mut() };
            let index = referencer.converter_id() as usize;
            if index >= self.data_converters.len() {
                continue;
            }
            referencer.set_converter(self.data_converters[index]);
        }

        for referencer in self.data_converter_referencers.iter_mut() {
            let referencer = unsafe { referencer.as_mut() };
            let index = referencer.converter_id() as usize;
            if index >= self.data_converters.len() {
                continue;
            }
            let clone = unsafe { self.data_converters[index].as_ref().clone_converter() };
            referencer.set_converter(clone);
        }
        StatusCode::Ok
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}
