use std::fmt::Write;
use wgsl_to_wgpu::{create_shader_module_embedded, MatrixVectorTypes, WriteOptions};

#[derive(Default)]
struct ShaderModule {
    name: String,
    path: String,
    module: String,
}

impl std::fmt::Display for ShaderModule {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "#[allow(dead_code)]")?;
        let snake_name = pascal_to_snake(&self.name);
        writeln!(f, "pub mod {snake_name} {}", '{')?;
        let shader_path = RustStrLiteral(&self.path);
        writeln!(f, "    pub const SOURCE_PATH: &str = {shader_path};")?;
        writeln!(
            f,
            "include!(concat!(env!(\"OUT_DIR\"), \"/shaders/{}.rs\"));",
            self.name
        )?;
        writeln!(f, "{}", '}')?;

        Ok(())
    }
}

// src/build.rs
fn main() {
    let mut shaders = vec![];
    shaders.push(watch_shader("../shaders/Shader.wgsl", "Shader"));
    shaders.push(watch_shader(
        "../shaders/ComputePatches.wgsl",
        "ComputePatches",
    ));
    shaders.push(watch_shader("../shaders/CopyPatches.wgsl", "CopyPatches"));
    shaders.push(watch_shader("../shaders/GroundPlane.wgsl", "GroundPlane"));

    let mut text = String::new();
    writeln!(&mut text, "// File automatically generated by build.rs.").unwrap();
    writeln!(&mut text, "// Changes made to this file will not be saved.").unwrap();

    let out_dir = std::env::var("OUT_DIR").unwrap();
    std::fs::write(
        format!("{out_dir}/shaders.rs"),
        shaders
            .into_iter()
            .map(|s| format!("{s}"))
            .collect::<String>()
            .as_bytes(),
    )
    .unwrap();
}

fn pascal_to_snake(input: &str) -> String {
    let mut chars = input.chars();
    let mut result = chars
        .next()
        .map(|c| c.to_lowercase().collect::<String>())
        .unwrap_or_default();
    for c in chars {
        if c.is_uppercase() {
            result.push('_');
            for c in c.to_lowercase().into_iter() {
                result.push(c);
            }
        } else {
            result.push(c);
        }
    }
    result
}

struct RustStrLiteral<'a>(&'a str);
impl std::fmt::Display for RustStrLiteral<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut max_hash_count = 0;
        let mut chars = self.0.chars();
        while let Some(_) = chars.find(|c| *c == '#') {
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
        },
    )
    .unwrap();
    ShaderModule {
        name: output_name.to_string(),
        path: path.to_string(),
        module,
    }
}
