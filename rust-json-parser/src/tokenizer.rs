use crate::{JsonError, Result};

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

pub fn tokenize(input: &str) -> Result<Vec<Token>> {
    let mut tokens = Vec::new();
    let mut pos = 0;

    while pos < input.len() {
        // Create a slice of the remaining string
        let remaining = &input[pos..];

        let mut chars = remaining.chars();
        let c = match chars.next() {
            Some(c) => c,
            None => break,
        };

        match c {
            c if c.is_whitespace() => {
                pos += c.len_utf8();
            }
            '{' => {
                tokens.push(Token::LeftBrace);
                pos += c.len_utf8();
            }
            '}' => {
                tokens.push(Token::RightBrace);
                pos += c.len_utf8();
            }
            '[' => {
                tokens.push(Token::LeftBracket);
                pos += c.len_utf8();
            }
            ']' => {
                tokens.push(Token::RightBracket);
                pos += c.len_utf8();
            }
            ',' => {
                tokens.push(Token::Comma);
                pos += c.len_utf8();
            }
            ':' => {
                tokens.push(Token::Colon);
                pos += c.len_utf8();
            }
            '"' => {
                // PASSING ONLY THE REMAINING SLICE
                let (token, consumed) = string(remaining, pos)?;
                tokens.push(token);
                pos += consumed;
            }
            '-' | '0'..='9' => {
                let (token, consumed) = number(remaining, pos)?;
                tokens.push(token);
                pos += consumed;
            }
            't' | 'f' | 'n' => {
                let (token, consumed) = keyword(remaining, pos)?;
                tokens.push(token);
                pos += consumed;
            }
            _ => {
                return Err(JsonError::UnexpectedToken {
                    expected: "valid JSON value".to_string(),
                    found: c.to_string(),
                    position: pos,
                });
            }
        }
    }
    Ok(tokens)
}

// Helper: Handle Strings, takes a slice, returns (Token, Bytes Consumed)
fn string(input: &str, start_pos: usize) -> Result<(Token, usize)> {
    let mut chars = input.char_indices();
    chars.next(); // Consume opening quote (")

    let mut extracted_str = String::new();

    for (local_pos, c) in chars {
        match c {
            '"' => {
                // Return the token and the total bytes used (local_pos + closing quote size)
                return Ok((Token::String(extracted_str), local_pos + 1));
            }
            '\\' => {
                return Err(JsonError::UnexpectedToken {
                    expected: "plain character".to_string(),
                    found: "unsupported escape sequence ('\\')".to_string(),
                    position: start_pos + local_pos,
                });
            }
            _ => {
                extracted_str.push(c);
            }
        }
    }

    Err(JsonError::UnexpectedEndOfInput {
        expected: "\"".to_string(),
        position: start_pos,
    })
}

// Helper: Handle Numbers
fn number(input: &str, global_pos: usize) -> Result<(Token, usize)> {
    let mut bytes_consumed = 0;
    let mut num_str = String::new();

    for c in input.chars() {
        if c.is_ascii_digit() || c == '.' || c == '-' || c == 'e' || c == 'E' || c == '+' {
            num_str.push(c);
            bytes_consumed += c.len_utf8();
        } else {
            break;
        }
    }

    let val = num_str
        .parse::<f64>()
        .map_err(|_| JsonError::InvalidNumber {
            value: num_str,
            position: global_pos,
        })?;

    Ok((Token::Number(val), bytes_consumed))
}

// Helper: Handle Keywords
fn keyword(input: &str, global_pos: usize) -> Result<(Token, usize)> {
    let mut bytes_consumed = 0;
    let mut word = String::new();

    for c in input.chars() {
        if c.is_alphabetic() {
            word.push(c);
            bytes_consumed += c.len_utf8();
        } else {
            break;
        }
    }

    let token = match word.as_str() {
        "true" => Token::Boolean(true),
        "false" => Token::Boolean(false),
        "null" => Token::Null,
        _ => {
            return Err(JsonError::UnexpectedToken {
                expected: "keyword".to_string(),
                found: word,
                position: global_pos,
            });
        }
    };

    Ok((token, bytes_consumed))
}

#[cfg(test)]
mod tests {
    use super::*;

    // --- Happy Path Tests ---
    mod success_cases {
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
        fn test_boolean_and_null() {
            let tokens = tokenize("true false null");
            assert!(tokens.is_ok());
            assert_eq!(
                tokens.unwrap(),
                vec![Token::Boolean(true), Token::Boolean(false), Token::Null]
            );
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
    // --- Error Handling Tests ---
    mod error_cases {
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

        #[test]
        fn test_unsupported_escape_error() {
            //TODO: don't support escape sequences for now, will implement it next week
            let result = tokenize(r#""hello\nworld""#);
            match result {
                Err(JsonError::UnexpectedToken { found, .. }) => {
                    // Verifying the placeholder message for now
                    assert!(found.contains("unsupported escape sequence"));
                }
                _ => panic!("Should have failed with UnexpectedToken error for backslash escape"),
            }
        }
    }
}
