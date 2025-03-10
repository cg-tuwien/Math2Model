
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

fn sampleObject(input2: vec2f) -> vec3f {
	var PI = 3.14159265359;
	var HALF_PI = 3.14159265359 / 2.0;
	var TWO_PI = 3.14159265359 * 2.0;
	var ca_baseThickness = 0.25000000000000000000;
	var columnHeight = 7.00000000000000000000;
	var ct_baseThickness = 2.00000000000000000000;
	var columnShape = 0.00000000000000000000;
	var numColumnsX = 2.00000000000000000000;
	var columnSpacing = 3.00000000000000000000;
	var halfCenterSpacingX = 3.00000000000000000000;
	var ref_88612 = numColumnsX / 2;
	var ref_7093a = 0 - ref_88612;
	var instanceId = f32(instance_id);
	var ref_a410f = ref_7093a * columnSpacing;
	var ref_30fd2 = instanceId / numColumnsX;
	var ref_0e1c8 = ref_30fd2 * columnSpacing;
	var ref_6a154 = 3 - ref_0e1c8;
	var ref_45e8e = Plane(input2);
	var ref_e2a78 = Cylinder(input2);
	var ref_085c9 = Sphere(input2);
	var ref_fea3d_1 = ref_e2a78[0];
	var ref_fea3d_2 = ref_e2a78[1];
	var ref_fea3d_3 = ref_e2a78[2];
	var ref_d2881 = ref_fea3d_2 + 0.5;
	var ref_24e00 = ref_d2881 * 3.2;
	var ref_7acc0 = ref_24e00 - columnShape;
	var ref_2cad0 = ref_7acc0 * ref_7acc0;
	var ref_b8874 = cos(1.00000000000000000000 * ref_2cad0 + 0.00000000000000000000);
	var ref_84da8 = ref_b8874 * ref_b8874;
	var ref_a086d = cos(1.00000000000000000000 * ref_fea3d_1 + 0.00000000000000000000);
	var ref_b75cb = ref_d2881 * columnHeight;
	var ref_00c36 = sin(1.00000000000000000000 * ref_fea3d_1 + 0.00000000000000000000);
	var ref_f0a16 = instanceId % numColumnsX;
	var ref_ab37d = ref_f0a16 * columnSpacing;
	var ref_8a81c = ref_ab37d + ref_a410f;
	var ref_09303 = ct_baseThickness + ref_84da8;
	var ref_16e35 = ca_baseThickness * ref_09303;
	var ref_9cc7a = ref_a086d * ref_16e35;
	var ref_04d28 = ref_00c36 * ref_16e35;
	var ref_68f9a = vec3f(ref_9cc7a, ref_b75cb, ref_04d28);
	var ref_e74ac_1 = ref_68f9a[0];
	var ref_e74ac_2 = ref_68f9a[1];
	var ref_e74ac_3 = ref_68f9a[2];
	var ref_f4e96 = ref_e74ac_3 + ref_8a81c;
	var ref_492c2 = ref_e74ac_1 + ref_6a154;
	var ref_583ed = vec3f(ref_e74ac_1, ref_e74ac_2, ref_f4e96);
	return ref_583ed;

}