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
  var positionUpdated = Cylinder(input);
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

  positionUpdated.x *= 1.0 + strength * exp(-positionUpdated.y * heightAffected);
  positionUpdated.z *= 1.0 + strength * exp(-positionUpdated.y * heightAffected);
  positionUpdated.y -= 0.41;

  positionUpdated.y *= 18.0;

  return positionUpdated;
}