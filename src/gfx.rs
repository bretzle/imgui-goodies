use crate::renderer::RendererImpl;
use gemstone::async_block::block_on;
use imgui::{Context, DrawData};
use std::iter;
use wgpu::{DeviceDescriptor, Features, Limits, PowerPreference, RequestAdapterOptions};
use winit::{dpi::PhysicalSize, window::Window as WinitWindow};

pub struct GfxContext {
    pub instance: wgpu::Instance,
    pub surface: wgpu::Surface,
    pub adapter: wgpu::Adapter,
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,
    pub sc_needs_rebuild: bool,
    pub surf_config: wgpu::SurfaceConfiguration,
    pub renderer: RendererImpl,
}

impl GfxContext {
    pub fn new(window: &WinitWindow, imgui: &mut Context) -> Self {
        block_on(async {
            let instance = wgpu::Instance::new(wgpu::Backends::all());
            let surface = unsafe { instance.create_surface(window) };
            let adapter = instance
                .request_adapter(&RequestAdapterOptions {
                    power_preference: PowerPreference::HighPerformance,
                    force_fallback_adapter: false,
                    compatible_surface: Some(&surface),
                })
                .await
                .expect("Couldn't create graphics adapter");

            let (device, queue) = adapter
                .request_device(
                    &DeviceDescriptor {
                        label: None,
                        features: Features::empty(),
                        limits: Limits::default(),
                    },
                    None,
                )
                .await
                .expect("Couldn't open connection to graphics device");
            let size = window.inner_size();
            let surf_config = wgpu::SurfaceConfiguration {
                usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
                format: surface
                    .get_preferred_format(&adapter)
                    .expect("Couldn't get surface preferred format"),
                width: size.width,
                height: size.height,
                present_mode: wgpu::PresentMode::Fifo,
            };
            surface.configure(&device, &surf_config);
            let renderer = RendererImpl::new(&device, &queue, imgui, surf_config.format);

            Self {
                instance,
                surface,
                adapter,
                device,
                queue,
                sc_needs_rebuild: false,
                surf_config,
                renderer,
            }
        })
    }

    pub fn redraw(&mut self, draw_data: &DrawData, size: PhysicalSize<u32>) {
        if self.sc_needs_rebuild {
            self.rebuild_swapchain(size);
        }
        let frame = loop {
            match self.surface.get_current_texture() {
                Ok(frame) => {
                    if frame.suboptimal {
                        self.update_format_and_rebuild_swapchain(size);
                    } else {
                        break frame;
                    }
                }
                Err(error) => match error {
                    wgpu::SurfaceError::Timeout | wgpu::SurfaceError::Outdated => {}
                    wgpu::SurfaceError::Lost => {
                        self.update_format_and_rebuild_swapchain(size);
                    }
                    wgpu::SurfaceError::OutOfMemory => panic!("Swapchain ran out of memory"),
                },
            }
        };
        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });
        self.renderer.render(
            &self.device,
            &self.queue,
            &mut encoder,
            &frame
                .texture
                .create_view(&wgpu::TextureViewDescriptor::default()),
            draw_data,
        );
        self.queue.submit(iter::once(encoder.finish()));
        frame.present();
    }

    fn update_format_and_rebuild_swapchain(&mut self, size: PhysicalSize<u32>) {
        self.surf_config.format = self
            .surface
            .get_preferred_format(&self.adapter)
            .expect("Couldn't get surface preferred format");
        self.rebuild_swapchain(size);
        self.renderer
            .change_swapchain_format(&self.device, self.surf_config.format);
    }

    fn rebuild_swapchain(&mut self, size: PhysicalSize<u32>) {
        self.sc_needs_rebuild = false;
        self.surf_config.width = size.width;
        self.surf_config.height = size.height;
        if size.width != 0 && size.height != 0 {
            self.surface.configure(&self.device, &self.surf_config);
        }
    }
}
