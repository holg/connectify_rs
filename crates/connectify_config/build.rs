use std::{env, fs};
use std::path::Path;
use connectify_config_static::load_config;
use serde_json::Value;

fn main() {
    let config = load_config().expect("Failed to load config");
    let json = serde_json::to_value(config).expect("Failed to serialize config to Value");

    let mut output = String::new();
    flatten_json("", &json, &mut output);

    let out_dir = env::var("OUT_DIR").unwrap();
    let dest = Path::new(&out_dir).join("generated_constants.rs");
    eprintln!("build.rs: starting config load");

    let config = load_config().unwrap_or_else(|err| {
        panic!("Failed to load config: {err:?}");
    });

    eprintln!("build.rs: successfully loaded config");
    eprintln!("build.rs: successfully loaded config");
    eprintln!("{:#?}", config);

    let json = serde_json::to_string_pretty(&config).expect("Failed to serialize config");

    let dest = Path::new(&out_dir).join("generated_config.rs");

    output.push_str(
        &format!(r##"pub static DEFAULT_CONFIG_JSON: &str = r#"{json}"#; "##),
    );
    fs::write(dest, output).expect("Failed to write generated_constants.rs");

    println!("cargo:rerun-if-env-changed=RUN_ENV");
    println!("cargo:rerun-if-changed=config/");
}

fn flatten_json(prefix: &str, val: &Value, output: &mut String) {
    match val {
        Value::Object(map) => {
            for (key, value) in map {
                let new_prefix = if prefix.is_empty() {
                    key.to_uppercase()
                } else {
                    format!("{}_{}", prefix, key.to_uppercase())
                };
                flatten_json(&new_prefix, value, output);
            }
        }
        Value::Array(_) => {
            // Skipping arrays in constant output for simplicity
        }
        Value::String(s) => {
            output.push_str(&format!("pub const {}: &str = \"{}\";\n", prefix, s));
        }
        Value::Number(n) => {
            if let Some(i) = n.as_i64() {
                output.push_str(&format!("pub const {}: i64 = {};\n", prefix, i));
            } else if let Some(f) = n.as_f64() {
                output.push_str(&format!("pub const {}: f64 = {};\n", prefix, f));
            }
        }
        Value::Bool(b) => {
            output.push_str(&format!("pub const {}: bool = {};\n", prefix, b));
        }
        Value::Null => {
            // Skip null values
        }
    }
}
