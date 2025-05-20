use base64::{engine::general_purpose, Engine as _};
use ring::aead::{self, BoundKey, Nonce, NonceSequence, SealingKey, UnboundKey};
use ring::error::Unspecified;
use ring::rand::{SecureRandom, SystemRandom};
use serde_json::Value;
use std::env;
use std::fmt;
use std::fs;
use std::path::Path;
use tracing::info;

/// Error type for secret management operations
#[derive(Debug)]
pub enum SecretError {
    /// Error encrypting a secret
    EncryptionError(String),
    /// Error decrypting a secret
    DecryptionError(String),
    /// Error with the encryption key
    KeyError(String),
    /// I/O error
    IoError(std::io::Error),
    /// JSON error
    JsonError(serde_json::Error),
    /// Base64 error
    Base64Error(base64::DecodeError),
    /// Ring crypto error
    CryptoError,
}

impl fmt::Display for SecretError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SecretError::EncryptionError(msg) => write!(f, "Encryption error: {}", msg),
            SecretError::DecryptionError(msg) => write!(f, "Decryption error: {}", msg),
            SecretError::KeyError(msg) => write!(f, "Key error: {}", msg),
            SecretError::IoError(err) => write!(f, "I/O error: {}", err),
            SecretError::JsonError(err) => write!(f, "JSON error: {}", err),
            SecretError::Base64Error(err) => write!(f, "Base64 error: {}", err),
            SecretError::CryptoError => write!(f, "Cryptographic operation failed"),
        }
    }
}

impl std::error::Error for SecretError {}

impl From<std::io::Error> for SecretError {
    fn from(err: std::io::Error) -> Self {
        SecretError::IoError(err)
    }
}

impl From<serde_json::Error> for SecretError {
    fn from(err: serde_json::Error) -> Self {
        SecretError::JsonError(err)
    }
}

impl From<base64::DecodeError> for SecretError {
    fn from(err: base64::DecodeError) -> Self {
        SecretError::Base64Error(err)
    }
}

impl From<Unspecified> for SecretError {
    fn from(_: Unspecified) -> Self {
        SecretError::CryptoError
    }
}

/// A simple nonce sequence for AES-GCM
struct CounterNonceSequence {
    counter: u64,
}

impl CounterNonceSequence {
    fn new() -> Self {
        Self { counter: 0 }
    }
}

impl NonceSequence for CounterNonceSequence {
    fn advance(&mut self) -> Result<Nonce, Unspecified> {
        let mut nonce_bytes = [0u8; 12]; // 96 bits
        let counter = self.counter;
        self.counter += 1;

        // Use the counter as the last 8 bytes of the nonce
        nonce_bytes[4..].copy_from_slice(&counter.to_be_bytes());

        Nonce::try_assume_unique_for_key(&nonce_bytes)
    }
}

/// Get the encryption key from the environment or generate a new one
fn get_encryption_key() -> Result<[u8; 32], SecretError> {
    // Try to get the key from the environment variable
    if let Ok(key_b64) = env::var("CONNECTIFY_ENCRYPTION_KEY") {
        let key_bytes = general_purpose::STANDARD.decode(key_b64)?;
        if key_bytes.len() != 32 {
            return Err(SecretError::KeyError(format!(
                "Encryption key must be 32 bytes, got {} bytes",
                key_bytes.len()
            )));
        }

        let mut key = [0u8; 32];
        key.copy_from_slice(&key_bytes);
        return Ok(key);
    }

    // If not found in environment, check for a key file
    let key_path = env::var("CONNECTIFY_ENCRYPTION_KEY_PATH")
        .unwrap_or_else(|_| ".connectify_key".to_string());

    if Path::new(&key_path).exists() {
        let key_b64 = fs::read_to_string(&key_path)?;
        let key_bytes = general_purpose::STANDARD.decode(key_b64.trim())?;
        if key_bytes.len() != 32 {
            return Err(SecretError::KeyError(format!(
                "Encryption key must be 32 bytes, got {} bytes",
                key_bytes.len()
            )));
        }

        let mut key = [0u8; 32];
        key.copy_from_slice(&key_bytes);
        return Ok(key);
    }

    // Generate a new key
    let rng = SystemRandom::new();
    let mut key = [0u8; 32];
    rng.fill(&mut key)
        .map_err(|_| SecretError::KeyError("Failed to generate encryption key".to_string()))?;

    // Save the key to a file
    let key_b64 = general_purpose::STANDARD.encode(key);
    fs::write(&key_path, &key_b64)?;

    info!("Generated new encryption key and saved to {}", key_path);
    info!("For production, set the CONNECTIFY_ENCRYPTION_KEY environment variable to:");
    info!("{}", key_b64);

    Ok(key)
}

/// Encrypt a string using AES-GCM
pub fn encrypt_string(plaintext: &str) -> Result<String, SecretError> {
    let key = get_encryption_key()?;
    let unbound_key = UnboundKey::new(&aead::AES_256_GCM, &key)
        .map_err(|_| SecretError::EncryptionError("Failed to create encryption key".to_string()))?;

    let mut sealing_key = SealingKey::new(unbound_key, CounterNonceSequence::new());

    let mut in_out = plaintext.as_bytes().to_vec();
    let tag_len = aead::AES_256_GCM.tag_len();

    // Reserve space for the authentication tag
    in_out.extend(std::iter::repeat_n(0, tag_len));

    sealing_key
        .seal_in_place_append_tag(aead::Aad::empty(), &mut in_out)
        .map_err(|_| SecretError::EncryptionError("Failed to encrypt data".to_string()))?;

    // Encode the result as base64
    Ok(general_purpose::STANDARD.encode(&in_out))
}

