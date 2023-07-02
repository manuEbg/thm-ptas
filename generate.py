import argparse
import json
import matplotlib.pyplot as plt
import math
import networkx as nx
import random

class RandomGraphGenerator:
    def __init__(self, args):
        self.args = args

    def generate(self):
        G = self.create_random_planar_graph(self.args.nodes, self.args.nprob, self.args.eprob)
        is_planar, embedding = nx.check_planarity(G)

        # sanity check
        assert is_planar

        # We assume that dividing by two always gives the correct amount of undirected
        # edges because we add all edges that do not violate planarity in our
        # generation algorithm. Adding a back-edge to another edge does not violate
        # planarity because if it would, the already existing edge would've been
        # incorrect. Thus, we add all back-edges and have 2m directed edges and m
        # undirected edges.
        # That also means dividing by two always produces an integer.

        # Sanity check for 2m edges.
        assert len(embedding.edges()) % 2 == 0
        return embedding, nx.planar_layout(embedding)

    def create_random_planar_graph(self, n, node_prob, edge_prob):
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

class CircularGraphGenerator:
    def __init__(self, args):
        self.args = args

    def generate(self):
        G = nx.Graph()

        root_node = 0
        G.add_node(root_node)

        SPACING = 1.0
        layout = {0: [0.0,0.0]}

        previous_ring = [root_node]

        for n in range(self.args.rings):
            # Add a new ring around the current graph
            ring = list(range(G.order(), G.order() + self.args.nodes))
            G.add_nodes_from(ring)

            # Connect the nodes in the ring
            for i in range(self.args.nodes-1):
                G.add_edge(ring[i], ring[i+1])
            G.add_edge(ring[-1], ring[0])

            # Connect nodes of this ring with the previous ring
            if len(previous_ring) == 1:
                # First ring is only connected with the root node
                for i in range(self.args.nodes):
                    G.add_edge(previous_ring[0], ring[i])
            else:
                for i in range(self.args.nodes):
                    G.add_edge(previous_ring[i], ring[i])

            # Generate layout positions
            for i in range(self.args.nodes):
                angle = i/self.args.nodes*2.0*math.pi
                dist = (n + 1) * SPACING
                x = math.sin(angle) * dist
                y = math.cos(angle) * dist

                layout[ring[i]] = [x, y]
            previous_ring = ring

        is_planar, embedding = nx.check_planarity(G)

        # sanity check
        assert is_planar
        return embedding, layout

def main():
    parser = argparse.ArgumentParser(
            description='Generate random planar graphs')
    parser.add_argument('--nodes', type=int, help='max number of nodes', required=True)
    parser.add_argument('--nprob', type=float, help='probability that a node is created', required=True)
    parser.add_argument('--eprob', type=float, help='probability that an edge is created', required=True)
    parser.add_argument('--rings', type=int, help='amount of rings for circular graph', default=3)
    parser.add_argument('--type', type=str, help='type of the graph', choices=['random', 'circular'], default='random')
    parser.add_argument('outfile', type=str, help='destination file')
    args = parser.parse_args()

    number_nodes = args.nodes
    outfile = args.outfile

    if args.type == 'random':
        gen = RandomGraphGenerator(args)
    else:
        gen = CircularGraphGenerator(args)

    embedding, layout = gen.generate()

    with open(outfile, "w", encoding="utf-8") as f:
        f.write(f"{number_nodes}\n{int(len(embedding.edges()) / 2)}\n")
        for node, dest_nodes in embedding.get_data().items():
            for dest in dest_nodes:
                f.write(f"{node} {dest}\n")

    print(f"Edges: {int(len(embedding.edges()) / 2)}")
    print(f"Output in '{outfile}'.")

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

if __name__ == "__main__":
    main()
