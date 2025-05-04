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
    var p = vec3f(input2.x * TWO_PI, input2.y, 0.0);
    let adjusted_angle = p.x + TWO_PI / 8.0;
    var s = sin(adjusted_angle);
    var c = cos(adjusted_angle);
    var y0 = (step(0.001,fract(p.y)));
    var a = abs(s) + abs(c);

    var box = vec3f(
        (s + c) / a * y0,
        input2.y,
        (s - c) / a * y0
    );
    return box;
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
	var ref_0c994_1 = input2[0];
	var ref_0c994_2 = input2[1];
	var ref_f48 = 10.00000000000000000000;
	var ref_dc7f0 = ref_0c994_2 * ref_f48;
	var TWO_PI = 3.14159265359 * 2.0;
	var ref_11910 = TWO_PI * ref_0c994_1;
	var ref_88b27 = cos(1 * ref_11910 + 0);
	var ref_35579 = ref_88b27 * 0.80000000000000004441;
	var ref_cddcc = sin(1 * ref_11910 + 0);
	var ref_ae854 = ref_cddcc * 0.80000000000000004441;
	var ref_c1dcd = vec3f(ref_35579, ref_dc7f0, ref_ae854);
	var ref_2aaca = vec3f(ref_35579, 0, ref_ae854);
	var ref_ea4c0 = ref_11910 * 10.00000000000000000000;
	var ref_a7101 = sin(1 * ref_ea4c0 + 0);
	var ref_1c4ba = abs(ref_a7101);
	var ref_69e24 = ref_1c4ba * -0.05000000000000000278;
	var ref_e78a8 = ref_2aaca * ref_69e24;
	var ref_e1bd6 = ref_e78a8 + ref_c1dcd;
	var ref_23d50 = sin(1 * ref_0c994_2 + 2.5);
	var ref_94c5f = ref_23d50 * 0.40000000000000002220;
	var ref_481c0 = ref_2aaca * ref_94c5f;
	var ref_a26c1 = ref_e1bd6 + ref_481c0;
	var ref_07213 = 0.00000000000000000000;
	var instanceId = f32(instance_id);
	var ref_6f597 = instanceId % 2.00000000000000000000;
	var ref_b6cd7 = ref_6f597 * 3.00000000000000000000;
	var ref_9d3cc = instanceId / 2.00000000000000000000;
	var ref_3be67 = floor(ref_9d3cc);
	var ref_47062 = ref_3be67 * 2.00000000000000000000;
	var ref_2df4e = ref_47062 * 1.50000000000000000000;
	var ref_62e9e = f32(8);
	var ref_2ae8d = ref_62e9e / 2.00000000000000000000;
	var ref_b590a = ref_2df4e - ref_2ae8d;
	var ref_81f81 = vec3f(ref_b6cd7, ref_07213, ref_b590a);
	var ref_5517c = ref_a26c1 + ref_81f81;
	return ref_5517c;

}