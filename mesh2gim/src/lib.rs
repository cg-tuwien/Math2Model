mod parametrization;
use glam::{UVec2, UVec3, Vec2, Vec3, Vec4};

pub struct Attribute {
    pub name: String,
    pub values: AttributeValues,
}

pub enum AttributeValues {
    Floats(Vec<f32>),
    Vec2s(Vec<Vec2>),
    Vec3s(Vec<Vec3>),
    Vec4s(Vec<Vec4>),
}

pub struct AABB {
    pub min: Vec3,
    pub max: Vec3,
}

pub struct Mesh {
    pub positions: Vec<Vec3>,
    pub attributes: Vec<Attribute>,
    pub indices: Vec<u32>,
}

impl Mesh {
    pub fn faces_count(&self) -> usize {
        self.indices.len() / 3
    }
    pub fn triangles(&self) -> impl Iterator<Item = UVec3> + '_ {
        self.indices
            .chunks_exact(3)
            .map(|chunk| UVec3::new(chunk[0], chunk[1], chunk[2]))
    }
    pub fn edges(&self) -> Vec<UVec2> {
        self.triangles()
            .flat_map(|triangle| {
                [
                    UVec2::new(triangle.x, triangle.y),
                    UVec2::new(triangle.y, triangle.z),
                    UVec2::new(triangle.z, triangle.x),
                ]
            })
            .collect()
    }
    pub fn get_bounds(&self) -> AABB {
        let mut min = Vec3::splat(f32::INFINITY);
        let mut max = Vec3::splat(f32::NEG_INFINITY);
        for position in &self.positions {
            min = min.min(*position);
            max = max.max(*position);
        }
        AABB { min, max }
    }
}

/// A floating point image with pixels in the range [0, 1]
pub struct Image {
    pub width: u32,
    pub height: u32,
    pub pixels: Vec<Vec3>,
}

pub fn make_geometry_image(mesh: &Mesh, size: (u32, u32)) -> Image {
    let parametrization = parametrization::spherical_parametrization(mesh, 500);
    let groups = parametrization::separate_triangle_groups(mesh, &parametrization);
    parametrization::to_image(mesh, &parametrization, groups, size)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_make_geometry_image() {
        let mesh = Mesh {
            positions: vec![
                Vec3::new(0.0, 0.0, 0.0),
                Vec3::new(1.0, 0.0, 0.0),
                Vec3::new(0.0, 1.0, 0.0),
            ],
            attributes: vec![],
            indices: vec![0, 1, 2],
        };
        let size = (256, 256);
        let image = make_geometry_image(&mesh, size);
        assert_eq!(image.width, size.0);
        assert_eq!(image.height, size.1);
        assert_eq!(image.pixels.len(), (size.0 * size.1 * 4) as usize);
    }
}
