use crate::{buffer::TypedBuffer, camera::Camera, shaders::shader, time::FrameTime};
use glam::{Mat4, UVec2, Vec2, Vec4};

use super::FrameData;

pub struct SceneData {
    pub time_buffer: TypedBuffer<shader::Time>,
    pub screen_buffer: TypedBuffer<shader::Screen>,
    pub mouse_buffer: TypedBuffer<shader::Mouse>,
    pub extra_buffer: TypedBuffer<shader::Extra>,
    pub camera_buffer: TypedBuffer<shader::Camera>,
    pub light_buffer: TypedBuffer<shader::Lights>,
    pub linear_sampler: wgpu::Sampler,
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
            extra_buffer: TypedBuffer::new_uniform(
                device,
                "Mouse Buffer",
                &shader::Extra { hot_value: 0. },
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
                    ambient: Vec4::new(0.05, 0.05, 0.05, 0.0),
                    points_length: 4,
                    points: vec![
                        shader::LightSource {
                            position_range: glam::Vec3::new(1.0, -4.0, 1.0).normalize().extend(1.0),
                            color_intensity: Vec4::new(0.5, 0.55, 0.5, 0.9),
                            light_type: shader::LIGHT_TYPE_DIRECTIONAL,
                        },
                        shader::LightSource {
                            position_range: Vec4::new(0.0, 8.0, 4.0, 80.0),
                            color_intensity: Vec4::new(1.0, 1.0, 1.0, 1.0),
                            light_type: shader::LIGHT_TYPE_POINT,
                        },
                        shader::LightSource {
                            position_range: Vec4::new(1.0, 8.0, -6.0, 70.0),
                            color_intensity: Vec4::new(1.0, 1.0, 1.0, 1.5),
                            light_type: shader::LIGHT_TYPE_POINT,
                        },
                        shader::LightSource {
                            position_range: Vec4::new(0.0, -8.0, 0.0, 80.0),
                            color_intensity: Vec4::new(0.8, 0.8, 1.0, 0.9),
                            light_type: shader::LIGHT_TYPE_POINT,
                        },
                    ],
                },
                wgpu::BufferUsages::COPY_DST,
            ),
            linear_sampler: device.create_sampler(&wgpu::SamplerDescriptor {
                address_mode_u: wgpu::AddressMode::Repeat,
                address_mode_v: wgpu::AddressMode::Repeat,
                address_mode_w: wgpu::AddressMode::Repeat,
                mag_filter: wgpu::FilterMode::Linear,
                min_filter: wgpu::FilterMode::Linear,
                mipmap_filter: wgpu::FilterMode::Nearest,
                ..Default::default()
            }),
        }
    }

    pub fn as_bind_group_0(&self, device: &wgpu::Device) -> shader::bind_groups::BindGroup0 {
        shader::bind_groups::BindGroup0::from_bindings(
            device,
            shader::bind_groups::BindGroupLayout0 {
                camera: self.camera_buffer.as_entire_buffer_binding(),
                time: self.time_buffer.as_entire_buffer_binding(),
                screen: self.screen_buffer.as_entire_buffer_binding(),
                extra: self.extra_buffer.as_entire_buffer_binding(),
                mouse: self.mouse_buffer.as_entire_buffer_binding(),
                lights: self.light_buffer.as_entire_buffer_binding(),
                linear_sampler: &self.linear_sampler,
            },
        )
    }

    pub fn write_buffers(
        &self,
        size: UVec2,
        render_data: &FrameData,
        frame_time: &FrameTime,
        queue: &wgpu::Queue,
    ) {
        self.time_buffer.write_buffer(
            queue,
            &shader::Time {
                elapsed: frame_time.elapsed.0,
                delta: frame_time.delta.0,
                frame: frame_time.frame as u32,
            },
        );
        self.screen_buffer.write_buffer(
            queue,
            &shader::Screen {
                resolution: size,
                inv_resolution: Vec2::new(1.0 / (size.x as f32), 1.0 / (size.y as f32)),
            },
        );
        self.mouse_buffer.write_buffer(
            queue,
            &shader::Mouse {
                pos: render_data.mouse_pos,
                buttons: if render_data.mouse_held { 1 } else { 0 },
            },
        );
        self.camera_buffer
            .write_buffer(queue, &render_data.camera.to_shader(size));
    }
}

impl Camera {
    fn to_shader(&self, size: UVec2) -> shader::Camera {
        shader::Camera {
            view: self.view_matrix(),
            projection: self.projection_matrix(size),
            world_position: self.position.extend(1.0),
        }
    }
}
