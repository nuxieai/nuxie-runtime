use crate::mechanical_port::source::{
    bones::skin::Skin,
    container_component::ContainerComponent,
    core::{binary_reader::BinaryReader, field_types::core_double_type::CoreDoubleType},
};

pub trait SkinBaseCallbacks:
    crate::mechanical_port::source::generated::component_base::ComponentBaseCallbacks
{
    fn xx_changed(&mut self) {}
    fn yx_changed(&mut self) {}
    fn xy_changed(&mut self) {}
    fn yy_changed(&mut self) {}
    fn tx_changed(&mut self) {}
    fn ty_changed(&mut self) {}
    fn notify_property_changed(&mut self, property_key: u16);
}

pub struct SkinBase {
    pub base: ContainerComponent,
    xx: f32,
    yx: f32,
    xy: f32,
    yy: f32,
    tx: f32,
    ty: f32,
}

impl Default for SkinBase {
    fn default() -> Self {
        Self {
            base: ContainerComponent::default(),
            xx: 1.0,
            yx: 0.0,
            xy: 0.0,
            yy: 1.0,
            tx: 0.0,
            ty: 0.0,
        }
    }
}

impl SkinBase {
    pub const TYPE_KEY: u16 = 43;
    pub const XX_PROPERTY_KEY: u16 = 104;
    pub const YX_PROPERTY_KEY: u16 = 105;
    pub const XY_PROPERTY_KEY: u16 = 106;
    pub const YY_PROPERTY_KEY: u16 = 107;
    pub const TX_PROPERTY_KEY: u16 = 108;
    pub const TY_PROPERTY_KEY: u16 = 109;
    pub fn is_type_of(type_key: u16) -> bool {
        matches!(type_key, Self::TYPE_KEY | 11 | 10)
    }
    pub fn core_type(&self) -> u16 {
        Self::TYPE_KEY
    }
    pub fn xx(&self) -> f32 {
        self.xx
    }
    pub fn yx(&self) -> f32 {
        self.yx
    }
    pub fn xy(&self) -> f32 {
        self.xy
    }
    pub fn yy(&self) -> f32 {
        self.yy
    }
    pub fn tx(&self) -> f32 {
        self.tx
    }
    pub fn ty(&self) -> f32 {
        self.ty
    }

    pub fn set_xx<C: SkinBaseCallbacks>(&mut self, value: f32, c: &mut C) {
        if !self.set_xx_value(value) {
            return;
        }
        c.xx_changed();
        c.notify_property_changed(Self::XX_PROPERTY_KEY);
    }

    pub(crate) fn set_xx_value(&mut self, value: f32) -> bool {
        if self.xx == value {
            return false;
        }
        self.xx = value;
        true
    }
    pub fn set_yx<C: SkinBaseCallbacks>(&mut self, value: f32, c: &mut C) {
        if !self.set_yx_value(value) {
            return;
        }
        c.yx_changed();
        c.notify_property_changed(Self::YX_PROPERTY_KEY);
    }

    pub(crate) fn set_yx_value(&mut self, value: f32) -> bool {
        if self.yx == value {
            return false;
        }
        self.yx = value;
        true
    }
    pub fn set_xy<C: SkinBaseCallbacks>(&mut self, value: f32, c: &mut C) {
        if !self.set_xy_value(value) {
            return;
        }
        c.xy_changed();
        c.notify_property_changed(Self::XY_PROPERTY_KEY);
    }

    pub(crate) fn set_xy_value(&mut self, value: f32) -> bool {
        if self.xy == value {
            return false;
        }
        self.xy = value;
        true
    }
    pub fn set_yy<C: SkinBaseCallbacks>(&mut self, value: f32, c: &mut C) {
        if !self.set_yy_value(value) {
            return;
        }
        c.yy_changed();
        c.notify_property_changed(Self::YY_PROPERTY_KEY);
    }

    pub(crate) fn set_yy_value(&mut self, value: f32) -> bool {
        if self.yy == value {
            return false;
        }
        self.yy = value;
        true
    }
    pub fn set_tx<C: SkinBaseCallbacks>(&mut self, value: f32, c: &mut C) {
        if !self.set_tx_value(value) {
            return;
        }
        c.tx_changed();
        c.notify_property_changed(Self::TX_PROPERTY_KEY);
    }

    pub(crate) fn set_tx_value(&mut self, value: f32) -> bool {
        if self.tx == value {
            return false;
        }
        self.tx = value;
        true
    }
    pub fn set_ty<C: SkinBaseCallbacks>(&mut self, value: f32, c: &mut C) {
        if !self.set_ty_value(value) {
            return;
        }
        c.ty_changed();
        c.notify_property_changed(Self::TY_PROPERTY_KEY);
    }

    pub(crate) fn set_ty_value(&mut self, value: f32) -> bool {
        if self.ty == value {
            return false;
        }
        self.ty = value;
        true
    }

    pub fn clone_into<C: SkinBaseCallbacks>(&self, c: &mut C) -> Skin {
        let mut cloned = Skin::default();
        cloned.base.copy(self, c);
        cloned
    }
    pub fn copy<C: SkinBaseCallbacks>(&mut self, object: &Self, c: &mut C) {
        self.xx = object.xx;
        self.yx = object.yx;
        self.xy = object.xy;
        self.yy = object.yy;
        self.tx = object.tx;
        self.ty = object.ty;
        self.base.copy(&object.base, c);
    }
    pub fn deserialize<C: SkinBaseCallbacks>(
        &mut self,
        key: u16,
        reader: &mut BinaryReader<'_>,
        c: &mut C,
    ) -> bool {
        match key {
            Self::XX_PROPERTY_KEY => {
                self.xx = CoreDoubleType::deserialize(reader);
                true
            }
            Self::YX_PROPERTY_KEY => {
                self.yx = CoreDoubleType::deserialize(reader);
                true
            }
            Self::XY_PROPERTY_KEY => {
                self.xy = CoreDoubleType::deserialize(reader);
                true
            }
            Self::YY_PROPERTY_KEY => {
                self.yy = CoreDoubleType::deserialize(reader);
                true
            }
            Self::TX_PROPERTY_KEY => {
                self.tx = CoreDoubleType::deserialize(reader);
                true
            }
            Self::TY_PROPERTY_KEY => {
                self.ty = CoreDoubleType::deserialize(reader);
                true
            }
            _ => self.base.deserialize(key, reader, c),
        }
    }
}

impl std::ops::Deref for SkinBase {
    type Target = ContainerComponent;

    fn deref(&self) -> &Self::Target {
        &self.base
    }
}

impl std::ops::DerefMut for SkinBase {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.base
    }
}
