use crate::mechanical_port::source::{
    core::CoreHandle,
    generated::scripted::scripted_layout_base::ScriptedLayoutBase,
    scripted::scripted_object::{ScriptProtocol, ScriptUpdateRequestHost, ScriptedObject},
};
pub use crate::mechanical_port::source::{
    layout::{
        layout_enums::{LayoutDirection, LayoutScaleType},
        layout_measure_mode::LayoutMeasureMode,
    },
    math::vec2d::Vec2D as Vec2,
};
use crate::scripting::{ScriptMethod, ScriptOptionalMethodResult, ScriptValue};

#[derive(Default)]
pub struct ScriptedLayout {
    pub base: ScriptedLayoutBase,
    size: Vec2,
    size_known: bool,
    natural_measure: Option<Vec2>,
}

impl Drop for ScriptedLayout {
    fn drop(&mut self) {
        ScriptedObject::dispose_owned_script_inputs(&mut self.base.base.properties);
    }
}

crate::mechanical_port::source::transform_component::impl_transform_update!(
    ScriptedLayout,
    |owner, dirt| {
        crate::mechanical_port::source::transform_component::update_transform_super::<Self>(
            owner, dirt,
        );
        if owner.is_alive() {
            crate::mechanical_port::source::scripted::scripted_drawable::ScriptedDrawable::update_after_super_occurrence(owner, dirt);
        }
    },
    crate::mechanical_port::source::transform_component::update_local_transform::<Self>,
    crate::mechanical_port::source::node::Node::update_world_transform_occurrence::<Self>,
    crate::mechanical_port::source::transform_component::compose_world_transform::<Self>,
    crate::mechanical_port::source::transform_component::update_constraints_super::<Self>
);

impl ScriptedLayout {
    /// The size the layout engine last gave this box; absent before it has.
    pub fn layout_size(&self) -> Option<Vec2> {
        if !self.size_known {
            return None;
        }
        Some(self.size)
    }

    pub fn did_hydrate_script_inputs(&mut self) {
        self.base.base.did_hydrate_script_inputs();
        if let Some(parent) = self.base.parent_handle() {
            parent.with_mut(|parent| {
                if let Some(layout) = parent.as_layout_component_mut() {
                    layout.mark_layout_node_dirty(true);
                }
            });
        }
    }

    fn call_scripted_resize(&mut self, size: Vec2) {
        if let Some(call) = self.resize_call(size)
            && call.run()
        {
            self.base.base.mark_needs_update();
        }
    }

    /// What a resize call needs, read out so the script can run with no
    /// borrow of this layout open.
    fn resize_call(&self, size: Vec2) -> Option<ResizeCall> {
        let scripted = &self.base.base.scripted;
        if !scripted.resizes() {
            return None;
        }
        Some(ResizeCall {
            instance: scripted.runtime_instance()?,
            vm: scripted.scripting_vm()?,
            size,
        })
    }

    /// Runs `call` with no borrow of `owner` open, then applies its update
    /// request under a new one.
    fn run_resize_occurrence(owner: &CoreHandle, call: Option<ResizeCall>) {
        if call.is_some_and(ResizeCall::run) {
            owner.with_downcast_mut::<Self, _>(|layout| layout.base.base.mark_needs_update());
        }
    }

    /// Pinned controlSize at handle level: the script's resize runs with no
    /// borrow of the layout open, since user code may reach it. False when
    /// `owner` is not a ScriptedLayout.
    pub(crate) fn control_size_occurrence(owner: &CoreHandle, size: Vec2) -> bool {
        let Some(call) = owner.with_downcast_mut::<Self, _>(|layout| {
            layout.size = size;
            layout.size_known = true;
            layout.resize_call(size)
        }) else {
            return false;
        };
        Self::run_resize_occurrence(owner, call);
        true
    }

    /// Pinned displayScaleChanged at handle level, as
    /// [`Self::control_size_occurrence`]. False when `owner` is not a
    /// ScriptedLayout.
    pub(crate) fn display_scale_changed_occurrence(owner: &CoreHandle) -> bool {
        let Some(call) = owner.with_downcast::<Self, _>(|layout| {
            if layout.size_known {
                layout.resize_call(layout.size)
            } else {
                None
            }
        }) else {
            return false;
        };
        Self::run_resize_occurrence(owner, call);
        true
    }

