use std::fmt;

use serde::de::{self, MapAccess, Visitor};
use serde::{Deserialize, Deserializer, Serialize};

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub struct Pricing {
    price: u64,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub struct PricingAt {
    price: u64,
    at: u64,
}

#[derive(Clone, Debug)]
struct Builtin {
    name: String,
    pricing: Vec<PricingAt>,
    at: u64,
}

impl<'de> Deserialize<'de> for Builtin {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        enum Field {
            Name,
            Pricing,
            At,
        };

        impl<'de> Deserialize<'de> for Field {
            fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
            where
                D: Deserializer<'de>,
            {
                struct FieldVisitor;

                impl<'de> Visitor<'de> for FieldVisitor {
                    type Value = Field;

                    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                        formatter.write_str("`name`, `pricing` or `at`")
                    }

                    fn visit_str<E>(self, value: &str) -> Result<Field, E>
                    where
                        E: de::Error,
                    {
                        match value {
                            "name" => Ok(Field::Name),
                            "pricing" => Ok(Field::Pricing),
                            "at" => Ok(Field::At),
                            _ => Err(de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(FieldVisitor)
            }
        }

        struct BuiltinVisitor;

        impl<'de> Visitor<'de> for BuiltinVisitor {
            type Value = Builtin;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("struct Builtin")
            }

            fn visit_map<V>(self, mut map: V) -> Result<Builtin, V::Error>
            where
                V: MapAccess<'de>,
            {
                let mut name = None;
                let mut pricing = None;
                let mut at = None;

                while let Some(key) = map.next_key()? {
                    match key {
                        Field::Name => {
                            if name.is_some() {
                                return Err(de::Error::duplicate_field("name"));
                            }
                            name = Some(map.next_value()?);
                        }
                        Field::Pricing => {
                            if pricing.is_some() {
                                return Err(de::Error::duplicate_field("pricing"));
                            }
                            pricing = Some(map.next_value()?);
                        }
                        Field::At => {
                            if at.is_some() {
                                return Err(de::Error::duplicate_field("at"));
                            }
                            at = Some(map.next_value()?);
                        }
                    }
                }

                let name = name.ok_or_else(|| de::Error::missing_field("name"))?;
                let pricing = pricing.ok_or_else(|| de::Error::missing_field("pricing"))?;
                let at = at.ok_or_else(|| de::Error::missing_field("at"))?;

                // TODO(niklasad1): how to check if pricing is of type `Amount` or `AmountAt`?
                // Then
                // ```rust
                //  let amount_at: AmountAt = amount.into();
                //  vec![amount_at]
                //```
                //
                // It could be possible to `serde_json::{to_value, from_value}` deserialize here to check

                Ok(Builtin { name, pricing, at })
            }
        }

        const FIELDS: &[&str] = &["name", "pricing", "at"];
        deserializer.deserialize_struct("Builtin", FIELDS, BuiltinVisitor)
    }
}

#[cfg(test)]
mod tests {
    use super::{Builtin, PricingAt};

    #[test]
    fn deserialize_empty_vec() {
        let raw = r#"{
            "name": "foo",
            "pricing": [],
            "at": 1
        }"#;
        let builtin: Builtin = serde_json::from_str(raw).unwrap();
        assert_eq!(builtin.name, "foo".to_string());
        assert!(builtin.pricing.is_empty());
        assert_eq!(builtin.at, 1);
    }

    #[test]
    fn deserialize_vec() {
        let raw = r#"{
            "name": "bar",
            "pricing": [ {"price": 100, "at": 0}, {"price": 0, "at": 11} ],
            "at": 0
        }"#;
        let builtin: Builtin = serde_json::from_str(raw).unwrap();
        assert_eq!(builtin.name, "bar".to_string());
        assert_eq!(
            builtin.pricing,
            vec![
                PricingAt { price: 100, at: 0 },
                PricingAt { price: 0, at: 11 }
            ]
        );
        assert_eq!(builtin.at, 0);
    }

    #[test]
    #[ignore]
    // don't work
    fn deserialize_object() {
        let raw = r#"{
            "name": "foo",
            "pricing": { "price": 1000 },
            "at": 999
        }"#;

        let builtin: Builtin = serde_json::from_str(raw).unwrap();
        assert_eq!(builtin.name, "foo".to_string());
        assert_eq!(
            builtin.pricing,
            vec![PricingAt {
                price: 100,
                at: 999
            }]
        );
        assert_eq!(builtin.at, 999);
    }
}
