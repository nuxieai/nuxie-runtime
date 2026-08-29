use std::{
    cell::RefCell,
    rc::{Rc, Weak},
};

use crate::mechanical_port::source::{
    component_dirt::ComponentDirt,
    core::CoreHandle,
    data_bind::data_bind_list_item_consumer::DataBindListItemConsumer,
    dirtyable::Dirtyable,
    generated::{
        core_registry::CoreRegistry,
        shapes::{
            cubic_detached_vertex_base::CubicDetachedVertexBase, list_path_base::ListPathBase,
            vertex_base::VertexBase,
        },
    },
    math::{math_types::PI, vec2d::Vec2D},
    shapes::cubic_detached_vertex::CubicDetachedVertex,
    viewmodel::{
        symbol_type::SymbolType, viewmodel_instance_value::ValueDependentHandle,
        viewmodel_value_dependent::ViewModelValueDependent,
    },
};

enum VertexProperty {
    Single {
        key: u16,
        multiplier: f32,
    },
    Multi {
        keys: Vec<u16>,
        multiplier: f32,
    },
    Point {
        distance_key: u16,
        rotation_key: u16,
    },
}

struct VertexPropertyListener {
    vertex: Weak<RefCell<CubicDetachedVertex>>,
    path: CoreHandle,
    x_value: Option<CoreHandle>,
    y_value: Option<CoreHandle>,
    property: VertexProperty,
}

impl VertexPropertyListener {
    fn number(value: Option<&CoreHandle>, borrowed: Option<(&CoreHandle, f32)>) -> f32 {
        if let (Some(value), Some((source, number))) = (value, borrowed) {
            if value == source {
                return number;
            }
        }
        value
            .and_then(|value| {
                value.with(|value| {
                    value
                        .as_view_model_instance_number()
                        .map(|value| value.base.property_value())
                })
            })
            .flatten()
            .unwrap_or_default()
    }

    fn set_value(vertex: &mut CubicDetachedVertex, key: u16, value: f32) {
        // Pinned VertexPropertyListener writes through CoreRegistry so the
        // most-derived callbacks run. Calling Vertex::x/yChanged directly skips
        // CubicVertex's cached control-point invalidation after a list remap.
        CoreRegistry::set_double(vertex, key.into(), value);
    }

    fn write_value(&mut self, borrowed: Option<(&CoreHandle, f32)>) {
        let Some(vertex) = self.vertex.upgrade() else {
            return;
        };
        let mut vertex = vertex.borrow_mut();
        match &self.property {
            VertexProperty::Single { key, multiplier } => {
                Self::set_value(
                    &mut vertex,
                    *key,
                    Self::number(self.x_value.as_ref(), borrowed) * *multiplier,
                );
            }
            VertexProperty::Multi { keys, multiplier } => {
                let value = Self::number(self.x_value.as_ref(), borrowed) * *multiplier;
                for key in keys {
                    Self::set_value(&mut vertex, *key, value);
                }
            }
            VertexProperty::Point {
                distance_key,
                rotation_key,
            } => {
                let point = Vec2D::new(
                    Self::number(self.x_value.as_ref(), borrowed),
                    Self::number(self.y_value.as_ref(), borrowed),
                );
                Self::set_value(&mut vertex, *distance_key, point.length());
                Self::set_value(&mut vertex, *rotation_key, point.y.atan2(point.x));
            }
        }
    }
    fn write_value_and_dirty_path(&mut self, borrowed: Option<(&CoreHandle, f32)>) {
        self.write_value(borrowed);
        self.path.with_mut(|path| {
            if let Some(path) = path.as_path_mut() {
                path.mark_path_dirty(true);
            }
        });
    }
}

impl Dirtyable for VertexPropertyListener {
    fn add_dirt(&mut self, _value: ComponentDirt, _recurse: bool) {
        self.write_value_and_dirty_path(None);
    }
}

