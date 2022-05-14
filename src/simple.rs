use crate::gfx::GfxContext;
use imgui::{Context, Ui};
use imgui_winit_support::{HiDpiMode, WinitPlatform};
use std::time::Instant;
use winit::{
    event::{Event, StartCause, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::{Window, WindowBuilder},
};

pub struct Framework<T> {
    event_loop: EventLoop<()>,
    window: Window,
    imgui: Context,
    imgui_platform: WinitPlatform,
    gfx: GfxContext,
    last_frame: Instant,
    state: T,
}

impl<T> Framework<T>
where
    T: 'static,
{
    pub fn new(title: &str, state: T) -> Self {
        let event_loop = EventLoop::new();

        let window = WindowBuilder::new()
            .with_title(title)
            .build(&event_loop)
            .unwrap();

        let mut imgui = Context::create();
        imgui.set_ini_filename(None);
        let hidpi_factor = window.scale_factor();
        let font_size = (13.0 * hidpi_factor) as f32;
        imgui.io_mut().font_global_scale = (1.0 / hidpi_factor) as f32;
        imgui
            .fonts()
            .add_font(&[imgui::FontSource::DefaultFontData {
                config: Some(imgui::FontConfig {
                    oversample_h: 1,
                    pixel_snap_h: true,
                    size_pixels: font_size,
                    ..Default::default()
                }),
            }]);

        let mut imgui_platform = WinitPlatform::init(&mut imgui);
        imgui_platform.attach_window(imgui.io_mut(), &window, HiDpiMode::Default);
        let gfx = GfxContext::new(&window, &mut imgui);
        let last_frame = Instant::now();

        Self {
            event_loop,
            window,
            imgui,
            imgui_platform,
            gfx,
            last_frame,
            state,
        }
    }

    pub fn run<F2>(self, mut draw: F2) -> !
    where
        F2: FnMut(&mut Ui, &mut T) + 'static,
    {
        let Self {
            event_loop,
            window,
            mut imgui,
            mut imgui_platform,
            mut gfx,
            mut last_frame,
            mut state,
        } = self;
        event_loop.run(move |event, _, flow| {
            imgui_platform.handle_event(imgui.io_mut(), &window, &event);

            match event {
                Event::NewEvents(StartCause::Init) => *flow = ControlFlow::Poll,
                Event::WindowEvent { event, .. } => match event {
                    WindowEvent::CloseRequested => *flow = ControlFlow::Exit,
                    _ => {}
                },
                Event::RedrawRequested(_) => {
                    let now = Instant::now();
                    let io = imgui.io_mut();
                    io.update_delta_time(now - last_frame);
                    last_frame = now;
                    imgui_platform
                        .prepare_frame(io, &window)
                        .expect("Couldn't prepare imgui frame");

                    let ui = imgui.frame();
                    draw(ui, &mut state);

                    imgui_platform.prepare_render(ui, &window);
                    gfx.redraw(imgui.render(), window.inner_size());
                    gfx.device.poll(wgpu::Maintain::Poll);
                }
                Event::RedrawEventsCleared => window.request_redraw(),
                _ => {}
            }
        });
    }
}
