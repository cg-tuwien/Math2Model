const PI = 3.14159265359;
const HALF_PI = PI / 2.0;
const TWO_PI = PI * 2.0;

fn binomial_coefficient(n: f32, k: f32) -> f32 {
    // return n.Factorial() / ((n - k).Factorial() * k.Factorial());

    // Optimized method, see http://stackoverflow.com/questions/9619743/how-to-calculate-binomial-coefficents-for-large-numbers
    // (n C k) and (n C (n-k)) are the same, so pick the smaller as k:
    var k_2 = k;
    if (k > n - k) {
        k_2 = n - k;
    }
    var result = 1.;
    for (var i = 1.; i <= k_2; i = i + 1.) {
        result *= n - k_2 + i;
        result /= i;
    }
    return result;
}

fn bernstein_polynomial (i: f32, n: f32, t: f32) -> vec3f {
    return vec3f(binomial_coefficient(n, i) * pow(t, i) * pow(1. - t, n - t));
}

fn bezier_value_at(controlPoints: array<vec3f, 4>, t: f32) -> vec3f {
    var n = 4. - 1.;
    var sum = vec3f(0.0f, 0.0f, 0.0f);
    sum += bernstein_polynomial(0., n, t) * controlPoints[0];
    sum += bernstein_polynomial(1., n, t) * controlPoints[1];
    sum += bernstein_polynomial(2., n, t) * controlPoints[2];
    sum += bernstein_polynomial(3., n, t) * controlPoints[3];
    return sum;
}

fn bezier_slope_at(controlPoints: array<vec3f, 4>, t: f32) -> vec3f {
    var  n         = 4. - 1.;
    var  nMinusOne = n - 1.;
    var sum = vec3(0.0f, 0.0f, 0.0f);
    sum += (controlPoints[0 + 1] - controlPoints[0]) * bernstein_polynomial(0., nMinusOne, t);
    sum += (controlPoints[1 + 1] - controlPoints[1]) * bernstein_polynomial(1., nMinusOne, t);
    sum += (controlPoints[2 + 1] - controlPoints[2]) * bernstein_polynomial(2., nMinusOne, t);
    return n * sum;
}

// Get a vector which is orthogonal to the given vector v:
fn orthogonal(v: vec3f) -> vec3f {
    if (v.x == 0. && v.y == 0.) {
        return vec3f(1, 0, 0);
    } else {
        return normalize(vec3f(v.y, -v.x, 0));
    }
}

fn rotate_around_axis(point: vec3f, rot_axis: vec3f, angle: f32) -> vec3f
{
    var axis = normalize(rot_axis);
    var s = sin(angle);
    var c = cos(angle);
    var oc = 1.0 - c;

    return (mat4x4(oc * axis.x * axis.x + c,           oc * axis.x * axis.y - axis.z * s,  oc * axis.z * axis.x + axis.y * s,  0.0,
                oc * axis.x * axis.y + axis.z * s,  oc * axis.y * axis.y + c,           oc * axis.y * axis.z - axis.x * s,  0.0,
                oc * axis.z * axis.x - axis.y * s,  oc * axis.y * axis.z + axis.x * s,  oc * axis.z * axis.z + c,           0.0,
                0.0,                                0.0,                                0.0,                                1.0) *
                vec4f(point, 1.)).xyz;
}

/** Rotates v around the z-axis by phi radians
 * \brief R_z
 * \param v Vector to be rotated
 * \param phi Angle to be rotated by
 * \return Rotated vector
 */
fn rotate_z(v: vec3<f32>, phi: f32) -> vec3f {
    let cos_phi = cos(phi);
    let sin_phi = sin(phi);
    return vec3(
        /* x: */ cos_phi * v.x - sin_phi * v.y,
        /* y: */ sin_phi * v.x + cos_phi * v.y,
        /* z: */ v.z);
}

fn getLeafPos(u: f32, v: f32) -> vec3f {
    var a = 2.5f;
    var b = 5.0;

    var leaves = vec3f(
        a * cos(b * u) * cos(v) * cos(u) * cos(v),
        a * cos(b * u) * cos(v) * sin(u) * cos(v),
        /* Offset from trunk to leaves = */ 0.2f,
    );

    var pos = rotate_z(leaves, /* Rotation of leaves = */ 0.0f * PI / 180.f);

    var start = vec2f(pos.x, pos.y);
    var end   = vec2f(0.f, 0.f);
    pos.z        = pos.z - pow((length(end - start) * /* leaves scale param 1: */ 0.664f * cos(v)), /* leaves scale param 1: */ 3.27f);

    pos.x *= /* leaves xy scale: */ 3.392f;
    pos.y *= /* leaves xy scale: */ 3.392f;
    return pos;
};

fn getPalmTreeTrunkPos(controlPoints: array<vec3f, 4>, u: f32, v: f32, thickness: f32) -> vec3f
{
  var pos            = bezier_value_at(controlPoints, u);
  var upwards        = normalize(bezier_slope_at(controlPoints, u));

  // Manually make a pipe (without using the pipeify function, but same steps):
  var nTrunk              = orthogonal(upwards);

  // Generate palm tree trunk shape:
  var numElements = 10.f;
  var sawtooth = (u * numElements) - floor(u * numElements);
  var sawtoothscale = sawtooth * 0.3f + 1.f;

  // Let the tree trunk be a bit thicker at the bottom:
  var thicknessscale = 1.f / (u + 1.f) + 0.5f;

  var treeTrunkPos   = pos + rotate_around_axis(nTrunk * thickness * sawtoothscale * thicknessscale, upwards, v);
  var n            = normalize(treeTrunkPos - pos);
  return treeTrunkPos;
}

fn sampleObject(input: vec2f) -> vec3f {
    let pos = vec3(input.x / PI, 0.0, 2. * input.y) * 3.14159265359;

    let x = sin(pos.x) * cos(pos.z);
    let y = sin(pos.x) * sin(pos.z);
    let z = cos(pos.x);

    let x2 = sin(pos.x) * (15. * sin(pos.z) - 4. * sin(3. * pos.z));
    let y2 = 8. * cos(pos.x);
    let z2 = sin(pos.x) * (15. * cos(pos.z) - 5. * cos(2. * pos.z) - 2. * cos(3. * pos.z) - cos(2. * pos.z));

    let sphere = vec3(x, y, z) * 3.0;
    let heart = vec3(x2, y2, z2) * 0.2;

    var controlPoints = array<vec3f, 4>(
        vec3f(1., -10., 1.),
        vec3f(0., -8., 1.),
        vec3f(5., -1., 1.),
        vec3f(-1., 0., 1.)
    );

    let p = getPalmTreeTrunkPos(controlPoints, pos.x, pos.z, 0.3);

    return p;
}