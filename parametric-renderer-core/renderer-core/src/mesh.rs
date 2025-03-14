use glam::{Vec2, Vec3};
use wgpu::util::DeviceExt;

use crate::shaders::shader;

pub struct Mesh {
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
            vertex_buffer,
            index_buffer,
            num_indices: index_buffer_contents.len() as u32,
        }
    }

    /// Create a new tesselated quad mesh
    pub fn new_tesselated_quad(device: &wgpu::Device, split_count: u32) -> Self {
        let (vertices, indices) = tesselated_quad(split_count);

        Mesh::with_contents(device, &vertices, &indices)
    }

    pub fn cubemap_cube(device: &wgpu::Device, min: Vec3, max: Vec3) -> Self {
        let (vertices, indices) = cubemap_cube(min, max);

        Mesh::with_contents(device, &vertices, &indices)
    }
}

fn tesselated_quad(split_count: u32) -> (Vec<shader::VertexInput>, Vec<u16>) {
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
    (vertices, indices)
}

fn cubemap_cube(min: Vec3, max: Vec3) -> (Vec<shader::VertexInput>, Vec<u16>) {
    let vertices = [
        (-1, 1, -1),
        (-1, -1, -1),
        (1, -1, -1),
        (1, -1, -1),
        (1, 1, -1),
        (-1, 1, -1),
        //
        (-1, -1, 1),
        (-1, -1, -1),
        (-1, 1, -1),
        (-1, 1, -1),
        (-1, 1, 1),
        (-1, -1, 1),
        //
        (1, -1, -1),
        (1, -1, 1),
        (1, 1, 1),
        (1, 1, 1),
        (1, 1, -1),
        (1, -1, -1),
        //
        (-1, -1, 1),
        (-1, 1, 1),
        (1, 1, 1),
        (1, 1, 1),
        (1, -1, 1),
        (-1, -1, 1),
        //
        (-1, 1, -1),
        (1, 1, -1),
        (1, 1, 1),
        (1, 1, 1),
        (-1, 1, 1),
        (-1, 1, -1),
        //
        (-1, -1, -1),
        (-1, -1, 1),
        (1, -1, -1),
        (1, -1, -1),
        (-1, -1, 1),
        (1, -1, 1),
    ]
    .into_iter()
    .map(|(x, y, z)| shader::VertexInput {
        position: Vec3::new(
            if x < 0 { min.x } else { max.x },
            if y < 0 { min.y } else { max.y },
            if z < 0 { min.z } else { max.z },
        ),
        uv: Vec2::ZERO,
    })
    .collect::<Vec<_>>();
    let indices = (0u16..(vertices.len() as u16)).collect();
    (vertices, indices)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unit_quad() {
        let (vertices, indices) = tesselated_quad(0);

        assert_eq!(vertices.len(), 4);
        assert_eq!(indices.len(), 6);
        assert_eq!(indices, vec![0, 1, 2, 2, 1, 3]);
        assert_eq!(vertices[0].position, Vec3::new(0.0, 0.0, 0.0));
        assert_eq!(vertices[1].position, Vec3::new(0.0, 1.0, 0.0));
        assert_eq!(vertices[2].position, Vec3::new(1.0, 0.0, 0.0));
        assert_eq!(vertices[3].position, Vec3::new(1.0, 1.0, 0.0));
    }
}
