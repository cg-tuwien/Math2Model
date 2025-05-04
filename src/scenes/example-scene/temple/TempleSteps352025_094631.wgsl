fn sampleObject(input2: vec2f) -> vec3f {
    // HERE to adjust floor to columns count
    let n_columns = 8.;

    let inp = vec2f(0.5) - input2;
    let dist = 5.-pyramid(inp) * 10.;
    let n = pyramid_normal(inp);
    let n3 = vec3f(n.x, 0.0, n.y);

    let steps = vec3f(triangle_wave(inp.x * 50.), 0., triangle_wave(inp.y* 50.));

    let top_cutoff = 3.0;
    let x = input2.x * 10.0;
    let y = min(dist, top_cutoff);
    let z = input2.y * 10.0;

    return vec3f(x * 3.5, y, z * n_columns / 2 - n_columns * 1.2) - 0.1 * n3 * steps;
}

fn pyramid(a: vec2f) -> f32 {
    let b = abs(a);
    return max(b.x, b.y);
}


fn pyramid_normal(a: vec2f) -> vec2f {
    let n = vec2f(
        sign(a.x) + sign(abs(a.x) - abs(a.y)) * sign(a.x),
        sign(a.y) + sign(abs(a.x) - abs(a.y)) * -sign(a.y),
    ) * 0.5;

    return n;
}

fn triangle_wave(v: f32) -> f32 {
    return abs(2.*fract(v)-1.);
}