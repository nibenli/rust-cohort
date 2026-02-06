// Week 2: Simple parser for primitive JSON values
use crate::{JsonError, JsonValue, Result, Token, tokenize};

pub fn parse_json(input: &str) -> Result<JsonValue> {
    // 1. Call tokenize(input)?
    let tokens = tokenize(input)?;

    // 2. Check if tokens is empty
    if tokens.is_empty() {
        return Err(JsonError::UnexpectedEndOfInput {
            expected: "JSON value".to_string(),
            position: 0,
        });
    }

    // 3. Match on tokens[0] and convert to JsonValue
    match &tokens[0] {
        Token::String(s) => Ok(JsonValue::String(s.clone())),
        Token::Number(n) => Ok(JsonValue::Number(*n)),
        Token::Boolean(b) => Ok(JsonValue::Boolean(*b)),
        Token::Null => Ok(JsonValue::Null),
        // Handle other token types...
        // If we hit structural tokens like { or [ without further logic,
        // they are technically "unexpected" for a single-token parser.
        t => Err(JsonError::UnexpectedToken {
            expected: "primitive JSON value".to_string(),
            found: format!("{t:?}"),
            position: 0, // In a full parser, you'd track the actual position
        }),
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    mod success_cases {
        use super::*;

        #[test]
        fn test_all_primitives() {
            // Table-driven test: (input_string, expected_value)
            let cases = vec![
                // Strings
                (
                    r#""hello world""#,
                    JsonValue::String("hello world".to_string()),
                ),
                (r#""""#, JsonValue::String("".to_string())),
                (r#""123""#, JsonValue::String("123".to_string())),
                // Numbers
                ("42.5", JsonValue::Number(42.5)),
                ("0", JsonValue::Number(0.0)),
                ("-10", JsonValue::Number(-10.0)),
                ("1e10", JsonValue::Number(1e10)),
                // Booleans
                ("true", JsonValue::Boolean(true)),
                ("false", JsonValue::Boolean(false)),
                // Null
                ("null", JsonValue::Null),
            ];

            for (input, expected) in cases {
                let result = parse_json(input).unwrap_or_else(|e| {
                    panic!("Failed to parse '{}': {}", input, e);
                });
                assert_eq!(result, expected, "Input failed: {}", input);
            }
        }

        #[test]
        fn test_parse_with_whitespace() {
            let cases = ["   true", "false   ", "\n123\t", "  null  "];
            for input in cases {
                assert!(
                    parse_json(input).is_ok(),
                    "Should handle whitespace for: {}",
                    input
                );
            }
        }
    }

    mod error_cases {
        use super::*;

        #[test]
        fn test_parse_error_empty() {
            let result = parse_json("");
            assert!(result.is_err());
            match result {
                Err(JsonError::UnexpectedEndOfInput { expected, position }) => {
                    assert_eq!(expected, "JSON value");
                    assert_eq!(position, 0);
                }
                _ => panic!("Expected UnexpectedEndOfInput error"),
            }
        }

        #[test]
        fn test_parse_error_invalid_token() {
            let invalid_inputs = ["@", "$", "%", "^", "!", "None", "undefined", "tru"];
            for input in invalid_inputs {
                let result = parse_json(input);
                assert!(
                    matches!(result, Err(JsonError::UnexpectedToken { .. })),
                    "Should return UnexpectedToken for: {}",
                    input
                );
            }
        }

        #[test]
        fn test_malformed_numbers() {
            let malformed = ["1.2.3", "1e", "--10", "1.0.e10"];
            for input in malformed {
                let result = parse_json(input);
                assert!(
                    matches!(result, Err(JsonError::InvalidNumber { .. })),
                    "Should return InvalidNumber for: {}",
                    input
                );
            }
        }
    }
}
