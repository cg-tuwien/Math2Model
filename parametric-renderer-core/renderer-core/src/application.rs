use std::sync::Arc;

use glamour::{Matrix4, Point3, ToRaw, Vector2, Vector4};
use tracing::info;
use wgpu_profiler::{GpuProfiler, GpuProfilerSettings};
use winit::{dpi::PhysicalPosition, window::Window};
use winit_input_helper::WinitInputHelper;

use crate::{
    buffer::TypedBuffer,
    camera::{
        camera_controller::{
            CameraController, ChosenKind, CursorCapture, GeneralController,
            GeneralControllerSettings,
        },
        camera_settings::CameraSettings,
        Camera,
    },
    mesh::Mesh,
    shaders::{compute_patches, copy_patches, shader},
    texture::Texture,
};

#[derive(Debug, Clone, Default)]
pub struct ProfilerSettings {
    pub gpu: bool,
}

pub struct CpuApplication {
    pub gpu: Option<GpuApplication>,
    camera: Camera,
    pub camera_controller: CameraController,
    pub delta_time: f32,
    profiler_settings: ProfilerSettings,
}

impl CpuApplication {
    pub fn new() -> anyhow::Result<Self> {
        let camera = Camera::new(1.0, CameraSettings::default());
        let camera_controller = CameraController::new(
            GeneralController {
                position: Point3::new(0.0, 0.0, 4.0),
                orientation: glam::Quat::IDENTITY,
                distance_to_center: 4.0,
            },
            GeneralControllerSettings {
                fly_speed: 5.0,
                pan_speed: 1.0,
                rotation_sensitivity: 0.01,
            },
            ChosenKind::Freecam,
        );

        Ok(Self {
            gpu: None,
            camera,
            camera_controller,
            delta_time: 0.0,
            profiler_settings: ProfilerSettings::default(),
        })
    }

    pub fn get_profiling(&self) -> ProfilerSettings {
        self.profiler_settings.clone()
    }

    pub fn set_profiling(&mut self, new_settings: ProfilerSettings) {
        self.profiler_settings = new_settings;
        if let Some(gpu) = &mut self.gpu {
            gpu.context
                .profiler
                .change_settings(GpuProfilerSettings {
                    enable_timer_queries: self.profiler_settings.gpu,
                    ..GpuProfilerSettings::default()
                })
                .unwrap();
        }
    }

    pub async fn create_surface(&mut self, window: Window) -> anyhow::Result<()> {
        if self.gpu.is_some() {
            return Ok(());
        }
        let size = window.inner_size();
        self.camera
            .update_aspect_ratio(size.width as f32 / size.height as f32);
        let gpu = GpuApplication::new(window, &self.camera, &self.profiler_settings).await?;
        self.gpu = Some(gpu);
        self.set_profiling(self.profiler_settings.clone());
        Ok(())
    }

    pub fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        if let Some(gpu) = &mut self.gpu {
            let new_size = gpu.resize(new_size);
            if let Some(new_size) = new_size {
                self.camera
                    .update_aspect_ratio(new_size.width as f32 / new_size.height as f32);
            }
        }
    }

    pub fn update(&mut self, inputs: &WinitInputHelper) {
        let cursor_capture = self.camera_controller.update(inputs, self.delta_time);
        if let Some(gpu) = &mut self.gpu {
            gpu.update_cursor_capture(cursor_capture, inputs);
        }
        self.camera.update_camera(&self.camera_controller);
    }

    fn render_data(&self) -> RenderData<'_> {
        RenderData {
            camera: &self.camera,
            // TODO: Set the data correctly
            time_data: shader::Time {
                elapsed: 0.0,
                delta: 1000.0 / 60.0,
                frame: 0,
            },
            mouse_data: shader::Mouse {
                pos: Vector2::<f32>::ZERO.to_raw(),
                buttons: 0,
            },
        }
    }

    pub fn render(&mut self) -> Result<(), wgpu::SurfaceError> {
        if let Some(mut gpu) = self.gpu.take() {
            gpu.render(self.render_data())?;
            self.gpu = Some(gpu);
        }
        Ok(())
    }

    pub fn get_profiling_data(&mut self) -> Option<Vec<wgpu_profiler::GpuTimerQueryResult>> {
        assert!(self.gpu.is_some());
        assert!(self.profiler_settings.gpu);
        self.gpu.as_mut().and_then(|gpu| {
            gpu.context
                .profiler
                .process_finished_frame(gpu.context.queue.get_timestamp_period())
        })
    }
}

