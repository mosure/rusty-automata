#define_import_path rusty_automata::automata

#import rusty_automata::noise                   gaussian_rand, simplex_2d


struct AutomataUniforms {
    edge_count: u32,
    max_radius: f32,
    max_edge_weight: f32,
    seed: f32,
    width: u32,
    height: u32,
};


// TODO: separate init and update shaders so read-only textures can be bound as readonly
@group(0) @binding(0)
var edges: texture_storage_2d_array<rgba32float, read_write>;

@group(0) @binding(1)
var nodes: texture_storage_2d<rgba32float, read_write>;

@group(0) @binding(2)
var<uniform> automata_uniforms: AutomataUniforms;


// TODO: 4th channel for synapse decay or mobility?
// TODO: add visualizer for edge (absolute location doesn't view well)
// TODO: from_node_location interpolation (e.g. non-integer locations)
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


fn get_edge(
    location: vec2<i32>,
    index: u32,
) -> Edge {
    let edge_lookup = textureLoad(
        edges,
        location,
        index,
    );

    return Edge(
        vec2<i32>(
            i32(edge_lookup.x),
            i32(edge_lookup.y),
        ),
        edge_lookup.z,
    );
}

fn set_edge(
    location: vec2<i32>,
    index: u32,
    edge: Edge,
) -> void {
    textureStore(
        edges,
        location,
        index,
        vec4<f32>(
            f32(edge.from_node_location.x),
            f32(edge.from_node_location.y),
            edge.weight,
            1.0,
        ),
    );
}

fn get_state(
    location: vec2<i32>,
) -> State {
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

fn set_state(
    location: vec2<i32>,
    state: State,
) -> void {
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

    // TODO: test performance of each barrier
    storageBarrier();
    //workgroupBarrier();

    set_state(location, next_state);
}


fn pre_activation(
    location: vec2<i32>,
) -> f32 {
    let current_state = get_state(location);

    var input_sum = 0.0;
    for (var i = 0u; i < automata_uniforms.edge_count; i = i + 1u) {
        let edge = get_edge(location, i);
        let from_node = get_state(edge.from_node_location);

        input_sum += edge.weight * from_node.value;
    }

    return current_state.value + input_sum / f32(automata_uniforms.edge_count);
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
    let scaled_location = vec2<f32>(location) / vec2<f32>(f32(automata_uniforms.width), f32(automata_uniforms.height));

    //let ring_factor = min(1.0, ring(vec2<f32>(location) / vec2<f32>(f32(automata_uniforms.height), f32(automata_uniforms.height)) - vec2<f32>(f32(automata_uniforms.width) / f32(automata_uniforms.height) / 2.0, 0.5)) + 0.6);
    let ring_factor = simplex_2d(vec2<f32>(location) * vec2<f32>(0.001, 0.001)) * 0.5 + 1.0;

    for (var i = 0u; i < automata_uniforms.edge_count; i = i + 1u) {
        // TODO: consider gaussian sampling with shaping function from above?
        let xr = gaussian_rand(scaled_location + f32(i * automata_uniforms.width) * 0.07 + automata_uniforms.seed) * automata_uniforms.max_radius;
        let yr = gaussian_rand(scaled_location + f32(3u * i * automata_uniforms.width) * 0.03 + automata_uniforms.seed) * automata_uniforms.max_radius;

        var edge_weight = gaussian_rand(scaled_location + f32(7u * i) * 0.01 + automata_uniforms.seed) * automata_uniforms.max_edge_weight;

        let edge_offset = vec2<f32>(
            xr,// * ring_factor,
            yr,// * ring_factor,
        );

        var from_node_location = location + vec2<i32>(edge_offset);

        if (from_node_location.x < 0 || from_node_location.x >= i32(automata_uniforms.width) ||
            from_node_location.y < 0 || from_node_location.y >= i32(automata_uniforms.height)) {
            from_node_location = vec2<i32>(0, 0);
            edge_weight = 0.0;
        }

        set_edge(
            location,
            i,
            Edge(
                from_node_location,
                edge_weight,
            )
        );
    }
}
