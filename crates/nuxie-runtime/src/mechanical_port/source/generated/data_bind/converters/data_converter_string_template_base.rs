use crate::source::core::binary_reader::BinaryReader;
use crate::source::data_bind::converters::data_converter::DataConverter;
pub struct DataConverterStringTemplateBase {
    pub base: DataConverter,
    open_delimiter: String,
    close_delimiter: String,
}
impl Default for DataConverterStringTemplateBase {
    fn default() -> Self {
        Self {
            base: Default::default(),
            open_delimiter: "${".to_owned(),
            close_delimiter: "}".to_owned(),
        }
    }
}
impl DataConverterStringTemplateBase {
    pub const TYPE_KEY: u16 = 177;
    pub fn is_type_of(key: u16) -> bool {
        key == Self::TYPE_KEY
            || crate::source::generated::core_type_tree::has_ancestor(Self::TYPE_KEY, key)
    }
    pub fn core_type(&self) -> u16 {
        Self::TYPE_KEY
    }
    pub const OPEN_DELIMITER_PROPERTY_KEY: u16 = 485;
    pub fn open_delimiter(&self) -> &str {
        &self.open_delimiter
    }
    pub(crate) fn set_open_delimiter_value(&mut self, value: String) -> bool {
        if self.open_delimiter == value {
            return false;
        }
        self.open_delimiter = value;
        true
    }
    pub const CLOSE_DELIMITER_PROPERTY_KEY: u16 = 486;
    pub fn close_delimiter(&self) -> &str {
        &self.close_delimiter
    }
    pub(crate) fn set_close_delimiter_value(&mut self, value: String) -> bool {
        if self.close_delimiter == value {
            return false;
        }
        self.close_delimiter = value;
        true
    }
    pub fn copy(&mut self, other: &Self) {
        self.open_delimiter = other.open_delimiter.clone();
        self.close_delimiter = other.close_delimiter.clone();
        self.base.copy(&other.base);
    }
    pub fn deserialize(&mut self, key: u16, reader: &mut BinaryReader<'_>) -> bool {
        match key {
            485 => {
                self.open_delimiter =
                    crate::source::core::field_types::core_string_type::CoreStringType::deserialize(
                        reader,
                    );
                true
            }
            486 => {
                self.close_delimiter =
                    crate::source::core::field_types::core_string_type::CoreStringType::deserialize(
                        reader,
                    );
                true
            }
            _ => {
                let mut callbacks = crate::source::data_bind::converters::data_converter_string_template::DataConverterStringTemplate::default();
                self.base.deserialize(key, reader, &mut callbacks)
            }
        }
    }
}
impl std::ops::Deref for DataConverterStringTemplateBase {
    type Target = DataConverter;
    fn deref(&self) -> &Self::Target {
        &self.base
    }
}
impl std::ops::DerefMut for DataConverterStringTemplateBase {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.base
    }
}
