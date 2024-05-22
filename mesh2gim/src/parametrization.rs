use std::collections::HashMap;

use crate::{Image, Mesh};
use glam::{FloatExt, Mat3, UVec2, UVec3, Vec2, Vec3};

pub struct TriangleGroups {
    pub groups: [Vec<UVec3>; 8],
}

/// Parametrizes the received mesh into a sphere and return the new set of vertices
pub fn spherical_parametrization(mesh: &Mesh, iterations: u32) -> Vec<Vec3> {
    let number_of_faces = mesh.faces_count();
    let number_of_vertices = mesh.positions.len();
    // E = [faces([1 2],:) faces([2 3],:) faces([3 1],:)];
    let mut edges = mesh.edges();
    assert!(edges.len() == number_of_faces * 3);

    // E = [E E(2:-1:1,:)]
    let edges_count = edges.len();
    for i in 0..edges_count {
        let current = edges[i];
        edges.push(UVec2::new(current.y, current.x));
    }

    let to_key = |x: u32, y: u32| -> u64 { (y as u64) * (number_of_vertices as u64) + (x as u64) };

    // W = make_sparse( E(1,:), E(2,:), ones(size(E,2),1) );
    let weights: HashMap<u64, u32> = edges
        .into_iter()
        .map(|edge| (to_key(edge.x, edge.y), 1))
        .collect::<HashMap<u64, u32>>();

    // d = full( sum(W,1) );
    let mut duplicates: Vec<u64> = Vec::with_capacity(number_of_vertices);
    for i in 0..number_of_vertices {
        let mut current_sum = 0;
        for j in 0..number_of_vertices {
            let key = to_key(j as u32, i as u32);
            if weights.get(&key) == Some(&1) {
                current_sum += 1;
            }
        }
        duplicates.push(current_sum);
    }

    // tW = iD * W;
    let mut t_weights = Vec::new();
    let mut t_weights_indexes = Vec::new();
    for i in 0..number_of_vertices {
        for j in 0..number_of_vertices {
            let key = to_key(j as u32, i as u32);
            if weights.get(&key) == Some(&1) {
                t_weights_indexes.push(UVec2::new(i as u32, j as u32));
                t_weights.push(1.0f32 / (duplicates[i] as f32));
            }
        }
    }

    /*
        Perform Smoothing and Projection
    */
    let mut parametrized_vertices = mesh.positions.clone();
    // vertex1 = vertex1 - repmat( mean(vertex1,2), [1 n] );
    let mean = parametrized_vertices.iter().sum::<Vec3>() / (number_of_vertices as f32);
    for vertex in &mut parametrized_vertices {
        *vertex -= mean;
    }
    // vertex1 = vertex1 ./ repmat( sqrt(sum(vertex1.^2,1)), [3 1] );
    for vertex in &mut parametrized_vertices {
        *vertex = vertex.normalize();
    }

    println!("Spherical Parametrization Iterations {iterations}");
    let mut result = vec![Vec3::ZERO; number_of_vertices];
    for i in 0..iterations {
        for pos in t_weights_indexes.iter() {
            result[pos.x as usize] += parametrized_vertices[pos.y as usize] * t_weights[i as usize];
        }
        for i in 0..number_of_vertices {
            // @TODO: check this...
            parametrized_vertices[i] = result[i].normalize_or(Vec3::ONE);
        }
        for item in &mut result {
            *item = Vec3::ZERO;
        }
    }

    parametrized_vertices
}

