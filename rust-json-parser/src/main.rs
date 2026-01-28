use rust_json_parser::tokenizer::tokenize;

fn main() {
    let json = r#"{"name": "Alice", "age": 30}"#;
    let tokens = tokenize(json);
    println!("Input JSON: {json}");
    println!("Tokens:");
    for token in &tokens {
        println!("{:?}", token);
    }
}
