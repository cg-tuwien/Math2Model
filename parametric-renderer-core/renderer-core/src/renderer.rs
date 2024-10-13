pub mod frame_counter;
mod scene;
mod virtual_model;
mod wgpu_context;

use std::sync::Arc;

use frame_counter::FrameCounter;
use glam::UVec2;

use reactive_graph::{
    computed::Memo,
    effect::{Effect, RenderEffect},
    owner::{LocalStorage, Owner, StoredValue},
    prelude::*,
    signal::{signal, ArcReadSignal, ReadSignal, RwSignal, WriteSignal},
};
use scene::SceneData;
use virtual_model::VirtualModel;
use wgpu_context::{create_profiler, WgpuContext};
use wgpu_profiler::GpuProfiler;

use crate::{
    buffer::TypedBuffer,
    game::{GameRes, ModelInfo},
    mesh::Mesh,
    reactive::{ForEach, MemoComputed, SignalVec},
    shaders::{compute_patches, copy_patches, shader},
    texture::Texture,
    window_or_fallback::WindowOrFallback,
};

struct ComputePatchesStep {
    bind_group_0: compute_patches::bind_groups::BindGroup0,
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

/// Values that change *every frame*.
#[derive(Clone)]
pub struct FrameData {
    pub camera: crate::camera::Camera,
    pub mouse_pos: glam::Vec2,
    pub mouse_held: bool,
    pub lod_stage: Option<std::sync::Arc<dyn Fn(&crate::game::ShaderId, &str) + 'static>>,
}

pub struct GpuApplication {
    context: StoredValue<WgpuContext>,
    profiler: StoredValue<GpuProfiler>,
    profiling_enabled: bool,
    _runtime: Owner,
    render_tree: Arc<dyn Fn(&FrameData) -> Result<(), wgpu::SurfaceError>>,
    set_desired_size: WriteSignal<UVec2>,
    set_force_wait: WriteSignal<bool>,
    /// Sets the threshold factor for the LOD algorithm
    set_threshold_factor: WriteSignal<f32>,
    cursor_capture: WindowCursorCapture,
    models: SignalVec<ModelInfo>,

    // TODO: Refactor
    shader_arena: StoredValue<virtual_model::ShaderArena>,
}

const PATCH_SIZES: [u32; 5] = [2, 4, 8, 16, 32];
const MAX_PATCH_COUNT: u32 = 100_000;

impl GpuApplication {
    pub fn new(context: WgpuContext) -> Self {
        let runtime = Owner::new();
        let context = StoredValue::new(context);
        let profiler = StoredValue::new(create_profiler(&context.read_value()));
        let (desired_size, set_desired_size) = signal(UVec2::new(1, 1));
        let (force_wait, set_force_wait) = signal(false);
        let (threshold_factor, set_threshold_factor) = signal(1.0f32);
        let models = SignalVec::new();

        let shader_arena = StoredValue::new(
            context.with_value(|context| virtual_model::ShaderArena::new(context)),
        );

        let render_tree = Owner::with(&runtime, || {
            Arc::new(render_component(
                context,
                profiler,
                desired_size,
                threshold_factor,
                force_wait,
                shader_arena,
                models.clone(),
            ))
        });

        Self {
            context,
            profiler,
            profiling_enabled: false,
            _runtime: runtime,
            render_tree,

            set_desired_size,
            set_threshold_factor,
            set_force_wait,
            cursor_capture: WindowCursorCapture::Free,
            models,

            shader_arena,
        }
    }

