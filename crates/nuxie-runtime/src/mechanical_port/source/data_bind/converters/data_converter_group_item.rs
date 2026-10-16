use std::{
    cell::{Cell, RefCell},
    rc::Rc,
};

use crate::mechanical_port::source::{
    core::CoreHandle,
    generated::{
        backboard_base::BackboardBase,
        data_bind::converters::{
            data_converter_group_base::DataConverterGroupBase,
            data_converter_group_item_base::DataConverterGroupItemBase,
        },
    },
    importers::{
        backboard_importer::BackboardImporter,
        data_converter_group_importer::DataConverterGroupImporter, import_stack::ImportStack,
    },
    status_code::StatusCode,
};
/// An item's converter, shared with its group so the group's per-frame
/// advance reads it without borrowing the item.
#[derive(Default)]
pub(crate) struct GroupItemCells {
    converter: RefCell<Option<CoreHandle>>,
    // Whether the converter may advance, cached as DataBind caches its own:
    // unknown until first asked, forgotten when the converter changes.
    may_advance: Cell<Option<bool>>,
    // The group's notifier for changes to the converters under it.
    structure: RefCell<Option<Rc<crate::mechanical_port::source::data_bind::data_bind::ConverterRest>>>,
}

impl GroupItemCells {
    fn converter(&self) -> Option<CoreHandle> {
        self.converter.borrow().clone()
    }

    fn set_converter(&self, value: Option<CoreHandle>) {
        *self.converter.borrow_mut() = value;
        self.may_advance.set(None);
        let structure = self.structure.borrow().clone();
        if let Some(structure) = structure {
            structure.restructured();
        }
    }

    pub(crate) fn set_structure(
        &self,
        structure: Rc<crate::mechanical_port::source::data_bind::data_bind::ConverterRest>,
    ) {
        *self.structure.borrow_mut() = Some(structure);
    }

    /// The converter, unless it may not advance. A converter that may not
    /// advance has DataConverter's advance, which does nothing.
    pub(crate) fn advancing_converter(&self) -> Option<CoreHandle> {
        // A cached answer was given for the current converter (changing it
        // forgets the answer), so a known non-advancing converter needs no
        // handle.
        match self.may_advance.get() {
            Some(false) => return None,
            Some(true) => return self.converter(),
            None => {}
        }
        let converter = self.converter()?;
        let may_advance = self.may_advance.get().unwrap_or_else(|| {
            let may_advance = converter
                .with(|converter| {
                    converter
                        .as_data_converter_capability()
                        .is_none_or(|converter| converter.may_advance())
                })
                .unwrap_or(true);
            self.may_advance.set(Some(may_advance));
            may_advance
        });
        may_advance.then_some(converter)
    }
}

pub struct DataConverterGroupItem {
    pub base: DataConverterGroupItemBase,
    cells: Rc<GroupItemCells>,
    owns_converter: bool,
}
impl Default for DataConverterGroupItem {
    fn default() -> Self {
        Self {
            base: DataConverterGroupItemBase::default(),
            cells: Rc::default(),
            owns_converter: false,
        }
    }
}
impl Drop for DataConverterGroupItem {
    fn drop(&mut self) {
        if self.owns_converter {
            // Cloned items own their converter, unlike authored references.
            // Retire that occurrence before this item's Core base drops.
            if let Some(converter) = self.cells.converter.borrow_mut().take() {
                converter.remove_occurrence();
            }
        }
    }
}
impl DataConverterGroupItem {
    pub fn converter(&self) -> Option<CoreHandle> {
        self.cells.converter()
    }
    pub fn set_converter(&mut self, value: Option<CoreHandle>) {
        self.cells.set_converter(value)
    }
    pub(crate) fn cells(&self) -> &Rc<GroupItemCells> {
        &self.cells
    }
    pub fn set_owns_converter(&mut self, value: bool) {
        self.owns_converter = value
    }
    pub fn import(&mut self, stack: &mut ImportStack) -> StatusCode {
        let Some(backboard) = stack.latest::<BackboardImporter>(BackboardBase::TYPE_KEY) else {
            return StatusCode::MissingObject;
        };
        let Some(item) = self.base.base.handle() else {
            return StatusCode::MissingObject;
        };
        backboard.add_data_converter_group_item_referencer(item.clone());
        let Some(group) =
            stack.latest::<DataConverterGroupImporter>(DataConverterGroupBase::TYPE_KEY)
        else {
            return StatusCode::MissingObject;
        };
        group
            .group()
            .with_downcast_mut::<super::data_converter_group::DataConverterGroup, _>(|group| {
                group.add_item_with_cells(item, Rc::clone(&self.cells))
            })
            .expect("DataConverterGroupImporter retains the actual group owner");
        self.base.base.import(stack)
    }
    pub fn clone_definition(&self) -> Self {
        let mut cloned = Self::default();
        cloned.base.set_converter_id_value(self.base.converter_id());
        cloned
    }
    pub fn complete_clone(source: &CoreHandle, cloned: &CoreHandle) -> bool {
        let Some(converter) = source.with_downcast::<Self, _>(Self::converter) else {
            return false;
        };
        if let Some(converter) = converter {
            let Some(converter) = converter.clone_occurrence() else {
                return false;
            };
            return cloned
                .with_downcast_mut::<Self, _>(|cloned| {
                    cloned.cells.set_converter(Some(converter));
                    cloned.owns_converter = true;
                })
                .is_some();
        }
        true
    }
}
