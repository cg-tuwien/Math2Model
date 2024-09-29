use crate::{buffer::TypedBuffer, camera::Camera, shaders::shader};
use glam::{Mat4, UVec2, Vec2, Vec4};

use super::RenderData;

pub struct SceneData {
    pub time_buffer: TypedBuffer<shader::Time>,
    pub screen_buffer: TypedBuffer<shader::Screen>,
    pub mouse_buffer: TypedBuffer<shader::Mouse>,
    pub camera_buffer: TypedBuffer<shader::Camera>,
    pub light_buffer: TypedBuffer<shader::Lights>,
}

impl SceneData {
    pub fn new(device: &wgpu::Device) -> Self {
        Self {
            time_buffer: TypedBuffer::new_uniform(
                device,
                "Time Buffer",
                &shader::Time {
                    elapsed: 0.0,
                    delta: 1000.0 / 60.0,
                    frame: 0,
                },
                wgpu::BufferUsages::COPY_DST,
            ),
            screen_buffer: TypedBuffer::new_uniform(
                device,
                "Screen Buffer",
                &shader::Screen {
                    resolution: UVec2::ONE,
                    inv_resolution: Vec2::ONE,
                },
                wgpu::BufferUsages::COPY_DST,
            ),
            mouse_buffer: TypedBuffer::new_uniform(
                device,
                "Mouse Buffer",
                &shader::Mouse {
                    pos: Vec2::ZERO,
                    buttons: 0,
                },
                wgpu::BufferUsages::COPY_DST,
            ),
            camera_buffer: TypedBuffer::new_uniform(
                device,
                "Camera Buffer",
                &shader::Camera {
                    view: Mat4::IDENTITY,
                    projection: Mat4::IDENTITY,
                    world_position: Vec4::ZERO,
                },
                wgpu::BufferUsages::COPY_DST,
            ),
            light_buffer: TypedBuffer::new_storage(
                device,
                "Light Buffer",
                &shader::Lights {
                    ambient: Vec4::new(0.1, 0.1, 0.1, 0.0),
                    points_length: 1,
                    points: vec![shader::PointLight {
                        position_range: Vec4::new(0.0, 4.0, 2.0, 40.0),
                        color_intensity: Vec4::new(1.0, 1.0, 1.0, 3.0),
                    }],
                },
                wgpu::BufferUsages::COPY_DST,
            ),
        }
    }

    pub fn as_bind_group_0(&self, device: &wgpu::Device) -> shader::bind_groups::BindGroup0 {
        shader::bind_groups::BindGroup0::from_bindings(
            device,
            shader::bind_groups::BindGroupLayout0 {
                camera: self.camera_buffer.as_entire_buffer_binding(),
                time: self.time_buffer.as_entire_buffer_binding(),
                screen: self.screen_buffer.as_entire_buffer_binding(),
                mouse: self.mouse_buffer.as_entire_buffer_binding(),
                lights: self.light_buffer.as_entire_buffer_binding(),
            },
        )
    }

    pub fn write_buffers(&self, render_data: &RenderData, queue: &wgpu::Queue) {
        self.time_buffer.write_buffer(queue, &render_data.time_data);
        let size = render_data.size;
        self.screen_buffer.write_buffer(
            queue,
            &shader::Screen {
                resolution: size,
                inv_resolution: Vec2::new(1.0 / (size.x as f32), 1.0 / (size.y as f32)),
            },
        );
        self.mouse_buffer
            .write_buffer(queue, &render_data.mouse_data);
        self.camera_buffer.write_buffer(queue, &render_data.camera);
    }
}

impl Camera {
    pub fn to_shader(&self, size: UVec2) -> shader::Camera {
        shader::Camera {
            view: self.view_matrix(),
            projection: self.projection_matrix(size),
            world_position: self.position.extend(1.0),
        }
    }
}
