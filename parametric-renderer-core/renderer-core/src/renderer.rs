mod frame_data;
mod scene;
mod shader_compiler;
mod virtual_model;
mod wgpu_context;

pub use frame_data::FrameData;

use std::{collections::HashMap, sync::Arc};

use encase::ShaderType;
use glam::UVec2;

use reactive_graph::{
    computed::Memo,
    effect::{Effect, RenderEffect},
    owner::{Owner, StoredValue, expect_context, provide_context, use_context},
    prelude::*,
    signal::{
        ArcReadSignal, ArcWriteSignal, ReadSignal, RwSignal, WriteSignal, arc_signal, signal,
    },
};
use scene::SceneData;
use virtual_model::{ShaderPipelines, VirtualModel, make_empty_texture, make_missing_shader};
use wgpu_context::{SurfaceOrFallback, WgpuContext, create_profiler};
use wgpu_profiler::GpuProfiler;

use crate::{
    buffer::{CommandEncoderExt, TypedBuffer},
    game::{GameRes, MaterialInfo, ModelInfo, ShaderId, TextureId, TextureInfo},
    input::WindowCursorCapture,
    mesh::Mesh,
    reactive::{ForEach, MemoComputed, SignalVec},
    shaders::{compute_patches, copy_patches, ground_plane, shader},
    texture::Texture,
    time::{FrameCounter, Seconds},
    window_or_fallback::WindowOrFallback,
};
struct ComputePatchesStep {
    bind_group_0: compute_patches::bind_groups::BindGroup0,
    patches_buffer_reset: TypedBuffer<compute_patches::Patches>,
    indirect_compute_buffer_reset: TypedBuffer<compute_patches::DispatchIndirectArgs>,
    force_render_false: TypedBuffer<compute_patches::ForceRenderFlag>,
    force_render_true: TypedBuffer<compute_patches::ForceRenderFlag>,
}
#[must_use]
pub struct GpuApplicationBuilder {
    pub context: WgpuContext,
    pub surface: SurfaceOrFallback,
}

impl GpuApplicationBuilder {
    #[must_use]
    pub async fn new(window: WindowOrFallback) -> anyhow::Result<Self> {
        let (context, surface) = WgpuContext::new(window).await?;
        Ok(Self { context, surface })
    }

    #[must_use]
    pub fn build(self) -> GpuApplication {
        GpuApplication::new(self.context, self.surface)
    }
}

pub struct GpuApplication {
    pub context: Arc<WgpuContext>,
    surface: RwSignal<SurfaceOrFallback>,
    profiler: StoredValue<GpuProfiler>,
    profiling_enabled: bool,
    runtime: Option<Owner>,
    // render_tree: Arc<dyn Fn(&FrameData) -> Result<RenderResults, wgpu::SurfaceError>>,
    render_effect: RenderEffect<Result<Option<RenderResults>, wgpu::SurfaceError>>,
    set_render_data: ArcWriteSignal<FrameData>,
    shaders: RwSignal<HashMap<ShaderId, Arc<ShaderPipelines>>>,
    textures: RwSignal<HashMap<TextureId, Arc<Texture>>>,
    set_desired_size: WriteSignal<UVec2>,
    set_force_wait: WriteSignal<bool>,
    /// Sets the threshold factor for the LOD algorithm
    set_threshold_factor: WriteSignal<f32>,
    cursor_capture: WindowCursorCapture,
    models: SignalVec<ModelInfo>,
}

const PATCH_SIZES: [u32; 5] = [2, 4, 8, 16, 32];
const MAX_PATCH_COUNT: u32 = 3_000_000;

#[derive(Clone)]
struct MissingShader(Arc<ShaderPipelines>);
#[derive(Clone)]
struct EmptyTexture(Arc<Texture>);

