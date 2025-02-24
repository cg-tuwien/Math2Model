fn Heart(input2: vec2f) -> vec3f {
	var PI = 3.14159265359;
	var HALF_PI = 3.14159265359 / 2.0;
	var TWO_PI = 3.14159265359 * 2.0;
	var ref_861d8_1 = input2[0];
	var ref_861d8_2 = input2[1];
	var ref_dee02 = ref_861d8_1 * PI;
	var ref_02c43 = ref_861d8_2 * TWO_PI;
	var ref_34532 = sin(ref_02c43);
	var ref_dcc84 = sin(ref_dee02);
	var ref_857d0 = cos(ref_dee02);
	var ref_0c7dd = cos(ref_02c43);
	var ref_3bd86 = ref_02c43 * 3;
	var ref_039cf = sin(ref_3bd86);
	var ref_77e2e = cos(ref_3bd86);
	var ref_7267f = ref_02c43 * 2;
	var ref_39b91 = cos(ref_7267f);
	var ref_2ef02 = 8 * ref_857d0;
	var ref_cc562 = ref_039cf * 4;
	var ref_2377b = ref_34532 * 15;
	var ref_62ac7 = ref_2377b - ref_cc562;
	var ref_bee97 = ref_dcc84 * ref_62ac7;
	var ref_a1a96 = 15 * ref_0c7dd;
	var ref_f1f41 = ref_39b91 * 5;
	var ref_b13a1 = ref_a1a96 - ref_f1f41;
	var ref_a26cd = ref_77e2e * 2;
	var ref_c39c6 = ref_b13a1 - ref_a26cd;
	var ref_17210 = ref_c39c6 - ref_39b91;
	var ref_ee416 = ref_17210 * ref_dcc84;
	var ref_2aa75 = ref_bee97 * 0.2;
	var ref_477a9 = ref_2ef02 * 0.2;
	var ref_dea4e = ref_ee416 * 0.2;
	var ref_648ae = vec3f(ref_2aa75, ref_477a9, ref_dea4e);
	return ref_648ae;
}
fn Sphere(input2: vec2f) -> vec3f {
	var PI = 3.14159265359;
	var HALF_PI = 3.14159265359 / 2.0;
	var TWO_PI = 3.14159265359 * 2.0;
	var ref_c66fd_1 = input2[0];
	var ref_c66fd_2 = input2[1];
	var ref_7f8d2 = ref_c66fd_1 * PI;
	var ref_714e6 = ref_c66fd_2 * TWO_PI;
	var ref_1f27b = sin(ref_7f8d2);
	var ref_4adfe = sin(ref_714e6);
	var ref_b7bb4 = cos(ref_7f8d2);
	var ref_bc4e8 = cos(ref_714e6);
	var ref_d22cb = ref_1f27b * ref_bc4e8;
	var ref_caaf2 = ref_1f27b * ref_4adfe;
	var ref_4c40c = ref_d22cb * 3;
	var ref_0c498 = ref_caaf2 * 3;
	var ref_e92df = ref_b7bb4 * 3;
	var ref_d1aa9 = vec3f(ref_4c40c, ref_0c498, ref_e92df);
	return ref_d1aa9;
}


fn sampleObject(input2: vec2f) -> vec3f {
	var PI = 3.14159265359;
	var HALF_PI = 3.14159265359 / 2.0;
	var TWO_PI = 3.14159265359 * 2.0;
	var ref_1643b = Heart(input2);
	var ref_1a736 = Sphere(input2);
	var ref_2d4fd = 0.20000000000000001110;
	var ref_a114d = mix(ref_1643b, ref_1a736, ref_2d4fd);
	return ref_a114d;

}