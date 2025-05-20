use std::env;
use connectify_config::secrets;

fn main() {
    let args: Vec<String> = env::args().collect();
    
    if args.len() < 2 {
        println!("Usage: encrypt_config <file_path>");
        println!("Encrypts sensitive values in the specified configuration file.");
        println!("Example: encrypt_config config/production.yml");
        return;
    }
    
    match secrets::encrypt_config_file(&args[1]) {
        Ok(_) => {
            println!("Successfully encrypted configuration file: {}", args[1]);
            println!("Note: The encryption key is stored in .connectify_key by default.");
            println!("For production, set the CONNECTIFY_ENCRYPTION_KEY environment variable.");
        }
        Err(err) => {
            eprintln!("Error encrypting configuration file: {}", err);
            std::process::exit(1);
        }
    }
}