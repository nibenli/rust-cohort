use rust_json_parser::{Result, parse_json, tokenize};

fn main() -> Result<()> {
    let json1 = r#"{"name": "Alice", "age": 30}"#;
    let tokens = tokenize(json1)?;
    println!("Input JSON: {json1}");
    println!("Tokens:");
    for token in &tokens {
        println!("{:?}", token);
    }
    println!();

    // Since the current implementation only handles the first token,
    // use a string for a successful parse.
    let json2 = r#""Hello, Rust!""#;

    // 1. Tokenize
    let tokens = tokenize(json2)?;
    println!("--- Tokenizing Stage ---");
    println!("Input JSON: {json2}");
    println!("Tokens: {:?}\n", tokens);

    // 2. Parse
    println!("--- Parsing Stage ---");
    let value = parse_json(json2)?;

    match &value {
        rust_json_parser::JsonValue::String(s) => println!("Parsed a String: {}", s),
        rust_json_parser::JsonValue::Number(n) => println!("Parsed a Number: {}", n),
        rust_json_parser::JsonValue::Boolean(b) => println!("Parsed a Boolean: {}", b),
        rust_json_parser::JsonValue::Null => println!("Parsed a Null value"),
    }

    println!("\nFinal JsonValue debug output:");
    println!("{:?}", value);
    Ok(())
}
