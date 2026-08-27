use crate::mechanical_port::source::generated::viewmodel::data_enum_custom_base::DataEnumCustomBase;

pub struct DataEnumCustom {
    pub base: DataEnumCustomBase,
}

impl DataEnumCustom {
    pub fn enum_name(&self) -> &str {
        self.base.name()
    }
}