/// This separates the triangles defined in the 'indexes' array in 8 groups, depending on where the triangle is located in the
/// parametrized sphere.
/// To avoid bugs later, when the triangle has vertices in multiple groups, we add it to all groups.
pub fn separate_triangle_groups(mesh: &Mesh, parametrization: &[Vec3]) -> TriangleGroups {
    let mut triangle_groups = {
        const ARRAY_REPEAT_VALUE: Vec<UVec3> = Vec::new();
        [ARRAY_REPEAT_VALUE; 8]
    };

    for triangle in mesh.triangles() {
        let p1 = parametrization[triangle.x as usize];
        let p2 = parametrization[triangle.y as usize];
        let p3 = parametrization[triangle.z as usize];
        let mut found_group = false;

        let px_positive = p1.x >= 0.0 && p2.x >= 0.0 && p3.x >= 0.0;
        let px_negative = p1.x <= 0.0 && p2.x <= 0.0 && p3.x <= 0.0;
        let py_positive = p1.y >= 0.0 && p2.y >= 0.0 && p3.y >= 0.0;
        let py_negative = p1.y <= 0.0 && p2.y <= 0.0 && p3.y <= 0.0;
        let pz_positive = p1.z >= 0.0 && p2.z >= 0.0 && p3.z >= 0.0;
        let pz_negative = p1.z <= 0.0 && p2.z <= 0.0 && p3.z <= 0.0;

        if px_positive {
            if py_positive {
                if pz_positive {
                    triangle_groups[0].push(triangle);
                    found_group = true;
                }
                if pz_negative {
                    triangle_groups[1].push(triangle);
                    found_group = true;
                }
            }
            if py_negative {
                if pz_positive {
                    triangle_groups[2].push(triangle);
                    found_group = true;
                }
                if pz_negative {
                    triangle_groups[3].push(triangle);
                    found_group = true;
                }
            }
        }
        if px_negative {
            if py_positive {
                if pz_positive {
                    triangle_groups[4].push(triangle);
                    found_group = true;
                }
                if pz_negative {
                    triangle_groups[5].push(triangle);
                    found_group = true;
                }
            }
            if py_negative {
                if pz_positive {
                    triangle_groups[6].push(triangle);
                    found_group = true;
                }
                if pz_negative {
                    triangle_groups[7].push(triangle);
                    found_group = true;
                }
            }
        }
        if !found_group {
            triangle_groups[0].push(triangle);
            triangle_groups[1].push(triangle);
            triangle_groups[2].push(triangle);
            triangle_groups[3].push(triangle);
            triangle_groups[4].push(triangle);
            triangle_groups[5].push(triangle);
            triangle_groups[6].push(triangle);
            triangle_groups[7].push(triangle);
        }
    }
    TriangleGroups {
        groups: triangle_groups,
    }
}

pub fn to_image(
    mesh: &Mesh,
    parametrization: &[Vec3],
    groups: TriangleGroups,
    size: (u32, u32),
) -> Image {
    // must be greater than 1 and odd
    assert!(size.0 > 1 && size.1 > 1 && size.0 % 2 == 1 && size.1 % 2 == 1);
    let mut gim_data = vec![Vec3::ZERO; (size.0 * size.1) as usize];

    let mut pixel_color = Vec3::ZERO;

    let mut error_positions = Vec::new();
    for y in 0..size.1 {
        for x in 0..size.0 {
            let y_normalized = scale_to_range(x as f32, 0.0, size.0 as f32 - 1.0, -1.0, 1.0);
            let x_normalized = scale_to_range(y as f32, 0.0, size.1 as f32 - 1.0, -1.0, 1.0);
            let (point_in_space, selected_triangle_group) =
                select_triangle_group(Vec2::new(x_normalized, y_normalized), &groups);

            pixel_color = match get_gim_pixel_by_sampling_mesh(
                &mesh.positions,
                parametrization,
                selected_triangle_group,
                point_in_space,
            ) {
                Some(color) => color,
                None => {
                    error_positions.push((x, y));
                    // @TODO: What to do when we have a hole in the mesh?
                    // For now, let's keep the same color as the previous pixel... this way we may visually 'hide' the holes
                    pixel_color
                }
            };

            // Here, if we are dealing with a border pixel, we manually copy it to all its matches.
            // Theoretically, the algorithm above already takes care of it, but we need to recopy here to avoid
            // problems because of floating-point precision (xNormalized and yNormalized varies a bit and it causes
            // inconsistency in the sampling)
            let gim_size = size.0;
            assert!(gim_size == size.1, "GIM size must be a square");
            if x == 0 || x == size.0 - 1 {
                gim_data[((gim_size - y - 1) * gim_size + x) as usize] = pixel_color;
            }
            if y == 0 || y == size.1 - 1 {
                gim_data[(y * gim_size + (gim_size - x - 1)) as usize] = pixel_color;
            }
            if (x == 0 && y == 0)
                || (x == size.0 - 1 && y == size.1 - 1)
                || (x == 0 && y == size.1 - 1)
                || (x == size.0 - 1 && y == 0)
            {
                gim_data[((gim_size - y - 1) * gim_size + (gim_size - x - 1)) as usize] =
                    pixel_color;
            }

            gim_data[(y * gim_size + x) as usize] = pixel_color;
        }
    }

    if !error_positions.is_empty() {
        eprintln!(
            "There are {} error positions in the GIM where we couldn't sample the mesh.",
            error_positions.len()
        );
    }

    Image {
        width: size.0,
        height: size.1,
        pixels: gim_data,
    }
}

