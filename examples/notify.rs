use imgui::{Condition, Context, TreeNodeFlags, Ui};
use imgui_winit_support::{HiDpiMode, WinitPlatform};
use iolite::{
    gfx::GfxContext,
    notify::{Notifications, Toast, ToastType},
};
use std::time::Instant;
use winit::{
    event::{Event, StartCause, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let event_loop = EventLoop::new();

    let window = WindowBuilder::new().build(&event_loop)?;

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
    let mut gfx = GfxContext::new(&window, &mut imgui);
    let mut last_frame = Instant::now();
    let mut notifications = Notifications::new();

    let mut title = "A wonderful quote!".to_string();
    let mut content = String::new();
    let mut duration = 5000;
    let mut typ = ToastType::Success;
    let mut enable_title = true;
    let mut enable_content = true;

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
                draw(
                    ui,
                    &mut notifications,
                    &mut title,
                    &mut content,
                    &mut duration,
                    &mut typ,
                    &mut enable_title,
                    &mut enable_content,
                );

                imgui_platform.prepare_render(ui, &window);
                gfx.redraw(imgui.render(), window.inner_size());
                gfx.device.poll(wgpu::Maintain::Poll);
            }
            Event::RedrawEventsCleared => window.request_redraw(),
            _ => {}
        }
    });
}

fn draw(
    ui: &mut Ui,
    notifications: &mut Notifications,
    title: &mut String,
    content: &mut String,
    duration: &mut i32,
    typ: &mut ToastType,
    enable_title: &mut bool,
    enable_content: &mut bool,
) {
    ui.window("Hello World!")
        .size([550.0, 550.0], Condition::FirstUseEver)
        .position([50.0, 50.0], Condition::FirstUseEver)
        .build(|| {
            if ui.collapsing_header("Examples without title", TreeNodeFlags::DEFAULT_OPEN) {
                if ui.button("Success") {
                    notifications.push(Toast::new(ToastType::Success, 3000).content("Hello World! This is a success!"));
                }
                ui.same_line();
                if ui.button("Warning") {
					notifications.push(Toast::new(ToastType::Warning, 3000).content("Hello World! This is a warning!"));
				}
                ui.same_line();
                if ui.button("Error") {
					notifications.push(Toast::new(ToastType::Error, 3000).content("Hello World! This is a error!"));
				}
                ui.same_line();
                if ui.button("Info") {
					notifications.push(Toast::new(ToastType::Info, 3000).content("Hello World! This is a info!"));
				}
                ui.same_line();
                if ui.button("Info long") {
					notifications.push(Toast::new(ToastType::Info, 3000).content("Lorem ipsum dolor sit amet, consectetur adipiscing elit, sed do eiusmod tempor incididunt ut labore et dolore magna aliqua. Ut enim ad minim veniam, quis nostrud exercitation"));
				}
            }

            if ui.collapsing_header("Do it yourself", TreeNodeFlags::DEFAULT_OPEN) {
                ui.input_text_multiline("Title", title, [0.0, 0.0]).build();
                ui.input_text_multiline("Content", content, [0.0, 0.0])
                    .build();
                ui.input_int("Duration (ms)", duration).step(100).build();

                // dont allow negative
                if *duration < 0 {
                    *duration = 0;
                }

                let typ_str = ["None", "Success", "Warning", "Error", "Info"];

                ui.combo_simple_string(
                    "Type",
                    unsafe { &mut *(typ as *mut _ as *mut usize) },
                    &typ_str,
                );

                ui.checkbox("Enable title", enable_title);
                ui.same_line();
                ui.checkbox("Enable content", enable_content);

                if ui.button("Show") {
                    let mut toast = Toast::new(*typ, *duration as usize);

                    if *enable_title {
                        toast = toast.title(title.clone());
                    }

                    if *enable_content {
                        toast = toast.content(content.clone());
                    }

                    notifications.push(toast);
                }
            }
        });

    notifications.render(ui);
}
