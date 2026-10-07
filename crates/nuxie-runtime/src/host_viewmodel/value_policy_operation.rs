use super::*;
use crate::mechanical_port::source::viewmodel::{
    viewmodel_instance_boolean::ViewModelInstanceBoolean,
    viewmodel_instance_color::ViewModelInstanceColor,
    viewmodel_instance_enum::ViewModelInstanceEnum, viewmodel_instance_list::ViewModelInstanceList,
    viewmodel_instance_list_item::ViewModelInstanceListItem,
    viewmodel_instance_number::ViewModelInstanceNumber,
    viewmodel_instance_string::ViewModelInstanceString,
};

type Key = (u64, usize);

#[path = "value_policy_groups.rs"]
pub(super) mod groups;
pub use groups::{RuntimeRuleGroup, RuntimeRuleGroupMember};

struct InitialOwners {
    file: RuntimeFileHandle,
    owners: BTreeMap<u64, RuntimeOwnedViewModelHandle>,
    values: BTreeMap<Key, RuntimeViewModelChangeValue>,
    lists: BTreeMap<Key, Vec<CoreHandle>>,
    overflowed: bool,
}

thread_local! {
    static INITIAL_OWNERS: RefCell<std::rc::Weak<RefCell<InitialOwners>>> = const { RefCell::new(std::rc::Weak::new()) };
}

/// Script-created owners must be observed before their first native write.
/// The weak operation lease prevents the file/VM from retaining policy state.
pub(crate) fn capture_initial_policy_owner(file: &RuntimeFileHandle, native: &CoreHandle) {
    if !crate::view_model_cell::is_capturing_view_model_changes() {
        return;
    }
    let Some(state) = INITIAL_OWNERS.with(|slot| slot.borrow().upgrade()) else {
        return;
    };
    let mut state = state.borrow_mut();
    if !state.file.ptr_eq(file) || state.overflowed {
        return;
    }
    let Some(root) = RuntimeOwnedViewModelHandle::from_native(file.clone(), native.clone()) else {
        return;
    };
    let Some(owners) = root.reachable_change_owner_snapshot() else {
        state.overflowed = true;
        return;
    };
    for owner in owners {
        let id = owner.instance_identity();
        if state.owners.contains_key(&id) {
            continue;
        }
        let Some(properties) = owner
            .native_handle()
            .with_downcast::<ViewModelInstance, _>(|owner| owner.property_values().to_vec())
        else {
            state.overflowed = true;
            return;
        };
        for property in properties {
            if state.values.len() >= 4_096 {
                state.overflowed = true;
                return;
            }
            let Some(index) = property
                .with(|property| {
                    property
                        .as_view_model_instance_value()
                        .map(|value| value.base.view_model_property_id() as usize)
                })
                .flatten()
            else {
                state.overflowed = true;
                return;
            };
            if let Some(value) = read(&property) {
                state.values.insert((id, index), value);
            }
            if let Some(items) = property
                .with_downcast::<ViewModelInstanceList, _>(|list| list.list_items().to_vec())
            {
                state.lists.insert((id, index), items);
            }
        }
        state.owners.insert(id, owner);
    }
}

#[derive(Clone, Debug)]
pub struct RuntimeValueRuleReport {
    pub owner_instance_identity: u64,
    pub property_index: usize,
    pub rule_index: usize,
    pub rule_owner_instance_identity: u64,
    pub rule_property_index: usize,
    pub refused: bool,
    pub attempted: RuntimeViewModelChangeValue,
}

#[derive(Clone, Debug)]
pub struct RuntimeCheckedValueWrite {
    pub applied: bool,
    /// Refusing rules on refusal, marking breaches on success, in table order.
    pub rule_indices: Vec<usize>,
}

