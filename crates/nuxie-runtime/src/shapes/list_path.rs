//! Occurrence-local dynamic `ListPath` ownership.
//!
//! C++ owns one `VertexListener` (and therefore one synthetic
//! `CubicDetachedVertex`) per retained, non-null list row.  The path's vertex
//! array is only a non-owning projection of those listeners.  Keep the same
//! ownership direction here: [`RuntimeListPathState`] owns rows and drawing
//! borrows a freshly projected [`PathVertexNode`] list.

use crate::RuntimeOwnedViewModelHandle;
use crate::view_model_cell::{
    RuntimeCellDirtSink, RuntimeCellNotificationQueue, RuntimeViewModelCell,
    RuntimeViewModelCellValue,
};
use nuxie_binary::RuntimeFile;
use nuxie_graph::PathVertexNode;

const VERTEX_X: u8 = 1;
const VERTEX_Y: u8 = 2;
const IN_POINT_X: u8 = 3;
const IN_POINT_Y: u8 = 4;
const OUT_POINT_X: u8 = 5;
const OUT_POINT_Y: u8 = 6;
const ROTATION: u8 = 7;
const IN_ROTATION: u8 = 8;
const OUT_ROTATION: u8 = 9;
const DISTANCE: u8 = 10;
const IN_DISTANCE: u8 = 11;
const OUT_DISTANCE: u8 = 12;

pub(crate) fn degrees_to_radians(value: f32) -> f32 {
    value * std::f32::consts::PI / 180.0
}

