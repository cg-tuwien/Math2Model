pub mod frame_counter;
mod scene;
mod virtual_model;
mod wgpu_context;

use std::{collections::HashMap, sync::Arc};

use encase::ShaderType;
use frame_counter::FrameCounter;
use glam::UVec2;

use reactive_graph::{
    computed::Memo,
    effect::{Effect, RenderEffect},
    owner::{provide_context, use_context, LocalStorage, Owner, StoredValue},
    prelude::*,
    signal::{signal, ArcReadSignal, ReadSignal, RwSignal, WriteSignal},
};
use scene::SceneData;
use virtual_model::{make_missing_shader, ShaderPipelines, VirtualModel};
use wgpu_context::{create_profiler, WgpuContext};
use wgpu_profiler::GpuProfiler;

use crate::{
    buffer::TypedBuffer,
    game::{GameRes, MaterialInfo, ModelInfo, ShaderId},
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
    shaders: RwSignal<HashMap<ShaderId, Arc<ShaderPipelines>>>,
    set_desired_size: WriteSignal<UVec2>,
    set_force_wait: WriteSignal<bool>,
    /// Sets the threshold factor for the LOD algorithm
    set_threshold_factor: WriteSignal<f32>,
    cursor_capture: WindowCursorCapture,
    models: SignalVec<ModelInfo>,
}

const PATCH_SIZES: [u32; 5] = [2, 4, 8, 16, 32];
const MAX_PATCH_COUNT: u32 = 100_000;

#[derive(Clone)]
struct MissingShader(Arc<ShaderPipelines>);

impl GpuApplication {
    pub fn new(context: WgpuContext) -> Self {
        let runtime = Owner::new();
        let context = StoredValue::new(context);
        let profiler = StoredValue::new(create_profiler(&context.read_value()));
        let (desired_size, set_desired_size) = signal(UVec2::new(1, 1));
        let (force_wait, set_force_wait) = signal(false);
        let (threshold_factor, set_threshold_factor) = signal(1.0f32);
        let models = SignalVec::new();

        provide_context(MissingShader(make_missing_shader(&context.read_value())));
        let shaders = RwSignal::new(HashMap::new());

        let render_tree = Owner::with(&runtime, || {
            Arc::new(render_component(
                context,
                profiler,
                desired_size,
                threshold_factor,
                force_wait,
                shaders,
                models.clone(),
            ))
        });

        Self {
            context,
            profiler,
            profiling_enabled: false,
            _runtime: runtime,
            render_tree,
            shaders,

            set_desired_size,
            set_threshold_factor,
            set_force_wait,
            cursor_capture: WindowCursorCapture::Free,
            models,
        }
    }

    pub fn set_shader(&self, shader_id: ShaderId, info: &crate::game::ShaderInfo) {
        self.shaders.update(move |shaders| {
            shaders.insert(
                shader_id,
                Arc::new(ShaderPipelines::new(
                    &info.label,
                    &info.code,
                    &self.context.read_value(),
                )),
            );
        });
    }

