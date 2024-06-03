mod frame_counter;
mod scene;
mod virtual_model;
mod wgpu_context;

use std::sync::Arc;

use frame_counter::{FrameCounter, FrameTime, Seconds};
use glamour::{Point3, ToRaw, Vector2};
use scene::SceneData;
pub use virtual_model::MaterialInfo;
use virtual_model::VirtualModel;
use wgpu_context::WgpuContext;
use wgpu_profiler::GpuProfilerSettings;
use winit::{dpi::PhysicalPosition, window::Window};

use crate::{
    buffer::TypedBuffer,
    camera::{
        camera_controller::{
            CameraController, ChosenKind, GeneralController, GeneralControllerSettings,
        },
        Camera, CameraSettings,
    },
    input::WindowInputs,
    mesh::Mesh,
    shaders::{compute_patches, copy_patches, shader},
    texture::Texture,
    transform::Transform,
};

#[derive(Debug, Clone, Default)]
pub struct ProfilerSettings {
    pub gpu: bool,
}

pub struct ModelInfo {
    pub label: String,
    pub transform: Transform,
    pub material_info: MaterialInfo,
    pub evaluate_image_code: String,
}

pub struct CpuApplication {
    pub gpu: Option<GpuApplication>,
    pub camera_controller: CameraController,
    models: Vec<ModelInfo>,
    last_update_instant: Option<std::time::Instant>,
    frame_counter: FrameCounter,
    camera: Camera,
    mouse: Vector2<f32>,
    mouse_held: bool,
    profiler_settings: ProfilerSettings,
}

impl CpuApplication {
    pub fn new() -> anyhow::Result<Self> {
        let camera = Camera::new(Vector2::new(1, 1), CameraSettings::default());
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
            models: vec![],
            last_update_instant: None,
            frame_counter: FrameCounter::new(),
            mouse: Vector2::ZERO,
            mouse_held: false,
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

    pub async fn create_surface(&mut self, window: Arc<Window>) -> anyhow::Result<()> {
        if self.gpu.is_some() {
            return Ok(());
        }
        let size = window.inner_size();
        self.camera
            .update_size(Vector2::new(size.width, size.height));
        let gpu = GpuApplication::new(window, &self.camera, &self.models, &self.profiler_settings)
            .await?;
        self.gpu = Some(gpu);
        self.set_profiling(self.profiler_settings.clone());
        Ok(())
    }

    pub fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        if let Some(gpu) = &mut self.gpu {
            let new_size = gpu.resize(new_size);
            if let Some(new_size) = new_size {
                self.camera
                    .update_size(Vector2::new(new_size.width, new_size.height));
            }
        }
    }

    pub fn update_models(&mut self, models: Vec<ModelInfo>) {
        self.models = models;
        if let Some(gpu) = &mut self.gpu {
            update_virtual_models(
                &mut gpu.virtual_models,
                &self.models,
                &gpu.context,
                &gpu.meshes,
            )
            .unwrap();
        }
    }

    pub fn update(&mut self, inputs: &WindowInputs) {
        let now = std::time::Instant::now();
        if let Some(last_update_instant) = self.last_update_instant {
            let delta = Seconds((now - last_update_instant).as_secs_f32());
            let cursor_capture = self.camera_controller.update(inputs, delta.0);
            if let Some(gpu) = &mut self.gpu {
                gpu.update_cursor_capture(cursor_capture, inputs);
            }
        }
        self.last_update_instant = Some(now);
        self.camera.update_camera(&self.camera_controller);
        self.mouse = Vector2::new(
            inputs.mouse.position.x as f32,
            inputs.mouse.position.y as f32,
        );
        self.mouse_held = inputs.mouse.pressed(winit::event::MouseButton::Left);
    }

