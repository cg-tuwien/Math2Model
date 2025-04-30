fn Cylinder(input2: vec2f) -> vec3f {
    let pos = vec3(2. * input2.x, 0.0, 2. * input2.y) * 3.14159265359;
    var y = input2.x;

    var sx = sin(pos.x);
    var sy = sin(pos.z);
    var cx = cos(pos.x);
    var cy = cos(pos.z);

    var x = cy;
    var z = sy;

    return vec3f(x, y, z);
}

fn sampleObject(input: vec2f) -> vec3f {
  var positionUpdated = bricks(input);
  var x = positionUpdated.x;
  var y = positionUpdated.y;
  var z = positionUpdated.z;
  var brickDensity = 0.4;
  var brickSpacing = 1.4;
  var brickEnabled = 1.0;
  var heightAffected = 1.0;
  var strength = 2.0;

  var circleOnY = vec3f(x, 0.0, z);
  var nrmCircleOnY = normalize(circleOnY);
  var r = sqrt(x*x + z*z); // radius
  // Around the circle:
  var phi = acos(nrmCircleOnY.x);

  // based on y, make bricks in y-direction:
  var fy = brickDensity; // step frequency
  var ay = 0.1; // step amplitude
  var heightFactor = smoothstep(2.0, 4.0, y); // ...but not for the top of the tower
  ay *= heightFactor;
  
  // based on phi, make bricks around the circle:
  var fphi = brickDensity * 2.0; // step frequency
  var aphi = 0.1; // step amplitude
  var phiFactor = smoothstep(2.0, 4.0, y); // ...but not for the top of the tower
  aphi *= phiFactor;
  
  var stepY = step(brickSpacing, fy*y     - floor(fy*y)    );
  var stepC = step(brickSpacing, fphi*phi - floor(fphi*phi));
  
  if (brickEnabled != 0.0) {
    positionUpdated = positionUpdated
    + nrmCircleOnY * ay   * stepY  // height-based
    + nrmCircleOnY * aphi * stepC; // circle-based
  }

  positionUpdated.x *= 15.0 + strength * exp(-positionUpdated.y * heightAffected);
  positionUpdated.z *= 15.0 + strength * exp(-positionUpdated.y * heightAffected);
  positionUpdated.y -= 0.41;

  positionUpdated.y *= 18.0;

  return positionUpdated;
}

// Constants not to be changed
const tau = 2. * 3.14159265359;
const height = 1.04;
const radiusBottom = 0.17;
const radiusTop = 0.1;

// HERE to change the size of the space between bricks
const brickSpacing = 0.1;
fn bricks(input: vec2f) -> vec3f {
    let te = time.elapsed;
    let pos = vec3(input.x*tau, 0.0, input.y);
    let prog = pos.x;
    let radius = mix(radiusBottom, radiusTop, input.y);
    var cyl = vec3f(sin(prog)*radius,input.y*height,cos(prog)*radius);
    let normal = normalize(vec3f(cyl.x,0,cyl.z));
    let sOffset = calcSurfaceOffset(pos);
    cyl+=normal*sOffset*0.01;
    return cyl;
}

fn calcSurfaceOffset(pos: vec3f) -> f32 {
    let te = time.elapsed*0.1;

    // HERE to change the frequency of bricks vertically
    let vSteps = 20.;
    let vstep = ceil(fract(pos.z)*vSteps)/vSteps;

    // HERE to change the frequency of bricks horizontally
    let hSteps = 10.;
    var sOffset = step(0.,(fract(pos.z*vSteps)-fract(pos.z*vSteps-brickSpacing)));

    sOffset*= step(0.,(fract((pos.x/tau+vstep)*hSteps)-fract((pos.x/tau+vstep)*hSteps-brickSpacing)));
    return sOffset;
}

fn getColor(input: vec2f) -> vec3f {
    let posUv = vec3(input.x*tau, 0.0, input.y);
    return material.color_roughness.rgb*(calcSurfaceOffset(posUv)+0.4);
}