use connectify_config::secrets;
use serde_json::Value;
use std::env;
use std::fs;
use std::path::Path;
use tracing::info;
fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 3 {
        info!("Usage: migrate_config <source_file> <target_file>");
        info!("Migrates configuration from source file to target file format.");
        info!("Example: migrate_config config/old_config.yml config/new_config.yml");
        return;
    }

    let source_path = &args[1];
    let target_path = &args[2];

    match migrate_config(source_path, target_path) {
        Ok(_) => {
            info!(
                "Successfully migrated configuration from {} to {}",
                source_path, target_path
            );
        }
        Err(err) => {
            info!("Error migrating configuration: {}", err);
            std::process::exit(1);
        }
    }
}

/// Migrates configuration from source file to target file format
fn migrate_config(source_path: &str, target_path: &str) -> Result<(), Box<dyn std::error::Error>> {
    // Check if source file exists
    if !Path::new(source_path).exists() {
        return Err(format!("Source file does not exist: {}", source_path).into());
    }

    // Read source file
    let source_content = fs::read_to_string(source_path)?;

    // Parse source file based on extension
    let source_ext = Path::new(source_path)
        .extension()
        .and_then(|ext| ext.to_str())
        .unwrap_or("");
    let source_value: Value = match source_ext {
        "yml" | "yaml" => serde_yaml::from_str(&source_content)?,
        "json" => serde_json::from_str(&source_content)?,
        "toml" => {
            let toml_value = toml::from_str::<toml::Value>(&source_content)?;
            serde_json::to_value(toml_value)?
        }
        _ => return Err(format!("Unsupported source file format: {}", source_ext).into()),
    };

    // Check if target file exists
    let target_exists = Path::new(target_path).exists();

    // If target file exists, read it and merge with source
    let mut target_value = if target_exists {
        let target_content = fs::read_to_string(target_path)?;
        let target_ext = Path::new(target_path)
            .extension()
            .and_then(|ext| ext.to_str())
            .unwrap_or("");
        match target_ext {
            "yml" | "yaml" => serde_yaml::from_str(&target_content)?,
            "json" => serde_json::from_str(&target_content)?,
            "toml" => {
                let toml_value = toml::from_str::<toml::Value>(&target_content)?;
                serde_json::to_value(toml_value)?
            }
            _ => return Err(format!("Unsupported target file format: {}", target_ext).into()),
        }
    } else {
        // If target file doesn't exist, use an empty object
        serde_json::json!({})
    };

    // Merge source into target
    merge_json(&mut target_value, &source_value);

    // Encrypt sensitive values
    secrets::process_json_for_encryption(&mut target_value)?;

    // Write target file based on extension
    let target_ext = Path::new(target_path)
        .extension()
        .and_then(|ext| ext.to_str())
        .unwrap_or("");
    match target_ext {
        "yml" | "yaml" => {
            let yaml_content = serde_yaml::to_string(&target_value)?;
            fs::write(target_path, yaml_content)?;
        }
        "json" => {
            let json_content = serde_json::to_string_pretty(&target_value)?;
            fs::write(target_path, json_content)?;
        }
        "toml" => {
            // Convert JSON to TOML
            let json_str = serde_json::to_string(&target_value)?;
            let toml_value: toml::Value = serde_json::from_str(&json_str)?;
            let toml_content = toml::to_string(&toml_value)?;
            fs::write(target_path, toml_content)?;
        }
        _ => return Err(format!("Unsupported target file format: {}", target_ext).into()),
    }

    Ok(())
}

/// Recursively merges source JSON into target JSON
fn merge_json(target: &mut Value, source: &Value) {
    match (target, source) {
        (Value::Object(target_map), Value::Object(source_map)) => {
            for (key, value) in source_map {
                if !target_map.contains_key(key) {
                    // If key doesn't exist in target, add it
                    target_map.insert(key.clone(), value.clone());
                } else {
                    // If key exists in target, recursively merge
                    let target_value = target_map.get_mut(key).unwrap();
                    merge_json(target_value, value);
                }
            }
        }
        (target, source) => {
            // For non-object values, replace target with source
            *target = source.clone();
        }
    }
}
