import paho.mqtt.client as mqtt
from graphviz import Digraph
from time import sleep, time
import os
import logging
from typing import Dict, Optional
from pathlib import Path

# Configure logging
logging.basicConfig(
    level=logging.INFO,
    format='%(asctime)s - %(levelname)s - %(message)s'
)
logger = logging.getLogger(__name__)

class MQTTMindMap:
    def __init__(self,
                 host: str = 'localhost',
                 port: int = 1884,
                 update_interval: float = 1.0,
                 output_dir: str = 'output'):
        """
        Initialize the MQTT Mind Map generator.
        
        Args:
            host: MQTT broker host
            port: MQTT broker port
            update_interval: Minimum time between graph updates in seconds
            output_dir: Directory to save the generated mind maps
        """
        self.host = host
        self.port = port
        self.update_interval = update_interval
        self.output_dir = Path(output_dir)
        self.output_dir.mkdir(exist_ok=True)

        # Initialize graph and state
        self.dot = Digraph(comment='MQTT Mind Map')
        self.dot.attr(rankdir='LR')  # Left to right layout
        self.topic_values: Dict[str, str] = {}
        self.last_update = 0
        self.running = False

        # Setup MQTT client
        self.client = mqtt.Client(mqtt.CallbackAPIVersion.VERSION1)
        self.client.on_message = self._on_message
        self.client.on_connect = self._on_connect
        self.client.on_disconnect = self._on_disconnect

    def _on_connect(self, client, userdata, flags, rc):
        """Callback for when the client connects to the broker."""
        if rc == 0:
            logger.info("Connected to MQTT broker")
            self.client.subscribe("#")
        else:
            logger.error(f"Failed to connect to MQTT broker with code: {rc}")

    def _on_disconnect(self, client, userdata, rc):
        """Callback for when the client disconnects from the broker."""
        logger.warning(f"Disconnected from MQTT broker with code: {rc}")

    def _on_message(self, client, userdata, message):
        """Handle incoming MQTT messages."""
        try:
            topic = message.topic
            value = message.payload.decode('utf-8')
            self.topic_values[topic] = value

            # Rate limit graph updates
            current_time = time()
            if current_time - self.last_update >= self.update_interval:
                self.update_mind_map()
                self.last_update = current_time

        except Exception as e:
            logger.error(f"Error processing message: {e}")

    def update_mind_map(self):
        """Update and render the mind map."""
        try:
            # Clear previous graph
            self.dot.clear()

            # Process all topics
            for topic, value in self.topic_values.items():
                parts = topic.split('/')

                # Add edges
                for i in range(1, len(parts)):
                    parent = '/'.join(parts[:i])
                    child = '/'.join(parts[:i+1])
                    self.dot.edge(parent, child)

                # Add value node
                self.dot.node(topic, label=f"{parts[-1]}: {value}")

            # Render the updated mind map
            output_path = self.output_dir / 'dynamic_mqtt_mind_map'
            self.dot.render(str(output_path), format='png', cleanup=True)

        except Exception as e:
            logger.error(f"Error updating mind map: {e}")

    def start(self):
        """Start the MQTT mind map generator."""
        try:
            self.running = True
            self.client.connect(self.host, self.port)
            self.client.loop_start()

            while self.running:
                sleep(0.1)

        except KeyboardInterrupt:
            logger.info("Shutting down...")
        except Exception as e:
            logger.error(f"Error in main loop: {e}")
        finally:
            self.stop()

    def stop(self):
        """Stop the MQTT mind map generator."""
        self.running = False
        self.client.loop_stop()
        self.client.disconnect()
        logger.info("Shutdown complete")

def main():
    """Main entry point for the application."""
    host = os.environ.get('AWSIP2', 'localhost')
    port = int(os.environ.get('AWSPORT', 1884))

    mind_map = MQTTMindMap(
        host=host,
        port=port,
        update_interval=1.0
    )
    mind_map.start()

if __name__ == "__main__":
    main()
