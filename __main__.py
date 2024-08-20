from graphviz import Digraph

# List of MQTT topics with values
topics_with_values = [
    "home/livingroom/temperature 72",
    "home/livingroom/humidity 30",
    "home/kitchen/temperature 74"
]

# Create a new directed graph
dot = Digraph()

# Add nodes and edges based on topics with values
for item in topics_with_values:
    topic, value = item.rsplit(' ', 1)  # Split topic and value
    parts = topic.split('/')
    for i in range(1, len(parts)):
        parent = '/'.join(parts[:i])
        child = '/'.join(parts[:i+1])
        dot.edge(parent, child)
    
    # Append the value to the final subtopic node
    dot.node(topic, label=f"{parts[-1]}: {value}")

# Render the mind map
dot.render('mqtt_mind_map_with_values', format='png', cleanup=True)
