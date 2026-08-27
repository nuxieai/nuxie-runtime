use crate::mechanical_port::source::{
    core::{Core, CoreRegistry},
    math::{math_types::PI, vec2d::Vec2D},
    shapes::{cubic_detached_vertex::CubicDetachedVertex, path::Path, vertex::Vertex},
    viewmodel::{
        core_object_listener::CoreObjectListener,
        property_symbol_dependent::{
            PropertySymbolDependent, PropertySymbolDependentMulti, PropertySymbolDependentSingle,
        },
        symbol_type::SymbolType,
        viewmodel_instance::ViewModelInstance,
        viewmodel_instance_list_item::ViewModelInstanceListItem,
        viewmodel_instance_value::ViewModelInstanceValue,
    },
};

pub struct VertexPropertyListenerSingle {
    pub base: PropertySymbolDependentSingle,
    multiplier: f32,
}
impl VertexPropertyListenerSingle {
    pub fn new(
        vertex: *mut Core,
        listener: *mut VertexListener,
        value: *mut ViewModelInstanceValue,
        key: u16,
        multiplier: f32,
    ) -> Self {
        Self {
            base: PropertySymbolDependentSingle::new(vertex, listener, value, key),
            multiplier,
        }
    }
    pub fn write_value(&mut self) {
        let value = self.base.instance_value().as_number().property_value() * self.multiplier;
        CoreRegistry::set_double(self.base.core_object(), self.base.property_key(), value);
    }
}

pub struct VertexPropertyListenerMulti {
    pub base: PropertySymbolDependentMulti,
    multiplier: f32,
}
impl VertexPropertyListenerMulti {
    pub fn new(
        vertex: *mut Core,
        listener: *mut VertexListener,
        value: *mut ViewModelInstanceValue,
        keys: Vec<u16>,
        multiplier: f32,
    ) -> Self {
        Self {
            base: PropertySymbolDependentMulti::new(vertex, listener, value, keys),
            multiplier,
        }
    }
    pub fn write_value(&mut self) {
        let value = self.base.instance_value().as_number().property_value() * self.multiplier;
        for key in self.base.property_keys() {
            CoreRegistry::set_double(self.base.core_object(), *key, value);
        }
    }
}

pub struct VertexPropertyListenerPoint {
    pub base: PropertySymbolDependent,
    y_value: Option<*mut ViewModelInstanceValue>,
    distance_key: u16,
    rotation_key: u16,
}
impl VertexPropertyListenerPoint {
    pub fn new(
        vertex: *mut Core,
        listener: *mut VertexListener,
        x_value: Option<*mut ViewModelInstanceValue>,
        y_value: Option<*mut ViewModelInstanceValue>,
        distance_key: u16,
        rotation_key: u16,
    ) -> Self {
        let mut value = Self {
            base: PropertySymbolDependent::new(vertex, listener, x_value),
            y_value,
            distance_key,
            rotation_key,
        };
        if let Some(y) = value.y_value {
            unsafe { &mut *y }.add_dependent(&mut value);
        }
        value
    }
    pub fn write_value(&mut self) {
        let x = self
            .base
            .instance_value()
            .map(|v| v.as_number().property_value())
            .unwrap_or(0.0);
        let y = self
            .y_value
            .map(|v| unsafe { &*v }.as_number().property_value())
            .unwrap_or(0.0);
        let point = Vec2D::new(x, y);
        CoreRegistry::set_double(self.base.core_object(), self.distance_key, point.length());
        CoreRegistry::set_double(
            self.base.core_object(),
            self.rotation_key,
            point.y.atan2(point.x),
        );
    }
}
impl Drop for VertexPropertyListenerPoint {
    fn drop(&mut self) {
        if let Some(y) = self.y_value {
            unsafe { &mut *y }.remove_dependent(self);
        }
    }
}