pub struct GpuApplication {
    context: WgpuContext,
    depth_texture: Texture,
    time_buffer: TypedBuffer<shader::Time>,
    screen_buffer: TypedBuffer<shader::Screen>,
    mouse_buffer: TypedBuffer<shader::Mouse>,
    camera_buffer: TypedBuffer<shader::Camera>,
    light_buffer: TypedBuffer<shader::Lights>,
    model_buffer: TypedBuffer<shader::Model>,
    render_pipeline: wgpu::RenderPipeline,
    render_bind_group_0: shader::bind_groups::BindGroup0,
    render_bind_group_1: Vec<shader::bind_groups::BindGroup1>,
    compute_patches_pipeline: wgpu::ComputePipeline,
    /// The second one is for "force_render"
    compute_patches_input_buffer: [TypedBuffer<compute_patches::InputBuffer>; 2],
    compute_patches_bind_group_0: compute_patches::bind_groups::BindGroup0,
    compute_patches_bind_group_1: compute_patches::bind_groups::BindGroup1,
    compute_patches_bind_group_2: [compute_patches::bind_groups::BindGroup2; 2],
    copy_patches_pipeline: wgpu::ComputePipeline,
    copy_patches_bind_group_0: copy_patches::bind_groups::BindGroup0,
    patches_buffer_starting_patch: TypedBuffer<compute_patches::Patches>,
    patches_buffer_reset: TypedBuffer<compute_patches::Patches>,
    patches_buffer: [TypedBuffer<compute_patches::Patches>; 2],
    render_buffer_initial: compute_patches::RenderBuffer,
    render_buffer: Vec<TypedBuffer<compute_patches::RenderBuffer>>,
    indirect_compute_buffer_initial: compute_patches::DispatchIndirectArgs,
    indirect_compute_buffer: [TypedBuffer<compute_patches::DispatchIndirectArgs>; 2],
    indirect_draw_buffers: TypedBuffer<copy_patches::DrawIndexedBuffers>,
    meshes: Vec<Mesh>,
    threshold_factor: f32, // TODO: Make this adjustable
    cursor_capture: WindowCursorCapture,
}

#[derive(Debug, Clone, Copy)]
enum WindowCursorCapture {
    Free,
    LockedAndHidden(PhysicalPosition<f64>),
}

impl GpuApplication {
    pub fn update_cursor_capture(
        &mut self,
        cursor_capture: CursorCapture,
        inputs: &WinitInputHelper,
    ) {
        let window = &self.context.window;
        match (self.cursor_capture, cursor_capture) {
            (WindowCursorCapture::LockedAndHidden(position), CursorCapture::Free) => {
                window
                    .set_cursor_grab(winit::window::CursorGrabMode::None)
                    .unwrap();
                window.set_cursor_visible(true);
                window.set_cursor_position(position).unwrap();
                self.cursor_capture = WindowCursorCapture::Free;
            }
            (WindowCursorCapture::Free, CursorCapture::Free) => {}
            (WindowCursorCapture::LockedAndHidden(_), CursorCapture::LockedAndHidden) => {}
            (WindowCursorCapture::Free, CursorCapture::LockedAndHidden) => {
                let cursor_position = inputs.cursor().unwrap_or((0.0, 0.0));
                window
                    .set_cursor_grab(winit::window::CursorGrabMode::Confined)
                    .or_else(|_e| window.set_cursor_grab(winit::window::CursorGrabMode::Locked))
                    .unwrap();
                window.set_cursor_visible(false);
                self.cursor_capture = WindowCursorCapture::LockedAndHidden(PhysicalPosition::new(
                    cursor_position.0 as f64,
                    cursor_position.1 as f64,
                ));
            }
        }
    }
}

fn patch_sizes() -> [u32; 5] {
    [2, 4, 8, 16, 32]
}

