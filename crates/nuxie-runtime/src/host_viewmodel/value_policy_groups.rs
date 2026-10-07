use super::*;
use crate::mechanical_port::source::viewmodel::viewmodel_property_viewmodel::ViewModelPropertyViewModel;

#[derive(Clone, Debug)]
pub struct RuntimeRuleGroupMember {
    pub property: String,
    /// A nested property path ending in a list.
    pub errors_path: String,
    pub item_model: String,
    pub code_property: String,
    pub message_property: String,
}

#[derive(Clone, Debug)]
pub struct RuntimeRuleGroup {
    pub model: String,
    pub valid: String,
    pub members: Vec<RuntimeRuleGroupMember>,
}

pub(crate) struct Group {
    model: usize,
    valid: usize,
    members: Vec<Member>,
}

struct Member {
    property: usize,
    path: Vec<usize>,
    target: (usize, usize),
    item_model: usize,
    code: usize,
    message: usize,
}

impl RuntimeValuePolicy {
    /// Replace the whole group table atomically. Install rules and markers
    /// first. Each output list contains the active rules in installer order.
    pub fn set_groups(
        &mut self,
        entries: &[RuntimeRuleGroup],
    ) -> Result<(), RuntimeValuePolicyError> {
        let mut groups = Vec::new();
        let mut targets = BTreeSet::new();
        let mut count = entries.len();
        let mut bytes = 0usize;
        for entry in entries {
            count = count.saturating_add(entry.members.len());
            bytes = bytes
                .saturating_add(entry.model.len())
                .saturating_add(entry.valid.len());
            if count > 4_096 {
                return Err(RuntimeValuePolicyError::LimitExceeded);
            }
            let (model, valid, native) = self.property(&entry.model, &entry.valid)?;
            if !native.is_type_of(ViewModelPropertyBooleanBase::TYPE_KEY)
                || !targets.insert((model, vec![valid]))
                || entry.members.is_empty()
            {
                return Err(RuntimeValuePolicyError::InvalidArgument);
            }
            let mut members = Vec::new();
            let mut values = BTreeSet::new();
            for member in &entry.members {
                bytes = bytes
                    .saturating_add(member.property.len())
                    .saturating_add(member.errors_path.len())
                    .saturating_add(member.item_model.len())
                    .saturating_add(member.code_property.len())
                    .saturating_add(member.message_property.len());
                if bytes > 8 * 1024 * 1024 {
                    return Err(RuntimeValuePolicyError::LimitExceeded);
                }
                let (_, mut property, _) = self.property(&entry.model, &member.property)?;
                if let Some(marker) = self
                    .markers
                    .iter()
                    .find(|pair| pair.model == model && pair.marker == property)
                {
                    property = marker.value;
                }
                if !values.insert(property)
                    || !self
                        .rules
                        .iter()
                        .any(|rule| rule.model == model && rule.property == property)
                        && !self
                            .markers
                            .iter()
                            .any(|pair| pair.model == model && pair.value == property)
                {
                    return Err(RuntimeValuePolicyError::InvalidArgument);
                }
                let mut path = Vec::new();
                let mut current_model = entry.model.clone();
                let mut parts = member.errors_path.split('/').peekable();
                let target = loop {
                    let part = parts
                        .next()
                        .ok_or(RuntimeValuePolicyError::InvalidArgument)?;
                    if part.is_empty() {
                        return Err(RuntimeValuePolicyError::InvalidArgument);
                    }
                    let (model, index, native) = self.property(&current_model, part)?;
                    path.push(index);
                    if parts.peek().is_none() {
                        if !native.is_type_of(ViewModelPropertyListBase::TYPE_KEY) {
                            return Err(RuntimeValuePolicyError::InvalidArgument);
                        }
                        break (model, index);
                    }
                    let next = native
                        .with_downcast::<ViewModelPropertyViewModel, _>(|property| {
                            property.base.view_model_reference_id() as usize
                        })
                        .ok_or(RuntimeValuePolicyError::InvalidArgument)?;
                    current_model = self
                        .file
                        .with_file(|file| {
                            file.view_model(next).and_then(|model| {
                                model
                                    .with(|model| {
                                        model
                                            .as_view_model()
                                            .map(|model| model.base.name().to_owned())
                                    })
                                    .flatten()
                            })
                        })
                        .ok_or(RuntimeValuePolicyError::NotFound)?;
                };
                if !targets.insert((model, path.clone())) {
                    return Err(RuntimeValuePolicyError::InvalidArgument);
                }
                let (item_model, code, code_native) =
                    self.property(&member.item_model, &member.code_property)?;
                let (_, message, message_native) =
                    self.property(&member.item_model, &member.message_property)?;
                if code == message
                    || !code_native.is_type_of(ViewModelPropertyStringBase::TYPE_KEY)
                    || !message_native.is_type_of(ViewModelPropertyStringBase::TYPE_KEY)
                {
                    return Err(RuntimeValuePolicyError::InvalidArgument);
                }
                members.push(Member {
                    property,
                    path,
                    target,
                    item_model,
                    code,
                    message,
                });
            }
            groups.push(Group {
                model,
                valid,
                members,
            });
        }
        validate_inputs(&groups, &self.rules, &self.markers)?;
        self.groups = groups;
        self.group_refusals.borrow_mut().clear();
        Ok(())
    }

