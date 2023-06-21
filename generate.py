# Generates a random planar graph
#
# Usage: python3 ./generate.py NUMBER_NODES NODE_PROBABILITY EDGE_PROBABILITY
#   NUMBER_NODES: number of nodes in the graph
#   NODE_PROBABILITY: probability that a node gets edges associated
#   EDGE_PROBABILITY: probability that a edge is created

import json
import matplotlib.pyplot as plt
import networkx as nx
import random
import sys

def create_random_planar_graph(n, node_prob, edge_prob):
    G = nx.Graph()
    G.add_nodes_from(range(n))
    for i in random.sample(range(n), k=int(n * node_prob)):
        for j in random.sample(range(n), k=int(n * edge_prob)):
            if i == j:
                continue

            G.add_edge(i, j)
            if not nx.is_planar(G):
                G.remove_edge(i, j)
    return G


if len(sys.argv) < 5:
    print("Usage: ", sys.argv[0], " NUMBER_NODES NODE_PROBABILITY EDGE_PROBABILITY OUTFILE")
    exit(1)

number_nodes = int(sys.argv[1])
node_prob = float(sys.argv[2])
edge_prob = float(sys.argv[3])
outfile = sys.argv[4]

G = create_random_planar_graph(number_nodes, node_prob, edge_prob)

is_planar, embedding = nx.check_planarity(G)

# sanity check
assert is_planar

data = embedding.get_data()

# We assume that dividing by two always gives the correct amount of undirected
# edges because we add all edges that do not violate planarity in our
# generation algorithm. Adding a back-edge to another edge does not violate
# planarity because if it would, the already existing edge would've been
# incorrect. Thus, we add all back-edges and have 2m directed edges and m
# undirected edges.
# That also means dividing by two always produces an integer.

# Sanity check for 2m edges.
assert len(embedding.edges()) % 2 == 0

with open(outfile, "w", encoding="utf-8") as f:
    f.write(f"{number_nodes}\n{int(len(embedding.edges()) / 2)}\n")
    for node, dest_nodes in embedding.get_data().items():
        for dest in dest_nodes:
            f.write(f"{node} {dest}\n")

print(f"Edges: {int(len(embedding.edges()) / 2)}")
print(f"Output in '{outfile}'.")

layout = nx.planar_layout(embedding)

layout_json = []

for vertex, position in layout.items():
    x = position[0]
    y = position[1]
    layout_json.append({
        'id': vertex,
        'x': x,
        'y': y,
    })

with open('./data/layout.js', 'w') as f:
    f.write('let layout = ')
    json.dump(layout_json, f)

nx.draw_networkx(embedding, pos=layout, with_labels=True)
plt.savefig('graph.pdf', format='pdf', bbox_inches='tight')
