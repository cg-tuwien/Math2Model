use reactive_graph::{
    computed::Memo,
    signal::RwSignal,
    traits::{Read, Set},
};

use crate::{
    buffer::DeviceBufferExt, mesh::Mesh, reactive::MemoComputed, shaders::ground_plane,
    texture::Texture,
};

use super::{FrameData, get_context, wgpu_context::SurfaceOrFallback};

/// Renders the ground plane
pub fn ground_plane_component(
    surface: RwSignal<SurfaceOrFallback>,
) -> impl Fn(&FrameData, &mut wgpu_profiler::OwningScope<'_, wgpu::RenderPass<'_>>) {
    let context = &get_context();
    let quad_mesh = Mesh::new_tesselated_quad(&context.device, 2);

    let shader_source = RwSignal::new(ground_plane::SOURCE.to_string());

    #[cfg(feature = "desktop")]
    let _file_watcher = {
        let source_path = "./shaders/GroundPlane.wgsl";
        let mut file_watcher = notify_debouncer_full::new_debouncer(
            std::time::Duration::from_millis(1000),
            None,
            move |result: notify_debouncer_full::DebounceEventResult| match result {
                Ok(events) => {
                    let any_modification = events.into_iter().any(|e| e.kind.is_modify());
                    if any_modification {
                        let contents = std::fs::read_to_string(source_path).unwrap();
                        shader_source.set(contents);
                    }
                }
                Err(err) => log::error!("Error watching shaders: {:?}", err),
            },
        )
        .unwrap();
        file_watcher
            .watch(
                "./shaders",
                notify_debouncer_full::notify::RecursiveMode::Recursive,
            )
            .unwrap();
        file_watcher
    };

    let shader = Memo::new_computed({
        move |_| {
            let context = &get_context();
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
            let context = &get_context();
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
                        &ground_plane::fs_main_entry([
                            Some(wgpu::ColorTargetState {
                                format: context.view_format,
                                blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                                write_mask: wgpu::ColorWrites::ALL,
                            }),
                            Some(wgpu::ColorTargetState {
                                format: wgpu::TextureFormat::R32Uint,
                                blend: None,
                                write_mask: wgpu::ColorWrites::empty(),
                            }),
                        ]),
                    )),
                    primitive: Default::default(),
                    depth_stencil: Some(wgpu::DepthStencilState {
                        format: Texture::DEPTH_FORMAT,
                        depth_write_enabled: false,
                        depth_compare: wgpu::CompareFunction::GreaterEqual,
                        stencil: Default::default(),
                        bias: Default::default(),
                    }),
                    multisample: Default::default(),
                    multiview: None,
                    cache: Default::default(),
                })
        }
    });

    let uniforms = context.device.uniform_buffer(
        "Ground Plane Uniforms",
        &ground_plane::Uniforms {
            model_matrix: Default::default(),
            view_projection_matrix: Default::default(),
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
        let context = &get_context();
        #[cfg(feature = "desktop")]
        let _watcher = &_file_watcher;
        let size = 100.0;
        let grid_scale = 1.0;
        uniforms.write_buffer(
            &context.queue,
            &ground_plane::Uniforms {
                model_matrix: glam::Mat4::from_scale_rotation_translation(
                    glam::Vec3::splat(size),
                    glam::Quat::from_rotation_x(-std::f32::consts::FRAC_PI_2),
                    glam::Vec3::new(-size / 2., 0.0, size / 2.),
                ),
                view_projection_matrix: render_data.view_projection_matrix(surface.read().size()),
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