    pub(in crate::host_viewmodel::value_policy) fn validate_group_inputs(
        &self,
        rules: &[Rule],
        markers: &[Marker],
    ) -> Result<(), RuntimeValuePolicyError> {
        validate_inputs(&self.groups, rules, markers)
    }
}

fn validate_inputs(
    groups: &[Group],
    rules: &[Rule],
    markers: &[Marker],
) -> Result<(), RuntimeValuePolicyError> {
    let mut computed = BTreeSet::new();
    for group in groups {
        computed.insert((group.model, group.valid));
        for member in &group.members {
            computed.extend([
                member.target,
                (member.item_model, member.code),
                (member.item_model, member.message),
            ]);
        }
    }
    if rules
        .iter()
        .any(|rule| computed.contains(&(rule.model, rule.property)))
        || markers.iter().any(|pair| {
            computed.contains(&(pair.model, pair.value))
                || computed.contains(&(pair.model, pair.marker))
        })
    {
        return Err(RuntimeValuePolicyError::InvalidArgument);
    }
    Ok(())
}

impl RuntimeValuePolicyOperation {
    pub(super) fn record_group_outcome(
        &mut self,
        policy: &RuntimeValuePolicy,
        owner: &RuntimeOwnedViewModelHandle,
        key: Key,
        failures: &[(usize, Key, bool)],
        refused: bool,
    ) -> Result<(), RuntimeValuePolicyError> {
        if !policy.has_groups() {
            return Ok(());
        }
        let targets = self
            .affected_rules(policy, owner, key)
            .into_iter()
            .map(|(_, target)| target)
            .collect::<BTreeSet<_>>();
        for target in targets {
            if !policy.groups.iter().any(|group| {
                self.owners
                    .get(&target.0)
                    .is_some_and(|owner| owner.borrow().view_model_index() == group.model)
                    && group
                        .members
                        .iter()
                        .any(|member| member.property == target.1)
            }) {
                continue;
            }
            if !refused {
                self.group_refusals.remove(&target);
            } else {
                let indices = failures
                    .iter()
                    .filter(|(_, candidate, refusal)| *candidate == target && *refusal)
                    .map(|(index, _, _)| *index)
                    .collect::<Vec<_>>();
                if !indices.is_empty() {
                    self.group_refusals.insert(target, indices);
                }
            }
        }
        if self.group_refusals.len() > 4_096
            || self.group_refusals.values().map(Vec::len).sum::<usize>() > 4_096
        {
            return Err(RuntimeValuePolicyError::LimitExceeded);
        }
        Ok(())
    }

