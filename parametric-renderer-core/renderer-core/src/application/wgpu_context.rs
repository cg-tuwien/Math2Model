use std::sync::Arc;

use tracing::info;
use wgpu_profiler::{GpuProfiler, GpuProfilerSettings};
use winit::window::Window;

use super::ProfilerSettings;

pub struct WgpuContext {
    pub instance: wgpu::Instance,
    pub surface: wgpu::Surface<'static>,
    pub adapter: wgpu::Adapter,
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,
    pub config: wgpu::SurfaceConfiguration,
    pub profiler: GpuProfiler,
    pub window: Arc<Window>,
    pub size: winit::dpi::PhysicalSize<u32>,
    pub view_format: wgpu::TextureFormat,
}

impl WgpuContext {
    pub async fn new(window: Window, profiler_settings: &ProfilerSettings) -> anyhow::Result<Self> {
        let window = Arc::new(window);
        let size = window.inner_size().min(winit::dpi::PhysicalSize::new(1, 1));
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: wgpu::Backends::PRIMARY,
            ..Default::default()
        });

        let surface = instance.create_surface(window.clone())?;

        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::HighPerformance,
                compatible_surface: Some(&surface),
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
                },
                None,
            )
            .await
            .unwrap();

        let surface_caps = surface.get_capabilities(&adapter);
        let surface_format = surface_caps
            .formats
            .get(0)
            .cloned()
            .ok_or_else(|| anyhow::anyhow!("No sRGB format surface found"))?;
        let view_format = if surface_format.is_srgb() {
            surface_format
        } else {
            surface_format.add_srgb_suffix()
        };

        let config = wgpu::SurfaceConfiguration {
            format: surface_format,
            view_formats: vec![view_format],
            ..surface
                .get_default_config(&adapter, size.width, size.height)
                .ok_or_else(|| anyhow::anyhow!("No default surface config found"))?
        };
        surface.configure(&device, &config);

        let gpu_profiler_settings = GpuProfilerSettings {
            enable_timer_queries: profiler_settings.gpu,
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

        window.request_redraw();

        Ok(WgpuContext {
            instance,
            surface,
            adapter,
            device,
            queue,
            config,
            profiler,
            window,
            size,
            view_format,
        })
    }
}
