fn sampleObject(input: vec2f) -> vec3f {
    let pos = vec3(input.x, 0.0, 2. * input.y) * 3.14159265359;

    let x = sin(pos.x) * cos(pos.z);
    let y = sin(pos.x) * sin(pos.z);
    let z = cos(pos.x);

    let x2 = sin(pos.x) * (15. * sin(pos.z) - 4. * sin(3. * pos.z));
    let y2 = 8. * cos(pos.x);
    let z2 = sin(pos.x) * (15. * cos(pos.z) - 5. * cos(2. * pos.z) - 2. * cos(3. * pos.z) - cos(2. * pos.z));

    let sphere = vec3(x, y, z) * 3.0;
    let heart = vec3(x2, y2, z2) * 0.2;

    let translation = vec3(f32(instance_id), 0., f32(instance_id) * 0.3) * 6.;

    let p = vec3(mix(sphere, heart, 0.7) * 1.) + translation;

    return p;
}