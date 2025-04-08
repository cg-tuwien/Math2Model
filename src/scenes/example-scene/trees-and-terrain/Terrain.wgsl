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
	var ref_96e8f_1 = input2[0];
	var ref_96e8f_2 = input2[1];
	var ref_0f4d7 = vec2f(ref_96e8f_1, ref_96e8f_2);
	var ref_64b01 = vec2f(ref_96e8f_2, ref_96e8f_1);
	var hillyness = 2.80000000000000004441;
	var hillHeight = 0.40000000000000002220;
	var ref_3ce74 = hillyness * 2;
	var ref_9d2a3 = hillyness * 0.5;
	var ref_0c4ef = hillHeight * 2;
	var ref_fd798 = hillHeight * 0.5;
	var ref_1c8b8 = hillyness * ref_64b01;
	var ref_1f762 = ref_9d2a3 * ref_0f4d7;
	var ref_3e379 = ref_3ce74 * ref_0f4d7;
	var ref_85bda = cnoise(ref_3e379);
	var ref_6cbaa = cnoise(ref_1f762);
	var ref_22172 = cnoise(ref_1c8b8);
	var ref_c49d3 = ref_85bda * ref_fd798;
	var ref_d17b8 = ref_6cbaa * ref_0c4ef;
	var ref_41355 = ref_22172 * hillHeight;
	var ref_3c079 = ref_c49d3 + ref_d17b8;
	var ref_52c00 = ref_3c079 + ref_41355;
	var ref_03492 = vec3f(ref_96e8f_1, ref_52c00, ref_96e8f_2);
	var ref_fd03c = mat3x3(vec3f(100,0.0,0.0), vec3f(0.0,10,0.0), vec3f(0.0,0.0,100)) * ref_03492;
	return ref_fd03c;

}

fn getColor(input: vec2f) -> vec3f {
    let height = sampleObject(input).y;
    var offset = 0.0;
    let thresh = -0.3;
    let d = 1.2;
    if(height > thresh + d) {
        offset = 1;
    } else if(height > thresh - d) {
        offset = smoothstep(0, 1, (height - (thresh-d)) * (1.0/(2*d)));
    }

    let uv = clamp((abs(input * 3)%1) * vec2f(0.5, 1), vec2f(0.001), vec2f(0.999));

    let ground = textureSample(t_diffuse, linear_sampler, uv + vec2f(0, 0)).rgb;
    let grass = textureSample(t_diffuse, linear_sampler, uv + vec2f(0.5, 0)).rgb;
    
    return mix(ground, grass, offset);
    // return vec3f(uv, 0.0);
}

fn triangle(v: f32) -> f32 {
    return abs(v%2 - 1);
}
fn triangle2(v: vec2f) -> vec2f {
    return abs(v%2 - 1);
}