

fn evaluateImage(input2: vec2f) -> vec3f {
	var PI = 3.14159265359;
	var HALF_PI = 3.14159265359 / 2.0;
	var TWO_PI = 3.14159265359 * 2.0;
	var ref_b2d6f_1 = input2[0];
	var ref_b2d6f_2 = input2[1];
	var ref_a9cf4 = ref_b2d6f_1 * PI;
	var ref_22362 = ref_b2d6f_2 * TWO_PI;
	var ref_842a5 = sin(ref_22362);
	var ref_feaf7 = sin(ref_a9cf4);
	var ref_d05ea = cos(ref_a9cf4);
	var ref_f2fff = cos(ref_22362);
	var ref_f171c = ref_22362 * 3;
	var ref_09d6d = sin(ref_f171c);
	var ref_0cfa3 = cos(ref_f171c);
	var ref_a8f2f = ref_22362 * 2;
	var ref_e6990 = cos(ref_a8f2f);
	var ref_cfc2d = 8 * ref_d05ea;
	var ref_f20f2 = ref_09d6d * 4;
	var ref_f00b4 = ref_842a5 * 15;
	var ref_b8cce = ref_f00b4 - ref_f20f2;
	var ref_02aa1 = ref_feaf7 * ref_b8cce;
	var ref_10d8e = 15 * ref_f2fff;
	var ref_21bc5 = ref_e6990 * 5;
	var ref_3207d = ref_10d8e - ref_21bc5;
	var ref_c6a22 = ref_0cfa3 * 2;
	var ref_db23e = ref_3207d - ref_c6a22;
	var ref_531f6 = ref_db23e - ref_e6990;
	var ref_72cdf = ref_531f6 * ref_feaf7;
	var ref_f46b6 = ref_02aa1 * 0.2;
	var ref_0df37 = ref_cfc2d * 0.2;
	var ref_74ab3 = ref_72cdf * 0.2;
	var ref_ff707 = vec3f(ref_f46b6, ref_0df37, ref_74ab3);
	return ref_ff707;

}