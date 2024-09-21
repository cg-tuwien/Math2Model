use std::sync::Arc;

use glam::UVec2;
use tracing::info;
use wgpu_profiler::{GpuProfiler, GpuProfilerSettings};
use winit::window::Window;

use super::WindowOrFallback;

pub struct WgpuContext {
    pub instance: wgpu::Instance,
    pub surface: SurfaceOrFallback,
    pub _adapter: wgpu::Adapter,
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,
    pub profiler: GpuProfiler,
    is_profiling_enabled: bool,
    size: UVec2,
    pub view_format: wgpu::TextureFormat,
}

impl WgpuContext {
    pub async fn new(window: WindowOrFallback) -> anyhow::Result<Self> {
        let size = window.size().max(UVec2::ONE);
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
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
                compatible_surface: surface.as_ref(),
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
                }
            }
            None => SurfaceOrFallback::Fallback {
                texture: Self::create_fallback_texture(&device, size, view_format),
            },
        };

        let gpu_profiler_settings = GpuProfilerSettings {
            enable_timer_queries: false, // Disabled by default
            ..GpuProfilerSettings::default()
        };

        #[cfg(feature = "tracy")]
        let profiler = GpuProfiler::new_with_tracy_client(
            gpu_profiler_settings.clone(),
            adapter.get_info().backend,
            &device,
            &queue,
        )
        .unwrap_or_else(|e| match e {
            wgpu_profiler::CreationError::TracyClientNotRunning
            | wgpu_profiler::CreationError::TracyGpuContextCreationError(_) => {
                warn!("Failed to connect to Tracy. Continuing without Tracy integration.");
                GpuProfiler::new(gpu_profiler_settings).expect("Failed to create profiler")
            }
            _ => {
                panic!("Failed to create profiler: {}", e);
            }
        });

        #[cfg(not(feature = "tracy"))]
        let profiler = GpuProfiler::new(gpu_profiler_settings).expect("Failed to create profiler");
        let is_profiling_enabled = false;

        Ok(WgpuContext {
            instance,
            surface: surface_or_fallback,
            _adapter: adapter,
            device,
            queue,
            profiler,
            is_profiling_enabled,
            size,
            view_format,
        })
    }

    pub fn set_profiling(&mut self, enabled: bool) {
        if self.is_profiling_enabled == enabled {
            return;
        }
        self.is_profiling_enabled = enabled;
        self.profiler
            .change_settings(GpuProfilerSettings {
                enable_timer_queries: enabled,
                ..GpuProfilerSettings::default()
            })
            .unwrap();
    }

    pub fn resize(&mut self, new_size: UVec2) {
        let new_size = new_size.max(UVec2::new(1, 1));
        if new_size == self.size {
            return;
        }
        self.size = new_size;
        match &mut self.surface {
            SurfaceOrFallback::Surface {
                surface, config, ..
            } => {
                config.width = new_size.x;
                config.height = new_size.y;
                surface.configure(&self.device, config);
            }
            SurfaceOrFallback::Fallback { texture } => {
                *texture = Self::create_fallback_texture(&self.device, new_size, self.view_format);
            }
        }
    }

    pub fn size(&self) -> UVec2 {
        self.size
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
}

pub enum SurfaceOrFallback {
    Surface {
        surface: wgpu::Surface<'static>,
        config: wgpu::SurfaceConfiguration,
        window: Arc<Window>,
    },
    Fallback {
        texture: wgpu::Texture,
    },
}
