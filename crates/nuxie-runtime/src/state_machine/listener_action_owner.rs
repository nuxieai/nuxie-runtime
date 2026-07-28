//! Shared generated-field owner for state-machine actions.
//!
//! Pinned C++ keeps every `ListenerAction` and `StateMachineFireAction` on the
//! source `StateMachine` definition. `Artboard::instance()` shares that
//! definition pointer, and action `perform` methods read generated fields from
//! it at the call site (`include/rive/artboard.hpp:548-594`;
//! `src/animation/listener_action.cpp`; `src/artboard.cpp:1038-1057`).
//! Consequently every occurrence created from one loaded file observes one
//! mutable action owner, while a separately loaded file owns a fresh one.

use nuxie_binary::RuntimeObject;
use std::collections::BTreeMap;
use std::sync::{Arc, RwLock, RwLockReadGuard, RwLockWriteGuard};

// Generated C++ base constants at pinned runtime d788e8ec. Reads and writes
// dispatch by generated property key, never by a runtime property-name scan.
pub(crate) const LISTENER_FLAGS_KEY: u16 = 980;
pub(crate) const LISTENER_FIRE_EVENT_ID_KEY: u16 = 389;
pub(crate) const LISTENER_INPUT_ID_KEY: u16 = 227;
pub(crate) const LISTENER_NESTED_INPUT_ID_KEY: u16 = 400;
pub(crate) const LISTENER_BOOL_VALUE_KEY: u16 = 228;
pub(crate) const LISTENER_NUMBER_VALUE_KEY: u16 = 229;
pub(crate) const LISTENER_ALIGN_TARGET_ID_KEY: u16 = 240;
pub(crate) const LISTENER_ALIGN_PRESERVE_OFFSET_KEY: u16 = 541;
pub(crate) const FOCUS_TARGET_ID_KEY: u16 = 952;
pub(crate) const FOCUS_TRAVERSAL_KIND_KEY: u16 = 1011;
pub(crate) const SCRIPTED_LISTENER_ASSET_ID_KEY: u16 = 930;
pub(crate) const FIRE_OCCURS_VALUE_KEY: u16 = 393;
pub(crate) const FIRE_EVENT_ID_KEY: u16 = 392;

#[derive(Debug, Clone)]
pub(crate) struct RuntimeActionCoreArena {
    inner: Arc<RuntimeActionCoreArenaInner>,
}

/// Source-StateMachine action owners retained by one loaded runtime file.
///
/// C++ `Artboard::instance()` shares the source `StateMachine` definitions
/// rather than cloning action objects. The high-level file facade retains one
/// catalog and passes it through every root, nested, list, and scripted
/// artboard build so all occurrences from that file read the same generated
/// action fields. A separately imported file constructs a fresh catalog.
#[derive(Debug, Clone)]
#[doc(hidden)]
pub struct RuntimeFileStateMachineActionCatalog {
    arenas: Arc<BTreeMap<u32, RuntimeActionCoreArena>>,
}

#[derive(Debug)]
struct RuntimeActionCoreArenaInner {
    owners: RwLock<Vec<RuntimeActionCoreOwner>>,
    index_by_global: BTreeMap<u32, usize>,
}

#[derive(Debug, Clone)]
pub(crate) struct RuntimeActionCoreHandle {
    inner: Arc<RuntimeActionCoreArenaInner>,
    index: usize,
}

#[derive(Debug, Clone)]
struct RuntimeActionCoreOwner {
    global_id: u32,
    flags: u32,
    event_id: u32,
    input_id: u32,
    nested_input_id: u32,
    uint_value: u32,
    number_value: f32,
    target_id: u32,
    preserve_offset: bool,
    traversal_kind: u32,
    script_asset_id: u32,
    occurs_value: u32,
}

impl RuntimeActionCoreOwner {
    fn from_object(object: &RuntimeObject) -> Self {
        Self {
            global_id: object.id,
            flags: object.uint_property("flags").unwrap_or(0) as u32,
            event_id: object
                .uint_property("eventId")
                .unwrap_or(u64::from(u32::MAX)) as u32,
            input_id: object
                .uint_property("inputId")
                .unwrap_or(u64::from(u32::MAX)) as u32,
            nested_input_id: object
                .uint_property("nestedInputId")
                .unwrap_or(u64::from(u32::MAX)) as u32,
            uint_value: object.uint_property("value").unwrap_or(1) as u32,
            number_value: object.double_property("value").unwrap_or(0.0),
            target_id: object
                .uint_property("targetId")
                .unwrap_or(u64::from(u32::MAX)) as u32,
            preserve_offset: object.bool_property("preserveOffset").unwrap_or(false),
            traversal_kind: object.uint_property("traversalKind").unwrap_or(0) as u32,
            script_asset_id: object
                .uint_property("scriptAssetId")
                .unwrap_or(u64::from(u32::MAX)) as u32,
            occurs_value: object.uint_property("occursValue").unwrap_or(0) as u32,
        }
    }

