//! Request containers require named fields. Deserialize field values directly
//! through serde, preserving its typed float parsing and field diagnostics.
use serde::{Deserialize, Deserializer, de::{MapAccess, Visitor, value::MapAccessDeserializer}};
use std::{fmt, marker::PhantomData};

pub(crate) fn object<'de, D, T>(deserializer: D, expected: &'static str) -> Result<T, D::Error>
where D: Deserializer<'de>, T: Deserialize<'de> {
    struct Object<T> { expected: &'static str, value: PhantomData<T> }
    impl<'de, T: Deserialize<'de>> Visitor<'de> for Object<T> {
        type Value = T;
        fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
            formatter.write_str(self.expected)
        }
        fn visit_map<A: MapAccess<'de>>(self, map: A) -> Result<T, A::Error> {
            T::deserialize(MapAccessDeserializer::new(map))
        }
    }
    deserializer.deserialize_map(Object { expected, value: PhantomData })
}
