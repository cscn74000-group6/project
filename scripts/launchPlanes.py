import random
import os
import sys
import time

planeCount = 10
planeSpeed = 5
duration = 10 
nodes_file = "nodes_good.txt"

# Read nodes from file
try:
    with open(nodes_file, "r") as f:
        nodes = [line.strip() for line in f.readlines()]
except FileNotFoundError:
    print(f"Error: {nodes_file} not found.")
    sys.exit(1)

if len(nodes) < 2:
    sys.exit(2)

interval = duration / max(1, planeCount)  # Calculate launch interval

for i in range(planeCount):
    start = random.choice(nodes)
    end = random.choice([node for node in nodes if node != start])
    
    command = f"start cmd /c client.exe {i} {start} {end} {planeSpeed}"
    os.system(command)
    print(f"Launched plane {i}: {command}")
    
    if i < planeCount - 1:
        time.sleep(interval)  # Wait before launching the next plane