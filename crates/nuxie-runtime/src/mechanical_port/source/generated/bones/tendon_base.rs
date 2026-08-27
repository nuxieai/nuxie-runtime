use crate::mechanical_port::source::{
    bones::tendon::Tendon,
    component::Component,
    core::{
        binary_reader::BinaryReader,
        field_types::{core_double_type::CoreDoubleType, core_uint_type::CoreUintType},
    },
};

pub trait TendonBaseCallbacks {
    fn bone_id_changed(&mut self) {}
    fn xx_changed(&mut self) {}
    fn yx_changed(&mut self) {}
    fn xy_changed(&mut self) {}
    fn yy_changed(&mut self) {}
    fn tx_changed(&mut self) {}
    fn ty_changed(&mut self) {}
    fn notify_property_changed(&mut self, property_key: u16);
}

pub struct TendonBase {
    pub base: Component,
    bone_id: u32,
    xx: f32,
    yx: f32,
    xy: f32,
    yy: f32,
    tx: f32,
    ty: f32,
}

impl Default for TendonBase {
    fn default() -> Self {
        Self {
            base: Component::default(),
            bone_id: u32::MAX,
            xx: 1.0,
            yx: 0.0,
            xy: 0.0,
            yy: 1.0,
            tx: 0.0,
            ty: 0.0,
        }
    }
}

impl TendonBase {
    pub const TYPE_KEY: u16 = 44;
    pub const BONE_ID_PROPERTY_KEY: u16 = 95;
    pub const XX_PROPERTY_KEY: u16 = 96;
    pub const YX_PROPERTY_KEY: u16 = 97;
    pub const XY_PROPERTY_KEY: u16 = 98;
    pub const YY_PROPERTY_KEY: u16 = 99;
    pub const TX_PROPERTY_KEY: u16 = 100;
    pub const TY_PROPERTY_KEY: u16 = 101;
    pub fn is_type_of(type_key: u16) -> bool {
        matches!(type_key, Self::TYPE_KEY | 1)
    }
    pub fn core_type(&self) -> u16 {
        Self::TYPE_KEY
    }
    pub fn bone_id(&self) -> u32 {
        self.bone_id
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

    pub fn set_bone_id<C: TendonBaseCallbacks>(&mut self, value: u32, c: &mut C) {
        if self.bone_id == value {
            return;
        }
        self.bone_id = value;
        c.bone_id_changed();
        c.notify_property_changed(Self::BONE_ID_PROPERTY_KEY);
    }
    pub fn set_xx<C: TendonBaseCallbacks>(&mut self, value: f32, c: &mut C) {
        if self.xx == value {
            return;
        }
        self.xx = value;
        c.xx_changed();
        c.notify_property_changed(Self::XX_PROPERTY_KEY);
    }
    pub fn set_yx<C: TendonBaseCallbacks>(&mut self, value: f32, c: &mut C) {
        if self.yx == value {
            return;
        }
        self.yx = value;
        c.yx_changed();
        c.notify_property_changed(Self::YX_PROPERTY_KEY);
    }
    pub fn set_xy<C: TendonBaseCallbacks>(&mut self, value: f32, c: &mut C) {
        if self.xy == value {
            return;
        }
        self.xy = value;
        c.xy_changed();
        c.notify_property_changed(Self::XY_PROPERTY_KEY);
    }
    pub fn set_yy<C: TendonBaseCallbacks>(&mut self, value: f32, c: &mut C) {
        if self.yy == value {
            return;
        }
        self.yy = value;
        c.yy_changed();
        c.notify_property_changed(Self::YY_PROPERTY_KEY);
    }
    pub fn set_tx<C: TendonBaseCallbacks>(&mut self, value: f32, c: &mut C) {
        if self.tx == value {
            return;
        }
        self.tx = value;
        c.tx_changed();
        c.notify_property_changed(Self::TX_PROPERTY_KEY);
    }
    pub fn set_ty<C: TendonBaseCallbacks>(&mut self, value: f32, c: &mut C) {
        if self.ty == value {
            return;
        }
        self.ty = value;
        c.ty_changed();
        c.notify_property_changed(Self::TY_PROPERTY_KEY);
    }

    pub fn clone_into<C: TendonBaseCallbacks>(&self, c: &mut C) -> Tendon {
        let mut cloned = Tendon::default();
        cloned.base.copy(self, c);
        cloned
    }
    pub fn copy<C: TendonBaseCallbacks>(&mut self, object: &Self, c: &mut C) {
        self.bone_id = object.bone_id;
        self.xx = object.xx;
        self.yx = object.yx;
        self.xy = object.xy;
        self.yy = object.yy;
        self.tx = object.tx;
        self.ty = object.ty;
        self.base.copy(&object.base, c);
    }
    pub fn deserialize<C: TendonBaseCallbacks>(
        &mut self,
        key: u16,
        reader: &mut BinaryReader<'_>,
        c: &mut C,
    ) -> bool {
        match key {
            Self::BONE_ID_PROPERTY_KEY => {
                self.bone_id = CoreUintType::deserialize(reader);
                true
            }
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
