//! Data-oriented scheduling state for one initialized Artboard.
//!
//! Upstream keeps Component::m_Dirt, the dependents list and the onDirty hook
//! on each heap object and walks them through pointers. Here the graph members'
//! dirt, their dependents (by graph order) and their onDirty/update dispatch
//! live in dense arrays indexed by graph order. The edges are fixed once
//! Artboard::initialize has sorted them, so the arrays are built once. The
//! addDirt cascade and the updateComponents walk run over these arrays and only
//! touch a component's payload when its type has real onDirty or update work.
//!
//! Members are the dependency order's entries except the Artboard root, whose
//! dirt shares a cell with its scheduling bits, and occurrences outside the
//! order (DrawTargets, cycle remnants); those keep the occurrence-handle path.

use std::cell::Cell;
use std::rc::Rc;

use crate::mechanical_port::source::{
    artboard::RuntimeArtboardDirtyHandle, component::ComponentOccurrenceHandle,
    component_dirt::ComponentDirt, core::CoreHandle,
};

type UpdateHandler = fn(&CoreHandle, ComponentDirt) -> bool;

#[derive(Clone, Copy)]
enum Entry {
    /// The root or another occurrence whose dirt lives outside the graph.
    Outside,
    /// An authored Component whose onDirty is Component's empty default.
    Noop(Option<UpdateHandler>),
    /// An authored Component whose onDirty runs after the payload borrow.
    Handler(fn(&CoreHandle, ComponentDirt), Option<UpdateHandler>),
    /// An authored Component whose onDirty runs inside the payload borrow.
    Receiver(Option<UpdateHandler>),
    /// Runtime helpers (PathComposer, TextVariationHelper).
    Helper,
}

enum Dependent {
    Member(u32),
    Outside(ComponentOccurrenceHandle),
}

pub(crate) struct ComponentGraph {
    dirt: Box<[Cell<ComponentDirt>]>,
    entries: Box<[Entry]>,
    occurrences: Box<[ComponentOccurrenceHandle]>,
    dependents_start: Box<[u32]>,
    dependents: Box<[Dependent]>,
    artboard: RuntimeArtboardDirtyHandle,
}

impl ComponentGraph {
    /// Build from a sorted dependency order and bind every member's
    /// Component dirt to the graph, keeping its current bits.
    pub(crate) fn build(
        root: &CoreHandle,
        order: &[ComponentOccurrenceHandle],
        artboard: RuntimeArtboardDirtyHandle,
    ) -> Rc<Self> {
        let index_of = |occurrence: &ComponentOccurrenceHandle| -> Option<u32> {
            // Graph order is the dependency-order index for every member.
            let order_index = occurrence
                .with_component(|component| component.graph_order())
                .filter(|&value| value != u32::MAX)?;
            (order.get(order_index as usize) == Some(occurrence)).then_some(order_index)
        };
        let mut entries = Vec::with_capacity(order.len());
        let mut dirt = Vec::with_capacity(order.len());
        let mut dependents_start = Vec::with_capacity(order.len() + 1);
        let mut dependents = Vec::new();
        for occurrence in order {
            dependents_start.push(dependents.len() as u32);
            let entry = match occurrence {
                ComponentOccurrenceHandle::Authored(handle) if handle == root => Entry::Outside,
                ComponentOccurrenceHandle::Authored(handle) => handle
                    .with(|object| {
                        let update = object.component_update_handler();
                        if let Some(handler) = object.component_on_dirty_handler() {
                            Entry::Handler(handler, update)
                        } else if object.component_on_dirty_is_noop() {
                            Entry::Noop(update)
                        } else {
                            Entry::Receiver(update)
                        }
                    })
                    .unwrap_or(Entry::Outside),
                _ => Entry::Helper,
            };
            dirt.push(Cell::new(
                occurrence
                    .with_component(|component| component.dirt())
                    .unwrap_or(ComponentDirt::NONE),
            ));
            entries.push(entry);
            for dependent in occurrence
                .with_component(|component| component.dependents_snapshot())
                .unwrap_or_default()
                .iter()
            {
                dependents.push(match index_of(dependent) {
                    Some(index) => Dependent::Member(index),
                    None => Dependent::Outside(dependent.clone()),
                });
            }
        }
        dependents_start.push(dependents.len() as u32);
        // Members are classified only once every entry is known: a dependent
        // in the order but Outside (the root) keeps the occurrence path.
        for dependent in &mut dependents {
            if let Dependent::Member(index) = *dependent
                && matches!(entries[index as usize], Entry::Outside)
            {
                *dependent = Dependent::Outside(order[index as usize].clone());
            }
        }
        let graph = Rc::new(Self {
            dirt: dirt.into_boxed_slice(),
            entries: entries.into_boxed_slice(),
            occurrences: order.to_vec().into_boxed_slice(),
            dependents_start: dependents_start.into_boxed_slice(),
            dependents: dependents.into_boxed_slice(),
            artboard,
        });
        for (index, occurrence) in order.iter().enumerate() {
            if matches!(graph.entries[index], Entry::Outside) {
                continue;
            }
            occurrence.with_component_mut(|component| {
                component.bind_graph_dirt(Rc::clone(&graph), index as u32)
            });
            if let ComponentOccurrenceHandle::Authored(handle) = occurrence {
                handle.bind_component_graph(Rc::clone(&graph));
            }
        }
        graph
    }

