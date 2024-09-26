fn Sphere(input2: vec2f) -> vec3f {
	var PI = 3.14159265359;
	var HALF_PI = 3.14159265359 / 2.0;
	var TWO_PI = 3.14159265359 * 2.0;
	var ref_ef97b_1 = input2[0];
	var ref_ef97b_2 = input2[1];
	var ref_2c047 = ref_ef97b_1 * PI;
	var ref_105e3 = ref_ef97b_2 * TWO_PI;
	var ref_6b1dd = sin(ref_2c047);
	var ref_f7d13 = sin(ref_105e3);
	var ref_a5d3e = cos(ref_2c047);
	var ref_9b66f = cos(ref_105e3);
	var ref_aa9e2 = ref_6b1dd * ref_9b66f;
	var ref_67ac8 = ref_6b1dd * ref_f7d13;
	var ref_6b4fe = ref_aa9e2 * 3;
	var ref_a275a = ref_67ac8 * 3;
	var ref_a4bc0 = ref_a5d3e * 3;
	var ref_8c690 = vec3f(ref_6b4fe, ref_a275a, ref_a4bc0);
	return ref_8c690;
}