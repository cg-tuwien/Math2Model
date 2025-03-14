use glam::Vec3;
use reactive_graph::{signal::RwSignal, traits::Read};

use crate::{buffer::DeviceBufferExt, mesh::Mesh, shaders, texture::Texture};

use super::{FrameData, get_context, wgpu_context::SurfaceOrFallback};

pub fn skybox_component(
    surface: RwSignal<SurfaceOrFallback>,
) -> impl Fn(&FrameData, &mut wgpu_profiler::OwningScope<'_, wgpu::RenderPass<'_>>) {
    let context = &get_context();
    let skybox_mesh = Mesh::cubemap_cube(&context.device, Vec3::NEG_ONE, Vec3::ONE);

    let context = &get_context();
    let shader = context
        .device
        .create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Skybox"),
            source: wgpu::ShaderSource::Wgsl(shaders::skybox::SOURCE.into()),
        });

    let pipeline = context
        .device
        .create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Skybox"),
            layout: Some(&shaders::skybox::create_pipeline_layout(&context.device)),
            vertex: shaders::skybox::vertex_state(
                &shader,
                &shaders::skybox::vs_main_entry(wgpu::VertexStepMode::Vertex),
            ),
            fragment: Some(shaders::skybox::fragment_state(
                &shader,
                &shaders::skybox::fs_main_entry([
                    Some(wgpu::ColorTargetState {
                        format: context.view_format,
                        blend: Some(wgpu::BlendState::REPLACE),
                        write_mask: wgpu::ColorWrites::ALL,
                    }),
                    Some(wgpu::ColorTargetState {
                        format: wgpu::TextureFormat::R32Uint,
                        blend: None,
                        write_mask: wgpu::ColorWrites::empty(),
                    }),
                ]),
            )),
            primitive: wgpu::PrimitiveState {
                cull_mode: Some(wgpu::Face::Back),
                ..Default::default()
            },
            depth_stencil: Some(wgpu::DepthStencilState {
                format: Texture::DEPTH_FORMAT,
                depth_write_enabled: false,
                // Reverse-Z range: [1.0, 0.0]
                // Depth buffer starts at far-away (0.0)
                // Skybox is at 0.0 (faaar away), and we want to replace the depth buffer values
                depth_compare: wgpu::CompareFunction::GreaterEqual,
                stencil: Default::default(),
                bias: Default::default(),
            }),
            multisample: Default::default(),
            multiview: None,
            cache: Default::default(),
        });

    let uniforms = context.device.uniform_buffer(
        "Skybox Uniforms",
        &shaders::skybox::Uniforms {
            view_projection_matrix: Default::default(),
            background_color: Default::default(),
            sun_direction: Default::default(),
        },
        wgpu::BufferUsages::COPY_DST,
    );

    let bind_group_0 = shaders::skybox::bind_groups::BindGroup0::from_bindings(
        &context.device,
        shaders::skybox::bind_groups::BindGroupLayout0 {
            uniforms: uniforms.as_entire_buffer_binding(),
        },
    );

    move |render_data: &FrameData, render_pass| {
        let context = &get_context();
        let view_matrix =
            glam::Mat4::from_mat3(glam::Mat3::from_mat4(render_data.camera.view_matrix()));
        uniforms.write_buffer(
            &context.queue,
            &shaders::skybox::Uniforms {
                view_projection_matrix: render_data.camera.projection_matrix(surface.read().size())
                    * view_matrix,
                background_color: (Vec3::new(0.09, 0.59, 0.85) * 0.8).extend(1.0),
                sun_direction: Vec3::new(1., 1., 1.).normalize().extend(1.0),
            },
        );

        render_pass.set_pipeline(&pipeline);
        render_pass.set_vertex_buffer(0, skybox_mesh.vertex_buffer.slice(..));
        render_pass.set_index_buffer(
            skybox_mesh.index_buffer.slice(..),
            wgpu::IndexFormat::Uint16,
        );
        bind_group_0.set(&mut render_pass.recorder);
        render_pass.draw_indexed(0..skybox_mesh.num_indices, 0, 0..1);
    }
}
