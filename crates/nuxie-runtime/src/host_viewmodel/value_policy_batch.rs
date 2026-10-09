//! Host-only whole-replacement policy planning. Native file behavior is unchanged.
use super::*;
use crate::host_viewmodel::checked_batch::CheckedBatchValue;
use crate::view_model_cell::RuntimeViewModelCapturedChange;

impl RuntimeValuePolicyOperation {
    pub(in crate::host_viewmodel) fn checked_batch(
        &mut self,
        policy: &RuntimeValuePolicy,
        roots: &BTreeMap<String, RuntimeOwnedViewModelHandle>,
        values: &[CheckedBatchValue],
        capture: &RuntimeViewModelChangeCapture,
        write: impl FnOnce(),
    ) -> Result<RuntimeCheckedValueBatchResult, RuntimeValuePolicyError> {
        if !self.initial.borrow().file.ptr_eq(&policy.file) {
            return Err(RuntimeValuePolicyError::InvalidArgument);
        }
        let roots = roots.values().cloned().collect::<Vec<_>>();
        self.apply_pending_writes(policy, &roots)?;
        // The batch rows go straight after the replayed journal. A row left
        // unreplayed here would otherwise be skipped by the cursor jump below.
        if policy.has_rules()
            && capture
                .write_count()
                .map_err(|_| RuntimeValuePolicyError::LimitExceeded)?
                != self.cursor
        {
            return Err(RuntimeValuePolicyError::BorrowConflict);
        }
        self.absorb_initial()?;
        self.retain(policy, &roots)?;
        for value in values.iter().filter(|value| !value.implicit_marker) {
            if let Some(indices) = self
                .checked_suppressed
                .get(&(value.owner.instance_identity(), value.index))
            {
                if let Some(index) = indices.first() {
                    return batch_result(policy, Some(*index));
                }
            }
        }
        // An operation-local plan, discarded on any preflight error. Native
        // values stay in Core; this extends the existing scalar candidate probe.
        let mut staged = self.batch_plan();
        for value in values {
            staged.values.insert(
                (value.owner.instance_identity(), value.index),
                value.value.clone(),
            );
        }
        let outcomes = values
            .iter()
            .filter(|value| !value.implicit_marker)
            .map(|value| {
                let key = (value.owner.instance_identity(), value.index);
                (value, staged.failures(policy, &value.owner, key))
            })
            .collect::<Vec<_>>();
        let first_refusal = outcomes
            .iter()
            .flat_map(|(_, failures)| failures)
            .find_map(|(index, _, refused)| refused.then_some(*index));
        let refused = first_refusal.is_some();
        let result = batch_result(policy, first_refusal)?;
        let reporters = Self::reporters(policy, self, values, &outcomes, refused);
        for (position, (value, failures)) in outcomes.iter().enumerate() {
            let key = (value.owner.instance_identity(), value.index);
            if !refused {
                staged.value_reports.remove(&key);
            }
            staged.record_group_outcome(policy, &value.owner, key, failures, refused)?;
            for &(rule_index, target, refusal) in failures {
                if (refusal || !refused) && reporters.get(&(rule_index, target)) == Some(&position)
                {
                    if key == target {
                        staged
                            .value_reports
                            .entry(key)
                            .or_default()
                            .insert(rule_index);
                    }
                    staged.push_report(RuntimeValueRuleReport {
                        owner_instance_identity: key.0,
                        property_index: key.1,
                        rule_index,
                        rule_owner_instance_identity: target.0,
                        rule_property_index: target.1,
                        refused: refusal,
                        attempted: value.value.clone(),
                    })?;
                }
            }
            if let Some(marker) = policy.marker_property(&value.owner, value.index) {
                let marker_key = (key.0, marker);
                if refused {
                    // Only this entry's own refusing rules block its marker; an
                    // entry no rule refused keeps its earlier marker state.
                    let indices = failures
                        .iter()
                        .filter_map(|(index, _, refusal)| refusal.then_some(*index))
                        .collect::<BTreeSet<_>>()
                        .into_iter()
                        .collect::<Vec<_>>();
                    if !indices.is_empty() {
                        staged.suppressed.insert(marker_key, indices.clone());
                        staged.checked_suppressed.insert(marker_key, indices);
                    }
                } else {
                    staged.suppressed.remove(&marker_key);
                    staged.checked_suppressed.remove(&marker_key);
                }
            }
        }
        if refused {
            staged.values.clone_from(&self.values);
            // A refused batch contributes reports and errors, never native rows.
        } else {
            let journal = values
                .iter()
                .map(|value| RuntimeViewModelCapturedChange {
                    cell_identity: instance::identity(&value.property) as usize,
                    publish: read(&value.property).as_ref() != Some(&value.value),
                    list_items: matches!(value.value, RuntimeViewModelChangeValue::List(_))
                        .then(Vec::new),
                    value: value.value.clone(),
                })
                .collect();
            // Reserve the full batch before any setter. Its rows are already
            // checked against the final replacement, so policy replay starts
            // after them. Setters suppress only their own duplicate capture;
            // dependency writes remain after this cursor and are checked normally.
            staged.cursor = capture
                .append_checked_batch(journal)
                .map_err(|_| RuntimeValuePolicyError::LimitExceeded)?;
            write();
            for value in values {
                if matches!(value.value, RuntimeViewModelChangeValue::List(_)) {
                    staged
                        .native_lists
                        .insert((value.owner.instance_identity(), value.index), Vec::new());
                }
            }
        }
        if !values.is_empty() {
            policy.invalidate();
            staged.pending_flush = true;
        }
        *self = staged;
        Ok(result)
    }

