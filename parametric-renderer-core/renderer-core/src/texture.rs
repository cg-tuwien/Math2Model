use glam::UVec2;

use crate::game::TextureInfo;

pub struct Texture {
    pub texture: wgpu::Texture,
    pub view: wgpu::TextureView,
}

impl Texture {
    pub const DEPTH_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Depth32Float;

    pub fn new_rgba(device: &wgpu::Device, queue: &wgpu::Queue, info: &TextureInfo) -> Self {
        let size = wgpu::Extent3d {
            width: info.width,
            height: info.height,
            depth_or_array_layers: 1,
        };
        let desc = wgpu::TextureDescriptor {
            label: None,
            size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8UnormSrgb,
            usage: wgpu::TextureUsages::TEXTURE_BINDING
                | wgpu::TextureUsages::COPY_DST
                | wgpu::TextureUsages::RENDER_ATTACHMENT,
            view_formats: &[],
        };
        let texture = device.create_texture(&desc);

        let copy_texture = wgpu::TexelCopyTextureInfo {
            aspect: wgpu::TextureAspect::All,
            texture: &texture,
            mip_level: 0,
            origin: wgpu::Origin3d::ZERO,
        };

        match &info.data {
            crate::game::TextureData::Bytes(data) => queue.write_texture(
                copy_texture,
                data,
                wgpu::TexelCopyBufferLayout {
                    offset: 0,
                    bytes_per_row: Some(4 * size.width),
                    rows_per_image: Some(size.height),
                },
                size,
            ),
            #[cfg(target_arch = "wasm32")]
            crate::game::TextureData::Image(image_bitmap) => {
                queue.copy_external_image_to_texture(
                    &wgpu::CopyExternalImageSourceInfo {
                        source: wgpu::ExternalImageSource::ImageBitmap(image_bitmap.clone()),
                        origin: wgpu::Origin2d::ZERO,
                        flip_y: false,
                    },
                    copy_texture.to_tagged(wgpu::PredefinedColorSpace::Srgb, true),
                    size,
                );
            }
        }

        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        Self { texture, view }
    }

    pub fn create_depth_texture(device: &wgpu::Device, size: UVec2, label: &str) -> Self {
        let desc = wgpu::TextureDescriptor {
            label: Some(label),
            size: size.to_extent(),
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: Self::DEPTH_FORMAT,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        };
        let texture = device.create_texture(&desc);

        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        Self { texture, view }
    }

    pub fn create_object_id_texture(device: &wgpu::Device, size: UVec2, label: &str) -> Self {
        let desc = wgpu::TextureDescriptor {
            label: Some(label),
            size: size.to_extent(),
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::R32Uint,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        };
        let texture = device.create_texture(&desc);

        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        Self { texture, view }
    }
}

trait ToExtent3d {
    fn to_extent(self) -> wgpu::Extent3d;
}

impl ToExtent3d for UVec2 {
    fn to_extent(self) -> wgpu::Extent3d {
        wgpu::Extent3d {
            width: self.x.max(1),
            height: self.y.max(1),
            depth_or_array_layers: 1,
        }
    }
}
