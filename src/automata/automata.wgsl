#define_import_path rusty_automata::automata


@group(0) @binding(1)
var edges: texture_storage_2d<rgba32float, read_write>;

@group(0) @binding(2)
var nodes: texture_storage_2d<rgba32float, read_write>;


// TODO: 4th channel for synapse decay or mobility?
struct Edge {
    from_node_offset: vec2<i32>, // TODO: change to from_node and precompute from offset in init stage
    weight: f32,
};

// TODO: add PID (instead of bias?)
struct State {
    value: f32,
    velocity: f32,
    bias: f32,
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
            f32(edge.from_node_offset.x),
            f32(edge.from_node_offset.y),
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
            state.velocity,
            state.bias,
            1.0,
        ),
    );
}
