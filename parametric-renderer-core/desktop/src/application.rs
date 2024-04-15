use std::sync::Arc;

use glamour::{Angle, Point3, ToRaw, Vector4};
use tracing::{error, info, warn};
use wgpu::util::DeviceExt;
use winit::window::Window;
use winit_input_helper::WinitInputHelper;

use crate::{
    camera::{camera_settings::CameraSettings, freecam_controller::FreecamController, Camera},
    config::{CacheFile, CachedCamera, ConfigFile},
    mesh::Mesh,
    shaders::shader,
    texture::Texture,
};

pub struct Application {
    surface: wgpu::Surface<'static>,
    device: wgpu::Device,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,
    render_pipeline: wgpu::RenderPipeline,
    depth_texture: Texture,
    camera: Camera,
    camera_buffer: wgpu::Buffer,
    bind_group_0: shader::bind_groups::BindGroup0,
    light_buffer: wgpu::Buffer,
    freecam_controller: FreecamController,
    delta_time: f32,
    mesh: Mesh,
    size: winit::dpi::PhysicalSize<u32>,
    inputs: WinitInputHelper,
    window: Arc<Window>,
    config_file: ConfigFile,
    cache_file: CacheFile,
}

impl Drop for Application {
    fn drop(&mut self) {
        self.cache_file.camera = Some(CachedCamera::FirstPerson {
            position: self.freecam_controller.position.to_array(),
            pitch: self.freecam_controller.pitch.radians,
            yaw: self.freecam_controller.yaw.radians,
        });
        self.cache_file
            .save_to_file(Application::CACHE_FILE)
            .unwrap();
    }
}

