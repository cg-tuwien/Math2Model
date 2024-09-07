fn sampleObject(input: vec2f) -> vec3f {
    let pos = vec3(input.x, 0.0, 2. * input.y) * 3.14159265359;
    let u = pos.x;
    let v = pos.z;
    var object = vec3(
    /* x: */ sin(u) * cos(v),
    /* y: */ cos(u),
    /* z: */ sin(u) * sin(v)
    );

    let PI = 3.14159265359;
    let HALF_PI = PI / 2.0;

    let NUMSPIKES       = 10.0 + 2.0;// * sin(time.elapsed);
    let SPIKENARROWNESS = 100.0;
    let spikeheight     = 0.1 + 0.1;// * sin(time.elapsed);

    var repeatU = u / PI * NUMSPIKES - round(u / PI * NUMSPIKES);
    var repeatV = v / PI * NUMSPIKES - round(v / PI * NUMSPIKES);
    var d       = repeatU * repeatU + repeatV * repeatV;
    var r       = 0.6 + exp(-d * SPIKENARROWNESS) * spikeheight;

    return object * r;
}