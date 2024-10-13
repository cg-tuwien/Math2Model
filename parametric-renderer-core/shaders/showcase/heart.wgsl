fn sampleObject(input: vec2f) -> vec3f {
    let pos = vec3(input.x, 0.0, 2. * input.y) * 3.14159265359;
    let PI = 3.14159265359;
    let HALF_PI = PI / 2.0;
    let u = pos.x;
    let v = pos.z;
    let x = sin(u) * cos(v);
    let y = cos(u);
    let z = sin(u) * sin(v);
    var px = x;
    var py = y;

    if (u < HALF_PI) {
        py = y * (1.0 - (cos(sqrt(sqrt(abs(x * PI * 0.7)))) * 0.8));
    }
    else {
        px = x * sin(u) * sin(u);
    }
    let p = vec3(px * 0.9, py, z * 0.4);

    return p;
}