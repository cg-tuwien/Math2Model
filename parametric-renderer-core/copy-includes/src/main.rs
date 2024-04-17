use copy_includes::copy_includes;

fn main() {
    copy_includes("./shaders/ComputePatches.wgsl").unwrap();
    copy_includes("./shaders/CopyPatches.wgsl").unwrap();
    copy_includes("./shaders/Shader.wgsl").unwrap();
}
