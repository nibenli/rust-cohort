use std::{iter::Peekable, str::Chars};

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

pub fn tokenize(input: &str) -> Vec<Token> {
    let mut tokens = Vec::new();
    let mut chars = input.chars().peekable();

    while let Some(&c) = chars.peek() {
        match c {
            // Skip whitespace
            c if c.is_whitespace() => {
                chars.next();
            }

            // Handle Single-character
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
            // Handle Strings
            '"' => {
                string(&mut tokens, &mut chars);
            }
            // Handle Numbers
            '-' | '0'..='9' => {
                number(&mut tokens, &mut chars);
            }
            // Handle keywords
            't' | 'f' | 'n' => {
                let mut word = String::new();
                while let Some(&next_c) = chars.peek() {
                    if next_c.is_alphabetic() {
                        word.push(next_c);
                        chars.next();
                    } else {
                        break;
                    }
                }

                match word.as_str() {
                    "true" => tokens.push(Token::Boolean(true)),
                    "false" => tokens.push(Token::Boolean(false)),
                    "null" => tokens.push(Token::Null),
                    // TODO: handle unknown keyword with explicit error handling
                    _ => println!("Unknown keyword: {word}"),
                }
            }
            // TODO: handle unknown characters with explicit error handling
            _ => {
                println!("Unknown character: '{c}'");
                chars.next();
            }
        }
    }

    tokens
}

fn string(tokens: &mut Vec<Token>, chars: &mut Peekable<Chars>) {
    chars.next(); // Consume opening quote
    let mut extracted_str = String::new();

    while let Some(&nc) = chars.peek() {
        if nc == '"' {
            break;
        }
        extracted_str.push(nc);
        chars.next();
    }
    chars.next(); // Consume closing quote
    tokens.push(Token::String(extracted_str));
}

fn number(tokens: &mut Vec<Token>, chars: &mut Peekable<Chars>) {
    let mut number = String::new();

    while let Some(&c) = chars.peek() {
        if c.is_ascii_digit() || c == '.' || c == '-' {
            number.push(c);
            chars.next();
        } else {
            break;
        }
    }

    // TODO: Handle a failed parse, don't silently ignore it
    if let Ok(num) = number.parse::<f64>() {
        tokens.push(Token::Number(num));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_empty_braces() {
        let tokens = tokenize("{}");
        assert_eq!(tokens.len(), 2);
        assert_eq!(tokens[0], Token::LeftBrace);
        assert_eq!(tokens[1], Token::RightBrace);
    }
    #[test]
    fn test_simple_string() {
        let tokens = tokenize(r#""hello""#);
        assert_eq!(tokens.len(), 1);
        assert_eq!(tokens[0], Token::String("hello".to_string()));
    }
    #[test]
    fn test_number() {
        let tokens = tokenize("42");
        assert_eq!(tokens.len(), 1);
        assert_eq!(tokens[0], Token::Number(42.0));
    }
    #[test]
    fn test_tokenize_string() {
        let tokens = tokenize(r#""hello world""#);
        assert_eq!(tokens.len(), 1);
        assert_eq!(tokens[0], Token::String("hello world".to_string()));
    }
    #[test]
    fn test_boolean_and_null() {
        let tokens = tokenize("true false null");
        assert_eq!(tokens.len(), 3);
        assert_eq!(tokens[0], Token::Boolean(true));
        assert_eq!(tokens[1], Token::Boolean(false));
        assert_eq!(tokens[2], Token::Null);
    }
    #[test]
    fn test_simple_object() {
        let tokens = tokenize(r#"{"name": "Alice"}"#);
        assert_eq!(tokens.len(), 5);
        assert_eq!(tokens[0], Token::LeftBrace);
        assert_eq!(tokens[1], Token::String("name".to_string()));
        assert_eq!(tokens[2], Token::Colon);
        assert_eq!(tokens[3], Token::String("Alice".to_string()));
        assert_eq!(tokens[4], Token::RightBrace);
    }
    #[test]
    fn test_multiple_values() {
        let tokens = tokenize(r#"{"age": 30, "active": true}"#);
        // Verify we have the right tokens
        assert!(tokens.contains(&Token::String("age".to_string())));
        assert!(tokens.contains(&Token::Number(30.0)));
        assert!(tokens.contains(&Token::Comma));
        assert!(tokens.contains(&Token::String("active".to_string())));
        assert!(tokens.contains(&Token::Boolean(true)));
    }
    #[test]
    fn test_array() {
        let tokens = tokenize("[1,2,3]");
        assert_eq!(tokens.len(), 7);

        assert_eq!(tokens[0], Token::LeftBracket);
        assert_eq!(tokens[1], Token::Number(1.0));
        assert_eq!(tokens[2], Token::Comma);
        assert_eq!(tokens[3], Token::Number(2.0));
        assert_eq!(tokens[4], Token::Comma);
        assert_eq!(tokens[5], Token::Number(3.0));
        assert_eq!(tokens[6], Token::RightBracket);
    }
    #[test]
    fn test_nested_object() {
        let tokens = tokenize(r#"{"user": {"name": "Alice", "active": true}}"#);

        assert_eq!(tokens.len(), 13);

        assert_eq!(tokens[0], Token::LeftBrace);
        assert_eq!(tokens[1], Token::String("user".to_string()));
        assert_eq!(tokens[2], Token::Colon);

        // Inner object
        assert_eq!(tokens[3], Token::LeftBrace);
        assert_eq!(tokens[4], Token::String("name".to_string()));
        assert_eq!(tokens[5], Token::Colon);
        assert_eq!(tokens[6], Token::String("Alice".to_string()));
        assert_eq!(tokens[7], Token::Comma);
        assert_eq!(tokens[8], Token::String("active".to_string()));
        assert_eq!(tokens[9], Token::Colon);
        assert_eq!(tokens[10], Token::Boolean(true));
        assert_eq!(tokens[11], Token::RightBrace);

        assert_eq!(tokens[12], Token::RightBrace);
    }
    #[test]
    fn test_empty_string() {
        let tokens = tokenize(r#""""#);
        assert_eq!(tokens.len(), 1);
        assert_eq!(tokens[0], Token::String("".to_string()));
    }
    #[test]
    fn test_zero_number() {
        let tokens = tokenize("0");
        assert_eq!(tokens.len(), 1);
        assert_eq!(tokens[0], Token::Number(0.0));
    }
    #[test]
    fn test_negative_number() {
        let tokens = tokenize("-5");
        assert_eq!(tokens.len(), 1);
        assert_eq!(tokens[0], Token::Number(-5.0));
    }
}
