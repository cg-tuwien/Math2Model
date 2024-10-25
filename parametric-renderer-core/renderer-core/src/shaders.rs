#[allow(dead_code)]
pub mod shader {
    pub const SOURCE: &str = include_str!("../../shaders/Shader.wgsl");
    include!(concat!(env!("OUT_DIR"), "/Shader.rs"));
}

#[allow(dead_code)]
pub mod compute_patches {
    pub const SOURCE: &str = include_str!("../../shaders/ComputePatches.wgsl");
    include!(concat!(env!("OUT_DIR"), "/ComputePatches.rs"));
}

#[allow(dead_code)]
pub mod copy_patches {
    pub const SOURCE: &str = include_str!("../../shaders/CopyPatches.wgsl");
    include!(concat!(env!("OUT_DIR"), "/CopyPatches.rs"));
}

#[allow(dead_code)]
pub mod ground_plane_shader {
    pub const SOURCE: &str = include_str!("../../shaders/GroundPlane.wgsl");
    include!(concat!(env!("OUT_DIR"), "/GroundPlane.rs"));
}