    pub fn render(&mut self, game: &GameRes) -> Result<(), wgpu::SurfaceError> {
        self.cursor_capture = self.update_cursor_capture(game.cursor_capture);

        if self.profiling_enabled != game.profiler_settings.gpu {
            self.profiling_enabled = game.profiler_settings.gpu;
            self.profiler
                .write_value()
                .change_settings(wgpu_profiler::GpuProfilerSettings {
                    enable_timer_queries: self.profiling_enabled,
                    ..Default::default()
                })
                .unwrap();
        }

        self.context.with_value(|context| {
            // TODO: Don't do "untrack"
            reactive_graph::graph::untrack(|| {
                update_models(self.models.clone(), &game.models);
                self.shader_arena.update_value(|shader_arena| {
                    update_shaders(shader_arena, &game.shaders, &context)
                });
            });
        });
        let render_tree = self.render_tree.clone();
        let frame_data = FrameData {
            camera: game.camera.clone(),
            mouse_pos: game.mouse,
            mouse_held: game.mouse_held,
            lod_stage: game.lod_stage.clone(),
        };
        let render = RenderEffect::new(move |_| (render_tree)(&frame_data));
        render.take_value().expect("Render should have executed")
    }

    pub fn resize(&self, new_size: UVec2) {
        self.set_desired_size.set(new_size);
    }

    pub fn force_wait(&self) {
        self.set_force_wait.set(true);
    }

    pub fn get_profiling_data(&self) -> Option<Vec<wgpu_profiler::GpuTimerQueryResult>> {
        self.profiler
            .write_value()
            .process_finished_frame(self.context.read_value().queue.get_timestamp_period())
    }
}

/// We're using Leptos :)
fn render_component(
    context: StoredValue<WgpuContext>,
    profiler: StoredValue<GpuProfiler>,
    desired_size: ReadSignal<UVec2>,
    threshold_factor: ReadSignal<f32>,
    force_wait: ReadSignal<bool>,
    shader_arena: StoredValue<virtual_model::ShaderArena>,
    models: SignalVec<ModelInfo>,
) -> impl Fn(&FrameData) -> Result<(), wgpu::SurfaceError> {
    let frame_counter = RwSignal::new(FrameCounter::new());
    let new_frame_time = move || frame_counter.write().new_frame();

    let depth_texture = StoredValue::new(context.with_value(|context| {
        Texture::create_depth_texture(&context.device, context.size(), "Depth Texture")
    }));

    let scene_data =
        StoredValue::new(context.with_value(|context| SceneData::new(&context.device)));
    let render_bind_group_0 = StoredValue::new(context.with_value(|context| {
        scene_data.with_value(|scene_data| scene_data.as_bind_group_0(&context.device))
    }));

    let compute_patches = StoredValue::new(context.with_value(|context| ComputePatchesStep {
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
    }));

    // Maybe I'll refactor that into a function that returns a component_function?
    let copy_patches_pipeline = StoredValue::new(context.with_value(|context| {
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
    }));

    // A reactive effect that reruns whenever any of its signals change
    let _ = resizer_component(desired_size, context, depth_texture);

    // size/2 - 1 == one quad per four pixels
    let meshes = StoredValue::new(context.with_value(|context| {
        PATCH_SIZES
            .iter()
            .map(|size| *size / 2 - 1)
            .map(|splits| Mesh::new_tesselated_quad(&context.device, splits))
            .collect::<Vec<_>>()
    }));

    // Alternative design:
    // 1. Create a virtual model for each model
    // 2. Use this virtual model to render the model
    /*
    let virtual_models = new_computed_vec(
        move || models.iter(),
        |model| model.clone(),
        move |model| {
            let context = context.read_value();
            let meshes = meshes.read_value();
            let model = model.read();
            VirtualModel::new(
                &context,
                &meshes,
                model.shader_id.clone(),
                &format!("ID{}", model.id),
            )
        },
    ); */

    let models_components = ForEach::new(
        move || models.iter(),
        |model| model.clone(),
        move |model| {
            model_component(
                context,
                scene_data,
                model.clone(),
                threshold_factor,
                compute_patches,
                copy_patches_pipeline,
                RenderInfo {
                    render_bind_group_0,
                    meshes,
                },
            )
        },
    );

    move |render_data: &FrameData| {
        let frame_time = new_frame_time();
        // 2. Render
        let surface_texture = match context.with_value(|context| context.surface_texture()) {
            Ok(v) => v,
            err @ Err(wgpu::SurfaceError::Lost | wgpu::SurfaceError::Outdated) => {
                // Roughly based on https://github.com/gfx-rs/wgpu/blob/a0c185a28c232ee2ab63f72d6fd3a63a3f787309/examples/src/framework.rs#L216
                context.update_value(|context| context.recreate_swapchain());
                return err.map(|_| ());
            }
            err => {
                return err.map(|_| ());
            }
        };

        context.with_value(|context| {
            scene_data.read_value().write_buffers(
                context.size(),
                &render_data,
                &frame_time,
                &context.queue,
            );
        });

        models_components.for_each(|renderers| {
            (renderers.write_buffers)(render_data);
        });

        context.with_value(|context| {
            let mut command_encoder = depth_texture.with_value(|depth_texture| {
                shader_arena.with_value(|shader_arena| {
                    let mut command_encoder =
                        context
                            .device
                            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                                label: Some("Render Encoder"),
                            });
                    // Profiling
                    let profiler_guard = profiler.read_value();
                    let mut commands =
                        profiler_guard.scope("Render", &mut command_encoder, &context.device);

                    models_components.for_each(|renderers| {
                        (renderers.lod_stage)(context, render_data, &mut commands, shader_arena);
                    });

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

                    // Render the models
                    models_components.for_each(|renderers| {
                        (renderers.render_stage)(&mut render_pass, &shader_arena);
                    });

                    // Finish the profiler
                    std::mem::drop(render_pass);
                    std::mem::drop(commands);

                    command_encoder
                })
            });
            profiler.update_value(|profiler| profiler.resolve_queries(&mut command_encoder));
            context
                .queue
                .submit(std::iter::once(command_encoder.finish()));
        });