impl GpuApplication {
    pub fn new(context: WgpuContext, surface: SurfaceOrFallback) -> Self {
        let runtime = Owner::new();
        runtime.set();
        let context = Arc::new(context);
        provide_context::<Arc<WgpuContext>>(context.clone());
        let surface = RwSignal::new(surface);
        let profiler = StoredValue::new(create_profiler(&context));
        let (desired_size, set_desired_size) = signal(UVec2::new(1, 1));
        let (force_wait, set_force_wait) = signal(false);
        let (threshold_factor, set_threshold_factor) = signal(1.0f32);
        let models = SignalVec::new();

        provide_context(MissingShader(make_missing_shader(&context)));
        provide_context(EmptyTexture(make_empty_texture(&context)));
        let shaders = RwSignal::new(HashMap::new());
        let textures = RwSignal::new(HashMap::new());

        let render_tree = Arc::new(render_component(
            surface,
            profiler,
            desired_size,
            threshold_factor,
            force_wait,
            shaders,
            textures,
            models.clone(),
        ));

        let (render_data, set_render_data) = arc_signal(FrameData::default());
        let render_effect = RenderEffect::new(move |_| (render_tree)(&render_data.read()));

        Self {
            context,
            surface,
            profiler,
            profiling_enabled: false,
            runtime: Some(runtime),
            render_effect,
            set_render_data,
            shaders,
            textures,

            set_desired_size,
            set_threshold_factor,
            set_force_wait,
            cursor_capture: WindowCursorCapture::Free,
            models,
        }
    }

    pub fn update_models(&self, game_models: &[ModelInfo]) {
        reactive_graph::graph::untrack(|| {
            for (model, game_model) in self.models.iter_mut().zip(game_models.iter()) {
                if model.with_untracked(|model| model.eq(game_model)) {
                    continue;
                }
                model.set(game_model.clone());
            }
            match self.models.len().cmp(&game_models.len()) {
                std::cmp::Ordering::Less => {
                    for game_model in game_models.iter().skip(self.models.len()) {
                        self.models.push(game_model.clone());
                    }
                }
                std::cmp::Ordering::Equal => {}
                std::cmp::Ordering::Greater => self.models.truncate(game_models.len()),
            }
        });
    }

    pub fn set_shader(
        &self,
        shader_id: ShaderId,
        info: &crate::game::ShaderInfo,
        on_shader_compiled: Option<Arc<dyn Fn(&ShaderId, Vec<wgpu::CompilationMessage>)>>,
    ) {
        let shaders = self.shaders;
        let new_shaders = ShaderPipelines::new(&info.label, &info.code, &self.context);
        any_spawner::Executor::spawn_local(async move {
            let compilation_results = new_shaders.get_compilation_info().await;
            let is_error = compilation_results
                .iter()
                .any(|v| v.message_type == wgpu::CompilationMessageType::Error);
            on_shader_compiled.map(|f| f(&shader_id, compilation_results));
            if !is_error {
                shaders.update(move |shaders| {
                    shaders.insert(shader_id, Arc::new(new_shaders));
                });
            }
        });
    }

    pub fn remove_shader(&self, shader_id: &ShaderId) {
        self.shaders.update(|shaders| {
            shaders.remove(shader_id);
        });
    }

    pub fn set_texture(&mut self, id: TextureId, info: &TextureInfo) {
        let texture = Texture::new_rgba(&self.context.device, &self.context.queue, info);
        self.textures.update(move |textures| {
            textures.insert(id, Arc::new(texture));
        });
    }

    pub fn remove_texture(&mut self, id: &TextureId) {
        self.textures.update(|textures| {
            textures.remove(id);
        });
    }

    pub fn render(&mut self, game: &GameRes) -> Result<Option<RenderResults>, wgpu::SurfaceError> {
        self.update_cursor_capture(game.cursor_capture);

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

        self.set_render_data.set(FrameData {
            camera: game.camera.clone(),
            mouse_pos: game.mouse,
            mouse_held: game.mouse_held,
            lod_stage: game.lod_stage.clone(),
        });
        //  TODO: We need a custom executor, I think. On wasm, we'll need to hook up the futures thingy
        // any_spawner::Executor::poll_local();
        // TODO: This currently gets the value of the last frame.
        self.render_effect
            .take_value()
            .unwrap_or(Err(wgpu::SurfaceError::Timeout)) // Temporary hack
    }

    pub fn resize(&self, new_size: UVec2) {
        self.set_desired_size.set(new_size);
    }

    pub fn force_wait(&self) {
        self.set_force_wait.set(true);
    }

    pub fn set_threshold_factor(&self, factor: f32) {
        self.set_threshold_factor
            .set(factor.clamp(0.0001, 100000.0));
    }

