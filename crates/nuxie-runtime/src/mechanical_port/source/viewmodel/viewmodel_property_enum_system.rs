use std::sync::{Mutex, MutexGuard, OnceLock};

use crate::mechanical_port::source::generated::viewmodel::viewmodel_property_enum_system_base::ViewModelPropertyEnumSystemBase;

use super::data_enum::DataEnum;

pub struct ViewModelPropertyEnumSystem {
    pub base: ViewModelPropertyEnumSystemBase,
}

static SYSTEM_DATA_ENUM: OnceLock<Mutex<DataEnum>> = OnceLock::new();

impl ViewModelPropertyEnumSystem {
    pub fn data_enum(&self) -> MutexGuard<'static, DataEnum> {
        SYSTEM_DATA_ENUM
            .get_or_init(|| Mutex::new(DataEnum::default()))
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }
}

// The upstream owner exposes one process-wide mutable DataEnum. Access is serialized here.
unsafe impl Send for DataEnum {}