impl ViewModelValueDependent for VertexPropertyListener {
    fn relink_data_bind(&mut self) {}

    fn add_dirt_from_number(
        &mut self,
        _value: ComponentDirt,
        _recurse: bool,
        source: &CoreHandle,
        number_value: f32,
    ) {
        self.write_value_and_dirty_path(Some((source, number_value)));
    }
}

struct VertexListener {
    vertex: Rc<RefCell<CubicDetachedVertex>>,
    instance: CoreHandle,
    path: CoreHandle,
    properties: Vec<Rc<RefCell<dyn ViewModelValueDependent>>>,
}

impl VertexListener {
    fn new(
        vertex: Rc<RefCell<CubicDetachedVertex>>,
        instance: CoreHandle,
        path: CoreHandle,
    ) -> Self {
        let mut listener = Self {
            vertex,
            instance,
            path,
            properties: Vec::new(),
        };
        listener.create_properties();
        listener
    }

    fn remap(&mut self, instance: CoreHandle) {
        if self.instance != instance {
            self.properties.clear();
            self.instance = instance;
            self.create_properties();
        }
    }

    fn instance_value(&self, symbol: SymbolType) -> Option<CoreHandle> {
        self.instance
            .with(|instance| {
                instance
                    .as_view_model_instance()
                    .and_then(|instance| instance.property_value_for_symbol(symbol))
            })
            .flatten()
            .filter(|value| {
                value
                    .is_type_of(crate::mechanical_port::source::generated::viewmodel::viewmodel_instance_number_base::ViewModelInstanceNumberBase::TYPE_KEY)
            })
    }

    fn add_listener(
        &mut self,
        x_value: Option<CoreHandle>,
        y_value: Option<CoreHandle>,
        property: VertexProperty,
    ) {
        if x_value.is_none() && y_value.is_none() {
            return;
        }
        let listener = Rc::new(RefCell::new(VertexPropertyListener {
            vertex: Rc::downgrade(&self.vertex),
            path: self.path.clone(),
            x_value: x_value.clone(),
            y_value: y_value.clone(),
            property,
        }));
        listener.borrow_mut().write_value(None);
        let dependent: Rc<RefCell<dyn ViewModelValueDependent>> = listener;
        let dependent_handle = ValueDependentHandle::runtime(&dependent);
        for value in [x_value, y_value].into_iter().flatten() {
            value.with_mut(|value| {
                if let Some(value) = value.as_view_model_instance_value_mut() {
                    value.add_dependent(dependent_handle.clone());
                }
            });
        }
        self.properties.push(dependent);
    }

