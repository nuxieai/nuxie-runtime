use crate::mechanical_port::source::{
    core::CoreHandle,
    generated::scripted::scripted_layout_base::ScriptedLayoutBase,
    scripted::scripted_object::{ScriptProtocol, ScriptUpdateRequestHost},
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
}
impl ScriptedLayout {
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
        if !self.base.base.scripted.resizes() {
            return;
        }
        let Some(instance) = self.base.base.scripted.runtime_instance() else {
            return;
        };
        let mut host = ScriptUpdateRequestHost::default();
        // Upstream passes one native Vec2D, not two scalar arguments.
        if let Err(error) = instance.borrow_mut().call_optional_method(
            ScriptMethod::Resize,
            &[ScriptValue::Vec2 {
                x: size.x,
                y: size.y,
            }],
            &mut host,
        ) {
            eprintln!("resize failed: {error}");
        }
        if host.take_requested() {
            self.base.base.mark_needs_update();
        }
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
            Ok(ScriptOptionalMethodResult::Returned(ScriptValue::Vec2 { x, y })) => Vec2::new(x, y),
            // Callback errors and non-vector results keep C++'s initial maxima.
            _ => Vec2::new(f32::MAX, f32::MAX),
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
        self.call_scripted_resize(size);
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
