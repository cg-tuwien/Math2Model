use glam::Vec3;

pub const QUAD_VERTICES: [Vec3; 4] = [
    Vec3::new(-0.5, -0.5, 0.0),
    Vec3::new(0.5, -0.5, 0.0),
    Vec3::new(0.5, 0.5, 0.0),
    Vec3::new(-0.5, 0.5, 0.0),
];
pub const QUAD_INDICES: [u16; 6] = [0, 1, 2, 2, 3, 0];