fn select_triangle_group(point: Vec2, groups: &TriangleGroups) -> (Vec3, &Vec<UVec3>) {
    if point.y >= 0.0 && point.x >= 0.0 {
        if point.x + point.y >= 1.0 {
            // TOP-RIGHT BLUE (1)
            let barycentric = convert_to_barycentric_2d(
                Vec2::new(0.0, 1.0),
                Vec2::new(1.0, 0.0),
                Vec2::new(1.0, 1.0),
                point,
            );
            (
                barycentric_to_euler(
                    Vec3::new(0.0, 1.0, 0.0),
                    Vec3::new(1.0, 0.0, 0.0),
                    Vec3::new(0.0, 0.0, -1.0),
                    barycentric,
                ),
                &groups.groups[1],
            )
        } else {
            // TOP-RIGHT RED (0)
            let barycentric = convert_to_barycentric_2d(
                Vec2::new(1.0, 0.0),
                Vec2::new(0.0, 1.0),
                Vec2::new(0.0, 0.0),
                point,
            );
            (
                barycentric_to_euler(
                    Vec3::new(1.0, 0.0, 0.0),
                    Vec3::new(0.0, 1.0, 0.0),
                    Vec3::new(0.0, 0.0, 1.0),
                    barycentric,
                ),
                &groups.groups[0],
            )
        }
    } else if point.y >= 0.0 && point.x <= 0.0 {
        if point.y >= point.x + 1.0 {
            // TOP-LEFT ORANGE (5)
            let barycentric = convert_to_barycentric_2d(
                Vec2::new(0.0, 1.0),
                Vec2::new(-1.0, 0.0),
                Vec2::new(-1.0, 1.0),
                point,
            );
            (
                barycentric_to_euler(
                    Vec3::new(0.0, 1.0, 0.0),
                    Vec3::new(-1.0, 0.0, 0.0),
                    Vec3::new(0.0, 0.0, -1.0),
                    barycentric,
                ),
                &groups.groups[5],
            )
        } else {
            // TOP-LEFT GREEN (4)
            let barycentric = convert_to_barycentric_2d(
                Vec2::new(-1.0, 0.0),
                Vec2::new(0.0, 1.0),
                Vec2::new(0.0, 0.0),
                point,
            );
            (
                barycentric_to_euler(
                    Vec3::new(-1.0, 0.0, 0.0),
                    Vec3::new(0.0, 1.0, 0.0),
                    Vec3::new(0.0, 0.0, 1.0),
                    barycentric,
                ),
                &groups.groups[4],
            )
        }
    } else if point.y <= 0.0 && point.x >= 0.0 {
        if point.y + 1.0 >= point.x {
            // BOTTOM-RIGHT ORANGE (2)
            let barycentric = convert_to_barycentric_2d(
                Vec2::new(1.0, 0.0),
                Vec2::new(0.0, -1.0),
                Vec2::new(0.0, 0.0),
                point,
            );
            (
                barycentric_to_euler(
                    Vec3::new(1.0, 0.0, 0.0),
                    Vec3::new(0.0, -1.0, 0.0),
                    Vec3::new(0.0, 0.0, 1.0),
                    barycentric,
                ),
                &groups.groups[2],
            )
        } else {
            // BOTTOM-RIGHT GREEN (3)
            let barycentric = convert_to_barycentric_2d(
                Vec2::new(0.0, -1.0),
                Vec2::new(1.0, 0.0),
                Vec2::new(1.0, -1.0),
                point,
            );
            (
                barycentric_to_euler(
                    Vec3::new(0.0, -1.0, 0.0),
                    Vec3::new(1.0, 0.0, 0.0),
                    Vec3::new(0.0, 0.0, -1.0),
                    barycentric,
                ),
                &groups.groups[3],
            )
        }
    } else {
        if point.x + point.y >= -1.0 {
            // BOTTOM-LEFT BLUE (6)
            let barycentric = convert_to_barycentric_2d(
                Vec2::new(-1.0, 0.0),
                Vec2::new(0.0, -1.0),
                Vec2::new(0.0, 0.0),
                point,
            );
            (
                barycentric_to_euler(
                    Vec3::new(-1.0, 0.0, 0.0),
                    Vec3::new(0.0, -1.0, 0.0),
                    Vec3::new(0.0, 0.0, 1.0),
                    barycentric,
                ),
                &groups.groups[6],
            )
        } else {
            // BOTTOM-LEFT RED (7)
            let barycentric = convert_to_barycentric_2d(
                Vec2::new(0.0, -1.0),
                Vec2::new(-1.0, 0.0),
                Vec2::new(-1.0, -1.0),
                point,
            );
            (
                barycentric_to_euler(
                    Vec3::new(0.0, -1.0, 0.0),
                    Vec3::new(-1.0, 0.0, 0.0),
                    Vec3::new(0.0, 0.0, -1.0),
                    barycentric,
                ),
                &groups.groups[7],
            )
        }
    }
}

