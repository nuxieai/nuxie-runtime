use crate::mechanical_port::source::{
    bones::{bone::Bone, root_bone::RootBone},
    core::{binary_reader::BinaryReader, field_types::core_double_type::CoreDoubleType},
    generated::bones::bone_base::BoneBaseCallbacks,
};

pub trait RootBoneBaseCallbacks: BoneBaseCallbacks {
    fn x_changed(&mut self) {}
    fn y_changed(&mut self) {}
}

pub struct RootBoneBase {
    pub base: Bone,
    x: f32,
    y: f32,
}

impl Default for RootBoneBase {
    fn default() -> Self {
        Self {
            base: Bone::default(),
            x: 0.0,
            y: 0.0,
        }
    }
}

impl RootBoneBase {
    pub const TYPE_KEY: u16 = 41;
    pub const X_PROPERTY_KEY: u16 = 90;
    pub const Y_PROPERTY_KEY: u16 = 91;

    pub fn is_type_of(type_key: u16) -> bool {
        matches!(type_key, Self::TYPE_KEY | 40 | 39 | 38 | 91 | 11 | 10)
    }

    pub fn core_type(&self) -> u16 {
        Self::TYPE_KEY
    }

    pub fn x(&self) -> f32 {
        self.x
    }

    pub fn set_x<C: RootBoneBaseCallbacks>(&mut self, value: f32, callbacks: &mut C) {
        if self.x == value {
            return;
        }
        self.x = value;
        callbacks.x_changed();
        callbacks.notify_property_changed(Self::X_PROPERTY_KEY);
    }

    pub fn y(&self) -> f32 {
        self.y
    }

    pub fn set_y<C: RootBoneBaseCallbacks>(&mut self, value: f32, callbacks: &mut C) {
        if self.y == value {
            return;
        }
        self.y = value;
        callbacks.y_changed();
        callbacks.notify_property_changed(Self::Y_PROPERTY_KEY);
    }

    pub fn clone_into<C: RootBoneBaseCallbacks>(&self, callbacks: &mut C) -> RootBone {
        let mut cloned = RootBone::default();
        cloned.base.copy(self, callbacks);
        cloned
    }

    pub fn copy<C: RootBoneBaseCallbacks>(&mut self, object: &Self, callbacks: &mut C) {
        self.x = object.x;
        self.y = object.y;
        self.base.base.copy(&object.base.base, callbacks);
    }

    pub fn deserialize<C: RootBoneBaseCallbacks>(
        &mut self,
        property_key: u16,
        reader: &mut BinaryReader<'_>,
        callbacks: &mut C,
    ) -> bool {
        match property_key {
            Self::X_PROPERTY_KEY => {
                self.x = CoreDoubleType::deserialize(reader);
                true
            }
            Self::Y_PROPERTY_KEY => {
                self.y = CoreDoubleType::deserialize(reader);
                true
            }
            _ => self.base.base.deserialize(property_key, reader, callbacks),
        }
    }
}
