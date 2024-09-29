pub mod frame_counter;
mod scene;
mod virtual_model;
mod wgpu_context;

use frame_counter::FrameCounter;
use glam::UVec2;
use scene::SceneData;
use virtual_model::VirtualModel;
use wgpu_context::WgpuContext;

use crate::{
    buffer::TypedBuffer,
    game::{GameRes, ModelInfo},
    mesh::Mesh,
    shaders::{compute_patches, copy_patches, shader},
    texture::Texture,
    window_or_fallback::WindowOrFallback,
};

struct RenderStep {
    bind_group_0: shader::bind_groups::BindGroup0,
}
struct ComputePatchesStep {
    bind_group_0: compute_patches::bind_groups::BindGroup0,
}
struct CopyPatchesStep {
    pipeline: wgpu::ComputePipeline,
}

#[must_use]
pub struct GpuApplicationBuilder {
    pub context: WgpuContext,
}

impl GpuApplicationBuilder {
    #[must_use]
    pub async fn new(window: WindowOrFallback) -> anyhow::Result<Self> {
        let context = WgpuContext::new(window).await?;
        Ok(Self { context })
    }

    #[must_use]
    pub fn build(self) -> GpuApplication {
        GpuApplication::new(self.context)
    }
}

pub struct GpuApplication {
    context: WgpuContext,
    depth_texture: Texture,
    scene_data: SceneData,
    shader_arena: virtual_model::ShaderArena,
    virtual_models: Vec<VirtualModel>,
    compute_patches: ComputePatchesStep,
    copy_patches: CopyPatchesStep,
    render_step: RenderStep,
    patches_buffer_reset: TypedBuffer<compute_patches::Patches>,
    force_render_values: [TypedBuffer<compute_patches::ForceRenderFlag>; 2],
    render_buffer_reset: compute_patches::RenderBuffer,
    indirect_compute_buffer_reset: TypedBuffer<compute_patches::DispatchIndirectArgs>,
    meshes: Vec<Mesh>,
    threshold_factor: f32, // TODO: Make this adjustable
    cursor_capture: WindowCursorCapture,
    frame_counter: FrameCounter,
}

const PATCH_SIZES: [u32; 5] = [2, 4, 8, 16, 32];
const MAX_PATCH_COUNT: u32 = 100_000;

impl GpuApplication {
    pub fn new(context: WgpuContext) -> Self {
        let device = &context.device;

        let scene_data = SceneData::new(device);

        // Some arbitrary splits (size/2 - 1 == one quad per four pixels)
        let meshes = [0, 1, 3, 7, 15]
            .iter()
            .map(|&size| Mesh::new_tesselated_quad(device, size))
            .collect::<Vec<_>>();
        assert!(meshes.len() == PATCH_SIZES.len());

        let shader_arena = virtual_model::ShaderArena::new(&context);
        let virtual_models = vec![];

        let render_buffer_reset = compute_patches::RenderBuffer {
            patches_length: 0,
            patches_capacity: MAX_PATCH_COUNT,
            patches: vec![],
        };

        let depth_texture = Texture::create_depth_texture(device, context.size(), "Depth Texture");

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
        );

        let indirect_compute_buffer_reset = TypedBuffer::new_storage(
            device,
            "Indirect Compute Dispatch Buffer Reset",
            &compute_patches::DispatchIndirectArgs { x: 0, y: 1, z: 1 },
            wgpu::BufferUsages::COPY_SRC,
        );