    fn uint(&self, property_key: u16) -> Option<u32> {
        Some(match property_key {
            LISTENER_FLAGS_KEY => self.flags,
            LISTENER_FIRE_EVENT_ID_KEY | FIRE_EVENT_ID_KEY => self.event_id,
            LISTENER_INPUT_ID_KEY => self.input_id,
            LISTENER_NESTED_INPUT_ID_KEY => self.nested_input_id,
            LISTENER_BOOL_VALUE_KEY => self.uint_value,
            LISTENER_ALIGN_TARGET_ID_KEY | FOCUS_TARGET_ID_KEY => self.target_id,
            FOCUS_TRAVERSAL_KIND_KEY => self.traversal_kind,
            SCRIPTED_LISTENER_ASSET_ID_KEY => self.script_asset_id,
            FIRE_OCCURS_VALUE_KEY => self.occurs_value,
            _ => return None,
        })
    }

    fn set_uint(&mut self, property_key: u16, value: u64) -> bool {
        let value = value as u32;
        let target = match property_key {
            LISTENER_FLAGS_KEY => &mut self.flags,
            LISTENER_FIRE_EVENT_ID_KEY | FIRE_EVENT_ID_KEY => &mut self.event_id,
            LISTENER_INPUT_ID_KEY => &mut self.input_id,
            LISTENER_NESTED_INPUT_ID_KEY => &mut self.nested_input_id,
            LISTENER_BOOL_VALUE_KEY => &mut self.uint_value,
            LISTENER_ALIGN_TARGET_ID_KEY | FOCUS_TARGET_ID_KEY => &mut self.target_id,
            FOCUS_TRAVERSAL_KIND_KEY => &mut self.traversal_kind,
            SCRIPTED_LISTENER_ASSET_ID_KEY => &mut self.script_asset_id,
            FIRE_OCCURS_VALUE_KEY => &mut self.occurs_value,
            _ => return false,
        };
        if *target == value {
            return false;
        }
        *target = value;
        true
    }

    fn bool(&self, property_key: u16) -> Option<bool> {
        match property_key {
            LISTENER_ALIGN_PRESERVE_OFFSET_KEY => Some(self.preserve_offset),
            _ => None,
        }
    }

    fn set_bool(&mut self, property_key: u16, value: bool) -> bool {
        let target = match property_key {
            LISTENER_ALIGN_PRESERVE_OFFSET_KEY => &mut self.preserve_offset,
            _ => return false,
        };
        if *target == value {
            return false;
        }
        *target = value;
        true
    }

    fn double(&self, property_key: u16) -> Option<f32> {
        match property_key {
            LISTENER_NUMBER_VALUE_KEY => Some(self.number_value),
            _ => None,
        }
    }

    fn set_double(&mut self, property_key: u16, value: f32) -> bool {
        let target = match property_key {
            LISTENER_NUMBER_VALUE_KEY => &mut self.number_value,
            _ => return false,
        };
        // Generated C++ setters use ordinary `==`: signed zero is equal,
        // while every NaN assignment is considered a change.
        if *target == value {
            return false;
        }
        *target = value;
        true
    }
}

impl RuntimeActionCoreArena {
    pub(crate) fn empty() -> Self {
        Self {
            inner: Arc::new(RuntimeActionCoreArenaInner {
                owners: RwLock::new(Vec::new()),
                index_by_global: BTreeMap::new(),
            }),
        }
    }

