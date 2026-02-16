use std::{collections::HashMap, fmt};

#[derive(Debug, Clone, PartialEq)]
pub enum JsonValue {
    Null,
    Boolean(bool),
    Number(f64),
    String(String),
    Array(Vec<JsonValue>),
    Object(HashMap<String, JsonValue>),
}

impl JsonValue {
    pub fn is_null(&self) -> bool {
        matches!(self, JsonValue::Null)
    }
    pub fn as_str(&self) -> Option<&str> {
        match self {
            JsonValue::String(s) => Some(s.as_str()),
            _ => None,
        }
    }
    pub fn as_f64(&self) -> Option<f64> {
        match self {
            JsonValue::Number(n) => Some(*n),
            _ => None,
        }
    }
    pub fn as_bool(&self) -> Option<bool> {
        match self {
            JsonValue::Boolean(b) => Some(*b),
            _ => None,
        }
    }

    pub fn as_array(&self) -> Option<&Vec<JsonValue>> {
        match self {
            JsonValue::Array(arr) => Some(arr),
            _ => None,
        }
    }
    pub fn as_object(&self) -> Option<&HashMap<String, JsonValue>> {
        match self {
            JsonValue::Object(obj) => Some(obj),
            _ => None,
        }
    }
    pub fn get(&self, key: &str) -> Option<&JsonValue> {
        match self {
            JsonValue::Object(obj) => obj.get(key),
            _ => None,
        }
    }
    pub fn get_index(&self, index: usize) -> Option<&JsonValue> {
        match self {
            JsonValue::Array(arr) => arr.get(index),
            _ => None,
        }
    }
}

impl fmt::Display for JsonValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            JsonValue::Null => write!(f, "null"),
            JsonValue::Boolean(b) => write!(f, "{b}"),
            JsonValue::Number(n) => {
                // To match tests like "42.0" -> "42", use default formatting
                write!(f, "{n}")
            }
            JsonValue::String(s) => {
                write!(f, "{}", escape_json_string(s))
            }
            JsonValue::Array(arr) => {
                write!(f, "[")?;
                for (i, val) in arr.iter().enumerate() {
                    if i > 0 {
                        write!(f, ",")?;
                    }
                    write!(f, "{val}")?;
                }
                write!(f, "]")
            }
            JsonValue::Object(obj) => {
                write!(f, "{{")?;
                for (i, (key, val)) in obj.iter().enumerate() {
                    if i > 0 {
                        write!(f, ",")?;
                    }
                    // Keys in JSON are always strings and must be escaped
                    write!(f, "{}:{}", escape_json_string(key), val)?;
                }
                write!(f, "}}")
            }
        }
    }
}

