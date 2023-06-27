#define_import_path rusty_automata::plot


fn mix_color(baseCol: vec4<f32>, color: vec4<f32>, alpha: f32) -> vec4<f32> {
    return vec4<f32>(mix(baseCol.rgb, color.rgb, alpha * color.a), 1.0);
}

fn draw_grid(baseCol: vec4<f32>, xy: vec2<f32>, dxy: vec2<f32>, stepSize: f32, gridCol: vec4<f32>) -> vec4<f32> {
    let mul: f32 = 1.0 / stepSize;
    var g: vec2<f32> = abs(vec2<f32>(-0.5) + fract((xy + vec2<f32>(stepSize) * 0.5) * mul)); // g passes 0 at stepSize intervals
    g = vec2<f32>(1.0) - smoothstep(vec2<f32>(0.0), dxy * mul * 1.5, g);
    return mix_color(baseCol, gridCol, max(g.x, g.y));
}

fn plot_point(f: f32, y: f32) -> f32 {
    var v: f32 = f - y;
    v /= length(vec2<f32>(dpdx(v), dpdy(v)));
    return clamp(
        1.0 - abs(v * 0.5),
        0.0,
        1.0
    );
}

fn draw_curve(baseCol: vec4<f32>, y: f32, value: f32, curveCol: vec4<f32>) -> vec4<f32> {
    return mix_color(baseCol, curveCol, plot_point(value, y) * curveCol.w);
}