/// One operation's retained native owners and last accepted observations.
/// The caller's graph transaction owns failure rollback. Drop this state at
/// commit; it is not a second persistent store of view-model values.
pub struct RuntimeValuePolicyOperation {
    owners: BTreeMap<u64, RuntimeOwnedViewModelHandle>,
    values: BTreeMap<Key, RuntimeViewModelChangeValue>,
    native_lists: BTreeMap<Key, Vec<CoreHandle>>,
    suppressed: BTreeMap<Key, Vec<usize>>,
    // Checked calls can run ahead of replay; their refusals must not suppress
    // marker writes earlier in the native journal.
    checked_suppressed: BTreeMap<Key, Vec<usize>>,
    cursor: usize,
    pending_flush: bool,
    reports: Vec<RuntimeValueRuleReport>,
    report_bytes: usize,
    value_reports: BTreeMap<Key, BTreeSet<usize>>,
    initial: Rc<RefCell<InitialOwners>>,
    group_refusals: BTreeMap<Key, Vec<usize>>,
    group_roots: Vec<RuntimeOwnedViewModelHandle>,
    groups_revision: u64,
}

impl RuntimeValuePolicy {
    pub fn begin_rules(
        &self,
        roots: &[RuntimeOwnedViewModelHandle],
    ) -> Result<RuntimeValuePolicyOperation, RuntimeValuePolicyError> {
        let initial = Rc::new(RefCell::new(InitialOwners {
            file: self.file.clone(),
            owners: BTreeMap::new(),
            values: BTreeMap::new(),
            lists: BTreeMap::new(),
            overflowed: false,
        }));
        INITIAL_OWNERS.with(|slot| {
            *slot.borrow_mut() = if self.has_rules() || self.has_groups() {
                Rc::downgrade(&initial)
            } else {
                std::rc::Weak::new()
            }
        });
        let mut operation = RuntimeValuePolicyOperation {
            owners: BTreeMap::new(),
            values: BTreeMap::new(),
            native_lists: BTreeMap::new(),
            suppressed: BTreeMap::new(),
            checked_suppressed: BTreeMap::new(),
            cursor: 0,
            pending_flush: false,
            reports: Vec::new(),
            report_bytes: 0,
            value_reports: BTreeMap::new(),
            initial,
            group_refusals: self.group_refusals.borrow().clone(),
            group_roots: roots.to_vec(),
            groups_revision: self.revision.get(),
        };
        if self.has_rules() || self.has_groups() {
            operation.retain(self, roots)?;
        }
        Ok(operation)
    }
}

impl RuntimeValuePolicyOperation {
    pub fn reports(&self) -> &[RuntimeValueRuleReport] {
        &self.reports
    }

    pub fn retained_roots(&self) -> Vec<RuntimeOwnedViewModelHandle> {
        self.owners.values().cloned().collect()
    }

    pub fn apply_pending_writes(
        &mut self,
        policy: &RuntimeValuePolicy,
        roots: &[RuntimeOwnedViewModelHandle],
    ) -> Result<bool, RuntimeValuePolicyError> {
        let changed = RuntimeViewModelChangeCapture::with_current(|capture| {
            self.apply(policy, capture, roots)
        })
        .ok_or(RuntimeValuePolicyError::BorrowConflict)??;
        self.pending_flush |= changed;
        Ok(changed)
    }

