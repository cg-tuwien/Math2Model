use std::fmt::Write;
use wgsl_to_wgpu::{create_shader_module_embedded, MatrixVectorTypes, WriteOptions};

// src/build.rs
fn main() {
    println!("cargo:rerun-if-changed=src/shader.wgsl");
    let wgsl_source = std::fs::read_to_string("src/shader.wgsl").unwrap();

    // Generate the Rust bindings and write to a file.
    let mut text = String::new();
    writeln!(&mut text, "// File automatically generated by build.rs.").unwrap();
    writeln!(&mut text, "// Changes made to this file will not be saved.").unwrap();
    text += &create_shader_module_embedded(
        &wgsl_source,
        WriteOptions {
            derive_bytemuck: true,
            derive_encase: false,
            derive_serde: false,
            matrix_vector_types: MatrixVectorTypes::Glam,
        },
    )
    .unwrap();
    let out_dir = std::env::var("OUT_DIR").unwrap();
    std::fs::write(format!("{out_dir}/shader.rs"), text.as_bytes()).unwrap();
}
