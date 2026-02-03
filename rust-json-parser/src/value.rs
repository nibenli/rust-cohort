#[derive(Debug, Clone, PartialEq)]
pub enum JsonValue {
    Null,
    Boolean(bool),
    Number(f64),
    String(String),
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
}