impl GpuApplication {
    pub async fn new(
        window: Window,
        camera: &Camera,
        profiler_settings: &ProfilerSettings,
    ) -> anyhow::Result<Self> {
        let context = WgpuContext::new(window, profiler_settings).await?;
        let device = &context.device;

        // Some arbitrary splits (size/2 - 1 == one quad per four pixels)
        let meshes = [0, 1, 3, 7, 15]
            .iter()
            .map(|&size| {
                let mut mesh = Mesh::new_tesselated_quad(device, size);
                mesh.transform.position = Point3::new(0.0, 1.0, 0.0);
                mesh
            })
            .collect::<Vec<_>>();
        assert!(meshes.len() == patch_sizes().len());

        let time_buffer = TypedBuffer::new_uniform(
            device,
            "Time Buffer",
            &shader::Time {
                elapsed: 0.0,
                delta: 1000.0 / 60.0,
                frame: 0,
            },
            wgpu::BufferUsages::COPY_DST,
        )?;
        let window_pixel_size = context.window.inner_size();
        let screen_buffer = TypedBuffer::new_uniform(
            device,
            "Screen Buffer",
            &shader::Screen {
                resolution: Vector2::<u32>::new(window_pixel_size.width, window_pixel_size.height)
                    .to_raw(),
                inv_resolution: Vector2::<f32>::new(
                    1.0 / (window_pixel_size.width as f32),
                    1.0 / (window_pixel_size.height as f32),
                )
                .to_raw(),
            },
            wgpu::BufferUsages::COPY_DST,
        )?;
        let mouse_buffer = TypedBuffer::new_uniform(
            device,
            "Mouse Buffer",
            &shader::Mouse {
                pos: Vector2::<f32>::ZERO.to_raw(),
                buttons: 0,
            },
            wgpu::BufferUsages::COPY_DST,
        )?;

        let camera_buffer = TypedBuffer::new_uniform(
            device,
            "Camera Buffer",
            &camera.to_shader(),
            wgpu::BufferUsages::COPY_DST,
        )?;

        let light_buffer = TypedBuffer::new_storage(
            device,
            "Light Buffer",
            &shader::Lights {
                ambient: Vector4::<f32>::new(0.1, 0.1, 0.1, 0.0).to_raw(),
                points_length: 1,
                points: vec![shader::PointLight {
                    position_range: Vector4::<f32>::new(0.0, 4.0, 2.0, 40.0).to_raw(),
                    color_intensity: Vector4::<f32>::new(1.0, 1.0, 1.0, 3.0).to_raw(),
                }],
            },
            wgpu::BufferUsages::COPY_DST,
        )?;

        let model_buffer = TypedBuffer::new_uniform(
            device,
            "Model Buffer",
            &shader::Model {
                model_similarity: meshes[0].get_model_matrix().to_raw(),
            },
            wgpu::BufferUsages::COPY_DST,
        )?;
        let max_patch_count = 10_000;
        let render_buffer_initial = compute_patches::RenderBuffer {
            patches_length: 0,
            patches_capacity: max_patch_count,
            patches: vec![],
        };
        let render_buffer = patch_sizes()
            .iter()
            .map(|size| {
                TypedBuffer::new_storage_with_runtime_array(
                    device,
                    &format!("Render Buffer {}", size),
                    &render_buffer_initial,
                    max_patch_count as u64,
                    wgpu::BufferUsages::COPY_DST,
                )
            })
            .collect::<Result<Vec<_>, _>>()?;

        let material_buffer = TypedBuffer::new_uniform(
            device,
            "Material Buffer",
            &shader::Material {
                color_roughness: Vector4::<f32>::new(0.6, 1.0, 1.0, 0.7).to_raw(),
                emissive_metallic: Vector4::<f32>::new(0.0, 0.0, 0.0, 0.1).to_raw(),
            },
            wgpu::BufferUsages::COPY_DST,
        )?;

        let render_bind_group_0 = shader::bind_groups::BindGroup0::from_bindings(
            device,
            shader::bind_groups::BindGroupLayout0 {
                camera: camera_buffer.as_entire_buffer_binding(),
                time: time_buffer.as_entire_buffer_binding(),
                screen: screen_buffer.as_entire_buffer_binding(),
                mouse: mouse_buffer.as_entire_buffer_binding(),
            },
        );
        let render_bind_group_1: Vec<_> = render_buffer
            .iter()
            .map(|v| {
                shader::bind_groups::BindGroup1::from_bindings(
                    device,
                    shader::bind_groups::BindGroupLayout1 {
                        lights: light_buffer.as_entire_buffer_binding(),
                        model: model_buffer.as_entire_buffer_binding(),
                        render_buffer: v.as_entire_buffer_binding(),
                        material: material_buffer.as_entire_buffer_binding(),
                    },
                )
            })
            .collect();

        let depth_texture = Texture::create_depth_texture(device, &context.config, "Depth Texture");

        let shader = shader::create_shader_module(device);
        let render_pipeline_layout = shader::create_pipeline_layout(device);
        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Render Pipeline"),
            layout: Some(&render_pipeline_layout),
            vertex: shader::vertex_state(
                &shader,
                &shader::vs_main_entry(wgpu::VertexStepMode::Vertex),
            ),
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: shader::ENTRY_FS_MAIN,
                targets: &[Some(wgpu::ColorTargetState {
                    format: context.config.format,
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
        });

        let compute_patches_pipeline =
            device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
                label: Some("Compute Patches Pipeline"),
                layout: Some(&compute_patches::create_pipeline_layout(device)),
                module: &compute_patches::create_shader_module(device),
                entry_point: compute_patches::ENTRY_MAIN,
                compilation_options: Default::default(),
            });

