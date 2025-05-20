use std::env;
use connectify_config::secrets;
use tracing::{info};
fn main() {
    let args: Vec<String> = env::args().collect();
    
    if args.len() < 2 {
        info!("Usage: encrypt_config <file_path>");
        info!("Encrypts sensitive values in the specified configuration file.");
        info!("Example: encrypt_config config/production.yml");
        return;
    }
    
    match secrets::encrypt_config_file(&args[1]) {
        Ok(_) => {
            info!("Successfully encrypted configuration file: {}", args[1]);
            info!("Note: The encryption key is stored in .connectify_key by default.");
            info!("For production, set the CONNECTIFY_ENCRYPTION_KEY environment variable.");
        }
        Err(err) => {
            info!("Error encrypting configuration file: {}", err);
            std::process::exit(1);
        }
    }
}