    pub(crate) fn from_objects<'a>(objects: impl IntoIterator<Item = &'a RuntimeObject>) -> Self {
        let mut objects = objects.into_iter().collect::<Vec<_>>();
        objects.sort_by_key(|object| object.id);
        objects.dedup_by_key(|object| object.id);
        let owners = objects
            .into_iter()
            .map(RuntimeActionCoreOwner::from_object)
            .collect::<Vec<_>>();
        let index_by_global = owners
            .iter()
            .enumerate()
            .map(|(index, owner)| (owner.global_id, index))
            .collect();
        Self {
            inner: Arc::new(RuntimeActionCoreArenaInner {
                owners: RwLock::new(owners),
                index_by_global,
            }),
        }
    }

    fn from_state_machine(state_machine: &nuxie_binary::RuntimeStateMachine<'_>) -> Self {
        let listener_actions = state_machine
            .listeners
            .iter()
            .flat_map(|listener| listener.actions.iter().map(|action| action.object));
        let layer_actions = state_machine.layers.iter().flat_map(|layer| {
            layer.states.iter().flat_map(|state| {
                let state_actions = state
                    .fire_actions
                    .iter()
                    .map(|action| action.object)
                    .chain(state.listener_actions.iter().map(|action| action.object));
                let transition_actions = state.transitions.iter().flat_map(|transition| {
                    transition
                        .fire_actions
                        .iter()
                        .map(|action| action.object)
                        .chain(
                            transition
                                .listener_actions
                                .iter()
                                .map(|action| action.object),
                        )
                });
                state_actions.chain(transition_actions)
            })
        });
        Self::from_objects(listener_actions.chain(layer_actions))
    }

    pub(crate) fn handle(&self, global_id: u32) -> Option<RuntimeActionCoreHandle> {
        let index = *self.inner.index_by_global.get(&global_id)?;
        Some(RuntimeActionCoreHandle {
            inner: Arc::clone(&self.inner),
            index,
        })
    }

    pub(crate) fn set_uint(&self, global_id: u32, property_key: u16, value: u64) -> bool {
        self.handle(global_id)
            .is_some_and(|handle| handle.set_uint(property_key, value))
    }

    pub(crate) fn set_bool(&self, global_id: u32, property_key: u16, value: bool) -> bool {
        self.handle(global_id)
            .is_some_and(|handle| handle.set_bool(property_key, value))
    }

    pub(crate) fn set_double(&self, global_id: u32, property_key: u16, value: f32) -> bool {
        self.handle(global_id)
            .is_some_and(|handle| handle.set_double(property_key, value))
    }
}

impl RuntimeFileStateMachineActionCatalog {
    #[doc(hidden)]
    pub fn new(file: &nuxie_binary::RuntimeFile) -> Self {
        let arenas = (0..file.artboards().len())
            .flat_map(|artboard_index| file.artboard_state_machine_graphs(artboard_index))
            .map(|state_machine| {
                (
                    state_machine.object.id,
                    RuntimeActionCoreArena::from_state_machine(&state_machine),
                )
            })
            .collect();
        Self {
            arenas: Arc::new(arenas),
        }
    }

    pub(crate) fn arena(&self, state_machine_global_id: u32) -> Option<RuntimeActionCoreArena> {
        self.arenas.get(&state_machine_global_id).cloned()
    }

    /// Exact generated-uint mutation seam for source-definition DataBinds and
    /// direct Core writes. It intentionally does not address an Artboard local
    /// object: action definitions are file-owned and are not cloned into the
    /// state-machine instance DataBind graph.
    #[doc(hidden)]
    pub fn set_uint(
        &self,
        state_machine_global_id: u32,
        action_global_id: u32,
        property_key: u16,
        value: u64,
    ) -> bool {
        self.arenas
            .get(&state_machine_global_id)
            .is_some_and(|arena| arena.set_uint(action_global_id, property_key, value))
    }

    #[doc(hidden)]
    pub fn set_bool(
        &self,
        state_machine_global_id: u32,
        action_global_id: u32,
        property_key: u16,
        value: bool,
    ) -> bool {
        self.arenas
            .get(&state_machine_global_id)
            .is_some_and(|arena| arena.set_bool(action_global_id, property_key, value))
    }

    #[doc(hidden)]
    pub fn set_double(
        &self,
        state_machine_global_id: u32,
        action_global_id: u32,
        property_key: u16,
        value: f32,
    ) -> bool {
        self.arenas
            .get(&state_machine_global_id)
            .is_some_and(|arena| arena.set_double(action_global_id, property_key, value))
    }
}

