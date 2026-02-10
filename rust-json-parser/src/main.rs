use rust_json_parser::{JsonParser, JsonValue, Result, Tokenizer};

fn main() {
    // Example 1: A JSON Object (Expected to fail with current implementation)
    let json1 = r#"{"name": "Alice", "age": 30}"#;
    println!("=== Processing Example 1 (Complex Object) ===");
    if let Err(e) = process_json_example(json1) {
        eprintln!("Status: Expected Error - {e}");
    }

    println!("\n");

    // Example 2: A JSON String (Expected to succeed)
    let json2 = r#""Hello, Rust!""#;
    println!("=== Processing Example 2 (Primitive String) ===");
    if let Err(e) = process_json_example(json2) {
        eprintln!("Status: Unexpected Error - {e}");
    }
}

fn process_json_example(input: &str) -> Result<()> {
    // Tokenize
    let tokens = Tokenizer::new(input).tokenize()?;
    println!("Input: {input}");
    println!("Tokens: {tokens:?}");

    // Parse
    let value = JsonParser::new(input)?.parse()?;

    // Display Result
    match &value {
        JsonValue::String(s) => println!("Result: Parsed a String -> {s}"),
        JsonValue::Number(n) => println!("Result: Parsed a Number -> {n}"),
        JsonValue::Boolean(b) => println!("Result: Parsed a Boolean -> {b}"),
        JsonValue::Null => println!("Result: Parsed a Null value"),
    }

    println!("Debug Value: {value:?}");
    Ok(())
}
