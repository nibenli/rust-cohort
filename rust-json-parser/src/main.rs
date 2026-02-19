use rust_json_parser::{JsonParser, JsonValue, Result, Tokenizer};

fn main() {
    // Example 1: Complex Object
    let json1 = r#"{"name": "Alice", "age": 30}"#;
    println!("=== Processing Example 1 (Complex Object) ===");
    if let Err(e) = process_json_example(json1) {
        eprintln!("Status: Unexpected Error - {e}");
    }

    println!("\n");

    // Example 2: Primitive String
    let json2 = r#""Hello, Rust!""#;
    println!("=== Processing Example 2 (Primitive String) ===");
    if let Err(e) = process_json_example(json2) {
        eprintln!("Status: Unexpected Error - {e}");
    }

    println!("\n");

    // Example 3: Nested Array and Object
    let json3 = r#"[1, 2, {"key": "value"}, null]"#;
    println!("=== Processing Example 3 (Nested Structure) ===");
    if let Err(e) = process_json_example(json3) {
        eprintln!("Status: Unexpected Error - {e}");
    }
}

fn process_json_example(input: &str) -> Result<()> {
    // Tokenize
    let tokens = Tokenizer::new(input).tokenize()?;
    println!("Input:  {input}");
    println!("Tokens: {tokens:?}");

    // Parse
    let value = JsonParser::new(input)?.parse()?;

    // Display Result using the Display trait
    println!("Formatted Output: {value}");

    // Provide specific feedback based on the high-level type
    match &value {
        JsonValue::Object(obj) => {
            println!("Result: Parsed an Object with {} keys", obj.len());
        }
        JsonValue::Array(arr) => {
            println!("Result: Parsed an Array with {} elements", arr.len());
        }
        JsonValue::String(s) => println!("Result: Parsed a String -> {s}"),
        JsonValue::Number(n) => println!("Result: Parsed a Number -> {n}"),
        JsonValue::Boolean(b) => println!("Result: Parsed a Boolean -> {b}"),
        JsonValue::Null => println!("Result: Parsed a Null value"),
    }

    // Debug for deep inspection
    println!("Debug Value:      {value:?}");
    Ok(())
}
