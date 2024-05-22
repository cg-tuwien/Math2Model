use clap::Parser;
use glam::Vec2;
use image::{DynamicImage, ImageBuffer};
use mesh2gim::{make_geometry_image, Attribute, AttributeValues, Mesh, AABB};
use miniserde::{json, Deserialize, Serialize};
use std::{fs::File, io::BufReader};

#[derive(Parser)]
#[command(version, about)]
struct Cli {
    /// Specify the path of the geometry image that will be generated
    #[clap(short, long, default_value = "./export.png")]
    output: String,

    /// Size of geometry image (<n> x <n>) [must be an odd number]
    #[clap(short, long, default_value_t = 255)]
    size: u32,

    /// The input .obj file
    input: String,
}

#[derive(Serialize, Deserialize)]
struct SerializeVec3 {
    x: f32,
    y: f32,
    z: f32,
}

#[derive(Serialize, Deserialize)]
struct SerializeAABB {
    min: SerializeVec3,
    max: SerializeVec3,
}

impl From<AABB> for SerializeAABB {
    fn from(aabb: AABB) -> Self {
        SerializeAABB {
            min: SerializeVec3 {
                x: aabb.min.x,
                y: aabb.min.y,
                z: aabb.min.z,
            },
            max: SerializeVec3 {
                x: aabb.max.x,
                y: aabb.max.y,
                z: aabb.max.z,
            },
        }
    }
}

fn main() {
    let args = Cli::parse();
    println!("Output: {}", args.output);
    println!("Size: {}", args.size);
    println!("Input: {}", args.input);

    let input = BufReader::new(File::open(&args.input).unwrap());

    let mesh = into_mesh(obj::raw::parse_obj(input).unwrap());
    let bounds = mesh.get_bounds();
    let geometry_image = make_geometry_image(&mesh, (args.size, args.size));

    let image = ImageBuffer::from_fn(geometry_image.width, geometry_image.height, |x, y| {
        let pixel = geometry_image.pixels[(y * geometry_image.width + x) as usize];
        let scaled_pixel = (pixel - bounds.min) / (bounds.max - bounds.min);
        let x: u16 = (scaled_pixel.x * u16::MAX as f32) as u16;
        let y: u16 = (scaled_pixel.y * u16::MAX as f32) as u16;
        let z: u16 = (scaled_pixel.z * u16::MAX as f32) as u16;
        image::Rgb([x, y, z])
    });
    let image = DynamicImage::ImageRgb16(image);
    image.save(&args.output).unwrap();

    let metadata_path = args.output.replace(".png", ".json");
    let ser_bounds = SerializeAABB::from(bounds);
    let metadata = json::to_string(&ser_bounds);
    std::fs::write(metadata_path, metadata).unwrap();
}

fn into_mesh(raw: obj::raw::RawObj) -> Mesh {
    if raw.normals.is_empty() {
        let obj: obj::Obj<obj::Position, u32> = obj::Obj::new(raw).unwrap();
        let positions = obj.vertices.iter().map(|v| v.position.into()).collect();
        Mesh {
            positions,
            attributes: vec![],
            indices: obj.indices,
        }
    } else if raw.tex_coords.is_empty() {
        // Normals but no texture coordinates
        let obj: obj::Obj<obj::Vertex, u32> = obj::Obj::new(raw).unwrap();
        let positions = obj.vertices.iter().map(|v| v.position.into()).collect();
        let normals = obj.vertices.iter().map(|v| v.normal.into()).collect();
        Mesh {
            positions,
            attributes: vec![Attribute {
                name: "normal".to_string(),
                values: AttributeValues::Vec3s(normals),
            }],
            indices: obj.indices,
        }
    } else {
        // Normals and texture coordinates
        let obj: obj::Obj<obj::TexturedVertex, u32> = obj::Obj::new(raw).unwrap();
        let positions = obj.vertices.iter().map(|v| v.position.into()).collect();
        let normals = obj.vertices.iter().map(|v| v.normal.into()).collect();
        let tex_coords = obj
            .vertices
            .iter()
            .map(|v| Vec2::new(v.texture[0], v.texture[1]))
            .collect();
        Mesh {
            positions,
            attributes: vec![
                Attribute {
                    name: "tex_coords".to_string(),
                    values: AttributeValues::Vec2s(tex_coords),
                },
                Attribute {
                    name: "normals".to_string(),
                    values: AttributeValues::Vec3s(normals),
                },
            ],
            indices: obj.indices,
        }
    }
}