    pub fn remove_shader(&self, shader_id: &ShaderId) {
        self.shaders.update(|shaders| {
            shaders.remove(shader_id);
        });
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

        // TODO: Don't do "untrack"
        reactive_graph::graph::untrack(|| {
            update_models(self.models.clone(), &game.models);
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
    shaders: RwSignal<HashMap<ShaderId, Arc<ShaderPipelines>>>,
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
    let quad_meshes = StoredValue::new(context.with_value(|context| {
        PATCH_SIZES
            .iter()
            .map(|size| *size / 2 - 1)
            .map(|splits| Mesh::new_tesselated_quad(&context.device, splits))
            .collect::<Vec<_>>()
    }));

    let models_components = ForEach::new(
        move || models.iter(),
        |model| model.clone(),
        move |model: ArcReadSignal<ModelInfo>| {
            model_component(
                context,
                shaders,
                model.clone(),
                threshold_factor,
                compute_patches,
                copy_patches_pipeline,
                RenderInfo {
                    render_bind_group_0,
                    meshes: quad_meshes,
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

        context.with_value(|context| {
            let mut command_encoder = depth_texture.with_value(|depth_texture| {
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
                    (renderers.lod_stage)(render_data, &mut commands);
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
                        depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                            view: &depth_texture.view,
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

                // Render the models
                models_components.for_each(|renderers| {
                    (renderers.render_stage)(&mut render_pass);
                });

                // Finish the profiler
                std::mem::drop(render_pass);
                std::mem::drop(commands);

                command_encoder
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
        let desired_size = desired_size.get();
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
    shaders: RwSignal<HashMap<ShaderId, Arc<ShaderPipelines>>>,
    model: ArcReadSignal<ModelInfo>,
    threshold_factor: ReadSignal<f32>,
    compute_patches: StoredValue<ComputePatchesStep>,
    copy_patches_pipeline: StoredValue<wgpu::ComputePipeline>,
    render_stage: RenderInfo,
) -> ModelRenderers<
    impl Fn(&FrameData, &mut wgpu_profiler::Scope<'_, wgpu::CommandEncoder>),
    impl Fn(&mut wgpu_profiler::OwningScope<'_, wgpu::RenderPass<'_>>),
> {
    let virtual_model = Memo::new_computed({
        let model = model.clone();
        move |_| {
            let context = context.read_value();
            let meshes = render_stage.meshes.read_value();
            let model = model.read();
            VirtualModel::new(&context, &meshes, &format!("ID{}", model.id))
        }
    });

    let lod_stage_component = lod_stage_component(
        context,
        shaders,
        model.clone(),
        virtual_model,
        compute_patches,
        copy_patches_pipeline,
        threshold_factor,
    );

    let render_component = render_model_component(
        context,
        render_stage.render_bind_group_0,
        shaders,
        model.clone(),
        virtual_model,
        render_stage.meshes,
    );

    ModelRenderers {
        lod_stage: lod_stage_component,
        render_stage: render_component,
    }
}

struct ModelRenderers<LodStage, RenderStage> {
    lod_stage: LodStage,
    render_stage: RenderStage,
}

fn lod_stage_component(
    context: StoredValue<WgpuContext>,
    shaders: RwSignal<HashMap<ShaderId, Arc<ShaderPipelines>>>,
    model: ArcReadSignal<ModelInfo>,
    virtual_model: Memo<VirtualModel>,
    compute_patches: StoredValue<ComputePatchesStep>,
    copy_patches_pipeline: StoredValue<wgpu::ComputePipeline>,
    threshold_factor: ReadSignal<f32>,
) -> impl Fn(&FrameData, &mut wgpu_profiler::Scope<'_, wgpu::CommandEncoder>) {
    let device = &context.read_value().device;
    let id = model.read().id.clone(); // I wonder if this ID stays the same
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

    let force_render_false = TypedBuffer::new_uniform(
        device,
        "Disable Force Render",
        &compute_patches::ForceRenderFlag { flag: 0 },
        wgpu::BufferUsages::COPY_SRC,
    );
    let force_render_true = TypedBuffer::new_uniform(
        device,
        "Enable Force Render",
        &compute_patches::ForceRenderFlag { flag: 1 },
        wgpu::BufferUsages::COPY_SRC,
    );

    let copy_patches_bind_group_0 = Memo::new_computed(move |_| {
        let virtual_model = virtual_model.read();
        let render_buffer = &virtual_model.render_buffer;
        copy_patches::bind_groups::BindGroup0::from_bindings(
            &context.read_value().device,
            copy_patches::bind_groups::BindGroupLayout0 {
                render_buffer_2: render_buffer[0].as_entire_buffer_binding(),
                render_buffer_4: render_buffer[1].as_entire_buffer_binding(),
                render_buffer_8: render_buffer[2].as_entire_buffer_binding(),
                render_buffer_16: render_buffer[3].as_entire_buffer_binding(),
                render_buffer_32: render_buffer[4].as_entire_buffer_binding(),
                indirect_draw: virtual_model.indirect_draw.as_entire_buffer_binding(),
            },
        )
    });

    let input_buffer = RwSignal::new(TypedBuffer::new_uniform(
        &context.read_value().device,
        &format!("{id} Compute Patches Input Buffer"),
        &compute_patches::InputBuffer {
            model_view_projection: glam::Mat4::IDENTITY,
            threshold_factor: 1.0,
        },
        wgpu::BufferUsages::COPY_DST,
    ));

    let patches_buffer_empty = compute_patches::Patches {
        patches_length: 0,
        patches_capacity: 0,
        patches: vec![],
    };
    let patches_buffer = [
        TypedBuffer::new_storage_with_runtime_array(
            device,
            &format!("{id} Patches Buffer 0"),
            &patches_buffer_empty,
            MAX_PATCH_COUNT as u64,
            wgpu::BufferUsages::COPY_SRC | wgpu::BufferUsages::COPY_DST,
        ),
        TypedBuffer::new_storage_with_runtime_array(
            device,
            &format!("{id} Patches Buffer 1"),
            &patches_buffer_empty,
            MAX_PATCH_COUNT as u64,
            wgpu::BufferUsages::COPY_SRC | wgpu::BufferUsages::COPY_DST,
        ),
    ];

    let indirect_compute_empty = compute_patches::DispatchIndirectArgs { x: 0, y: 0, z: 0 };
    let indirect_compute_buffer = [
        TypedBuffer::new_storage(
            device,
            &format!("{id} Indirect Compute Dispatch Buffer 0"),
            &indirect_compute_empty,
            wgpu::BufferUsages::INDIRECT | wgpu::BufferUsages::COPY_DST,
        ),
        TypedBuffer::new_storage(
            device,
            &format!("{id} Indirect Compute Dispatch Buffer 1"),
            &indirect_compute_empty,
            wgpu::BufferUsages::INDIRECT | wgpu::BufferUsages::COPY_DST,
        ),
    ];

    let force_render_uniform = TypedBuffer::new_uniform(
        device,
        &format!("{id} Force Render Uniform"),
        &compute_patches::ForceRenderFlag { flag: 0 },
        wgpu::BufferUsages::COPY_DST,
    );

    let bind_group_1 = Memo::new_computed(move |_| {
        let virtual_model = virtual_model.read();
        let render_buffer = &virtual_model.render_buffer;
        compute_patches::bind_groups::BindGroup1::from_bindings(
            &context.read_value().device,
            compute_patches::bind_groups::BindGroupLayout1 {
                input_buffer: input_buffer.read().as_entire_buffer_binding(),
                render_buffer_2: render_buffer[0].as_entire_buffer_binding(),
                render_buffer_4: render_buffer[1].as_entire_buffer_binding(),
                render_buffer_8: render_buffer[2].as_entire_buffer_binding(),
                render_buffer_16: render_buffer[3].as_entire_buffer_binding(),
                render_buffer_32: render_buffer[4].as_entire_buffer_binding(),
            },
        )
    });
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

    let shader = Memo::new({
        let model = model.clone();
        move |_| {
            let model = model.read();
            let shaders_guard = shaders.read();
            let shader = shaders_guard.get(&model.shader_id).cloned();
            shader.unwrap_or_else(|| use_context::<MissingShader>().unwrap().0.clone())
        }
    });

    move |frame_data: &FrameData, commands: &mut wgpu_profiler::Scope<'_, wgpu::CommandEncoder>| {
        let context = context.read_value();
        let queue = &context.queue;
        let device = &context.device;
        force_render_uniform.write_buffer(queue, &compute_patches::ForceRenderFlag { flag: 0 });
        let model_view_projection = frame_data.camera.projection_matrix(context.size())
            * frame_data.camera.view_matrix()
            * model.read().transform.to_matrix();
        input_buffer.read().write_buffer(
            queue,
            &compute_patches::InputBuffer {
                model_view_projection,
                threshold_factor: threshold_factor.get(),
            },
        );
        patches_buffer[0].write_buffer(
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
        indirect_compute_buffer[0].write_buffer(
            queue,
            &compute_patches::DispatchIndirectArgs { x: 1, y: 1, z: 1 },
        );

        let render_buffer_reset = compute_patches::RenderBuffer {
            patches_length: 0,
            patches_capacity: MAX_PATCH_COUNT,
            patches: vec![],
        };
        for render_buffer in virtual_model.read().render_buffer.iter() {
            render_buffer.write_buffer(queue, &render_buffer_reset);
        }

        if let Some(overriden_lod_stage) = frame_data.lod_stage.as_ref() {
            (overriden_lod_stage)(&model.read().shader_id, &model.read().id);
        } else {
            // Each round, we do a ping-pong and pong-ping
            // 2*4 rounds is enough to subdivide a 4k screen into 16x16 pixel patches
            let double_number_of_rounds = 4;
            for i in 0..double_number_of_rounds {
                let is_last_round = i == double_number_of_rounds - 1;
                // TODO: Should I create many compute passes, or just one?
                {
                    patches_buffer[1].copy_all_from(&patches_buffer_reset, commands);
                    indirect_compute_buffer[1]
                        .copy_all_from(&indirect_compute_buffer_reset, commands);
                    let mut compute_pass = commands
                        .scoped_compute_pass(format!("Compute Patches From-To {i}"), device);
                    compute_pass.set_pipeline(&shader.read().compute_patches);
                    compute_patches::set_bind_groups(
                        &mut compute_pass.recorder,
                        &compute_patches.read_value().bind_group_0,
                        &bind_group_1.read(),
                        &bind_group_2[0],
                    );
                    compute_pass.dispatch_workgroups_indirect(&indirect_compute_buffer[0], 0);
                }
                if is_last_round {
                    // Set to true
                    force_render_uniform.copy_all_from(&force_render_true, commands);
                }
                {
                    patches_buffer[0].copy_all_from(&patches_buffer_reset, commands);
                    indirect_compute_buffer[0]
                        .copy_all_from(&indirect_compute_buffer_reset, commands);

                    let mut compute_pass = commands
                        .scoped_compute_pass(format!("Compute Patches To-From {i}"), device);
                    compute_pass.set_pipeline(&shader.read().compute_patches);
                    compute_patches::set_bind_groups(
                        &mut compute_pass.recorder,
                        // Maybe refactor so that parent components set bind groups, and children just assume that they're set?
                        &compute_patches.read_value().bind_group_0,
                        &bind_group_1.read(),
                        &bind_group_2[1],
                    );
                    compute_pass.dispatch_workgroups_indirect(&indirect_compute_buffer[1], 0);
                }
                if is_last_round {
                    // Set to false
                    force_render_uniform.copy_all_from(&force_render_false, commands);
                }
            }
        }
        {
            let mut compute_pass = commands.scoped_compute_pass("Copy Patch Sizes Pass", device);
            compute_pass.set_pipeline(&copy_patches_pipeline.read_value());
            copy_patches::set_bind_groups(
                &mut compute_pass.recorder,
                &copy_patches_bind_group_0.read(),
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
    shaders: RwSignal<HashMap<ShaderId, Arc<ShaderPipelines>>>,
    model: ArcReadSignal<ModelInfo>,
    virtual_model: Memo<VirtualModel>,
    meshes: StoredValue<Vec<Mesh>>,
) -> impl Fn(&mut wgpu_profiler::OwningScope<'_, wgpu::RenderPass<'_>>) {
    let shader = Memo::new({
        let model = model.clone();
        move |_| {
            let model = model.read();
            let shaders_guard = shaders.read();
            let shader = shaders_guard.get(&model.shader_id).cloned();
            shader.unwrap_or_else(|| use_context::<MissingShader>().unwrap().0.clone())
        }
    });

    let device = &context.read_value().device;

    let model_buffer = TypedBuffer::new_uniform(
        device,
        "Model Buffer",
        &shader::Model {
            model_similarity: glam::Mat4::IDENTITY,
        },
        wgpu::BufferUsages::COPY_DST,
    );

    let material_buffer = TypedBuffer::new_uniform(
        device,
        "Material Buffer",
        &MaterialInfo::missing().to_shader(),
        wgpu::BufferUsages::COPY_DST,
    );

    let bind_group_1 = virtual_model.with(|virtual_model| {
        virtual_model
            .render_buffer
            .iter()
            .map(|render| {
                shader::bind_groups::BindGroup1::from_bindings(
                    device,
                    shader::bind_groups::BindGroupLayout1 {
                        model: model_buffer.as_entire_buffer_binding(),
                        render_buffer: render.as_entire_buffer_binding(),
                        material: material_buffer.as_entire_buffer_binding(),
                    },
                )
            })
            .collect::<Vec<_>>()
    });

    Effect::new(move |_| {
        let model = model.read();
        let queue = &context.read_value().queue;
        model_buffer.write_buffer(
            queue,
            &shader::Model {
                model_similarity: model.transform.to_matrix(),
            },
        );
        material_buffer.write_buffer(queue, &model.material_info.to_shader());
    });

    move |render_pass: &mut wgpu_profiler::OwningScope<'_, wgpu::RenderPass<'_>>| {
        let virtual_model = virtual_model.read();
        render_pass.set_pipeline(&shader.read().render);

        meshes.with_value(|meshes| {
            for (i, (bind_group_1, mesh)) in bind_group_1.iter().zip(meshes.iter()).enumerate() {
                let buffer_offset = (i as u64)
                    * Vec::<copy_patches::DrawIndexedIndirectArgs>::METADATA
                        .extra
                        .stride
                        .get();
                shader::set_bind_groups(
                    &mut render_pass.recorder,
                    &render_bind_group_0.read_value(),
                    bind_group_1,
                );
                render_pass.set_vertex_buffer(0, mesh.vertex_buffer.slice(..));
                render_pass
                    .set_index_buffer(mesh.index_buffer.slice(..), wgpu::IndexFormat::Uint16);
                render_pass.draw_indexed_indirect(&virtual_model.indirect_draw, buffer_offset);
            }
        });
    }
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
