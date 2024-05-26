#[allow(dead_code)]
pub mod shader {
    include!(concat!(env!("OUT_DIR"), "/Shader.rs"));
}

#[allow(dead_code)]
pub mod compute_patches {
    include!(concat!(env!("OUT_DIR"), "/ComputePatches.rs"));
}

#[allow(dead_code)]
pub mod copy_patches {
    include!(concat!(env!("OUT_DIR"), "/CopyPatches.rs"));
}