        let threshold_factor = 1.0;
        let compute_patches_input_buffer = [
            TypedBuffer::new_uniform(
                device,
                "Compute Patches Input Buffer",
                &compute_patches::InputBuffer {
                    model_view_projection: meshes[0].get_model_view_projection(camera).to_raw(),
                    threshold_factor,
                    force_render: 0,
                },
                wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::COPY_SRC,
            )?,
            TypedBuffer::new_uniform(
                device,
                "Compute Patches Input Buffer Force Render",
                &compute_patches::InputBuffer {
                    model_view_projection: meshes[0].get_model_view_projection(camera).to_raw(),
                    threshold_factor,
                    force_render: 1,
                },
                wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::COPY_SRC,
            )?,
        ];
        let patches_buffer_starting_patch = compute_patches::Patches {
            patches_length: 1,
            patches_capacity: max_patch_count,
            patches: vec![compute_patches::EncodedPatch {
                // Just the leading 1 bit
                u: 1,
                v: 1,
            }],
        };
        let patches_buffer_reset = compute_patches::Patches {
            patches_length: 0,
            patches_capacity: max_patch_count,
            patches: vec![],
        };

        let patches_buffer = [
            TypedBuffer::new_storage_with_runtime_array(
                device,
                "Patches Buffer 0",
                &patches_buffer_starting_patch,
                max_patch_count as u64,
                wgpu::BufferUsages::STORAGE
                    | wgpu::BufferUsages::COPY_SRC
                    | wgpu::BufferUsages::COPY_DST,
            )?,
            TypedBuffer::new_storage_with_runtime_array(
                device,
                "Patches Buffer 1",
                &patches_buffer_reset,
                max_patch_count as u64,
                wgpu::BufferUsages::STORAGE
                    | wgpu::BufferUsages::COPY_SRC
                    | wgpu::BufferUsages::COPY_DST,
            )?,
        ];
        let patches_buffer_starting_patch = TypedBuffer::new_storage_with_runtime_array(
            device,
            "Patches Buffer Starting Patch",
            &patches_buffer_starting_patch,
            1,
            wgpu::BufferUsages::COPY_SRC,
        )?;
        let patches_buffer_reset = TypedBuffer::new_storage_with_runtime_array(
            device,
            "Patches Buffer Reset",
            &patches_buffer_reset,
            1,
            wgpu::BufferUsages::COPY_SRC,
        )?;

