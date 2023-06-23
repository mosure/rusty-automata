import networkx as nx
import numpy as np
import random

def get_cycle_probability(dendrite_radius, num_dendrites, iterations=100000, world_size=2):
    cycle_count = 0
    for _ in range(iterations):
        G = nx.random_geometric_graph(world_size, dendrite_radius)
        if nx.is_connected(G):
            # Connect random neurons with dendrites
            for _ in range(num_dendrites):
                neuron1 = random.choice(list(G.nodes))
                neuron2 = random.choice(list(G.nodes))
                
                # Get the positions of each neuron
                pos1 = np.array(G.nodes[neuron1]['pos'])
                pos2 = np.array(G.nodes[neuron2]['pos'])

                # Only add the edge if the neurons are within the dendrite radius of each other
                if np.linalg.norm(pos1 - pos2) <= dendrite_radius:
                    G.add_edge(neuron1, neuron2)

            # Check if a cycle exists
            if nx.cycle_basis(G):  # if the list is not empty, a cycle exists
                cycle_count += 1

    # Return the probability
    return cycle_count / iterations


print(get_cycle_probability(1, 1))
