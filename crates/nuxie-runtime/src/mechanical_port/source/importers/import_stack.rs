use std::{any::Any, collections::HashMap};

use crate::mechanical_port::source::status_code::StatusCode;

use super::text_asset_importer::TextAssetImporter;
use super::{
    backboard_importer::BackboardImporter,
    file_asset_importer::{FileAssetImporter, FileAssetImporterBehavior},
};
use crate::mechanical_port::source::generated::{
    assets::file_asset_base::FileAssetBase, backboard_base::BackboardBase,
};

pub trait ImportStackObject: Any {
    fn resolve(&mut self) -> StatusCode {
        StatusCode::Ok
    }

    fn read_null_object(&mut self) -> bool {
        false
    }

    fn as_any_mut(&mut self) -> &mut dyn Any;
}

pub struct ImportStack {
    latests: HashMap<u16, Box<dyn ImportStackObject>>,
    // A core type uniquely identifies each live latest entry. Replacing an
    // entry removes its earlier occurrence before resolving it.
    last_added: Vec<u16>,
    major_version: i32,
    minor_version: i32,
}

impl Default for ImportStack {
    fn default() -> Self {
        Self {
            latests: HashMap::new(),
            last_added: Vec::new(),
            major_version: 0,
            minor_version: 0,
        }
    }
}

impl ImportStack {
    pub fn latest<T: ImportStackObject>(&mut self, core_type: u16) -> Option<&mut T> {
        self.latests
            .get_mut(&core_type)?
            .as_any_mut()
            .downcast_mut::<T>()
    }

    pub fn latest_backboard_importer(&mut self) -> Option<&mut BackboardImporter> {
        self.latest::<BackboardImporter>(BackboardBase::TYPE_KEY)
    }

    pub fn latest_file_asset_importer(&mut self) -> Option<&mut dyn FileAssetImporterBehavior> {
        let object = self.latests.get_mut(&FileAssetBase::TYPE_KEY)?;
        if object.as_any_mut().is::<FileAssetImporter>() {
            return object
                .as_any_mut()
                .downcast_mut::<FileAssetImporter>()
                .map(|importer| importer as &mut dyn FileAssetImporterBehavior);
        }
        if let Some(importer) = object.as_any_mut().downcast_mut::<TextAssetImporter>() {
            return Some(importer);
        }
        None
    }

    pub fn make_latest(
        &mut self,
        core_type: u16,
        object: Option<Box<dyn ImportStackObject>>,
    ) -> StatusCode {
        if self.latests.contains_key(&core_type) {
            if let Some(index) = self.last_added.iter().position(|value| *value == core_type) {
                self.last_added.remove(index);
            }
            let code = self
                .latests
                .get_mut(&core_type)
                .expect("the existing latest entry remains live while it resolves")
                .resolve();
            if code != StatusCode::Ok {
                self.latests.remove(&core_type);
                return code;
            }
        }

        if let Some(object) = object {
            self.last_added.push(core_type);
            self.latests.insert(core_type, object);
        } else {
            self.latests.remove(&core_type);
        }
        StatusCode::Ok
    }

    pub fn resolve(&mut self) -> StatusCode {
        let mut return_code = StatusCode::Ok;
        for core_type in self.last_added.iter().rev().copied() {
            let code = self
                .latests
                .get_mut(&core_type)
                .expect("lastAdded contains only live latest entries")
                .resolve();
            if code != StatusCode::Ok {
                return_code = code;
                break;
            }
        }
        self.latests.clear();
        self.last_added.clear();
        return_code
    }

    pub fn read_null_object(&mut self) -> bool {
        for core_type in self.last_added.iter().rev().copied() {
            if self
                .latests
                .get_mut(&core_type)
                .expect("lastAdded contains only live latest entries")
                .read_null_object()
            {
                return true;
            }
        }
        false
    }

    pub fn major_version(&self) -> i32 {
        self.major_version
    }

    pub fn minor_version(&self) -> i32 {
        self.minor_version
    }

    pub fn set_version(&mut self, major: i32, minor: i32) {
        self.major_version = major;
        self.minor_version = minor;
    }
}
