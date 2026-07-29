use crate::properties::property_key_for_name;
use crate::{
    ArtboardInstance, RuntimeGeometryHit, RuntimeGeometryHitOccurrence,
    RuntimeGeometryHitPathSegment,
};
use nuxie_binary::RuntimeObject;

/// Backward-compatible string projection of one typed Event custom property.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StateMachineEventStringProperty {
    name: String,
    value: String,
}

impl StateMachineEventStringProperty {
    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn value(&self) -> &str {
        &self.value
    }
}

/// Rust host projection of pinned C++ `EventReport`.
///
/// C++ retains the live `Event*`. Rust retains its occurrence-local id and
/// refreshes this projection at each observation seam because borrowing the
/// owning Artboard inside the report would be self-referential.
#[derive(Debug, Clone)]
pub struct StateMachineReportedEvent {
    pub(crate) event_local_index: usize,
    pub(crate) event_core_type: u32,
    pub(crate) name: Option<String>,
    pub(crate) url: Option<String>,
    pub(crate) target: Option<String>,
    pub(crate) properties: Vec<crate::RuntimeEventProperty>,
    /// Backward-compatible projection of the string-valued entries in
    /// `properties`.
    pub(crate) string_properties: Vec<StateMachineEventStringProperty>,
    pub(crate) seconds_delay: f32,
    pub(crate) context: Option<StateMachineEventContext>,
}

/// Exact rendered occurrence that caused a pointer-listener event.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StateMachineEventContext {
    pub(super) path: Vec<RuntimeGeometryHitPathSegment>,
    pub(super) occurrence: Vec<RuntimeGeometryHitOccurrence>,
}

impl StateMachineEventContext {
    pub fn from_geometry_hit(hit: &RuntimeGeometryHit) -> Self {
        Self {
            path: hit.path.clone(),
            occurrence: hit.occurrence.clone(),
        }
    }

    pub fn path(&self) -> &[RuntimeGeometryHitPathSegment] {
        &self.path
    }

    pub fn occurrence(&self) -> &[RuntimeGeometryHitOccurrence] {
        &self.occurrence
    }
}

impl StateMachineReportedEvent {
    pub(crate) fn from_runtime_event(event_local_index: usize, event: &RuntimeObject) -> Self {
        let (url, target) = if event.type_name == "OpenUrlEvent" {
            (
                Some(event.string_property("url").unwrap_or_default().to_owned()),
                Some(open_url_target(event.uint_property("targetValue").unwrap_or(0)).to_owned()),
            )
        } else {
            (None, None)
        };
        Self {
            event_local_index,
            event_core_type: u32::from(event.type_key),
            name: event
                .string_property("name")
                .filter(|name| !name.is_empty())
                .map(ToOwned::to_owned),
            url,
            target,
            properties: Vec::new(),
            string_properties: Vec::new(),
            seconds_delay: 0.0,
            context: None,
        }
    }

    pub(crate) fn from_live_artboard_event(
        artboard: &ArtboardInstance,
        event_local_index: usize,
    ) -> Option<Self> {
        let type_name = artboard.runtime_object_type_name(event_local_index)?;
        let definition = nuxie_schema::definition_by_name(type_name)?;
        if !definition.is_a("Event") {
            return None;
        }
        let live_string = |local_id: usize, owner: &str, name: &str| {
            let key = property_key_for_name(owner, name)?;
            artboard
                .string_property(local_id, key)
                .map(|value| String::from_utf8_lossy(value).into_owned())
        };
        let name =
            live_string(event_local_index, type_name, "name").filter(|name| !name.is_empty());
        let (url, target) = if type_name == "OpenUrlEvent" {
            let url = live_string(event_local_index, type_name, "url").unwrap_or_default();
            let target_value = property_key_for_name(type_name, "targetValue")
                .and_then(|key| artboard.uint_property(event_local_index, key))
                .unwrap_or(0);
            (Some(url), Some(open_url_target(target_value).to_owned()))
        } else {
            (None, None)
        };
        // C++ retains the live Event occurrence, including every typed custom
        // property attached to it. Snapshot that occurrence at the Rust host
        // boundary in exact child order; do not rebuild a string-only template
        // from the imported file (`listener_fire_event.cpp:8-18`;
        // `state_machine_fire_event.cpp:10-18`).
        let properties = artboard.event_properties(event_local_index);
        let string_properties = properties
            .iter()
            .filter_map(|property| {
                let crate::RuntimeEventPropertyValue::String(value) = &property.value else {
                    return None;
                };
                let name = property.name.as_deref()?.trim();
                if name.is_empty() {
                    return None;
                }
                Some(StateMachineEventStringProperty {
                    name: name.to_owned(),
                    value: String::from_utf8_lossy(value).into_owned(),
                })
            })
            .collect();
        Some(Self {
            event_local_index,
            event_core_type: u32::from(definition.type_key.int),
            name,
            url,
            target,
            properties,
            string_properties,
            seconds_delay: 0.0,
            context: None,
        })
    }

    /// Re-resolve the retained Event identity against its live Artboard.
    ///
    /// C++ `EventReport` stores an `Event*`, not a payload copy. Rust cannot
    /// retain a self-referential borrow into `ArtboardInstance`, so the
    /// source-corresponding adaptation retains the occurrence-local id and
    /// refreshes the public projection whenever the report is observed or
    /// delivered. Delay and nested/pointer context belong to the report and
    /// therefore survive the Event refresh.
    pub(crate) fn refresh_from_live_artboard(&mut self, artboard: &ArtboardInstance) -> bool {
        let Some(mut live) = Self::from_live_artboard_event(artboard, self.event_local_index)
        else {
            return false;
        };
        live.seconds_delay = self.seconds_delay;
        live.context = self.context.clone();
        *self = live;
        true
    }

    pub fn event_local_index(&self) -> usize {
        self.event_local_index
    }

    pub fn event_core_type(&self) -> u32 {
        self.event_core_type
    }

    pub(crate) fn is_audio_event(&self) -> bool {
        nuxie_schema::definition_by_name("AudioEvent")
            .is_some_and(|definition| self.event_core_type == u32::from(definition.type_key.int))
    }

    pub fn name(&self) -> Option<&str> {
        self.name.as_deref()
    }

    pub fn url(&self) -> Option<&str> {
        self.url.as_deref()
    }

    pub fn target(&self) -> Option<&str> {
        self.target.as_deref()
    }

    pub fn string_properties(&self) -> &[StateMachineEventStringProperty] {
        &self.string_properties
    }

    /// Typed custom properties retained on the live Event when it fired.
    pub fn properties(&self) -> &[crate::RuntimeEventProperty] {
        &self.properties
    }

    pub fn seconds_delay(&self) -> f32 {
        self.seconds_delay
    }

    pub fn context(&self) -> Option<&StateMachineEventContext> {
        self.context.as_ref()
    }
}

pub(super) fn open_url_target(value: u64) -> &'static str {
    match value {
        0 => "_blank",
        1 => "_parent",
        2 => "_self",
        3 => "_top",
        _ => "",
    }
}