        let indirect_compute_buffer_initial =
            compute_patches::DispatchIndirectArgs { x: 1, y: 1, z: 1 };
        let indirect_compute_buffer = [
            TypedBuffer::new_storage(
                device,
                "Indirect Compute Dispatch Buffer 0",
                &indirect_compute_buffer_initial,
                wgpu::BufferUsages::INDIRECT
                    | wgpu::BufferUsages::STORAGE
                    | wgpu::BufferUsages::COPY_DST,
            )?,
            TypedBuffer::new_storage(
                device,
                "Indirect Compute Dispatch Buffer 1",
                &indirect_compute_buffer_initial,
                wgpu::BufferUsages::INDIRECT
                    | wgpu::BufferUsages::STORAGE
                    | wgpu::BufferUsages::COPY_DST,
            )?,
        ];

        let compute_patches_bind_group_0 = compute_patches::bind_groups::BindGroup0::from_bindings(
            device,
            compute_patches::bind_groups::BindGroupLayout0 {
                mouse: mouse_buffer.as_entire_buffer_binding(),
                screen: screen_buffer.as_entire_buffer_binding(),
                time: time_buffer.as_entire_buffer_binding(),
            },
        );

        let compute_patches_bind_group_1 = compute_patches::bind_groups::BindGroup1::from_bindings(
            device,
            compute_patches::bind_groups::BindGroupLayout1 {
                input_buffer: compute_patches_input_buffer[0].as_entire_buffer_binding(),
                render_buffer_2: render_buffer[0].as_entire_buffer_binding(),
                render_buffer_4: render_buffer[1].as_entire_buffer_binding(),
                render_buffer_8: render_buffer[2].as_entire_buffer_binding(),
                render_buffer_16: render_buffer[3].as_entire_buffer_binding(),
                render_buffer_32: render_buffer[4].as_entire_buffer_binding(),
            },
        );
        let compute_patches_bind_group_2 = [
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

        let indirect_draw_data = meshes
            .iter()
            .map(|mesh| {
                copy_patches::DrawIndexedIndirectArgs {
                    index_count: mesh.num_indices,
                    instance_count: 0, // Our shader sets this
                    first_index: 0,
                    base_vertex: 0,
                    first_instance: 0,
                }
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
            wgpu::BufferUsages::INDIRECT
                | wgpu::BufferUsages::STORAGE
                | wgpu::BufferUsages::COPY_SRC,
        )?;

        let copy_patches_pipeline =
            device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
                label: Some("Copy Patches Pipeline"),
                layout: Some(&copy_patches::create_pipeline_layout(device)),
                module: &copy_patches::create_shader_module(device),
                entry_point: "main",
                compilation_options: Default::default(),
            });
        let copy_patches_bind_group_0 = copy_patches::bind_groups::BindGroup0::from_bindings(
            device,
            copy_patches::bind_groups::BindGroupLayout0 {
                render_buffer_2: render_buffer[0].as_entire_buffer_binding(),
                render_buffer_4: render_buffer[1].as_entire_buffer_binding(),
                render_buffer_8: render_buffer[2].as_entire_buffer_binding(),
                render_buffer_16: render_buffer[3].as_entire_buffer_binding(),
                render_buffer_32: render_buffer[4].as_entire_buffer_binding(),
                indirect_draw: indirect_draw_buffers.as_entire_buffer_binding(),
            },
        );