    pub fn measure_layout(
        &mut self,
        width: f32,
        width_mode: LayoutMeasureMode,
        height: f32,
        height_mode: LayoutMeasureMode,
    ) -> Vec2 {
        if !self.base.base.scripted.measures() {
            return Vec2::default();
        }
        let measured = if crate::page_player::IntrinsicProbe::active() {
            match self.natural_measure {
                Some(size) => size,
                None => {
                    crate::page_player::IntrinsicProbe::unavailable();
                    Vec2::default()
                }
            }
        } else {
            let Some(instance) = self.base.base.scripted.runtime_instance() else {
                return Vec2::default();
            };
            let mut host = ScriptUpdateRequestHost::default();
            let result =
                instance
                    .borrow_mut()
                    .call_optional_method(ScriptMethod::Measure, &[], &mut host);
            if host.take_requested() {
                self.base.base.mark_needs_update();
            }
            let measured = match result {
                Ok(ScriptOptionalMethodResult::Missing) => return Vec2::default(),
                Ok(ScriptOptionalMethodResult::Returned(ScriptValue::Vec2 { x, y })) => {
                    Vec2::new(x, y)
                }
                // Callback errors and non-vector results keep C++'s initial maxima.
                _ => Vec2::new(f32::MAX, f32::MAX),
            };
            self.natural_measure = Some(measured);
            measured
        };
        let width_limit = if width_mode == LayoutMeasureMode::Undefined {
            f32::MAX
        } else {
            width
        };
        let height_limit = if height_mode == LayoutMeasureMode::Undefined {
            f32::MAX
        } else {
            height
        };
        // std::min(a,b) selects a when b is NaN; f32::min differs for a=NaN.
        Vec2::new(
            if measured.x < width_limit {
                measured.x
            } else {
                width_limit
            },
            if measured.y < height_limit {
                measured.y
            } else {
                height_limit
            },
        )
    }

    pub fn control_size(
        &mut self,
        size: Vec2,
        _width_scale: LayoutScaleType,
        _height_scale: LayoutScaleType,
        _direction: LayoutDirection,
    ) {
        self.size = size;
        self.size_known = true;
        self.call_scripted_resize(size);
    }

    pub fn display_scale_changed(&mut self) {
        if self.size_known {
            self.call_scripted_resize(self.size);
        }
    }

    pub fn add_property(&mut self, property: CoreHandle) {
        self.base.base.add_property(property);
    }

    pub(crate) fn add_property_from_input(
        &mut self,
        property: CoreHandle,
        input: &mut crate::mechanical_port::source::assets::script_asset::ScriptInput,
    ) {
        self.base.base.add_property_from_input(property, input);
    }

    pub fn remove_property(&mut self, property: &CoreHandle) {
        self.base.base.remove_property(property);
    }

    pub fn clone_definition(&self) -> Self {
        let mut clone = self.base.clone_into();
        clone
            .base
            .base
            .scripted
            .file_asset_referencer_mut()
            .set_asset_unattached(self.base.base.scripted.script_asset());
        clone
    }

    pub fn script_protocol(&self) -> ScriptProtocol {
        ScriptProtocol::Layout
    }
}

/// One call of a layout script's resize, taken out of its layout.
struct ResizeCall {
    instance: crate::scripting::RuntimeScriptInstanceHandle,
    vm: crate::mechanical_port::source::lua::scripting_vm::RuntimeScriptingVmHandle,
    size: Vec2,
}

impl ResizeCall {
    /// Calls the script; true when it asked for an update.
    fn run(self) -> bool {
        let mut host = ScriptUpdateRequestHost::default();
        if let Err(error) =
            self.vm
                .call_layout_resize(&mut **self.instance.borrow_mut(), self.size, &mut host)
        {
            eprintln!("resize failed: {error}");
        }
        host.take_requested()
    }
}