    fn render_data(&self, frame_time: &FrameTime) -> RenderData<'_> {
        RenderData {
            camera: &self.camera,
            time_data: shader::Time {
                elapsed: frame_time.elapsed.0,
                delta: frame_time.delta.0,
                frame: frame_time.frame as u32,
            },
            mouse_data: shader::Mouse {
                pos: self.mouse.to_raw(),
                buttons: if self.mouse_held { 1 } else { 0 },
            },
        }
    }

    pub fn render(&mut self) -> Result<(), wgpu::SurfaceError> {
        let frame_time = self.frame_counter.new_frame();
        if let Some(mut gpu) = self.gpu.take() {
            gpu.render(self.render_data(&frame_time))?;
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

struct RenderStep {
    bind_group_0: shader::bind_groups::BindGroup0,
}
struct ComputePatchesStep {
    bind_group_0: compute_patches::bind_groups::BindGroup0,
}
struct CopyPatchesStep {
    pipeline: wgpu::ComputePipeline,
}

pub struct GpuApplication {
    context: WgpuContext,
    depth_texture: Texture,
    scene_data: SceneData,
    virtual_models: Vec<VirtualModel>,
    compute_patches: ComputePatchesStep,
    copy_patches: CopyPatchesStep,
    render_step: RenderStep,
    patches_buffer_reset: TypedBuffer<compute_patches::Patches>,
    render_buffer_reset: compute_patches::RenderBuffer,
    indirect_compute_buffer_reset: TypedBuffer<compute_patches::DispatchIndirectArgs>,
    meshes: Vec<Mesh>,
    threshold_factor: f32, // TODO: Make this adjustable
    cursor_capture: WindowCursorCapture,
}

const PATCH_SIZES: [u32; 5] = [2, 4, 8, 16, 32];
const MAX_PATCH_COUNT: u32 = 100_000;

impl GpuApplication {
    pub async fn new(
        window: Arc<Window>,
        camera: &Camera,
        models: &[ModelInfo],
        profiler_settings: &ProfilerSettings,
    ) -> anyhow::Result<Self> {
        let context = WgpuContext::new(window, profiler_settings).await?;
        let device = &context.device;

        let scene_data = SceneData::new(device, camera)?;

        // Some arbitrary splits (size/2 - 1 == one quad per four pixels)
        let meshes = [0, 1, 3, 7, 15]
            .iter()
            .map(|&size| Mesh::new_tesselated_quad(device, size))
            .collect::<Vec<_>>();
        assert!(meshes.len() == PATCH_SIZES.len());

        let mut virtual_models = vec![];
        update_virtual_models(&mut virtual_models, models, &context, &meshes)?;

        let render_buffer_reset = compute_patches::RenderBuffer {
            patches_length: 0,
            patches_capacity: MAX_PATCH_COUNT,
            patches: vec![],
        };

        let depth_texture = Texture::create_depth_texture(device, &context.config, "Depth Texture");

        let render_step = RenderStep {
            bind_group_0: scene_data.as_bind_group_0(device),
        };

        let threshold_factor = 1.0;

        let patches_buffer_reset = TypedBuffer::new_storage_with_runtime_array(
            device,
            "Patches Buffer Reset",
            &compute_patches::Patches {
                patches_length: 0,
                patches_capacity: MAX_PATCH_COUNT,
                patches: vec![],
            },
            1,
            wgpu::BufferUsages::COPY_SRC,
        )?;

        let indirect_compute_buffer_reset = TypedBuffer::new_storage(
            device,
            "Indirect Compute Dispatch Buffer Reset",
            &compute_patches::DispatchIndirectArgs { x: 0, y: 1, z: 1 },
            wgpu::BufferUsages::COPY_SRC,
        )?;

        let compute_patches = ComputePatchesStep {
            bind_group_0: compute_patches::bind_groups::BindGroup0::from_bindings(
                device,
                compute_patches::bind_groups::BindGroupLayout0 {
                    mouse: scene_data.mouse_buffer.as_entire_buffer_binding(),
                    screen: scene_data.screen_buffer.as_entire_buffer_binding(),
                    time: scene_data.time_buffer.as_entire_buffer_binding(),
                },
            ),
        };

        let copy_patches = CopyPatchesStep {
            pipeline: device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
                label: Some("Copy Patches"),
                layout: Some(&copy_patches::create_pipeline_layout(device)),
                module: &copy_patches::create_shader_module(device),
                entry_point: copy_patches::ENTRY_MAIN,
                compilation_options: Default::default(),
            }),
        };

        Ok(Self {
            context,
            indirect_compute_buffer_reset,
            render_step,
            compute_patches,
            copy_patches,
            patches_buffer_reset,
            render_buffer_reset,
            depth_texture,
            scene_data,
            meshes,
            virtual_models,
            threshold_factor,
            cursor_capture: WindowCursorCapture::Free,
        })
    }

    #[must_use]
    pub fn resize(
        &mut self,
        new_size: winit::dpi::PhysicalSize<u32>,
    ) -> Option<winit::dpi::PhysicalSize<u32>> {
        let new_size = new_size.max(winit::dpi::PhysicalSize::new(1, 1));
        if new_size != self.context.size {
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
        let view = output.texture.create_view(&wgpu::TextureViewDescriptor {
            format: Some(self.context.view_format),
            ..Default::default()
        });

        self.scene_data.write_buffers(&render_data, queue);

        // Write the buffers for each model
        for model in self.virtual_models.iter() {
            model
                .compute_patches
                .input_buffer
                .write_buffer(
                    queue,
                    &compute_patches::InputBuffer {
                        model_view_projection: model
                            .get_model_view_projection(render_data.camera)
                            .to_raw(),
                        threshold_factor: self.threshold_factor,
                    },
                )
                .unwrap();
            model.compute_patches.patches_buffer[0]
                .write_buffer(
                    queue,
                    &compute_patches::Patches {
                        patches_length: 1,
                        patches_capacity: MAX_PATCH_COUNT,
                        patches: vec![compute_patches::EncodedPatch {
                            // Just the leading 1 bit
                            u: 1,
                            v: 1,
                        }],
                    },
                )
                .unwrap();
            model.compute_patches.indirect_compute_buffer[0]
                .write_buffer(
                    queue,
                    &compute_patches::DispatchIndirectArgs { x: 1, y: 1, z: 1 },
                )
                .unwrap();

            for render_buffer in model.compute_patches.render_buffer.iter() {
                render_buffer
                    .write_buffer(queue, &self.render_buffer_reset)
                    .unwrap();
            }

            model
                .render_step
                .model_buffer
                .write_buffer(
                    queue,
                    &shader::Model {
                        model_similarity: model.get_model_matrix().to_raw(),
                    },
                )
                .unwrap();
            model
                .render_step
                .material_buffer
                .write_buffer(queue, &model.material_info.to_shader())
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

        for model in self.virtual_models.iter() {
            // Each round, we do a ping-pong and pong-ping
            // 2*4 rounds is enough to subdivide a 4k screen into 16x16 pixel patches
            let double_number_of_rounds = 4;
            for i in 0..double_number_of_rounds {
                let is_last_round = i == double_number_of_rounds - 1;
                // TODO: Should I create many compute passes, or just one?
                {
                    model.compute_patches.patches_buffer[1]
                        .copy_all_from(&self.patches_buffer_reset, &mut commands);
                    model.compute_patches.indirect_compute_buffer[1]
                        .copy_all_from(&self.indirect_compute_buffer_reset, &mut commands);

                    let mut compute_pass = commands.scoped_compute_pass(
                        format!("Compute Patches From-To {i}"),
                        &self.context.device,
                    );
                    compute_pass.set_pipeline(&model.compute_patches.pipeline[0]);
                    compute_patches::set_bind_groups(
                        &mut compute_pass,
                        &self.compute_patches.bind_group_0,
                        &model.compute_patches.bind_group_1,
                        &model.compute_patches.bind_group_2[0],
                    );
                    compute_pass.dispatch_workgroups_indirect(
                        &model.compute_patches.indirect_compute_buffer[0],
                        0,
                    );
                }
                {
                    model.compute_patches.patches_buffer[0]
                        .copy_all_from(&self.patches_buffer_reset, &mut commands);
                    model.compute_patches.indirect_compute_buffer[0]
                        .copy_all_from(&self.indirect_compute_buffer_reset, &mut commands);

                    let mut compute_pass = commands.scoped_compute_pass(
                        format!("Compute Patches To-From {i}"),
                        &self.context.device,
                    );
                    if is_last_round {
                        compute_pass.set_pipeline(&model.compute_patches.pipeline[1]);
                    } else {
                        compute_pass.set_pipeline(&model.compute_patches.pipeline[0]);
                    }
                    compute_patches::set_bind_groups(
                        &mut compute_pass,
                        &self.compute_patches.bind_group_0,
                        &model.compute_patches.bind_group_1,
                        &model.compute_patches.bind_group_2[1],
                    );
                    compute_pass.dispatch_workgroups_indirect(
                        &model.compute_patches.indirect_compute_buffer[1],
                        0,
                    );
                }
            }

            {
                let mut compute_pass =
                    commands.scoped_compute_pass("Copy Patch Sizes Pass", &self.context.device);
                compute_pass.set_pipeline(&self.copy_patches.pipeline);
                copy_patches::set_bind_groups(&mut compute_pass, &model.copy_patches.bind_group_0);
                compute_pass.dispatch_workgroups(1, 1, 1);
            }
        }

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
        for model in self.virtual_models.iter() {
            {
                render_pass.set_pipeline(&model.render_step.pipeline);
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
                for ((bind_group_1, mesh), buffer_offset) in model
                    .render_step
                    .bind_group_1
                    .iter()
                    .zip(self.meshes.iter())
                    .zip(indirect_draw_offsets)
                {
                    shader::set_bind_groups(
                        &mut render_pass,
                        &self.render_step.bind_group_0,
                        bind_group_1,
                    );
                    render_pass.set_vertex_buffer(0, mesh.vertex_buffer.slice(..));
                    render_pass
                        .set_index_buffer(mesh.index_buffer.slice(..), wgpu::IndexFormat::Uint16);
                    render_pass.draw_indexed_indirect(
                        &model.copy_patches.indirect_draw_buffers,
                        buffer_offset,
                    )
                }
            }
        }

        // Finish the profiler
        std::mem::drop(render_pass);
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

fn update_virtual_models(
    virtual_models: &mut Vec<VirtualModel>,
    models: &[ModelInfo],
    context: &WgpuContext,
    meshes: &[Mesh],
) -> anyhow::Result<()> {
    // Throwing away and recreating all models is a bit wasteful, but it's fine for now
    *virtual_models = models
        .iter()
        .map(|model| -> Result<VirtualModel, anyhow::Error> {
            let mut virtual_model = VirtualModel::new(Some(model.label.clone()), context, meshes)?;
            virtual_model.transform = model.transform.clone();
            virtual_model.material_info = model.material_info.clone();
            virtual_model.update_code(context, Some(model.evaluate_image_code.as_str()));
            Ok(virtual_model)
        })
        .collect::<Result<Vec<_>, _>>()?;
    Ok(())
}
pub struct RenderData<'a> {
    camera: &'a Camera,
    time_data: shader::Time,
    mouse_data: shader::Mouse,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum CursorCapture {
    Free,
    LockedAndHidden,
}

#[derive(Debug, Clone, Copy)]
enum WindowCursorCapture {
    Free,
    LockedAndHidden(PhysicalPosition<f64>),
}

impl GpuApplication {
    pub fn update_cursor_capture(&mut self, cursor_capture: CursorCapture, inputs: &WindowInputs) {
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
                let cursor_position = inputs.mouse.position;
                window
                    .set_cursor_grab(winit::window::CursorGrabMode::Confined)
                    .or_else(|_e| window.set_cursor_grab(winit::window::CursorGrabMode::Locked))
                    .unwrap();
                window.set_cursor_visible(false);
                self.cursor_capture = WindowCursorCapture::LockedAndHidden(cursor_position);
            }
        }
    }
}
