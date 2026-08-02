use crate::ArtboardInstance;
use crate::scripting::{NoopScriptHost, ScriptMethod, ScriptOptionalMethodResult, ScriptValue};

impl ArtboardInstance {
    /// Resolve the retained layout node that owns one `ScriptedLayout`.
    ///
    /// C++ exposes the scripted object as an `IntrinsicallySizeable` child;
    /// its direct parent `LayoutComponent` owns the actual Yoga node.
    #[doc(hidden)]
    pub fn scripted_layout_node(&self, scripted_local: usize) -> Option<usize> {
        if self.component(scripted_local)?.type_name != "ScriptedLayout" {
            return None;
        }
        let parent = self.component_parent_local(scripted_local)?;
        (self.component(parent)?.type_name == "LayoutComponent").then_some(parent)
    }

    /// `ScriptedLayout::didHydrateScriptInputs`: force the owning layout node
    /// to publish its settled bounds even when hydration did not change them.
    #[doc(hidden)]
    pub fn did_hydrate_scripted_layout(&mut self, scripted_local: usize) -> bool {
        let Some(scripted_global) = self
            .component(scripted_local)
            .filter(|component| component.type_name == "ScriptedLayout")
            .map(|component| component.global_id)
        else {
            return false;
        };
        // The C++ override first invokes ScriptedDrawable's hydration hook,
        // which rearms advance and invalidates paint.
        let mut changed = self.wake_script_advance_for_global(scripted_global);
        let Some(layout_local) = self.scripted_layout_node(scripted_local) else {
            return changed;
        };
        if let Some(layout) = self
            .component(layout_local)
            .and_then(|component| component.concrete.layout.as_ref())
        {
            layout.force_update_layout_bounds();
        }
        changed |= self.mark_layout_node_changed(layout_local);
        changed
    }

    #[doc(hidden)]
    pub fn did_hydrate_scripted_layout_for_global(&mut self, scripted_global: u32) -> bool {
        let Some(scripted_local) = self
            .components()
            .iter()
            .find(|component| {
                component.global_id == scripted_global && component.type_name == "ScriptedLayout"
            })
            .map(|component| component.local_id)
        else {
            return false;
        };
        self.did_hydrate_scripted_layout(scripted_local)
    }

    pub(crate) fn measure_scripted_layout(
        &self,
        scripted_local: usize,
        maximum_width: Option<f32>,
        maximum_height: Option<f32>,
    ) -> (f32, f32) {
        let Some(component) = self.component(scripted_local) else {
            return (0.0, 0.0);
        };
        if component.type_name != "ScriptedLayout" {
            return (0.0, 0.0);
        }
        let Some(methods) = self.script_implemented_methods_for_global(component.global_id) else {
            return (0.0, 0.0);
        };
        if !methods.measures() {
            return (0.0, 0.0);
        }
        let Some(handle) = self.script_instance_for_global(component.global_id) else {
            return (0.0, 0.0);
        };
        let measured = handle.borrow_mut().call_optional_method(
            ScriptMethod::Measure,
            &[],
            &mut NoopScriptHost,
        );
        let (width, height) = match measured {
            Ok(ScriptOptionalMethodResult::Missing) => return (0.0, 0.0),
            Ok(ScriptOptionalMethodResult::Returned(
                ScriptValue::Vec2 { x, y } | ScriptValue::Vec3 { x, y, .. },
            )) => (x, y),
            // C++ contains callback errors and non-vector returns, leaving
            // the measured axes at max-float before constraint clamping.
            Ok(_) | Err(_) => (f32::MAX, f32::MAX),
        };
        constrained_measurement(width, height, maximum_width, maximum_height)
    }

    pub(crate) fn control_scripted_layouts_for_node(
        &self,
        layout_local: usize,
        width: f32,
        height: f32,
    ) {
        let scripted_locals = self
            .components()
            .iter()
            .filter(|component| component.type_name == "ScriptedLayout")
            .filter_map(|component| {
                self.scripted_layout_controlled_by(component.local_id, layout_local)
                    .then_some(component.local_id)
            })
            .collect::<Vec<_>>();
        for scripted_local in scripted_locals {
            let Some(component) = self.component(scripted_local) else {
                continue;
            };
            let Some(methods) = self.script_implemented_methods_for_global(component.global_id)
            else {
                continue;
            };
            if !methods.resizes() {
                continue;
            }
            let Some(handle) = self.script_instance_for_global(component.global_id) else {
                continue;
            };
            let _ = handle.borrow_mut().call_optional_method(
                ScriptMethod::Resize,
                &[ScriptValue::Vec2 {
                    x: width,
                    y: height,
                }],
                &mut NoopScriptHost,
            );
        }
    }

    pub(crate) fn propagate_scripted_layout_size(&self, layout_local: usize) {
        let Some(component) = self.component(layout_local) else {
            return;
        };
        if self.runtime_component_is_collapsed_for_draw(layout_local) {
            return;
        }
        let drawable_hidden = component
            .concrete
            .drawable
            .as_ref()
            .and_then(|drawable| drawable.drawable_flags_property_key)
            .and_then(|key| self.uint_property(layout_local, key))
            .is_some_and(|flags| flags & 1 != 0);
        if drawable_hidden {
            return;
        }
        let Some(layout) = component.concrete.layout.as_ref() else {
            return;
        };
        // Pinned LayoutComponent propagation requires an attached style and
        // passes the current m_layout size, not an animation's solved target.
        if layout.style.is_none() {
            return;
        }
        let (_, _, width, height) = layout.current_bounds();
        self.control_scripted_layouts_for_node(layout_local, width, height);
    }

    fn scripted_layout_controlled_by(&self, scripted_local: usize, layout_local: usize) -> bool {
        let mut parent = self.component_parent_local(scripted_local);
        while let Some(local_id) = parent {
            let Some(component) = self.component(local_id) else {
                return false;
            };
            if component.type_name == "LayoutComponent" {
                return local_id == layout_local;
            }
            // `LayoutComponent::propagateSizeToChildren` does not traverse an
            // exact Node child. Other container components remain transparent.
            if component.type_name == "Node" {
                return false;
            }
            parent = self.component_parent_local(local_id);
        }
        false
    }
}

fn constrained_measurement(
    width: f32,
    height: f32,
    maximum_width: Option<f32>,
    maximum_height: Option<f32>,
) -> (f32, f32) {
    (
        maximum_width.unwrap_or(f32::MAX).min(width),
        maximum_height.unwrap_or(f32::MAX).min(height),
    )
}
