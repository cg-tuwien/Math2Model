use std::fmt::Write;
use wgsl_to_wgpu::{MatrixVectorTypes, WriteOptions, create_shader_module_embedded};

#[derive(Default)]
struct ShaderModule {
    name: String,
    path: String,
    module: String,
}

impl std::fmt::Display for ShaderModule {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "#[allow(dead_code)]")?;
        writeln!(f, "pub mod {} {{", &self.name)?;
        let shader_path = RustStrLiteral(&self.path);
        writeln!(f, "pub const SOURCE_PATH: &str = {shader_path};")?;
        writeln!(f, "{}", self.module)?;
        writeln!(f, "}}")?;

        Ok(())
    }
}

// src/build.rs
fn main() {
    let start_time = std::time::Instant::now();
    let mut shaders = vec![];
    shaders.push(watch_shader("../shaders/Shader.wgsl", "shader"));
    shaders.push(watch_shader(
        "../shaders/ComputePatches.wgsl",
        "compute_patches",
    ));
    shaders.push(watch_shader("../shaders/CopyPatches.wgsl", "copy_patches"));
    shaders.push(watch_shader("../shaders/GroundPlane.wgsl", "ground_plane"));
    shaders.push(watch_shader("../shaders/Skybox.wgsl", "skybox"));

    let mut text = String::new();
    writeln!(&mut text, "// File automatically generated by build.rs.").unwrap();
    writeln!(&mut text, "// Changes made to this file will not be saved.").unwrap();
    for shader in shaders.into_iter() {
        writeln!(&mut text, "{shader}").unwrap();
    }

    let out_dir = std::env::var("OUT_DIR").unwrap();
    std::fs::write(format!("{out_dir}/shaders.rs"), text.as_bytes()).unwrap();
    println!(
        "cargo::warning=Compiled shaders in {}ms",
        start_time.elapsed().as_millis()
    );
}

struct RustStrLiteral<'a>(&'a str);
impl std::fmt::Display for RustStrLiteral<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut max_hash_count = 0;
        let mut chars = self.0.chars();
        while chars.find(|c| *c == '#').is_some() {
            let count = 1 + chars.by_ref().take_while(|c| *c == '#').count();
            max_hash_count = max_hash_count.max(count);
        }

        write!(f, "r\"")?;
        for _ in 0..max_hash_count {
            write!(f, "#")?;
        }
        write!(f, "{}", self.0)?;
        for _ in 0..max_hash_count {
            write!(f, "#")?;
        }
        write!(f, "\"")?;
        Ok(())
    }
}

fn watch_shader(path: &str, output_name: &str) -> ShaderModule {
    println!("cargo:rerun-if-changed={path}");
    let wgsl_source = match std::fs::read_to_string(path) {
        Ok(source) => source,
        Err(err) => panic!("Failed to read shader file {}: {}", path, err),
    };

    let module = create_shader_module_embedded(
        &wgsl_source,
        WriteOptions {
            // We need to use bytemuck for vertex buffer
            derive_bytemuck_vertex: true,
            derive_bytemuck_host_shareable: false,
            // And encase for uniform buffers and storage buffers
            derive_encase_host_shareable: true,
            derive_serde: false,
            matrix_vector_types: MatrixVectorTypes::Glam,
            rustfmt: false,
            validate: None,
        },
    )
    .unwrap();
    ShaderModule {
        name: output_name.to_string(),
        path: path.to_string(),
        module,
    }
}
