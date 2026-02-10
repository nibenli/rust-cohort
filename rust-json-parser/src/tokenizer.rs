use crate::{JsonError, Result};
use std::char::from_u32;

#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    LeftBrace,
    RightBrace,
    LeftBracket,
    RightBracket,
    Comma,
    Colon,
    String(String),
    Number(f64),
    Boolean(bool),
    Null,
}

pub struct Tokenizer {
    input: Vec<char>,
    position: usize,
}

impl Tokenizer {
    const UNICODE_HEX_LEN: usize = 4;
    pub fn new(input: &str) -> Self {
        Self {
            input: input.chars().collect(),
            position: 0,
        }
    }

    pub fn tokenize(&mut self) -> Result<Vec<Token>> {
        let mut tokens = Vec::new();

        while !self.is_at_end() {
            let start_pos = self.position;

            let c = match self.peek() {
                Some(character) => character,
                None => break, // Safety valve: exit if peek fails unexpectedly
            };

            match c {
                c if c.is_whitespace() => {
                    self.advance();
                }
                '{' => {
                    self.advance();
                    tokens.push(Token::LeftBrace);
                }
                '}' => {
                    self.advance();
                    tokens.push(Token::RightBrace);
                }
                '[' => {
                    self.advance();
                    tokens.push(Token::LeftBracket);
                }
                ']' => {
                    self.advance();
                    tokens.push(Token::RightBracket);
                }
                ',' => {
                    self.advance();
                    tokens.push(Token::Comma);
                }
                ':' => {
                    self.advance();
                    tokens.push(Token::Colon);
                }

                '"' => tokens.push(self.string(start_pos)?),
                '-' | '0'..='9' => tokens.push(self.number(start_pos)?),
                't' | 'f' | 'n' => tokens.push(self.keyword(start_pos)?),

                _ => {
                    return Err(JsonError::UnexpectedToken {
                        expected: "valid JSON value".to_string(),
                        found: c.to_string(),
                        position: start_pos,
                    });
                }
            }
        }
        Ok(tokens)
    }

    // --- Private Helper Methods ---
    fn advance(&mut self) -> Option<char> {
        let c = self.input.get(self.position).copied();
        if c.is_some() {
            self.position += 1;
        }
        c
    }

    fn peek(&self) -> Option<char> {
        self.input.get(self.position).copied()
    }

    fn is_at_end(&self) -> bool {
        self.position >= self.input.len()
    }

    // --- Specialized Token Parsers ---

    fn string(&mut self, start_pos: usize) -> Result<Token> {
        self.advance(); // Consume the opening quote
        let mut extracted = String::new();

        while let Some(c) = self.advance() {
            match c {
                '"' => return Ok(Token::String(extracted)),
                '\\' => {
                    let escaped_char = self.parse_escape()?;
                    extracted.push(escaped_char);
                }
                _ => extracted.push(c),
            }
        }

        // If we hit None before a closing quote
        Err(JsonError::UnexpectedEndOfInput {
            expected: "\"".to_string(),
            position: start_pos,
        })
    }

    fn number(&mut self, start_pos: usize) -> Result<Token> {
        let mut num_str = String::new();

        // Consume characters as long as they belong to a JSON number
        while let Some(c) = self.peek() {
            if c.is_ascii_digit() || c == '.' || c == '-' || c == 'e' || c == 'E' || c == '+' {
                num_str.push(self.advance().unwrap_or(c));
            } else {
                break;
            }
        }

        let val = num_str
            .parse::<f64>()
            .map_err(|_| JsonError::InvalidNumber {
                value: num_str,
                position: start_pos,
            })?;

        Ok(Token::Number(val))
    }

    fn keyword(&mut self, start_pos: usize) -> Result<Token> {
        let mut word = String::new();

        while let Some(c) = self.peek() {
            if c.is_alphabetic() {
                word.push(self.advance().unwrap_or(c));
            } else {
                break;
            }
        }

        match word.as_str() {
            "true" => Ok(Token::Boolean(true)),
            "false" => Ok(Token::Boolean(false)),
            "null" => Ok(Token::Null),
            _ => Err(JsonError::UnexpectedToken {
                expected: "keyword".to_string(),
                found: word,
                position: start_pos,
            }),
        }
    }

