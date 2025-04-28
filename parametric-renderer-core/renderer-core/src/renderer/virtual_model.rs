// This is the simplest design, where each virtual model has its own set of resources.

use crate::{
    buffer::TypedBuffer,
    game::{MaterialInfo, TextureData, TextureInfo},
    mesh::Mesh,
    shaders::{compute_patches, copy_patches, shader},
    texture::Texture,
};

use super::{MAX_PATCH_COUNT, PATCH_SIZES, wgpu_context::WgpuContext};
use std::sync::Arc;

use glam::{Vec2, Vec3, Vec4};
use uuid::Uuid;
use wgpu::ShaderModule;

pub struct ShaderPipelines {
    /// Pipeline per model, for different parametric functions.
    pub compute_patches: wgpu::ComputePipeline,
    /// Pipeline per model, for different parametric functions.
    pub render: wgpu::RenderPipeline,
    pub shaders: [ShaderModule; 2],
    pub id: Uuid,
}

impl PartialEq for ShaderPipelines {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl Eq for ShaderPipelines {}

impl ShaderPipelines {
    pub fn new(label: &str, code: &str, context: &WgpuContext) -> Self {
        let (compute_patches, shader_a) =
            create_compute_patches_pipeline(label, &context.device, code);
        let (render, shader_b) = create_render_pipeline(label, context, code);

        Self {
            compute_patches,
            render,
            shaders: [shader_a, shader_b],
            id: Uuid::new_v4(),
        }
    }

    pub async fn get_compilation_info(&self) -> Vec<wgpu::CompilationMessage> {
        let mut messages = self.shaders[0].get_compilation_info().await.messages;
        messages.extend(self.shaders[1].get_compilation_info().await.messages);
        messages
    }
}

const MISSING_SHADER: &str = include_str!("../../../shaders/DefaultParametric.wgsl");

pub fn make_missing_shader(context: &WgpuContext) -> Arc<ShaderPipelines> {
    Arc::new(ShaderPipelines::new(
        "Missing Shader",
        MISSING_SHADER,
        context,
    ))
}

pub fn make_empty_texture(context: &WgpuContext) -> Arc<Texture> {
    Arc::new(Texture::new_rgba(
        &context.device,
        &context.queue,
        &TextureInfo {
            width: 1,
            height: 1,
            data: TextureData::Bytes(vec![u8::MAX, u8::MAX, u8::MAX, u8::MAX]),
        },
    ))
}

// Minimal amount of info to pass from patches stage to render stage
pub struct VirtualModel {
    pub render_buffer: Vec<TypedBuffer<compute_patches::RenderBuffer>>,
    pub indirect_draw: TypedBuffer<Vec<copy_patches::DrawIndexedIndirectArgs>>,
}

impl VirtualModel {
    pub fn new(context: &WgpuContext, meshes: &[Mesh], id: &str) -> Self {
        let render_buffer_initial = compute_patches::RenderBuffer {
            patches_length: 0,
            patches_capacity: 0,
            patches: vec![],
        };
        let render_buffer = PATCH_SIZES
            .iter()
            .map(|size| {
                TypedBuffer::new_storage_with_runtime_array(
                    &context.device,
                    &format!("{id} Render Buffer {size}"),
                    &render_buffer_initial,
                    MAX_PATCH_COUNT as u64,
                    wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::COPY_SRC,
                )
            })
            .collect::<Vec<_>>();

        let indirect_draw_data = copy_patches::DrawIndexedIndirectArgs {
            index_count: 0,
            instance_count: 0, // Our shader sets this
            first_index: 0,
            base_vertex: 0,
            first_instance: 0,
        };

        let indirect_draw = TypedBuffer::new_storage(
            &context.device,
            &format!("{} Indirect Draw Buffers", id),
            &meshes
                .iter()
                .map(|mesh| copy_patches::DrawIndexedIndirectArgs {
                    index_count: mesh.num_indices,
                    ..indirect_draw_data
                })
                .collect::<Vec<_>>(),
            wgpu::BufferUsages::INDIRECT | wgpu::BufferUsages::COPY_SRC,
        );

        Self {
            render_buffer,
            indirect_draw,
        }
    }
}

impl MaterialInfo {
    pub fn to_shader(&self) -> shader::Material {
        shader::Material {
            color_roughness: Vec4::new(self.color.x, self.color.y, self.color.z, self.roughness),
            emissive_metallic: self.emissive.extend(self.metallic),
            has_texture: if self.diffuse_texture.is_some() { 1 } else { 0 },
            texture_scale: self.texture_scale,
        }
    }