        surface_texture.present();

        profiler.update_value(|profiler| profiler.end_frame().unwrap());

        if force_wait.get() {
            context.with_value(|context| context.instance.poll_all(true));
        }

        Ok(())
    }
}

fn resizer_component(
    desired_size: ReadSignal<UVec2>,
    context: StoredValue<WgpuContext>,
    depth_texture: StoredValue<Texture>,
) -> Effect<LocalStorage> {
    Effect::new(move |_| {
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

/// Returns multiple render functions
fn model_component(
    context: StoredValue<WgpuContext>,
    scene_data: StoredValue<SceneData>,
    model: ArcReadSignal<ModelInfo>,
    threshold_factor: ReadSignal<f32>,
    compute_patches: StoredValue<ComputePatchesStep>,
    copy_patches_pipeline: StoredValue<wgpu::ComputePipeline>,
    render_stage: RenderInfo,
) -> ModelRenderers<
    impl Fn(&FrameData),
    impl Fn(
        &WgpuContext,
        &FrameData,
        &mut wgpu_profiler::Scope<'_, wgpu::CommandEncoder>,
        &virtual_model::ShaderArena,
    ),
    impl Fn(&mut wgpu_profiler::OwningScope<'_, wgpu::RenderPass<'_>>, &virtual_model::ShaderArena),
> {
    let virtual_model = Memo::new_computed({
        let model = model.clone();
        move |_| {
            let context = context.read_value();
            let meshes = render_stage.meshes.read_value();
            let model = model.read();
            VirtualModel::new(
                &context,
                &meshes,
                model.shader_id.clone(),
                &format!("ID{}", model.id),
            )
        }
    });

    let write_buffers_component =
        write_buffers_component(context, model.clone(), virtual_model, threshold_factor);

    let lod_stage_component = context.with_value(|context| {
        lod_stage_component(
            context,
            virtual_model,
            compute_patches,
            copy_patches_pipeline,
            scene_data,
        )
    });

    let render_component = render_model_component(
        context,
        render_stage.render_bind_group_0,
        model.clone(),
        virtual_model,
        render_stage.meshes,
    );

    ModelRenderers {
        write_buffers: write_buffers_component,
        lod_stage: lod_stage_component,
        render_stage: render_component,
    }
}

struct ModelRenderers<WriteBuffers, LodStage, RenderStage> {
    write_buffers: WriteBuffers,
    lod_stage: LodStage,
    render_stage: RenderStage,
}

fn write_buffers_component(
    context: StoredValue<WgpuContext>,
    model: ArcReadSignal<ModelInfo>,
    virtual_model: Memo<VirtualModel>,
    threshold_factor: ReadSignal<f32>,
) -> impl Fn(&FrameData) {
    // TODO: Refactor this more (move it to the components where it's needed, depend on the signals => granular writes)
    move |render_data: &FrameData| {
        // Write the buffers for each model
        let context = context.read_value();
        let queue = &context.queue;
        let virtual_model = virtual_model.read();
        virtual_model
            .compute_patches
            .force_render_uniform
            .write_buffer(
                &context.queue,
                &compute_patches::ForceRenderFlag { flag: 0 },
            );
        let model_view_projection = render_data.camera.projection_matrix(context.size())
            * render_data.camera.view_matrix()
            * model.read().transform.to_matrix();
        virtual_model.compute_patches.input_buffer.write_buffer(
            queue,
            &compute_patches::InputBuffer {
                model_view_projection,
                threshold_factor: threshold_factor.get(),
            },
        );
        virtual_model.compute_patches.patches_buffer[0].write_buffer(
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
        virtual_model.compute_patches.indirect_compute_buffer[0].write_buffer(
            queue,
            &compute_patches::DispatchIndirectArgs { x: 1, y: 1, z: 1 },
        );

        let render_buffer_reset = compute_patches::RenderBuffer {
            patches_length: 0,
            patches_capacity: MAX_PATCH_COUNT,
            patches: vec![],
        };
        for render_buffer in virtual_model.compute_patches.render_buffer.iter() {
            render_buffer.write_buffer(queue, &render_buffer_reset);
        }

        virtual_model.render_step.model_buffer.write_buffer(
            queue,
            &shader::Model {
                model_similarity: model.read().transform.to_matrix(),
            },
        );
        virtual_model
            .render_step
            .material_buffer
            .write_buffer(queue, &model.read().material_info.to_shader());
    }
}

fn lod_stage_component(
    // TODO: Refactor to take a StoredValue<Context>
    context: &WgpuContext,
    virtual_model: Memo<VirtualModel>,
    compute_patches: StoredValue<ComputePatchesStep>,
    copy_patches_pipeline: StoredValue<wgpu::ComputePipeline>,
    scene_data: StoredValue<SceneData>,
) -> impl Fn(
    &WgpuContext,
    &FrameData,
    &mut wgpu_profiler::Scope<'_, wgpu::CommandEncoder>,
    &virtual_model::ShaderArena,
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

    move |context: &WgpuContext,
          frame_data: &FrameData,
          commands: &mut wgpu_profiler::Scope<'_, wgpu::CommandEncoder>,
          shader_arena: &virtual_model::ShaderArena| {
        let model = virtual_model.read();

        let shaders = shader_arena
            .get_shader(&model.shader_key)
            .unwrap_or_else(|| shader_arena.get_missing_shader());
        if let Some(overriden_lod_stage) = frame_data.lod_stage.as_ref() {
            (overriden_lod_stage)(&model.shader_key, &model.id);
        } else {
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

                    let mut compute_pass = commands.scoped_compute_pass(
                        format!("Compute Patches From-To {i}"),
                        &context.device,
                    );
                    compute_pass.set_pipeline(&shaders.compute_patches);
                    compute_patches::set_bind_groups(
                        &mut compute_pass.recorder,
                        &compute_patches.read_value().bind_group_0,
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

                    let mut compute_pass = commands.scoped_compute_pass(
                        format!("Compute Patches To-From {i}"),
                        &context.device,
                    );
                    compute_pass.set_pipeline(&shaders.compute_patches);
                    compute_patches::set_bind_groups(
                        &mut compute_pass.recorder,
                        // Maybe refactor so that parent components set bind groups, and children just assume that they're set?
                        &compute_patches.read_value().bind_group_0,
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
        {
            let mut compute_pass =
                commands.scoped_compute_pass("Copy Patch Sizes Pass", &context.device);
            compute_pass.set_pipeline(&copy_patches_pipeline.read_value());
            copy_patches::set_bind_groups(
                &mut compute_pass.recorder,
                &model.copy_patches.bind_group_0,
            );
            compute_pass.dispatch_workgroups(1, 1, 1);
        }
    }
}

struct RenderInfo {
    render_bind_group_0: StoredValue<shader::bind_groups::BindGroup0>,
    meshes: StoredValue<Vec<Mesh>>,
}

/// Renders a single model
/// A model can change even when its ID stays the same. But the number of allocated buffers stays the same.
fn render_model_component(
    context: StoredValue<WgpuContext>,
    render_bind_group_0: StoredValue<shader::bind_groups::BindGroup0>,
    model: ArcReadSignal<ModelInfo>,
    virtual_model: Memo<VirtualModel>,
    meshes: StoredValue<Vec<Mesh>>,
) -> impl Fn(&mut wgpu_profiler::OwningScope<'_, wgpu::RenderPass<'_>>, &virtual_model::ShaderArena)
{
    let indirect_draw_offsets = [
        std::mem::offset_of!(copy_patches::DrawIndexedBuffers, indirect_draw_2),
        std::mem::offset_of!(copy_patches::DrawIndexedBuffers, indirect_draw_4),
        std::mem::offset_of!(copy_patches::DrawIndexedBuffers, indirect_draw_8),
        std::mem::offset_of!(copy_patches::DrawIndexedBuffers, indirect_draw_16),
        std::mem::offset_of!(copy_patches::DrawIndexedBuffers, indirect_draw_32),
    ]
    .map(|v| v as wgpu::BufferAddress);

    move |render_pass: &mut wgpu_profiler::OwningScope<'_, wgpu::RenderPass<'_>>,
          shader_arena: &virtual_model::ShaderArena| {
        let virtual_model = virtual_model.read();
        let shaders = shader_arena
            .get_shader(&virtual_model.shader_key)
            .unwrap_or_else(|| shader_arena.get_missing_shader());
        render_pass.set_pipeline(&shaders.render);

        meshes.with_value(|meshes| {
            for ((bind_group_1, mesh), buffer_offset) in virtual_model
                .render_step
                .bind_group_1
                .iter()
                .zip(meshes.iter())
                .zip(indirect_draw_offsets)
            {
                shader::set_bind_groups(
                    &mut render_pass.recorder,
                    &render_bind_group_0.read_value(),
                    bind_group_1,
                );
                render_pass.set_vertex_buffer(0, mesh.vertex_buffer.slice(..));
                render_pass
                    .set_index_buffer(mesh.index_buffer.slice(..), wgpu::IndexFormat::Uint16);
                render_pass.draw_indexed_indirect(
                    &virtual_model.copy_patches.indirect_draw_buffers,
                    buffer_offset,
                )
            }
        });
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

fn update_models(models: SignalVec<ModelInfo>, game_models: &Vec<ModelInfo>) {
    for (model, game_model) in models.iter_mut().zip(game_models.iter()) {
        if model.with(|model| model.eq(game_model)) {
            continue;
        }
        model.set(game_model.clone());
    }
    match models.len().cmp(&game_models.len()) {
        std::cmp::Ordering::Less => {
            for game_model in game_models.iter().skip(models.len()) {
                models.push(game_model.clone());
            }
        }
        std::cmp::Ordering::Equal => {}
        std::cmp::Ordering::Greater => models.truncate(game_models.len()),
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
