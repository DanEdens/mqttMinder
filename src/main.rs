use mqtt_mind_map::MQTTMindMap;
use std::env;
use std::fs;
use std::path::Path;

mod web_server;

fn clean_output_dir(dir: &str) {
    if let Ok(entries) = fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                // Skip latest symlinks
                if !name.contains("latest") {
                    let _ = fs::remove_file(path);
                }
            }
        }
        println!("Cleaned output directory, keeping only latest files.");
    }
}

#[tokio::main]
async fn main() {
    env_logger::init();

    let args: Vec<String> = env::args().collect();
    let command = args.get(1).map(|s| s.as_str());

    match command {
        Some("clean") => {
            clean_output_dir("output");
            return;
        }
        Some("web") => {
            let web_port = args.get(2)
                .and_then(|p| p.parse().ok())
                .unwrap_or(8080);
            
            // Create output directory if it doesn't exist
            let _ = fs::create_dir_all("output");
            
            // Start MQTT client in a background thread
            let host = env::var("AWSIP2").unwrap_or_else(|_| "localhost".to_string());
            let mqtt_port = env::var("AWSPORT").unwrap_or_else(|_| "3003".to_string()).parse().unwrap_or(3003);
            
            std::thread::spawn(move || {
                let mind_map = MQTTMindMap::new(host, mqtt_port, 1.0, "output".to_string());
                mind_map.start();
            });

            // Start web server
            web_server::run_server("output".to_string(), web_port).await;
            return;
        }
        Some(cmd) => {
            println!("Unknown command: {}", cmd);
            println!("Available commands:");
            println!("  clean - Remove all files except latest");
            println!("  web [port] - Start web dashboard (default port: 8080)");
            println!("  (no command) - Start MQTT mind map");
            return;
        }
        None => {
            // Create output directory if it doesn't exist
            let _ = fs::create_dir_all("output");
        }
    }

    let host = env::var("AWSIP2").unwrap_or_else(|_| "localhost".to_string());
    let port = env::var("AWSPORT").unwrap_or_else(|_| "3003".to_string()).parse().unwrap_or(3003);

    let mind_map = MQTTMindMap::new(host, port, 1.0, "output".to_string());
    mind_map.start();
}
