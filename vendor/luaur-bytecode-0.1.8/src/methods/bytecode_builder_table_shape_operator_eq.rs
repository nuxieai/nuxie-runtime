use crate::records::table_shape::TableShape;

impl TableShape {
    #[allow(non_snake_case)]
    pub(crate) fn operator_eq(&self, other: &TableShape) -> bool {
        if self.length != other.length {
            return false;
        }

        let len = self.length as usize;
        if self.keys[..len] != other.keys[..len] {
            return false;
        }

        if self.hasConstants != other.hasConstants {
            return false;
        }

        if self.hasConstants {
            if self.constants[..len] != other.constants[..len] {
                return false;
            }
        }

        true
    }
}
