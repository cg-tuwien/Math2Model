use std::sync::Arc;

use glam::UVec2;
use log::info;
use wgpu_profiler::{GpuProfiler, GpuProfilerSettings};
use winit::window::Window;

use super::WindowOrFallback;

pub struct WgpuContext {
    pub instance: wgpu::Instance,
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,
    pub view_format: wgpu::TextureFormat,
}

impl WgpuContext {
    pub async fn new(window: WindowOrFallback) -> anyhow::Result<(Self, SurfaceOrFallback)> {
        let size = window.size().max(UVec2::ONE);
        let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor {
            backends: wgpu::Backends::PRIMARY,
            ..Default::default()
        });

        let surface = window
            .as_window()
            .map(|window| instance.create_surface(window))
            .transpose()?;

        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::HighPerformance,
                // Setting this is only needed for a fallback adapter. Which we don't want.
                compatible_surface: None,
                force_fallback_adapter: false,
            })
            .await
            .ok_or_else(|| anyhow::anyhow!("No adapter found"))?;
        info!("Adapter: {:?}", adapter.get_info());

        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    required_features: wgpu::Features::default()
                        | (adapter.features() & GpuProfiler::ALL_WGPU_TIMER_FEATURES)
                        | (adapter.features() & wgpu::Features::POLYGON_MODE_LINE),
                    required_limits: wgpu::Limits::default(),
                    label: None,
                    memory_hints: Default::default(),
                },
                None,
            )
            .await
            .unwrap();

        let (view_format, surface_format) = match &surface {
            Some(surface) => {
                let surface_format = surface
                    .get_capabilities(&adapter)
                    .formats
                    .first()
                    .cloned()
                    .ok_or_else(|| anyhow::anyhow!("No valid surface format found"))?;
                let view_format = if surface_format.is_srgb() {
                    surface_format
                } else {
                    surface_format.add_srgb_suffix()
                };
                (view_format, surface_format)
            }
            None => (
                wgpu::TextureFormat::Bgra8UnormSrgb,
                wgpu::TextureFormat::Bgra8UnormSrgb,
            ),
        };

        let surface_or_fallback = match surface {
            Some(surface) => {
                let config = wgpu::SurfaceConfiguration {
                    format: surface_format,
                    view_formats: vec![view_format],
                    present_mode: wgpu::PresentMode::AutoVsync,
                    ..surface
                        .get_default_config(&adapter, size.x, size.y)
                        .ok_or_else(|| anyhow::anyhow!("No default surface config found"))?
                };
                surface.configure(&device, &config);
                SurfaceOrFallback::Surface {
                    surface,
                    config,
                    window: window
                        .as_window()
                        .expect("Expected window if there is a surface"),
                    size,
                }
            }
            None => SurfaceOrFallback::Fallback {
                texture: create_fallback_texture(&device, size, view_format),
                size,
            },
        };

        Ok((
            WgpuContext {
                instance,
                device,
                queue,
                view_format,
            },
            surface_or_fallback,
        ))
    }

    fn create_view(&self, texture: &wgpu::Texture) -> wgpu::TextureView {
        texture.create_view(&wgpu::TextureViewDescriptor {
            format: Some(self.view_format),
            ..Default::default()
        })
    }
}

pub enum SurfaceTexture {
    Surface(wgpu::SurfaceTexture, wgpu::TextureView, Arc<Window>),
    Fallback(wgpu::TextureView),
}

impl SurfaceTexture {
    pub fn texture_view(&self) -> &wgpu::TextureView {
        match self {
            SurfaceTexture::Surface(_, view, _) => &view,
            SurfaceTexture::Fallback(view) => &view,
        }
    }

    pub fn present(self) {
        match self {
            SurfaceTexture::Surface(surface_texture, _, window) => {
                window.pre_present_notify();
                surface_texture.present();
            }
            SurfaceTexture::Fallback(_) => {}
        }
    }
}

pub enum SurfaceOrFallback {
    Surface {
        surface: wgpu::Surface<'static>,
        config: wgpu::SurfaceConfiguration,
        window: Arc<Window>,
        size: UVec2,
    },
    Fallback {
        texture: wgpu::Texture,
        size: UVec2,
    },
}

impl SurfaceOrFallback {
    pub fn size(&self) -> UVec2 {
        match self {
            SurfaceOrFallback::Surface { size, .. } => *size,
            SurfaceOrFallback::Fallback { size, .. } => *size,
        }
    }

    /// Tries to resize the swapchain to the new size.
    /// Returns the actual size of the swapchain if it was resized.
    pub fn try_resize(&mut self, context: &WgpuContext, new_size: UVec2) -> Option<UVec2> {
        let new_size = new_size.max(UVec2::new(1, 1));
        if new_size == self.size() {
            return None;
        }
        match self {
            SurfaceOrFallback::Surface {
                surface,
                config,
                size,
                ..
            } => {
                config.width = new_size.x;
                config.height = new_size.y;
                surface.configure(&context.device, config);
                *size = new_size;
                Some(new_size)
            }
            SurfaceOrFallback::Fallback { texture, size } => {
                *texture = create_fallback_texture(&context.device, new_size, context.view_format);
                *size = new_size;
                Some(new_size)
            }
        }
    }

    pub fn recreate_swapchain(&self, context: &WgpuContext) {
        match self {
            SurfaceOrFallback::Surface {
                surface, config, ..
            } => {
                surface.configure(&context.device, config);
            }
            SurfaceOrFallback::Fallback { .. } => {
                // No-op
            }
        }
    }

    pub fn surface_texture(
        &self,
        context: &WgpuContext,
    ) -> Result<SurfaceTexture, wgpu::SurfaceError> {
        match self {
            SurfaceOrFallback::Surface {
                surface, window, ..
            } => surface.get_current_texture().map(|surface_texture| {
                let view = context.create_view(&surface_texture.texture);
                SurfaceTexture::Surface(surface_texture, view, window.clone())
            }),
            SurfaceOrFallback::Fallback { texture, .. } => {
                Ok(SurfaceTexture::Fallback(context.create_view(&texture)))
            }
        }
    }
}

fn create_fallback_texture(
    device: &wgpu::Device,
    size: UVec2,
    format: wgpu::TextureFormat,
) -> wgpu::Texture {
    device.create_texture(&wgpu::TextureDescriptor {
        label: Some("Fallback surface"),
        size: wgpu::Extent3d {
            width: size.x,
            height: size.y,
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format,
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
        view_formats: &[],
    })
}

pub fn create_profiler(context: &WgpuContext) -> GpuProfiler {
    let gpu_profiler_settings = GpuProfilerSettings {
        enable_timer_queries: true, // Enabled by default
        ..GpuProfilerSettings::default()
    };

    let profiler = GpuProfiler::new(&context.device, gpu_profiler_settings)
        .expect("Failed to create profiler");
    profiler
}
