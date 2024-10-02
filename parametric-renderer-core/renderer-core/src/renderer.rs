pub mod frame_counter;
mod scene;
mod virtual_model;
mod wgpu_context;

use frame_counter::FrameCounter;
use glam::UVec2;
use leptos_reactive::{
    batch, create_effect, create_runtime, create_signal, store_value, Effect, ReadSignal,
    SignalGet, SignalGetUntracked, SignalSet, SignalUpdate, SignalWith, StoredValue, WriteSignal,
};
use scene::SceneData;
use virtual_model::VirtualModel;
use wgpu_context::WgpuContext;

use crate::{
    buffer::TypedBuffer,
    game::{GameRes, ModelInfo},
    mesh::Mesh,
    reactive::ForEach,
    shaders::{compute_patches, copy_patches, shader},
    texture::Texture,
    window_or_fallback::WindowOrFallback,
};

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

#[derive(Clone)]
pub struct RenderData {
    pub camera: crate::camera::Camera,
    pub mouse_pos: glam::Vec2,
    pub mouse_held: bool,
    pub lod_stage: Option<std::sync::Arc<dyn Fn(&crate::game::ShaderId, u32) + 'static>>,
}

pub struct GpuApplication {
    context: StoredValue<WgpuContext>,
    runtime: leptos_reactive::RuntimeId,
    _render_tree: Effect<()>,
    set_render_data: WriteSignal<RenderData>,
    set_desired_size: WriteSignal<UVec2>,
    set_force_wait: WriteSignal<bool>,
    render_result: ReadSignal<Result<(), wgpu::SurfaceError>>,
    cursor_capture: WindowCursorCapture,
    set_models: WriteSignal<Vec<VirtualModel>>,

    // TODO: Refactor
    shader_arena: StoredValue<virtual_model::ShaderArena>,
}

impl Drop for GpuApplication {
    fn drop(&mut self) {
        self.runtime.dispose();
    }
}

const PATCH_SIZES: [u32; 5] = [2, 4, 8, 16, 32];
const MAX_PATCH_COUNT: u32 = 100_000;

impl GpuApplication {
    pub fn new(context: WgpuContext) -> Self {
        let runtime = create_runtime();
        let context = store_value(context);
        let (desired_size, set_desired_size) = create_signal(UVec2::new(1, 1));
        let (force_wait, set_force_wait) = create_signal(false);
        let (render_result, set_render_result) = create_signal(Ok(()));
        let (models, set_models) = create_signal(vec![]);

        let (render_data, set_render_data) = create_signal(RenderData {
            camera: crate::camera::Camera::new(crate::camera::CameraSettings::default()),
            mouse_pos: glam::Vec2::ZERO,
            mouse_held: false,
            lod_stage: None,
        });

        let shader_arena =
            store_value(context.with_value(|context| virtual_model::ShaderArena::new(context)));

        let render_tree = create_effect(move |_| {
            render_component(
                context,
                desired_size,
                force_wait,
                render_data,
                set_render_result,
                shader_arena,
                models,
            );
        });

        Self {
            context,
            runtime,
            _render_tree: render_tree,

            set_render_data,
            set_desired_size,
            set_force_wait,
            render_result,
            cursor_capture: WindowCursorCapture::Free,

            shader_arena,
            set_models,
        }
    }

    pub fn render(&mut self, game: &GameRes) -> Result<(), wgpu::SurfaceError> {
        batch(|| {
            self.cursor_capture = self.update_cursor_capture(game.cursor_capture);

            self.context.update_value(|context| {
                context.set_profiling(game.profiler_settings.gpu);
            });

            self.context.with_value(|context| {
                // TODO: Consider passing the models in in the constructor
                self.set_models.update(|virtual_models| {
                    update_virtual_models(virtual_models, &game.models, &context)
                });

                self.shader_arena.update_value(|shader_arena| {
                    update_shaders(shader_arena, &game.shaders, &context)
                });
            });
            self.set_render_data.update(|render_data| {
                *render_data = RenderData {
                    camera: game.camera.clone(),
                    mouse_pos: game.mouse,
                    mouse_held: game.mouse_held,
                    lod_stage: game.lod_stage.clone(),
                };
            });
        });
        self.render_result.get_untracked()
    }

