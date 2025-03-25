import random
import os
import sys
import time

planeCount = 1
planeSpeed = 5
duration = 10 
nodes_file = "nodes_good.txt"

# Read nodes from file
try:
    with open(nodes_file, "r") as f:
        nodes = [line.strip() for line in f.readlines()]
except FileNotFoundError:
    sys.exit(1)


if len(nodes) < 2:
    sys.exit(2)

interval = duration / max(1, planeCount) 

for i in range(planeCount):
    start = random.choice(nodes)
    end = random.choice([node for node in nodes if node != start])
    
    # Split coordinates
    sx, sy, sz = start.strip('[]').split(',')
    ex, ey, ez = end.strip('[]').split(',')

    command = f"start cmd /c client.exe {i} {sx} {sy} {sz} {ex} {ey} {ez} {planeSpeed}"
    os.system(command)
    print(f"Launched plane {i}: {command}")
    

    time.sleep(interval) 