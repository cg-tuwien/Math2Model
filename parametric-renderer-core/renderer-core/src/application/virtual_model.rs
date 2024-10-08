// This is the simplest design, where each virtual model has its own set of resources.

use crate::{
    buffer::TypedBuffer,
    camera::Camera,
    mesh::Mesh,
    shaders::{compute_patches, copy_patches, shader},
    texture::Texture,
    transform::Transform,
};

use super::{wgpu_context::WgpuContext, MAX_PATCH_COUNT, PATCH_SIZES};
use std::collections::HashMap;

use glam::{Mat4, Vec3, Vec4};
use slotmap::{DefaultKey, SlotMap};

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct NamedShader {
    pub label: String,
    pub shader: String,
}

pub struct ShaderArena {
    shader_keys: HashMap<NamedShader, DefaultKey>,
    shaders: SlotMap<DefaultKey, ShaderPipelines>,
}

pub struct ShaderPipelines {
    /// Pipeline per model, for different parametric functions.
    pub compute_patches: wgpu::ComputePipeline,
    /// Pipeline per model, for different parametric functions.
    pub render: wgpu::RenderPipeline,
}

impl ShaderArena {
    pub fn new() -> Self {
        Self {
            shader_keys: HashMap::new(),
            shaders: SlotMap::new(),
        }
    }

    pub fn get_shader(&self, key: DefaultKey) -> &ShaderPipelines {
        &self.shaders[key]
    }

    pub fn get_or_create_shader(
        &mut self,
        label: &str,
        context: &WgpuContext,
        evaluate_image_code: &str,
    ) -> DefaultKey {
        let key = NamedShader {
            label: label.to_string(),
            shader: evaluate_image_code.to_owned(),
        };
        *self.shader_keys.entry(key).or_insert_with(|| {
            let compute_patches = create_compute_patches_pipeline(
                Some(label),
                &context.device,
                Some(evaluate_image_code),
            );
            let render = create_render_pipeline(Some(label), context, Some(evaluate_image_code));
            self.shaders.insert(ShaderPipelines {
                compute_patches,
                render,
            })
        })
    }

    pub fn retain(&mut self, used_keys: impl Iterator<Item = DefaultKey>) {
        let keys_set: std::collections::HashSet<_> = used_keys.collect();
        self.shader_keys.retain(|_, key| keys_set.contains(key));
        self.shaders.retain(|key, _| keys_set.contains(&key));
    }
}

pub struct ComputePatchesStep {
    pub input_buffer: TypedBuffer<compute_patches::InputBuffer>,
    pub patches_buffer: [TypedBuffer<compute_patches::Patches>; 2],
    pub render_buffer: Vec<TypedBuffer<compute_patches::RenderBuffer>>,
    pub indirect_compute_buffer: [TypedBuffer<compute_patches::DispatchIndirectArgs>; 2],
    pub bind_group_1: compute_patches::bind_groups::BindGroup1,
    pub bind_group_2: [compute_patches::bind_groups::BindGroup2; 2],
    pub force_render_uniform: TypedBuffer<compute_patches::ForceRenderFlag>,
}

