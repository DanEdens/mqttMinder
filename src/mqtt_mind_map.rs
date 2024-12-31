use rumqttc::{MqttOptions, Client, Event, EventLoop, Incoming};
use std::{collections::HashMap, fs, sync::Arc, sync::Mutex, thread, time::{Duration, Instant}};
use petgraph::graph::{Graph, NodeIndex};
use petgraph::dot::{Dot, Config};
use std::env;
use log::{info, warn, error};

struct MQTTMindMap {
    host: String,
    port: u16,
    update_interval: Duration,
    output_dir: String,
    topic_values: Arc<Mutex<HashMap<String, String>>>,
    last_update: Arc<Mutex<Instant>>,
}

impl MQTTMindMap {
    fn new(host: String, port: u16, update_interval: f64, output_dir: String) -> Self {
        let update_interval = Duration::from_secs_f64(update_interval);
        fs::create_dir_all(&output_dir).unwrap();

        MQTTMindMap {
            host,
            port,
            update_interval,
            output_dir,
            topic_values: Arc::new(Mutex::new(HashMap::new())),
            last_update: Arc::new(Mutex::new(Instant::now())),
        }
    }

    fn connect(&self) -> Result<Client, rumqttc::ClientError> {
        let mut mqtt_options = MqttOptions::new("mind_map_client", &self.host, self.port);
        mqtt_options.set_keep_alive(5);

        let (client, event_loop) = Client::new(mqtt_options, 10);
        let topic_values = Arc::clone(&self.topic_values);
        let last_update = Arc::clone(&self.last_update);
        let update_interval = self.update_interval;
        let output_dir = self.output_dir.clone();

        thread::spawn(move || {
            let mut event_loop = event_loop;

            loop {
                match event_loop.poll() {
                    Ok(Event::Incoming(Incoming::Publish(p))) => {
                        let topic = p.topic.clone();
                        let value = String::from_utf8_lossy(&p.payload).to_string();

                        let mut values = topic_values.lock().unwrap();
                        values.insert(topic, value);

                        let now = Instant::now();
                        let mut last = last_update.lock().unwrap();
                        if now.duration_since(*last) >= update_interval {
                            *last = now;
                            if let Err(err) = Self::update_mind_map(&output_dir, &values) {
                                error!("Failed to update mind map: {:?}", err);
                            }
                        }
                    }
                    Ok(_) => {}
                    Err(err) => {
                        error!("Error in MQTT loop: {:?}", err);
                        break;
                    }
                }
            }
        });

        Ok(client)
    }

    fn update_mind_map(output_dir: &str, topic_values: &HashMap<String, String>) -> Result<(), std::io::Error> {
        let mut graph = Graph::<String, ()>::new();
        let mut nodes = HashMap::new();

        for (topic, value) in topic_values.iter() {
            let parts: Vec<&str> = topic.split('/').collect();
            let mut parent_index: Option<NodeIndex> = None;

            for (i, &part) in parts.iter().enumerate() {
                let current_topic = parts[..=i].join("/");
                let node_index = *nodes.entry(current_topic.clone()).or_insert_with(|| {
                    graph.add_node(format!("{}: {}", part, topic_values.get(&current_topic).unwrap_or(&"".to_string())))
                });

                if let Some(parent) = parent_index {
                    graph.add_edge(parent, node_index, ());
                }

                parent_index = Some(node_index);
            }
        }

        let dot = Dot::with_config(&graph, &[Config::EdgeNoLabel, Config::NodeColor, Config::NodeShape]);
        let output_path = format!("{}/dynamic_mqtt_mind_map.svg", output_dir);
        fs::write(&output_path, format!("{:?}", dot))?;
        info!("Mind map updated at {}", output_path);

        Ok(())
    }


    fn start(&self) {
        match self.connect() {
            Ok(client) => {
                client.subscribe("#", rumqttc::QoS::AtLeastOnce).unwrap();
                loop {
                    thread::sleep(Duration::from_millis(100));
                }
            }
            Err(err) => error!("Failed to connect to MQTT broker: {:?}", err),
        }
    }
}

fn main() {
    env_logger::init();

    let host = env::var("AWSIP2").unwrap_or_else(|_| "localhost".to_string());
    let port = env::var("AWSPORT").unwrap_or_else(|_| "1884".to_string()).parse().unwrap_or(1884);

    let mind_map = MQTTMindMap::new(host, port, 1.0, "output".to_string());
    mind_map.start();
}