pub(crate) fn point_to_distance_rotation(x: f32, y: f32) -> (f32, f32) {
    ((x * x + y * y).sqrt(), y.atan2(x))
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct RuntimeCubicDetachedVertex {
    x: f32,
    y: f32,
    in_rotation: f32,
    in_distance: f32,
    out_rotation: f32,
    out_distance: f32,
    in_valid: bool,
    out_valid: bool,
    in_point: (f32, f32),
    out_point: (f32, f32),
}

impl Default for RuntimeCubicDetachedVertex {
    fn default() -> Self {
        Self {
            x: 0.0,
            y: 0.0,
            in_rotation: 0.0,
            in_distance: 0.0,
            out_rotation: 0.0,
            out_distance: 0.0,
            in_valid: false,
            out_valid: false,
            in_point: (0.0, 0.0),
            out_point: (0.0, 0.0),
        }
    }
}

#[derive(Debug, Clone, Copy)]
enum RuntimeListPathVertexProperty {
    X,
    Y,
    InRotation,
    InDistance,
    OutRotation,
    OutDistance,
}

impl RuntimeCubicDetachedVertex {
    /// Generated-setter equivalent shared by initialization and live writes.
    /// Equality guards and side-specific cubic cache invalidation deliberately
    /// mirror `CubicDetachedVertexBase`/`CubicVertex`.
    fn set(&mut self, property: RuntimeListPathVertexProperty, value: f32) -> bool {
        let (field, invalidate_in, invalidate_out) = match property {
            RuntimeListPathVertexProperty::X => (&mut self.x, true, true),
            RuntimeListPathVertexProperty::Y => (&mut self.y, true, true),
            RuntimeListPathVertexProperty::InRotation => (&mut self.in_rotation, true, false),
            RuntimeListPathVertexProperty::InDistance => (&mut self.in_distance, true, false),
            RuntimeListPathVertexProperty::OutRotation => (&mut self.out_rotation, false, true),
            RuntimeListPathVertexProperty::OutDistance => (&mut self.out_distance, false, true),
        };
        if *field == value {
            return false;
        }
        *field = value;
        if invalidate_in {
            self.in_valid = false;
        }
        if invalidate_out {
            self.out_valid = false;
        }
        true
    }

    fn compute_in(&mut self) -> (f32, f32) {
        if !self.in_valid {
            self.in_point = (
                self.x + self.in_rotation.cos() * self.in_distance,
                self.y + self.in_rotation.sin() * self.in_distance,
            );
            self.in_valid = true;
        }
        self.in_point
    }

    fn compute_out(&mut self) -> (f32, f32) {
        if !self.out_valid {
            self.out_point = (
                self.x + self.out_rotation.cos() * self.out_distance,
                self.y + self.out_rotation.sin() * self.out_distance,
            );
            self.out_valid = true;
        }
        self.out_point
    }

    fn project(&mut self, row: usize) -> PathVertexNode {
        // Exercise the same lazy cache lifecycle even though PathVertexNode
        // carries scalar detached fields and the drawing adapter computes the
        // final controls from them.
        let _ = self.compute_in();
        let _ = self.compute_out();
        PathVertexNode {
            // Synthetic vertices are deliberately absent from the authored
            // component arena.  A stable impossible local id prevents the
            // ordinary authored-property overlay from finding one.
            local_id: usize::MAX - row,
            global_id: u32::MAX,
            type_name: "CubicDetachedVertex",
            x: self.x,
            y: self.y,
            radius: 0.0,
            rotation: 0.0,
            distance: 0.0,
            in_rotation: self.in_rotation,
            in_distance: self.in_distance,
            out_rotation: self.out_rotation,
            out_distance: self.out_distance,
            weight_local: None,
            weight_global: None,
            weight_type_name: None,
            weight_values: None,
            weight_indices: None,
            weight_in_values: None,
            weight_in_indices: None,
            weight_out_values: None,
            weight_out_indices: None,
        }
    }
}

#[derive(Debug, Clone, Copy)]
enum RuntimeListPathWrite {
    Single {
        target: RuntimeListPathVertexProperty,
        multiplier: f32,
    },
    Multi {
        first: RuntimeListPathVertexProperty,
        second: RuntimeListPathVertexProperty,
        multiplier: f32,
    },
    Point {
        distance: RuntimeListPathVertexProperty,
        rotation: RuntimeListPathVertexProperty,
    },
}

#[derive(Debug)]
struct RuntimeListPathSubscription {
    cells: Vec<Option<RuntimeViewModelCell>>,
    sink: RuntimeCellDirtSink,
    write: RuntimeListPathWrite,
}

impl RuntimeListPathSubscription {
    fn write(&self, vertex: &mut RuntimeCubicDetachedVertex) {
        let number = |cell: Option<&Option<RuntimeViewModelCell>>| match cell
            .and_then(Option::as_ref)
            .map(RuntimeViewModelCell::value)
        {
            Some(RuntimeViewModelCellValue::Number(value)) => value,
            _ => 0.0,
        };
        match self.write {
            RuntimeListPathWrite::Single { target, multiplier } => {
                vertex.set(target, number(self.cells.first()) * multiplier);
            }
            RuntimeListPathWrite::Multi {
                first,
                second,
                multiplier,
            } => {
                let value = number(self.cells.first()) * multiplier;
                vertex.set(first, value);
                vertex.set(second, value);
            }
            RuntimeListPathWrite::Point { distance, rotation } => {
                let x = number(self.cells.first());
                let y = number(self.cells.get(1));
                let (distance_value, rotation_value) = point_to_distance_rotation(x, y);
                vertex.set(distance, distance_value);
                vertex.set(rotation, rotation_value);
            }
        }
    }
}

impl Drop for RuntimeListPathSubscription {
    fn drop(&mut self) {
        for cell in self.cells.iter().flatten() {
            cell.remove_dependent(&self.sink);
        }
    }
}

#[derive(Debug)]
struct RuntimeListPathVertexListener {
    instance_identity: u64,
    instance: RuntimeOwnedViewModelHandle,
    vertex: RuntimeCubicDetachedVertex,
    subscriptions: Vec<RuntimeListPathSubscription>,
    notifications: RuntimeCellNotificationQueue,
}

impl RuntimeListPathVertexListener {
    fn new(file: &RuntimeFile, instance: RuntimeOwnedViewModelHandle) -> Self {
        let instance_identity = instance.borrow().allocation_identity();
        let mut listener = Self {
            instance_identity,
            instance,
            vertex: RuntimeCubicDetachedVertex::default(),
            subscriptions: Vec::new(),
            notifications: RuntimeCellNotificationQueue::default(),
        };
        listener.create_properties(file);
        listener
    }

    fn remap(&mut self, file: &RuntimeFile, instance: RuntimeOwnedViewModelHandle) -> bool {
        let identity = instance.borrow().allocation_identity();
        if self.instance_identity == identity {
            return false;
        }
        self.subscriptions.clear();
        self.notifications = RuntimeCellNotificationQueue::default();
        self.instance_identity = identity;
        self.instance = instance;
        self.create_properties(file);
        true
    }

    fn cell(&self, file: &RuntimeFile, symbol: u8) -> Option<RuntimeViewModelCell> {
        self.instance.borrow().number_cell_for_symbol(file, symbol)
    }

    fn push(&mut self, cells: Vec<Option<RuntimeViewModelCell>>, write: RuntimeListPathWrite) {
        if cells.iter().all(Option::is_none) {
            return;
        }
        let index = self.subscriptions.len();
        let sink = RuntimeCellDirtSink::reporting_listener(&self.notifications, index);
        for cell in cells.iter().flatten() {
            cell.add_dependent(&sink);
        }
        let subscription = RuntimeListPathSubscription { cells, sink, write };
        subscription.write(&mut self.vertex);
        self.subscriptions.push(subscription);
    }

    fn push_single(
        &mut self,
        file: &RuntimeFile,
        symbol: u8,
        target: RuntimeListPathVertexProperty,
        multiplier: f32,
    ) {
        let Some(cell) = self.cell(file, symbol) else {
            return;
        };
        self.push(
            vec![Some(cell)],
            RuntimeListPathWrite::Single { target, multiplier },
        );
    }

    fn push_multi(
        &mut self,
        file: &RuntimeFile,
        symbol: u8,
        first: RuntimeListPathVertexProperty,
        second: RuntimeListPathVertexProperty,
        multiplier: f32,
    ) {
        let Some(cell) = self.cell(file, symbol) else {
            return;
        };
        self.push(
            vec![Some(cell)],
            RuntimeListPathWrite::Multi {
                first,
                second,
                multiplier,
            },
        );
    }

    fn push_point(
        &mut self,
        file: &RuntimeFile,
        x_symbol: u8,
        y_symbol: u8,
        distance: RuntimeListPathVertexProperty,
        rotation: RuntimeListPathVertexProperty,
    ) {
        let x = self.cell(file, x_symbol);
        let y = self.cell(file, y_symbol);
        if x.is_none() && y.is_none() {
            return;
        }
        // Preserve coordinate positions so an absent coordinate reads as zero
        // without manufacturing a cell (and therefore a spurious subscription).
        self.push(
            vec![x, y],
            RuntimeListPathWrite::Point { distance, rotation },
        );
    }

    fn create_properties(&mut self, file: &RuntimeFile) {
        // C++ creation order is observable: later initial writes win, while
        // every source remains subscribed for last-source-changed wins.
        self.push_single(file, VERTEX_X, RuntimeListPathVertexProperty::X, 1.0);
        self.push_single(file, VERTEX_Y, RuntimeListPathVertexProperty::Y, 1.0);
        self.push_multi(
            file,
            ROTATION,
            RuntimeListPathVertexProperty::InRotation,
            RuntimeListPathVertexProperty::OutRotation,
            degrees_to_radians(1.0),
        );
        self.push_single(
            file,
            IN_ROTATION,
            RuntimeListPathVertexProperty::InRotation,
            degrees_to_radians(1.0),
        );
        self.push_single(
            file,
            OUT_ROTATION,
            RuntimeListPathVertexProperty::OutRotation,
            degrees_to_radians(1.0),
        );
        self.push_multi(
            file,
            DISTANCE,
            RuntimeListPathVertexProperty::InDistance,
            RuntimeListPathVertexProperty::OutDistance,
            1.0,
        );
        self.push_single(
            file,
            IN_DISTANCE,
            RuntimeListPathVertexProperty::InDistance,
            1.0,
        );
        self.push_single(
            file,
            OUT_DISTANCE,
            RuntimeListPathVertexProperty::OutDistance,
            1.0,
        );
        self.push_point(
            file,
            IN_POINT_X,
            IN_POINT_Y,
            RuntimeListPathVertexProperty::InDistance,
            RuntimeListPathVertexProperty::InRotation,
        );
        self.push_point(
            file,
            OUT_POINT_X,
            OUT_POINT_Y,
            RuntimeListPathVertexProperty::OutDistance,
            RuntimeListPathVertexProperty::OutRotation,
        );
    }

    fn flush(&mut self) -> usize {
        let mut pending = Vec::new();
        self.notifications.swap_into(&mut pending);
        let mut writes = 0;
        for index in pending {
            let Some(subscription) = self.subscriptions.get(index) else {
                continue;
            };
            subscription.sink.take_dirt();
            subscription.write(&mut self.vertex);
            writes += 1;
        }
        writes
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum RuntimeListPathInputError {
    NullList,
    NullListItem,
    WrongConverterType,
}

#[derive(Debug, Clone, PartialEq)]
pub struct RuntimeListPathDebugVertex {
    pub x: f32,
    pub y: f32,
    pub in_rotation: f32,
    pub in_distance: f32,
    pub out_rotation: f32,
    pub out_distance: f32,
}

#[derive(Debug, Clone, PartialEq)]
pub struct RuntimeListPathDebugReport {
    pub path_local: usize,
    pub reconciliation_generation: u64,
    pub instance_identities: Vec<u64>,
    pub subscription_count: usize,
    pub vertices: Vec<RuntimeListPathDebugVertex>,
}

#[derive(Debug)]
pub(crate) struct RuntimeListPathState {
    path_local: usize,
    rows: Vec<RuntimeListPathVertexListener>,
    reconciliation_generation: u64,
}

impl RuntimeListPathState {
    pub(crate) fn new(path_local: usize) -> Self {
        Self {
            path_local,
            rows: Vec::new(),
            reconciliation_generation: 0,
        }
    }

    pub(crate) fn cold_clone(&self) -> Self {
        Self::new(self.path_local)
    }

    pub(crate) fn path_local(&self) -> usize {
        self.path_local
    }

    pub(crate) fn reconciliation_generation(&self) -> u64 {
        self.reconciliation_generation
    }

    pub(crate) fn row_count(&self) -> usize {
        self.rows.len()
    }

    pub(crate) fn subscription_count(&self) -> usize {
        self.rows
            .iter()
            .map(|row| {
                row.subscriptions
                    .iter()
                    .map(|sub| sub.cells.iter().flatten().count())
                    .sum::<usize>()
            })
            .sum()
    }

    pub(crate) fn reconcile(
        &mut self,
        file: &RuntimeFile,
        rows: Option<&[Option<RuntimeOwnedViewModelHandle>]>,
    ) -> Result<(), RuntimeListPathInputError> {
        let Some(rows) = rows else {
            self.clear_invalid();
            return Err(RuntimeListPathInputError::NullList);
        };
        let old_count = self.rows.len();
        let mut filtered_index = 0;
        for row in rows {
            let Some(instance) = row else {
                // A valid list-item with a null ViewModel instance is skipped.
                continue;
            };
            if filtered_index >= old_count {
                self.rows
                    .push(RuntimeListPathVertexListener::new(file, instance.clone()));
            } else {
                self.rows[filtered_index].remap(file, instance.clone());
            }
            filtered_index += 1;
        }
        self.rows.truncate(filtered_index);
        // `ListPath::updateList` dirties unconditionally, including an
        // identical list and same-instance positional no-op.
        self.reconciliation_generation = self.reconciliation_generation.wrapping_add(1);
        Ok(())
    }

    pub(crate) fn reject_invalid(
        &mut self,
        error: RuntimeListPathInputError,
    ) -> Result<(), RuntimeListPathInputError> {
        self.clear_invalid();
        Err(error)
    }

    fn clear_invalid(&mut self) {
        // Never report success while preserving a stale path at a C++
        // precondition boundary.  Rust safely clears/unsubscribes and dirties.
        self.rows.clear();
        self.reconciliation_generation = self.reconciliation_generation.wrapping_add(1);
    }

    pub(crate) fn flush_live_changes(&mut self) -> usize {
        let writes = self.rows.iter_mut().map(|row| row.flush()).sum::<usize>();
        for _ in 0..writes {
            self.reconciliation_generation = self.reconciliation_generation.wrapping_add(1);
        }
        writes
    }

    pub(crate) fn projected_vertices(&mut self) -> Vec<PathVertexNode> {
        self.rows
            .iter_mut()
            .enumerate()
            .map(|(index, row)| row.vertex.project(index))
            .collect()
    }

    pub(crate) fn instance_identities(&self) -> Vec<u64> {
        self.rows.iter().map(|row| row.instance_identity).collect()
    }

    pub(crate) fn debug_report(&self) -> RuntimeListPathDebugReport {
        RuntimeListPathDebugReport {
            path_local: self.path_local,
            reconciliation_generation: self.reconciliation_generation,
            instance_identities: self.instance_identities(),
            subscription_count: self.subscription_count(),
            vertices: self
                .rows
                .iter()
                .map(|row| RuntimeListPathDebugVertex {
                    x: row.vertex.x,
                    y: row.vertex.y,
                    in_rotation: row.vertex.in_rotation,
                    in_distance: row.vertex.in_distance,
                    out_rotation: row.vertex.out_rotation,
                    out_distance: row.vertex.out_distance,
                })
                .collect(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::view_model_cell::RuntimeCellDirt;

    #[test]
    fn vertex_listener_units_match_pinned_list_path() {
        assert_eq!(degrees_to_radians(180.0), std::f32::consts::PI);
        assert_eq!(degrees_to_radians(-90.0), -std::f32::consts::FRAC_PI_2);
        assert!(degrees_to_radians(f32::INFINITY).is_infinite());
        assert!(degrees_to_radians(f32::NAN).is_nan());
        assert_eq!(
            point_to_distance_rotation(3.0, 4.0),
            (5.0, 4.0_f32.atan2(3.0))
        );
        assert_eq!(point_to_distance_rotation(0.0, 0.0), (0.0, 0.0));
        assert_eq!(point_to_distance_rotation(0.0, -4.0).0, 4.0);
        let negative_x = point_to_distance_rotation(-3.0, 0.0);
        assert_eq!(negative_x.0, 3.0);
        assert!((negative_x.1 - std::f32::consts::PI).abs() <= 4.0 * f32::EPSILON);
        assert!(
            point_to_distance_rotation(f32::INFINITY, 1.0)
                .0
                .is_infinite()
        );
        assert!(point_to_distance_rotation(f32::NAN, 1.0).0.is_nan());
    }

    #[test]
    fn generated_vertex_setters_invalidate_only_the_cpp_cache_side() {
        let mut vertex = RuntimeCubicDetachedVertex::default();
        vertex.compute_in();
        vertex.compute_out();
        assert!(vertex.in_valid && vertex.out_valid);
        assert!(vertex.set(RuntimeListPathVertexProperty::InDistance, 5.0));
        assert!(!vertex.in_valid && vertex.out_valid);
        vertex.compute_in();
        assert!(vertex.set(RuntimeListPathVertexProperty::OutRotation, 1.0));
        assert!(vertex.in_valid && !vertex.out_valid);
        vertex.compute_out();
        assert!(vertex.set(RuntimeListPathVertexProperty::X, 10.0));
        assert!(!vertex.in_valid && !vertex.out_valid);
        assert!(!vertex.set(RuntimeListPathVertexProperty::X, 10.0));
    }

    #[test]
    fn invalid_input_clears_stale_rows_and_dirties_without_panicking() {
        let mut state = RuntimeListPathState::new(7);
        state.rows = Vec::new();
        assert_eq!(
            state.reconcile(
                // This branch does not inspect the file because the list is
                // rejected before row traversal.
                &RuntimeFile::from_authoring_records(Vec::new()).unwrap(),
                None,
            ),
            Err(RuntimeListPathInputError::NullList)
        );
        assert_eq!(state.row_count(), 0);
        assert_eq!(state.reconciliation_generation(), 1);

        // A valid list containing only rows whose instance is absent is
        // compressed to zero output rows and still reconciles/dirties.
        assert_eq!(
            state.reconcile(
                &RuntimeFile::from_authoring_records(Vec::new()).unwrap(),
                Some(&[None, None])
            ),
            Ok(())
        );
        assert_eq!(state.row_count(), 0);
        assert_eq!(state.reconciliation_generation(), 2);

        for error in [
            RuntimeListPathInputError::NullListItem,
            RuntimeListPathInputError::WrongConverterType,
        ] {
            assert_eq!(state.reject_invalid(error), Err(error));
        }
        assert_eq!(state.row_count(), 0);
        assert_eq!(state.reconciliation_generation(), 4);
    }

    #[test]
    fn cell_dirt_is_consumed_in_actual_mutation_order() {
        let queue = RuntimeCellNotificationQueue::default();
        let first = RuntimeViewModelCell::new(RuntimeViewModelCellValue::Number(1.0));
        let second = RuntimeViewModelCell::new(RuntimeViewModelCellValue::Number(2.0));
        let first_sink = RuntimeCellDirtSink::reporting_listener(&queue, 0);
        let second_sink = RuntimeCellDirtSink::reporting_listener(&queue, 1);
        first.add_dependent(&first_sink);
        second.add_dependent(&second_sink);
        second.set_value(RuntimeViewModelCellValue::Number(3.0));
        first.set_value(RuntimeViewModelCellValue::Number(4.0));
        let mut reports = Vec::new();
        queue.swap_into(&mut reports);
        assert_eq!(reports, vec![1, 0]);
        assert!(first_sink.take_dirt().contains(RuntimeCellDirt::BINDINGS));
        assert!(second_sink.take_dirt().contains(RuntimeCellDirt::BINDINGS));
    }
}
