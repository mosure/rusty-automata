#define_import_path rusty_automata::neat

#import rusty_automata::automata                init_automata, pre_activation, set_next_state
#import rusty_automata::noise                   simplex_2d
#import rusty_automata::uaf                     fUAFp, UafParameters


@group(1) @binding(0)
var uaf_activations: texture_storage_2d<rgba32float, read_write>;

@compute @workgroup_size(4, 4, 1)
fn init(
    @builtin(global_invocation_id) invocation_id: vec3<u32>,
) {
    // TODO: change location type to user defined location_t?
    let location = vec2<i32>(i32(invocation_id.x), i32(invocation_id.y));
    init_neat_field(location);
}

@compute @workgroup_size(4, 4, 1)
fn update(
    @builtin(global_invocation_id) invocation_id: vec3<u32>,
) {
    let location = vec2<i32>(i32(invocation_id.x), i32(invocation_id.y));
    compute_next_neat_state(location);
}



fn get_uaf_params(
    location: vec2<i32>,
) -> UafParameters {
    let activation = textureLoad(
        uaf_activations,
        location,
    );

    return UafParameters(
        activation.x,
        activation.y,
        activation.z,
        activation.w,
        0.0,
    );
}

fn set_uaf_params(
    location: vec2<i32>,
    activation: UafParameters,
) {
    textureStore(
        uaf_activations,
        location,
        vec4<f32>(
            activation.a,
            activation.b,
            activation.c,
            activation.d,
        )
    );
}


fn compute_next_neat_state(
    location: vec2<i32>,
) {
    let x = pre_activation(location);
    let uaf_params = get_uaf_params(location);

    let next_value = fUAFp(x, uaf_params);

    set_next_state(location, next_value);
}


fn init_neat_field(
    location: vec2<i32>,
) {
    let location_f32 = vec2<f32>(location);

    let uaf_a = simplex_2d(location_f32 * vec2<f32>(-1.0, -2.0));
    let uaf_b = simplex_2d(location_f32 * vec2<f32>(11.0, 31.0));
    let uaf_c = simplex_2d(location_f32 * vec2<f32>(43.0, -41.0));
    let uaf_d = simplex_2d(location_f32 * vec2<f32>(-37.0, -17.0));
    set_uaf_params(
        location,
        UafParameters(
            -abs(uaf_a),
            abs(uaf_b) / 1000.0,
            -abs(uaf_c),
            abs(uaf_d),
            0.0,
        ),
    );

    init_automata(location);
}