impl RuntimeActionCoreHandle {
    fn owners(&self) -> RwLockReadGuard<'_, Vec<RuntimeActionCoreOwner>> {
        self.inner
            .owners
            .read()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }

    fn owners_mut(&self) -> RwLockWriteGuard<'_, Vec<RuntimeActionCoreOwner>> {
        self.inner
            .owners
            .write()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }

    pub(crate) fn uint(&self, property_key: u16) -> u64 {
        u64::from(
            self.owners()
                .get(self.index)
                .and_then(|owner| owner.uint(property_key))
                .expect("accepted action handle must address its generated uint field"),
        )
    }

    pub(crate) fn bool(&self, property_key: u16) -> bool {
        self.owners()
            .get(self.index)
            .and_then(|owner| owner.bool(property_key))
            .expect("accepted action handle must address its generated bool field")
    }

    pub(crate) fn double(&self, property_key: u16) -> f32 {
        self.owners()
            .get(self.index)
            .and_then(|owner| owner.double(property_key))
            .expect("accepted action handle must address its generated double field")
    }

    pub(crate) fn set_uint(&self, property_key: u16, value: u64) -> bool {
        self.owners_mut()
            .get_mut(self.index)
            .is_some_and(|owner| owner.set_uint(property_key, value))
    }

    pub(crate) fn set_bool(&self, property_key: u16, value: bool) -> bool {
        self.owners_mut()
            .get_mut(self.index)
            .is_some_and(|owner| owner.set_bool(property_key, value))
    }

    pub(crate) fn set_double(&self, property_key: u16, value: f32) -> bool {
        self.owners_mut()
            .get_mut(self.index)
            .is_some_and(|owner| owner.set_double(property_key, value))
    }

    #[cfg(test)]
    pub(crate) fn set_double_imported_for_test(&self, property_key: u16, value: f32) {
        let owner = &mut self.owners_mut()[self.index];
        match property_key {
            LISTENER_NUMBER_VALUE_KEY => owner.number_value = value,
            _ => panic!("unsupported imported test double property key {property_key}"),
        }
    }

    #[cfg(test)]
    pub(crate) fn for_test(type_name: &str) -> Self {
        assert!(
            nuxie_schema::definition_by_name(type_name).is_some(),
            "missing test action type {type_name}"
        );
        let inner = Arc::new(RuntimeActionCoreArenaInner {
            owners: RwLock::new(vec![RuntimeActionCoreOwner {
                global_id: 0,
                flags: 0,
                event_id: u32::MAX,
                input_id: u32::MAX,
                nested_input_id: u32::MAX,
                uint_value: 1,
                number_value: 0.0,
                target_id: u32::MAX,
                preserve_offset: false,
                traversal_kind: 0,
                script_asset_id: u32::MAX,
                occurs_value: 0,
            }]),
            index_by_global: BTreeMap::from([(0, 0)]),
        });
        Self { inner, index: 0 }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn generated_keys_match_schema_and_uints_narrow_like_cpp_storage() {
        for (owner, property, key) in [
            ("ListenerAction", "flags", LISTENER_FLAGS_KEY),
            ("ListenerFireEvent", "eventId", LISTENER_FIRE_EVENT_ID_KEY),
            ("ListenerInputChange", "inputId", LISTENER_INPUT_ID_KEY),
            (
                "ListenerInputChange",
                "nestedInputId",
                LISTENER_NESTED_INPUT_ID_KEY,
            ),
            ("ListenerBoolChange", "value", LISTENER_BOOL_VALUE_KEY),
            ("ListenerNumberChange", "value", LISTENER_NUMBER_VALUE_KEY),
            (
                "ListenerAlignTarget",
                "targetId",
                LISTENER_ALIGN_TARGET_ID_KEY,
            ),
            (
                "ListenerAlignTarget",
                "preserveOffset",
                LISTENER_ALIGN_PRESERVE_OFFSET_KEY,
            ),
            ("FocusActionTarget", "targetId", FOCUS_TARGET_ID_KEY),
            (
                "FocusActionTraversal",
                "traversalKind",
                FOCUS_TRAVERSAL_KIND_KEY,
            ),
            (
                "ScriptedListenerAction",
                "scriptAssetId",
                SCRIPTED_LISTENER_ASSET_ID_KEY,
            ),
            (
                "StateMachineFireAction",
                "occursValue",
                FIRE_OCCURS_VALUE_KEY,
            ),
            ("StateMachineFireEvent", "eventId", FIRE_EVENT_ID_KEY),
        ] {
            assert_eq!(
                crate::properties::property_key_for_name(owner, property),
                Some(key),
                "{owner}.{property}",
            );
        }

        let owner = RuntimeActionCoreHandle::for_test("ListenerFireEvent");
        assert!(owner.set_uint(LISTENER_FLAGS_KEY, u64::from(u32::MAX) + 2));
        assert_eq!(owner.uint(LISTENER_FLAGS_KEY), 1);

        let number = RuntimeActionCoreHandle::for_test("ListenerNumberChange");
        assert!(!number.set_double(LISTENER_NUMBER_VALUE_KEY, -0.0));
        assert!(number.set_double(LISTENER_NUMBER_VALUE_KEY, f32::NAN));
        assert!(number.set_double(LISTENER_NUMBER_VALUE_KEY, f32::NAN));
    }
}
