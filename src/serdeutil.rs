use serde::Deserializer;

pub fn deserialize_bool_from_int_or_bool<'de, D>(deserializer: D) -> Result<Option<bool>, D::Error>
where
    D: Deserializer<'de>,
{
    use serde::de::{self, Unexpected, Visitor};
    use std::fmt;

    struct BoolVisitor;

    impl<'de> Visitor<'de> for BoolVisitor {
        type Value = Option<bool>;

        fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
            formatter.write_str("a boolean, integer (0 or 1), or null")
        }

        fn visit_bool<E>(self, value: bool) -> Result<Self::Value, E>
        where
            E: de::Error,
        {
            Ok(Some(value))
        }

        fn visit_i64<E>(self, value: i64) -> Result<Self::Value, E>
        where
            E: de::Error,
        {
            match value {
                0 => Ok(Some(false)),
                1 => Ok(Some(true)),
                _ => Err(E::invalid_value(Unexpected::Signed(value), &self)),
            }
        }

        fn visit_u64<E>(self, value: u64) -> Result<Self::Value, E>
        where
            E: de::Error,
        {
            match value {
                0 => Ok(Some(false)),
                1 => Ok(Some(true)),
                _ => Err(E::invalid_value(Unexpected::Unsigned(value), &self)),
            }
        }

        fn visit_unit<E>(self) -> Result<Self::Value, E>
        where
            E: de::Error,
        {
            Ok(None)
        }

        fn visit_none<E>(self) -> Result<Self::Value, E>
        where
            E: de::Error,
        {
            Ok(None)
        }
    }

    deserializer.deserialize_any(BoolVisitor)
}