    fn parse_escape(&mut self) -> Result<char> {
        let slash_pos = self.position - 1; // Position of the '\'
        match self.advance() {
            Some('n') => Ok('\n'),
            Some('r') => Ok('\r'),
            Some('t') => Ok('\t'),
            Some('b') => Ok('\u{0008}'),
            Some('f') => Ok('\u{000C}'),
            Some('"') => Ok('"'),
            Some('\\') => Ok('\\'),
            Some('/') => Ok('/'),
            Some('u') => self.unicode_escape(slash_pos),
            Some(other) => {
                // Found a backslash followed by something invalid (like \q)
                Err(JsonError::InvalidEscape {
                    character: other,
                    position: slash_pos,
                })
            }
            None => Err(JsonError::UnexpectedEndOfInput {
                expected: "escape character".to_string(),
                position: self.position,
            }),
        }
    }

    fn unicode_escape(&mut self, start_pos: usize) -> Result<char> {
        let mut hex_string = String::with_capacity(Self::UNICODE_HEX_LEN);

        // Collect exactly 4 characters
        for _ in 0..Self::UNICODE_HEX_LEN {
            match self.advance() {
                Some(c) => hex_string.push(c),
                None => {
                    return Err(JsonError::InvalidUnicode {
                        sequence: hex_string,
                        position: start_pos,
                    });
                }
            }
        }

        // Validate length
        if hex_string.len() != Self::UNICODE_HEX_LEN {
            return Err(JsonError::InvalidUnicode {
                sequence: hex_string,
                position: start_pos,
            });
        }

        // Convert Hex to u32, then to char
        let code_point =
            u32::from_str_radix(&hex_string, 16).map_err(|_| JsonError::InvalidUnicode {
                sequence: hex_string.clone(),
                position: start_pos,
            })?;

        from_u32(code_point).ok_or(JsonError::InvalidUnicode {
            sequence: hex_string,
            position: start_pos,
        })
    }

