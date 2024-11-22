mod mqtt_mind_map;

fn main() {
    // Initialize the logger for structured logging
    env_logger::init();

    // Read MQTT configuration from environment variables or use defaults
    let host = std::env::var("AWSIP2").unwrap_or_else(|_| "localhost".to_string());
    let port = std::env::var("AWSPORT")
        .unwrap_or_else(|_| "1884".to_string())
        .parse()
        .unwrap_or(1884);

    // Create the MQTT mind map instance
    let mind_map = mqtt_mind_map::MQTTMindMap::new(host, port, 1.0, "output".to_string());

    // Start the mind map generator
    mind_map.start();
}

