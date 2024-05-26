use glam::{Vec2, Vec3};
use glamour::{Matrix4, Point3, Vector3};
use wgpu::util::DeviceExt;

use crate::shaders::shader;

pub struct Transform {
    pub position: Point3,
    pub rotation: glam::Quat,
    pub scale: f32,
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

impl Default for Transform {
    fn default() -> Self {
        Self {
            position: Point3::new(0.0, 0.0, 0.0),
            rotation: glam::Quat::IDENTITY,
            scale: 1.0,
        }
    }
}

pub struct Mesh {
    pub transform: Transform,
    pub vertex_buffer: wgpu::Buffer,
    pub index_buffer: wgpu::Buffer,
    pub num_indices: u32,
}
impl Mesh {
    fn with_contents(
        device: &wgpu::Device,
        vertex_buffer_contents: &[shader::VertexInput],
        index_buffer_contents: &[u16],
    ) -> Self {
        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Vertex Buffer"),
            contents: bytemuck::cast_slice(vertex_buffer_contents),
            usage: wgpu::BufferUsages::VERTEX,
        });

        let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Index Buffer"),
            contents: bytemuck::cast_slice(index_buffer_contents),
            usage: wgpu::BufferUsages::INDEX,
        });

        Self {
            transform: Default::default(),
            vertex_buffer,
            index_buffer,
            num_indices: index_buffer_contents.len() as u32,
        }
    }

    /// Create a new tesselated quad mesh
    pub fn new_tesselated_quad(device: &wgpu::Device, split_count: u32) -> Self {
        let mut vertices = Vec::new();
        let mut indices = Vec::new();

        let quad_count_one_side = split_count + 1;

        for i in 0..=quad_count_one_side {
            for j in 0..=quad_count_one_side {
                let position = Vec3::new(i as f32, j as f32, 0.0) / (quad_count_one_side as f32);
                let uv = Vec2::new(i as f32, j as f32) / (quad_count_one_side as f32);
                vertices.push(shader::VertexInput { position, uv });
            }
        }

        let vertices_per_row = quad_count_one_side + 1;
        for i in 0..quad_count_one_side {
            for j in 0..quad_count_one_side {
                let i0 = i * vertices_per_row + j;
                let i1 = i0 + 1;
                let i2 = i0 + vertices_per_row;
                let i3 = i2 + 1;

                indices.push(i0 as u16);
                indices.push(i1 as u16);
                indices.push(i2 as u16);

                indices.push(i2 as u16);
                indices.push(i1 as u16);
                indices.push(i3 as u16);
            }
        }

        Mesh::with_contents(device, &vertices, &indices)
    }

    pub fn get_model_matrix(&self) -> Matrix4<f32> {
        self.transform.to_matrix()
    }
}