    fn update_cursor_capture(&mut self, cursor_capture: WindowCursorCapture) {
        if let Some(window) = self.surface.with_untracked(|surface| match surface {
            wgpu_context::SurfaceOrFallback::Surface { window, .. } => Some(window.clone()),
            wgpu_context::SurfaceOrFallback::Fallback { .. } => None,
        }) {
            self.cursor_capture.update(cursor_capture, &window);
        }
    }
}

impl Drop for GpuApplication {
    fn drop(&mut self) {
        // Make sure to dispose of the Leptos runtime
        if let Some(runtime) = self.runtime.take() {
            runtime.unset();
        }
    }
}

fn wgpu_context() -> Arc<WgpuContext> {
    expect_context::<Arc<WgpuContext>>()
}

/// We're using Leptos :)
fn render_component(
    surface: RwSignal<SurfaceOrFallback>,
    profiler: StoredValue<GpuProfiler>,
    desired_size: ReadSignal<UVec2>,
    threshold_factor: ReadSignal<f32>,
    force_wait: ReadSignal<bool>,
    shaders: RwSignal<HashMap<ShaderId, Arc<ShaderPipelines>>>,
    textures: RwSignal<HashMap<TextureId, Arc<Texture>>>,
    models: SignalVec<ModelInfo>,
) -> impl Fn(&FrameData) -> Result<Option<RenderResults>, wgpu::SurfaceError> {
    let context = &wgpu_context();
    let frame_counter = RwSignal::new(FrameCounter::new());
    let new_frame_time = move || frame_counter.write().new_frame();

    let depth_texture = Memo::new_computed(move |_| {
        Texture::create_depth_texture(
            &wgpu_context().device,
            surface.read().size(),
            "Depth Texture",
        )
    });

    let scene_data = StoredValue::new(SceneData::new(&context.device));
    let render_bind_group_0 = StoredValue::new(
        scene_data.with_value(|scene_data| scene_data.as_bind_group_0(&context.device)),
    );

    let compute_patches = StoredValue::new(ComputePatchesStep {
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
        patches_buffer_reset: TypedBuffer::new_storage_with_runtime_array(
            &context.device,
            "Patches Buffer Reset",
            &compute_patches::Patches {
                patches_length: 0,
                patches_capacity: MAX_PATCH_COUNT,
                patches: vec![],
            },
            1,
            wgpu::BufferUsages::COPY_SRC,
        ),
        indirect_compute_buffer_reset: TypedBuffer::new_storage(
            &context.device,
            "Indirect Compute Dispatch Buffer Reset",
            // We only write to x. y and z have their default value.
            &compute_patches::DispatchIndirectArgs { x: 0, y: 1, z: 1 },
            wgpu::BufferUsages::COPY_SRC,
        ),
        force_render_false: TypedBuffer::new_uniform(
            &context.device,
            "Disable Force Render",
            &compute_patches::ForceRenderFlag { flag: 0 },
            wgpu::BufferUsages::COPY_SRC,
        ),
        force_render_true: TypedBuffer::new_uniform(
            &context.device,
            "Enable Force Render",
            &compute_patches::ForceRenderFlag { flag: 1 },
            wgpu::BufferUsages::COPY_SRC,
        ),
    });

    let copy_patches_pipeline = StoredValue::new(context.device.create_compute_pipeline(
        &wgpu::ComputePipelineDescriptor {
            label: Some("Copy Patches"),
            layout: Some(&copy_patches::create_pipeline_layout(&context.device)),
            module: &copy_patches::create_shader_module(&context.device),
            entry_point: Some(copy_patches::ENTRY_MAIN),
            compilation_options: Default::default(),
            cache: Default::default(),
        },
    ));

    // A reactive effect that reruns whenever any of its signals change
    Effect::new({
        let new_size = Memo::new(move |_| desired_size.get());
        move |_| {
            surface.write().try_resize(&wgpu_context(), new_size.get());
        }
    });

    // size/2 - 1 == one quad per four pixels
    let quad_meshes = StoredValue::new(
        PATCH_SIZES
            .iter()
            .map(|size| *size / 2 - 1)
            .map(|splits| Mesh::new_tesselated_quad(&context.device, splits))
            .collect::<Vec<_>>(),
    );

    let ground_plane_component = ground_plane_component(surface);

    let models_components = ForEach::new(
        move || models.iter(),
        |model| model.get_untracked().id.clone(),
        {
            move |model: ArcReadSignal<ModelInfo>| {
                model_component(
                    surface,
                    shaders,
                    textures,
                    model.clone(),
                    threshold_factor,
                    compute_patches,
                    copy_patches_pipeline,
                    RenderInfo {
                        render_bind_group_0,
                        meshes: quad_meshes,
                    },
                )
            }
        },
    );

    move |render_data: &FrameData| {
        let context = &wgpu_context();
        let frame_time = new_frame_time();
        // 2. Render
        let surface_texture = match surface.with(|surface| surface.surface_texture(&context)) {
            Ok(v) => v,
            err @ Err(wgpu::SurfaceError::Lost | wgpu::SurfaceError::Outdated) => {
                // Roughly based on https://github.com/gfx-rs/wgpu/blob/a0c185a28c232ee2ab63f72d6fd3a63a3f787309/examples/src/framework.rs#L216
                surface.update(|surface| surface.recreate_swapchain(&context));
                return err.map(|_| None);
            }
            err => {
                return err.map(|_| None);
            }
        };

        scene_data.read_value().write_buffers(
            surface.read().size(),
            &render_data,
            &frame_time,
            &context.queue,
        );

        let mut command_encoder = {
            let mut command_encoder =
                context
                    .device
                    .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                        label: Some("Render Encoder"),
                    });
            // Profiling
            let profiler_guard = profiler.read_value();
            let mut commands = profiler_guard.scope("Render", &mut command_encoder);

            models_components.for_each(|v| {
                (v.lod_stage)(render_data, &mut commands);
            });

            let mut render_pass = commands.scoped_render_pass(
                "Render Pass",
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
                        view: &depth_texture.read().view,
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
            models_components.for_each(|v| {
                (v.render_stage)(&mut render_pass);
            });

            // Render transparent objects
            (ground_plane_component)(render_data, &mut render_pass);

            // Finish the profiler
            std::mem::drop(render_pass);
            std::mem::drop(commands);

            command_encoder
        };
        profiler.update_value(|profiler| profiler.resolve_queries(&mut command_encoder));
        context
            .queue
            .submit(std::iter::once(command_encoder.finish()));

        surface_texture.present();

        let render_results = {
            let delta_time = frame_time.delta;
            let mut profiler = profiler.write_value();
            profiler.end_frame().unwrap();
            let profiler_results =
                profiler.process_finished_frame(context.queue.get_timestamp_period());
            Some(RenderResults {
                delta_time,
                profiler_results,
            })
        };

        if force_wait.get() {
            context.instance.poll_all(true);
        }

        Ok(render_results)
    }
}