impl ComputePatchesStep {
    pub fn new(device: &wgpu::Device) -> anyhow::Result<Self> {
        let input_buffer = TypedBuffer::new_uniform(
            device,
            "Input Buffer",
            &compute_patches::InputBuffer {
                model_view_projection: Mat4::IDENTITY,
                threshold_factor: 1.0,
            },
            wgpu::BufferUsages::COPY_DST,
        )?;

        let patches_buffer_empty = compute_patches::Patches {
            patches_length: 0,
            patches_capacity: 0,
            patches: vec![],
        };
        let patches_buffer = [
            TypedBuffer::new_storage_with_runtime_array(
                device,
                "Patches Buffer 0",
                &patches_buffer_empty,
                MAX_PATCH_COUNT as u64,
                wgpu::BufferUsages::COPY_SRC | wgpu::BufferUsages::COPY_DST,
            )?,
            TypedBuffer::new_storage_with_runtime_array(
                device,
                "Patches Buffer 1",
                &patches_buffer_empty,
                MAX_PATCH_COUNT as u64,
                wgpu::BufferUsages::COPY_SRC | wgpu::BufferUsages::COPY_DST,
            )?,
        ];

        let render_buffer_initial = compute_patches::RenderBuffer {
            patches_length: 0,
            patches_capacity: 0,
            patches: vec![],
        };
        let render_buffer = PATCH_SIZES
            .iter()
            .map(|_| {
                TypedBuffer::new_storage_with_runtime_array(
                    device,
                    "Render Buffer",
                    &render_buffer_initial,
                    MAX_PATCH_COUNT as u64,
                    wgpu::BufferUsages::COPY_DST,
                )
            })
            .collect::<Result<Vec<_>, _>>()?;

        let indirect_compute_empty = compute_patches::DispatchIndirectArgs { x: 0, y: 0, z: 0 };
        let indirect_compute_buffer = [
            TypedBuffer::new_storage(
                device,
                "Indirect Compute Dispatch Buffer 0",
                &indirect_compute_empty,
                wgpu::BufferUsages::INDIRECT | wgpu::BufferUsages::COPY_DST,
            )?,
            TypedBuffer::new_storage(
                device,
                "Indirect Compute Dispatch Buffer 1",
                &indirect_compute_empty,
                wgpu::BufferUsages::INDIRECT | wgpu::BufferUsages::COPY_DST,
            )?,
        ];

        let force_render_uniform = TypedBuffer::new_uniform(
            device,
            "Force Render Uniform",
            &compute_patches::ForceRenderFlag { flag: 0 },
            wgpu::BufferUsages::COPY_DST,
        )?;

        let bind_group_1 = compute_patches::bind_groups::BindGroup1::from_bindings(
            device,
            compute_patches::bind_groups::BindGroupLayout1 {
                input_buffer: input_buffer.as_entire_buffer_binding(),
                render_buffer_2: render_buffer[0].as_entire_buffer_binding(),
                render_buffer_4: render_buffer[1].as_entire_buffer_binding(),
                render_buffer_8: render_buffer[2].as_entire_buffer_binding(),
                render_buffer_16: render_buffer[3].as_entire_buffer_binding(),
                render_buffer_32: render_buffer[4].as_entire_buffer_binding(),
            },
        );
        let bind_group_2 = [
            compute_patches::bind_groups::BindGroup2::from_bindings(
                device,
                compute_patches::bind_groups::BindGroupLayout2 {
                    patches_from_buffer: patches_buffer[0].as_entire_buffer_binding(),
                    patches_to_buffer: patches_buffer[1].as_entire_buffer_binding(),
                    dispatch_next: indirect_compute_buffer[1].as_entire_buffer_binding(),
                    force_render: force_render_uniform.as_entire_buffer_binding(),
                },
            ),
            compute_patches::bind_groups::BindGroup2::from_bindings(
                device,
                compute_patches::bind_groups::BindGroupLayout2 {
                    patches_from_buffer: patches_buffer[1].as_entire_buffer_binding(), // Swap the order :)
                    patches_to_buffer: patches_buffer[0].as_entire_buffer_binding(),
                    dispatch_next: indirect_compute_buffer[0].as_entire_buffer_binding(),
                    force_render: force_render_uniform.as_entire_buffer_binding(),
                },
            ),
        ];

        Ok(Self {
            input_buffer,
            patches_buffer,
            render_buffer,
            indirect_compute_buffer,
            bind_group_1,
            bind_group_2,
            force_render_uniform,
        })
    }
}

pub struct CopyPatchesStep {
    pub bind_group_0: copy_patches::bind_groups::BindGroup0,
    pub indirect_draw_buffers: TypedBuffer<copy_patches::DrawIndexedBuffers>,
}

