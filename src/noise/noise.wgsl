#define_import_path rusty_automata::noise


//  MIT License. © Ian McEwan, Stefan Gustavson, Munrocket, Johan Helsing
//
fn mod289_2d(x: vec2<f32>) -> vec2<f32> {
    return x - floor(x * (1.0 / 289.0)) * 289.0;
}

fn mod289_3d(x: vec3<f32>) -> vec3<f32> {
    return x - floor(x * (1.0 / 289.0)) * 289.0;
}

fn permute_3d(x: vec3<f32>) -> vec3<f32> {
    return mod289_3d(((x * 34.0) + 1.0) * x);
}

//  MIT License. © Ian McEwan, Stefan Gustavson, Munrocket
fn simplex_2d(v: vec2<f32>) -> f32 {
    let C = vec4(
        0.211324865405187, // (3.0-sqrt(3.0))/6.0
        0.366025403784439, // 0.5*(sqrt(3.0)-1.0)
        -0.577350269189626, // -1.0 + 2.0 * C.x
        0.024390243902439 // 1.0 / 41.0
    );

    // First corner
    var i = floor(v + dot(v, C.yy));
    let x0 = v - i + dot(i, C.xx);

    // Other corners
    var i1 = select(vec2(0., 1.), vec2(1., 0.), x0.x > x0.y);

    // x0 = x0 - 0.0 + 0.0 * C.xx ;
    // x1 = x0 - i1 + 1.0 * C.xx ;
    // x2 = x0 - 1.0 + 2.0 * C.xx ;
    var x12 = x0.xyxy + C.xxzz;
    x12.x = x12.x - i1.x;
    x12.y = x12.y - i1.y;

    // Permutations
    i = mod289_2d(i); // Avoid truncation effects in permutation

    var p = permute_3d(permute_3d(i.y + vec3(0., i1.y, 1.)) + i.x + vec3(0., i1.x, 1.));
    var m = max(0.5 - vec3(dot(x0, x0), dot(x12.xy, x12.xy), dot(x12.zw, x12.zw)), vec3(0.));
    m *= m;
    m *= m;

    // Gradients: 41 points uniformly over a line, mapped onto a diamond.
    // The ring size 17*17 = 289 is close to a multiple of 41 (41*7 = 287)
    let x = 2. * fract(p * C.www) - 1.;
    let h = abs(x) - 0.5;
    let ox = floor(x + 0.5);
    let a0 = x - ox;

    // Normalize gradients implicitly by scaling m
    // Approximation of: m *= inversesqrt( a0*a0 + h*h );
    m *= 1.79284291400159 - 0.85373472095314 * (a0 * a0 + h * h);

    // Compute final noise value at P
    let g = vec3(a0.x * x0.x + h.x * x0.y, a0.yz * x12.xz + h.yz * x12.yw);
    return 130. * dot(m, g);
}



//  <https://www.shadertoy.com/view/Xd23Dh>
//  by Inigo Quilez
//
fn hash_23_(p: vec2<f32>) -> vec3<f32> {
    let q = vec3<f32>(dot(p, vec2<f32>(127.1, 311.7)),
        dot(p, vec2<f32>(269.5, 183.3)),
        dot(p, vec2<f32>(419.2, 371.9)));
    return fract(sin(q) * 43758.5453);
}

fn voro_2d(x: vec2<f32>, u: f32, v: f32) -> f32 {
    let p = floor(x);
    let f = fract(x);
    let k = 1.0 + 63.0 * pow(1. - v, 4.0);
    var va: f32 = 0.0;
    var wt: f32 = 0.0;
    for(var j: i32 = -2; j <= 2; j = j + 1) {
      for(var i: i32 = -2; i <= 2; i = i + 1) {
        let g = vec2<f32>(f32(i), f32(j));
        let o = hash_23_(p + g) * vec3<f32>(u, u, 1.0);
        let r = g - f + o.xy;
        let d = dot(r, r);
        let ww = pow(1. - smoothstep(0.0, 1.414, sqrt(d)), k);
        va = va + o.z * ww;
        wt = wt + ww;
      }
    }
    return va / wt;
}



fn nrand(n: vec2<f32>) -> f32 {
    return fract(sin(dot(n, vec2<f32>(12.9898, 4.1414))) * 43758.5453);
}

fn noise_2d(n: vec2<f32>) -> f32 {
    let d = vec2<f32>(0.0, 1.0);
    let b = floor(n);
    let f = smoothstep(vec2<f32>(0.0), vec2<f32>(1.0), fract(n));
    return mix(mix(nrand(b), nrand(b + d.yx), f.x), mix(nrand(b + d.xy), nrand(b + d.yy), f.x), f.y);
}


// https://www.shadertoy.com/view/MlVSzw
const ALPHA: f32 = 0.14;
const INV_ALPHA: f32 = 7.14285714286;
const K: f32 = 0.08912676813;

fn inv_error_function(x: f32) -> f32 {
    let y: f32 = log(1.0 - x*x);
    let z: f32 = K + 0.5 * y;
    return sqrt(sqrt(z*z - y * INV_ALPHA) - z) * sign(x);
}

fn gaussian_rand(n: vec2<f32>) -> f32 {
    let x: f32 = nrand(n);

    return inv_error_function(x * 2.0 - 1.0) * 0.15 + 0.5;
}
