use std::{env, fs};
use std::path::Path;
use connectify_config_static::load_config;
use serde_json::Value;
use tracing::{info};

fn main() {
    let out_dir = env::var("OUT_DIR").expect("Failed to get OUT_DIR environment variable");
    info!("build.rs: starting config load");

    // Load the configuration with improved error handling
    let config = load_config().unwrap_or_else(|err| {
        info!("Error loading configuration:");
        info!("  {}", err);

        // Provide more context based on the error
        let error_message = format!("{:?}", err);
        if error_message.contains("NotFound") {
            info!("  Configuration file not found");
            info!("  Make sure the file exists and is accessible.");
        } else if error_message.contains("PathParse") {
            info!("  Invalid configuration path");
            info!("  Check the path format in your configuration.");
        } else if error_message.contains("FileParse") {
            info!("  Failed to parse configuration file");
            info!("  Check the syntax of your configuration file.");
        } else {
            info!("  Check your configuration files and environment variables.");
        }

        panic!("Failed to load configuration. See error details above.");
    });

    info!("build.rs: successfully loaded config");

    // Convert the configuration to JSON
    let json = serde_json::to_value(&config).unwrap_or_else(|err| {
        info!("Failed to serialize configuration to JSON: {}", err);
        panic!("Failed to serialize configuration to JSON");
    });

    // Generate constants from the configuration
    let mut output = String::new();
    flatten_json("", &json, &mut output);

    // Convert the configuration to a pretty-printed JSON string
    let json = serde_json::to_string_pretty(&config).unwrap_or_else(|err| {
        info!("Failed to serialize configuration to pretty JSON: {}", err);
        panic!("Failed to serialize configuration to pretty JSON");
    });

    // Add the JSON string as a static constant
    output.push_str(
        &format!(r##"pub static DEFAULT_CONFIG_JSON: &str = r#"{json}"#; "##),
    );

    // Write the generated code to a file
    let dest = Path::new(&out_dir).join("generated_config.rs");
    fs::write(&dest, output).unwrap_or_else(|err| {
        info!("Failed to write generated configuration file: {}", err);
        info!("  Destination: {}", dest.display());
        panic!("Failed to write generated configuration file");
    });

    // Tell Cargo to rerun this build script if the environment or config files change
    info!("cargo:rerun-if-env-changed=RUN_ENV");
    info!("cargo:rerun-if-changed=config/");
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