impl CopyPatchesStep {
    pub fn new(
        device: &wgpu::Device,
        compute_patches: &ComputePatchesStep,
        meshes: &[crate::mesh::Mesh],
    ) -> anyhow::Result<Self> {
        let indirect_draw_data = meshes
            .iter()
            .map(|mesh| copy_patches::DrawIndexedIndirectArgs {
                index_count: mesh.num_indices,
                instance_count: 0, // Our shader sets this
                first_index: 0,
                base_vertex: 0,
                first_instance: 0,
            })
            .collect::<Vec<_>>();

        let indirect_draw_buffers = TypedBuffer::new_storage(
            device,
            "Indirect Draw Buffers",
            &copy_patches::DrawIndexedBuffers {
                indirect_draw_2: indirect_draw_data[0],
                indirect_draw_4: indirect_draw_data[1],
                indirect_draw_8: indirect_draw_data[2],
                indirect_draw_16: indirect_draw_data[3],
                indirect_draw_32: indirect_draw_data[4],
            },
            wgpu::BufferUsages::INDIRECT | wgpu::BufferUsages::COPY_SRC,
        )?;

        let bind_group_0 = copy_patches::bind_groups::BindGroup0::from_bindings(
            device,
            copy_patches::bind_groups::BindGroupLayout0 {
                render_buffer_2: compute_patches.render_buffer[0].as_entire_buffer_binding(),
                render_buffer_4: compute_patches.render_buffer[1].as_entire_buffer_binding(),
                render_buffer_8: compute_patches.render_buffer[2].as_entire_buffer_binding(),
                render_buffer_16: compute_patches.render_buffer[3].as_entire_buffer_binding(),
                render_buffer_32: compute_patches.render_buffer[4].as_entire_buffer_binding(),
                indirect_draw: indirect_draw_buffers.as_entire_buffer_binding(),
            },
        );

        Ok(Self {
            bind_group_0,
            indirect_draw_buffers,
        })
    }
}

pub struct RenderStep {
    pub model_buffer: TypedBuffer<shader::Model>,
    pub material_buffer: TypedBuffer<shader::Material>,
    pub bind_group_1: Vec<shader::bind_groups::BindGroup1>,
}

impl RenderStep {
    pub fn new(
        context: &WgpuContext,
        compute_patches: &ComputePatchesStep,
    ) -> anyhow::Result<Self> {
        let model_buffer = TypedBuffer::new_uniform(
            &context.device,
            "Model Buffer",
            &shader::Model {
                model_similarity: Mat4::IDENTITY,
            },
            wgpu::BufferUsages::COPY_DST,
        )?;

        let material_buffer = TypedBuffer::new_uniform(
            &context.device,
            "Material Buffer",
            &MaterialInfo::missing().to_shader(),
            wgpu::BufferUsages::COPY_DST,
        )?;

        let bind_group_1 = compute_patches
            .render_buffer
            .iter()
            .map(|render| {
                shader::bind_groups::BindGroup1::from_bindings(
                    &context.device,
                    shader::bind_groups::BindGroupLayout1 {
                        model: model_buffer.as_entire_buffer_binding(),
                        render_buffer: render.as_entire_buffer_binding(),
                        material: material_buffer.as_entire_buffer_binding(),
                    },
                )
            })
            .collect();

        Ok(Self {
            model_buffer,
            material_buffer,
            bind_group_1,
        })
    }
}

pub struct VirtualModel {
    pub compute_patches: ComputePatchesStep,
    pub copy_patches: CopyPatchesStep,
    pub render_step: RenderStep,

    pub shaders_key: DefaultKey,
    pub transform: Transform,
    pub material_info: MaterialInfo,
}

#[derive(Debug, Clone)]
pub struct MaterialInfo {
    pub color: Vec3,
    pub emissive: Vec3,
    pub roughness: f32,
    pub metallic: f32,
}
impl MaterialInfo {
    pub fn to_shader(&self) -> shader::Material {
        shader::Material {
            color_roughness: Vec4::new(self.color.x, self.color.y, self.color.z, self.roughness),
            emissive_metallic: Vec4::new(
                self.emissive.x,
                self.emissive.y,
                self.emissive.z,
                self.metallic,
            ),
        }
    }