        let force_render_values = [
            TypedBuffer::new_uniform(
                device,
                "Disable Force Render",
                &compute_patches::ForceRenderFlag { flag: 0 },
                wgpu::BufferUsages::COPY_SRC,
            ),
            TypedBuffer::new_uniform(
                device,
                "Enable Force Render",
                &compute_patches::ForceRenderFlag { flag: 1 },
                wgpu::BufferUsages::COPY_SRC,
            ),
        ];

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
                cache: Default::default(),
            }),
        };

        Self {
            context,
            indirect_compute_buffer_reset,
            render_step,
            compute_patches,
            copy_patches,
            patches_buffer_reset,
            render_buffer_reset,
            force_render_values,
            depth_texture,
            scene_data,
            meshes,
            shader_arena,
            virtual_models,
            threshold_factor,
            cursor_capture: WindowCursorCapture::Free,
            frame_counter: FrameCounter::new(),
        }
    }

    pub fn resize(&mut self, new_size: UVec2) {
        let new_size = new_size.max(UVec2::new(1, 1));
        if new_size == self.context.size() {
            return;
        }
        self.context.resize(new_size);
        self.depth_texture =
            Texture::create_depth_texture(&self.context.device, new_size, "Depth Texture");
    }

    pub fn render(&mut self, game: &GameRes) -> Result<(), wgpu::SurfaceError> {
        // 1. Read from game
        let frame_time = self.frame_counter.new_frame();
        let render_data = RenderData {
            size: self.context.size(),
            camera: game.camera.to_shader(self.context.size()),
            time_data: shader::Time {
                elapsed: frame_time.elapsed.0,
                delta: frame_time.delta.0,
                frame: frame_time.frame as u32,
            },
            mouse_data: shader::Mouse {
                pos: game.mouse,
                buttons: if game.mouse_held { 1 } else { 0 },
            },
        };

        self.update_cursor_capture(game.cursor_capture);

        self.context.set_profiling(game.profiler_settings.gpu);

        update_virtual_models(
            &mut self.shader_arena,
            &mut self.virtual_models,
            &game.models,
            &self.context,
            &self.meshes,
        );

        update_shaders(&mut self.shader_arena, &game.shaders, &self.context);

        // 2. Render
        match &self.context.surface {
            wgpu_context::SurfaceOrFallback::Surface { surface, .. } => {
                // TODO: Handle all cases properly https://github.com/gfx-rs/wgpu/blob/a0c185a28c232ee2ab63f72d6fd3a63a3f787309/examples/src/framework.rs#L216
                let output = surface.get_current_texture()?;
                let mut command_encoder =
                    self.render_commands(&output.texture, render_data, &game.lod_stage)?;
                self.context.profiler.resolve_queries(&mut command_encoder);
                self.context
                    .queue
                    .submit(std::iter::once(command_encoder.finish()));
                output.present();
            }
            wgpu_context::SurfaceOrFallback::Fallback { texture } => {
                let mut command_encoder =
                    self.render_commands(texture, render_data, &game.lod_stage)?;
                self.context.profiler.resolve_queries(&mut command_encoder);
                self.context
                    .queue
                    .submit(std::iter::once(command_encoder.finish()));
            }
        };

        self.context.profiler.end_frame().unwrap();
        Ok(())
    }

    pub fn render_commands(
        &self,
        surface_texture: &wgpu::Texture,
        render_data: RenderData,
        lod_stage: &Option<Box<dyn Fn(&crate::game::ShaderId, u32)>>,
    ) -> Result<wgpu::CommandEncoder, wgpu::SurfaceError> {
        let queue = &self.context.queue;
        let view = surface_texture.create_view(&wgpu::TextureViewDescriptor {
            format: Some(self.context.view_format),
            ..Default::default()
        });

        self.scene_data.write_buffers(&render_data, queue);

        // Write the buffers for each model
        for model in self.virtual_models.iter() {
            model.compute_patches.input_buffer.write_buffer(
                queue,
                &compute_patches::InputBuffer {
                    model_view_projection: model.get_model_view_projection(&render_data.camera),
                    threshold_factor: self.threshold_factor,
                },
            );
            model.compute_patches.patches_buffer[0].write_buffer(
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
            );
            model.compute_patches.indirect_compute_buffer[0].write_buffer(
                queue,
                &compute_patches::DispatchIndirectArgs { x: 1, y: 1, z: 1 },
            );

            for render_buffer in model.compute_patches.render_buffer.iter() {
                render_buffer.write_buffer(queue, &self.render_buffer_reset);
            }

            model.render_step.model_buffer.write_buffer(
                queue,
                &shader::Model {
                    model_similarity: model.get_model_matrix(),
                },
            );
            model
                .render_step
                .material_buffer
                .write_buffer(queue, &model.material_info.to_shader());
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

        for (index, model) in self.virtual_models.iter().enumerate() {
            let shaders = self
                .shader_arena
                .get_shader(&model.shader_key)
                .unwrap_or_else(|| self.shader_arena.get_missing_shader());
            match lod_stage {
                Some(lod_stage) => lod_stage(&model.shader_key, index as u32),
                None => {
                    self.do_lod_stage(model, &mut commands, &shaders);
                }
            }

            {
                let mut compute_pass =
                    commands.scoped_compute_pass("Copy Patch Sizes Pass", &self.context.device);
                compute_pass.set_pipeline(&self.copy_patches.pipeline);
                copy_patches::set_bind_groups(
                    &mut compute_pass.recorder,
                    &model.copy_patches.bind_group_0,
                );
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
            let shaders = self
                .shader_arena
                .get_shader(&model.shader_key)
                .unwrap_or_else(|| self.shader_arena.get_missing_shader());
            {
                render_pass.set_pipeline(&shaders.render);
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
                        &mut render_pass.recorder,
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

        Ok(command_encoder)
    }

    fn do_lod_stage(
        &self,
        model: &VirtualModel,
        commands: &mut wgpu_profiler::Scope<'_, wgpu::CommandEncoder>,
        shaders: &virtual_model::ShaderPipelines,
    ) {
        // Each round, we do a ping-pong and pong-ping
        // 2*4 rounds is enough to subdivide a 4k screen into 16x16 pixel patches
        let double_number_of_rounds = 4;
        for i in 0..double_number_of_rounds {
            let is_last_round = i == double_number_of_rounds - 1;
            // TODO: Should I create many compute passes, or just one?
            {
                model.compute_patches.patches_buffer[1]
                    .copy_all_from(&self.patches_buffer_reset, commands);
                model.compute_patches.indirect_compute_buffer[1]
                    .copy_all_from(&self.indirect_compute_buffer_reset, commands);

                let mut compute_pass = commands.scoped_compute_pass(
                    format!("Compute Patches From-To {i}"),
                    &self.context.device,
                );
                compute_pass.set_pipeline(&shaders.compute_patches);
                compute_patches::set_bind_groups(
                    &mut compute_pass.recorder,
                    &self.compute_patches.bind_group_0,
                    &model.compute_patches.bind_group_1,
                    &model.compute_patches.bind_group_2[0],
                );
                compute_pass.dispatch_workgroups_indirect(
                    &model.compute_patches.indirect_compute_buffer[0],
                    0,
                );
            }
            if is_last_round {
                // Set to true
                model
                    .compute_patches
                    .force_render_uniform
                    .copy_all_from(&self.force_render_values[1], commands);
            }
            {
                model.compute_patches.patches_buffer[0]
                    .copy_all_from(&self.patches_buffer_reset, commands);
                model.compute_patches.indirect_compute_buffer[0]
                    .copy_all_from(&self.indirect_compute_buffer_reset, commands);

                let mut compute_pass = commands.scoped_compute_pass(
                    format!("Compute Patches To-From {i}"),
                    &self.context.device,
                );
                compute_pass.set_pipeline(&shaders.compute_patches);
                compute_patches::set_bind_groups(
                    &mut compute_pass.recorder,
                    &self.compute_patches.bind_group_0,
                    &model.compute_patches.bind_group_1,
                    &model.compute_patches.bind_group_2[1],
                );
                compute_pass.dispatch_workgroups_indirect(
                    &model.compute_patches.indirect_compute_buffer[1],
                    0,
                );
            }
            if is_last_round {
                // Set to false
                model
                    .compute_patches
                    .force_render_uniform
                    .copy_all_from(&self.force_render_values[0], commands);
            }
        }
    }

    pub fn size(&self) -> UVec2 {
        self.context.size()
    }

    pub fn device(&self) -> &wgpu::Device {
        &self.context.device
    }

    pub fn force_wait(&self) {
        self.context.instance.poll_all(true);
    }
}

fn update_shaders(
    shader_arena: &mut virtual_model::ShaderArena,
    shaders: &std::collections::HashMap<crate::game::ShaderId, crate::game::ShaderInfo>,
    context: &WgpuContext,
) {
    for (shader_id, shader_info) in shaders.iter() {
        shader_arena.add_shader(
            shader_id.clone(),
            &shader_info.label,
            &shader_info.code,
            &context,
        );
    }
    // TODO: Delete shaders that are not in the list
}

impl GpuApplication {
    pub fn get_profiling_data(&mut self) -> Option<Vec<wgpu_profiler::GpuTimerQueryResult>> {
        self.context
            .profiler
            .process_finished_frame(self.context.queue.get_timestamp_period())
    }
}

fn update_virtual_models(
    _shader_arena: &mut virtual_model::ShaderArena,
    virtual_models: &mut Vec<VirtualModel>,
    models: &[ModelInfo],
    context: &WgpuContext,
    meshes: &[Mesh],
) {
    // Update existing models
    for (virtual_model, model) in virtual_models.iter_mut().zip(models.iter()) {
        virtual_model.transform = model.transform.clone();
        virtual_model.material_info = model.material_info.clone();
        virtual_model.shader_key = model.shader_id.clone();
    }
    // Resize virtual models
    if virtual_models.len() > models.len() {
        virtual_models.truncate(models.len());
    } else if virtual_models.len() < models.len() {
        for (index, model) in models.iter().enumerate().skip(virtual_models.len()) {
            let mut virtual_model = VirtualModel::new(
                context,
                meshes,
                model.shader_id.clone(),
                &format!("ID{index}"),
            );
            virtual_model.transform = model.transform.clone();
            virtual_model.material_info = model.material_info.clone();
            virtual_models.push(virtual_model);
        }
    }
}

pub struct RenderData {
    pub size: UVec2,
    pub camera: shader::Camera,
    pub time_data: shader::Time,
    pub mouse_data: shader::Mouse,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum CursorCapture {
    Free,
    LockedAndHidden,
}

#[derive(Debug, Clone, Copy)]
pub enum WindowCursorCapture {
    Free,
    LockedAndHidden(winit::dpi::PhysicalPosition<f64>),
}

impl GpuApplication {
    pub fn update_cursor_capture(&mut self, cursor_capture: WindowCursorCapture) {
        let window = match &self.context.surface {
            wgpu_context::SurfaceOrFallback::Surface { window, .. } => window,
            wgpu_context::SurfaceOrFallback::Fallback { .. } => return,
        };
        match (self.cursor_capture, cursor_capture) {
            (WindowCursorCapture::LockedAndHidden(position), WindowCursorCapture::Free) => {
                window
                    .set_cursor_grab(winit::window::CursorGrabMode::None)
                    .unwrap();
                window.set_cursor_visible(true);
                let _ = window.set_cursor_position(position);
                self.cursor_capture = WindowCursorCapture::Free;
            }
            (WindowCursorCapture::Free, WindowCursorCapture::Free) => {}
            (
                WindowCursorCapture::LockedAndHidden(_),
                WindowCursorCapture::LockedAndHidden(position),
            ) => {
                self.cursor_capture = WindowCursorCapture::LockedAndHidden(position);
            }
            (WindowCursorCapture::Free, WindowCursorCapture::LockedAndHidden(cursor_position)) => {
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
