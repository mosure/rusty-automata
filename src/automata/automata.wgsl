#define_import_path rusty_automata::automata


@group(0) @binding(1)
var edges: texture_storage_2d<rgba32float, read_write>;

@group(0) @binding(2)
var nodes: texture_storage_2d<rgba32float, read_write>;


// TODO: 4th channel for synapse decay or mobility?
// TODO: add visualizer for edge (absolute location doesn't view well)
struct Edge {
    from_node_location: vec2<i32>,
    weight: f32,
};

// TODO: add PID (instead of bias?)
struct State {
    value: f32,
    derivative: f32,
    integral: f32,
};


fn get_edge(location: vec2<i32>) -> Edge {
    let edge_lookup = textureLoad(
        edges,
        location,
    );

    return Edge(
        vec2<i32>(
            i32(edge_lookup.x),
            i32(edge_lookup.y),
        ),
        edge_lookup.z,
    );
}

fn set_edge(location: vec2<i32>, edge: Edge) -> void {
    textureStore(
        edges,
        location,
        vec4<f32>(
            f32(edge.from_node_location.x),
            f32(edge.from_node_location.y),
            edge.weight,
            1.0,
        ),
    );
}

fn get_state(location: vec2<i32>) -> State {
    let state_lookup = textureLoad(
        nodes,
        location,
    );

    return State(
        state_lookup.x,
        state_lookup.y,
        state_lookup.z,
    );
}

fn set_state(location: vec2<i32>, state: State) -> void {
    textureStore(
        nodes,
        location,
        vec4<f32>(
            state.value,
            state.derivative,
            state.integral,
            1.0,
        ),
    );
}
