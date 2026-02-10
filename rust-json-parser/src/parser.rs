// Week 2: Simple parser for primitive JSON values
use crate::{JsonError, JsonValue, Result, Token, Tokenizer};

#[derive(Debug)]
pub struct JsonParser {
    tokens: Vec<Token>,
    current: usize,
}

impl JsonParser {
    pub fn new(input: &str) -> Result<Self> {
        let mut tokenizer = Tokenizer::new(input);
        let tokens = tokenizer.tokenize()?;

        Ok(Self { tokens, current: 0 })
    }

    pub fn parse(&mut self) -> Result<JsonValue> {
        // Check if it's at the end (empty input or fully consumed)
        if self.is_at_end() {
            return Err(JsonError::UnexpectedEndOfInput {
                expected: "JSON value".to_string(),
                position: self.current,
            });
        }

        // Get the next token with advance()
        let token = match self.advance() {
            Some(t) => t,
            None => {
                return Err(JsonError::UnexpectedEndOfInput {
                    expected: "JSON value".to_string(),
                    position: self.current,
                });
            }
        };

        // Match on the token type and convert to the corresponding JsonValue
        match token {
            Token::String(s) => Ok(JsonValue::String(s)),
            Token::Number(n) => Ok(JsonValue::Number(n)),
            Token::Boolean(b) => Ok(JsonValue::Boolean(b)),
            Token::Null => Ok(JsonValue::Null),
            // Handle unexpected tokens
            t => Err(JsonError::UnexpectedToken {
                expected: "primitive JSON value".to_string(),
                found: format!("{t:?}"),
                position: self.current - 1,
            }),
        }
    }

    // --- Private Helper Methods ---
    /// Consumes and returns the current token, advancing the internal cursor.
    fn advance(&mut self) -> Option<Token> {
        if !self.is_at_end() {
            let token = self.tokens[self.current].clone();
            self.current += 1;
            Some(token)
        } else {
            None
        }
    }

    /// Checks if the parser has run out of tokens.
    pub fn is_at_end(&self) -> bool {
        self.current >= self.tokens.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    // Helper
    fn parse_json(input: &str) -> Result<JsonValue> {
        JsonParser::new(input)?.parse()
    }

    mod parser_creation {
        use super::*;

        #[test]
        fn test_parser_creation() {
            let parser = JsonParser::new("42");
            assert!(parser.is_ok());
        }
        #[test]
        fn test_parser_creation_tokenize_error() {
            let result = JsonParser::new(r#""\q""#);
            assert!(result.is_err());
            match result {
                Err(JsonError::InvalidEscape {
                    character,
                    position,
                }) => {
                    assert_eq!(character, 'q');
                    assert_eq!(position, 1);
                }
                _ => panic!("Expected InvalidEscape error, got {:?}", result),
            }
        }
    }

    mod parser_state_transitions {
        use super::*;

        #[test]
        fn test_successful_consumption_state() {
            // Initial State: Created but not parsed
            let mut parser = JsonParser::new("true").unwrap();
            assert!(
                !parser.is_at_end(),
                "Parser should not be at end before parsing"
            );

            // Transition: Parse the value
            let result = parser.parse();
            assert!(result.is_ok());
            assert_eq!(result.unwrap(), JsonValue::Boolean(true));

            // Final State: Fully consumed
            assert!(
                parser.is_at_end(),
                "Parser should be at end after consuming the only token"
            );
        }

        #[test]
        fn test_repeated_parse_calls_exhaustion() {
            let mut parser = JsonParser::new("null").unwrap();

            // Consume the only token
            let _ = parser.parse();

            // Any subsequent calls to parse should transition to an Error state (EndOfInput)
            let second_call = parser.parse();
            match second_call {
                Err(JsonError::UnexpectedEndOfInput { .. }) => {}
                _ => panic!("Expected UnexpectedEndOfInput error on exhausted parser"),
            }
        }
    }

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

    mod escape_sequences {
        use super::*;

        #[test]
        fn test_parse_string_with_newline() {
            let mut parser = JsonParser::new(r#""hello\nworld""#).unwrap();
            let value = parser.parse().unwrap();
            assert_eq!(value, JsonValue::String("hello\nworld".to_string()));
        }

        #[test]
        fn test_parse_string_with_tab() {
            let mut parser = JsonParser::new(r#""col1\tcol2""#).unwrap();
            let value = parser.parse().unwrap();
            assert_eq!(value, JsonValue::String("col1\tcol2".to_string()));
        }
        #[test]
        fn test_parse_string_with_quotes() {
            let mut parser = JsonParser::new(r#""say \"hi\"""#).unwrap();
            let value = parser.parse().unwrap();
            assert_eq!(value, JsonValue::String("say \"hi\"".to_string()));
        }
        #[test]
        fn test_parse_string_with_unicode() {
            let mut parser = JsonParser::new(r#""\u0048\u0065\u006c\u006c\u006f""#).unwrap();
            let value = parser.parse().unwrap();
            assert_eq!(value, JsonValue::String("Hello".to_string()));
        }
        #[test]
        fn test_parse_complex_escapes() {
            let mut parser = JsonParser::new(r#""line1\nline2\t\"quoted\"\u0021""#).unwrap();
            let value = parser.parse().unwrap();
            assert_eq!(
                value,
                JsonValue::String("line1\nline2\t\"quoted\"!".to_string())
            );
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

        #[test]
        fn test_parse_whitespace_only() {
            let parser = JsonParser::new("   ");
            assert!(parser.is_err() || parser.unwrap().parse().is_err());
        }
    }
}
