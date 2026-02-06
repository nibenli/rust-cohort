use crate::{JsonError, Result};
use std::{iter::Peekable, str::CharIndices};

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
    let mut chars = input.char_indices().peekable();

    while let Some(&(pos, c)) = chars.peek() {
        match c {
            c if c.is_whitespace() => {
                chars.next();
            }
            '{' => {
                chars.next();
                tokens.push(Token::LeftBrace);
            }
            '}' => {
                chars.next();
                tokens.push(Token::RightBrace);
            }
            '[' => {
                chars.next();
                tokens.push(Token::LeftBracket);
            }
            ']' => {
                chars.next();
                tokens.push(Token::RightBracket);
            }
            ',' => {
                chars.next();
                tokens.push(Token::Comma);
            }
            ':' => {
                chars.next();
                tokens.push(Token::Colon);
            }
            '"' => {
                tokens.push(string(&mut chars, pos)?);
            }
            '-' | '0'..='9' => {
                tokens.push(number(&mut chars, pos)?);
            }
            't' | 'f' | 'n' => {
                tokens.push(keyword(&mut chars, pos)?);
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

// Helper: Handle Strings
fn string(chars: &mut Peekable<CharIndices>, start_pos: usize) -> Result<Token> {
    chars.next(); // Consume "
    let mut extracted_str = String::new();

    while let Some(&(pos, nc)) = chars.peek() {
        match nc {
            '"' => {
                chars.next();
                return Ok(Token::String(extracted_str));
            }
            '\\' => {
                //TODO: Implement the logic to support escape sequences and unicode escapes
                // Placeholder for escape sequence support next week
                return Err(JsonError::UnexpectedToken {
                    expected: "plain string character".to_string(),
                    found: "unsupported escape sequence".to_string(),
                    position: pos,
                });
            }
            _ => {
                extracted_str.push(nc);
                chars.next();
            }
        }
    }
    Err(JsonError::UnexpectedEndOfInput {
        expected: "\"".to_string(),
        position: start_pos,
    })
}

// Helper: Handle Numbers
fn number(chars: &mut Peekable<CharIndices>, start_pos: usize) -> Result<Token> {
    let mut num_str = String::new();
    while let Some(&(_, c)) = chars.peek() {
        if c.is_ascii_digit() || c == '.' || c == '-' || c == 'e' || c == 'E' || c == '+' {
            num_str.push(c);
            chars.next();
        } else {
            break;
        }
    }
    num_str
        .parse::<f64>()
        .map(Token::Number)
        .map_err(|_| JsonError::InvalidNumber {
            value: num_str,
            position: start_pos,
        })
}

// Helper: Handle Keywords
fn keyword(chars: &mut Peekable<CharIndices>, start_pos: usize) -> Result<Token> {
    let mut word = String::new();
    while let Some(&(_, next_c)) = chars.peek() {
        if next_c.is_alphabetic() {
            word.push(next_c);
            chars.next();
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
