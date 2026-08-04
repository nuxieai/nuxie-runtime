use crate::records::constant_key::ConstantKey;

impl ConstantKey {
    #[inline]
    pub(crate) fn eq(&self, other: &ConstantKey) -> bool {
        self.r#type == other.r#type
            && self.value == other.value
            && self.extra1 == other.extra1
            && self.extra2 == other.extra2
            && self.extra3 == other.extra3
    }
}
