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
	var PI = 3.14159265359;
	var HALF_PI = 3.14159265359 / 2.0;
	var TWO_PI = 3.14159265359 * 2.0;
	var ref_1b67e = Sphere(input2);
	var ref_fb3af_1 = ref_1b67e[0];
	var ref_fb3af_2 = ref_1b67e[1];
	var ref_fb3af_3 = ref_1b67e[2];
	var ref_75fae = sign(sin(ref_1b67e*1));
	var instanceId = f32(instance_id);
	var ref_2733c = instanceId * HALF_PI;
	var ref_480cf = ref_fb3af_2 * 1.5;
	var ref_f7960_1 = ref_75fae[0];
	var ref_f7960_2 = ref_75fae[1];
	var ref_f7960_3 = ref_75fae[2];
	var ref_b7133 = sin(1 * ref_2733c + 0);
	var ref_7efc3 = cos(1 * ref_2733c + 0);
	var ref_b07c1 = ref_b7133 * ref_f7960_3;
	var ref_76e38 = ref_fb3af_1 * ref_7efc3;
	var ref_a33df = ref_fb3af_1 + ref_7efc3;
	var ref_23d6d = ref_b7133 + ref_fb3af_3;
	var ref_f57c4 = vec3f(ref_a33df, ref_480cf, ref_23d6d);
	return ref_f57c4;

}