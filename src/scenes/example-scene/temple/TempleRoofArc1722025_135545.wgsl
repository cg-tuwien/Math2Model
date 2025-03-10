fn sampleObject(input2: vec2f) -> vec3f {
    let pos = vec3(input2.x, 0.0, input2.y) * 3.14159;

    var x = pos.x;
    var y = sign(sin(input2.x * 3 * 3.14159 * 100));
    var z = pos.z;
    
    let arcStrength = 0.5;

    let numColumnsX = 2;
    let numColumnsZ = 2;

    let arcWidth = 2.0; 
    var arcParam = arcWidth*arcWidth - z*z;
    arcParam *= arcStrength*arcStrength;
    // Input has height=4, arc has height=2 => make it:
    if (arcParam > 0.0 && y < 0.0) {
        y += sqrt(arcParam);
    }
    // Scale the top part of the arced box a bit smaller:
    if (y > 0.0) {
        y *= 0.5;
    }
    // Now put the arc on top of the columns:
    y += 9.0;

    // Scale the sides to the side:
    let columnSpacing = 3.0;
    if (abs(z) > 2.0 && numColumnsX > 2) {
        z += f32((numColumnsX - 2)/2) * columnSpacing * sign(z);
    }

    x = x * 0.7 + columnSpacing*0.5;
    if (abs(x) < 2.0 && numColumnsZ > 2) {
        x -= f32((numColumnsZ - 2)) * columnSpacing;
    }

    if (abs(x) > 3.0) {
        // x = 0.0;
    }

    let p = vec3f(x * 1.5, y, z);
    return p;   
}