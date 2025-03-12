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