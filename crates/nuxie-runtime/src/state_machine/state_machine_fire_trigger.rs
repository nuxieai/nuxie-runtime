use crate::RuntimeDataBindGraph;
use crate::artboard_data_bind::RuntimeOwnedDataContext;
use crate::view_model_cell::{RuntimeViewModelCellValue, RuntimeViewModelInstanceCells};
use nuxie_binary::{RuntimeFile, RuntimeObject};
use std::sync::Arc;

/// Retained DataBind path owned by one `StateMachineFireTrigger` definition.
#[derive(Debug, Clone)]
pub(crate) struct RuntimeStateMachineFireTriggerPath {
    pub(crate) file: Arc<RuntimeFile>,
    pub(crate) source_path: Vec<u32>,
    pub(crate) is_relative: bool,
}

impl RuntimeStateMachineFireTriggerPath {
    /// Resolve and fire the retained source exactly when the action performs.
    ///
    /// Pinned C++ asks the live DataContext for the path and fires only a
    /// trigger-valued property; unresolved or wrong-typed paths are no-ops
    /// (`state_machine_fire_trigger.cpp:13-33`;
    /// `data_context.cpp:443-462`).
    pub(crate) fn perform(
        &self,
        data_bind_facilities_ready: bool,
        owned_data_context: Option<&RuntimeOwnedDataContext>,
        file_data_context_instance: Option<&RuntimeViewModelInstanceCells>,
        data_bind_graph: &mut RuntimeDataBindGraph,
    ) -> bool {
        if !data_bind_facilities_ready {
            return false;
        }
        if let Some(data_context) = owned_data_context
            && data_context.fire_trigger(&self.file, &self.source_path, self.is_relative)
        {
            return true;
        }
        if let Some(context) = file_data_context_instance
            && let Some(cell) = context
                .cell_for_source_path(&self.source_path)
                .filter(|cell| matches!(cell.value(), RuntimeViewModelCellValue::Trigger(_)))
        {
            cell.fire_trigger();
            return true;
        }
        // File/default contexts are retained by the DataBind graph rather
        // than `owned_data_context`. The retained trigger cell is the
        // occurrence-owned C++ property identity.
        data_bind_graph.fire_retained_trigger_for_source_path(&self.source_path)
    }
}

pub(super) fn runtime_fire_trigger_path(
    file: &RuntimeFile,
    object: &RuntimeObject,
) -> Option<RuntimeStateMachineFireTriggerPath> {
    let data_bind_path = file.data_bind_path_for_referencer_object(object)?;
    let is_relative = data_bind_path
        .object
        .and_then(|path_object| path_object.bool_property("isRelative"))
        .or_else(|| object.bool_property("isDataBindPathRelative"))
        .unwrap_or(false);
    // Pinned `DataContext::getViewModelProperty(DataBindPath*)` expands a
    // claimed manifest path only for a relative DataBindPath. A non-relative
    // claim keeps its authored tokens literally (`data_context.cpp:443-462`).
    let source_path = if is_relative {
        data_bind_path.resolved_path_ids
    } else {
        data_bind_path.path_ids
    };
    Some(RuntimeStateMachineFireTriggerPath {
        file: Arc::new(file.clone()),
        source_path,
        is_relative,
    })
}