        Ok(Self {
            context,
            render_pipeline,
            render_bind_group_0,
            render_bind_group_1,
            indirect_compute_buffer_initial,
            indirect_compute_buffer,
            compute_patches_input_buffer,
            compute_patches_pipeline,
            compute_patches_bind_group_0,
            compute_patches_bind_group_1,
            compute_patches_bind_group_2,
            copy_patches_pipeline,
            copy_patches_bind_group_0,
            patches_buffer_starting_patch,
            patches_buffer_reset,
            patches_buffer,
            render_buffer_initial,
            render_buffer,
            indirect_draw_buffers,
            depth_texture,
            time_buffer,
            screen_buffer,
            mouse_buffer,
            camera_buffer,
            light_buffer,
            model_buffer,
            meshes,
            threshold_factor,
            cursor_capture: WindowCursorCapture::Free,
        })
    }

    #[must_use]
    pub fn resize(
        &mut self,
        new_size: winit::dpi::PhysicalSize<u32>,
    ) -> Option<winit::dpi::PhysicalSize<u32>> {
        if new_size.width > 0 && new_size.height > 0 && new_size != self.context.size {
            self.context.size = new_size;
            self.context.config.width = new_size.width;
            self.context.config.height = new_size.height;
            self.context
                .surface
                .configure(&self.context.device, &self.context.config);
            self.depth_texture = Texture::create_depth_texture(
                &self.context.device,
                &self.context.config,
                "Depth Texture",
            );
            Some(new_size)
        } else {
            None
        }
    }

    pub fn render(&mut self, render_data: RenderData) -> Result<(), wgpu::SurfaceError> {
        let surface = &self.context.surface;
        let queue = &self.context.queue;
        let output = surface.get_current_texture()?;
        let view = output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        self.time_buffer
            .update(queue, &render_data.time_data)
            .unwrap();
        let screen_size = self.context.size;
        self.screen_buffer
            .update(
                queue,
                &shader::Screen {
                    resolution: Vector2::<u32>::new(screen_size.width, screen_size.height).to_raw(),
                    inv_resolution: Vector2::<f32>::new(
                        1.0 / (screen_size.width as f32),
                        1.0 / (screen_size.height as f32),
                    )
                    .to_raw(),
                },
            )
            .unwrap();
        self.mouse_buffer
            .update(queue, &render_data.mouse_data)
            .unwrap();

        self.camera_buffer
            .update(queue, &render_data.camera.to_shader())
            .unwrap();
        self.model_buffer
            .update(
                queue,
                &shader::Model {
                    model_similarity: self.meshes[0].get_model_matrix().to_raw(),
                },
            )
            .unwrap();
        {
            let mut data = compute_patches::InputBuffer {
                model_view_projection: self.meshes[0]
                    .get_model_view_projection(render_data.camera)
                    .to_raw(),
                threshold_factor: self.threshold_factor,
                force_render: 0,
            };
            self.compute_patches_input_buffer[0]
                .update(queue, &data)
                .unwrap();
            data.force_render = 1;
            self.compute_patches_input_buffer[1]
                .update(queue, &data)
                .unwrap();
        }

        self.indirect_compute_buffer[0]
            .update(queue, &self.indirect_compute_buffer_initial)
            .unwrap();
        self.indirect_compute_buffer[1]
            .update(queue, &self.indirect_compute_buffer_initial)
            .unwrap();
        for render_buffer in self.render_buffer.iter() {
            render_buffer
                .update(queue, &self.render_buffer_initial)
                .unwrap();
        }

        let mut command_encoder =
            self.context
                .device
                .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                    label: Some("Render Encoder"),
                });
        // Profiling
        let mut commands =
            self.context
                .profiler
                .scope("Render", &mut command_encoder, &self.context.device);

        commands.copy_buffer_to_buffer(
            self.patches_buffer_starting_patch.buffer(),
            0,
            self.patches_buffer[0].buffer(),
            0,
            self.patches_buffer_starting_patch.buffer().size(),
        );
        let number_of_rounds = 4;
        for i in 0..number_of_rounds {
            let is_last_round = i == number_of_rounds - 1;
            {
                commands.copy_buffer_to_buffer(
                    self.patches_buffer_reset.buffer(),
                    0,
                    self.patches_buffer[1].buffer(),
                    0,
                    self.patches_buffer_reset.buffer().size(),
                );
                let mut compute_pass = commands.scoped_compute_pass(
                    format!("Compute Patches From-To {i}"),
                    &self.context.device,
                );
                compute_pass.set_pipeline(&self.compute_patches_pipeline);
                compute_patches::set_bind_groups(
                    &mut compute_pass,
                    &self.compute_patches_bind_group_0,
                    &self.compute_patches_bind_group_1,
                    &self.compute_patches_bind_group_2[0],
                );
                compute_pass
                    .dispatch_workgroups_indirect(self.indirect_compute_buffer[0].buffer(), 0);
            }
            {
                commands.copy_buffer_to_buffer(
                    self.patches_buffer_reset.buffer(),
                    0,
                    self.patches_buffer[0].buffer(),
                    0,
                    self.patches_buffer_reset.buffer().size(),
                );
                if is_last_round {
                    // Set the "force_render" flag to 1
                    commands.copy_buffer_to_buffer(
                        self.compute_patches_input_buffer[1].buffer(),
                        0,
                        self.compute_patches_input_buffer[0].buffer(),
                        0,
                        self.compute_patches_input_buffer[0].buffer().size(),
                    );
                }

                let mut compute_pass = commands.begin_compute_pass(&wgpu::ComputePassDescriptor {
                    label: Some(&format!("Compute Patches To-From {i}")),
                    timestamp_writes: None,
                });
                compute_pass.set_pipeline(&self.compute_patches_pipeline);
                compute_patches::set_bind_groups(
                    &mut compute_pass,
                    &self.compute_patches_bind_group_0,
                    &self.compute_patches_bind_group_1,
                    &self.compute_patches_bind_group_2[1],
                );
                compute_pass
                    .dispatch_workgroups_indirect(self.indirect_compute_buffer[1].buffer(), 0);
            }
        }

        {
            let mut compute_pass =
                commands.scoped_compute_pass("Copy Patches Pass", &self.context.device);
            compute_pass.set_pipeline(&self.copy_patches_pipeline);
            copy_patches::set_bind_groups(&mut compute_pass, &self.copy_patches_bind_group_0);
            compute_pass.dispatch_workgroups_indirect(self.indirect_compute_buffer[0].buffer(), 0);
        }

        {
            let mut render_pass = commands.scoped_render_pass(
                "Render Pass",
                &self.context.device,
                wgpu::RenderPassDescriptor {
                    label: Some("Render Pass"),
                    color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                        view: &view,
                        resolve_target: None,
                        ops: wgpu::Operations {
                            load: wgpu::LoadOp::Clear(wgpu::Color {
                                r: 0.1,
                                g: 0.2,
                                b: 0.3,
                                a: 1.0,
                            }),
                            store: wgpu::StoreOp::Store,
                        },
                    })],
                    depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                        view: &self.depth_texture.view,
                        depth_ops: Some(wgpu::Operations {
                            load: wgpu::LoadOp::Clear(0.0), // Reverse Z checklist https://iolite-engine.com/blog_posts/reverse_z_cheatsheet
                            store: wgpu::StoreOp::Store,
                        }),
                        stencil_ops: None,
                    }),
                    timestamp_writes: None,
                    occlusion_query_set: None,
                },
            );
            render_pass.set_pipeline(&self.render_pipeline);
            let indirect_draw_offsets = [
                std::mem::offset_of!(copy_patches::DrawIndexedBuffers, indirect_draw_2)
                    as wgpu::BufferAddress,
                std::mem::offset_of!(copy_patches::DrawIndexedBuffers, indirect_draw_4)
                    as wgpu::BufferAddress,
                std::mem::offset_of!(copy_patches::DrawIndexedBuffers, indirect_draw_8)
                    as wgpu::BufferAddress,
                std::mem::offset_of!(copy_patches::DrawIndexedBuffers, indirect_draw_16)
                    as wgpu::BufferAddress,
                std::mem::offset_of!(copy_patches::DrawIndexedBuffers, indirect_draw_32)
                    as wgpu::BufferAddress,
            ];
            for ((bind_group_1, mesh), buffer_offset) in self
                .render_bind_group_1
                .iter()
                .zip(self.meshes.iter())
                .zip(indirect_draw_offsets)
            {
                shader::set_bind_groups(&mut render_pass, &self.render_bind_group_0, bind_group_1);
                render_pass.set_vertex_buffer(0, mesh.vertex_buffer.slice(..));
                render_pass
                    .set_index_buffer(mesh.index_buffer.slice(..), wgpu::IndexFormat::Uint16);
                render_pass
                    .draw_indexed_indirect(self.indirect_draw_buffers.buffer(), buffer_offset)
            }
        }

        // Finish the profiler
        std::mem::drop(commands);
        self.context.profiler.resolve_queries(&mut command_encoder);
        // Submit the commands
        self.context
            .queue
            .submit(std::iter::once(command_encoder.finish()));
        output.present();

        // Finish the frame after all commands have been submitted
        self.context.profiler.end_frame().unwrap();
        Ok(())
    }

    pub fn request_redraw(&self) {
        self.context.window.request_redraw();
    }

    pub fn size(&self) -> winit::dpi::PhysicalSize<u32> {
        self.context.size
    }
}

