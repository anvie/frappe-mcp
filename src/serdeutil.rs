// Copyright (C) 2025 Nuwaira
// All Rights Reserved.
//
// NOTICE: All information contained herein is, and remains
// the property of Nuwaira.
// The intellectual and technical concepts contained
// herein are proprietary to Nuwaira
// and are protected by trade secret or copyright law.
// Dissemination of this information or reproduction of this material
// is strictly forbidden unless prior written permission is obtained
// from Nuwaira.
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

pub fn deserialize_bool_from_int_or_bool_to_int<'de, D>(
    deserializer: D,
) -> Result<Option<u16>, D::Error>
where
    D: Deserializer<'de>,
{
    use serde::de::{self, Unexpected, Visitor};
    use std::fmt;

    struct BoolVisitor;

    impl<'de> Visitor<'de> for BoolVisitor {
        type Value = Option<u16>;

        fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
            formatter.write_str("a boolean, integer (0 or 1), or null")
        }

        fn visit_bool<E>(self, value: bool) -> Result<Self::Value, E>
        where
            E: de::Error,
        {
            match value {
                true => Ok(Some(1)),
                false => Ok(Some(0)),
            }
        }

        fn visit_i64<E>(self, value: i64) -> Result<Self::Value, E>
        where
            E: de::Error,
        {
            match value {
                0 => Ok(Some(0)),
                1 => Ok(Some(1)),
                _ => Err(E::invalid_value(Unexpected::Signed(value), &self)),
            }
        }

        fn visit_u64<E>(self, value: u64) -> Result<Self::Value, E>
        where
            E: de::Error,
        {
            match value {
                0 => Ok(Some(0)),
                1 => Ok(Some(1)),
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
