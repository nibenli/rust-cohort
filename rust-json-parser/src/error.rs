use std::fmt;
#[derive(Debug, Clone, PartialEq)]
pub enum JsonError {
    UnexpectedToken {
        expected: String,
        found: String,
        position: usize,
    },
    UnexpectedEndOfInput {
        expected: String,
        position: usize,
    },
    InvalidNumber {
        value: String,
        position: usize,
    },
}
impl fmt::Display for JsonError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            JsonError::UnexpectedToken {
                expected,
                found,
                position,
            } => {
                write!(
                    f,
                    "Unexpected token at position {position}: expected {expected}, found {found}"
                )
            }
            JsonError::UnexpectedEndOfInput { expected, position } => {
                write!(
                    f,
                    "Unexpected end of input at position {position}: expected {expected}"
                )
            }
            JsonError::InvalidNumber { value, position } => {
                write!(f, "Invalid number at position {position}: value {value}")
            }
        }
    }
}

impl std::error::Error for JsonError {}

#[cfg(test)]
mod tests {
    use super::*;
    use std::error::Error;
    #[test]
    fn test_error_creation() {
        let error = JsonError::UnexpectedToken {
            expected: "number".into(),
            found: "@".into(),
            position: 5,
        };
        // Error should be Debug-printable
        assert!(format!("{:?}", error).contains("UnexpectedToken"));
    }
    #[test]
    fn test_error_display() {
        let error = JsonError::UnexpectedToken {
            expected: "valid JSON".into(),
            found: "@".into(),
            position: 0,
        };
        let message = format!("{}", error);
        assert!(message.contains("position 0"));
        assert!(message.contains("valid JSON"));
        assert!(message.contains("@"));
    }
    #[test]
    fn test_error_variants() {
        let token_error = JsonError::UnexpectedToken {
            expected: "number".into(),
            found: "x".into(),
            position: 3,
        };
        let eof_error = JsonError::UnexpectedEndOfInput {
            expected: "closing quote".into(),
            position: 10,
        };
        let num_error = JsonError::InvalidNumber {
            value: "12.34.56".into(),
            position: 0,
        };
        // All variants should be Debug-printable
        assert!(format!("{:?}", token_error).contains("UnexpectedToken"));
        assert!(format!("{:?}", eof_error).contains("UnexpectedEndOfInput"));
        assert!(format!("{:?}", num_error).contains("InvalidNumber"));
    }

    #[test]
    fn test_display_output_variants() {
        let cases = vec![
            (
                JsonError::UnexpectedToken {
                    expected: "string".into(),
                    found: "true".into(),
                    position: 42,
                },
                "Unexpected token at position 42: expected string, found true",
            ),
            (
                JsonError::UnexpectedEndOfInput {
                    expected: "}".into(),
                    position: 100,
                },
                "Unexpected end of input at position 100: expected }",
            ),
            (
                JsonError::InvalidNumber {
                    value: "1.2.3".into(),
                    position: 5,
                },
                "Invalid number at position 5: value 1.2.3",
            ),
        ];

        for (error, expected_msg) in cases {
            assert_eq!(format!("{}", error), expected_msg);
        }
    }

    #[test]
    fn test_equality_and_inequality() {
        let e1 = JsonError::InvalidNumber {
            value: "abc".into(),
            position: 1,
        };
        let e1_dup = e1.clone();
        let e2 = JsonError::InvalidNumber {
            value: "efg".into(),
            position: 1,
        };
        let e3 = JsonError::UnexpectedEndOfInput {
            expected: "end".into(),
            position: 1,
        };

        // Test PartialEq works correctly
        assert_eq!(e1, e1_dup);
        assert_ne!(e1, e2, "Errors with different values should not be equal");
        assert_ne!(e1, e3, "Different variants should not be equal");
    }

    #[test]
    fn test_error_source() {
        let err = JsonError::InvalidNumber {
            value: "12.3.4".into(),
            position: 0,
        };
        assert!(err.source().is_none());
    }
}