    /// Check the candidate before invoking its native writer. The optional
    /// marker is the explicit paired write the caller will make after this
    /// value (false for a clear); absent means an ordinary nonempty write.
    /// Accepted writes still enter the operation's native journal. Run `apply`
    /// before calling this from a script if earlier unchecked native writes
    /// are pending, and again at the operation boundary before publication.
    pub fn checked_write(
        &mut self,
        policy: &RuntimeValuePolicy,
        root: &RuntimeOwnedViewModelHandle,
        path: &str,
        candidate: RuntimeViewModelChangeValue,
        marker: Option<bool>,
        write: impl FnOnce() -> Result<(), RuntimeValuePolicyError>,
    ) -> Result<RuntimeCheckedValueWrite, RuntimeValuePolicyError> {
        policy.invalidate();
        if !policy.has_rules() {
            write()?;
            return Ok(RuntimeCheckedValueWrite {
                applied: true,
                rule_indices: Vec::new(),
            });
        }
        let (owner, index) = policy.resolve_property(root, path)?;
        let key = (owner.instance_identity(), index);
        if let Some(rule_indices) = self.checked_suppressed.get(&key) {
            return Ok(RuntimeCheckedValueWrite {
                applied: false,
                rule_indices: rule_indices.clone(),
            });
        }
        let mut probe = Self {
            owners: BTreeMap::new(),
            values: BTreeMap::new(),
            native_lists: BTreeMap::new(),
            suppressed: BTreeMap::new(),
            checked_suppressed: BTreeMap::new(),
            cursor: 0,
            pending_flush: false,
            reports: Vec::new(),
            report_bytes: 0,
            value_reports: BTreeMap::new(),
            initial: Rc::clone(&self.initial),
            group_refusals: BTreeMap::new(),
            group_roots: Vec::new(),
            groups_revision: policy.revision.get(),
        };
        let mut roots = self.retained_roots();
        roots.push(root.clone());
        probe.retain(policy, &roots)?;
        let current = probe
            .values
            .get(&key)
            .ok_or(RuntimeValuePolicyError::InvalidArgument)?;
        if std::mem::discriminant(current) != std::mem::discriminant(&candidate) {
            return Err(RuntimeValuePolicyError::InvalidArgument);
        }
        probe.values.insert(key, candidate.clone());
        let model = owner.borrow().view_model_index();
        let marker_key = policy
            .markers
            .iter()
            .find(|pair| pair.model == model && pair.value == index)
            .map(|pair| (key.0, pair.marker));
        if let Some(marker_key) = marker_key {
            probe.values.insert(
                marker_key,
                RuntimeViewModelChangeValue::Boolean(marker.unwrap_or(true)),
            );
        }
        let failures = probe.failures(policy, &owner, key);
        let refused = failures.iter().any(|(_, _, refused)| *refused);
        let mut rule_indices = Vec::new();
        for (rule_index, _, refusal) in failures {
            if refusal || !refused {
                rule_indices.push(rule_index);
            }
        }
        if let Some(marker_key) = marker_key {
            if refused {
                self.checked_suppressed
                    .insert(marker_key, rule_indices.clone());
            } else {
                self.checked_suppressed.remove(&marker_key);
            }
        }
        if refused {
            let property = owner
                .borrow()
                .property_by_path(&[index])
                .ok_or(RuntimeValuePolicyError::NotFound)?;
            // The ordered attempt is evaluated with every other write at the
            // boundary, but the refused native writer is never invoked.
            crate::view_model_cell::capture_view_model_change(
                instance::identity(&property) as usize,
                candidate,
            );
        } else {
            write()?;
        }
        Ok(RuntimeCheckedValueWrite {
            applied: !refused,
            rule_indices,
        })
    }

