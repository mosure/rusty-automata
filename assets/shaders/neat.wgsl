#import bevy_sprite::mesh2d_view_bindings
#import bevy_pbr::utils


const Pi: f32 = 3.14159265359;
fn rsqrt(x: f32) -> f32 {
    return 1.0 / sqrt(x);
}

fn clamp01(x: f32) -> f32 {
    return clamp(x, 0.0, 1.0);
}

// https://en.wikipedia.org/wiki/Error_function#Approximation_with_elementary_functions
fn Erf(x: f32) -> f32 {
    let neg: bool = x < 0.0;

    let a: f32 = 0.147;
    let b: f32 = 1.27324; // 4.0/Pi

    let xx: f32 = x * x;
    let xxa: f32 = xx * a;
    let y: f32 = sqrt(1.0 - exp(-xx * (xxa + b) / (xxa + 1.0)));

    if (neg) {
        return -y;
    } else {
        return y;
    };
}

fn ErfI(x: f32) -> f32 {
    let neg: bool = x < 0.0;

    let a: f32 = 6.802721; // 1.0/0.147
    let b: f32 = 4.330747; // 2.0 / Pi * a

    let u: f32 = log(1.0 - x * x);
    let c: f32 = u * 0.5 + b;

    let y: f32 = sqrt(sqrt(c * c - u * a) - c);

    if (neg) {
        return -y;
    } else {
        return y;
    };
}

fn ErfStep(x: f32, s: f32) -> f32 {
    return Erf(ErfI(x) * s);
}

fn Erf2(x: f32) -> f32 {
    let neg: bool = x < 0.0;
    let x: f32 = abs(x);

    let p: f32 = 0.3275911;
    let a1: f32 = 0.254829592;
    let a2: f32 = -0.284496736;
    let a3: f32 = 1.421413741;
    let a4: f32 = -1.453152027;
    let a5: f32 = 1.061405429;

    let t: f32 = 1.0 / (1.0 + p * x);
    
    let y: f32 = 1.0 - (a1 * t + a2 * t * t + a3 * t * t * t + a4 * t * t * t * t + a5 * t * t * t * t * t) * exp(-x * x);

    if (neg) {
        return -y;
    } else {
        return y;
    };
}



// ----- The rest is just for demo

fn mixColor(baseCol: vec4<f32>, color: vec4<f32>, alpha: f32) -> vec4<f32> {
    return vec4<f32>(mix(baseCol.rgb, color.rgb, alpha * color.a), 1.0);
}

fn drawGrid(baseCol: vec4<f32>, xy: vec2<f32>, dxy: vec2<f32>, stepSize: f32, gridCol: vec4<f32>) -> vec4<f32> {
    let mul: f32 = 1.0 / stepSize;
    var g: vec2<f32> = abs(vec2<f32>(-0.5) + fract((xy + vec2<f32>(stepSize) * 0.5) * mul)); // g passes 0 at stepSize intervals
    g = vec2<f32>(1.0) - smoothstep(vec2<f32>(0.0), dxy * mul * 1.5, g);
    return mixColor(baseCol, gridCol, max(g.x, g.y));
}

fn Plot(f: f32, y: f32) -> f32 {
    var v: f32 = f - y;
    v /= length(vec2<f32>(dpdx(v), dpdy(v)));
    return clamp01(1.0 - abs(v * 0.5));
}

fn drawCurve(baseCol: vec4<f32>, y: f32, value: f32, curveCol: vec4<f32>) -> vec4<f32> {
    return mixColor(baseCol, curveCol, Plot(value, y) * curveCol.w);
}


@fragment
fn fragment(
    @builtin(position) position: vec4<f32>,
    #import bevy_sprite::mesh2d_vertex_output
) -> @location(0) vec4<f32> {
    var uv = coords_to_viewport_uv(position.xy, view.viewport);
    uv = vec2<f32>(uv.x, 1.0 - uv.y);

    let aspect: f32 = view.viewport.z / view.viewport.w;
    let graphSize: vec2<f32> = vec2<f32>(aspect * 1.2, 1.2);
    let graphPos: vec2<f32> = 0.5 - graphSize * 0.5;

    var xy: vec2<f32> = graphPos + uv * graphSize; // graph coords
    let dxy: vec2<f32> = graphSize / view.viewport.zw; // pixel size in graph units

    // background
    var col: vec4<f32> = mix(vec4<f32>(1.0, 1.0, 1.0, 1.0), vec4<f32>(0.7, 0.7, 0.7, 1.0), pow(length(0.5 - uv) * 1.414, 3.5));

    // grid
    col = drawGrid(col, xy, dxy, 0.1, vec4<f32>(0.0, 0.0, 0.0, 0.2));
    col = drawGrid(col, xy, dxy, 0.5, vec4<f32>(0.0, 0.0, 0.0, 0.3));
    col = drawGrid(col, xy, dxy, 1.0, vec4<f32>(0.0, 0.0, 0.0, 0.4));

    // curves
    xy = xy * 2.0 - 1.0;
    let l: f32 = mix(1.0 / 6.0, 6.0, sin(globals.time) * 0.5 + 0.5);

    col = drawCurve(col, xy.y, Erf(xy.x * l), vec4<f32>(0.91, 0.13, 0.23, 1.0));

    return col;
}