    /// Compute outputs from current native values and this operation's latest
    /// refusal. The caller checkpoints each target before the ordinary write.
    /// Repeat after binding updates until quiet, before publishing the journal.
    pub fn apply_groups(
        &mut self,
        policy: &RuntimeValuePolicy,
        roots: &[RuntimeOwnedViewModelHandle],
        mut checkpoint: impl FnMut(
            &RuntimeOwnedViewModelHandle,
            usize,
        ) -> Result<(), RuntimeValuePolicyError>,
    ) -> Result<bool, RuntimeValuePolicyError> {
        if !policy.has_groups() {
            return Ok(false);
        }
        self.absorb_initial()?;
        self.retain(policy, roots)?;
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
        };
        probe.retain(policy, &self.retained_roots())?;
        let mut occupied = BTreeSet::new();
        let mut changed = false;
        let mut error_count = 0usize;
        let mut error_bytes = 0usize;
        for group in &policy.groups {
            for (id, owner) in &probe.owners {
                if owner.borrow().view_model_index() != group.model {
                    continue;
                }
                if !occupied.insert((*id, group.valid)) {
                    return Err(RuntimeValuePolicyError::InvalidArgument);
                }
                let mut valid = true;
                for member in &group.members {
                    let key = (*id, member.property);
                    let mut errors = Vec::new();
                    for (index, rule) in policy.rules.iter().enumerate() {
                        if rule.model != group.model || rule.property != member.property {
                            continue;
                        }
                        let holds = probe.holds(policy, rule, key);
                        valid &= holds;
                        if !holds
                            || self
                                .group_refusals
                                .get(&key)
                                .is_some_and(|indices| indices.contains(&index))
                        {
                            error_count = error_count.saturating_add(1);
                            error_bytes = error_bytes
                                .saturating_add(rule.entry.code.len())
                                .saturating_add(rule.entry.message.len());
                            if error_count > 4_096 || error_bytes > 8 * 1024 * 1024 {
                                return Err(RuntimeValuePolicyError::LimitExceeded);
                            }
                            errors.push((&rule.entry.code, &rule.entry.message));
                        }
                    }
                    let (index, parents) = member
                        .path
                        .split_last()
                        .ok_or(RuntimeValuePolicyError::InvalidArgument)?;
                    let target = if parents.is_empty() {
                        owner.clone()
                    } else {
                        owner
                            .linked_view_model_by_property_path(parents)
                            .ok_or(RuntimeValuePolicyError::NotFound)?
                    };
                    if !occupied.insert((target.instance_identity(), *index)) {
                        return Err(RuntimeValuePolicyError::InvalidArgument);
                    }
                    let property = target
                        .borrow()
                        .property_by_path(&[*index])
                        .ok_or(RuntimeValuePolicyError::NotFound)?;
                    let items = property
                        .with_downcast::<ViewModelInstanceList, _>(|list| {
                            list.list_items().to_vec()
                        })
                        .ok_or(RuntimeValuePolicyError::InvalidArgument)?;
                    let equal = items.len() == errors.len()
                        && items.iter().zip(&errors).all(|(item, (code, message))| {
                            let Some(native) = item
                                .with_downcast::<ViewModelInstanceListItem, _>(
                                    ViewModelInstanceListItem::view_model_instance,
                                )
                                .flatten()
                            else {
                                return false;
                            };
                            let Some(item) = RuntimeOwnedViewModelHandle::from_native(
                                policy.file.clone(),
                                native,
                            ) else {
                                return false;
                            };
                            if item.borrow().view_model_index() != member.item_model {
                                return false;
                            }
                            let value = |index| {
                                item.borrow()
                                    .property_by_path(&[index])
                                    .and_then(|property| read(&property))
                            };
                            value(member.code)
                                == Some(RuntimeViewModelChangeValue::String(Arc::from(
                                    code.as_bytes(),
                                )))
                                && value(member.message)
                                    == Some(RuntimeViewModelChangeValue::String(Arc::from(
                                        message.as_bytes(),
                                    )))
                        });
                    if equal {
                        continue;
                    }
                    checkpoint(&target, *index)?;
                    let mut replacement = Vec::with_capacity(errors.len());
                    for (code, message) in errors {
                        let item = RuntimeOwnedViewModelInstance::new(
                            policy.file.clone(),
                            member.item_model,
                        )
                        .ok_or(RuntimeValuePolicyError::BorrowConflict)?;
                        let item = RuntimeOwnedViewModelHandle::new(item);
                        item.borrow_mut()
                            .set_string_by_property_index(member.code, code.as_bytes());
                        item.borrow_mut()
                            .set_string_by_property_index(member.message, message.as_bytes());
                        let mut native_item = ViewModelInstanceListItem::default();
                        native_item.set_view_model_instance(Some(item.native_handle()));
                        replacement.push(
                            policy
                                .file
                                .with_file(|file| file.core_arena().insert(native_item)),
                        );
                    }
                    property
                        .with_downcast_mut::<ViewModelInstanceList, _>(|list| {
                            list.remove_all_items();
                            for item in replacement {
                                list.add_item(item);
                            }
                        })
                        .ok_or(RuntimeValuePolicyError::BorrowConflict)?;
                    changed = true;
                }
                let property = owner
                    .borrow()
                    .property_by_path(&[group.valid])
                    .ok_or(RuntimeValuePolicyError::NotFound)?;
                if read(&property) != Some(RuntimeViewModelChangeValue::Boolean(valid)) {
                    checkpoint(owner, group.valid)?;
                    owner
                        .borrow_mut()
                        .set_boolean_by_property_index(group.valid, valid);
                    changed = true;
                }
            }
        }
        self.absorb_initial()?;
        self.retain(policy, roots)?;
        Ok(changed)
    }

    /// Commit only after the caller successfully publishes its native
    /// transaction. Dropping a failed operation discards its staged refusals.
    pub fn commit_groups(&mut self, policy: &RuntimeValuePolicy) {
        if policy.has_groups() {
            *policy.group_refusals.borrow_mut() = std::mem::take(&mut self.group_refusals);
        }
    }
}