impl Application {
    const CONFIG_FILE: &'static str = "config.json";
    const CACHE_FILE: &'static str = "cache.json";
    pub async fn new(window: Arc<Window>) -> anyhow::Result<Self> {
        let config_file = ConfigFile::from_file(Application::CONFIG_FILE).unwrap_or_default();
        let cache_file = CacheFile::from_file(Application::CACHE_FILE).unwrap_or_default();
        let size = window.inner_size();

        let camera = Camera::new(
            size.width as f32 / size.height as f32,
            CameraSettings::default(),
        );
        let mut freecam_controller = FreecamController::new(5.0, 0.01);
        match cache_file.camera {
            Some(CachedCamera::FirstPerson {
                position,
                pitch,
                yaw,
            }) => {
                freecam_controller.position = Point3::from(position);
                freecam_controller.pitch = Angle::from(pitch);
                freecam_controller.yaw = Angle::from(yaw);
            }
            _ => {
                freecam_controller.position = Point3::new(0.0, 0.0, 2.0);
            }
        }

        let inputs = WinitInputHelper::new();

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
                    required_features: wgpu::Features::empty(),
                    required_limits: wgpu::Limits::default(),
                    label: None,
                },
                None,
            )
            .await?;

        let surface_caps = surface.get_capabilities(&adapter);
        let surface_format = surface_caps
            .formats
            .iter()
            .find(|format| format.is_srgb())
            .cloned()
            .ok_or_else(|| anyhow::anyhow!("No sRGB format surface found"))?;

        let config = wgpu::SurfaceConfiguration {
            format: surface_format,
            ..surface
                .get_default_config(&adapter, size.width, size.height)
                .ok_or_else(|| anyhow::anyhow!("No default surface config found"))?
        };
        surface.configure(&device, &config);

        let camera_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Camera Buffer"),
            contents: bytemuck::cast_slice(&[shader::Camera {
                view: camera.view_matrix().to_raw(),
                projection: camera.projection_matrix().to_raw(),
                view_position: camera.position.to_raw().extend(1.0),
            }]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let light_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Light Buffer"),
            contents: bytemuck::cast_slice(&[shader::Light {
                position: Vector4::<f32>::new(0.0, 0.0, 2.0, 0.0).to_raw(),
                color: Vector4::<f32>::new(1.0, 1.0, 1.0, 0.0).to_raw(),
            }]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });
        let bind_group_0 = shader::bind_groups::BindGroup0::from_bindings(
            &device,
            shader::bind_groups::BindGroupLayout0 {
                camera: camera_buffer.as_entire_buffer_binding(),
                light: light_buffer.as_entire_buffer_binding(),
            },
        );

        let mesh = Mesh::new_quad(&device);

        let shader = shader::create_shader_module(&device);

        let render_pipeline_layout = shader::create_pipeline_layout(&device);

        let depth_texture = Texture::create_depth_texture(&device, &config, "Depth Texture");

        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Render Pipeline"),
            layout: Some(&render_pipeline_layout),
            vertex: shader::vertex_state(
                &shader,
                &shader::vs_main_entry(
                    wgpu::VertexStepMode::Vertex,
                    wgpu::VertexStepMode::Instance,
                ),
            ),
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: shader::ENTRY_FS_MAIN,
                targets: &[Some(wgpu::ColorTargetState {
                    format: config.format,
                    blend: Some(wgpu::BlendState::REPLACE),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: None,
                // Setting this to anything other than Fill requires Features::NON_FILL_POLYGON_MODE
                polygon_mode: wgpu::PolygonMode::Fill,
                // Requires Features::DEPTH_CLIP_CONTROL
                unclipped_depth: false,
                // Requires Features::CONSERVATIVE_RASTERIZATION
                conservative: false,
            },
            depth_stencil: Some(wgpu::DepthStencilState {
                format: Texture::DEPTH_FORMAT,
                depth_write_enabled: true,
                depth_compare: wgpu::CompareFunction::Greater,
                stencil: Default::default(),
                bias: Default::default(),
            }),
            multisample: Default::default(),
            multiview: None,
        });

        Ok(Self {
            surface,
            device,
            queue,
            config,
            render_pipeline,
            depth_texture,
            camera,
            camera_buffer,
            light_buffer,
            bind_group_0,
            freecam_controller,
            delta_time: 0.0,
            mesh,
            size,
            inputs,
            window,
            config_file,
            cache_file,
        })
    }

    pub fn window(&self) -> &winit::window::Window {
        &self.window
    }

    fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        if new_size.width > 0 && new_size.height > 0 {
            self.size = new_size;
            self.config.width = new_size.width;
            self.config.height = new_size.height;
            self.surface.configure(&self.device, &self.config);
            self.depth_texture =
                Texture::create_depth_texture(&self.device, &self.config, "Depth Texture");

            self.camera
                .update_aspect_ratio(new_size.width as f32 / new_size.height as f32);
        }
    }

    pub fn update(&mut self) {
        self.freecam_controller
            .update(&self.inputs, self.delta_time);
        self.camera.update_camera(&self.freecam_controller);
        self.queue.write_buffer(
            &self.camera_buffer,
            0,
            bytemuck::cast_slice(&[shader::Camera {
                view: self.camera.view_matrix().to_raw(),
                projection: self.camera.projection_matrix().to_raw(),
                view_position: self.camera.position.to_raw().extend(1.0),
            }]),
        );
    }

    pub fn render(&mut self) -> Result<(), wgpu::SurfaceError> {
        let output = self.surface.get_current_texture()?;
        let view = output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());
        let mut commands = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Render Encoder"),
            });

        {
            let mut render_pass = commands.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: 0.1,
                            g: 0.2,
                            b: 0.3,
                            a: 1.0,
                        }),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                    view: &self.depth_texture.view,
                    depth_ops: Some(wgpu::Operations {
                        load: wgpu::LoadOp::Clear(0.0), // Reverse Z checklist https://iolite-engine.com/blog_posts/reverse_z_cheatsheet
                        store: wgpu::StoreOp::Store,
                    }),
                    stencil_ops: None,
                }),
                timestamp_writes: None,
                occlusion_query_set: None,
            });
            render_pass.set_pipeline(&self.render_pipeline);
            shader::set_bind_groups(&mut render_pass, &self.bind_group_0);
            render_pass.set_vertex_buffer(0, self.mesh.vertex_buffer.slice(..));
            render_pass.set_vertex_buffer(1, self.mesh.instance_buffer.slice(..));
            render_pass
                .set_index_buffer(self.mesh.index_buffer.slice(..), wgpu::IndexFormat::Uint16);
            render_pass.draw_indexed(0..self.mesh.num_indices, 0, 0..1);
        }

        self.queue.submit(std::iter::once(commands.finish()));
        output.present();

        Ok(())
    }
}

pub async fn run() -> anyhow::Result<()> {
    use winit::{
        event_loop::{ControlFlow, EventLoop},
        keyboard::{Key, NamedKey},
        window::WindowBuilder,
    };

    let event_loop = EventLoop::new()?;
    let window = Arc::new(WindowBuilder::new().build(&event_loop)?);
    event_loop.set_control_flow(ControlFlow::Poll);

    let mut application = Application::new(window.clone()).await?;

    event_loop.run(move |event, elwt| {
        let input = &mut application.inputs;
        if input.update(&event) {
            if input.key_released_logical(Key::Named(NamedKey::Escape))
                || input.close_requested()
                || input.destroyed()
            {
                info!("Stopping the application.");
                elwt.exit();
                return;
            }
            application.delta_time = input.delta_time().unwrap_or_default().as_secs_f32();
            if let Some((width, height)) = input.resolution() {
                application.resize(winit::dpi::PhysicalSize { width, height });
            }
            application.update();
            match application.render() {
                Ok(_) => (),
                Err(wgpu::SurfaceError::Lost) => {
                    application.resize(application.size);
                }
                Err(wgpu::SurfaceError::OutOfMemory) => {
                    error!("Out of memory");
                    elwt.exit();
                }
                Err(e) => {
                    warn!("Unexpected error: {:?}", e);
                }
            }
        }
    })?;

    Ok(())
}
