#import rusty_automata::noise
#import rusty_automata::uaf


@group(0) @binding(0)
var activations: texture_storage_2d<rgba32float, read_write>;

@group(0) @binding(1)
var edges: texture_storage_2d<rgba32float, read_write>;

@group(0) @binding(2)
var nodes: texture_storage_2d<rgba32float, read_write>;

struct NeatUniforms {
    edge_neighborhood: u32,
};

@group(0) @binding(3)
var<uniform> uniforms: NeatUniforms;


fn hash(value: u32) -> u32 {
    var state = value;
    state = state ^ 2747636419u;
    state = state * 2654435769u;
    state = state ^ state >> 16u;
    state = state * 2654435769u;
    state = state ^ state >> 16u;
    state = state * 2654435769u;
    return state;
}

fn randomFloat(value: u32) -> f32 {
    return f32(hash(value)) / 4294967295.0;
}

@compute @workgroup_size(8, 8, 1)
fn init(
    @builtin(global_invocation_id) invocation_id: vec3<u32>,
    @builtin(num_workgroups) num_workgroups: vec3<u32>,
) {
    let location = vec2<i32>(i32(invocation_id.x), i32(invocation_id.y));
    let location_f32 = vec2<f32>(location);

    let initial_state = simplexNoise2(location_f32);

    let uaf_a = simplexNoise2(location_f32 * vec2<f32>(-1.0, -2.0));
    let uaf_b = simplexNoise2(location_f32 * vec2<f32>(11.0, 31.0));
    let uaf_c = simplexNoise2(location_f32 * vec2<f32>(43.0, -41.0));
    let uaf_d = simplexNoise2(location_f32 * vec2<f32>(-37.0, -17.0));
    let uaf_e = simplexNoise2(location_f32 * vec2<f32>(3.0, -5.0));

    let activation = vec4<f32>(
        -abs(uaf_a),
        uaf_b,
        -abs(uaf_c),
        abs(uaf_d),
    );
    let node = vec4<f32>(
        initial_state,
        0.0,
        0.0,//uaf_e,
        0.0,
    );

    textureStore(
        activations,
        location,
        activation
    );
    textureStore(
        nodes,
        location,
        node
    );

    let center_val = uniforms.edge_neighborhood / 2u;
    let center = vec2<i32>(vec2<u32>(center_val));

    for (var x = 0u; x < uniforms.edge_neighborhood; x = x + 1u) {
        for (var y = 0u; y < uniforms.edge_neighborhood; y = y + 1u) {
            let offset = vec2<i32>(vec2<u32>(x, y));

            let xr = simplexNoise2(location_f32 * vec2<f32>(23.0 * 34.0 * f32(x), -23.0 + 12.0 * f32(y))) * 100.0;
            let yr = simplexNoise2(location_f32 * vec2<f32>(-12.0 * 27.0 * f32(x), 72.0 + 25.0 * f32(y))) * 100.0;
            let edge_weight = simplexNoise2(location_f32 * vec2<f32>(13.0 * -23.0 * f32(x), 17.0 + -11.0 * f32(y))) * 9.0;

            let max_radius = 5.0;
            let edge_offset = vec2<f32>(
                f32(xr) % max_radius,
                f32(yr) % max_radius,
            );

            // basic CA neighborhood
            let edge = vec4<f32>(
                edge_offset,
                edge_weight,
                1.0,
            );

            textureStore(
                edges,
                location + offset,
                edge
            );
        }
    }
}


@compute @workgroup_size(8, 8, 1)
fn update(
    @builtin(global_invocation_id) invocation_id: vec3<u32>,
) {
    let location = vec2<i32>(i32(invocation_id.x), i32(invocation_id.y));

    let activation = textureLoad(
        activations,
        location,
    );

    let current_state = textureLoad(
        nodes,
        location,
    );

    var input_sum = 0.0;
    for (var x = 0u; x < uniforms.edge_neighborhood; x = x + 1u) {
        for (var y = 0u; y < uniforms.edge_neighborhood; y = y + 1u) {
            let offset = vec2<i32>(vec2<u32>(x, y));

            let edge = textureLoad(
                edges,
                location + offset,
            );
            let edge_weight = edge.z;

            let sampled_node_offset = vec2<i32>(edge.xy);
            let sampled_node_location = location + sampled_node_offset;
            let edge_node = textureLoad(
                nodes,
                sampled_node_location,
            );

            input_sum += edge_weight * edge_node.x;
        }
    }

    let uaf_params = UafParameters(
        activation.x,
        activation.y,
        activation.z,
        activation.w,
        current_state.z,
    );
    let pre_activation = current_state.x + input_sum / pow(f32(uniforms.edge_neighborhood), 2.0);
    let next_state = fUAFp(pre_activation, uaf_params);

    storageBarrier();

    textureStore(
        nodes,
        location,
        vec4<f32>(
            next_state,
            0.0,//current_state.x - next_state,
            current_state.z,
            1.0,
        )
    );
}
