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
	var ref_477c6_1 = input2[0];
	var ref_477c6_2 = input2[1];
	var ref_7361c = smoothstep(-0.04, 0.5, ref_477c6_2);
	var ref_b30cb = ((1 * ref_477c6_2) - floor(1 * ref_477c6_2));
	var ref_027aa = step(0.0001, ref_b30cb);
	var ref_dde6c = ref_7361c * ref_027aa;
	var ref_4132e = 0.00000000000000000000;
	var TWO_PI = 3.14159265359 * 2.0;
	var ref_79dd9 = 1.00000000100000008274 / 8.00000099999999925160;
	var ref_9807d = ref_477c6_1 + ref_79dd9;
	var ref_aa7b9 = ((1 * ref_9807d) - floor(1 * ref_9807d));
	var ref_d28e0 = ref_aa7b9 * 4.00000000000000000000;
	var ref_ce1ba = ceil(ref_d28e0);
	var ref_cf597 = ref_ce1ba / 4.00000000000000000000;
	var ref_cf3fc = ((1 * ref_cf597) - floor(1 * ref_cf597));
	var ref_16c2e = ref_cf3fc + 0.25000000000000000000;
	var ref_289cf = ref_16c2e * TWO_PI;
	var PI = 3.14159265359;
	var ref_82c7e = ref_289cf + PI;
	var ref_6ea11 = cos(1 * ref_82c7e + 0);
	var ref_0ee22 = sin(1 * ref_82c7e + 0);
	var ref_49045 = vec3f(ref_6ea11, ref_4132e, ref_0ee22);
	var ref_13bcc = ref_dde6c * ref_49045;
	var ref_47aa8 = ref_477c6_1 * 100.00000000000000000000;
	var ref_5f75b = ref_47aa8 + 0.00000000000000000000;
	var ref_7c2e7 = sin(1.3 * ref_5f75b + 0.6);
	var ref_a4c76 = step(0.3, ref_7c2e7);
	var ref_16da9 = ref_7c2e7 * ref_a4c76;
	var ref_7116d = ref_16da9 * 0.05000000000000000278;
	var ref_25602 = ref_13bcc * ref_7116d;
	var ref_8b1b8 = Cube(input2);
	var ref_e07b5_1 = ref_8b1b8[0];
	var ref_e07b5_2 = ref_8b1b8[1];
	var ref_e07b5_3 = ref_8b1b8[2];
	var ref_44b9e = ref_e07b5_1 * ref_7361c;
	var ref_a6d20 = ref_e07b5_3 * ref_7361c;
	var ref_a9a6a = vec3f(ref_44b9e, ref_e07b5_2, ref_a6d20);
	var ref_a59e1 = ref_25602 + ref_a9a6a;
	var ref_86f70 = mat3x3(vec3f(7,0.0,0.0), vec3f(0.0,4,0.0), vec3f(0.0,0.0,1.5)) * ref_a59e1;
	var ref_3bd26_1 = ref_86f70[0];
	var ref_3bd26_2 = ref_86f70[1];
	var ref_3bd26_3 = ref_86f70[2];
	var ref_5fc75 = f32(8);
	var ref_31bd1 = ref_5fc75 * 0.50000000000000000000;
	var ref_3eb0a = ref_3bd26_3 * ref_31bd1;
	var ref_1fb74 = ref_5fc75 - 10.00000000000000000000;
	var ref_69c01 = max(ref_1fb74, 2.00000000000000000000);
	var ref_17576 = ref_5fc75 / ref_69c01;
	var ref_7fa1e = ref_3eb0a - ref_17576;
	var ref_270e3 = vec3f(ref_3bd26_1, ref_3bd26_2, ref_7fa1e);
	return ref_270e3;

}