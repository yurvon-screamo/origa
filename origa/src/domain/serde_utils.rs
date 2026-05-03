use serde::de;

pub fn serialize_u64_as_string<S: serde::Serializer>(val: &u64, s: S) -> Result<S::Ok, S::Error> {
    s.serialize_str(&val.to_string())
}

pub fn deserialize_u64_from_str_or_num<'de, D: serde::Deserializer<'de>>(
    d: D,
) -> Result<u64, D::Error> {
    struct U64Visitor;

    impl<'de> de::Visitor<'de> for U64Visitor {
        type Value = u64;

        fn expecting(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
            write!(f, "a u64 as number or string")
        }

        fn visit_u64<E: de::Error>(self, v: u64) -> Result<u64, E> {
            Ok(v)
        }

        fn visit_i64<E: de::Error>(self, v: i64) -> Result<u64, E> {
            Ok(v as u64)
        }

        fn visit_str<E: de::Error>(self, v: &str) -> Result<u64, E> {
            v.parse::<u64>().map_err(de::Error::custom)
        }
    }

    d.deserialize_any(U64Visitor)
}

pub mod option_u64_as_string {
    use serde::de;
    use super::{deserialize_u64_from_str_or_num, serialize_u64_as_string};

    pub fn serialize<S: serde::Serializer>(val: &Option<u64>, s: S) -> Result<S::Ok, S::Error> {
        match val {
            Some(v) => serialize_u64_as_string(v, s),
            None => s.serialize_none(),
        }
    }

    pub fn deserialize<'de, D: serde::Deserializer<'de>>(d: D) -> Result<Option<u64>, D::Error> {
        d.deserialize_option(OptionVisitor)
    }

    struct OptionVisitor;

    impl<'de> de::Visitor<'de> for OptionVisitor {
        type Value = Option<u64>;

        fn expecting(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
            write!(f, "a u64 as number, string, or null")
        }

        fn visit_none<E: de::Error>(self) -> Result<Option<u64>, E> {
            Ok(None)
        }

        fn visit_some<D: de::Deserializer<'de>>(self, d: D) -> Result<Option<u64>, D::Error> {
            deserialize_u64_from_str_or_num(d).map(Some)
        }

        fn visit_unit<E: de::Error>(self) -> Result<Option<u64>, E> {
            Ok(None)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug, serde::Serialize, serde::Deserialize)]
    struct TestStruct {
        #[serde(
            serialize_with = "serialize_u64_as_string",
            deserialize_with = "deserialize_u64_from_str_or_num"
        )]
        value: u64,
    }

    #[derive(Debug, serde::Serialize, serde::Deserialize)]
    struct TestOptionStruct {
        #[serde(
            default,
            serialize_with = "option_u64_as_string::serialize",
            deserialize_with = "option_u64_as_string::deserialize"
        )]
        value: Option<u64>,
    }

    #[test]
    fn serializes_u64_as_string() {
        let item = TestStruct { value: 123456789 };
        let json = serde_json::to_string(&item).expect("Failed to serialize");
        assert_eq!(json, r#"{"value":"123456789"}"#);
    }

    #[test]
    fn deserializes_u64_from_string() {
        let json = r#"{"value":"123456789"}"#;
        let item: TestStruct = serde_json::from_str(json).expect("Failed to deserialize");
        assert_eq!(item.value, 123456789);
    }

    #[test]
    fn deserializes_u64_from_number() {
        let json = r#"{"value":123456789}"#;
        let item: TestStruct = serde_json::from_str(json).expect("Failed to deserialize");
        assert_eq!(item.value, 123456789);
    }

    #[test]
    fn roundtrip_preserves_value() {
        let original = TestStruct {
            value: u64::MAX / 2,
        };
        let json = serde_json::to_string(&original).expect("Failed to serialize");
        let restored: TestStruct = serde_json::from_str(&json).expect("Failed to deserialize");
        assert_eq!(original.value, restored.value);
    }

    #[test]
    fn serializes_option_some_as_string() {
        let item = TestOptionStruct {
            value: Some(123456789),
        };
        let json = serde_json::to_string(&item).expect("Failed to serialize");
        assert_eq!(json, r#"{"value":"123456789"}"#);
    }

    #[test]
    fn serializes_option_none_as_null() {
        let item = TestOptionStruct { value: None };
        let json = serde_json::to_string(&item).expect("Failed to serialize");
        assert_eq!(json, r#"{"value":null}"#);
    }

    #[test]
    fn deserializes_option_from_string() {
        let json = r#"{"value":"123456789"}"#;
        let item: TestOptionStruct = serde_json::from_str(json).expect("Failed to deserialize");
        assert_eq!(item.value, Some(123456789));
    }

    #[test]
    fn deserializes_option_from_number() {
        let json = r#"{"value":123456789}"#;
        let item: TestOptionStruct = serde_json::from_str(json).expect("Failed to deserialize");
        assert_eq!(item.value, Some(123456789));
    }

    #[test]
    fn deserializes_option_from_null() {
        let json = r#"{"value":null}"#;
        let item: TestOptionStruct = serde_json::from_str(json).expect("Failed to deserialize");
        assert_eq!(item.value, None);
    }

    #[test]
    fn deserializes_option_from_missing_field() {
        let json = r#"{}"#;
        let item: TestOptionStruct = serde_json::from_str(json).expect("Failed to deserialize");
        assert_eq!(item.value, None);
    }

    #[test]
    fn option_roundtrip_preserves_some() {
        let original = TestOptionStruct {
            value: Some(9007199254740993),
        };
        let json = serde_json::to_string(&original).expect("Failed to serialize");
        let restored: TestOptionStruct =
            serde_json::from_str(&json).expect("Failed to deserialize");
        assert_eq!(original.value, restored.value);
    }
}
