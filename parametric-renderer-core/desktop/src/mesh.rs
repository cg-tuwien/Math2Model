use glam::Vec3;
use glamour::{Matrix4, Point3, ToRaw, Vector3};
use wgpu::util::DeviceExt;

use crate::shaders::shader;

pub const QUAD_VERTICES: [Vec3; 4] = [
    Vec3::new(-0.5, -0.5, 0.0),
    Vec3::new(0.5, -0.5, 0.0),
    Vec3::new(0.5, 0.5, 0.0),
    Vec3::new(-0.5, 0.5, 0.0),
];
pub const QUAD_INDICES: [u16; 6] = [0, 1, 2, 2, 3, 0];

pub struct Transform {
    position: Point3,
    rotation: glam::Quat,
    scale: f32,
}

impl Transform {
    pub fn to_matrix(&self) -> Matrix4<f32> {
        Matrix4::from_scale_rotation_translation(
            Vector3::new(self.scale, self.scale, self.scale),
            self.rotation,
            self.position.to_vector(),
        )
    }
}

pub struct Mesh {
    pub transform: Transform,
    pub vertex_buffer: wgpu::Buffer,
    pub index_buffer: wgpu::Buffer,
    pub instance_buffer: wgpu::Buffer,
    pub num_indices: u32,
}
impl Mesh {
    pub fn new_quad(device: &wgpu::Device) -> Self {
        let transform = Transform {
            position: Point3::new(0.0, 0.0, 0.0),
            rotation: glam::Quat::IDENTITY,
            scale: 1.0,
        };
        let vertex_buffer_contents = QUAD_VERTICES
            .iter()
            .map(|&position| shader::VertexInput { position })
            .collect::<Vec<_>>();

        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Vertex Buffer"),
            contents: bytemuck::cast_slice(&vertex_buffer_contents),
            usage: wgpu::BufferUsages::VERTEX,
        });

        let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Index Buffer"),
            contents: bytemuck::cast_slice(&QUAD_INDICES),
            usage: wgpu::BufferUsages::INDEX,
        });

        let instance_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Instance Buffer"),
            contents: bytemuck::cast_slice(&[Mesh::get_instance_input(&transform)]),
            usage: wgpu::BufferUsages::VERTEX,
        });

        Self {
            transform,
            vertex_buffer,
            index_buffer,
            instance_buffer,
            num_indices: QUAD_INDICES.len() as u32,
        }
    }

    pub fn get_instance_input(transform: &Transform) -> shader::InstanceInput {
        let model = transform.to_matrix().to_raw();
        shader::InstanceInput {
            model_similarity_0: model.x_axis,
            model_similarity_1: model.y_axis,
            model_similarity_2: model.z_axis,
            model_similarity_3: model.w_axis,
        }
    }
}