    #[cfg(test)]
    fn position(&self) -> usize {
        self.position
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tokenize(input: &str) -> Result<Vec<Token>> {
        Tokenizer::new(input).tokenize()
    }

    mod tokenizer_creation {
        use super::*;
        #[test]
        fn test_tokenizer_struct_creation() {
            let mut tokenizer = Tokenizer::new(r#""hello""#);

            let tokens = tokenizer.tokenize().expect("Should parse successfully");

            assert_eq!(tokens.len(), 1);
            assert_eq!(tokens[0], Token::String("hello".to_string()));

            // Verify it is now at the end of the input
            assert!(tokenizer.is_at_end())
        }
        #[test]
        fn test_tokenizer_multiple_tokens() {
            let mut tokenizer = Tokenizer::new("123 456");
            // First pass: Consumes everything
            let first_pass = tokenizer.tokenize().unwrap();
            assert_eq!(first_pass.len(), 2, "Should have found two number tokens");
            assert!(tokenizer.is_at_end(), "Tokenizer should be exhausted");

            // Second pass: Since position is at the end, it should return an empty Vec
            let second_pass = tokenizer.tokenize().unwrap();
            assert!(
                second_pass.is_empty(),
                "Subsequent calls should return empty results"
            );
        }

        #[test]
        fn test_initial_position() {
            let tokenizer = Tokenizer::new("initial");
            assert_eq!(tokenizer.position(), 0);
        }
    }

    mod tokenizer_state_transitions {
        use super::*;

        #[test]
        fn test_advance_sequence() {
            let mut tokenizer = Tokenizer::new("abc");
            // Each advance moves forward
            assert_eq!(tokenizer.advance(), Some('a'));
            assert_eq!(tokenizer.advance(), Some('b'));
            assert_eq!(tokenizer.advance(), Some('c'));
            assert_eq!(tokenizer.advance(), None);
        }

        #[test]
        fn test_is_at_end_after_consuming_all() {
            let mut tokenizer = Tokenizer::new("x");
            assert!(!tokenizer.is_at_end());
            tokenizer.advance();
            assert!(tokenizer.is_at_end());
        }

        #[test]
        fn test_peek_doesnt_advance() {
            let mut tokenizer = Tokenizer::new("ab");
            // Multiple peeks should return the same thing
            assert_eq!(tokenizer.peek(), Some('a'));
            assert_eq!(tokenizer.peek(), Some('a'));
            assert_eq!(tokenizer.peek(), Some('a'));
            // Position unchanged - advance still gets 'a'
            assert_eq!(tokenizer.advance(), Some('a'));
        }

        #[test]
        fn test_is_at_end_multiple_calls() {
            let tokenizer = Tokenizer::new("test");
            // Should always return the same thing
            assert!(!tokenizer.is_at_end());
            assert!(!tokenizer.is_at_end());
            assert!(!tokenizer.is_at_end());
        }

        #[test]
        fn test_advance_order_matters() {
            let mut t1 = Tokenizer::new("12");
            let mut t2 = Tokenizer::new("12");
            // Same operations in same order
            assert_eq!(t1.advance(), t2.advance());
            assert_eq!(t1.advance(), t2.advance());
        }
    }

    // --- Basic Tokens Happy Path Tests ---
    mod basic_tokens_success_cases {
        use super::*;
        #[test]
        fn test_empty_braces() {
            let tokens = tokenize("{}");
            assert!(tokens.is_ok());
            assert_eq!(tokens.unwrap(), vec![Token::LeftBrace, Token::RightBrace]);
        }

        #[test]
        fn test_simple_string() {
            let tokens = tokenize(r#""hello""#);
            assert!(tokens.is_ok());
            assert_eq!(tokens.unwrap(), vec![Token::String("hello".to_string())]);
        }

        #[test]
        fn test_number() {
            let tokens = tokenize("42");
            assert!(tokens.is_ok());
            assert_eq!(tokens.unwrap(), vec![Token::Number(42.0)]);
        }

        #[test]
        fn test_literals() {
            let mut t1 = Tokenizer::new("true");
            assert_eq!(t1.tokenize().unwrap(), vec![Token::Boolean(true)]);

            let mut t2 = Tokenizer::new("false");
            assert_eq!(t2.tokenize().unwrap(), vec![Token::Boolean(false)]);

            let mut t3 = Tokenizer::new("null");
            assert_eq!(t3.tokenize().unwrap(), vec![Token::Null]);
        }

        #[test]
        fn test_simple_object() {
            let tokens = tokenize(r#"{"name": "Alice"}"#);
            assert!(tokens.is_ok());
            let expected = vec![
                Token::LeftBrace,
                Token::String("name".to_string()),
                Token::Colon,
                Token::String("Alice".to_string()),
                Token::RightBrace,
            ];
            assert_eq!(tokens.unwrap(), expected);
        }

        #[test]
        fn test_array() {
            let tokens = tokenize("[1, 2, 3]");
            assert!(tokens.is_ok());
            let unwrapped_tokens = tokens.unwrap();
            assert_eq!(unwrapped_tokens.len(), 7);
            assert_eq!(unwrapped_tokens[1], Token::Number(1.0));
            assert_eq!(unwrapped_tokens[3], Token::Number(2.0));
            assert_eq!(unwrapped_tokens[5], Token::Number(3.0));
        }

        #[test]
        fn test_nested_object() {
            let tokens = tokenize(r#"{"user": {"name": "Alice", "active": true}}"#);
            assert!(tokens.is_ok());
            let unwrapped_tokens = tokens.unwrap();
            assert_eq!(unwrapped_tokens.len(), 13);
            assert_eq!(unwrapped_tokens[0], Token::LeftBrace);
            assert_eq!(unwrapped_tokens[1], Token::String("user".to_string()));
            assert_eq!(unwrapped_tokens[2], Token::Colon);

            // Inner object
            assert_eq!(unwrapped_tokens[3], Token::LeftBrace);
            assert_eq!(unwrapped_tokens[4], Token::String("name".to_string()));
            assert_eq!(unwrapped_tokens[5], Token::Colon);
            assert_eq!(unwrapped_tokens[6], Token::String("Alice".to_string()));
            assert_eq!(unwrapped_tokens[7], Token::Comma);
            assert_eq!(unwrapped_tokens[8], Token::String("active".to_string()));
            assert_eq!(unwrapped_tokens[9], Token::Colon);
            assert_eq!(unwrapped_tokens[10], Token::Boolean(true));
            assert_eq!(unwrapped_tokens[11], Token::RightBrace);

            assert_eq!(unwrapped_tokens[12], Token::RightBrace);
        }

        #[test]
        fn test_complex_object() {
            let input = r#"{"id": 1, "vec": [true, null], "nested": {"ok": false}}"#;
            let tokens = tokenize(input);
            assert!(tokens.is_ok());
            let unwrapped_tokens = tokens.unwrap();
            assert_eq!(unwrapped_tokens[0], Token::LeftBrace);
            assert_eq!(unwrapped_tokens[4], Token::Comma);
            assert_eq!(unwrapped_tokens[5], Token::String("vec".to_string()));
            assert_eq!(unwrapped_tokens[11], Token::RightBracket);
        }

        #[test]
        fn test_scientific_notation() {
            let tokens = tokenize("1e10 -2.5E-2");
            assert!(tokens.is_ok());
            let unwrapped_tokens = tokens.unwrap();
            assert_eq!(unwrapped_tokens[0], Token::Number(1e10));
            assert_eq!(unwrapped_tokens[1], Token::Number(-0.025));
        }

        // edge cases
        #[test]
        fn test_whitespace_tolerance() {
            let input = "  { \n\t \"a\" : \r 1 }  ";
            let tokens = tokenize(input);
            assert!(tokens.is_ok());
            let unwrapped_tokens = tokens.unwrap();
            assert_eq!(unwrapped_tokens.len(), 5); // { "a" : 1 }
            assert_eq!(unwrapped_tokens[0], Token::LeftBrace);
            assert_eq!(unwrapped_tokens[1], Token::String("a".to_string()));
            assert_eq!(unwrapped_tokens[2], Token::Colon);
            assert_eq!(unwrapped_tokens[3], Token::Number(1.0));
            assert_eq!(unwrapped_tokens[4], Token::RightBrace);
        }

        #[test]
        fn test_empty_input() {
            let tokens = tokenize("   ");
            assert!(tokens.is_ok());
            assert!(tokens.unwrap().is_empty());
        }

        #[test]
        fn test_empty_string() {
            let tokens = tokenize(r#""""#);
            assert!(tokens.is_ok());
            let unwrapped_tokens = tokens.unwrap();
            assert_eq!(unwrapped_tokens.len(), 1);
            assert_eq!(unwrapped_tokens[0], Token::String("".to_string()));
        }
        #[test]
        fn test_zero_number() {
            let tokens = tokenize("0");
            assert!(tokens.is_ok());
            let unwrapped_tokens = tokens.unwrap();
            assert_eq!(unwrapped_tokens.len(), 1);
            assert_eq!(unwrapped_tokens[0], Token::Number(0.0));
        }
        #[test]
        fn test_negative_number() {
            let tokens = tokenize("-5");
            assert!(tokens.is_ok());
            let unwrapped_tokens = tokens.unwrap();
            assert_eq!(unwrapped_tokens.len(), 1);
            assert_eq!(unwrapped_tokens[0], Token::Number(-5.0));
        }

        #[test]
        fn test_deeply_nested_structure() {
            let input = "[[[[[]]]]]";
            let tokens = tokenize(input);
            assert!(tokens.is_ok());
            let unwrapped_tokens = tokens.unwrap();
            assert_eq!(unwrapped_tokens.len(), 10);
        }

        #[test]
        fn test_multibyte_whitespace() {
            let input = "{\u{3000}}";
            let tokens = tokenize(input);
            assert!(tokens.is_ok());
            let unwrapped_tokens = tokens.unwrap();

            assert_eq!(unwrapped_tokens, vec![Token::LeftBrace, Token::RightBrace]);
        }
    }
    // --- Basic Tokens Error Handling Tests ---
    mod basic_tokens_error_cases {
        use super::*;
        #[test]
        fn test_invalid_character_error() {
            let result = tokenize(".5");
            match result {
                Err(JsonError::UnexpectedToken {
                    found, position, ..
                }) => {
                    assert_eq!(found, ".");
                    assert_eq!(position, 0);
                }
                _ => panic!("Should have failed with UnexpectedToken error for '.5'"),
            }
        }

        #[test]
        fn test_error_invalid_number() {
            // "1.2.3" is not a valid f64
            let result = tokenize("1.2.3");
            match result {
                Err(JsonError::InvalidNumber { value, position }) => {
                    assert_eq!(value, "1.2.3");
                    assert_eq!(position, 0);
                }
                _ => panic!("Should have failed with InvalidNumber error for '1.2.3'"),
            }
        }

        #[test]
        fn test_unterminated_string_error() {
            let result = tokenize(r#""hello"#);
            match result {
                Err(JsonError::UnexpectedEndOfInput { position, .. }) => {
                    assert_eq!(position, 0);
                }
                _ => panic!("Should have failed with UnexpectedEndOfInput error"),
            }
        }

        #[test]
        fn test_unknown_keyword_error() {
            let result = tokenize("truthy");
            match result {
                Err(JsonError::UnexpectedToken {
                    found, expected, ..
                }) => {
                    assert_eq!(found, "truthy");
                    assert!(expected.contains("keyword"));
                }
                _ => panic!("Should have failed with UnexpectedToken error for 'truthy'"),
            }
        }

        #[test]
        fn test_error_illegal_character() {
            let result = tokenize("{ @ }");
            match result {
                Err(JsonError::UnexpectedToken {
                    found, position, ..
                }) => {
                    assert_eq!(found, "@");
                    assert_eq!(position, 2);
                }
                _ => panic!("Should have failed with UnexpectedToken at @"),
            }
        }
    }

    mod escape_sequences {
        use super::*;

        // === Success Tests ===
        #[test]
        fn test_escape_newline() {
            let mut tokenizer = Tokenizer::new(r#""hello\nworld""#);
            let tokens = tokenizer.tokenize().unwrap();
            assert_eq!(tokens, vec![Token::String("hello\nworld".to_string())]);
        }
        #[test]
        fn test_escape_tab() {
            let mut tokenizer = Tokenizer::new(r#""col1\tcol2""#);
            let tokens = tokenizer.tokenize().unwrap();
            assert_eq!(tokens, vec![Token::String("col1\tcol2".to_string())]);
        }
        #[test]
        fn test_escape_quote() {
            let mut tokenizer = Tokenizer::new(r#""say \"hello\"""#);
            let tokens = tokenizer.tokenize().unwrap();
            assert_eq!(tokens, vec![Token::String("say \"hello\"".to_string())]);
        }
        #[test]
        fn test_escape_backslash() {
            let mut tokenizer = Tokenizer::new(r#""path\\to\\file""#);
            let tokens = tokenizer.tokenize().unwrap();
            assert_eq!(tokens, vec![Token::String("path\\to\\file".to_string())]);
        }

        #[test]
        fn test_escape_forward_slash() {
            let mut tokenizer = Tokenizer::new(r#""a\/b""#);
            let tokens = tokenizer.tokenize().unwrap();
            assert_eq!(tokens, vec![Token::String("a/b".to_string())]);
        }
        #[test]
        fn test_escape_carriage_return() {
            let mut tokenizer = Tokenizer::new(r#""line\r\n""#);
            let tokens = tokenizer.tokenize().unwrap();
            assert_eq!(tokens, vec![Token::String("line\r\n".to_string())]);
        }
        #[test]
        fn test_escape_backspace_formfeed() {
            let mut tokenizer = Tokenizer::new(r#""\b\f""#);
            let tokens = tokenizer.tokenize().unwrap();
            assert_eq!(tokens, vec![Token::String("\u{0008}\u{000C}".to_string())]);
        }
        #[test]
        fn test_multiple_escapes() {
            let mut tokenizer = Tokenizer::new(r#""a\nb\tc\"""#);
            let tokens = tokenizer.tokenize().unwrap();
            assert_eq!(tokens, vec![Token::String("a\nb\tc\"".to_string())]);
        }

        // === Error Tests ===
        #[test]
        fn test_invalid_escape_sequence() {
            let mut tokenizer = Tokenizer::new(r#""\q""#);
            let result = tokenizer.tokenize();
            assert!(matches!(result, Err(JsonError::InvalidEscape { .. })));
        }

        #[test]
        fn test_unterminated_string_with_escape() {
            let mut tokenizer = Tokenizer::new(r#""hello\n"#);
            let result = tokenizer.tokenize();
            assert!(result.is_err());
        }
    }

    mod unicode_escapes {
        use super::*;

        #[test]
        fn test_unicode_escape_basic() {
            // \u0041 is 'A'
            let mut tokenizer = Tokenizer::new(r#""\u0041""#);
            let tokens = tokenizer.tokenize().unwrap();
            assert_eq!(tokens, vec![Token::String("A".to_string())]);
        }
        #[test]
        fn test_unicode_escape_multiple() {
            // \u0048\u0069 is "Hi"
            let mut tokenizer = Tokenizer::new(r#""\u0048\u0069""#);
            let tokens = tokenizer.tokenize().unwrap();
            assert_eq!(tokens, vec![Token::String("Hi".to_string())]);
        }
        #[test]
        fn test_unicode_escape_mixed() {
            // Mix of regular chars and unicode escapes
            let mut tokenizer = Tokenizer::new(r#""Hello \u0057orld""#);
            let tokens = tokenizer.tokenize().unwrap();
            assert_eq!(tokens, vec![Token::String("Hello World".to_string())]);
        }

        #[test]
        fn test_unicode_escape_lowercase() {
            // Lowercase hex digits should work too
            let mut tokenizer = Tokenizer::new(r#""\u004a""#);
            let tokens = tokenizer.tokenize().unwrap();
            assert_eq!(tokens, vec![Token::String("J".to_string())]);
        }

        // unicode escapes error cases
        #[test]
        fn test_invalid_unicode_too_short() {
            let mut tokenizer = Tokenizer::new(r#""\u004""#);
            let result = tokenizer.tokenize();
            assert!(matches!(result, Err(JsonError::InvalidUnicode { .. })));
        }
        #[test]
        fn test_invalid_unicode_bad_hex() {
            let mut tokenizer = Tokenizer::new(r#""\u00GG""#);
            let result = tokenizer.tokenize();
            assert!(matches!(result, Err(JsonError::InvalidUnicode { .. })));
        }
    }
}
