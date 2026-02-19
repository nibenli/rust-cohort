use crate::{JsonError, JsonValue, Result, Token, Tokenizer};
use std::collections::HashMap;
use std::mem::discriminant;

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
        let token = self.peek().ok_or(JsonError::UnexpectedEndOfInput {
            expected: "JSON value".to_string(),
            position: self.current,
        })?;

        match token {
            Token::LeftBracket => self.parse_array(),
            Token::LeftBrace => self.parse_object(),
            // All other tokens are treated as potential primitives
            _ => self.parse_primitives(),
        }
    }

    /// Handles Null, Boolean, Number, and String variants.
    fn parse_primitives(&mut self) -> Result<JsonValue> {
        if let Some(token) = self.advance() {
            match token {
                Token::Null => Ok(JsonValue::Null),
                Token::Boolean(b) => Ok(JsonValue::Boolean(b)),
                Token::Number(n) => Ok(JsonValue::Number(n)),
                Token::String(s) => Ok(JsonValue::String(s)),
                t => {
                    let pos = self.previous_pos();
                    Err(JsonError::UnexpectedToken {
                        expected: "value".to_string(),
                        found: format!("{t:?}"),
                        position: pos,
                    })
                }
            }
        } else {
            Err(JsonError::UnexpectedEndOfInput {
                expected: "JSON value".to_string(),
                position: self.current,
            })
        }
    }

    fn parse_array(&mut self) -> Result<JsonValue> {
        self.advance(); // Consume '['
        let mut elements = Vec::new();

        if self.check(&Token::RightBracket) {
            self.advance();
            return Ok(JsonValue::Array(elements));
        }

        loop {
            elements.push(self.parse()?);

            match self.advance() {
                Some(Token::Comma) => {
                    if self.check(&Token::RightBracket) {
                        return Err(JsonError::UnexpectedToken {
                            expected: "value".to_string(),
                            found: "']' (trailing comma)".to_string(),
                            position: self.previous_pos(),
                        });
                    }
                }
                Some(Token::RightBracket) => break,
                Some(t) => {
                    return Err(JsonError::UnexpectedToken {
                        expected: "',' or ']'".to_string(),
                        found: format!("{t:?}"),
                        position: self.previous_pos(),
                    });
                }
                None => {
                    return Err(JsonError::UnexpectedEndOfInput {
                        expected: "']'".to_string(),
                        position: self.current,
                    });
                }
            }
        }
        Ok(JsonValue::Array(elements))
    }

    fn parse_object(&mut self) -> Result<JsonValue> {
        self.advance(); // Consume '{'
        let mut map = HashMap::new();

        if self.check(&Token::RightBrace) {
            self.advance();
            return Ok(JsonValue::Object(map));
        }

        loop {
            let key = match self.advance() {
                Some(Token::String(s)) => s,
                Some(t) => {
                    return Err(JsonError::UnexpectedToken {
                        expected: "string key".to_string(),
                        found: format!("{t:?}"),
                        position: self.previous_pos(),
                    });
                }
                None => {
                    return Err(JsonError::UnexpectedEndOfInput {
                        expected: "string key".to_string(),
                        position: self.current,
                    });
                }
            };

            match self.advance() {
                Some(Token::Colon) => {}
                Some(t) => {
                    return Err(JsonError::UnexpectedToken {
                        expected: "':'".to_string(),
                        found: format!("{t:?}"),
                        position: self.previous_pos(),
                    });
                }
                None => {
                    return Err(JsonError::UnexpectedEndOfInput {
                        expected: "':'".to_string(),
                        position: self.current,
                    });
                }
            }

            map.insert(key, self.parse()?);

            match self.advance() {
                Some(Token::Comma) => {
                    if self.check(&Token::RightBrace) {
                        return Err(JsonError::UnexpectedToken {
                            expected: "string key".to_string(),
                            found: "'}' (trailing comma)".to_string(),
                            position: self.previous_pos(),
                        });
                    }
                }
                Some(Token::RightBrace) => break,
                Some(t) => {
                    return Err(JsonError::UnexpectedToken {
                        expected: "',' or '}'".to_string(),
                        found: format!("{t:?}"),
                        position: self.previous_pos(),
                    });
                }
                None => {
                    return Err(JsonError::UnexpectedEndOfInput {
                        expected: "'}'".to_string(),
                        position: self.current,
                    });
                }
            }
        }
        Ok(JsonValue::Object(map))
    }

    // --- Helpers ---

    /// Returns the index of the token just consumed.
    fn previous_pos(&self) -> usize {
        self.current.saturating_sub(1)
    }

    fn check(&self, expected: &Token) -> bool {
        self.peek()
            .is_some_and(|actual| discriminant(actual) == discriminant(expected))
    }

    fn peek(&self) -> Option<&Token> {
        self.tokens.get(self.current)
    }

    fn advance(&mut self) -> Option<Token> {
        if !self.is_at_end() {
            let token = self.tokens[self.current].clone();
            self.current += 1;
            Some(token)
        } else {
            None
        }
    }

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

        #[test]
        fn test_error_unclosed_array() {
            let result = parse_json("[1, 2");
            assert!(result.is_err());
        }
        #[test]
        fn test_error_unclosed_object() {
            let result = parse_json(r#"{"key": 1"#);
            assert!(result.is_err());
        }
        #[test]
        fn test_error_trailing_comma_array() {
            let result = parse_json("[1, 2,]");
            assert!(result.is_err());
        }
        #[test]
        fn test_error_trailing_comma_object() {
            let result = parse_json(r#"{"a": 1,}"#);
            assert!(result.is_err());
        }
        #[test]
        fn test_error_missing_colon() {
            let result = parse_json(r#"{"key" 1}"#);
            assert!(result.is_err());
        }
        #[test]
        fn test_error_invalid_key() {
            let result = parse_json(r#"{123: "value"}"#);
            assert!(result.is_err());
        }
        #[test]
        fn test_error_missing_comma_array() {
            let result = parse_json("[1 2 3]");
            assert!(result.is_err());
        }
        #[test]
        fn test_error_missing_comma_object() {
            let result = parse_json(r#"{"a": 1 "b": 2}"#);
            assert!(result.is_err());
        }
    }

    mod array_tests {
        use super::*;
        #[test]
        fn test_parse_empty_array() {
            let value = parse_json("[]").unwrap();
            assert_eq!(value, JsonValue::Array(vec![]));
        }
        #[test]
        fn test_parse_array_single() {
            let value = parse_json("[1]").unwrap();
            assert_eq!(value, JsonValue::Array(vec![JsonValue::Number(1.0)]));
        }
        #[test]
        fn test_parse_array_multiple() {
            let value = parse_json("[1, 2, 3]").unwrap();
            let expected = JsonValue::Array(vec![
                JsonValue::Number(1.0),
                JsonValue::Number(2.0),
                JsonValue::Number(3.0),
            ]);
            assert_eq!(value, expected);
        }
        #[test]
        fn test_parse_array_mixed_types() {
            let value = parse_json(r#"[1, "two", true, null]"#).unwrap();
            let expected = JsonValue::Array(vec![
                JsonValue::Number(1.0),
                JsonValue::String("two".to_string()),
                JsonValue::Boolean(true),
                JsonValue::Null,
            ]);
            assert_eq!(value, expected);
        }
        #[test]
        fn test_parse_nested_arrays() {
            let value = parse_json("[[1, 2], [3, 4]]").unwrap();
            let expected = JsonValue::Array(vec![
                JsonValue::Array(vec![JsonValue::Number(1.0), JsonValue::Number(2.0)]),
                JsonValue::Array(vec![JsonValue::Number(3.0), JsonValue::Number(4.0)]),
            ]);
            assert_eq!(value, expected);
        }
        #[test]
        fn test_parse_deeply_nested() {
            let value = parse_json("[[[1]]]").unwrap();
            let expected = JsonValue::Array(vec![JsonValue::Array(vec![JsonValue::Array(vec![
                JsonValue::Number(1.0),
            ])])]);
            assert_eq!(value, expected);
        }
        #[test]
        fn test_array_accessor() {
            let value = parse_json("[1, 2, 3]").unwrap();
            let arr = value.as_array().unwrap();
            assert_eq!(arr.len(), 3);
        }
        #[test]
        fn test_array_get_index() {
            let value = parse_json("[10, 20, 30]").unwrap();
            assert_eq!(value.get_index(1), Some(&JsonValue::Number(20.0)));
            assert_eq!(value.get_index(5), None);
        }
    }

    mod object_tests {
        use super::*;
        #[test]
        fn test_parse_empty_object() {
            let value = parse_json("{}").unwrap();
            assert_eq!(value, JsonValue::Object(HashMap::new()));
        }
        #[test]
        fn test_parse_object_single_key() {
            let value = parse_json(r#"{"key": "value"}"#).unwrap();
            let mut expected = HashMap::new();
            expected.insert("key".to_string(), JsonValue::String("value".to_string()));
            assert_eq!(value, JsonValue::Object(expected));
        }
        #[test]
        fn test_parse_object_multiple_keys() {
            let value = parse_json(r#"{"name": "Alice", "age": 30}"#).unwrap();
            if let JsonValue::Object(obj) = value {
                assert_eq!(
                    obj.get("name"),
                    Some(&JsonValue::String("Alice".to_string()))
                );
                assert_eq!(obj.get("age"), Some(&JsonValue::Number(30.0)))
            } else {
                panic!("Expected Object");
            }
        }
        #[test]
        fn test_parse_nested_object() {
            let value = parse_json(r#"{"outer": {"inner": 1}}"#).unwrap();
            if let JsonValue::Object(outer) = value {
                if let Some(JsonValue::Object(inner)) = outer.get("outer") {
                    assert_eq!(inner.get("inner"), Some(&JsonValue::Number(1.0)));
                } else {
                    panic!("Expected nested object");
                }
            } else {
                panic!("Expected object");
            }
        }
        #[test]
        fn test_parse_array_in_object() {
            let value = parse_json(r#"{"items": [1, 2, 3]}"#).unwrap();
            if let JsonValue::Object(obj) = value {
                if let Some(JsonValue::Array(arr)) = obj.get("items") {
                    assert_eq!(arr.len(), 3);
                } else {
                    panic!("Expected array");
                }
            } else {
                panic!("Expected object");
            }
        }
        #[test]
        fn test_parse_object_in_array() {
            let value = parse_json(r#"[{"a": 1}, {"b": 2}]"#).unwrap();
            if let JsonValue::Array(arr) = value {
                assert_eq!(arr.len(), 2);
            } else {
                panic!("Expected array");
            }
        }
        #[test]
        fn test_object_accessor() {
            let value = parse_json(r#"{"name": "test"}"#).unwrap();
            let obj = value.as_object().unwrap();
            assert_eq!(obj.len(), 1);
        }
        #[test]
        fn test_object_get() {
            let value = parse_json(r#"{"name": "Alice", "age": 30}"#).unwrap();
            assert_eq!(
                value.get("name"),
                Some(&JsonValue::String("Alice".to_string()))
            );
        }
    }
}