pub struct RenderData<'a> {
    camera: &'a Camera,
    time_data: shader::Time,
    mouse_data: shader::Mouse,
}

struct WgpuContext {
    instance: wgpu::Instance,
    surface: wgpu::Surface<'static>,
    adapter: wgpu::Adapter,
    device: wgpu::Device,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,
    profiler: GpuProfiler,
    window: Arc<Window>,
    size: winit::dpi::PhysicalSize<u32>,
}

impl WgpuContext {
    async fn new(window: Window, profiler_settings: &ProfilerSettings) -> anyhow::Result<Self> {
        let window = Arc::new(window);
        let size = window.inner_size();
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: wgpu::Backends::PRIMARY,
            ..Default::default()
        });

        let surface = instance.create_surface(window.clone())?;

        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::HighPerformance,
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            })
            .await
            .ok_or_else(|| anyhow::anyhow!("No adapter found"))?;
        info!("Adapter: {:?}", adapter.get_info());

        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    required_features: wgpu::Features::default()
                        | (adapter.features() & GpuProfiler::ALL_WGPU_TIMER_FEATURES)
                        | (adapter.features() & wgpu::Features::POLYGON_MODE_LINE),
                    required_limits: wgpu::Limits::default(),
                    label: None,
                },
                None,
            )
            .await
            .unwrap();

        // TODO: Srgb support https://sotrh.github.io/learn-wgpu/intermediate/tutorial13-hdr/#output-too-dark-on-webgpu
        let surface_caps = surface.get_capabilities(&adapter);
        let surface_format = surface_caps
            .formats
            .iter()
            .find(|_format| true)
            // .find(|format| format.is_srgb())
            .cloned()
            .ok_or_else(|| anyhow::anyhow!("No sRGB format surface found"))?;

        let config = wgpu::SurfaceConfiguration {
            format: surface_format,
            ..surface
                .get_default_config(&adapter, size.width, size.height)
                .ok_or_else(|| anyhow::anyhow!("No default surface config found"))?
        };
        surface.configure(&device, &config);

        let gpu_profiler_settings = GpuProfilerSettings {
            enable_timer_queries: profiler_settings.gpu,
            ..GpuProfilerSettings::default()
        };

        #[cfg(feature = "tracy")]
        let profiler = GpuProfiler::new_with_tracy_client(
            gpu_profiler_settings.clone(),
            adapter.get_info().backend,
            &device,
            &queue,
        )
        .unwrap_or_else(|e| match e {
            wgpu_profiler::CreationError::TracyClientNotRunning
            | wgpu_profiler::CreationError::TracyGpuContextCreationError(_) => {
                warn!("Failed to connect to Tracy. Continuing without Tracy integration.");
                GpuProfiler::new(gpu_profiler_settings).expect("Failed to create profiler")
            }
            _ => {
                panic!("Failed to create profiler: {}", e);
            }
        });

        #[cfg(not(feature = "tracy"))]
        let profiler = GpuProfiler::new(gpu_profiler_settings).expect("Failed to create profiler");

        window.request_redraw();

        Ok(WgpuContext {
            instance,
            surface,
            adapter,
            device,
            queue,
            config,
            profiler,
            window,
            size,
        })
    }
}

impl Camera {
    pub fn to_shader(&self) -> shader::Camera {
        shader::Camera {
            view: self.view_matrix().to_raw(),
            projection: self.projection_matrix().to_raw(),
            world_position: self.position.to_raw().extend(1.0),
        }
    }
}

impl Mesh {
    pub fn get_model_view_projection(&self, camera: &Camera) -> Matrix4<f32> {
        camera.projection_matrix() * camera.view_matrix() * self.transform.to_matrix()
    }
}
