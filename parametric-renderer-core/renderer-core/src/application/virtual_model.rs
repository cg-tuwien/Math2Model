// This is the simplest design, where each virtual model has its own set of resources.

use glamour::{Matrix4, ToRaw, Vector3, Vector4};

use crate::{
    buffer::TypedBuffer,
    camera::Camera,
    mesh::Mesh,
    shaders::{compute_patches, copy_patches, shader},
    texture::Texture,
    transform::Transform,
};

use super::{wgpu_context::WgpuContext, MAX_PATCH_COUNT, PATCH_SIZES};

pub struct ComputePatchesStep {
    /// Pipeline per model, for different materials.
    /// The second one is for "force_render"
    pub pipeline: [wgpu::ComputePipeline; 2],
    pub input_buffer: TypedBuffer<compute_patches::InputBuffer>,
    pub patches_buffer: [TypedBuffer<compute_patches::Patches>; 2],
    pub render_buffer: Vec<TypedBuffer<compute_patches::RenderBuffer>>,
    pub indirect_compute_buffer: [TypedBuffer<compute_patches::DispatchIndirectArgs>; 2],
    pub bind_group_1: compute_patches::bind_groups::BindGroup1,
    pub bind_group_2: [compute_patches::bind_groups::BindGroup2; 2],
}

impl ComputePatchesStep {
    pub fn new(label: Option<&str>, device: &wgpu::Device) -> anyhow::Result<Self> {
        let pipeline = create_compute_patches_pipeline(label, device, None);
        let label = label.as_deref().unwrap_or_default();
        let input_buffer = TypedBuffer::new_uniform(
            device,
            &format!("Input Buffer {}", label),
            &compute_patches::InputBuffer {
                model_view_projection: Matrix4::<f32>::IDENTITY.to_raw(),
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
                &format!("Patches Buffer 0 {}", label),
                &patches_buffer_empty,
                MAX_PATCH_COUNT as u64,
                wgpu::BufferUsages::COPY_SRC | wgpu::BufferUsages::COPY_DST,
            )?,
            TypedBuffer::new_storage_with_runtime_array(
                device,
                &format!("Patches Buffer 1 {}", label),
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
            .map(|size| {
                TypedBuffer::new_storage_with_runtime_array(
                    device,
                    &format!("Render Buffer {}", size),
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
                &format!("Indirect Compute Dispatch Buffer 0 {}", label),
                &indirect_compute_empty,
                wgpu::BufferUsages::INDIRECT | wgpu::BufferUsages::COPY_DST,
            )?,
            TypedBuffer::new_storage(
                device,
                &format!("Indirect Compute Dispatch Buffer 1 {}", label),
                &indirect_compute_empty,
                wgpu::BufferUsages::INDIRECT | wgpu::BufferUsages::COPY_DST,
            )?,
        ];

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
                },
            ),
            compute_patches::bind_groups::BindGroup2::from_bindings(
                device,
                compute_patches::bind_groups::BindGroupLayout2 {
                    patches_from_buffer: patches_buffer[1].as_entire_buffer_binding(), // Swap the order :)
                    patches_to_buffer: patches_buffer[0].as_entire_buffer_binding(),
                    dispatch_next: indirect_compute_buffer[0].as_entire_buffer_binding(),
                },
            ),
        ];

        Ok(Self {
            pipeline,
            input_buffer,
            patches_buffer,
            render_buffer,
            indirect_compute_buffer,
            bind_group_1,
            bind_group_2,
        })
    }
}

pub struct CopyPatchesStep {
    pub bind_group_0: copy_patches::bind_groups::BindGroup0,
    pub indirect_draw_buffers: TypedBuffer<copy_patches::DrawIndexedBuffers>,
}

impl CopyPatchesStep {
    pub fn new(
        label: Option<&str>,
        device: &wgpu::Device,
        compute_patches: &ComputePatchesStep,
        meshes: &[crate::mesh::Mesh],
    ) -> anyhow::Result<Self> {
        let label = label.as_deref().unwrap_or_default();

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
            &format!("Indirect Draw Buffers {}", label),
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
    /// Pipeline per model, for different materials.
    pub pipeline: wgpu::RenderPipeline,
    pub model_buffer: TypedBuffer<shader::Model>,
    pub material_buffer: TypedBuffer<shader::Material>,
    pub bind_group_1: Vec<shader::bind_groups::BindGroup1>,
}

impl RenderStep {
    pub fn new(
        label: Option<&str>,
        context: &WgpuContext,
        compute_patches: &ComputePatchesStep,
    ) -> anyhow::Result<Self> {
        let pipeline = create_render_pipeline(label, context, None);

        let label = label.as_deref().unwrap_or_default();
        let model_buffer = TypedBuffer::new_uniform(
            &context.device,
            &format!("Model Buffer {}", label),
            &shader::Model {
                model_similarity: Matrix4::<f32>::IDENTITY.to_raw(),
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
            pipeline,
            model_buffer,
            material_buffer,
            bind_group_1,
        })
    }
}

pub struct VirtualModel {
    /// For debugging purposes, this will show up in RenderDoc
    label: Option<String>,
    pub compute_patches: ComputePatchesStep,
    pub copy_patches: CopyPatchesStep,
    pub render_step: RenderStep,

