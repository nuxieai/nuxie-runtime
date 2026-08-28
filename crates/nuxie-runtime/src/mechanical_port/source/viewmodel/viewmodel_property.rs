use crate::mechanical_port::source::{
    generated::viewmodel::viewmodel_property_base::ViewModelPropertyBase,
    importers::{import_stack::ImportStack, viewmodel_importer::ViewModelImporter},
    status_code::StatusCode,
};

#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Direction {
    None = 0,
    Input = 1,
    Output = 2,
    Both = 3,
}

#[derive(Default)]
pub struct ViewModelProperty {
    pub base: ViewModelPropertyBase,
}

impl std::ops::Deref for ViewModelProperty {
    type Target = ViewModelPropertyBase;
    fn deref(&self) -> &Self::Target {
        &self.base
    }
}
impl std::ops::DerefMut for ViewModelProperty {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.base
    }
}

impl ViewModelProperty {
    pub fn import(&mut self, import_stack: &mut ImportStack) -> StatusCode {
        let Some(importer) = import_stack.latest::<ViewModelImporter>(
            crate::mechanical_port::source::generated::viewmodel::viewmodel_base::ViewModelBase::TYPE_KEY,
        ) else {
            return StatusCode::MissingObject;
        };
        let Some(property) = self.base.base.base.base.handle() else {
            return StatusCode::MissingObject;
        };
        importer.add_property(property);
        self.base.import(import_stack)
    }
    pub fn const_name(&self) -> &str {
        self.base.name()
    }
    pub fn direction(&self) -> Direction {
        match self.base.component_props() & 3 {
            0 => Direction::None,
            1 => Direction::Input,
            2 => Direction::Output,
            _ => Direction::Both,
        }
    }
    pub fn is_input(&self) -> bool {
        matches!(self.direction(), Direction::Input | Direction::Both)
    }
    pub fn is_output(&self) -> bool {
        matches!(self.direction(), Direction::Output | Direction::Both)
    }

    pub fn is_symbol_list_index(&self) -> bool {
        self.base.is_type(
            crate::mechanical_port::source::generated::viewmodel::viewmodel_property_symbol_list_index_base::ViewModelPropertySymbolListIndexBase::TYPE_KEY,
        )
    }
}
