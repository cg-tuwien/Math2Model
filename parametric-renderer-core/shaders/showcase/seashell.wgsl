const PI = 3.14159265359;
const HALF_PI = PI / 2.0;
const TWO_PI = PI * 2.0;

fn getSpiralPosition(theta: f32, A: f32, alphaRad: f32, betaRad: f32, scale: f32) -> vec4f {
    var radius = A * exp(theta * (1.0 / tan(alphaRad)));
    var pos = vec3(
        /* x: */ radius * sin(theta),
        /* y: */ radius * cos(theta),
    /* z: */ radius);
    pos.x *= sin(betaRad) * scale;
    pos.y *= sin(betaRad) * scale;
    pos.z *= -cos(betaRad) * scale;
    return vec4(pos, radius);
}

// gets position on the surface of a sheashell:
// u, v parrameters =^= ellipsePos, theta
// @param alphaDeg      angle of tangent in degrees
// @param betaDeg       enlarging angle in z in degrees
// @param a             radius of ellipse at 0 deg
// @param b             radius of ellipse at 90 deg
// @param muDeg         rotation of ellipse along side-to-side axis relative to face in degrees
// @param nodWidthE     nodule width relative to opening
// @param numNodE       number of nodules along ellipse direction
// @param numNodS       number of nodules along spiral direction
// @param nodHeightF    nodule height scale factor
// @param spikynessE    how spiky the nodules shall be along ellipse direction
// @param spikynessS    how spiky the nodules shall be along spiral direction
fn getSeashellPos(ellipsePos: f32,  theta: f32,  alphaDeg: f32,  betaDeg: f32,  a: f32,  b: f32,  muDeg: f32,  nodWidthE: f32,  numNodE: f32,  numNodS: f32,  nodHeightF: f32,  spikynessE: f32,  spikynessS: f32) -> vec3f
{
    var scale = 0.01;

    // spiral parameters
    var A        = 25.0;                 // distance from origin at theta = 0
    var coils    = 8.0;                  // number coils
    var alphaRad = alphaDeg * (PI / 180.0);  // angle of tangent (offset from 90 degrees)
    var betaRad  = betaDeg * (PI / 180.0);  // enlarging angle in z

    // ellipse opening parameters
    var phi     =  60.674 * (PI / 180.0);    // rotation of ellipse along z-axis (front-to-back axis relative to face)
    var omega   =   7.584 * (PI / 180.0);    // rotation of ellipse along y-axis (top-to-bottom axis relative to face)
    var mu      = muDeg * (PI / 180.0);      // rotation of ellipse along x-axis (side-to-side axis relative to face)

    // nodule parameters
    //    ellipse:
    var nodHeightE = 20.0 * nodHeightF;
    var nodOffsetE = 52.331 * (PI / 180.0);
    //    spiral:
    var nodHeightS =   5.674 * nodHeightF;
    var nodOffsetS =  40.65 * (PI / 180.f);
    var nodWidthS  = 125.4;

    // (float theta, float A, float alphaRad, float betaRad, float scale)
    var spiral = getSpiralPosition(ellipsePos, A, alphaRad, betaRad, scale);

    var spiralPos = vec3(spiral.x, spiral.y, spiral.z);
    var spiralRadius = spiral.w;


    // float re  = pow(pow2(cos(theta) / a) + pow2(sin(theta) / b), -0.5f);
    var pos = vec3(a * sin(theta), b * cos(theta), 0.0);
    // Position correctly for initial position:
    var xold = pos.x;
    pos.x = pos.z;
    pos.z = xold;
    // swap(pos.x, pos.z);
    // Tilt:
    pos = rotate_x(pos, -phi);
    pos = rotate_y(pos, -mu);
    pos = rotate_z(pos, -omega);
    // Rotate according to spiral position:
    pos = rotate_z(pos, -ellipsePos);

    // Add nodule along the ellipse:
    var eper   = (TWO_PI / numNodE);
    var eshift = theta - nodOffsetE;
    var ern    = eshift / eper - round(eshift / eper);
    var enh    = exp(-pow(nodWidthE / numNodE, spikynessE) * ern * ern) * nodHeightE / 20.f;

    // Add nodule along the spiral:
    var sper   = (TWO_PI / numNodS);
    var sshift = ellipsePos - nodOffsetS;
    var srn    = sshift / sper - round(sshift / sper);
    var snh    = exp(-pow(nodWidthS / numNodS, spikynessS) * srn * srn) * nodHeightS / 20.f;

    pos *= spiralRadius / A * (scale + scale * enh * snh);
    // Translate the ellipse according to the spiral position:
    pos += spiralPos;

    var oldy = pos.y;
    pos.y = pos.z;
    pos.z = oldy;
    // swap(pos.y, pos.z);
    return pos;
}

fn rotate_x(v: vec3<f32>, phi: f32) -> vec3f {
    let cos_phi = cos(phi);
    let sin_phi = sin(phi);
    return vec3(
        v.x,
        cos_phi * v.y - sin_phi * v.z,
        sin_phi * v.y + cos_phi * v.z
    );
}

fn rotate_y(v: vec3<f32>, phi: f32) -> vec3f {
    let cos_phi = cos(phi);
    let sin_phi = sin(phi);
    return vec3(
        /* x: */ cos_phi * v.x + sin_phi * v.z,
        /* y: */ v.y,
        /* z: */ -sin_phi * v.x + cos_phi * v.z);
}

fn rotate_z(v: vec3<f32>, phi: f32) -> vec3f {
    let cos_phi = cos(phi);
    let sin_phi = sin(phi);
    return vec3(
        /* x: */ cos_phi * v.x - sin_phi * v.y,
        /* y: */ sin_phi * v.x + cos_phi * v.y,
        /* z: */ v.z);
}

fn sampleObject(input: vec2f) -> vec3f {
    let pos = vec3(16. * input.x, 0.0, 2. * input.y) * 3.14159265359;
    let u = pos.x;
    let v = pos.z;

    let seashell1 = getSeashellPos(u, v, 86.5, 10.0, 6.071, 5.153, -39.944, 1.0, 9.0, 11.0, 1.0, 1.5, 1.5);
    let seashell2 = getSeashellPos(u, v, 86.5, 10.0, 6.071, 5.153, -39.944, 100.0, 50.0, 70.0, 0.6, 4.0, 5.0);

    return mix(seashell1, seashell2, 0.);
}

/*
return getSeashellPos(u, v, 
        86.5, 10.0, 6.071, 5.153, -39.944, 
        1.0,    9.0, 1.0, 1.0, 1.5, 1.5
    );

return getSeashellPos(u, v, 
        86.5, 10.0, 6.071, 5.153, -39.944, 
        1.0,    9.0, 11.0, 1.0, 1.5, 1.5
    );

return getSeashellPos(u, v, 
        86.5, 10.0, 6.071, 5.153, -39.944, 
        100.0, 50.0, 70.0, 0.6, 4.0, 5.0
    );
*/