    pub transform: Transform,
    pub material_info: MaterialInfo,
}

#[derive(Debug, Clone)]
pub struct MaterialInfo {
    pub color: Vector3<f32>,
    pub emissive: Vector3<f32>,
    pub roughness: f32,
    pub metallic: f32,
}
impl MaterialInfo {
    pub fn to_shader(&self) -> shader::Material {
        shader::Material {
            color_roughness: Vector4::<f32>::new(
                self.color.x,
                self.color.y,
                self.color.z,
                self.roughness,
            )
            .to_raw(),
            emissive_metallic: Vector4::<f32>::new(
                self.emissive.x,
                self.emissive.y,
                self.emissive.z,
                self.metallic,
            )
            .to_raw(),
        }
    }

    fn missing() -> Self {
        Self {
            color: Vector3::new(1.0, 0.0, 1.0),
            emissive: Vector3::new(1.0, 0.0, 1.0),
            roughness: 0.7,
            metallic: 0.0,
        }
    }
}
impl Default for MaterialInfo {
    fn default() -> Self {
        Self {
            color: Vector3::new(0.0, 0.0, 0.0),
            emissive: Vector3::new(0.0, 0.0, 0.0),
            roughness: 0.0,
            metallic: 0.0,
        }
    }
}

impl VirtualModel {
    pub fn new(
        label: Option<String>,
        context: &WgpuContext,
        meshes: &[Mesh],
    ) -> anyhow::Result<Self> {
        let compute_patches_step = ComputePatchesStep::new(label.as_deref(), &context.device)?;
        let copy_patches_step = CopyPatchesStep::new(
            label.as_deref(),
            &context.device,
            &compute_patches_step,
            meshes,
        )?;
        let render_step = RenderStep::new(label.as_deref(), context, &compute_patches_step)?;

        Ok(Self {
            label,
            compute_patches: compute_patches_step,
            copy_patches: copy_patches_step,
            render_step,
            transform: Transform::default(),
            material_info: MaterialInfo::default(),
        })
    }

    pub fn get_model_matrix(&self) -> Matrix4<f32> {
        self.transform.to_matrix()
    }

    pub fn get_model_view_projection(&self, camera: &Camera) -> Matrix4<f32> {
        camera.projection_matrix() * camera.view_matrix() * self.transform.to_matrix()
    }

    pub fn update_code(&mut self, context: &WgpuContext, evaluate_image_code: Option<&str>) {
        self.compute_patches.pipeline = create_compute_patches_pipeline(
            self.label.as_deref(),
            &context.device,
            evaluate_image_code,
        );
        self.render_step.pipeline =
            create_render_pipeline(self.label.as_deref(), context, evaluate_image_code);
    }
}

fn create_render_pipeline(
    label: Option<&str>,
    context: &WgpuContext,
    evaluate_image_code: Option<&str>,
) -> wgpu::RenderPipeline {
    let label = label.as_deref().unwrap_or_default();
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
    })
}

fn create_compute_patches_pipeline(
    label: Option<&str>,
    device: &wgpu::Device,
    evaluate_image_code: Option<&str>,
) -> [wgpu::ComputePipeline; 2] {
    let label = label.as_deref().unwrap_or_default();
    [
        device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: Some(&format!("Compute Patches {}", label)),
            layout: Some(&compute_patches::create_pipeline_layout(device)),
            module: &device.create_shader_module(wgpu::ShaderModuleDescriptor {
                label: None,
                source: wgpu::ShaderSource::Wgsl(std::borrow::Cow::Borrowed(
                    compute_patches::SOURCE,
                )),
            }),
            entry_point: compute_patches::ENTRY_MAIN,
            compilation_options: Default::default(),
        }),
        device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: Some(&format!("Compute Patches Force Render {}", label)),
            layout: Some(&compute_patches::create_pipeline_layout(device)),
            module: &device.create_shader_module(wgpu::ShaderModuleDescriptor {
                label: None,
                source: wgpu::ShaderSource::Wgsl(replace_evaluate_image_code(
                    compute_patches::SOURCE,
                    evaluate_image_code,
                )),
            }),
            entry_point: compute_patches::ENTRY_MAIN,
            compilation_options: wgpu::PipelineCompilationOptions {
                constants: &compute_patches::OverrideConstants {
                    force_render: Some(true),
                }
                .constants(),
                ..Default::default()
            },
        }),
    ]
}

fn replace_evaluate_image_code<'a>(
    source: &'a str,
    evaluate_image_code: Option<&str>,
) -> std::borrow::Cow<'a, str> {
    // TODO: use wgsl-parser instead of this
    if let Some(evaluate_image_code) = evaluate_image_code {
        let start = source.find("//// START evaluateImage").unwrap_or_default();
        let end = source.find("//// END evaluateImage").unwrap_or_default();

        let mut result = String::with_capacity(
            &source[..start].len() + evaluate_image_code.len() + &source[end..].len(),
        );
        result.push_str(&source[..start]);
        result.push_str(&evaluate_image_code);
        result.push_str(&source[end..]);
        std::borrow::Cow::Owned(result)
    } else {
        std::borrow::Cow::Borrowed(source)
    }
}
