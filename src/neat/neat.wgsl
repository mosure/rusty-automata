#define_import_path rusty_automata::neat

#import rusty_automata::automata                automata_uniforms, get_state, init_automata, pre_activation, set_next_state
#import rusty_automata::noise                   gaussian_rand
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
        0.0, // TODO: bind UAF.e to a texture (or uniform 'noise-floor'?)
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
    let current_state = get_state(location);

    let next_value = fUAFp(
        pre_activation(location, current_state),
        get_uaf_params(location),
    );

    set_next_state(location, current_state, next_value);
}


fn init_neat_field(
    location: vec2<i32>,
) {
    let scaled_location = vec2<f32>(location) / vec2<f32>(f32(automata_uniforms.width), f32(automata_uniforms.height));

    let uaf_a = gaussian_rand(scaled_location + vec2<f32>(-0.01, -0.02));
    let uaf_b = gaussian_rand(scaled_location + vec2<f32>(0.011, 0.031));
    let uaf_c = gaussian_rand(scaled_location + vec2<f32>(0.043, -0.041));
    let uaf_d = gaussian_rand(scaled_location + vec2<f32>(-0.037, -0.017));
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
