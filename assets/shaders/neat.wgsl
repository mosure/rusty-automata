#import rusty_automata::neat                    compute_next_neat_state, init_neat_field
#import rusty_automata::noise                   simplex_2d
#import rusty_automata::uaf                     UafParameters


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