    /// Evaluate newly captured writes in native order, then restore only the
    /// properties whose final attempted value differs from the accepted one.
    /// Call again after the native binding update until it produces no writes.
    pub fn apply(
        &mut self,
        policy: &RuntimeValuePolicy,
        capture: &RuntimeViewModelChangeCapture,
        roots: &[RuntimeOwnedViewModelHandle],
    ) -> Result<bool, RuntimeValuePolicyError> {
        if !policy.has_rules() {
            return Ok(false);
        }
        if capture
            .write_count()
            .map_err(|_| RuntimeValuePolicyError::LimitExceeded)?
            == self.cursor
        {
            return Ok(std::mem::take(&mut self.pending_flush));
        }
        policy.invalidate();
        #[cfg(any(test, feature = "policy-pass-counter"))]
        RuntimeValuePolicy::record_test_pass();
        self.absorb_initial()?;
        self.retain(policy, roots)?;
        let retained = self.owners.values().cloned().collect::<Vec<_>>();
        let changes = RuntimeOwnedViewModelHandle::resolve_change_snapshot(&retained, capture)
            .ok_or(RuntimeValuePolicyError::LimitExceeded)?;
        let captured = capture
            .snapshot()
            .map_err(|_| RuntimeValuePolicyError::LimitExceeded)?;
        let mut omitted = BTreeSet::new();
        let mut restore = BTreeSet::new();
        for (order, (owner, change)) in changes.iter().enumerate().skip(self.cursor) {
            let id = owner.instance_identity();
            let model = owner.borrow().view_model_index();
            let key = (id, change.property_index);
            if self.suppressed.contains_key(&key) {
                omitted.insert(order);
                restore.insert(key);
                continue;
            }
            let pair = policy
                .markers
                .iter()
                .find(|pair| pair.model == model && pair.value == key.1);
            let marker_key = pair.map(|pair| (id, pair.marker));
            if let Some(marker_key) = marker_key {
                self.suppressed.remove(&marker_key);
            }
            let previous = self.values.insert(key, change.value.clone());
            let previous_marker = marker_key.and_then(|key| self.values.get(&key).cloned());
            if let Some(marker_key) = marker_key {
                let mut effective = true;
                for (later_owner, later) in changes.iter().skip(order.saturating_add(1)) {
                    if later_owner.instance_identity() != id {
                        continue;
                    }
                    if later.property_index == key.1 {
                        break;
                    }
                    if later.property_index == marker_key.1
                        && let RuntimeViewModelChangeValue::Boolean(value) = later.value
                    {
                        effective = value;
                    }
                }
                self.values
                    .insert(marker_key, RuntimeViewModelChangeValue::Boolean(effective));
            }
            let failures = self.failures(policy, owner, key);
            let refused = failures.iter().any(|(_, _, refused)| *refused);
            let refusing_rules = failures
                .iter()
                .filter_map(|(index, _, refused)| refused.then_some(*index))
                .collect::<Vec<_>>();
            // A refusal keeps the prior accepted value, including the marking
            // breaches already reported for its eventual paired marker.
            if !refused {
                self.value_reports.remove(&key);
            }
            self.record_group_outcome(policy, owner, key, &failures, refused)?;
            for (rule_index, target, refusal) in failures {
                if refusal || !refused {
                    let paired_marker = key.0 == target.0
                        && policy.markers.iter().any(|pair| {
                            pair.model == model && pair.value == target.1 && pair.marker == key.1
                        });
                    if paired_marker
                        && self
                            .value_reports
                            .get(&target)
                            .is_some_and(|reported| reported.contains(&rule_index))
                    {
                        continue;
                    }
                    if key == target {
                        self.value_reports
                            .entry(key)
                            .or_default()
                            .insert(rule_index);
                    }
                    if self.reports.len() == 4_096 {
                        return Err(RuntimeValuePolicyError::LimitExceeded);
                    }
                    let payload_bytes = match &change.value {
                        RuntimeViewModelChangeValue::String(value) => value.len(),
                        RuntimeViewModelChangeValue::List(items) => {
                            items.len().saturating_mul(std::mem::size_of::<u64>())
                        }
                        _ => 0,
                    };
                    self.report_bytes = self
                        .report_bytes
                        .checked_add(payload_bytes)
                        .and_then(|bytes| {
                            bytes.checked_add(std::mem::size_of::<RuntimeValueRuleReport>())
                        })
                        .filter(|bytes| *bytes <= 8 * 1024 * 1024)
                        .ok_or(RuntimeValuePolicyError::LimitExceeded)?;
                    self.reports.push(RuntimeValueRuleReport {
                        owner_instance_identity: key.0,
                        property_index: key.1,
                        rule_index,
                        rule_owner_instance_identity: target.0,
                        rule_property_index: target.1,
                        refused: refusal,
                        attempted: change.value.clone(),
                    });
                }
            }
            if let (Some(marker_key), Some(previous_marker)) =
                (marker_key, previous_marker.as_ref())
            {
                self.values.insert(marker_key, previous_marker.clone());
            }
            if !refused {
                if let Some(items) = captured
                    .get(order)
                    .and_then(|change| change.list_items.as_ref())
                {
                    self.native_lists.insert(key, items.clone());
                }
                if let Some(previous) = previous.as_ref() {
                    capture.set_write_publication(order, previous != &change.value);
                }
            }
            if refused {
                let previous = previous.ok_or(RuntimeValuePolicyError::BorrowConflict)?;
                self.values.insert(key, previous);
                omitted.insert(order);
                restore.insert(key);
                if let Some(marker_key) = marker_key {
                    self.values.insert(
                        marker_key,
                        previous_marker.ok_or(RuntimeValuePolicyError::BorrowConflict)?,
                    );
                    self.suppressed.insert(marker_key, refusing_rules);
                    restore.insert(marker_key);
                }
            }
        }
        let mut changed = std::mem::take(&mut self.pending_flush);
        for key in restore {
            changed |= self.restore(capture, key)?;
        }
        // Corrections use the ordinary native setters to dirty bindings. Their
        // rows are redundant with the kept writes and must not re-enter policy.
        capture.omit_writes(&omitted);
        // A surviving value-only write still needs its implicit marker. A
        // later refused value may suppress its own explicit marker, but must
        // not suppress the marker pass for the earlier accepted value.
        let mut last = BTreeMap::new();
        for (order, (owner, change)) in changes.iter().enumerate() {
            if !omitted.contains(&order) {
                last.insert((owner.instance_identity(), change.property_index), order);
            }
        }
        for (owner, change) in &changes {
            let model = owner.borrow().view_model_index();
            for pair in &policy.markers {
                if pair.model != model || pair.value != change.property_index {
                    continue;
                }
                let id = owner.instance_identity();
                if last.get(&(id, pair.value)).is_some_and(|value| {
                    last.get(&(id, pair.marker))
                        .is_none_or(|marker| value > marker)
                }) {
                    self.suppressed.remove(&(id, pair.marker));
                }
            }
        }

        self.cursor = changes
            .len()
            .checked_sub(omitted.len())
            .ok_or(RuntimeValuePolicyError::InvalidArgument)?;
        self.checked_suppressed.clone_from(&self.suppressed);
        Ok(changed)
    }

