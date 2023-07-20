#define_import_path rusty_automata::automata

#import rusty_automata::noise                   simplex_2d


struct AutomataUniforms {
    edge_neighborhood: u32,
    width: u32,
    height: u32,
};

@group(0) @binding(0)
var<uniform> automata_uniforms: AutomataUniforms;

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

fn set_next_state(
    location: vec2<i32>,
    next_value: f32,
) {
    let current_state = get_state(location);

    let derivative = current_state.value - next_value;
    let integral = current_state.integral + derivative;
    let next_state = State(
        next_value,
        derivative,
        integral,
    );

    storageBarrier();

    set_state(location, next_state);
}


fn pre_activation(location: vec2<i32>) -> f32 {
    let edge_location = location * i32(automata_uniforms.edge_neighborhood);

    let current_state = get_state(location);

    var input_sum = 0.0;
    for (var x = 0u; x < automata_uniforms.edge_neighborhood; x = x + 1u) {
        for (var y = 0u; y < automata_uniforms.edge_neighborhood; y = y + 1u) {
            let offset = vec2<i32>(vec2<u32>(x, y));

            let edge = get_edge(edge_location + offset);
            let from_node = get_state(edge.from_node_location);

            input_sum += edge.weight * from_node.value;
        }
    }

    return current_state.value + input_sum / pow(f32(automata_uniforms.edge_neighborhood), 2.0);
}


fn init_automata(
    location: vec2<i32>,
) {
    init_state(location);
    init_edges(location);
}

fn init_state(
    location: vec2<i32>,
) {
    let initial_state = 0.0;//simplex_2d(location_f32 / 128.0);
    set_state(
        location,
        State(
            initial_state,
            0.0,
            0.0,
        ),
    );
}


// // TODO: move to shaping module
// // https://www.shadertoy.com/view/4sVBRz
// fn ring(st: vec2<f32>) -> f32 {
//     let r = 0.25;
//     let dr = 0.50;

//     let d = length(st);
//     let c = smoothstep(r, r - (dr / 2.0), d) +
//             smoothstep(r, r + (dr / 2.0), d);
//     return c;
// }

// TODO: expose more init params through uniforms
// TODO: edge initialization (with init based on gaussian sampling /w transform offset from location)
// TODO: initialize with more IoC orders of noise/shaping functions and non-uniform edge direction (e.g. skew along a curve)
fn init_edges(
    location: vec2<i32>,
) {
    let edge_location = location * i32(automata_uniforms.edge_neighborhood);
    let edge_location_f32 = vec2<f32>(edge_location);

    //let ring_factor = min(1.0, ring(vec2<f32>(location) / vec2<f32>(f32(automata_uniforms.height), f32(automata_uniforms.height)) - vec2<f32>(f32(automata_uniforms.width) / f32(automata_uniforms.height) / 2.0, 0.5)) + 0.6);
    let ring_factor = simplex_2d(vec2<f32>(location) * vec2<f32>(0.001, 0.001)) * 0.5 + 1.0;

    for (var x = 0u; x < automata_uniforms.edge_neighborhood; x = x + 1u) {
        for (var y = 0u; y < automata_uniforms.edge_neighborhood; y = y + 1u) {
            let offset = vec2<i32>(vec2<u32>(x, y));

            let xr = simplex_2d(edge_location_f32 + vec2<f32>(23.0 + f32(x), -23.0 + 12.0 * f32(y))) * 120.0;
            let yr = simplex_2d(-edge_location_f32 + vec2<f32>(-12.0 + 27.0 * f32(x), 72.0 + -25.0 * f32(y))) * 120.0;

            let edge_weight = simplex_2d(edge_location_f32 + vec2<f32>(13.0 + -23.0 * f32(x), 17.0 + -11.0 * f32(y))) * neat_uniforms.max_edge_weight;

            let edge_offset = vec2<f32>(
                f32(xr) % neat_uniforms.max_radius * ring_factor,
                f32(yr) % neat_uniforms.max_radius * ring_factor,
            );

            let from_node_location = location + vec2<i32>(edge_offset);
            set_edge(
                edge_location + offset,
                Edge(
                    from_node_location,
                    edge_weight,
                )
            );
        }
    }
}
