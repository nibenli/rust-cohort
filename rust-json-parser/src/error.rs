use std::fmt;

/// Errors that can occur during the JSON parsing process.
///
/// This enum covers failures ranging from lexical issues (invalid numbers)
/// to structural issues (unexpected tokens or early end-of-file).
#[derive(Debug, Clone, PartialEq)]
pub enum JsonError {
    /// An unexpected character or token was encountered.
    ///
    /// This error occurs when the parser expects a specific structural element
    /// (like a `:` after a key or a `,` between array items) but finds something else.
    ///
    /// ### Example
    /// Parsing `{"key" "value"}` (missing colon) results in an `UnexpectedToken`.
    UnexpectedToken {
        expected: String,
        found: String,
        position: usize,
    },

    /// The input ended before the JSON structure was complete.
    ///
    /// This occurs when brackets, braces, or quotes are left unclosed at the
    /// end of the string.
    ///
    /// ### Example
    /// Parsing `{"name": "Alice"` (missing closing brace) results in `UnexpectedEndOfInput`.
    UnexpectedEndOfInput { expected: String, position: usize },

    /// A sequence of characters intended to be a number could not be parsed.
    ///
    /// This occurs if a number has multiple decimal points, an invalid exponent,
    /// or leading zeros in a way that violates the JSON specification.
    ///
    /// ### Example
    /// Parsing `1.2.3` or `0123` (leading zeros are invalid in JSON) results in `InvalidNumber`.
    InvalidNumber { value: String, position: usize },

    /// An invalid escape sequence was found inside a string.
    ///
    /// JSON strings only support specific escape characters like `\n`, `\t`, `\"`, etc.
    /// Using a backslash followed by an unsupported character triggers this error.
    ///
    /// ### Example
    /// Parsing `"hello \z"` results in `InvalidEscape` because `\z` is not a valid JSON escape.
    InvalidEscape { character: char, position: usize },

    /// A Unicode escape sequence (`\uXXXX`) is malformed or invalid.
    ///
    /// This occurs if the four characters following `\u` are not valid
    /// hexadecimal digits.
    ///
    /// ### Example
    /// Parsing `"\uGGGG"` results in `InvalidUnicode`.
    InvalidUnicode { sequence: String, position: usize },
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
            JsonError::InvalidEscape {
                character,
                position,
            } => {
                write!(
                    f,
                    "Invalid escape sequence '{character}' at position {position}"
                )
            }
            JsonError::InvalidUnicode { sequence, position } => {
                write!(
                    f,
                    "Invalid Unicode escape '\\u{sequence}' at position {position}"
                )
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
        assert!(format!("{error:?}").contains("UnexpectedToken"));
    }
    #[test]
    fn test_error_display() {
        let error = JsonError::UnexpectedToken {
            expected: "valid JSON".into(),
            found: "@".into(),
            position: 0,
        };
        let message = format!("{error}");
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
        let escape_error = JsonError::InvalidEscape {
            character: 'q',
            position: 5,
        };
        let unicode_error = JsonError::InvalidUnicode {
            sequence: "00GG".into(),
            position: 3,
        };
        // All variants should be Debug-printable
        assert!(format!("{token_error:?}").contains("UnexpectedToken"));
        assert!(format!("{eof_error:?}").contains("UnexpectedEndOfInput"));
        assert!(format!("{num_error:?}").contains("InvalidNumber"));
        assert!(format!("{escape_error:?}").contains("InvalidEscape"));
        assert!(format!("{unicode_error:?}").contains("InvalidUnicode"));
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
            (
                JsonError::InvalidEscape {
                    character: 'q',
                    position: 5,
                },
                "Invalid escape sequence 'q' at position 5",
            ),
            (
                JsonError::InvalidUnicode {
                    sequence: "00GG".into(),
                    position: 3,
                },
                "Invalid Unicode escape '\\u00GG' at position 3",
            ),
        ];

        for (error, expected_msg) in cases {
            assert_eq!(format!("{error}"), expected_msg);
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
    fn test_invalid_escape_display() {
        let err = JsonError::InvalidEscape {
            character: 'q',
            position: 5,
        };
        let msg = format!("{err}");
        assert!(msg.contains("escape"));
        assert!(msg.contains("q"));
    }

    #[test]
    fn test_invalid_unicode_display() {
        let err = JsonError::InvalidUnicode {
            sequence: "00GG".to_string(),
            position: 3,
        };
        let msg = format!("{err}");
        assert!(msg.contains("unicode") || msg.contains("Unicode"));
    }

    #[test]
    fn test_error_source() {
        let err = JsonError::InvalidNumber {
            value: "12.3.4".into(),
            position: 0,
        };
        assert!(err.source().is_none());
    }

    #[test]
    fn test_error_is_std_error() {
        let err = JsonError::InvalidEscape {
            character: 'x',
            position: 0,
        };
        let _: &dyn std::error::Error = &err; // Must implement Error trait
    }
}