    pub fn resize(&self, new_size: UVec2) {
        self.set_desired_size.set(new_size);
    }

    pub fn force_wait(&self) {
        self.set_force_wait.set(true);
    }
}

/// We're using Leptos :)
fn render_component(
    context: StoredValue<WgpuContext>,
    desired_size: ReadSignal<UVec2>,
    force_wait: ReadSignal<bool>,
    render_data: ReadSignal<RenderData>,
    set_render_result: WriteSignal<Result<(), wgpu::SurfaceError>>,
    shader_arena: StoredValue<virtual_model::ShaderArena>,
    virtual_models: ReadSignal<Vec<VirtualModel>>,
) -> Effect<()> {
    // size/2 - 1 == one quad per four pixels
    let meshes = context.with_value(|context| {
        PATCH_SIZES
            .iter()
            .map(|size| *size / 2 - 1)
            .map(|splits| Mesh::new_tesselated_quad(&context.device, splits))
            .collect::<Vec<_>>()
    });

    // The threshold factor for the LOD algorithm
    // TODO: Should be adjustable
    let threshold_factor = 1.0;

    let frame_counter = store_value(FrameCounter::new());
    let new_frame_time = move || {
        frame_counter
            .try_update_value(|counter| counter.new_frame())
            .expect("Frame counter has been dropped?")
    };

    let depth_texture = store_value(context.with_value(|context| {
        Texture::create_depth_texture(&context.device, context.size(), "Depth Texture")
    }));

    let scene_data = store_value(context.with_value(|context| SceneData::new(&context.device)));
    let render_bind_group_0 = context.with_value(|context| {
        scene_data.with_value(|scene_data| scene_data.as_bind_group_0(&context.device))
    });

    let copy_patches = CopyPatchesStep {
        pipeline: context.with_value(|context| {
            context
                .device
                .create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
                    label: Some("Copy Patches"),
                    layout: Some(&copy_patches::create_pipeline_layout(&context.device)),
                    module: &copy_patches::create_shader_module(&context.device),
                    entry_point: copy_patches::ENTRY_MAIN,
                    compilation_options: Default::default(),
                    cache: Default::default(),
                })
        }),
    };

    // A reactive effect that reruns whenever any of its signals change
    let _ = resizer_component(desired_size, context, depth_texture);
    let lod_stage = context.with_value(|context| lod_stage_component(context, scene_data));

    // A reactive for loop that reruns whenever the virtual_models change
    let for_shader_ids = ForEach::new(
        virtual_models.into(),
        |model| model.shader_key.clone(),
        |model| model.shader_key.clone(),
    );

    create_effect(move |_| {
        let render_data = render_data.get();
        let frame_time = new_frame_time();
        // 2. Render
        let surface_texture = match context.with_value(|context| context.surface_texture()) {
            Ok(v) => v,
            Err(wgpu::SurfaceError::Lost | wgpu::SurfaceError::Outdated) => {
                // Roughly based on https://github.com/gfx-rs/wgpu/blob/a0c185a28c232ee2ab63f72d6fd3a63a3f787309/examples/src/framework.rs#L216
                context.update_value(|context| context.recreate_swapchain());
                return;
            }
            Err(v) => {
                set_render_result.set(Err(v));
                return;
            }
        };

        context.update_value(|context| {
            let mut command_encoder = depth_texture.with_value(|depth_texture| {
                shader_arena.with_value(|shader_arena| {
                    virtual_models.with(|virtual_models| {
                        for_shader_ids.for_each(|shader_id| {
                            // TODO: Remove this testing code
                            shader_arena
                                .get_shader(&shader_id)
                                .unwrap_or_else(|| shader_arena.get_missing_shader());
                        });

                        for model in virtual_models.iter() {
                            model.compute_patches.force_render_uniform.write_buffer(
                                &context.queue,
                                &compute_patches::ForceRenderFlag { flag: 0 },
                            );
                        }

                        scene_data.with_value(|scene_data| {
                            scene_data.write_buffers(
                                context.size(),
                                &render_data,
                                &frame_time,
                                &context.queue,
                            )
                        });

                        let queue = &context.queue;
                        // Write the buffers for each model
                        // TODO: For each model: Create a new component (and recycle the old one (use a key))
                        // Then we can simplify the code once again
                        // 1. Make the virtual_models a signal
                        // 2. Create the "for" component
                        // 3. Derived value: Call the for component outside of the render loop
                        // 4. Use the derived value in the render loop. Without dependency tracking though! We don't want to re-render the scene if the models change, we want our predictable time-based rendering.
                        for model in virtual_models.iter() {
                            model.compute_patches.input_buffer.write_buffer(
                                queue,
                                &compute_patches::InputBuffer {
                                    model_view_projection: model.get_model_view_projection(
                                        context.size(),
                                        &render_data.camera,
                                    ),
                                    threshold_factor,
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

                            let render_buffer_reset = compute_patches::RenderBuffer {
                                patches_length: 0,
                                patches_capacity: MAX_PATCH_COUNT,
                                patches: vec![],
                            };
                            for render_buffer in model.compute_patches.render_buffer.iter() {
                                render_buffer.write_buffer(queue, &render_buffer_reset);
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

                        let mut command_encoder = context.device.create_command_encoder(
                            &wgpu::CommandEncoderDescriptor {
                                label: Some("Render Encoder"),
                            },
                        );
                        // Profiling
                        let mut commands =
                            context
                                .profiler
                                .scope("Render", &mut command_encoder, &context.device);

                        for (index, model) in virtual_models.iter().enumerate() {
                            let shaders = shader_arena
                                .get_shader(&model.shader_key)
                                .unwrap_or_else(|| shader_arena.get_missing_shader());
                            match render_data.lod_stage.as_ref() {
                                Some(lod_stage) => lod_stage(&model.shader_key, index as u32),
                                None => {
                                    lod_stage(context, model, &mut commands, &shaders);
                                }
                            }

                            {
                                let mut compute_pass = commands
                                    .scoped_compute_pass("Copy Patch Sizes Pass", &context.device);
                                compute_pass.set_pipeline(&copy_patches.pipeline);
                                copy_patches::set_bind_groups(
                                    &mut compute_pass.recorder,
                                    &model.copy_patches.bind_group_0,
                                );
                                compute_pass.dispatch_workgroups(1, 1, 1);
                            }
                        }

                        let mut render_pass = commands.scoped_render_pass(
                            "Render Pass",
                            &context.device,
                            wgpu::RenderPassDescriptor {
                                label: Some("Render Pass"),
                                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                                    view: surface_texture.texture_view(),
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
                                depth_stencil_attachment: Some(
                                    wgpu::RenderPassDepthStencilAttachment {
                                        view: &depth_texture.view,
                                        depth_ops: Some(wgpu::Operations {
                                            load: wgpu::LoadOp::Clear(0.0), // Reverse Z checklist https://iolite-engine.com/blog_posts/reverse_z_cheatsheet
                                            store: wgpu::StoreOp::Store,
                                        }),
                                        stencil_ops: None,
                                    },
                                ),
                                timestamp_writes: None,
                                occlusion_query_set: None,
                            },
                        );
                        for model in virtual_models.iter() {
                            let shaders = shader_arena
                                .get_shader(&model.shader_key)
                                .unwrap_or_else(|| shader_arena.get_missing_shader());
                            {
                                render_pass.set_pipeline(&shaders.render);
                                let indirect_draw_offsets = [
                                    std::mem::offset_of!(
                                        copy_patches::DrawIndexedBuffers,
                                        indirect_draw_2
                                    ) as wgpu::BufferAddress,
                                    std::mem::offset_of!(
                                        copy_patches::DrawIndexedBuffers,
                                        indirect_draw_4
                                    ) as wgpu::BufferAddress,
                                    std::mem::offset_of!(
                                        copy_patches::DrawIndexedBuffers,
                                        indirect_draw_8
                                    ) as wgpu::BufferAddress,
                                    std::mem::offset_of!(
                                        copy_patches::DrawIndexedBuffers,
                                        indirect_draw_16
                                    ) as wgpu::BufferAddress,
                                    std::mem::offset_of!(
                                        copy_patches::DrawIndexedBuffers,
                                        indirect_draw_32
                                    ) as wgpu::BufferAddress,
                                ];
                                for ((bind_group_1, mesh), buffer_offset) in model
                                    .render_step
                                    .bind_group_1
                                    .iter()
                                    .zip(meshes.iter())
                                    .zip(indirect_draw_offsets)
                                {
                                    shader::set_bind_groups(
                                        &mut render_pass.recorder,
                                        &render_bind_group_0,
                                        bind_group_1,
                                    );
                                    render_pass.set_vertex_buffer(0, mesh.vertex_buffer.slice(..));
                                    render_pass.set_index_buffer(
                                        mesh.index_buffer.slice(..),
                                        wgpu::IndexFormat::Uint16,
                                    );
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

                        command_encoder
                    })
                })
            });
            context.profiler.resolve_queries(&mut command_encoder);
            context
                .queue
                .submit(std::iter::once(command_encoder.finish()));
        });

        surface_texture.present();

        context.update_value(|context| context.profiler.end_frame().unwrap());

        if force_wait.get() {
            context.with_value(|context| context.instance.poll_all(true));
        }

        set_render_result.set(Ok(()));
    })
}

fn resizer_component(
    desired_size: ReadSignal<UVec2>,
    context: StoredValue<WgpuContext>,
    depth_texture: StoredValue<Texture>,
) -> Effect<()> {
    create_effect(move |_| {
        // Get the value from a signal
        let desired_size = desired_size.get();

        // Do the resizing. The whole ".update_value(|context| ...)" stuff is because of a Leptos API quirk.
        context.update_value(|context| {
            if let Some(new_size) = context.try_resize(desired_size) {
                depth_texture.set_value(Texture::create_depth_texture(
                    &context.device,
                    new_size,
                    "Depth Texture",
                ));
            }
        });
    })
}

fn lod_stage_component(
    context: &WgpuContext,
    scene_data: StoredValue<SceneData>,
) -> impl Fn(
    &WgpuContext,
    &VirtualModel,
    &mut wgpu_profiler::Scope<'_, wgpu::CommandEncoder>,
    &virtual_model::ShaderPipelines,
) {
    let patches_buffer_reset = TypedBuffer::new_storage_with_runtime_array(
        &context.device,
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
        &context.device,
        "Indirect Compute Dispatch Buffer Reset",
        &compute_patches::DispatchIndirectArgs { x: 0, y: 1, z: 1 },
        wgpu::BufferUsages::COPY_SRC,
    );

    let force_render_false = TypedBuffer::new_uniform(
        &context.device,
        "Disable Force Render",
        &compute_patches::ForceRenderFlag { flag: 0 },
        wgpu::BufferUsages::COPY_SRC,
    );
    let force_render_true = TypedBuffer::new_uniform(
        &context.device,
        "Enable Force Render",
        &compute_patches::ForceRenderFlag { flag: 1 },
        wgpu::BufferUsages::COPY_SRC,
    );

    let compute_patches = ComputePatchesStep {
        bind_group_0: scene_data.with_value(|scene_data| {
            compute_patches::bind_groups::BindGroup0::from_bindings(
                &context.device,
                compute_patches::bind_groups::BindGroupLayout0 {
                    mouse: scene_data.mouse_buffer.as_entire_buffer_binding(),
                    screen: scene_data.screen_buffer.as_entire_buffer_binding(),
                    time: scene_data.time_buffer.as_entire_buffer_binding(),
                },
            )
        }),
    };

    move |context: &WgpuContext,
          model: &VirtualModel,
          commands: &mut wgpu_profiler::Scope<'_, wgpu::CommandEncoder>,
          shaders: &virtual_model::ShaderPipelines| {
        // Each round, we do a ping-pong and pong-ping
        // 2*4 rounds is enough to subdivide a 4k screen into 16x16 pixel patches
        let double_number_of_rounds = 4;
        for i in 0..double_number_of_rounds {
            let is_last_round = i == double_number_of_rounds - 1;
            // TODO: Should I create many compute passes, or just one?
            {
                model.compute_patches.patches_buffer[1]
                    .copy_all_from(&patches_buffer_reset, commands);
                model.compute_patches.indirect_compute_buffer[1]
                    .copy_all_from(&indirect_compute_buffer_reset, commands);

                let mut compute_pass = commands
                    .scoped_compute_pass(format!("Compute Patches From-To {i}"), &context.device);
                compute_pass.set_pipeline(&shaders.compute_patches);
                compute_patches::set_bind_groups(
                    &mut compute_pass.recorder,
                    &compute_patches.bind_group_0,
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
                    .copy_all_from(&force_render_true, commands);
            }
            {
                model.compute_patches.patches_buffer[0]
                    .copy_all_from(&patches_buffer_reset, commands);
                model.compute_patches.indirect_compute_buffer[0]
                    .copy_all_from(&indirect_compute_buffer_reset, commands);

                let mut compute_pass = commands
                    .scoped_compute_pass(format!("Compute Patches To-From {i}"), &context.device);
                compute_pass.set_pipeline(&shaders.compute_patches);
                compute_patches::set_bind_groups(
                    &mut compute_pass.recorder,
                    &compute_patches.bind_group_0,
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
                    .copy_all_from(&force_render_false, commands);
            }
        }
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
            .try_update_value(|context| {
                context
                    .profiler
                    .process_finished_frame(context.queue.get_timestamp_period())
            })
            .flatten()
    }
}

fn update_virtual_models(
    virtual_models: &mut Vec<VirtualModel>,
    models: &[ModelInfo],
    context: &WgpuContext,
) {
    // TODO: Don't recreate the meshes every frame only to read the number of indices
    let meshes = PATCH_SIZES
        .iter()
        .map(|size| *size / 2 - 1)
        .map(|size| Mesh::new_tesselated_quad(&context.device, size))
        .collect::<Vec<_>>();

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
                &meshes,
                model.shader_id.clone(),
                &format!("ID{index}"),
            );
            virtual_model.transform = model.transform.clone();
            virtual_model.material_info = model.material_info.clone();
            virtual_models.push(virtual_model);
        }
    }
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
    pub fn update_cursor_capture(
        &self,
        cursor_capture: WindowCursorCapture,
    ) -> WindowCursorCapture {
        let window_result = self.context.with_value(|context| match &context.surface {
            wgpu_context::SurfaceOrFallback::Surface { window, .. } => Result::Ok(window.clone()),
            wgpu_context::SurfaceOrFallback::Fallback { .. } => Result::Err(self.cursor_capture),
        });
        let window = match window_result {
            Ok(window) => window,
            Err(v) => return v,
        };
        match (self.cursor_capture, cursor_capture) {
            (WindowCursorCapture::LockedAndHidden(position), WindowCursorCapture::Free) => {
                window
                    .set_cursor_grab(winit::window::CursorGrabMode::None)
                    .unwrap();
                window.set_cursor_visible(true);
                let _ = window.set_cursor_position(position);
                WindowCursorCapture::Free
            }
            (WindowCursorCapture::Free, WindowCursorCapture::Free) => WindowCursorCapture::Free,
            (
                WindowCursorCapture::LockedAndHidden(_),
                WindowCursorCapture::LockedAndHidden(position),
            ) => WindowCursorCapture::LockedAndHidden(position),
            (WindowCursorCapture::Free, WindowCursorCapture::LockedAndHidden(cursor_position)) => {
                window
                    .set_cursor_grab(winit::window::CursorGrabMode::Confined)
                    .or_else(|_e| window.set_cursor_grab(winit::window::CursorGrabMode::Locked))
                    .unwrap();
                window.set_cursor_visible(false);
                WindowCursorCapture::LockedAndHidden(cursor_position)
            }
        }
    }
}
