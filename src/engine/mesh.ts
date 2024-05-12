import { mat4, quat, vec3, type Mat4, type Quat, type Vec3 } from "wgpu-matrix";

/*
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

        Self {
            transform,
            vertex_buffer,
            index_buffer,
            num_indices: QUAD_INDICES.len() as u32,
        }
    }

    pub fn get_model_matrix(&self) -> Matrix4<f32> {
        self.transform.to_matrix()
    }
}
*/

const QUAD_VERTICES: Vec3[] = [
  vec3.create(-0.5, -0.5, 0.0),
  vec3.create(-0.5, -0.5, 0.0),
  vec3.create(0.5, -0.5, 0.0),
  vec3.create(0.5, 0.5, 0.0),
];
const QUAD_INDICES: Uint16Array = new Uint16Array([0, 1, 2, 2, 3, 0]);

export class Transform {
  constructor(
    public position: Vec3 = vec3.create(),
    public rotation: Quat = quat.create(),
    public scale: number = 1.0
  ) {}

  toMatrix(): Mat4 {
    return mat4.mul(
      mat4.scaling(vec3.create(this.scale, this.scale, this.scale)),
      mat4.mul(mat4.fromQuat(this.rotation), mat4.translation(this.position))
    );
  }
}
