// =================== ground functionality to place the trees
fn mod289(x: vec4f) -> vec4f
{
  return x - floor(x * (1.0 / 289.0)) * 289.0;
}

fn permute(x: vec4f) -> vec4f
{
  return mod289(((x*34.0)+10.0)*x);
}

fn taylorInvSqrt(r: vec4f) -> vec4f
{
  return 1.79284291400159 - 0.85373472095314 * r;
}

fn fade(t: vec2f) -> vec2f {
  return t*t*t*(t*(t*6.0-15.0)+10.0);
}

fn cnoise(P: vec2f) -> f32
{
  var Pi = floor(P.xyxy) + vec4f(0.0, 0.0, 1.0, 1.0);
  var Pf = fract(P.xyxy) - vec4f(0.0, 0.0, 1.0, 1.0);
  Pi = mod289(Pi);
  var ix = Pi.xzxz;
  var iy = Pi.yyww;
  var fx = Pf.xzxz;
  var fy = Pf.yyww;

  var i = permute(permute(ix) + iy);

  var gx = fract(i * (1.0 / 41.0)) * 2.0 - 1.0 ;
  var gy = abs(gx) - 0.5 ;
  var tx = floor(gx + 0.5);
  gx = gx - tx;

  var g00 = vec2f(gx.x,gy.x);
  var g10 = vec2f(gx.y,gy.y);
  var g01 = vec2f(gx.z,gy.z);
  var g11 = vec2f(gx.w,gy.w);

  var norm = taylorInvSqrt(vec4f(dot(g00, g00), dot(g01, g01), dot(g10, g10), dot(g11, g11)));
  g00 *= norm.x;  
  g01 *= norm.y;  
  g10 *= norm.z;  
  g11 *= norm.w;  

  var n00 = dot(g00, vec2f(fx.x, fy.x));
  var n10 = dot(g10, vec2f(fx.y, fy.y));
  var n01 = dot(g01, vec2f(fx.z, fy.z));
  var n11 = dot(g11, vec2f(fx.w, fy.w));

  var fade_xy = fade(Pf.xy);
  var n_x = mix(vec2f(n00, n01), vec2f(n10, n11), fade_xy.x);
  var n_xy = mix(n_x.x, n_x.y, fade_xy.y);
  return 2.3 * n_xy;
}

fn pnoise(P: vec2f, rep: vec2f) -> f32
{
  var Pi = floor(P.xyxy) + vec4f(0.0, 0.0, 1.0, 1.0);
  var Pf = fract(P.xyxy) - vec4f(0.0, 0.0, 1.0, 1.0);
  Pi = Pi % rep.xyxy;
  Pi = mod289(Pi);
  var ix = Pi.xzxz;
  var iy = Pi.yyww;
  var fx = Pf.xzxz;
  var fy = Pf.yyww;

  var i = permute(permute(ix) + iy);

  var gx = fract(i * (1.0 / 41.0)) * 2.0 - 1.0 ;
  var gy = abs(gx) - 0.5 ;
  var tx = floor(gx + 0.5);
  gx = gx - tx;

  var g00 = vec2f(gx.x,gy.x);
  var g10 = vec2f(gx.y,gy.y);
  var g01 = vec2f(gx.z,gy.z);
  var g11 = vec2f(gx.w,gy.w);

  var norm = taylorInvSqrt(vec4f(dot(g00, g00), dot(g01, g01), dot(g10, g10), dot(g11, g11)));
  g00 *= norm.x;  
  g01 *= norm.y;  
  g10 *= norm.z;  
  g11 *= norm.w;  

  var n00 = dot(g00, vec2f(fx.x, fy.x));
  var n10 = dot(g10, vec2f(fx.y, fy.y));
  var n01 = dot(g01, vec2f(fx.z, fy.z));
  var n11 = dot(g11, vec2f(fx.w, fy.w));

  var fade_xy = fade(Pf.xy);
  var n_x = mix(vec2f(n00, n01), vec2f(n10, n11), fade_xy.x);
  var n_xy = mix(n_x.x, n_x.y, fade_xy.y);
  return 2.3 * n_xy;
}

fn ground_at(x: f32, z: f32) -> vec3f {
    let hillHeight = 0.4;
    let f = 2.8; // frequency
    let scale = vec3f(100.,10.,100.);
    
    return vec3f(x, 
                  2.0 * hillHeight * cnoise(vec2f(x, z) * f / 2.0)
                +       hillHeight * cnoise(vec2f(z, x) * f)
                + 0.5 * hillHeight * cnoise(vec2f(x, z) * f * 2.0),
                z) * scale;
}
// =================== ground functionality copy&paste END






