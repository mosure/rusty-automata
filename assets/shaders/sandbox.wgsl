
#import rusty_automata::noise                   gaussian_rand

#import bevy_sprite::mesh2d_vertex_output       MeshVertexOutput
#import bevy_sprite::mesh2d_view_bindings       globals, view
#import bevy_pbr::utils                         coords_to_viewport_uv


@fragment
fn fragment(
    in: MeshVertexOutput,
) -> @location(0) vec4<f32> {
    let uv = in.uv;

    let noise = gaussian_rand(uv);
    let col = vec4<f32>(
        vec3<f32>(noise),
        1.0,
    );

    return col;
}