pub struct VertexListener {
    pub base: CoreObjectListener,
    path: *mut Path,
}
impl VertexListener {
    pub fn new(vertex: *mut Vertex, instance: ViewModelInstance, path: *mut Path) -> Self {
        let mut value = Self {
            base: CoreObjectListener::new(vertex.cast(), instance),
            path,
        };
        value.create_properties();
        value
    }
    pub fn vertex(&mut self) -> &mut Vertex {
        self.base.core_mut().as_vertex_mut().unwrap()
    }
    pub fn mark_dirty(&mut self) {
        unsafe { &mut *self.path }.mark_path_dirty(true);
    }
    fn create_properties(&mut self) {
        self.base.create_properties();
        for symbol in [
            SymbolType::VertexX,
            SymbolType::VertexY,
            SymbolType::Rotation,
            SymbolType::InRotation,
            SymbolType::OutRotation,
            SymbolType::Distance,
            SymbolType::InDistance,
            SymbolType::OutDistance,
        ] {
            self.create_property_listener(symbol);
        }
        self.create_point_property_listener(
            SymbolType::CubicVertexInPointX,
            SymbolType::CubicVertexInPointY,
            CubicDetachedVertexBase::IN_DISTANCE_PROPERTY_KEY,
            CubicDetachedVertexBase::IN_ROTATION_PROPERTY_KEY,
        );
        self.create_point_property_listener(
            SymbolType::CubicVertexOutPointX,
            SymbolType::CubicVertexOutPointY,
            CubicDetachedVertexBase::OUT_DISTANCE_PROPERTY_KEY,
            CubicDetachedVertexBase::OUT_ROTATION_PROPERTY_KEY,
        );
    }
    fn create_single_property_listener(
        &mut self,
        symbol: SymbolType,
    ) -> Option<Box<dyn PropertyListener>> {
        let (key, multiplier) = match symbol {
            SymbolType::VertexX => (VertexBase::X_PROPERTY_KEY, 1.0),
            SymbolType::VertexY => (VertexBase::Y_PROPERTY_KEY, 1.0),
            SymbolType::InRotation => (
                CubicDetachedVertexBase::IN_ROTATION_PROPERTY_KEY,
                PI / 180.0,
            ),
            SymbolType::OutRotation => (
                CubicDetachedVertexBase::OUT_ROTATION_PROPERTY_KEY,
                PI / 180.0,
            ),
            SymbolType::InDistance => (CubicDetachedVertexBase::IN_DISTANCE_PROPERTY_KEY, 1.0),
            SymbolType::OutDistance => (CubicDetachedVertexBase::OUT_DISTANCE_PROPERTY_KEY, 1.0),
            _ => (0, 1.0),
        };
        let value = self
            .base
            .instance()
            .property_value(symbol)?
            .as_number_mut()?;
        Some(Box::new(VertexPropertyListenerSingle::new(
            self.base.core_pointer(),
            self,
            value,
            key,
            multiplier,
        )))
    }
    fn create_multi_property_listener(
        &mut self,
        symbol: SymbolType,
        keys: Vec<u16>,
        multiplier: f32,
    ) -> Option<Box<dyn PropertyListener>> {
        let value = self
            .base
            .instance()
            .property_value(symbol)?
            .as_number_mut()?;
        Some(Box::new(VertexPropertyListenerMulti::new(
            self.base.core_pointer(),
            self,
            value,
            keys,
            multiplier,
        )))
    }
    fn create_point_property_listener(
        &mut self,
        x_symbol: SymbolType,
        y_symbol: SymbolType,
        distance_key: u16,
        rotation_key: u16,
    ) {
        let x = self
            .base
            .instance()
            .property_value(x_symbol)
            .and_then(ViewModelInstanceValue::as_number_mut);
        let y = self
            .base
            .instance()
            .property_value(y_symbol)
            .and_then(ViewModelInstanceValue::as_number_mut);
        if x.is_some() || y.is_some() {
            let mut listener = VertexPropertyListenerPoint::new(
                self.base.core_pointer(),
                self,
                x,
                y,
                distance_key,
                rotation_key,
            );
            listener.write_value();
            self.base.properties_mut().push(Box::new(listener));
        }
    }
    fn create_property_listener(&mut self, symbol: SymbolType) {
        let listener = match symbol {
            SymbolType::VertexX
            | SymbolType::VertexY
            | SymbolType::InRotation
            | SymbolType::OutRotation
            | SymbolType::InDistance
            | SymbolType::OutDistance => self.create_single_property_listener(symbol),
            SymbolType::Distance => self.create_multi_property_listener(
                symbol,
                vec![
                    CubicDetachedVertexBase::IN_DISTANCE_PROPERTY_KEY,
                    CubicDetachedVertexBase::OUT_DISTANCE_PROPERTY_KEY,
                ],
                1.0,
            ),
            SymbolType::Rotation => self.create_multi_property_listener(
                symbol,
                vec![
                    CubicDetachedVertexBase::IN_ROTATION_PROPERTY_KEY,
                    CubicDetachedVertexBase::OUT_ROTATION_PROPERTY_KEY,
                ],
                PI / 180.0,
            ),
            _ => None,
        };
        if let Some(mut listener) = listener {
            listener.write_value();
            self.base.properties_mut().push(listener);
        }
    }
}

pub struct ListPath {
    pub base: ListPathBase,
    vertex_listeners: Vec<VertexListener>,
}
impl ListPath {
    pub fn update_list(&mut self, list: &[ViewModelInstanceListItem]) {
        let current_size = self.vertex_listeners.len();
        let mut index = 0;
        for item in list {
            if let Some(instance) = item.viewmodel_instance() {
                if index >= current_size {
                    let mut vertex = Box::new(CubicDetachedVertex::default());
                    let listener = VertexListener::new(
                        vertex.as_mut().as_vertex_mut(),
                        instance.clone(),
                        self.base.as_path_mut(),
                    );
                    self.vertex_listeners.push(listener);
                    self.base.add_vertex(vertex.into_vertex());
                } else {
                    self.vertex_listeners[index].base.remap(instance.clone());
                }
                index += 1;
            }
        }
        while self.vertex_listeners.len() > index {
            self.vertex_listeners.pop();
            self.base.pop_vertex();
        }
        self.base.mark_path_dirty(true);
    }
}