    fn affected_rules(
        &self,
        policy: &RuntimeValuePolicy,
        owner: &RuntimeOwnedViewModelHandle,
        key: Key,
    ) -> Vec<(usize, Key)> {
        let mut failures = Vec::new();
        for (rule_index, rule) in policy.rules.iter().enumerate() {
            for (rule_owner_id, rule_owner) in &self.owners {
                if rule_owner.borrow().view_model_index() != rule.model {
                    continue;
                }
                let target = (*rule_owner_id, rule.property);
                let direct = target == key
                    || policy.markers.iter().any(|pair| {
                        pair.model == rule.model
                            && pair.value == rule.property
                            && *rule_owner_id == key.0
                            && pair.marker == key.1
                    });
                let picked = picked_property(policy, rule).is_some_and(|property| {
                        owner.borrow().unique_boolean_property_index_by_name(property) == Some(key.1)
                            && matches!(self.values.get(&target), Some(RuntimeViewModelChangeValue::List(items)) if items.contains(&key.0))
                    });
                if direct || picked {
                    failures.push((rule_index, target));
                }
            }
        }
        failures
    }

    fn failures(
        &self,
        policy: &RuntimeValuePolicy,
        owner: &RuntimeOwnedViewModelHandle,
        key: Key,
    ) -> Vec<(usize, Key, bool)> {
        self.affected_rules(policy, owner, key)
            .into_iter()
            .filter_map(|(index, target)| {
                let rule = policy.rules.get(index)?;
                (!self.holds(policy, rule, target)).then(|| {
                    (
                        index,
                        target,
                        rule.entry.mode == RuntimeValueRuleMode::Refuse
                            && !self.empty(policy, rule, target),
                    )
                })
            })
            .collect()
    }

