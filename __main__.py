from graphviz import Digraph

# List of MQTT topics
topics = [
    "home/livingroom/temperature",
    "home/livingroom/humidity",
    "home/kitchen/temperature"
]

# Create a new directed graph
dot = Digraph()

# Add nodes and edges based on topics
for topic in topics:
    parts = topic.split('/')
    for i in range(1, len(parts)):
        parent = '/'.join(parts[:i])
        child = '/'.join(parts[:i+1])
        dot.edge(parent, child)

# Render the mind map
dot.render('mqtt_mind_map', format='png', cleanup=True)