fn escape_json_string(s: &str) -> String {
    let mut escaped = String::with_capacity(s.len() + 2);
    escaped.push('"');
    for c in s.chars() {
        match c {
            '"' => escaped.push_str("\\\""),
            '\\' => escaped.push_str("\\\\"),
            '\n' => escaped.push_str("\\n"),
            '\r' => escaped.push_str("\\r"),
            '\t' => escaped.push_str("\\t"),
            _ => escaped.push(c),
        }
    }
    escaped.push('"');
    escaped
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_json_value_creation() {
        let null_val = JsonValue::Null;
        let bool_val = JsonValue::Boolean(true);
        let num_val = JsonValue::Number(42.5);
        let str_val = JsonValue::String("hello".to_string());
        assert!(null_val.is_null());
        assert_eq!(bool_val.as_bool(), Some(true));
        assert_eq!(num_val.as_f64(), Some(42.5));
        assert_eq!(str_val.as_str(), Some("hello"));
    }
    #[test]
    fn test_json_value_accessors() {
        let value = JsonValue::String("test".to_string());
        assert_eq!(value.as_str(), Some("test"));
        assert_eq!(value.as_f64(), None);
        assert_eq!(value.as_bool(), None);
        assert!(!value.is_null());
        let value = JsonValue::Number(42.0);
        assert_eq!(value.as_f64(), Some(42.0));
        assert_eq!(value.as_str(), None);
        let value = JsonValue::Boolean(true);
        assert_eq!(value.as_bool(), Some(true));
        let value = JsonValue::Null;
        assert!(value.is_null());
    }
    #[test]
    fn test_json_value_equality() {
        assert_eq!(JsonValue::Null, JsonValue::Null);
        assert_eq!(JsonValue::Boolean(true), JsonValue::Boolean(true));
        assert_eq!(JsonValue::Number(42.0), JsonValue::Number(42.0));
        assert_eq!(
            JsonValue::String("test".to_string()),
            JsonValue::String("test".to_string())
        );
        assert_ne!(JsonValue::Null, JsonValue::Boolean(false));
        assert_ne!(JsonValue::Number(1.0), JsonValue::Number(2.0));
    }

    #[test]
    fn test_accessors_return_correct_values() {
        let cases = vec![
            (JsonValue::Null, None, None, None, true),
            (JsonValue::Boolean(true), None, None, Some(true), false),
            (JsonValue::Number(123.45), None, Some(123.45), None, false),
            (
                JsonValue::String("Rust".into()),
                Some("Rust"),
                None,
                None,
                false,
            ),
        ];

        for (val, expected_str, expected_f64, expected_bool, is_null) in cases {
            assert_eq!(val.as_str(), expected_str, "Failed as_str on {:?}", val);
            assert_eq!(val.as_f64(), expected_f64, "Failed as_f64 on {:?}", val);
            assert_eq!(val.as_bool(), expected_bool, "Failed as_bool on {:?}", val);
            assert_eq!(val.is_null(), is_null, "Failed is_null on {:?}", val);
        }
    }

    #[test]
    fn test_numeric_edge_cases() {
        let nan_val = JsonValue::Number(f64::NAN);
        let inf_val = JsonValue::Number(f64::INFINITY);

        // NaN != NaN by IEEE 754 standards.
        assert!(nan_val.as_f64().unwrap().is_nan());
        assert_eq!(inf_val.as_f64(), Some(f64::INFINITY));
    }

    #[test]
    fn test_array_accessor() {
        // Create: [true, 42.0]
        let array_val = JsonValue::Array(vec![JsonValue::Boolean(true), JsonValue::Number(42.0)]);

        // Success case
        assert!(array_val.as_array().is_some());
        assert_eq!(array_val.as_array().unwrap().len(), 2);

        // Failure case: A boolean is not an array
        let bool_val = JsonValue::Boolean(true);
        assert!(bool_val.as_array().is_none());
    }

    #[test]
    fn test_object_accessor() {
        let mut map = HashMap::new();
        map.insert("id".to_string(), JsonValue::Number(1.0));
        let obj_val = JsonValue::Object(map);

        // Success case
        assert!(obj_val.as_object().is_some());
        assert!(obj_val.as_object().unwrap().contains_key("id"));

        // Failure case: A null is not an object
        let null_val = JsonValue::Null;
        assert!(null_val.as_object().is_none());
    }

    #[test]
    fn test_array_get_index() {
        let array_val = JsonValue::Array(vec![
            JsonValue::String("first".to_string()),
            JsonValue::String("second".to_string()),
        ]);

        // Valid indexes
        assert_eq!(
            array_val.get_index(0).and_then(|v| v.as_str()),
            Some("first")
        );
        assert_eq!(
            array_val.get_index(1).and_then(|v| v.as_str()),
            Some("second")
        );

        // Out of bounds
        assert!(array_val.get_index(2).is_none());

        // Type mismatch: calling get_index on a Number
        let num_val = JsonValue::Number(10.0);
        assert!(num_val.get_index(0).is_none());
    }

    #[test]
    fn test_object_get() {
        let mut map = HashMap::new();
        map.insert("name".to_string(), JsonValue::String("Mike".to_string()));
        let obj_val = JsonValue::Object(map);

        // Valid key
        let name = obj_val.get("name");
        assert!(name.is_some());
        assert_eq!(name.unwrap().as_str(), Some("Mike"));

        // Missing key
        assert!(obj_val.get("age").is_none());

        // Type mismatch: calling get on an Array
        let arr_val = JsonValue::Array(vec![]);
        assert!(arr_val.get("name").is_none());
    }

    mod display_tests {
        use super::*;
        use crate::JsonParser;

        // Helper
        fn parse_json(input: &str) -> crate::Result<JsonValue> {
            JsonParser::new(input)?.parse()
        }
        #[test]
        fn test_display_primitives() {
            assert_eq!(JsonValue::Null.to_string(), "null");
            assert_eq!(JsonValue::Boolean(true).to_string(), "true");
            assert_eq!(JsonValue::Boolean(false).to_string(), "false");
            assert_eq!(JsonValue::Number(42.0).to_string(), "42");
            assert_eq!(JsonValue::Number(3.14).to_string(), "3.14");
            assert_eq!(
                JsonValue::String("hello".to_string()).to_string(),
                "\"hello\""
            );
        }
        #[test]
        fn test_display_array() {
            let value = JsonValue::Array(vec![JsonValue::Number(1.0), JsonValue::Number(2.0)]);
            assert_eq!(value.to_string(), "[1,2]");
        }
        #[test]
        fn test_display_empty_containers() {
            assert_eq!(JsonValue::Array(vec![]).to_string(), "[]");
            assert_eq!(JsonValue::Object(HashMap::new()).to_string(), "{}");
        }
        #[test]
        fn test_display_escape_string() {
            let value = JsonValue::String("hello\nworld".to_string());
            assert_eq!(value.to_string(), "\"hello\\nworld\"");
        }
        #[test]
        fn test_display_escape_quotes() {
            let value = JsonValue::String("say \"hi\"".to_string());
            assert_eq!(value.to_string(), "\"say \\\"hi\\\"\"");
        }
        #[test]
        fn test_display_nested() {
            let value = parse_json(r#"{"arr": [1, 2]}"#).unwrap();
            let output = value.to_string();
            // Object key order may vary, so check components
            assert!(output.contains("\"arr\""));
            assert!(output.contains("[1,2]"));
        }
    }
}