    fn empty(&self, policy: &RuntimeValuePolicy, rule: &Rule, key: Key) -> bool {
        if policy.markers.iter().any(|pair| {
            pair.model == rule.model
                && pair.value == rule.property
                && matches!(
                    self.values.get(&(key.0, pair.marker)),
                    Some(RuntimeViewModelChangeValue::Boolean(false))
                )
        }) {
            return true;
        }
        match self.values.get(&key) {
            Some(RuntimeViewModelChangeValue::String(value)) => value.is_empty(),
            Some(RuntimeViewModelChangeValue::List(items)) => {
                items.is_empty()
                    || self
                        .values
                        .get(&key)
                        .and_then(|value| self.observation(policy, rule, key, value))
                        .is_some_and(|value| {
                            matches!(
                                value,
                                RuntimeRuleValue::List {
                                    picked: Some(0),
                                    ..
                                }
                            )
                        })
            }
            _ => false,
        }
    }

    fn restore(
        &self,
        capture: &RuntimeViewModelChangeCapture,
        key: Key,
    ) -> Result<bool, RuntimeValuePolicyError> {
        let owner = self
            .owners
            .get(&key.0)
            .ok_or(RuntimeValuePolicyError::BorrowConflict)?;
        let property = owner
            .borrow()
            .property_by_path(&[key.1])
            .ok_or(RuntimeValuePolicyError::BorrowConflict)?;
        let kept = self
            .values
            .get(&key)
            .ok_or(RuntimeValuePolicyError::BorrowConflict)?;
        if read(&property).as_ref() == Some(kept) {
            return Ok(false);
        }
        capture.correcting(instance::identity(&property) as usize, || {
            let changed = match kept {
                RuntimeViewModelChangeValue::Number(value) => owner
                    .borrow_mut()
                    .set_number_by_property_index(key.1, *value),
                RuntimeViewModelChangeValue::Boolean(value) => owner
                    .borrow_mut()
                    .set_boolean_by_property_index(key.1, *value),
                RuntimeViewModelChangeValue::Color(value) => owner
                    .borrow_mut()
                    .set_color_by_property_index(key.1, *value),
                RuntimeViewModelChangeValue::Enum(value) => {
                    owner.borrow_mut().set_enum_by_property_index(key.1, *value)
                }
                RuntimeViewModelChangeValue::String(value) => owner
                    .borrow_mut()
                    .set_string_by_property_index(key.1, value),
                RuntimeViewModelChangeValue::List(_) => {
                    let items = self
                        .native_lists
                        .get(&key)
                        .ok_or(RuntimeValuePolicyError::BorrowConflict)?;
                    property
                        .with_downcast_mut::<ViewModelInstanceList, _>(|list| {
                            list.remove_all_items();
                            for item in items {
                                list.add_item(item.clone());
                            }
                        })
                        .ok_or(RuntimeValuePolicyError::BorrowConflict)?;
                    true
                }
                _ => return Err(RuntimeValuePolicyError::InvalidArgument),
            };
            Ok(changed)
        })
    }

    fn absorb_initial(&mut self) -> Result<(), RuntimeValuePolicyError> {
        let initial = self.initial.borrow();
        if initial.overflowed {
            return Err(RuntimeValuePolicyError::LimitExceeded);
        }
        for (id, owner) in &initial.owners {
            if self.owners.contains_key(id) {
                continue;
            }
            if self.owners.len() >= 4_096 {
                return Err(RuntimeValuePolicyError::LimitExceeded);
            }
            self.owners.insert(*id, owner.clone());
            for (key, value) in initial.values.range((*id, 0)..=(*id, usize::MAX)) {
                self.values.insert(*key, value.clone());
            }
            for (key, items) in initial.lists.range((*id, 0)..=(*id, usize::MAX)) {
                self.native_lists.insert(*key, items.clone());
            }
        }
        Ok(())
    }

