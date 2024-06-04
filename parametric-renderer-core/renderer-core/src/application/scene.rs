use glamour::{ToRaw, Vector2, Vector4};

use crate::{buffer::TypedBuffer, camera::Camera, shaders::shader};

use super::RenderData;

pub struct SceneData {
    pub time_buffer: TypedBuffer<shader::Time>,
    pub screen_buffer: TypedBuffer<shader::Screen>,
    pub mouse_buffer: TypedBuffer<shader::Mouse>,
    pub camera_buffer: TypedBuffer<shader::Camera>,
    pub light_buffer: TypedBuffer<shader::Lights>,
}

impl SceneData {
    pub fn new(device: &wgpu::Device, camera: &Camera) -> anyhow::Result<Self> {
        let size = camera.size();
        Ok(Self {
            time_buffer: TypedBuffer::new_uniform(
                device,
                "Time Buffer",
                &shader::Time {
                    elapsed: 0.0,
                    delta: 1000.0 / 60.0,
                    frame: 0,
                },
                wgpu::BufferUsages::COPY_DST,
            )?,
            screen_buffer: TypedBuffer::new_uniform(
                device,
                "Screen Buffer",
                &shader::Screen {
                    resolution: size.to_raw(),
                    inv_resolution: Vector2::<f32>::new(
                        1.0 / (size.x as f32),
                        1.0 / (size.y as f32),
                    )
                    .to_raw(),
                },
                wgpu::BufferUsages::COPY_DST,
            )?,
            mouse_buffer: TypedBuffer::new_uniform(
                device,
                "Mouse Buffer",
                &shader::Mouse {
                    pos: Vector2::<f32>::ZERO.to_raw(),
                    buttons: 0,
                },
                wgpu::BufferUsages::COPY_DST,
            )?,
            camera_buffer: TypedBuffer::new_uniform(
                device,
                "Camera Buffer",
                &camera.to_shader(),
                wgpu::BufferUsages::COPY_DST,
            )?,
            light_buffer: TypedBuffer::new_storage(
                device,
                "Light Buffer",
                &shader::Lights {
                    ambient: Vector4::<f32>::new(0.1, 0.1, 0.1, 0.0).to_raw(),
                    points_length: 1,
                    points: vec![shader::PointLight {
                        position_range: Vector4::<f32>::new(0.0, 4.0, 2.0, 40.0).to_raw(),
                        color_intensity: Vector4::<f32>::new(1.0, 1.0, 1.0, 3.0).to_raw(),
                    }],
                },
                wgpu::BufferUsages::COPY_DST,
            )?,
        })
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

    pub fn write_buffers(&mut self, render_data: &RenderData, queue: &wgpu::Queue) {
        self.time_buffer
            .write_buffer(queue, &render_data.time_data)
            .unwrap();
        let size = render_data.camera.size();
        self.screen_buffer
            .write_buffer(
                queue,
                &shader::Screen {
                    resolution: size.to_raw(),
                    inv_resolution: Vector2::<f32>::new(
                        1.0 / (size.x as f32),
                        1.0 / (size.y as f32),
                    )
                    .to_raw(),
                },
            )
            .unwrap();
        self.mouse_buffer
            .write_buffer(queue, &render_data.mouse_data)
            .unwrap();
        self.camera_buffer
            .write_buffer(queue, &render_data.camera.to_shader())
            .unwrap();
    }
}

impl Camera {
    pub fn to_shader(&self) -> shader::Camera {
        shader::Camera {
            view: self.view_matrix().to_raw(),
            projection: self.projection_matrix().to_raw(),
            world_position: self.position.to_raw().extend(1.0),
        }
    }
}
