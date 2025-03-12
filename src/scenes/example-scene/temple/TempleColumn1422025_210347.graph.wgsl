
fn Heart(input2: vec2f) -> vec3f {
	var PI = 3.14159265359;
	var HALF_PI = 3.14159265359 / 2.0;
	var TWO_PI = 3.14159265359 * 2.0;
	let pos = vec3(input2.x, 0.0, 2. * input2.y) * PI;

    let x2 = sin(pos.x) * (15. * sin(pos.z) - 4. * sin(3. * pos.z));
    let y2 = 8. * cos(pos.x);
    let z2 = sin(pos.x) * (15. * cos(pos.z) - 5. * cos(2. * pos.z) - 2. * cos(3. * pos.z) - cos(2. * pos.z));
    let heart = vec3(x2, y2, z2) * 0.2;
    
    return heart;
}
fn Sphere(input2: vec2f) -> vec3f {
	var PI = 3.14159265359;
	var HALF_PI = 3.14159265359 / 2.0;
	var TWO_PI = 3.14159265359 * 2.0;
    var u = input2.x * PI;
    var v = input2.y * TWO_PI;
	var sx = sin(u);
	var sy = sin(v);
	var cx = cos(u);
	var cy = cos(v);
	var x = sx * cy;
	var y = sx * sy;
	var result = vec3f(x, y, cx);
	return result;
}
fn Plane(input2: vec2f) -> vec3f {
	return vec3f(input2.x, 0, input2.y);
}
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
fn Cube(input2: vec2f) -> vec3f {
	var PI = 3.14159265359;
	var HALF_PI = 3.14159265359 / 2.0;
	var TWO_PI = 3.14159265359 * 2.0;
    var u = input2.x * TWO_PI;
    var v = input2.y * TWO_PI;
    let x = sign(sin(u));
    let y = sign(sin(u * v));
    let z = sign(sin(v));
	var result = vec3f(x, y, z);
	return result;
}
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

fn sampleObject(input2: vec2f) -> vec3f {
	var PI = 3.14159265359;
	var HALF_PI = 3.14159265359 / 2.0;
	var TWO_PI = 3.14159265359 * 2.0;
	var ca_baseThickness = 0.25000000000000000000;
	var columnHeight = 7.00000000000000000000;
	var ct_baseThickness = 2.00000000000000000000;
	var columnShape = 0.00000000000000000000;
	var numColumnsX = 2.00000000000000000000;
	var columnSpacing = 3.50000000000000000000;
	var halfCenterSpacingX = 3.00000000000000000000;
	var ref_b8259 = numColumnsX / 2;
	var ref_714b0 = 0 - ref_b8259;
	var instanceId = f32(instance_id);
	var ref_97b72 = ref_714b0 * columnSpacing;
	var ref_2056a = instanceId / numColumnsX;
	var ref_0f40e = ref_2056a * columnSpacing;
	var ref_dd71c = 3 - ref_0f40e;
	var ref_a69b8 = Plane(input2);
	var ref_91c3c = Cylinder(input2);
	var ref_0e1f8 = Sphere(input2);
	var ref_84e9f_1 = ref_91c3c[0];
	var ref_84e9f_2 = ref_91c3c[1];
	var ref_84e9f_3 = ref_91c3c[2];
	var ref_53a4d = ref_84e9f_2 + 0.5;
	var ref_9ca16 = ref_53a4d * 3.2;
	var ref_4d8e7 = ref_9ca16 - columnShape;
	var ref_bfa54 = ref_4d8e7 * ref_4d8e7;
	var ref_48300 = cos(1.00000000000000000000 * ref_bfa54 + 0.00000000000000000000);
	var ref_39cd7 = ref_48300 * ref_48300;
	var ref_89cd5 = ref_53a4d * columnHeight;
	var ref_2f0f9 = instanceId % numColumnsX;
	var ref_336ac = ref_2f0f9 * columnSpacing;
	var ref_5bf06 = ref_336ac + ref_97b72;
	var ref_3a454 = ref_84e9f_1 * PI;
	var ref_4e26b = ct_baseThickness + ref_39cd7;
	var ref_1f326 = ca_baseThickness * ref_4e26b;
	var ref_2ccbe = cos(1.00000000000000000000 * ref_3a454 + 0.00000000000000000000);
	var ref_262db = ref_2ccbe * ref_1f326;
	var ref_7413e = sin(1.00000000000000000000 * ref_3a454 + 0.00000000000000000000);
	var ref_a0bbe = ref_7413e * ref_1f326;
	var ref_e3c40 = vec3f(ref_262db, ref_89cd5, ref_a0bbe);
	var ref_cd701_1 = ref_e3c40[0];
	var ref_cd701_2 = ref_e3c40[1];
	var ref_cd701_3 = ref_e3c40[2];
	var ref_ed779 = ref_cd701_3 + ref_5bf06;
	var ref_ba25c = ref_cd701_1 + ref_dd71c;
	var ref_9beb0 = vec3f(ref_cd701_1, ref_cd701_2, ref_ed779);
	return ref_9beb0;

}