    fn missing() -> Self {
        Self {
            color: Vec3::new(1.0, 0.0, 1.0),
            emissive: Vec3::new(1.0, 0.0, 1.0),
            roughness: 0.7,
            metallic: 0.0,
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
        }
    }
}

impl VirtualModel {
    pub fn new(
        context: &WgpuContext,
        meshes: &[Mesh],
        shaders_key: DefaultKey,
    ) -> anyhow::Result<Self> {
        let compute_patches_step = ComputePatchesStep::new(&context.device)?;
        let copy_patches_step =
            CopyPatchesStep::new(&context.device, &compute_patches_step, meshes)?;
        let render_step = RenderStep::new(context, &compute_patches_step)?;

        Ok(Self {
            compute_patches: compute_patches_step,
            copy_patches: copy_patches_step,
            render_step,
            shaders_key,
            transform: Transform::default(),
            material_info: MaterialInfo::default(),
        })
    }

    pub fn get_model_matrix(&self) -> Mat4 {
        self.transform.to_matrix()
    }

    pub fn get_model_view_projection(&self, camera: &Camera) -> Mat4 {
        camera.projection_matrix() * camera.view_matrix() * self.transform.to_matrix()
    }
}

fn create_render_pipeline(
    label: Option<&str>,
    context: &WgpuContext,
    evaluate_image_code: Option<&str>,
) -> wgpu::RenderPipeline {
    let label = label.unwrap_or_default();
    let device = &context.device;
    let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some(&format!("Render Shader {}", label)),
        source: wgpu::ShaderSource::Wgsl(replace_evaluate_image_code(
            shader::SOURCE,
            evaluate_image_code,
        )),
    });
    device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some(&format!("Render Pipeline {}", label)),
        layout: Some(&shader::create_pipeline_layout(device)),
        vertex: shader::vertex_state(
            &shader,
            &shader::vs_main_entry(wgpu::VertexStepMode::Vertex),
        ),
        fragment: Some(wgpu::FragmentState {
            module: &shader,
            entry_point: shader::ENTRY_FS_MAIN,
            targets: &[Some(wgpu::ColorTargetState {
                format: context.view_format,
                blend: Some(wgpu::BlendState::REPLACE),
                write_mask: wgpu::ColorWrites::ALL,
            })],
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
            depth_compare: wgpu::CompareFunction::Greater,
            stencil: Default::default(),
            bias: Default::default(),
        }),
        multisample: Default::default(),
        multiview: None,
        cache: Default::default(),
    })
}

fn create_compute_patches_pipeline(
    label: Option<&str>,
    device: &wgpu::Device,
    evaluate_image_code: Option<&str>,
) -> wgpu::ComputePipeline {
    let label = label.unwrap_or_default();
    let source = replace_evaluate_image_code(compute_patches::SOURCE, evaluate_image_code);

    device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
        label: Some(&format!("Compute Patches {}", label)),
        layout: Some(&compute_patches::create_pipeline_layout(device)),
        module: &device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: None,
            source: wgpu::ShaderSource::Wgsl(std::borrow::Cow::Borrowed(source.as_ref())),
        }),
        entry_point: compute_patches::ENTRY_MAIN,
        compilation_options: Default::default(),
        cache: Default::default(),
    })
}

fn replace_evaluate_image_code<'a>(
    source: &'a str,
    evaluate_image_code: Option<&str>,
) -> std::borrow::Cow<'a, str> {
    // TODO: use wgsl-parser instead of this
    if let Some(evaluate_image_code) = evaluate_image_code {
        let start = source.find("//// START sampleObject").unwrap();
        let end = source.find("//// END sampleObject").unwrap();

        let mut result = String::with_capacity(
            source[..start].len() + evaluate_image_code.len() + source[end..].len(),
        );
        result.push_str(&source[..start]);
        result.push_str(evaluate_image_code);
        result.push_str(&source[end..]);
        std::borrow::Cow::Owned(result)
    } else {
        std::borrow::Cow::Borrowed(source)
    }
}