fn rand(co: vec2f) -> f32 {
    return fract(sin(dot(co, vec2(12.9898, 78.233))) * 43758.5453);
}

// returns the thickness based on height=[0, 1]
fn treeThickness(height: f32, instance: f32) -> f32 {
    var instanceThickness = 1.0 / f32(1 + (instance % 3));
    var gotoZero = 1.0 - height*height*height*height;
    var thickerAtBottom = exp(-1.1 * height);
    return thickerAtBottom * gotoZero * instanceThickness;
}

// How much the tree shall bend based on its height
// phi is the circle parameter and determines in which direction to bend
fn bendOnHeight(height: f32, instance: f32) -> f32 {
    var bendStrength = 1.5;
    if (instance % 3 == 1) { bendStrength = -1.0; }
    if (instance % 3 == 2) { bendStrength = -0.5; }
    return smoothstep(0.2, 1.0, height) * bendStrength;
}

// Parametric construction of the tree:
fn tree_at(phi: f32, height: f32) -> vec3f {
    var limbVariation = 0.0;
    var baseThickness = 0.25;
    var thickness = treeThickness(height, f32(instance_id)) * baseThickness;
    var treeHeight = 7.0;
    var treeInstanceHeighth = treeHeight / f32(1 + (instance_id % 3));
    var straightTree = vec3f(thickness * cos(phi), treeInstanceHeighth * height, thickness * sin(phi));
    var bentTree = vec3f(straightTree.x, straightTree.y, straightTree.z - bendOnHeight(height, f32(instance_id)));

    // Rotation matrix around y:
    var rots = vec3f(0.0, -1.0, 1.0);
    var u = rots[instance_id % 3] + limbVariation * 2.0;
    if ((instance_id % 3) != 0) {
        u *= rand(vec2f(f32(instance_id)));
    }
    var rotY = mat3x3(cos(u), 0.0, sin(u),
                      0.0,    1.0, 0.0,
                     -sin(u), 0.0, cos(u));
    var rotatedTree = rotY * bentTree;

    var translatedTree = vec3(
        rotatedTree.x, 
        rotatedTree.y + f32(instance_id % 3) * treeHeight/4.0, 
        rotatedTree.z
    );
    if (instance_id % 3 == 2) {
        translatedTree.z -= 0.5;
    }

    return translatedTree;
}

fn sampleObject(input: vec2f) -> vec3f {
    // CHANGE the value to 1.0 to make the trees be aligned to the terrain
    var adaptToTerrain = 0.0;
    var f = f32(instance_id/3) * 0.25;
    var treePosZ = rand(vec2f(f, cos(f)));
    var treePosX = rand(vec2f(2.0*f, sin(f)));
    // Calculate ground at tree pos:
    // ground_at's input domain is the unit square
    var groundPos = ground_at(treePosX, treePosZ);
    var groundNormalDelta = 0.1;
    var groundTang   = normalize(ground_at(treePosX - groundNormalDelta, treePosZ                    ) - groundPos);
    var groundBitang = normalize(ground_at(treePosX                    , treePosZ + groundNormalDelta) - groundPos);
    var groundNormal = cross(groundTang, groundBitang);

    // Position trees accordingly:
    
    // Parametric modeling:
    var p = tree_at(input.x * 3.14159 * 2, input.y) * 2.;

    // Rotate the whole trees:
    var u = f32(instance_id / 3);
    var rotY = mat3x3( cos(u), 0.0, sin(u),
                      0.0,    1.0, 0.0,
                     -sin(u), 0.0, cos(u));
    
    // Transform position and normal:
    p = rotY * p;
    
    if (adaptToTerrain > 0.0) {
      var dVec = normalize(groundNormal + vec3f(0.0, 1.0, 0.0));
      var uVec = vec3f(0.0, 1.0, 0.0);
      var rVec = normalize(cross(dVec, uVec));
      if (dot(dVec, uVec) > 0.9999999) {
        rVec = rotY * vec3(0.0, 0.0, 1.0);
      }
      var tVec = normalize(cross(rVec, dVec));

      rVec = normalize(mix(vec3f(1.0, 0.0, 0.0), rVec, adaptToTerrain));
      dVec = normalize(mix(vec3f(0.0, 1.0, 0.0), dVec, adaptToTerrain));
      tVec = normalize(mix(vec3f(0.0, 0.0, 1.0), tVec, adaptToTerrain));

      var groundRot = mat3x3(rVec, dVec, tVec);

      // Apply the orientation change:
      p = groundRot * p;
    }
 
    // Position the tree at and according to the terrain:
    p.x += treePosX * 100.; // Terrain scaling factor
    p.y += groundPos.y;
    p.z += treePosZ * 100.;

    return p;    
}