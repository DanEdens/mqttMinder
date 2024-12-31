use mqtt_mind_map::MQTTMindMap;
use std::env;

fn main() {
    env_logger::init();

    let host = env::var("AWSIP2").unwrap_or_else(|_| "localhost".to_string());
    let port = env::var("AWSPORT").unwrap_or_else(|_| "3003".to_string()).parse().unwrap_or(3003);

    let mind_map = MQTTMindMap::new(host, port, 1.0, "output".to_string());
    mind_map.start();
}
