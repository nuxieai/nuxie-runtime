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
        write: impl FnOnce() -> Result<(), RuntimeValuePolicyError>,
    ) -> Result<RuntimeCheckedValueBatchResult, RuntimeValuePolicyError> {
        if !self.initial.borrow().file.ptr_eq(&policy.file) {
            return Err(RuntimeValuePolicyError::InvalidArgument);
        }
        let roots = roots.values().cloned().collect::<Vec<_>>();
        self.apply_pending_writes(policy, &roots)?;
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
        let mut reported = BTreeSet::new();
        for (value, failures) in &outcomes {
            let key = (value.owner.instance_identity(), value.index);
            if !refused {
                staged.value_reports.remove(&key);
            }
            staged.record_group_outcome(policy, &value.owner, key, failures, refused)?;
            for &(rule_index, target, refusal) in failures {
                if (refusal || !refused) && reported.insert((rule_index, target)) {
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
                    let indices = outcomes
                        .iter()
                        .flat_map(|(_, failures)| failures)
                        .filter_map(|(index, _, refusal)| refusal.then_some(*index))
                        .collect::<BTreeSet<_>>()
                        .into_iter()
                        .collect::<Vec<_>>();
                    staged.suppressed.insert(marker_key, indices.clone());
                    staged.checked_suppressed.insert(marker_key, indices);
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
            write()?;
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
