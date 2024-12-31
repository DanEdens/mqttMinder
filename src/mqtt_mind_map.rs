use rumqttc::{MqttOptions, Client, Event, Incoming};
use std::{collections::HashMap, fs, sync::Arc, sync::Mutex, thread, time::{Duration, Instant}};
use petgraph::graph::{Graph, NodeIndex};
use petgraph::visit::EdgeRef;
use std::env;
use log::{info, error};
use chrono::Local;
use palette::LinSrgb;
use std::process::Command;

#[derive(Clone)]
pub struct NodeData {
    label: String,
    level: usize,
    last_update: Instant,
    value: String,
}

pub struct MQTTMindMap {
    host: String,
    port: u16,
    update_interval: Duration,
    output_dir: String,
    topic_values: Arc<Mutex<HashMap<String, String>>>,
    last_update: Arc<Mutex<Instant>>,
}

impl MQTTMindMap {
    pub fn new(host: String, port: u16, update_interval: f64, output_dir: String) -> Self {
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
        mqtt_options.set_keep_alive(Duration::from_secs(5));

        let (client, mut event_loop) = Client::new(mqtt_options, 10);
        let topic_values = Arc::clone(&self.topic_values);
        let last_update = Arc::clone(&self.last_update);
        let update_interval = self.update_interval;
        let output_dir = self.output_dir.clone();

        thread::spawn(move || {
            loop {
                match event_loop.recv() {
                    Ok(notification) => {
                        if let Ok(Event::Incoming(Incoming::Publish(p))) = notification {
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
                    }
                    Err(e) => {
                        error!("Error in MQTT loop: {:?}", e);
                        break;
                    }
                }
            }
        });

        Ok(client)
    }

    fn update_mind_map(output_dir: &str, topic_values: &HashMap<String, String>) -> Result<(), std::io::Error> {
        let mut graph = Graph::<NodeData, String>::new();
        let mut nodes = HashMap::new();
        let now = Instant::now();

        // Create a color gradient for different levels
        let colors = vec![
            LinSrgb::new(0.2, 0.7, 1.0),  // Light blue
            LinSrgb::new(0.1, 0.5, 0.9),  // Medium blue
            LinSrgb::new(0.0, 0.3, 0.8),  // Dark blue
        ];

        for (topic, value) in topic_values.iter() {
            let parts: Vec<&str> = topic.split('/').collect();
            let mut parent_index: Option<NodeIndex> = None;

            for (i, &part) in parts.iter().enumerate() {
                let current_topic = parts[..=i].join("/");
                let node_data = NodeData {
                    label: part.to_string(),
                    level: i,
                    last_update: now,
                    value: if i == parts.len() - 1 {
                        value.clone()
                    } else {
                        String::new()
                    },
                };

                let node_index = *nodes.entry(current_topic.clone()).or_insert_with(|| {
                    graph.add_node(node_data)
                });

                if let Some(parent) = parent_index {
                    let edge_label = format!("level_{}", i);
                    graph.add_edge(parent, node_index, edge_label);
                }

                parent_index = Some(node_index);
            }
        }

        // Generate timestamp for file names
        let timestamp = Local::now().format("%Y%m%d_%H%M%S");

        // Create DOT file with enhanced styling
        let mut dot_content = String::from("digraph {\n");
        dot_content.push_str("    graph [rankdir=LR, splines=ortho];\n");
        dot_content.push_str("    node [style=filled, fontname=Arial];\n");

        // Add nodes with styling
        for node_idx in graph.node_indices() {
            let node = &graph[node_idx];
            let color_idx = (node.level as f32 / colors.len() as f32).min(1.0);
            let color_idx = (color_idx * (colors.len() - 1) as f32) as usize;
            let color = colors[color_idx];

            let color_str = format!("#{:02x}{:02x}{:02x}",
                (color.red * 255.0) as u8,
                (color.green * 255.0) as u8,
                (color.blue * 255.0) as u8
            );

            let label = if node.value.is_empty() {
                node.label.clone()
            } else {
                format!("{}\n{}", node.label, node.value)
            };

            dot_content.push_str(&format!(
                "    n{} [label=\"{}\", fillcolor=\"{}\", shape=box];\n",
                node_idx.index(), label, color_str
            ));
        }

        // Add edges with styling
        for edge in graph.edge_references() {
            dot_content.push_str(&format!(
                "    n{} -> n{} [color=\"#666666\"];\n",
                edge.source().index(),
                edge.target().index()
            ));
        }

        dot_content.push_str("}\n");

        // Write DOT file
        let dot_path = format!("{}/mqtt_mind_map_{}.dot", output_dir, timestamp);
        fs::write(&dot_path, &dot_content)?;

        // Generate SVG using dot
        let svg_path = format!("{}/mqtt_mind_map_{}.svg", output_dir, timestamp);
        let png_path = format!("{}/mqtt_mind_map_{}.png", output_dir, timestamp);

        // Generate SVG
        Command::new("dot")
            .args(&["-Tsvg", &dot_path, "-o", &svg_path])
            .output()
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;

        // Generate PNG
        Command::new("dot")
            .args(&["-Tpng", &dot_path, "-o", &png_path])
            .output()
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;

        // Create symlinks to latest versions
        let latest_svg = format!("{}/mqtt_mind_map_latest.svg", output_dir);
        let latest_png = format!("{}/mqtt_mind_map_latest.png", output_dir);

        // Remove existing symlinks if they exist
        let _ = fs::remove_file(&latest_svg);
        let _ = fs::remove_file(&latest_png);

        // Create new symlinks
        std::os::unix::fs::symlink(&svg_path, &latest_svg)?;
        std::os::unix::fs::symlink(&png_path, &latest_png)?;

        info!("Mind map updated at {}", svg_path);
        Ok(())
    }

    pub fn start(&self) {
        match self.connect() {
            Ok(mut client) => {
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
    let port = env::var("AWSPORT").unwrap_or_else(|_| "3003".to_string()).parse().unwrap_or(3003);

    let mind_map = MQTTMindMap::new(host, port, 1.0, "output".to_string());
    mind_map.start();
}
