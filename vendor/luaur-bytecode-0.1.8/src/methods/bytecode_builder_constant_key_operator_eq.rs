use crate::records::constant_key::ConstantKey;

impl ConstantKey {
    #[inline]
    pub(crate) fn eq(&self, other: &ConstantKey) -> bool {
        self.r#type == other.r#type && self.value == other.value && self.extra == other.extra
    }
}