    fn create_properties(&mut self) {
        self.properties.clear();
        // Match VertexListener::createProperties: the shared rotation/distance
        // are written before the individual in/out overrides, not after them.
        for (symbol, property) in [
            (
                SymbolType::VertexX,
                VertexProperty::Single {
                    key: VertexBase::X_PROPERTY_KEY,
                    multiplier: 1.0,
                },
            ),
            (
                SymbolType::VertexY,
                VertexProperty::Single {
                    key: VertexBase::Y_PROPERTY_KEY,
                    multiplier: 1.0,
                },
            ),
            (
                SymbolType::Rotation,
                VertexProperty::Multi {
                    keys: vec![
                        CubicDetachedVertexBase::IN_ROTATION_PROPERTY_KEY,
                        CubicDetachedVertexBase::OUT_ROTATION_PROPERTY_KEY,
                    ],
                    multiplier: PI / 180.0,
                },
            ),
            (
                SymbolType::InRotation,
                VertexProperty::Single {
                    key: CubicDetachedVertexBase::IN_ROTATION_PROPERTY_KEY,
                    multiplier: PI / 180.0,
                },
            ),
            (
                SymbolType::OutRotation,
                VertexProperty::Single {
                    key: CubicDetachedVertexBase::OUT_ROTATION_PROPERTY_KEY,
                    multiplier: PI / 180.0,
                },
            ),
            (
                SymbolType::Distance,
                VertexProperty::Multi {
                    keys: vec![
                        CubicDetachedVertexBase::IN_DISTANCE_PROPERTY_KEY,
                        CubicDetachedVertexBase::OUT_DISTANCE_PROPERTY_KEY,
                    ],
                    multiplier: 1.0,
                },
            ),
            (
                SymbolType::InDistance,
                VertexProperty::Single {
                    key: CubicDetachedVertexBase::IN_DISTANCE_PROPERTY_KEY,
                    multiplier: 1.0,
                },
            ),
            (
                SymbolType::OutDistance,
                VertexProperty::Single {
                    key: CubicDetachedVertexBase::OUT_DISTANCE_PROPERTY_KEY,
                    multiplier: 1.0,
                },
            ),
        ] {
            let value = self.instance_value(symbol);
            self.add_listener(value, None, property);
        }
        for (x_symbol, y_symbol, distance_key, rotation_key) in [
            (
                SymbolType::CubicVertexInPointX,
                SymbolType::CubicVertexInPointY,
                CubicDetachedVertexBase::IN_DISTANCE_PROPERTY_KEY,
                CubicDetachedVertexBase::IN_ROTATION_PROPERTY_KEY,
            ),
            (
                SymbolType::CubicVertexOutPointX,
                SymbolType::CubicVertexOutPointY,
                CubicDetachedVertexBase::OUT_DISTANCE_PROPERTY_KEY,
                CubicDetachedVertexBase::OUT_ROTATION_PROPERTY_KEY,
            ),
        ] {
            self.add_listener(
                self.instance_value(x_symbol),
                self.instance_value(y_symbol),
                VertexProperty::Point {
                    distance_key,
                    rotation_key,
                },
            );
        }
    }
}

impl std::ops::Deref for ListPath {
    type Target = ListPathBase;
    fn deref(&self) -> &Self::Target {
        &self.base
    }
}

impl std::ops::DerefMut for ListPath {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.base
    }
}

impl ListPath {
    pub const TYPE_KEY: u16 = ListPathBase::TYPE_KEY;
}

#[derive(Default)]
pub struct ListPath {
    pub base: ListPathBase,
    vertex_listeners: Vec<VertexListener>,
}

impl ListPath {
    pub fn update_list(&mut self, list: &[CoreHandle]) {
        let current_size = self.vertex_listeners.len();
        let Some(path) = self.base.handle() else {
            return;
        };
        let mut index = 0;
        for item in list {
            let instance = item
                .with(|item| {
                    item.as_view_model_instance_list_item()
                        .and_then(|item| item.view_model_instance())
                })
                .flatten();
            let Some(instance) = instance else {
                continue;
            };
            if index >= current_size {
                let vertex = Rc::new(RefCell::new(CubicDetachedVertex::default()));
                self.vertex_listeners.push(VertexListener::new(
                    vertex.clone(),
                    instance,
                    path.clone(),
                ));
                self.base.add_runtime_cubic_vertex(vertex);
            } else {
                self.vertex_listeners[index].remap(instance);
            }
            index += 1;
        }
        while self.vertex_listeners.len() > index {
            self.vertex_listeners.pop();
            self.base.pop_vertex();
        }
        self.base.mark_path_dirty(true);
    }
}