    pub fn missing() -> Self {
        Self {
            color: Vec3::new(1.0, 0.0, 1.0),
            emissive: Vec3::new(1.0, 0.0, 1.0),
            roughness: 0.7,
            metallic: 0.0,
            diffuse_texture: None,
            texture_scale: Vec2::ONE,
        }
    }
}
impl Default for MaterialInfo {
    fn default() -> Self {
        Self {
            color: Vec3::new(0.0, 0.0, 0.0),
            emissive: Vec3::new(0.0, 0.0, 0.0),
            roughness: 0.0,
            metallic: 0.0,
            diffuse_texture: None,
            texture_scale: Vec2::ONE,
        }
    }
}

fn create_render_pipeline(
    label: &str,
    context: &WgpuContext,
    code: &str,
) -> (wgpu::RenderPipeline, ShaderModule) {
    let device = &context.device;
    let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some(&format!("Render Shader {}", label)),
        source: wgpu::ShaderSource::Wgsl(replace_render_code(shader::SOURCE, code).into()),
    });
    (
        device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some(&format!("Render Pipeline {}", label)),
            layout: Some(&shader::create_pipeline_layout(device)),
            vertex: shader::vertex_state(
                &shader,
                &shader::vs_main_entry(wgpu::VertexStepMode::Vertex),
            ),
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some(shader::ENTRY_FS_MAIN),
                targets: &[
                    Some(wgpu::ColorTargetState {
                        format: context.view_format,
                        blend: Some(wgpu::BlendState::REPLACE),
                        write_mask: wgpu::ColorWrites::ALL,
                    }),
                    Some(wgpu::ColorTargetState {
                        format: wgpu::TextureFormat::R32Uint,
                        blend: None,
                        write_mask: wgpu::ColorWrites::empty(),
                    }),
                ],
                compilation_options: Default::default(),
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: None,
                // Setting this to anything other than Fill requires Features::NON_FILL_POLYGON_MODE
                polygon_mode: wgpu::PolygonMode::Fill, // Wireframe mode can be toggled here on the desktop backend
                // Requires Features::DEPTH_CLIP_CONTROL
                unclipped_depth: false,
                // Requires Features::CONSERVATIVE_RASTERIZATION
                conservative: false,
            },
            depth_stencil: Some(wgpu::DepthStencilState {
                format: Texture::DEPTH_FORMAT,
                depth_write_enabled: true,
                depth_compare: wgpu::CompareFunction::Greater, // Reverse Z
                stencil: Default::default(),
                bias: Default::default(),
            }),
            multisample: Default::default(),
            multiview: None,
            cache: Default::default(),
        }),
        shader,
    )
}

pub fn create_compute_patches_pipeline(
    label: &str,
    device: &wgpu::Device,
    code: &str,
) -> (wgpu::ComputePipeline, ShaderModule) {
    let source = replace_compute_code(compute_patches::SOURCE, code);
    let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: None,
        source: wgpu::ShaderSource::Wgsl(std::borrow::Cow::Borrowed(source.as_ref())),
    });
    (
        device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: Some(&format!("Compute Patches {}", label)),
            layout: Some(&compute_patches::create_pipeline_layout(device)),
            module: &shader,
            entry_point: Some(compute_patches::ENTRY_MAIN),
            compilation_options: Default::default(),
            cache: Default::default(),
        }),
        shader,
    )
}

fn replace_render_code<'a>(source: &'a str, sample_object_code: &str) -> String {
    // TODO: use wgsl-parser instead of this
    let start_1 = source.find("//// START sampleObject").unwrap();
    let end_1 = source.find("//// END sampleObject").unwrap();
    let start_2 = source.find("//// START getColor").unwrap();
    let end_2 = source.find("//// END getColor").unwrap();

    let mut result = String::new();
    result.push_str(&source[..start_1]);
    result.push_str(sample_object_code);
    result.push_str(&source[end_1..start_2]);

    if sample_object_code.contains("fn getColor") {
        result.push_str(&source[end_2..]);
    } else {
        result.push_str(&source[start_2..]);
    }

    result
}

fn replace_compute_code<'a>(source: &'a str, sample_object_code: &str) -> String {
    // TODO: use wgsl-parser instead of this
    let start_1 = source.find("//// START sampleObject").unwrap();
    let end_1 = source.find("//// END sampleObject").unwrap();

    let mut result = String::new();
    result.push_str(&source[..start_1]);
    if sample_object_code.contains("fn getColor") {
        let get_color = sample_object_code.find("fn getColor").unwrap();
        result.push_str(&sample_object_code[0..get_color]);
    } else {
        result.push_str(sample_object_code);
    }
    result.push_str(&source[end_1..]);
    result
}
