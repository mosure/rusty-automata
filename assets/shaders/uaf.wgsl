
#import rusty_automata::plot
#import rusty_automata::uaf

#import noisy_bevy::prelude

#import bevy_sprite::mesh2d_view_bindings
#import bevy_pbr::utils


struct UafMaterial {
    a: f32,
    b: f32,
    c: f32,
    d: f32,
    e: f32,
    animate: f32,
};

@group(1) @binding(0)
var<uniform> material: UafMaterial;

const Pi: f32 = 3.14159265359;

@fragment
fn fragment(
    @builtin(position) position: vec4<f32>,
    #import bevy_sprite::mesh2d_vertex_output
) -> @location(0) vec4<f32> {
    var uv = coords_to_viewport_uv(position.xy, view.viewport);
    uv = vec2<f32>(uv.x, 1.0 - uv.y);

    let aspect: f32 = view.viewport.z / view.viewport.w;
    let graphSize: vec2<f32> = vec2<f32>(aspect * 10.2, 10.2);
    let graphPos: vec2<f32> = graphSize * -0.5;

    var xy: vec2<f32> = graphPos + uv * graphSize; // graph coords
    let dxy: vec2<f32> = graphSize / view.viewport.zw; // pixel size in graph units

    // background
    var col: vec4<f32> = mix(vec4<f32>(0.1, 0.1, 0.1, 1.0), vec4<f32>(0.0, 0.0, 0.0, 1.0), pow(length(0.5 - uv) * 1.414, 3.5));

    // grid
    col = draw_grid(col, xy, dxy, 1.0, vec4<f32>(0.6, 0.6, 0.6, 0.1));
    col = draw_grid(col, xy, dxy, 5.0, vec4<f32>(0.6, 0.6, 0.6, 0.2));
    col = draw_grid(col, xy, dxy, 10.0, vec4<f32>(0.6, 0.6, 0.6, 0.3));

    // curves
    let t = globals.time * 0.5;

    let n1 = mix(
        -2.0,
        2.0,
        simplex_noise_2d(vec2(t * 0.5, 10.1))
    );
    let n2 = mix(
        -0.8,
        0.8,
        simplex_noise_2d(vec2(t, -345.6))
    );
    let n3 = mix(
        -1.0,
        1.0,
        simplex_noise_2d(vec2(t, 400.3))
    );
    let n4 = mix(
        -2.0,
        2.0,
        simplex_noise_2d(vec2(t * 0.5, 800.6))
    );

    let result = fUAF(
        xy.x,
        material.a + n1 * material.animate,
        material.b + n2 * material.animate,
        material.c + n3 * material.animate,
        material.d + n4 * material.animate,
        material.e,
    );

    col = draw_curve(col, xy.y, result, vec4<f32>(0.91, 0.13, 0.23, 1.0));
    return col;
}
