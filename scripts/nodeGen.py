import random
import math

gridSize = 40
airports = 5  # Number of airports within the map
connections = 5  # Number of border connections

margin = gridSize // 12  # Minimum distance from edges and between airports and connections

# Ensure the margin is at least 1 to avoid zero-margin issues
margin = max(1, margin)

def generate_airports():
    airport_coords = []
    while len(airport_coords) < airports:
        x = random.randint(margin, gridSize - margin - 1)
        y = random.randint(margin, gridSize - margin - 1)
        if all(math.dist((x, y), (ax, ay)) >= margin for ax, ay in airport_coords):
            airport_coords.append((x, y))
    return airport_coords

def generate_connections():
    connection_coords = []
    while len(connection_coords) < connections:
        edge = random.choice(['top', 'bottom', 'left', 'right'])
        if edge == 'top':
            x, y = random.randint(0, gridSize - 1), 0
        elif edge == 'bottom':
            x, y = random.randint(0, gridSize - 1), gridSize - 1
        elif edge == 'left':
            x, y = 0, random.randint(0, gridSize - 1)
        else:  # right
            x, y = gridSize - 1, random.randint(0, gridSize - 1)
        
        if all(math.dist((x, y), (ax, ay)) >= margin for ax, ay in connection_coords + airports_list):
            connection_coords.append((x, y))
    return connection_coords

# Generate airport and connection coordinates
airports_list = generate_airports()
connections_list = generate_connections()

# Combine and format data
all_nodes = airports_list + connections_list
node_data = [f"[{x},{y},0]" for x, y in all_nodes]

# Write to file
with open("nodes.txt", "w") as f:
    f.write("\n".join(node_data))

# Print the map
for y in range(gridSize):
    row = ""
    for x in range(gridSize):
        if (x, y) in airports_list or (x, y) in connections_list:
            row += "X "  # Airports and connections marked with X
        else:
            row += ". "  # Empty spaces marked with .
    print(row)