    fn retain(
        &mut self,
        policy: &RuntimeValuePolicy,
        roots: &[RuntimeOwnedViewModelHandle],
    ) -> Result<(), RuntimeValuePolicyError> {
        let mut pending = roots.to_vec();
        while let Some(root) = pending.pop() {
            if !root.native_file().ptr_eq(&policy.file) {
                return Err(RuntimeValuePolicyError::InvalidArgument);
            }
            for owner in root
                .reachable_change_owner_snapshot()
                .ok_or(RuntimeValuePolicyError::BorrowConflict)?
            {
                let id = owner.instance_identity();
                if self.owners.contains_key(&id) {
                    continue;
                }
                if self.owners.len() == 4_096 {
                    return Err(RuntimeValuePolicyError::LimitExceeded);
                }
                let native = owner.native_handle();
                let parents = native
                    .with_downcast::<ViewModelInstance, _>(ViewModelInstance::parents)
                    .ok_or(RuntimeValuePolicyError::BorrowConflict)?;
                for parent in parents {
                    pending.push(
                        RuntimeOwnedViewModelHandle::from_native(policy.file.clone(), parent)
                            .ok_or(RuntimeValuePolicyError::BorrowConflict)?,
                    );
                }
                let properties = native
                    .with_downcast::<ViewModelInstance, _>(|owner| owner.property_values().to_vec())
                    .ok_or(RuntimeValuePolicyError::BorrowConflict)?;
                for property in properties {
                    let index = property
                        .with(|value| {
                            value
                                .as_view_model_instance_value()
                                .map(|value| value.base.view_model_property_id() as usize)
                        })
                        .flatten()
                        .ok_or(RuntimeValuePolicyError::BorrowConflict)?;
                    if let Some(items) =
                        property.with_downcast::<ViewModelInstanceList, _>(|list| {
                            list.list_items().to_vec()
                        })
                    {
                        self.native_lists.insert((id, index), items);
                    }
                    if let Some(value) = read(&property) {
                        self.values.insert((id, index), value);
                    }
                }
                self.owners.insert(id, owner);
            }
        }
        Ok(())
    }

