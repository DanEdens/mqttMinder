import paho.mqtt.client as mqtt
from graphviz import Digraph
from time import sleep
import os

# Initialize the directed graph
dot = Digraph()

# Dictionary to store the latest values for each topic
topic_values = {}

# Callback function for when a message is received
def on_message(client, userdata, message):
    topic = message.topic
    value = message.payload.decode('utf-8')
    topic_values[topic] = value  # Update the value for this topic
    
    # Update the graph
    update_mind_map(topic, value)
    
# Function to update the mind map
def update_mind_map(topic, value):
    parts = topic.split('/')
    
    # Add edges
    for i in range(1, len(parts)):
        parent = '/'.join(parts[:i])
        child = '/'.join(parts[:i+1])
        dot.edge(parent, child)
    
    # Update or add the final node with the value
    dot.node(topic, label=f"{parts[-1]}: {value}")
    
    # Render the updated mind map
    dot.render('dynamic_mqtt_mind_map', format='png', cleanup=True)

# MQTT setup
client = mqtt.Client(mqtt.CallbackAPIVersion.VERSION1)
client.on_message = on_message

# Connect to the broker and subscribe to relevant topics
client.connect(host=os.environ.get('AWSIP2', 'localhost'),
                    port=int(os.environ.get('AWSPORT', 1884))
                    )
client.subscribe("#")  # Subscribe to all topics under 'home/'

# Start the MQTT client
client.loop_start()

# Keep the script running to receive messages and update the mind map
try:
    while True:
        sleep(1)  # Wait for messages
except KeyboardInterrupt:
    client.loop_stop()