    /// Name one entry for each reported rule and target: the first entry that
    /// breaks that rule when written alone over the values before the batch
    /// (the scalar write's view), else the first entry that changes its value,
    /// else the first entry the rule affects. A deselect that only accompanies
    /// the breaking selections is never named for a maximum.
    fn reporters(
        policy: &RuntimeValuePolicy,
        before: &Self,
        values: &[CheckedBatchValue],
        outcomes: &[(&CheckedBatchValue, Vec<(usize, Key, bool)>)],
        refused: bool,
    ) -> BTreeMap<(usize, Key), usize> {
        let mut candidates = BTreeMap::<(usize, Key), Vec<usize>>::new();
        for (position, (_, failures)) in outcomes.iter().enumerate() {
            for &(rule_index, target, refusal) in failures {
                if refusal || !refused {
                    candidates
                        .entry((rule_index, target))
                        .or_default()
                        .push(position);
                }
            }
        }
        let mut reporters = BTreeMap::new();
        if candidates.is_empty() {
            return reporters;
        }
        let mut alone = before.batch_plan();
        for (reported, positions) in candidates {
            let mut breaks_alone = |position: &usize| {
                let value = outcomes[*position].0;
                let key = (value.owner.instance_identity(), value.index);
                // The entry and its implicit paired marker, as one scalar write.
                let written = values
                    .iter()
                    .filter(|other| {
                        other.owner.instance_identity() == key.0
                            && (other.index == key.1
                                || (other.implicit_marker
                                    && policy.marker_property(&other.owner, key.1)
                                        == Some(other.index)))
                    })
                    .map(|other| {
                        let other_key = (key.0, other.index);
                        let previous = alone.values.insert(other_key, other.value.clone());
                        (other_key, previous)
                    })
                    .collect::<Vec<_>>();
                let breaks = alone
                    .failures(policy, &value.owner, key)
                    .iter()
                    .any(|&(rule_index, target, _)| (rule_index, target) == reported);
                for (other_key, previous) in written.into_iter().rev() {
                    match previous {
                        Some(previous) => alone.values.insert(other_key, previous),
                        None => alone.values.remove(&other_key),
                    };
                }
                breaks
            };
            let changes = |position: &usize| {
                let value = outcomes[*position].0;
                before
                    .values
                    .get(&(value.owner.instance_identity(), value.index))
                    != Some(&value.value)
            };
            let reporter = positions
                .iter()
                .copied()
                .find(|position| breaks_alone(position))
                .or_else(|| positions.iter().copied().find(|position| changes(position)))
                .unwrap_or(positions[0]);
            reporters.insert(reported, reporter);
        }
        reporters
    }

    fn batch_plan(&self) -> Self {
        Self {
            owners: self.owners.clone(),
            values: self.values.clone(),
            native_lists: self.native_lists.clone(),
            suppressed: self.suppressed.clone(),
            checked_suppressed: self.checked_suppressed.clone(),
            cursor: self.cursor,
            pending_flush: self.pending_flush,
            reports: self.reports.clone(),
            report_bytes: self.report_bytes,
            value_reports: self.value_reports.clone(),
            initial: Rc::clone(&self.initial),
            group_refusals: self.group_refusals.clone(),
            group_roots: self.group_roots.clone(),
            groups_revision: self.groups_revision,
        }
    }
}

fn batch_result(
    policy: &RuntimeValuePolicy,
    index: Option<usize>,
) -> Result<RuntimeCheckedValueBatchResult, RuntimeValuePolicyError> {
    let refusal = index
        .map(|index| {
            let rule = policy
                .rule(index)
                .ok_or(RuntimeValuePolicyError::InvalidArgument)?;
            Ok(RuntimeCheckedValueRefusal {
                rule_index: index,
                code: rule.code.clone(),
                message: rule.message.clone(),
            })
        })
        .transpose()?;
    Ok(RuntimeCheckedValueBatchResult {
        applied: refusal.is_none(),
        refusal,
    })
}
