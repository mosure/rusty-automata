#import rusty_automata::automata                Edge, get_edge, set_edge, State, get_state, set_state
#import rusty_automata::noise                   simplex_2d
#import rusty_automata::uaf                     fUAFp, UafParameters


// TODO: move activations/uniform to NEAT lib (not example)
@group(0) @binding(0)
var activations: texture_storage_2d<rgba32float, read_write>;

struct NeatUniforms {
    edge_neighborhood: u32,
    activation_floor: f32, // 4.0 is sparse but interesting, 10.0 has more activity
    max_radius: f32,
    max_edge_weight: f32,
};

@group(0) @binding(3)
var<uniform> uniforms: NeatUniforms;


// TODO: move init pipeline to NEAT lib (inherit init of state/edges from automata), including parameterization for init
@compute @workgroup_size(8, 8, 1)
fn init(
    @builtin(global_invocation_id) invocation_id: vec3<u32>,
) {
    let location = vec2<i32>(i32(invocation_id.x), i32(invocation_id.y));
    let location_f32 = vec2<f32>(location);
    let edge_location = location * i32(uniforms.edge_neighborhood);
    let edge_location_f32 = vec2<f32>(edge_location);

    let initial_state = 0.0;//simplex_2d(location_f32 / 128.0);

    let uaf_a = simplex_2d(location_f32 * vec2<f32>(-1.0, -2.0));
    let uaf_b = simplex_2d(location_f32 * vec2<f32>(11.0, 31.0));
    let uaf_c = simplex_2d(location_f32 * vec2<f32>(43.0, -41.0));
    let uaf_d = simplex_2d(location_f32 * vec2<f32>(-37.0, -17.0));
    let uaf_e = simplex_2d(location_f32 * vec2<f32>(3.0, -5.0)) / uniforms.activation_floor;

    let activation = vec4<f32>(
        -abs(uaf_a),
        abs(uaf_b) / 1000.0,
        -abs(uaf_c),
        abs(uaf_d),
    );
    let state = State(
        initial_state,
        0.0,
        -abs(uaf_e)
    );

    textureStore(
        activations,
        location,
        activation
    );
    set_state(
        location,
        state
    );

    let center_val = uniforms.edge_neighborhood / 2u;
    let center = vec2<i32>(vec2<u32>(center_val));

    for (var x = 0u; x < uniforms.edge_neighborhood; x = x + 1u) {
        for (var y = 0u; y < uniforms.edge_neighborhood; y = y + 1u) {
            let offset = vec2<i32>(vec2<u32>(x, y));

            let xr = simplex_2d(edge_location_f32 + vec2<f32>(23.0 + f32(x), -23.0 + 12.0 * f32(y))) * 120.0;
            let yr = simplex_2d(edge_location_f32 * 3.0 + vec2<f32>(-12.0 + 27.0 * f32(x), 72.0 + -25.0 * f32(y))) * 120.0;

            let edge_weight = simplex_2d(edge_location_f32 + vec2<f32>(13.0 + -23.0 * f32(x), 17.0 + -11.0 * f32(y))) * uniforms.max_edge_weight;

            let edge_offset = vec2<f32>(
                f32(xr) % uniforms.max_radius,
                f32(yr) % uniforms.max_radius,
            );

            set_edge(
                edge_location + offset,
                Edge(
                    vec2<i32>(edge_offset),
                    edge_weight,
                )
            );
        }
    }
}


// TODO: allow IoC activation function (e.g. GoL rules vs. UAF)
@compute @workgroup_size(8, 8, 1)
fn update(
    @builtin(global_invocation_id) invocation_id: vec3<u32>,
) {
    let location = vec2<i32>(i32(invocation_id.x), i32(invocation_id.y));
    let edge_location = location * i32(uniforms.edge_neighborhood);

    let activation = textureLoad(
        activations,
        location,
    );

    let current_state = get_state(location);

    var input_sum = 0.0;
    for (var x = 0u; x < uniforms.edge_neighborhood; x = x + 1u) {
        for (var y = 0u; y < uniforms.edge_neighborhood; y = y + 1u) {
            let offset = vec2<i32>(vec2<u32>(x, y));

            let edge = get_edge(edge_location + offset);

            let sampled_node_location = location + edge.from_node_offset;
            let from_node = get_state(sampled_node_location);

            input_sum += edge.weight * from_node.value;
        }
    }

    let uaf_params = UafParameters(
        activation.x,
        activation.y,
        activation.z,
        activation.w,
        current_state.bias,
    );
    let pre_activation = current_state.value + input_sum / pow(f32(uniforms.edge_neighborhood), 2.0);
    var next_state = fUAFp(pre_activation, uaf_params);

    let delta = abs(current_state.value - next_state); // TODO: add visual layer

    storageBarrier();

    set_state(
        location,
        State(
            next_state,
            delta, // (current_state.velocity + delta) / 2.0,
            current_state.bias,
        )
    );

    // TODO: adapt edges (weights and offsets)
}
