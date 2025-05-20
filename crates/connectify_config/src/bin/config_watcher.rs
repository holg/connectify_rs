use std::env;
use std::path::{Path, PathBuf};
use std::process::{Command, Child};
use std::time::Duration;
use notify::{Watcher, RecursiveMode, watcher};
use std::sync::mpsc::channel;

fn main() {
    let args: Vec<String> = env::args().collect();
    
    if args.len() < 2 {
        println!("Usage: config_watcher <command> [args...]");
        println!("Watches for changes in configuration files and restarts the command when they change.");
        println!("Example: config_watcher cargo run --bin connectify-backend");
        return;
    }
    
    // Get the command and arguments
    let command = &args[1];
    let command_args: Vec<&str> = args.iter().skip(2).map(|s| s.as_str()).collect();
    
    // Get the config directory
    let config_dir = env::var("CONFIG_DIR").unwrap_or_else(|_| "config".to_string());
    let config_path = PathBuf::from(&config_dir);
    
    println!("Watching for changes in {}", config_path.display());
    println!("Running command: {} {}", command, command_args.join(" "));
    
    // Create a channel to receive events
    let (tx, rx) = channel();
    
    // Create a watcher
    let mut watcher = watcher(tx, Duration::from_secs(1)).unwrap();
    
    // Watch the config directory
    watcher.watch(&config_path, RecursiveMode::Recursive).unwrap();
    
    // Start the command
    let mut child = start_command(command, &command_args);
    
    // Wait for events
    loop {
        match rx.recv() {
            Ok(event) => {
                println!("Config change detected: {:?}", event);
                
                // Kill the current process
                if let Err(err) = child.kill() {
                    println!("Failed to kill process: {}", err);
                }
                
                // Wait for the process to exit
                if let Err(err) = child.wait() {
                    println!("Failed to wait for process: {}", err);
                }
                
                // Restart the command
                println!("Restarting command: {} {}", command, command_args.join(" "));
                child = start_command(command, &command_args);
            }
            Err(err) => {
                println!("Watch error: {}", err);
                break;
            }
        }
    }
}

/// Starts a command with the given arguments
fn start_command(command: &str, args: &[&str]) -> Child {
    Command::new(command)
        .args(args)
        .spawn()
        .expect("Failed to start command")
}