    #[inline]
    pub(crate) fn dirt(&self, index: u32) -> ComponentDirt {
        self.dirt[index as usize].get()
    }

    #[inline]
    pub(crate) fn set_dirt(&self, index: u32, value: ComponentDirt) {
        self.dirt[index as usize].set(value);
    }

    /// Artboard::onComponentDirty for a member: its graph order is its index.
    #[inline]
    pub(crate) fn notify_artboard(&self, index: u32) {
        self.artboard.on_component_dirty_at(index);
    }

    /// Component::addDirt for a member reached without a held payload borrow.
    pub(crate) fn add_dirt(&self, index: u32, value: ComponentDirt, recurse: bool) -> bool {
        let i = index as usize;
        let entry = self.entries[i];
        let occurrence = &self.occurrences[i];
        if matches!(entry, Entry::Outside) {
            return occurrence.add_dirt(value, recurse);
        }
        if let ComponentOccurrenceHandle::Authored(handle) = occurrence
            && !handle.is_current_generation()
        {
            return false;
        }
        let cell = &self.dirt[i];
        let dirt = cell.get();
        if dirt.contains(value) {
            return false;
        }
        let dirt = dirt | value;
        cell.set(dirt);
        match (entry, occurrence) {
            (Entry::Noop(_), _) => {}
            (Entry::Handler(handler, _), ComponentOccurrenceHandle::Authored(handle)) => {
                handler(handle, dirt)
            }
            (Entry::Receiver(_), ComponentOccurrenceHandle::Authored(handle)) => {
                handle.with_mut(|object| {
                    object.component_on_dirty(dirt);
                });
            }
            (Entry::Helper, helper) => helper.on_dirty_helper(dirt),
            _ => unreachable!("authored entries hold authored occurrences"),
        }
        self.notify_artboard(index);
        if recurse {
            self.add_dirt_to_dependents(index, value);
        }
        true
    }

    /// DependencyHelper::addDirtToDependents over the static edges.
    pub(crate) fn add_dirt_to_dependents(&self, index: u32, value: ComponentDirt) {
        let start = self.dependents_start[index as usize] as usize;
        let end = self.dependents_start[index as usize + 1] as usize;
        for dependent in &self.dependents[start..end] {
            match dependent {
                Dependent::Member(member) => {
                    self.add_dirt(*member, value, true);
                }
                Dependent::Outside(occurrence) => {
                    occurrence.add_dirt(value, true);
                }
            }
        }
    }

    /// Artboard::updateComponents over the graph. `depth` and the root's
    /// COMPONENTS bit live in the shared dirty handle, so no Artboard borrow is
    /// held while components update.
    pub(crate) fn update_components(&self) {
        let dirty = &self.artboard;
        let count = self.entries.len();
        let mut step = 0;
        while step < 100 && dirty.has_component_dirt() {
            dirty.clear_components_dirt();
            for i in 0..count {
                dirty.set_depth(i as u32);
                if !self.update_entry(i) {
                    continue;
                }
                if dirty.depth() < i as u32 {
                    break;
                }
            }
            step += 1;
        }
    }

    fn update_entry(&self, i: usize) -> bool {
        let occurrence = &self.occurrences[i];
        let update = match self.entries[i] {
            Entry::Outside => return occurrence.update_if_dirty(),
            Entry::Helper => {
                let cell = &self.dirt[i];
                let dirt = cell.get();
                if !needs_update(dirt) {
                    return false;
                }
                cell.set(ComponentDirt::NONE);
                occurrence.update_helper(dirt);
                return true;
            }
            Entry::Noop(update) | Entry::Handler(_, update) | Entry::Receiver(update) => update,
        };
        let ComponentOccurrenceHandle::Authored(handle) = occurrence else {
            unreachable!("authored entries hold authored occurrences");
        };
        assert!(
            handle.is_current_generation(),
            "live component in dependency graph"
        );
        let cell = &self.dirt[i];
        let dirt = cell.get();
        if !needs_update(dirt) {
            return false;
        }
        cell.set(ComponentDirt::NONE);
        if let Some(update) = update {
            update(handle, dirt);
        } else {
            handle.with_mut(|object| object.component_update(dirt));
        }
        true
    }
}

#[inline]
fn needs_update(dirt: ComponentDirt) -> bool {
    dirt != ComponentDirt::NONE && !dirt.contains(ComponentDirt::COLLAPSED)
}
