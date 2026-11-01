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
    // The converter's may_advance answer, current while epoch is the
    // container may-advance epoch. 0 is never current.
    may_advance: Cell<bool>,
    epoch: Cell<u32>,
}

impl GroupItemCells {
    fn converter(&self) -> Option<CoreHandle> {
        self.converter.borrow().clone()
    }

    fn set_converter(&self, value: Option<CoreHandle>) {
        *self.converter.borrow_mut() = value;
        self.epoch.set(0);
        crate::source::data_bind::data_bind_container::DataBindContainer::may_advance_changed();
    }

    /// The converter, unless it may not advance: then its advance is a no-op.
    /// The answer is asked once per may-advance epoch.
    pub(crate) fn advancing_converter(&self) -> Option<CoreHandle> {
        let epoch =
            crate::source::data_bind::data_bind_container::DataBindContainer::may_advance_epoch();
        if self.epoch.get() == epoch && !self.may_advance.get() {
            return None;
        }
        let converter = self.converter()?;
        if self.epoch.get() != epoch {
            let may_advance = super::data_converter::DataConverter::may_advance_handle(&converter);
            self.may_advance.set(may_advance);
            self.epoch.set(epoch);
            if !may_advance {
                return None;
            }
        }
        Some(converter)
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
