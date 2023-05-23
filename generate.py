# Generates a random planar graph
#
# Usage: python3 ./generate.py NUMBER_NODES NODE_PROBABILITY EDGE_PROBABILITY
#   NUMBER_NODES: number of nodes in the graph
#   NODE_PROBABILITY: probability that a node gets edges associated
#   EDGE_PROBABILITY: probability that a edge is created

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

if len(sys.argv) < 4:
    print("Usage: ", sys.argv[0], " NUMBER_NODES NODE_PROBABILITY EDGE_PROBABILITY")
    exit(1)

number_nodes = int(sys.argv[1])
node_prob = float(sys.argv[2])
edge_prob = float(sys.argv[3])

G = create_random_planar_graph(number_nodes, node_prob, edge_prob)

is_planar, embedding = nx.check_planarity(G)

# sanity check
assert is_planar

data = embedding.get_data()

print(len(embedding.edges()))

for node, dst_nodes in embedding.get_data().items():
    for dst in dst_nodes:
        print(node, dst)
