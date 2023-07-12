#define_import_path rusty_automata::uaf


fn log1p(x: f32) -> f32 {
    return log(1.0 + x);
}

fn fUAF(x: f32, a: f32, b: f32, c: f32, d: f32, e: f32) -> f32 {
    let p1: f32 = (a * (x + b)) + (c * (x * x));
    let p2: f32 = (d * (x - b));

    let p3: f32 = max(p1, 0.0) + log1p(exp(-abs(p1)));
    let p4: f32 = max(p2, 0.0) + log1p(exp(-abs(p2)));

    return p3 - p4 + e;
}


struct UafParameters {
    a: f32,
    b: f32,
    c: f32,
    d: f32,
    e: f32,
};

fn fUAFp(x: f32, params: UafParameters) -> f32 {
    return fUAF(x, params.a, params.b, params.c, params.d, params.e);
}
