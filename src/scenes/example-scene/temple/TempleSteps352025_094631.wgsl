fn sampleObject(input2: vec2f) -> vec3f {
    let mouse_pos_normalized = mouse.pos.x*screen.inv_resolution.x;

    let inp = vec2f(0.5) - input2;
    let dist = 5.-pyramid(inp) * 10.;
    let n = pyramid_normal(inp);
    let n3 = vec3f(n.x, 0.0, n.y);

    let steps = vec3f(triangle_wave(inp.x * 50.), 0., triangle_wave(inp.y* 50.));

    let top_cutoff = 3.0;
    let x = input2.x * 10.0;
    let y = min(dist, top_cutoff);
    let z = input2.y * 10.0;

    return vec3f(x * 3.5, y, z * 3.) - 0.1 * n3 * steps;
}

fn pyramid(a: vec2f) -> f32 {
    let b = abs(a);
    return max(b.x, b.y);
}


fn pyramid_normal(a: vec2f) -> vec2f {
    // We have f(x,y)=max(abs(x),abs(y));
    // We want d/dx f and d/dy f
    // f(x, y)
    // = (abs(x) + abs(y) + |abs(x)-abs(y)|)/2
    // = abs(x)*0.5 + abs(y)*0.5 + abs(abs(x)-abs(y))*0.5

    // f_x =
    // = sign(x)*0.5 + sign(abs(x)-abs(y))*0.5*(sign(x))

    // f_y =
    // = sign(y)*0.5 + sign(abs(x)-abs(y))*0.5*(sign(y))

    // Note:
    // abs' = sign
    // max(x,y) = (x + y + |x-y|)/2

    let n = vec2f(
        sign(a.x) + sign(abs(a.x) - abs(a.y)) * sign(a.x),
        sign(a.y) + sign(abs(a.x) - abs(a.y)) * -sign(a.y),
    ) * 0.5;

    return n;
}

fn triangle_wave(v: f32) -> f32 {
    return abs(2.*fract(v)-1.);
}