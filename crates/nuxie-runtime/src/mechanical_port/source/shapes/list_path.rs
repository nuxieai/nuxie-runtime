use std::{
    cell::RefCell,
    rc::{Rc, Weak},
};

use crate::mechanical_port::source::{
    component_dirt::ComponentDirt,
    core::CoreHandle,
    data_bind::data_bind_list_item_consumer::DataBindListItemConsumer,
    dirtyable::Dirtyable,
    generated::shapes::{
        cubic_detached_vertex_base::CubicDetachedVertexBase, list_path_base::ListPathBase,
        vertex_base::VertexBase,
    },
    math::{math_types::PI, vec2d::Vec2D},
    shapes::{cubic_detached_vertex::CubicDetachedVertex, vertex::VertexBehavior},
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
    fn number(value: Option<&CoreHandle>) -> f32 {
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
        match key {
            VertexBase::X_PROPERTY_KEY => {
                if vertex.base.set_x_value(value) {
                    VertexBehavior::x_changed(vertex);
                }
            }
            VertexBase::Y_PROPERTY_KEY => {
                if vertex.base.set_y_value(value) {
                    VertexBehavior::y_changed(vertex);
                }
            }
            CubicDetachedVertexBase::IN_ROTATION_PROPERTY_KEY => {
                if vertex.base.set_in_rotation_value(value) {
                    vertex.in_rotation_changed();
                }
            }
            CubicDetachedVertexBase::IN_DISTANCE_PROPERTY_KEY => {
                if vertex.base.set_in_distance_value(value) {
                    vertex.in_distance_changed();
                }
            }
            CubicDetachedVertexBase::OUT_ROTATION_PROPERTY_KEY => {
                if vertex.base.set_out_rotation_value(value) {
                    vertex.out_rotation_changed();
                }
            }
            CubicDetachedVertexBase::OUT_DISTANCE_PROPERTY_KEY => {
                if vertex.base.set_out_distance_value(value) {
                    vertex.out_distance_changed();
                }
            }
            _ => {}
        }
    }

    fn write_value(&mut self) {
        let Some(vertex) = self.vertex.upgrade() else {
            return;
        };
        let mut vertex = vertex.borrow_mut();
        match &self.property {
            VertexProperty::Single { key, multiplier } => {
                Self::set_value(
                    &mut vertex,
                    *key,
                    Self::number(self.x_value.as_ref()) * *multiplier,
                );
            }
            VertexProperty::Multi { keys, multiplier } => {
                let value = Self::number(self.x_value.as_ref()) * *multiplier;
                for key in keys {
                    Self::set_value(&mut vertex, *key, value);
                }
            }
            VertexProperty::Point {
                distance_key,
                rotation_key,
            } => {
                let point = Vec2D::new(
                    Self::number(self.x_value.as_ref()),
                    Self::number(self.y_value.as_ref()),
                );
                Self::set_value(&mut vertex, *distance_key, point.length());
                Self::set_value(&mut vertex, *rotation_key, point.y.atan2(point.x));
            }
        }
    }
}

impl Dirtyable for VertexPropertyListener {
    fn add_dirt(&mut self, _value: ComponentDirt, _recurse: bool) {
        self.write_value();
        self.path.with_mut(|path| {
            if let Some(path) = path.as_path_mut() {
                path.mark_path_dirty(true);
            }
        });
    }
}

impl ViewModelValueDependent for VertexPropertyListener {
    fn relink_data_bind(&mut self) {}
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
                    .with(|value| value.as_view_model_instance_number().is_some())
                    .unwrap_or(false)
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
        listener.borrow_mut().write_value();
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
        for (symbol, key, multiplier) in [
            (SymbolType::VertexX, VertexBase::X_PROPERTY_KEY, 1.0),
            (SymbolType::VertexY, VertexBase::Y_PROPERTY_KEY, 1.0),
            (
                SymbolType::InRotation,
                CubicDetachedVertexBase::IN_ROTATION_PROPERTY_KEY,
                PI / 180.0,
            ),
            (
                SymbolType::OutRotation,
                CubicDetachedVertexBase::OUT_ROTATION_PROPERTY_KEY,
                PI / 180.0,
            ),
            (
                SymbolType::InDistance,
                CubicDetachedVertexBase::IN_DISTANCE_PROPERTY_KEY,
                1.0,
            ),
            (
                SymbolType::OutDistance,
                CubicDetachedVertexBase::OUT_DISTANCE_PROPERTY_KEY,
                1.0,
            ),
        ] {
            let value = self.instance_value(symbol);
            self.add_listener(value, None, VertexProperty::Single { key, multiplier });
        }
        self.add_listener(
            self.instance_value(SymbolType::Distance),
            None,
            VertexProperty::Multi {
                keys: vec![
                    CubicDetachedVertexBase::IN_DISTANCE_PROPERTY_KEY,
                    CubicDetachedVertexBase::OUT_DISTANCE_PROPERTY_KEY,
                ],
                multiplier: 1.0,
            },
        );
        self.add_listener(
            self.instance_value(SymbolType::Rotation),
            None,
            VertexProperty::Multi {
                keys: vec![
                    CubicDetachedVertexBase::IN_ROTATION_PROPERTY_KEY,
                    CubicDetachedVertexBase::OUT_ROTATION_PROPERTY_KEY,
                ],
                multiplier: PI / 180.0,
            },
        );
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