/// Renders the ground plane
fn ground_plane_component(
    surface: RwSignal<SurfaceOrFallback>,
) -> impl Fn(&FrameData, &mut wgpu_profiler::OwningScope<'_, wgpu::RenderPass<'_>>) {
    let context = &wgpu_context();
    let quad_mesh = Mesh::new_tesselated_quad(&context.device, 2);

    let shader_source = RwSignal::new(ground_plane::SOURCE.to_string());

    // #[cfg(feature = "desktop")]
    // let _file_watcher = {
    //     let source_path = "./shaders/GroundPlane.wgsl";
    //     let mut file_watcher = notify_debouncer_full::new_debouncer(
    //         std::time::Duration::from_millis(1000),
    //         None,
    //         move |result: notify_debouncer_full::DebounceEventResult| match result {
    //             Ok(events) => {
    //                 let any_modification = events.into_iter().any(|e| e.kind.is_modify());
    //                 if any_modification {
    //                     let contents = std::fs::read_to_string(source_path).unwrap();
    //                     shader_source.set(contents);
    //                 }
    //             }
    //             Err(err) => log::error!("Error watching shaders: {:?}", err),
    //         },
    //     )
    //     .unwrap();
    //     file_watcher
    //         .watch(
    //             "./shaders",
    //             notify_debouncer_full::notify::RecursiveMode::Recursive,
    //         )
    //         .unwrap();
    //     file_watcher
    // };

    let shader = Memo::new_computed({
        move |_| {
            let context = &wgpu_context();
            context
                .device
                .create_shader_module(wgpu::ShaderModuleDescriptor {
                    label: Some("Ground Plane Grid"),
                    source: wgpu::ShaderSource::Wgsl(shader_source.read().as_str().into()),
                })
        }
    });

    let pipeline = Memo::new_computed({
        move |_| {
            let context = &wgpu_context();
            let shader = shader.read();
            context
                .device
                .create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                    label: Some("Ground Plane Grid"),
                    layout: Some(&ground_plane::create_pipeline_layout(&context.device)),
                    vertex: ground_plane::vertex_state(
                        &shader,
                        &ground_plane::vs_main_entry(wgpu::VertexStepMode::Vertex),
                    ),
                    fragment: Some(ground_plane::fragment_state(
                        &shader,
                        &ground_plane::fs_main_entry([Some(wgpu::ColorTargetState {
                            format: context.view_format,
                            blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                            write_mask: wgpu::ColorWrites::ALL,
                        })]),
                    )),
                    primitive: Default::default(),
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
    });

    let uniforms = TypedBuffer::new_uniform(
        &context.device,
        "Ground Plane Uniforms",
        &ground_plane::Uniforms {
            model_matrix: glam::Mat4::IDENTITY,
            view_projection_matrix: glam::Mat4::IDENTITY,
            grid_scale: 0.0,
        },
        wgpu::BufferUsages::COPY_DST,
    );

    let bind_group_0 = ground_plane::bind_groups::BindGroup0::from_bindings(
        &context.device,
        ground_plane::bind_groups::BindGroupLayout0 {
            uniforms: uniforms.as_entire_buffer_binding(),
        },
    );

    move |render_data: &FrameData, render_pass| {
        let context = &wgpu_context();
        // #[cfg(feature = "desktop")]
        // let _watcher = &_file_watcher;
        let size = 100_000.0;
        let inv_grid_scale = (render_data.camera.position.distance(glam::Vec3::ZERO) / 50.0)
            .max(1.0)
            .log2()
            .ceil()
            .exp2()
            .max(1.0);
        let grid_scale = 1.0 / inv_grid_scale;
        uniforms.write_buffer(
            &context.queue,
            &ground_plane::Uniforms {
                model_matrix: glam::Mat4::from_scale_rotation_translation(
                    glam::Vec3::splat(size),
                    glam::Quat::from_rotation_x(-std::f32::consts::FRAC_PI_2),
                    glam::Vec3::new(-size / 2., 0.0, size / 2.),
                ),
                view_projection_matrix: render_data.camera.projection_matrix(surface.read().size())
                    * render_data.camera.view_matrix(),
                grid_scale,
            },
        );

        render_pass.set_pipeline(&pipeline.read());
        render_pass.set_vertex_buffer(0, quad_mesh.vertex_buffer.slice(..));
        render_pass.set_index_buffer(quad_mesh.index_buffer.slice(..), wgpu::IndexFormat::Uint16);
        bind_group_0.set(&mut render_pass.recorder);
        render_pass.draw_indexed(0..quad_mesh.num_indices, 0, 0..1);
    }
}

/// Returns multiple render functions
fn model_component(
    surface: RwSignal<SurfaceOrFallback>,
    shaders: RwSignal<HashMap<ShaderId, Arc<ShaderPipelines>>>,
    textures: RwSignal<HashMap<TextureId, Arc<Texture>>>,
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
            let meshes = render_stage.meshes.read_value();
            let model = model.read();
            VirtualModel::new(&wgpu_context(), &meshes, &format!("ID{}", model.id))
        }
    });

    let lod_stage_component = lod_stage_component(
        surface,
        shaders,
        model.clone(),
        virtual_model,
        compute_patches,
        copy_patches_pipeline,
        threshold_factor,
    );

    let render_component = render_model_component(
        render_stage.render_bind_group_0,
        shaders,
        textures,
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
    surface: RwSignal<SurfaceOrFallback>,
    shaders: RwSignal<HashMap<ShaderId, Arc<ShaderPipelines>>>,
    model: ArcReadSignal<ModelInfo>,
    virtual_model: Memo<VirtualModel>,
    compute_patches: StoredValue<ComputePatchesStep>,
    copy_patches_pipeline: StoredValue<wgpu::ComputePipeline>,
    threshold_factor: ReadSignal<f32>,
) -> impl Fn(&FrameData, &mut wgpu_profiler::Scope<'_, wgpu::CommandEncoder>) {
    let context = &wgpu_context();
    let device = &context.device;
    let id = model.read_untracked().id.clone(); // I wonder if this ID stays the same

    let copy_patches_bind_group_0 = Memo::new_computed(move |_| {
        let virtual_model = virtual_model.read();
        let render_buffer = &virtual_model.render_buffer;
        copy_patches::bind_groups::BindGroup0::from_bindings(
            &wgpu_context().device,
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
        &context.device,
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

    let indirect_compute_buffer = [
        TypedBuffer::new_storage(
            device,
            &format!("{id} Indirect Compute Dispatch Buffer 0"),
            // None of these values will ever be read
            &compute_patches::DispatchIndirectArgs { x: 0, y: 0, z: 0 },
            wgpu::BufferUsages::INDIRECT | wgpu::BufferUsages::COPY_DST,
        ),
        TypedBuffer::new_storage(
            device,
            &format!("{id} Indirect Compute Dispatch Buffer 1"),
            &compute_patches::DispatchIndirectArgs { x: 0, y: 0, z: 0 },
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
        let context = &wgpu_context();
        let virtual_model = virtual_model.read();
        let render_buffer = &virtual_model.render_buffer;
        compute_patches::bind_groups::BindGroup1::from_bindings(
            &context.device,
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
            shader.unwrap_or_else(|| expect_context::<MissingShader>().0.clone())
        }
    });

    move |frame_data: &FrameData, commands: &mut wgpu_profiler::Scope<'_, wgpu::CommandEncoder>| {
        let context = &wgpu_context();
        let queue = &context.queue;
        force_render_uniform.write_buffer(queue, &compute_patches::ForceRenderFlag { flag: 0 });
        let model_view_projection = frame_data.camera.projection_matrix(surface.read().size())
            * frame_data.camera.view_matrix()
            * model.read().transform.to_matrix();
        input_buffer.read().write_buffer(
            queue,
            &compute_patches::InputBuffer {
                model_view_projection,
                threshold_factor: threshold_factor.get(),
            },
        );
        let instance_count = model.with(|v| v.instance_count);
        patches_buffer[0].write_buffer(
            queue,
            &compute_patches::Patches {
                patches_length: instance_count,
                patches_capacity: MAX_PATCH_COUNT,
                patches: (0..instance_count)
                    .map(|i| {
                        compute_patches::EncodedPatch {
                            // Just the leading 1 bit
                            u: 1,
                            v: 1,
                            instance: i,
                        }
                    })
                    .collect(),
            },
        );
        indirect_compute_buffer[0].write_buffer(
            queue,
            &compute_patches::DispatchIndirectArgs {
                x: instance_count,
                y: 1,
                z: 1,
            },
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
            let compute_patches = compute_patches.read_value();
            let double_number_of_rounds = 4;
            for i in 0..double_number_of_rounds {
                let is_last_round = i == double_number_of_rounds - 1;
                // TODO: Should I create many compute passes, or just one?
                {
                    commands.copy_tbuffer_to_tbuffer(
                        &compute_patches.patches_buffer_reset,
                        &patches_buffer[1],
                    );
                    commands.copy_tbuffer_to_tbuffer(
                        &compute_patches.indirect_compute_buffer_reset,
                        &indirect_compute_buffer[1],
                    );
                    let mut compute_pass =
                        commands.scoped_compute_pass(format!("Compute Patches From-To {i}"));
                    compute_pass.set_pipeline(&shader.read().compute_patches);
                    compute_patches::set_bind_groups(
                        &mut compute_pass.recorder,
                        &compute_patches.bind_group_0,
                        &bind_group_1.read(),
                        &bind_group_2[0],
                    );
                    compute_pass.dispatch_workgroups_indirect(&indirect_compute_buffer[0], 0);
                }
                if is_last_round {
                    commands.copy_tbuffer_to_tbuffer(
                        &compute_patches.force_render_true,
                        &force_render_uniform,
                    );
                }
                {
                    commands.copy_tbuffer_to_tbuffer(
                        &compute_patches.patches_buffer_reset,
                        &patches_buffer[0],
                    );
                    commands.copy_tbuffer_to_tbuffer(
                        &compute_patches.indirect_compute_buffer_reset,
                        &indirect_compute_buffer[0],
                    );
                    let mut compute_pass =
                        commands.scoped_compute_pass(format!("Compute Patches To-From {i}"));
                    compute_pass.set_pipeline(&shader.read().compute_patches);
                    compute_patches::set_bind_groups(
                        &mut compute_pass.recorder,
                        // Maybe refactor so that parent components set bind groups, and children just assume that they're set?
                        &compute_patches.bind_group_0,
                        &bind_group_1.read(),
                        &bind_group_2[1],
                    );
                    compute_pass.dispatch_workgroups_indirect(&indirect_compute_buffer[1], 0);
                }
                if is_last_round {
                    commands.copy_tbuffer_to_tbuffer(
                        &compute_patches.force_render_false,
                        &force_render_uniform,
                    );
                }
            }
        }
        {
            let mut compute_pass = commands.scoped_compute_pass("Copy Patch Sizes Pass");
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
    render_bind_group_0: StoredValue<shader::bind_groups::BindGroup0>,
    shaders: RwSignal<HashMap<ShaderId, Arc<ShaderPipelines>>>,
    textures: RwSignal<HashMap<TextureId, Arc<Texture>>>,
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
    let texture = Memo::new_computed({
        let model = model.clone();
        move |_| {
            let model = model.read();
            model
                .material_info
                .diffuse_texture
                .as_ref()
                .and_then(|id| textures.with(|t| t.get(&id).cloned()))
                .unwrap_or_else(|| use_context::<EmptyTexture>().unwrap().0.clone())
        }
    });

    let context = &wgpu_context();
    let device = &context.device;

    let model_buffer = RwSignal::new(TypedBuffer::new_uniform(
        device,
        "Model Buffer",
        &shader::Model {
            model_similarity: glam::Mat4::IDENTITY,
        },
        wgpu::BufferUsages::COPY_DST,
    ));

    let material_buffer = RwSignal::new(TypedBuffer::new_uniform(
        device,
        "Material Buffer",
        &MaterialInfo::missing().to_shader(),
        wgpu::BufferUsages::COPY_DST,
    ));

    let bind_group_1 = Memo::new_computed({
        let model = model.clone();
        move |_| {
            let _m = model.read();
            let context = &wgpu_context();
            let device = &context.device;
            let model_buffer = model_buffer.read();
            let material_buffer = material_buffer.read();
            let t_diffuse = texture.read();
            virtual_model
                .read()
                .render_buffer
                .iter()
                .map(|render| {
                    shader::bind_groups::BindGroup1::from_bindings(
                        device,
                        shader::bind_groups::BindGroupLayout1 {
                            model: model_buffer.as_entire_buffer_binding(),
                            render_buffer: render.as_entire_buffer_binding(),
                            material: material_buffer.as_entire_buffer_binding(),
                            t_diffuse: &t_diffuse.view,
                        },
                    )
                })
                .collect::<Vec<_>>()
        }
    });
    Effect::new(move |_| {
        let model = model.read();
        let queue = &wgpu_context().queue;
        model_buffer.read().write_buffer(
            queue,
            &shader::Model {
                model_similarity: model.transform.to_matrix(),
            },
        );
        material_buffer
            .read()
            .write_buffer(queue, &model.material_info.to_shader());
    });

    move |render_pass: &mut wgpu_profiler::OwningScope<'_, wgpu::RenderPass<'_>>| {
        let virtual_model = virtual_model.read();
        render_pass.set_pipeline(&shader.read().render);

        meshes.with_value(|meshes| {
            for (i, (bind_group_1, mesh)) in
                bind_group_1.read().iter().zip(meshes.iter()).enumerate()
            {
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

#[derive(Default)]
pub struct RenderResults {
    pub delta_time: Seconds,
    pub profiler_results: Option<Vec<wgpu_profiler::GpuTimerQueryResult>>,
}