    fn observation<'a>(
        &'a self,
        policy: &RuntimeValuePolicy,
        rule: &Rule,
        key: Key,
        value: &'a RuntimeViewModelChangeValue,
    ) -> Option<RuntimeRuleValue<'a>> {
        match value {
            RuntimeViewModelChangeValue::Number(value) => Some(RuntimeRuleValue::Number(*value)),
            RuntimeViewModelChangeValue::String(value) => {
                Some(RuntimeRuleValue::Text(std::str::from_utf8(value).ok()?))
            }
            RuntimeViewModelChangeValue::List(items) => {
                let picked_property = picked_property(policy, rule);
                let picked = if let Some(property) = picked_property {
                    let mut count = 0usize;
                    for id in items {
                        let owner = self.owners.get(id)?;
                        let index = owner
                            .borrow()
                            .unique_boolean_property_index_by_name(property)?;
                        match self.values.get(&(*id, index))? {
                            RuntimeViewModelChangeValue::Boolean(value) => {
                                count = count.saturating_add(usize::from(*value))
                            }
                            _ => return None,
                        }
                    }
                    Some(count)
                } else {
                    None
                };
                Some(RuntimeRuleValue::List {
                    items: items.len(),
                    picked,
                })
            }
            RuntimeViewModelChangeValue::Boolean(_) | RuntimeViewModelChangeValue::Color(_) => {
                Some(RuntimeRuleValue::Scalar)
            }
            // Enum labels are resolved separately because the native catalog
            // owns the string returned by its enum mapping.
            RuntimeViewModelChangeValue::Enum(_) => {
                let _ = key;
                None
            }
            _ => None,
        }
    }

    fn holds(&self, policy: &RuntimeValuePolicy, rule: &Rule, key: Key) -> bool {
        let Some(value) = self.values.get(&key) else {
            return false;
        };
        let marker = policy
            .markers
            .iter()
            .find(|marker| marker.model == rule.model && marker.value == rule.property)
            .and_then(|marker| match self.values.get(&(key.0, marker.marker)) {
                Some(RuntimeViewModelChangeValue::Boolean(value)) => Some(*value),
                _ => None,
            });
        if marker == Some(false) && !matches!(rule.compiled.kind(), RuntimeValueRuleKind::Required)
        {
            return true;
        }
        if let RuntimeViewModelChangeValue::Enum(index) = value {
            let label = self.owners.get(&key.0).and_then(|owner| {
                let native = owner.borrow().property_by_path(&[key.1])?;
                native
                    .with_downcast::<ViewModelInstanceEnum, _>(|value| {
                        let property = value.base.view_model_property()?;
                        property
                            .with(|property| {
                                let property = property.as_view_model_property_enum()?;
                                let index = u32::try_from(*index).ok()?;
                                (property.value_index_at(index) >= 0)
                                    .then(|| property.value_at(index))
                            })
                            .flatten()
                    })
                    .flatten()
            });
            return rule
                .compiled
                .holds(RuntimeRuleValue::Enum(label.as_deref()), marker);
        }
        self.observation(policy, rule, key, value)
            .is_some_and(|value| rule.compiled.holds(value, marker))
    }
}

pub(super) fn read(property: &CoreHandle) -> Option<RuntimeViewModelChangeValue> {
    property
        .with(|property| {
            let value = property.as_any();
            if let Some(value) = value.downcast_ref::<ViewModelInstanceNumber>() {
                Some(RuntimeViewModelChangeValue::Number(value.value()))
            } else if let Some(value) = value.downcast_ref::<ViewModelInstanceBoolean>() {
                Some(RuntimeViewModelChangeValue::Boolean(value.value()))
            } else if let Some(value) = value.downcast_ref::<ViewModelInstanceColor>() {
                Some(RuntimeViewModelChangeValue::Color(value.value() as u32))
            } else if let Some(value) = value.downcast_ref::<ViewModelInstanceEnum>() {
                Some(RuntimeViewModelChangeValue::Enum(u64::from(
                    value.base.property_value(),
                )))
            } else if let Some(value) = value.downcast_ref::<ViewModelInstanceString>() {
                Some(RuntimeViewModelChangeValue::String(Arc::from(
                    value.value().as_bytes(),
                )))
            } else {
                let list = value.downcast_ref::<ViewModelInstanceList>()?;
                let items = list
                    .list_items()
                    .iter()
                    .map(|item| {
                        item.with_downcast::<ViewModelInstanceListItem, _>(
                            ViewModelInstanceListItem::view_model_instance,
                        )
                        .flatten()
                        .map(|value| instance::identity(&value))
                    })
                    .collect::<Option<Vec<_>>>()?;
                Some(RuntimeViewModelChangeValue::List(items))
            }
        })
        .flatten()
}

fn picked_property<'a>(policy: &'a RuntimeValuePolicy, rule: &'a Rule) -> Option<&'a str> {
    match rule.compiled.kind() {
        RuntimeValueRuleKind::PickedCount { property, .. } => Some(property),
        RuntimeValueRuleKind::Required => policy.rules.iter().find_map(|other| {
            if (other.model, other.property) != (rule.model, rule.property) {
                return None;
            }
            match other.compiled.kind() {
                RuntimeValueRuleKind::PickedCount { property, .. } => Some(property.as_str()),
                _ => None,
            }
        }),
        _ => None,
    }
}