impl DataBindListItemConsumer for ListPath {
    fn update_list(&mut self, list: &[CoreHandle]) {
        Self::update_list(self, list);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mechanical_port::source::{
        core::CoreArena,
        shapes::cubic_vertex::CubicVertexBehavior,
        viewmodel::{
            viewmodel_instance::ViewModelInstance,
            viewmodel_instance_number::ViewModelInstanceNumber,
        },
    };

    fn number(
        arena: &CoreArena,
        instance: &CoreHandle,
        symbol: SymbolType,
        value: f32,
    ) -> CoreHandle {
        let number = arena.insert(ViewModelInstanceNumber::default());
        number
            .with_downcast_mut::<ViewModelInstanceNumber, _>(|number| number.set_value(value))
            .unwrap();
        instance
            .with_downcast_mut::<ViewModelInstance, _>(|instance| {
                instance.set_property_symbol(symbol, number.clone());
            })
            .unwrap();
        number
    }

    #[test]
    fn individual_controls_override_shared_controls_in_pinned_creation_order() {
        let arena = CoreArena::default();
        let instance = arena.insert(ViewModelInstance::default());
        for (symbol, value) in [
            (SymbolType::Rotation, 10.0),
            (SymbolType::InRotation, 20.0),
            (SymbolType::OutRotation, 30.0),
            (SymbolType::Distance, 4.0),
            (SymbolType::InDistance, 5.0),
            (SymbolType::OutDistance, 6.0),
        ] {
            number(&arena, &instance, symbol, value);
        }
        let vertex = Rc::new(RefCell::new(CubicDetachedVertex::default()));
        let _listener =
            VertexListener::new(vertex.clone(), instance, arena.insert(ListPath::default()));
        let vertex = vertex.borrow();
        assert_eq!(vertex.base.in_rotation(), 20.0 * (PI / 180.0));
        assert_eq!(vertex.base.out_rotation(), 30.0 * (PI / 180.0));
        assert_eq!(vertex.base.in_distance(), 5.0);
        assert_eq!(vertex.base.out_distance(), 6.0);
    }

    #[test]
    fn live_point_listener_reads_the_borrowed_number_and_the_other_coordinate() {
        let arena = CoreArena::default();
        let instance = arena.insert(ViewModelInstance::default());
        let x = number(&arena, &instance, SymbolType::CubicVertexInPointX, 0.0);
        let y = number(&arena, &instance, SymbolType::CubicVertexInPointY, 4.0);
        let vertex = Rc::new(RefCell::new(CubicDetachedVertex::default()));
        let _listener =
            VertexListener::new(vertex.clone(), instance, arena.insert(ListPath::default()));
        x.with_downcast_mut::<ViewModelInstanceNumber, _>(|number| number.set_value(3.0))
            .unwrap();
        assert_eq!(vertex.borrow().base.in_distance(), 5.0);
        assert_eq!(vertex.borrow().base.in_rotation(), 4.0_f32.atan2(3.0));
        y.with_downcast_mut::<ViewModelInstanceNumber, _>(|number| number.set_value(0.0))
            .unwrap();
        assert_eq!(vertex.borrow().base.in_distance(), 3.0);
        assert_eq!(vertex.borrow().base.in_rotation(), 0.0);
    }

    #[test]
    fn remapping_a_vertex_invalidates_its_previously_rendered_control_points() {
        let arena = CoreArena::default();
        let first = arena.insert(ViewModelInstance::default());
        number(&arena, &first, SymbolType::VertexX, 0.0);
        number(&arena, &first, SymbolType::VertexY, 100.0);
        let next = arena.insert(ViewModelInstance::default());
        number(&arena, &next, SymbolType::VertexX, 100.0);
        number(&arena, &next, SymbolType::VertexY, 100.0);
        let vertex = Rc::new(RefCell::new(CubicDetachedVertex::default()));
        let mut listener =
            VertexListener::new(vertex.clone(), first, arena.insert(ListPath::default()));
        assert_eq!(vertex.borrow_mut().render_in(), Vec2D::new(0.0, 100.0));
        assert_eq!(vertex.borrow_mut().render_out(), Vec2D::new(0.0, 100.0));
        listener.remap(next);
        assert_eq!(vertex.borrow_mut().render_in(), Vec2D::new(100.0, 100.0));
        assert_eq!(vertex.borrow_mut().render_out(), Vec2D::new(100.0, 100.0));
    }
}
