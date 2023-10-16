from rusty_automata import _rusty_automata



population = _rusty_automata.NeatPopulation([
    _rusty_automata.NeatGraph([
        _rusty_automata.NeatNode(
            _rusty_automata.NeatUafActivation(1, 1, 1, 1, 100),
            [
                _rusty_automata.NeatEdge(1.0, 0),
            ]
        ),
        _rusty_automata.NeatNode(
            _rusty_automata.NeatUafActivation(1, 1, 1, 1, 100),
            [
                _rusty_automata.NeatEdge(1.0, 1),
            ]
        ),
    ]),
    _rusty_automata.NeatGraph([
        _rusty_automata.NeatNode(
            _rusty_automata.NeatUafActivation(1, 1, 1, 1, 100),
            [
                _rusty_automata.NeatEdge(1.0, 0),
            ]
        ),
    ]),
])


textures = _rusty_automata.population_to_textures(population)
print(textures.node_data())
