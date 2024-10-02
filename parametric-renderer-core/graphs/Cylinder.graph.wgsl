fn Cylinder(input2: vec2f, radius: f32, height: f32) -> vec3f {
    let pos = vec3(2. * input2.x, 0.0, 2. * input2.y) * 3.14159265359;
    var y = input2.x * height;

    var sx = sin(pos.x);
    var sy = sin(pos.z);
    var cx = cos(pos.x);
    var cy = cos(pos.z);

    var x = radius * cy;
    var z = radius * sy;

    return vec3f(x, y, z);
}