fn convert_to_barycentric_3d(a: Vec3, b: Vec3, c: Vec3, p: Vec3) -> Vec3 {
    let v0 = b - a;
    let v1 = c - a;
    let v2 = p - a;
    let d00 = v0.dot(v0);
    let d01 = v0.dot(v1);
    let d11 = v1.dot(v1);
    let d20 = v2.dot(v0);
    let d21 = v2.dot(v1);
    let denom = d00 * d11 - d01 * d01;

    let mut result = Vec3::ZERO;
    result.y = (d11 * d20 - d01 * d21) / denom;
    result.z = (d00 * d21 - d01 * d20) / denom;
    result.x = 1.0 - result.y - result.z;
    result
}

fn convert_to_barycentric_2d(a: Vec2, b: Vec2, c: Vec2, p: Vec2) -> Vec3 {
    let v0 = b - a;
    let v1 = c - a;
    let v2 = p - a;
    let d00 = v0.dot(v0);
    let d01 = v0.dot(v1);
    let d11 = v1.dot(v1);
    let d20 = v2.dot(v0);
    let d21 = v2.dot(v1);
    let denom = d00 * d11 - d01 * d01;

    let mut result = Vec3::ZERO;
    result.y = (d11 * d20 - d01 * d21) / denom;
    result.z = (d00 * d21 - d01 * d20) / denom;
    result.x = 1.0 - result.y - result.z;
    result
}

fn barycentric_to_euler(a: Vec3, b: Vec3, c: Vec3, barycentric: Vec3) -> Vec3 {
    Mat3::from_cols(a, b, c) * barycentric
}

fn scale_to_range(value: f32, in_max: f32, in_min: f32, out_min: f32, out_max: f32) -> f32 {
    value.remap(in_min, in_max, out_min, out_max)
}

fn intersection_ray_triangle(
    ray_origin: Vec3,
    ray_vector: Vec3,
    triangle: (Vec3, Vec3, Vec3),
) -> Option<Vec3> {
    const EPSILON: f32 = 0.0000001;
    let (vertex0, vertex1, vertex2) = triangle;
    let edge1 = vertex1 - vertex0;
    let edge2 = vertex2 - vertex0;
    let h = ray_vector.cross(edge2);
    let a = edge1.dot(h);

    if a > -EPSILON && a < EPSILON {
        return None;
    }

    let f = 1.0 / a;
    let s = ray_origin - vertex0;
    let u = f * s.dot(h);

    if u < 0.0 || u > 1.0 {
        return None;
    }

    let q = s.cross(edge1);
    let v = f * ray_vector.dot(q);

    if v < 0.0 || u + v > 1.0 {
        return None;
    }

    // At this stage we can compute t to find out where the intersection point is on the line.
    let t = f * edge2.dot(q);

    if t > EPSILON {
        Some(ray_origin + ray_vector * t)
    } else {
        None
    }
}

fn get_gim_pixel_by_sampling_mesh(
    original_vertices: &[Vec3],
    parametrized_vertices: &[Vec3],
    indices: &Vec<UVec3>,
    point_in_space: Vec3,
) -> Option<Vec3> {
    let ray_vector = match point_in_space.try_normalize() {
        Some(v) => v,
        None => {
            eprintln!("Point in space is the zero vector");
            Vec3::ZERO
        }
    };
    for triangle in indices {
        let v1 = parametrized_vertices[triangle.x as usize];
        let v2 = parametrized_vertices[triangle.y as usize];
        let v3 = parametrized_vertices[triangle.z as usize];

        if let Some(intersection_point) =
            intersection_ray_triangle(Vec3::ZERO, ray_vector, (v1, v2, v3))
        {
            let barycentric_coordinates = convert_to_barycentric_3d(v1, v2, v3, intersection_point);
            let original_vertices = Mat3::from_cols(
                original_vertices[triangle.x as usize],
                original_vertices[triangle.y as usize],
                original_vertices[triangle.z as usize],
            );
            // Scale each vertex by its barycentric coordinate and sum them
            return Some(original_vertices * barycentric_coordinates);
        }
    }
    None
}