/// Decrypt a string using AES-GCM
pub fn decrypt_string(ciphertext_b64: &str) -> Result<String, SecretError> {
    let key = get_encryption_key()?;
    let unbound_key = UnboundKey::new(&aead::AES_256_GCM, &key)
        .map_err(|_| SecretError::DecryptionError("Failed to create decryption key".to_string()))?;

    // Decode the base64 ciphertext
    let mut ciphertext = general_purpose::STANDARD.decode(ciphertext_b64)?;

    // Create an opening key with a counter nonce sequence
    let mut opening_key = aead::OpeningKey::new(unbound_key, CounterNonceSequence::new());

    // Decrypt the data
    let plaintext = opening_key
        .open_in_place(aead::Aad::empty(), &mut ciphertext)
        .map_err(|_| SecretError::DecryptionError("Failed to decrypt data".to_string()))?;

    // Convert the plaintext bytes to a string
    String::from_utf8(plaintext.to_vec()).map_err(|_| {
        SecretError::DecryptionError("Failed to convert decrypted data to string".to_string())
    })
}

/// Marker for encrypted values in configuration files
pub const ENCRYPTED_MARKER: &str = "encrypted:";

/// Check if a string is an encrypted value
pub fn is_encrypted(value: &str) -> bool {
    value.starts_with(ENCRYPTED_MARKER)
}

/// Encrypt a value if it's not already encrypted
pub fn ensure_encrypted(value: &str) -> Result<String, SecretError> {
    if is_encrypted(value) {
        Ok(value.to_string())
    } else {
        let encrypted = encrypt_string(value)?;
        Ok(format!("{}{}", ENCRYPTED_MARKER, encrypted))
    }
}

/// Decrypt a value if it's encrypted
pub fn ensure_decrypted(value: &str) -> Result<String, SecretError> {
    if is_encrypted(value) {
        let encrypted_part = &value[ENCRYPTED_MARKER.len()..];
        decrypt_string(encrypted_part)
    } else {
        Ok(value.to_string())
    }
}

/// Process a JSON value, encrypting all strings that match the secret pattern
pub fn process_json_for_encryption(value: &mut Value) -> Result<(), SecretError> {
    match value {
        Value::Object(map) => {
            for (_, v) in map {
                process_json_for_encryption(v)?;
            }
        }
        Value::Array(arr) => {
            for v in arr {
                process_json_for_encryption(v)?;
            }
        }
        Value::String(s) => {
            // Don't encrypt values that are already encrypted
            if !is_encrypted(s) && should_encrypt(s) {
                *s = ensure_encrypted(s)?;
            }
        }
        _ => {}
    }
    Ok(())
}

/// Process a JSON value, decrypting all encrypted strings
pub fn process_json_for_decryption(value: &mut Value) -> Result<(), SecretError> {
    match value {
        Value::Object(map) => {
            for (_, v) in map {
                process_json_for_decryption(v)?;
            }
        }
        Value::Array(arr) => {
            for v in arr {
                process_json_for_decryption(v)?;
            }
        }
        Value::String(s) => {
            if is_encrypted(s) {
                *s = ensure_decrypted(s)?;
            }
        }
        _ => {}
    }
    Ok(())
}

/// Determine if a string should be encrypted
/// This function can be customized to match your security requirements
fn should_encrypt(value: &str) -> bool {
    // Don't encrypt empty strings or the special marker for environment variables
    if value.is_empty() || value == "secret_from_env" {
        return false;
    }

    // Encrypt strings that look like secrets
    // This is a simple heuristic and can be improved
    value.contains("secret")
        || value.contains("key")
        || value.contains("password")
        || value.contains("token")
        || value.contains("sid")
}

/// Encrypt a configuration file
pub fn encrypt_config_file(file_path: &str) -> Result<(), SecretError> {
    // Read the file
    let content = fs::read_to_string(file_path)?;

    // Parse the YAML content to JSON
    let yaml_value: Value = serde_yaml::from_str(&content).map_err(|e| {
        SecretError::JsonError(serde_json::Error::io(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            format!("Failed to parse YAML: {}", e),
        )))
    })?;

    // Convert to JSON for processing
    let mut json_value = serde_json::to_value(yaml_value)?;

    // Process the JSON value, encrypting all strings that match the secret pattern
    process_json_for_encryption(&mut json_value)?;

    // Convert back to YAML
    let yaml_content = serde_yaml::to_string(&json_value).map_err(|e| {
        SecretError::JsonError(serde_json::Error::io(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            format!("Failed to convert to YAML: {}", e),
        )))
    })?;

    // Write the encrypted content back to the file
    fs::write(file_path, yaml_content)?;

    Ok(())
}

/// Command-line tool for encrypting configuration files
pub fn encrypt_config_command(args: &[String]) -> Result<(), SecretError> {
    if args.len() < 2 {
        info!("Usage: encrypt_config <file_path>");
        return Ok(());
    }

    let file_path = &args[1];
    encrypt_config_file(file_path)?;

    info!("Successfully encrypted configuration file: {}", file_path);
    Ok(())
}
